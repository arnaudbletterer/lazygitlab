use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_app() -> (
    MockServer,
    App,
    tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"})),
        )
        .mount(&server)
        .await;
    let client = GitlabBuilder::new(server.uri().replace("http://", ""), "t")
        .insecure()
        .build_async()
        .await
        .unwrap();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    (server, App::new(client, tx, None), rx)
}

async fn drain(app: &mut App, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
    while let Ok(update) = rx.try_recv() {
        app.process_update_single(update).await;
    }
}

fn project() -> Project {
    Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("g/p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    }
}

#[tokio::test]
async fn test_schedules_fetched_and_applied() {
    let (server, mut app, mut rx) = setup_app().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/pipeline_schedules"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 7, "description": "nightly", "ref": "main", "cron": "0 2 * * *", "active": true},
            {"id": 8, "description": "weekly", "ref": "dev", "cron": "0 4 * * 1", "active": false}
        ])))
        .mount(&server)
        .await;
    app.selected_project = Some(project());

    app.fetch_pipeline_schedules().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    drain(&mut app, &mut rx).await;

    assert_eq!(app.pipeline_schedules.len(), 2);
    assert_eq!(app.pipeline_schedules[0].description, "nightly");
    assert_eq!(app.pipeline_schedules[0].cron, "0 2 * * *");
    assert!(app.pipeline_schedules[0].active);
    assert!(!app.pipeline_schedules[1].active);
}

#[tokio::test]
async fn test_run_schedule_sends_post() {
    let (server, mut app, _rx) = setup_app().await;
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/1/pipeline_schedules/7/play"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({"id": 99, "status": "pending"})),
        )
        .mount(&server)
        .await;
    app.selected_project = Some(project());

    app.run_pipeline_schedule(7).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let requests = server.received_requests().await.unwrap();
    assert!(
        requests.iter().any(|r| r.url.path().ends_with("/play")
            && r.method.as_str() == "POST"
            && r.url.path().ends_with("/pipeline_schedules/7/play")),
        "run schedule must POST to the play endpoint"
    );
}

#[tokio::test]
async fn test_variable_crud_endpoints_and_state() {
    let (server, mut app, mut rx) = setup_app().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v4/projects/1/variables/OLD_KEY"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/1/variables"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"key": "NEW_KEY", "value": "v1", "variable_type": "env_var", "protected": false, "masked": false, "environment_scope": "*"})))
        .mount(&server)
        .await;
    app.selected_project = Some(project());
    app.project_variables = vec![ProjectVariable {
        key: "OLD_KEY".into(),
        value: "x".into(),
        variable_type: "env_var".into(),
        protected: false,
        masked: false,
        environment_scope: "*".into(),
    }];

    app.delete_project_variable("OLD_KEY".into()).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    drain(&mut app, &mut rx).await;
    assert!(
        app.project_variables.iter().all(|v| v.key != "OLD_KEY"),
        "deleted var removed from local state"
    );

    app.upsert_project_variable("NEW_KEY".into(), "v1".into(), false)
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    drain(&mut app, &mut rx).await;
    assert!(
        app.project_variables
            .iter()
            .any(|v| v.key == "NEW_KEY" && v.value == "v1"),
        "created var present in local state"
    );

    let requests = server.received_requests().await.unwrap();
    assert!(
        requests
            .iter()
            .any(|r| r.url.path().ends_with("/variables/OLD_KEY") && r.method.as_str() == "DELETE")
    );
    let post_hit = requests.iter().any(|r| {
        r.url.path().ends_with("/api/v4/projects/1/variables") && r.method.as_str() == "POST"
    });
    assert!(post_hit, "variable create must POST to /variables");
}

#[tokio::test]
async fn test_approvals_fetched_and_rendered_in_confirm_popup() {
    let (server, mut app, mut rx) = setup_app().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/merge_requests/10/approvals"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "approvals_required": 2, "approvals_left": 0,
            "approved_by": [{"user": {"id": 5, "username": "bob"}}]
        })))
        .mount(&server)
        .await;
    app.selected_project = Some(project());
    app.selected_mr = Some(MergeRequest {
        iid: 10,
        project_id: 1,
        title: "M".into(),
        author: Author::new(0, "".into()),
        web_url: "u".into(),
        state: "opened".into(),
        reactions: Vec::new(),
        description: None,
        references: None,
        upvotes: None,
        downvotes: None,
        labels: Vec::new(),
        milestone: None,
        assignees: Vec::new(),
        reviewers: Vec::new(),
        squash: Some(false),
        created_at: Some("".into()),
        draft: Some(false),
        diff_refs: None,
        merge_status: None,
        detailed_merge_status: None,
        has_conflicts: None,
        head_pipeline: None,
        source_branch: "a".into(),
        target_branch: "b".into(),
        participants: Vec::new(),
    });

    // drive the approvals fetch through fetch_related_items
    app.fetch_related_items().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    drain(&mut app, &mut rx).await;

    let info = app.approvals.as_ref().expect("approvals should be applied");
    assert_eq!(info.approved_by.len(), 1);
    assert_eq!(
        info.approved_by[0]
            .user
            .as_ref()
            .map(|u| u.username.as_str()),
        Some("bob")
    );

    use ratatui::Terminal;
    let mut terminal = Terminal::new(ratatui::backend::TestBackend::new(120, 40)).unwrap();
    app.show_confirm_merge_popup = true;
    terminal.draw(|f| lazygitlab::ui::ui(f, &mut app)).unwrap();
    let frame = lazygitlab::headless::render_to_string(&terminal);
    assert!(
        frame.contains("Approvals:"),
        "merge confirm shows approvals"
    );
    assert!(frame.contains("@bob"), "approver username rendered");
}
