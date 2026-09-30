use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
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
    let app = App::new(client, tx, None);
    (server, app, rx)
}

#[tokio::test]
async fn test_selection_coherence_on_navigation() {
    let (_server, mut app, _rx) = setup_app().await;

    // 1. Initial State: Dashboard, Todos selected but list empty
    assert_eq!(app.active_panel, Panel::Dashboard);
    assert_eq!(app.dashboard_menu_state.selected(), Some(0));
    assert!(app.todos.is_empty());
    assert_eq!(app.todos_state.selected(), None);

    // 2. Load data and verify auto-selection
    app.todos = vec![Todo {
        id: 1,
        action_name: "t".into(),
        body: "b".into(),
        target_url: "u".into(),
        target_type: "Issue".into(),
        state: "pending".into(),
        created_at: "t".into(),
        project: Some(Project {
            id: 1,
            name: "p".into(),
            web_url: None,
            path_with_namespace: None,
            default_branch: None,
            namespace: None,
        }),
        group: None,
    }];
    app.filter_lists(); // This calls clamp_selections

    assert_eq!(
        app.todos_state.selected(),
        Some(0),
        "First item should be auto-selected when data arrives"
    );

    // 3. Move to Panel::Main
    let enter = KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    app.handle_key(enter).await;
    assert_eq!(app.active_panel, Panel::Main);
    assert_eq!(app.todos_state.selected(), Some(0));

    // 4. Move to Panel::Explore, verify dashboard selection is preserved but focus is clear
    let down = KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    app.active_panel = Panel::Dashboard;
    app.handle_key(down).await;
    assert_eq!(app.dashboard_menu_state.selected(), Some(1));
}

#[tokio::test]
async fn test_selection_clamping_on_filtering() {
    let (_server, mut app, _rx) = setup_app().await;

    app.todos = vec![
        Todo {
            id: 1,
            action_name: "match".into(),
            body: "match".into(),
            target_url: "u".into(),
            target_type: "Issue".into(),
            state: "pending".into(),
            created_at: "t".into(),
            project: Some(Project {
                id: 1,
                name: "p".into(),
                web_url: None,
                path_with_namespace: None,
                default_branch: None,
                namespace: None,
            }),
            group: None,
        },
        Todo {
            id: 2,
            action_name: "other".into(),
            body: "other".into(),
            target_url: "u".into(),
            target_type: "Issue".into(),
            state: "pending".into(),
            created_at: "t".into(),
            project: Some(Project {
                id: 1,
                name: "p".into(),
                web_url: None,
                path_with_namespace: None,
                default_branch: None,
                namespace: None,
            }),
            group: None,
        },
    ];
    app.filter_lists();
    app.todos_state.select(Some(2)); // Select "other" (index 0 is Header, 1 is "match", 2 is "other")

    // Filter so only "match" remains
    app.filter_input = "match".into();
    app.filter_lists();

    assert_eq!(app.filtered_todos.len(), 2); // 1 Header + 1 Item
    assert_eq!(
        app.todos_state.selected(),
        Some(0),
        "Selection should be reset to 0 after filtering"
    );

    // Filter so nothing remains
    app.filter_input = "nomatch".into();
    app.filter_lists();
    assert_eq!(
        app.todos_state.selected(),
        None,
        "Selection should be None if no items remain"
    );
}
