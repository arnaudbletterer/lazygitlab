use lazygitlab::models::*;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::path;
use gitlab::GitlabBuilder;

async fn setup_app() -> (MockServer, App) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    (server, App::new(client))
}

#[tokio::test]
async fn test_panel_cycling_forward() {
    let (_server, mut app) = setup_app().await;
    app.active_panel = Panel::Dashboard;

    let right = KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };
    let tab = KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };

    // Dashboard -> Explore
    app.handle_key(right).await;
    assert_eq!(app.active_panel, Panel::Explore);

    // Explore -> Project
    app.handle_key(tab).await;
    assert_eq!(app.active_panel, Panel::Project);

    // Project -> Main
    app.handle_key(right).await;
    assert_eq!(app.active_panel, Panel::Main);

    // Main -> MainDetail
    app.handle_key(tab).await;
    assert_eq!(app.active_panel, Panel::MainDetail);

    // MainDetail -> Dashboard (Cycle)
    app.handle_key(right).await;
    assert_eq!(app.active_panel, Panel::Dashboard);
}

#[tokio::test]
async fn test_panel_cycling_backward() {
    let (_server, mut app) = setup_app().await;
    app.active_panel = Panel::Dashboard;

    let left = KeyEvent { code: KeyCode::Left, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };
    let shift_tab = KeyEvent { code: KeyCode::BackTab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };

    // Dashboard -> MainDetail (Cycle)
    app.handle_key(left).await;
    assert_eq!(app.active_panel, Panel::MainDetail);

    // MainDetail -> Main
    app.handle_key(shift_tab).await;
    assert_eq!(app.active_panel, Panel::Main);

    // Main -> Project
    app.handle_key(left).await;
    assert_eq!(app.active_panel, Panel::Project);

    // Project -> Explore
    app.handle_key(shift_tab).await;
    assert_eq!(app.active_panel, Panel::Explore);

    // Explore -> Dashboard
    app.handle_key(left).await;
    assert_eq!(app.active_panel, Panel::Dashboard);
}

#[tokio::test]
async fn test_job_log_toggle_section() {
    let (_server, mut app) = setup_app().await;
    app.view_mode = ViewMode::JobLog;
    app.job_log_lines = vec![
        LogLine { timestamp: "t".into(), content: "section start".into(), section: Some("s1".into()), is_section_start: true, is_section_end: false },
        LogLine { timestamp: "t".into(), content: "content".into(), section: Some("s1".into()), is_section_start: false, is_section_end: false },
    ];
    app.job_log_scroll = 0;
    app.collapsed_sections.clear();

    let tab = KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };
    
    // Tab should toggle section instead of cycling panels
    app.handle_key(tab).await;
    assert!(app.collapsed_sections.contains("s1"));
    assert_eq!(app.active_panel, Panel::Dashboard); // Should NOT have changed

    // Another Tab should uncollapse
    app.handle_key(tab).await;
    assert!(!app.collapsed_sections.contains("s1"));

    // Right should also toggle in JobLog mode
    let right = KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };
    app.handle_key(right).await;
    assert!(app.collapsed_sections.contains("s1"));
}

#[tokio::test]
async fn test_vim_keys_navigation() {
    let (_server, mut app) = setup_app().await;
    app.active_panel = Panel::Dashboard;

    let l = KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };
    let h = KeyEvent { code: KeyCode::Char('h'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };

    // Dashboard -> Explore
    app.handle_key(l).await;
    assert_eq!(app.active_panel, Panel::Explore);

    // Explore -> Dashboard
    app.handle_key(h).await;
    assert_eq!(app.active_panel, Panel::Dashboard);
}
