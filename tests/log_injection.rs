use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use lazygitlab::ui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn log_line(content: &str) -> LogLine {
    LogLine {
        timestamp: "t".into(),
        content: content.into(),
        section: None,
        is_section_start: false,
        is_section_end: false,
    }
}

#[tokio::test]
async fn job_log_escape_sequences_never_reach_the_terminal_buffer() {
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
    app.view_mode = ViewMode::JobLog;
    app.job_log_lines = vec![
        log_line("visible-start \u{1b}]0;pwned-title\u{7} visible-end"),
        log_line("\u{1b}[2J\u{1b}[1;1Hclear-attempt \u{1b}[31mred\u{1b}[0m"),
        log_line("\u{1b}]52;c;ZXZpbA==\u{1b}\\clipboard-attempt"),
    ];

    let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    let text: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();

    assert!(text.contains("visible-start"), "log not rendered");
    assert!(text.contains("clear-attempt"), "log not rendered");
    assert!(!text.contains("pwned-title"), "OSC payload leaked");
    assert!(!text.contains("ZXZpbA"), "OSC 52 payload leaked");
    assert!(!text.contains('\u{1b}'), "raw escape reached the buffer");
}

#[test]
fn markdown_from_mr_text_has_no_escape_characters() {
    use lazygitlab::ui::markdown::render_markdown;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;

    let theme = lazygitlab::ui::Theme::default();
    let lines = render_markdown(
        "hello \u{1b}]0;pwned\u{7} **bold** \u{1b}[2J world",
        &theme,
        &SyntaxSet::load_defaults_newlines(),
        &ThemeSet::load_defaults(),
    );
    let joined: String = lines
        .iter()
        .flat_map(|l| l.spans.iter())
        .map(|s| s.content.as_ref())
        .collect();
    assert!(joined.contains("hello"), "{joined}");
    assert!(!joined.contains('\u{1b}'), "{joined:?}");
    assert!(!joined.contains("pwned"), "{joined:?}");
}
