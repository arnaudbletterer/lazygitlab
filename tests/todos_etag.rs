use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_app() -> (
    MockServer,
    App,
    tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
) {
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
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    (server, App::new(client, tx, None), rx)
}

fn todo(id: u64) -> serde_json::Value {
    serde_json::json!({"id": id, "action_name": "mentioned", "target_type": "Issue", "target_url": "http://x/issues/1", "body": "b", "state": "pending", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z", "project": {"id": 1, "name": "p", "path_with_namespace": "g/p"}, "author": {"id": 2, "username": "a"}})
}

async fn poll_once(app: &mut App, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
    app.spawn_todos_poll();
    let start = std::time::Instant::now();
    while app.todos_poll_in_flight && start.elapsed() < std::time::Duration::from_secs(10) {
        app.process_updates(rx).await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(!app.todos_poll_in_flight, "poll did not finish");
}

#[tokio::test]
async fn test_etag_conditional_poll_sends_if_none_match_and_304_is_noop() {
    let (server, mut app, mut rx) = setup_app().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/todos"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(vec![todo(1)])
                .append_header("ETag", "\"w-abc123\""),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/todos"))
        .and(wiremock::matchers::header("if-none-match", "\"w-abc123\""))
        .respond_with(ResponseTemplate::new(304))
        .named("conditional_304")
        .mount(&server)
        .await;

    // First poll: no ETag yet -> full fetch stores the ETag
    poll_once(&mut app, &mut rx).await;
    assert_eq!(
        app.todos_etag.as_deref(),
        Some("\"w-abc123\""),
        "ETag stored from first response"
    );

    // Second poll: sends If-None-Match; server answers 304; nothing to apply
    poll_once(&mut app, &mut rx).await;
    assert_eq!(app.todos_etag.as_deref(), Some("\"w-abc123\""));

    let requests = server.received_requests().await.unwrap();
    let conditional = requests
        .iter()
        .filter(|r| r.headers.get("if-none-match").is_some())
        .count();
    assert_eq!(conditional, 1, "exactly one request carried If-None-Match");
    assert!(
        requests.iter().any(|r| r.url.path().ends_with("/todos")),
        "polls hit /todos"
    );
}

#[tokio::test]
async fn test_stale_poll_after_filter_change_is_dropped() {
    let (server, mut app, mut rx) = setup_app().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/todos"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_millis(300))
                .set_body_json(vec![todo(1)])
                .append_header("ETag", "\"w-pending\""),
        )
        .mount(&server)
        .await;

    app.spawn_todos_poll();
    app.todo_filter = TodoFilter::Done;
    let start = std::time::Instant::now();
    while app.todos_poll_in_flight && start.elapsed() < std::time::Duration::from_secs(10) {
        app.process_updates(&mut rx).await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(app.todos.is_empty(), "stale pending result must be dropped");
    assert_eq!(app.todos_etag, None, "pending ETag must not be stored under Done");
}
