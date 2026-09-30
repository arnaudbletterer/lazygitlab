use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use lazygitlab::redaction::register_secret;
use lazygitlab::ui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn render_with(set: impl FnOnce(&mut App)) -> String {
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
        .expect("client");
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);

    register_secret("plainSelfHostedSecret42");
    set(&mut app);

    let mut terminal = Terminal::new(TestBackend::new(200, 50)).expect("terminal");
    terminal.draw(|f| ui(f, &mut app)).expect("draw");

    let buffer = terminal.backend().buffer();
    let mut content = String::new();
    for y in 0..buffer.area().height {
        for x in 0..buffer.area().width {
            content.push_str(buffer[(x, y)].symbol());
        }
        content.push('\n');
    }
    content
}

#[tokio::test]
async fn status_bar_error_never_shows_tokens() {
    let content = render_with(|app| {
        app.last_error = Some("bad glpat-AbCdEf0123456789xy and plainSelfHostedSecret42".into());
    })
    .await;
    assert!(content.contains("[REDACTED]"), "{content}");
    assert!(!content.contains("glpat-AbCdEf0123456789xy"));
    assert!(!content.contains("plainSelfHostedSecret42"));
}

#[tokio::test]
async fn status_message_never_shows_tokens() {
    let content = render_with(|app| {
        app.status_message = Some("Pipeline failed: glpat-AbCdEf0123456789xy".into());
    })
    .await;
    assert!(content.contains("[REDACTED]"), "{content}");
    assert!(!content.contains("glpat-AbCdEf0123456789xy"));
}
