use gitlab::GitlabBuilder;
use lazygitlab::models::{
    App, Author, Context, DetailSection, GroupedItem, Issue, MRDiff, MenuSelection, MergeRequest,
    Panel, Project, SearchResult, Todo, ViewMode,
};
use tokio::sync::mpsc;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_app() -> (
    MockServer,
    App,
    mpsc::UnboundedReceiver<lazygitlab::models::DataUpdate>,
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
    let (tx, rx) = mpsc::unbounded_channel();
    let app = App::new(client, tx, None);
    (server, app, rx)
}

fn dummy_mr(iid: u64, title: &str) -> MergeRequest {
    MergeRequest {
        updated_at: None,
        iid,
        project_id: 1,
        title: title.into(),
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
    }
}

#[tokio::test]
async fn test_lazy_load_mrs_diffs() {
    let (server, mut app, mut _rx) = setup_app().await;

    // Mock Version/Metadata responses
    Mock::given(path("/api/v4/projects/1/merge_requests/1/discussions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    Mock::given(path("/api/v4/projects/1/merge_requests/2/discussions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    Mock::given(path("/api/v4/projects/1/labels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
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

    Mock::given(path("/api/v4/projects/1/merge_requests/1/diffs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let mr = dummy_mr(1, "Test MR");

    // Simulate selecting the MR in a list
    app.active_panel = Panel::Main;
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::MergeRequests;
    app.filtered_mrs = vec![GroupedItem::Item(mr.clone())];
    app.mrs_state.select(Some(0));

    app.update_preview().await;

    // Verify MR is selected, but diff fetch was NOT initiated eagerly
    assert_eq!(app.selected_mr.as_ref().unwrap().iid, 1);
    assert!(
        !app.is_fetching_diffs,
        "Diff fetching should not be triggered eagerly"
    );
    assert!(
        app.mr_diffs.is_empty(),
        "MR diffs should be empty initially"
    );

    // Enable Diff view mode -> should trigger diff fetching lazy load
    app.view_mode = ViewMode::Diff;
    app.update_preview().await;

    assert!(
        app.is_fetching_diffs,
        "Diff fetching should be lazy-loaded in ViewMode::Diff"
    );

    // Reset diff fetching state for testing Changes panel lazy-load
    app.is_fetching_diffs = false;
    app.view_mode = ViewMode::Detail;
    app.active_panel = Panel::MainDetail;
    app.detail_section = DetailSection::Changes;

    app.update_preview().await;
    assert!(
        app.is_fetching_diffs,
        "Diff fetching should be lazy-loaded when Changes section is focused"
    );

    // Test selection from Todos clears diff state
    app.mr_diffs = vec![MRDiff {
        old_path: "a".into(),
        new_path: "b".into(),
        diff: "diff".into(),
        collapsed: false,
        display_lines: vec![],
    }];
    app.is_fetching_diffs = true;

    // Select target Todo pointing to a DIFFERENT MR (iid = 2)
    let todo = Todo {
        id: 1,
        project: Some(Project {
            id: 1,
            name: "p".into(),
            path_with_namespace: Some("p/n".into()),
            web_url: Some("url".into()),
            default_branch: None,
            namespace: None,
        }),
        action_name: "assigned".into(),
        target_type: "MergeRequest".into(),
        target_url: "https://gitlab.com/p/n/-/merge_requests/2".into(),
        body: "todo".into(),
        state: "pending".into(),
        created_at: "t".into(),
        group: None,
    };
    app.active_panel = Panel::Main;
    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::Todos;
    app.filtered_todos = vec![GroupedItem::Item(todo)];
    app.todos_state.select(Some(0));

    app.update_preview().await;
    assert_eq!(app.selected_mr.as_ref().unwrap().iid, 2);
    assert!(
        app.mr_diffs.is_empty(),
        "mr_diffs should be cleared on new Todo MR selection"
    );
    assert!(
        !app.is_fetching_diffs,
        "is_fetching_diffs should be reset on new Todo MR selection"
    );

    // Transition selection state to prevent update_preview from overwriting the search selection
    app.active_selection = MenuSelection::MergeRequests;
    app.filtered_mrs.clear();
    app.mrs_state.select(None);
    app.active_panel = Panel::Main;
    app.update_preview().await;

    // Test selection from SearchResult clears diff state
    app.mr_diffs = vec![MRDiff {
        old_path: "a".into(),
        new_path: "b".into(),
        diff: "diff".into(),
        collapsed: false,
        display_lines: vec![],
    }];
    app.is_fetching_diffs = true;
    app.selected_issue = Some(Issue {
        updated_at: None,
        iid: 1,
        project_id: 1,
        title: "issue".into(),
        description: None,
        author: Author::new(1, "u".into()),
        assignees: vec![],
        state: "opened".into(),
        web_url: "url".into(),
        references: None,
        issue_type: None,
        labels: vec![],
        milestone: None,
        due_date: None,
        start_date: None,
        weight: None,
        task_completion_status: None,
        confidential: false,
        created_at: None,
        upvotes: None,
        downvotes: None,
        reactions: vec![],
        participants: vec![],
    });

    let search_mr = SearchResult::MergeRequest(dummy_mr(3, "Search MR"));
    app.search_results = vec![search_mr];
    app.search_state.select(Some(0));

    // Simulate Enter on search result (need input_mode to be false to bypass search input editing)
    app.show_search = true;
    app.input_mode = false;
    app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Enter,
        crossterm::event::KeyModifiers::NONE,
    ))
    .await;

    assert_eq!(app.selected_mr.as_ref().unwrap().iid, 3);
    assert!(
        app.selected_issue.is_none(),
        "selected_issue should be cleared"
    );
    assert!(
        app.mr_diffs.is_empty(),
        "mr_diffs should be cleared on Search MR selection"
    );
    assert!(
        !app.is_fetching_diffs,
        "is_fetching_diffs should be reset on Search MR selection"
    );
}
