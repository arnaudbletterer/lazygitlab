use gitlab::GitlabBuilder;
use lazygitlab::errors::friendly_error;
use lazygitlab::models::*;
use lazygitlab::redaction::register_secret;
use lazygitlab::sanitize::strip_invisible;
use lazygitlab::ui::markdown::render_markdown;
use lazygitlab::ui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn passthrough_errors_are_redacted_and_stripped() {
    register_secret("glpat-passThroughSecret77");
    let msg = friendly_error("boom glpat-passThroughSecret77 \u{1b}]0;title\u{7}tail \u{1b}[2J");
    assert!(!msg.contains("passThroughSecret77"), "{msg}");
    assert!(!msg.contains('\u{1b}'), "{msg:?}");
    assert!(msg.contains("tail"), "{msg}");
}

#[test]
fn hinted_errors_are_stripped_too() {
    let msg = friendly_error("gitlab server error (401 Unauthorized): \u{1b}]52;c;AAAA\u{7}bad");
    assert!(msg.contains("token is invalid"), "{msg}");
    assert!(!msg.contains('\u{1b}'), "{msg:?}");
}

#[test]
fn bidi_and_zero_width_characters_are_removed() {
    assert_eq!(
        strip_invisible("a\u{202e}b\u{200b}c\u{feff}d\u{2066}e\u{200f}"),
        "abcde"
    );
    assert_eq!(strip_invisible("plain"), "plain");
}

#[test]
fn markdown_entity_encoded_escape_does_not_reform() {
    let theme = lazygitlab::ui::Theme::default();
    let lines = render_markdown(
        "x &#27;]52;c;ZXZpbA==&#7; y `a&#27;[2Jb` z\u{202e}w",
        &theme,
        &SyntaxSet::load_defaults_newlines(),
        &ThemeSet::load_defaults(),
    );
    let joined: String = lines
        .iter()
        .flat_map(|l| l.spans.iter())
        .map(|s| s.content.as_ref())
        .collect();
    assert!(!joined.contains('\u{1b}'), "{joined:?}");
    assert!(!joined.contains('\u{7}'), "{joined:?}");
    assert!(!joined.contains('\u{202e}'), "{joined:?}");
}

#[tokio::test]
async fn rendered_buffer_never_contains_bidi_overrides() {
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
    app.last_error = Some("evil\u{202e}gnp.exe and zero\u{200b}width a\u{202e}".to_string());
    let mut terminal = Terminal::new(TestBackend::new(140, 30)).unwrap();
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    let text: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();
    assert!(text.contains("evil"), "error not rendered");
    for c in ['\u{202e}', '\u{200b}', '\u{feff}', '\u{2066}'] {
        assert!(!text.contains(c), "{c:?} reached the terminal");
    }
}
