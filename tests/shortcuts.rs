use lazygitlab::models::{App, Context, MenuSelection, Panel};
use gitlab::GitlabBuilder;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::path;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};

async fn setup_app() -> (MockServer, App) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    (server, App::new(client))
}

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

#[tokio::test]
async fn test_g_prefix_shortcuts() {
    let (_server, mut app) = setup_app().await;
    
    // Initial state
    assert_eq!(app.active_context, Context::Dashboard);
    assert_eq!(app.active_selection, MenuSelection::Todos);
    assert!(!app.pending_g_prefix);

    // Press 'g'
    app.handle_key(key_event(KeyCode::Char('g'))).await;
    assert!(app.pending_g_prefix);

    // Press 'i' -> Should go to Issues
    app.handle_key(key_event(KeyCode::Char('i'))).await;
    assert!(!app.pending_g_prefix);
    assert_eq!(app.active_selection, MenuSelection::Issues);
    assert_eq!(app.active_panel, Panel::Main);

    // Press 'g' then 't' -> Should go to Todos
    app.handle_key(key_event(KeyCode::Char('g'))).await;
    app.handle_key(key_event(KeyCode::Char('t'))).await;
    assert_eq!(app.active_selection, MenuSelection::Todos);
}
