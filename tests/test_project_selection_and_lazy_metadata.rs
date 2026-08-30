use gitlab::GitlabBuilder;
use lazygitlab::models::{
    App, Author, Context, DataUpdate, GroupedItem, MenuSelection, MergeRequest, Panel, Project,
    SearchResult,
};
use tokio::sync::mpsc;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_app() -> (MockServer, App, mpsc::UnboundedReceiver<DataUpdate>) {
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
    let (tx, rx) = mpsc::unbounded_channel();
    let app = App::new(client, tx, None);
    (server, app, rx)
}

#[tokio::test]
async fn test_project_selection_focus_and_lazy_metadata() {
    let (server, mut app, mut rx) = setup_app().await;

    // We mock the API endpoints to make sure they return 200
    Mock::given(path("/api/v4/projects/1/merge_requests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    // Simulate Explore Projects Selection (MenuSelection::Projects key handler)
    let proj = Project {
        id: 1,
        name: "test-project".into(),
        path_with_namespace: Some("namespace/test-project".into()),
        web_url: Some("http://url".into()),
        default_branch: None,
        namespace: None,
    };
    app.filtered_projects = vec![proj.clone()];
    app.projects_state.select(Some(0));
    app.active_selection = MenuSelection::Projects;
    app.active_panel = Panel::Main;

    // Press Enter to select the project
    app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Enter,
        crossterm::event::KeyModifiers::NONE,
    ))
    .await;

    // Verify Focus Alignment
    assert_eq!(app.selected_project.as_ref().unwrap().id, 1);
    assert_eq!(
        app.active_panel,
        Panel::Project,
        "Explore selection should focus Panel::Project sidebar"
    );
    assert_eq!(app.active_context, Context::Project);
    assert_eq!(
        app.project_menu_state.selected(),
        Some(0),
        "Explore selection should select index 0"
    );
    assert_eq!(app.active_selection, MenuSelection::MergeRequests);

    // Verify no eager metadata queries were sent
    assert!(
        !app.project_label_cache.contains_key(&1),
        "Metadata should not be eagerly fetched upon selection"
    );

    // Clear state and test Project Switcher Popup selection (select_project())
    app.selected_project = None;
    app.active_panel = Panel::Main;
    app.projects = vec![proj.clone()];
    app.projects_state.select(Some(0));
    app.select_project().await;

    assert_eq!(app.selected_project.as_ref().unwrap().id, 1);
    assert_eq!(
        app.active_panel,
        Panel::Project,
        "Switcher popup selection should focus Panel::Project sidebar"
    );
    assert_eq!(app.project_menu_state.selected(), Some(0));

    // Clear state and test Global Search Result selection (SearchResult::Project)
    app.selected_project = None;
    app.active_panel = Panel::Main;
    app.search_results = vec![SearchResult::Project(proj.clone())];
    app.search_state.select(Some(0));
    app.show_search = true;
    app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Enter,
        crossterm::event::KeyModifiers::NONE,
    ))
    .await;

    assert_eq!(app.selected_project.as_ref().unwrap().id, 1);
    assert_eq!(
        app.active_panel,
        Panel::Project,
        "Global search selection should focus Panel::Project sidebar"
    );
    assert_eq!(app.project_menu_state.selected(), Some(0));

    // Test Lazy Metadata Loading: Select an MR in list view, metadata should not load yet
    let mr = MergeRequest {
        iid: 1,
        project_id: 1,
        title: "mr title".into(),
        description: None,
        author: Author::new(1, "u".into()),
        web_url: "u".into(),
        state: "opened".into(),
        references: None,
        upvotes: Some(0),
        downvotes: Some(0),
        labels: vec![],
        milestone: None,
        assignees: vec![],
        reviewers: vec![],
        squash: Some(false),
        created_at: Some("t".into()),
        draft: Some(false),
        diff_refs: None,
        reactions: vec![],
        merge_status: None,
        detailed_merge_status: None,
        has_conflicts: None,
        head_pipeline: None,
        source_branch: "source".into(),
        target_branch: "target".into(),
        participants: vec![],
    };
    app.active_panel = Panel::Main;
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::MergeRequests;
    app.filtered_mrs = vec![GroupedItem::Item(mr.clone())];
    app.mrs_state.select(Some(0));

    app.update_preview().await;
    assert_eq!(app.selected_mr.as_ref().unwrap().iid, 1);
    assert!(
        !app.project_label_cache.contains_key(&1),
        "Metadata should not load in list view"
    );

    // Enter Detail View: should trigger lazy metadata fetch
    app.active_panel = Panel::MainDetail;

    // We mock the metadata endpoints
    Mock::given(path("/api/v4/projects/1/labels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 1, "name": "bug", "color": "#ff0000", "description": "Bug label"}
        ])))
        .mount(&server)
        .await;
    Mock::given(path("/api/v4/projects/1/milestones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    Mock::given(path("/api/v4/projects/1/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    app.refresh().await;

    // Drain the receiver to let DataUpdate execute
    // Wait up to 1 second for updates to be processed
    let _ = tokio::time::timeout(tokio::time::Duration::from_secs(1), async {
        loop {
            app.process_updates(&mut rx).await;
            if app.project_label_cache.contains_key(&1) {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    })
    .await;

    assert!(
        app.project_label_cache.contains_key(&1),
        "Metadata fetch should be triggered in Panel::MainDetail and populate the cache"
    );
}
