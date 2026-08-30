use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_app() -> (MockServer, App) {
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
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.dashboard_menu_state.select(Some(1));
    app.active_selection = MenuSelection::MergeRequests;

    let mr = MergeRequest {
        iid: 1,
        project_id: 1,
        title: "Test MR".into(),
        author: Author::new(1, "u".into()),
        web_url: "http://example.com".into(),
        state: "opened".into(),
        reactions: vec![],
        description: None,
        references: None,
        upvotes: None,
        downvotes: None,
        labels: vec![],
        milestone: None,
        assignees: vec![],
        reviewers: vec![],
        squash: None,
        created_at: None,
        draft: None,
        diff_refs: None,
        merge_status: None,
        detailed_merge_status: None,
        has_conflicts: None,
        head_pipeline: None,
        source_branch: "".into(),
        target_branch: "".into(),
        participants: vec![],
    };
    app.selected_mr = Some(mr.clone());
    app.merge_requests = vec![mr.clone()];
    app.filtered_mrs = vec![GroupedItem::Item(mr)];
    app.mrs_state.select(Some(0));
    (server, app)
}

#[tokio::test]
async fn test_panel_cycling_forward() {
    let (_server, mut app) = setup_app().await;
    app.active_panel = Panel::Main;

    let right = KeyEvent {
        code: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    let tab = KeyEvent {
        code: KeyCode::Tab,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    // Under app's new logic:
    // Tab cycles Sidebar <-> Main panels (excluding MainDetail).

    // Main -> Sidebar (Dashboard)
    app.handle_key(tab).await;
    assert!(matches!(
        app.active_panel,
        Panel::Dashboard | Panel::Explore | Panel::Project
    ));

    // Sidebar -> Main
    app.handle_key(tab).await;
    assert_eq!(app.active_panel, Panel::Main);

    // Main -> MainDetail via Right/l
    app.handle_key(right).await;
    assert_eq!(app.active_panel, Panel::MainDetail);

    // When focused on MainDetail, Tab cycles detail tabs locally instead of changing active_panel
    app.detail_section = DetailSection::Metadata;
    app.handle_key(tab).await;
    assert_eq!(app.active_panel, Panel::MainDetail);
    assert_eq!(app.detail_section, DetailSection::Discussions);

    // Dashboard -> Main via Right (under app's logic, Sidebar + Right -> Main)
    app.active_panel = Panel::Dashboard;
    app.handle_key(right).await;
    assert_eq!(app.active_panel, Panel::Main);
}

#[tokio::test]
async fn test_panel_cycling_backward() {
    let (_server, mut app) = setup_app().await;
    app.active_panel = Panel::Main;

    let left = KeyEvent {
        code: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    let shift_tab = KeyEvent {
        code: KeyCode::BackTab,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    // Under app's new logic:
    // BackTab cycles Sidebar <-> Main panels (excluding MainDetail).

    // Main -> Sidebar
    app.handle_key(shift_tab).await;
    assert!(matches!(
        app.active_panel,
        Panel::Dashboard | Panel::Explore | Panel::Project
    ));

    // Sidebar -> Main
    app.handle_key(shift_tab).await;
    assert_eq!(app.active_panel, Panel::Main);

    // Main -> Sidebar via Left
    app.handle_key(left).await;
    assert!(matches!(
        app.active_panel,
        Panel::Dashboard | Panel::Explore | Panel::Project
    ));

    // Focus MainDetail again to test Tab cycling and Left navigation
    app.active_panel = Panel::MainDetail;
    app.detail_section = DetailSection::Discussions;

    // Focused on MainDetail + BackTab cycles detail tabs backward locally
    app.handle_key(shift_tab).await;
    assert_eq!(app.active_panel, Panel::MainDetail);
    assert_eq!(app.detail_section, DetailSection::Metadata);

    // MainDetail -> Main via Left
    app.handle_key(left).await;
    assert_eq!(app.active_panel, Panel::Main);
}

#[tokio::test]
async fn test_job_log_toggle_section() {
    let (_server, mut app) = setup_app().await;
    app.view_mode = ViewMode::JobLog;
    app.job_log_lines = vec![
        LogLine {
            timestamp: "t".into(),
            content: "section_start:1:s1".into(),
            section: Some("s1".into()),
            is_section_start: true,
            is_section_end: false,
        },
        LogLine {
            timestamp: "t".into(),
            content: "content".into(),
            section: Some("s1".into()),
            is_section_start: false,
            is_section_end: false,
        },
    ];
    app.job_log_scroll = 0;
    app.collapsed_sections.clear();

    let tab = KeyEvent {
        code: KeyCode::Tab,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    // Tab should toggle section instead of cycling panels
    app.handle_key(tab).await;
    assert!(app.collapsed_sections.contains("s1"));

    // Another Tab should uncollapse
    app.handle_key(tab).await;
    assert!(!app.collapsed_sections.contains("s1"));
}

#[tokio::test]
async fn test_vim_keys_navigation() {
    let (_server, mut app) = setup_app().await;
    app.active_panel = Panel::Main;

    let l = KeyEvent {
        code: KeyCode::Char('l'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    let h = KeyEvent {
        code: KeyCode::Char('h'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    // Main -> MainDetail
    app.handle_key(l).await;
    assert_eq!(app.active_panel, Panel::MainDetail);

    // MainDetail -> Main
    app.handle_key(h).await;
    assert_eq!(app.active_panel, Panel::Main);
}
