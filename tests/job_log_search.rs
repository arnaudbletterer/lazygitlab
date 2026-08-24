use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

async fn setup_app() -> App {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::path("/api/v4/user"))
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
    App::new(client, tx)
}

fn log_line(content: &str) -> LogLine {
    LogLine {
        timestamp: String::new(),
        content: content.into(),
        section: None,
        is_section_start: false,
        is_section_end: false,
    }
}

#[tokio::test]
async fn test_search_jumps_to_next_match_and_wraps() {
    let mut app = setup_app().await;
    app.job_log_lines = vec![
        log_line("start"),
        log_line("ERROR boom"),
        log_line("ok"),
        log_line("ERROR again"),
    ];
    app.job_log_cursor = 0;
    app.job_log_viewport_height = 10;

    app.job_log_search = "error".into();
    app.job_log_search_next();
    assert_eq!(app.job_log_cursor, 1, "first match after cursor");

    app.job_log_search_next();
    assert_eq!(app.job_log_cursor, 3, "next match");

    app.job_log_search_next();
    assert_eq!(app.job_log_cursor, 1, "wraps around");
}

#[tokio::test]
async fn test_search_no_match_sets_status() {
    let mut app = setup_app().await;
    app.job_log_lines = vec![log_line("a"), log_line("b")];
    app.job_log_cursor = 0;
    app.job_log_search = "zzz".into();

    app.job_log_search_next();
    assert_eq!(app.status_message.as_deref(), Some("No match for 'zzz'"));
}

#[tokio::test]
async fn test_slash_opens_input_and_enter_jumps() {
    let mut app = setup_app().await;
    app.view_mode = ViewMode::JobLog;
    app.job_log_lines = vec![
        log_line("fine"),
        log_line("panic: exploded"),
        log_line("done"),
    ];
    app.job_log_cursor = 0;
    app.job_log_viewport_height = 10;

    app.handle_key(key(KeyCode::Char('/'))).await;
    assert!(app.input_mode);
    assert_eq!(app.input_target, InputTarget::JobLogSearch);

    for ch in "panic".chars() {
        app.handle_key(key(KeyCode::Char(ch))).await;
    }
    assert_eq!(app.job_log_search, "panic");

    app.handle_key(key(KeyCode::Enter)).await;
    assert!(!app.input_mode);
    assert_eq!(app.job_log_cursor, 1, "Enter jumps to first matching line");
}

#[tokio::test]
async fn test_page_and_end_navigation() {
    let mut app = setup_app().await;
    app.view_mode = ViewMode::JobLog;
    app.job_log_lines = (0..50).map(|i| log_line(&format!("line {}", i))).collect();
    app.job_log_cursor = 0;
    app.job_log_viewport_height = 10;

    app.handle_key(key(KeyCode::Char('G'))).await;
    assert_eq!(app.job_log_cursor, 49, "G jumps to last line");

    app.handle_key(key(KeyCode::PageUp)).await;
    assert_eq!(app.job_log_cursor, 39, "PageUp moves a viewport up");

    app.handle_key(key(KeyCode::Home)).await;
    assert_eq!(app.job_log_cursor, 0);

    app.handle_key(key(KeyCode::PageDown)).await;
    assert_eq!(app.job_log_cursor, 10, "PageDown moves a viewport down");
}
