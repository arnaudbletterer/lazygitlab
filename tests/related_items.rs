use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use wiremock::matchers::path;
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

fn mr(iid: u64) -> serde_json::Value {
    serde_json::json!({"iid": iid, "project_id": 1, "title": format!("MR {}", iid), "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "source_branch": "a", "target_branch": "b", "draft": false})
}

fn issue(iid: u64) -> serde_json::Value {
    serde_json::json!({"iid": iid, "project_id": 1, "title": format!("Issue {}", iid), "author": {"id": 1, "username": "u"}, "state": "opened", "web_url": "u", "confidential": false})
}

async fn drain(app: &mut App, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
    while let Ok(update) = rx.try_recv() {
        app.process_update_single(update).await;
    }
}

#[tokio::test]
async fn test_mr_related_items_fetched_and_applied() {
    let (server, mut app, mut rx) = setup_app().await;
    Mock::given(path(
        "/api/v4/projects/1/merge_requests/10/related_merge_requests",
    ))
    .respond_with(ResponseTemplate::new(200).set_body_json(vec![mr(11)]))
    .mount(&server)
    .await;
    Mock::given(path("/api/v4/projects/1/merge_requests/10/closes_issues"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![issue(7)]))
        .mount(&server)
        .await;
    app.active_context = Context::Project;
    app.selected_project = Some(Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    });
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

    app.fetch_related_items().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    drain(&mut app, &mut rx).await;

    assert_eq!(
        app.related_mrs.len(),
        1,
        "related MRs applied via DataUpdate"
    );
    assert_eq!(app.related_mrs[0].iid, 11);
    assert_eq!(app.mr_closing_issues.len(), 1);
    assert_eq!(app.mr_closing_issues[0].iid, 7);
}

#[tokio::test]
async fn test_issue_links_and_closed_by_applied() {
    let (server, mut app, mut rx) = setup_app().await;
    Mock::given(path("/api/v4/projects/1/issues/5/links"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![issue(6)]))
        .mount(&server)
        .await;
    Mock::given(path("/api/v4/projects/1/issues/5/closed_by"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![mr(12)]))
        .mount(&server)
        .await;
    app.active_context = Context::Project;
    app.selected_project = Some(Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    });
    app.selected_issue = Some(Issue {
        iid: 5,
        project_id: 1,
        title: "I".into(),
        description: None,
        author: Author::new(0, "".into()),
        assignees: Vec::new(),
        state: "opened".into(),
        web_url: "u".into(),
        references: None,
        issue_type: None,
        labels: Vec::new(),
        milestone: None,
        due_date: None,
        start_date: None,
        weight: None,
        task_completion_status: None,
        confidential: false,
        created_at: Some("".into()),
        upvotes: None,
        downvotes: None,
        reactions: Vec::new(),
        participants: Vec::new(),
    });

    app.fetch_related_items().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    drain(&mut app, &mut rx).await;

    assert_eq!(app.linked_issues.len(), 1);
    assert_eq!(app.linked_issues[0].iid, 6);
    assert_eq!(app.issue_closing_mrs.len(), 1);
    assert_eq!(app.issue_closing_mrs[0].iid, 12);
}

#[tokio::test]
async fn test_stale_updates_rejected() {
    let (_server, mut app, mut rx) = setup_app().await;
    app.active_context = Context::Project;
    app.selected_project = Some(Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    });
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

    let stale: MergeRequest = serde_json::from_value(mr(99)).unwrap();
    app.process_update_single(DataUpdate::RelatedMRs {
        project_id: 2,
        iid: 99,
        data: vec![stale],
    })
    .await;
    drain(&mut app, &mut rx).await;

    assert!(
        app.related_mrs.is_empty(),
        "updates for a different project/iid must be ignored"
    );
}

#[tokio::test]
async fn test_metadata_panel_renders_related_rows() {
    let (_server, mut app, _rx) = setup_app().await;
    app.active_context = Context::Project;
    app.selected_project = Some(Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    });
    app.active_panel = Panel::MainDetail;
    app.view_mode = ViewMode::Detail;
    app.detail_section = DetailSection::Description;
    let related: MergeRequest = serde_json::from_value(mr(11)).unwrap();
    let closes: Issue = serde_json::from_value(issue(7)).unwrap();
    app.related_mrs.push(related);
    app.mr_closing_issues.push(closes);
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

    let backend = ratatui::backend::TestBackend::new(120, 40);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|f| lazygitlab::ui::ui(f, &mut app)).unwrap();

    let frame = lazygitlab::headless::render_to_string(&terminal);
    assert!(
        frame.contains("Rel. MRs:"),
        "metadata panel shows the related MRs row"
    );
    assert!(frame.contains("!11"), "related MR iid rendered");
    assert!(frame.contains("#7"), "closing issue iid rendered");
}
