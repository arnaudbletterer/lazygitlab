use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use wiremock::{Mock, MockServer, ResponseTemplate};

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

#[tokio::test]
async fn test_transient_status_expires() {
    let mut app = setup_app().await;
    app.status_message = Some("Copied to clipboard!".into());
    app.tick_status();
    assert!(app.status_message.is_some(), "fresh status must stay");

    app.status_since -= std::time::Duration::from_secs(7);
    app.tick_status();
    assert!(
        app.status_message.is_none(),
        "stale transient status must expire"
    );
}

#[tokio::test]
async fn test_progress_status_is_not_expired() {
    let mut app = setup_app().await;
    app.status_message = Some("Fetching Todos...".into());
    app.status_since -= std::time::Duration::from_secs(60);
    app.tick_status();
    assert!(
        app.status_message.is_some(),
        "progress messages clear via Loading(false), not TTL"
    );
}

#[tokio::test]
async fn test_error_expires_after_ttl() {
    let mut app = setup_app().await;
    app.last_error = Some("boom".into());
    app.last_error_since = Some(std::time::Instant::now());
    app.tick_status();
    assert_eq!(
        app.last_error.as_deref(),
        Some("boom"),
        "fresh error must stay"
    );

    app.last_error_since = Some(std::time::Instant::now() - std::time::Duration::from_secs(16));
    app.tick_status();
    assert!(app.last_error.is_none(), "stale error must expire");
}
