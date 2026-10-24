use gitlab::GitlabBuilder;
use lazygitlab::errors::friendly_error;
use lazygitlab::models::*;
use lazygitlab::ui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use std::process::{Command, Stdio};
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn run_with_config(config_body: &str) -> String {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("lazygitlab-tls-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".config/lazygitlab")).unwrap();
    std::fs::write(dir.join(".config/lazygitlab/config.toml"), config_body).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_lazygitlab"))
        .current_dir(&dir)
        .env("HOME", &dir)
        .env("XDG_STATE_HOME", dir.join("state"))
        .env_remove("GITLAB_URL")
        .env("GITLAB_TOKEN", "glpat-tlsWarningToken123")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn config_file_insecure_warns_on_stderr_every_start() {
    let stderr = run_with_config("[gitlab]\nurl = \"127.0.0.1:9\"\ninsecure = true\n");
    assert!(
        stderr.contains("TLS certificate verification is disabled"),
        "{stderr}"
    );
    assert!(!stderr.contains("glpat-tlsWarningToken123"), "{stderr}");
}

#[test]
fn verified_tls_does_not_warn() {
    let stderr = run_with_config("[gitlab]\nurl = \"https://127.0.0.1:9\"\n");
    assert!(!stderr.contains("verification is disabled"), "{stderr}");
}

#[test]
fn certificate_hint_recommends_trust_store_not_permanent_config() {
    let msg = friendly_error("client error: invalid peer certificate: UnknownIssuer");
    assert!(msg.contains("trust store"), "{msg}");
    assert!(!msg.contains("config.toml"), "{msg}");
}

#[tokio::test]
async fn header_marks_unverified_tls() {
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
    let render = |app: &mut App| {
        let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();
        terminal.draw(|f| ui(f, app)).unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>()
    };
    assert!(!render(&mut app).contains("TLS NOT VERIFIED"));
    app.skip_tls_verify = true;
    assert!(render(&mut app).contains("TLS NOT VERIFIED"));
}
