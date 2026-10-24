use gitlab::GitlabBuilder;
use lazygitlab::errors::friendly_error;
use lazygitlab::models::*;
use lazygitlab::redaction::register_secret;
use std::time::Duration;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

const TOKEN: &str = "glpat-launchHardeningSecret99";

async fn first_error(host: String, insecure: bool) -> String {
    register_secret(TOKEN);
    let mut builder = GitlabBuilder::new(host, TOKEN);
    if insecure {
        builder.insecure();
    }
    let client = match builder.build_async().await {
        Ok(client) => client,
        Err(e) => return friendly_error(&e.to_string()),
    };
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    let _ = app.initial_setup().await;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    while app.last_error.is_none() {
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        match tokio::time::timeout(left, rx.recv()).await {
            Ok(Some(update)) => app.process_update_single(update).await,
            _ => break,
        }
    }
    app.last_error.expect("an error should be surfaced")
}

async fn server_answering(status: u16, body: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(status).set_body_string(body))
        .mount(&server)
        .await;
    server
}

fn host(server: &MockServer) -> String {
    server.uri().replace("http://", "")
}

#[tokio::test]
async fn wrong_token_is_actionable_and_never_echoed() {
    let server = server_answering(401, r#"{"message":"401 Unauthorized"}"#).await;
    let err = first_error(host(&server), true).await;
    assert!(err.contains("token is invalid"), "{err}");
    assert!(err.contains("read_api"), "{err}");
    assert!(!err.contains(TOKEN), "{err}");
}

#[tokio::test]
async fn forbidden_explains_scope() {
    let server = server_answering(403, r#"{"message":"403 Forbidden"}"#).await;
    let err = first_error(host(&server), true).await;
    assert!(err.contains("permission denied"), "{err}");
    assert!(err.contains("scope"), "{err}");
}

#[tokio::test]
async fn rate_limit_says_to_wait() {
    let server = server_answering(429, r#"{"message":"Retry later"}"#).await;
    let err = first_error(host(&server), true).await;
    assert!(err.contains("rate limited"), "{err}");
}

#[tokio::test]
async fn non_gitlab_url_hints_at_path_prefix() {
    let server = server_answering(200, "<html>welcome to nginx</html>").await;
    let err = first_error(host(&server), true).await;
    assert!(err.contains("path prefix"), "{err}");
}

#[tokio::test]
async fn network_down_says_cannot_reach() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    drop(listener);
    let err = first_error(addr.to_string(), true).await;
    assert!(err.contains("cannot reach GitLab"), "{err}");
}

#[tokio::test]
async fn path_prefix_instance_is_reachable() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::path("/gitlab/api/v4/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "username": "u"
        })))
        .expect(1..)
        .mount(&server)
        .await;
    let _ = first_error_or_none(format!("{}/gitlab", host(&server))).await;
}

async fn first_error_or_none(host: String) {
    let client = GitlabBuilder::new(host, TOKEN)
        .insecure()
        .build_async()
        .await
        .expect("client");
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    let _ = app.initial_setup().await;
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while app.current_user.is_none() {
            match rx.recv().await {
                Some(u) => app.process_update_single(u).await,
                None => break,
            }
        }
    })
    .await;
    assert!(app.current_user.is_some(), "prefixed API path was not used");
}

#[test]
fn certificate_errors_point_at_insecure() {
    let msg = friendly_error("client error: invalid peer certificate: UnknownIssuer");
    assert!(msg.contains("--insecure"), "{msg}");
}

#[test]
fn unknown_errors_pass_through_unchanged() {
    assert_eq!(
        friendly_error("Merge failed: conflict"),
        "Merge failed: conflict"
    );
}

#[test]
fn hint_redacts_token_in_original_message() {
    register_secret(TOKEN);
    let msg = friendly_error(&format!("gitlab server error (401 Unauthorized): {TOKEN}"));
    assert!(!msg.contains(TOKEN), "{msg}");
}

#[test]
fn partial_diff_position_does_not_panic() {
    use gitlab::api::Endpoint;
    let ep = lazygitlab::CreateMergeRequestDiscussion {
        project: 1,
        merge_request: 2,
        body: "hi".into(),
        base_sha: Some("b".into()),
        start_sha: None,
        head_sha: None,
        new_path: None,
        old_path: None,
        new_line: Some(3),
        old_line: None,
        range_start_new: None,
        range_start_old: None,
    };
    assert!(ep.body().is_ok());
}
