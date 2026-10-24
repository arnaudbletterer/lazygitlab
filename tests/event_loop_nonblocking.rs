use std::time::{Duration, Instant};

use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use tokio::sync::mpsc;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const DELAY: Duration = Duration::from_secs(2);
const BOUND: Duration = Duration::from_millis(100);

async fn setup() -> (MockServer, App, mpsc::UnboundedReceiver<DataUpdate>) {
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
    let (tx, rx) = mpsc::unbounded_channel();
    (server, App::new(client, tx, None), rx)
}

fn todo(id: u64) -> serde_json::Value {
    serde_json::json!({"id": id, "action_name": "mentioned", "target_type": "Issue", "target_url": "http://x/issues/1", "body": "b", "state": "pending", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z", "project": {"id": 1, "name": "p", "path_with_namespace": "g/p"}, "author": {"id": 2, "username": "a"}})
}

#[tokio::test]
async fn todos_poll_does_not_block_ui_task() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/todos"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(DELAY)
                .set_body_json(vec![todo(1)])
                .append_header("ETag", "\"w-1\""),
        )
        .mount(&server)
        .await;

    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::Todos;
    app.view_mode = ViewMode::List;
    app.active_panel = Panel::Main;
    app.todos_etag = Some("\"w-0\"".into());
    // Pretend membership sync already ran so only the todos poll is under test.
    app.membership_sync_attempted = true;

    let t = Instant::now();
    app.refresh_background().await;
    assert!(
        t.elapsed() < BOUND,
        "refresh_background took {:?}",
        t.elapsed()
    );

    let t = Instant::now();
    app.handle_key(crossterm::event::KeyEvent::from(
        crossterm::event::KeyCode::Char('j'),
    ))
    .await;
    assert!(t.elapsed() < BOUND, "key handling took {:?}", t.elapsed());

    // A second refresh while the poll is pending must not start another request.
    app.refresh_background().await;

    let start = Instant::now();
    while app.todos_etag.as_deref() != Some("\"w-1\"") && start.elapsed() < Duration::from_secs(10)
    {
        app.process_updates(&mut rx).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(app.todos_etag.as_deref(), Some("\"w-1\""));
    assert_eq!(app.todos.len(), 1);
    assert!(!app.todos_poll_in_flight);
    let polls = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path() == "/api/v4/todos")
        .count();
    assert_eq!(polls, 1, "only one poll may be in flight");
}

#[tokio::test]
async fn opening_job_log_does_not_block_ui_task() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("GET"))
        .and(path_regex(r"/jobs/\d+/trace$"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(DELAY)
                .set_body_string("hello log\n"),
        )
        .mount(&server)
        .await;

    let job: Job = serde_json::from_value(serde_json::json!({
        "id": 7, "name": "build", "stage": "test", "status": "success", "created_at": "2026-01-01T00:00:00Z", "allow_failure": false, "web_url": "http://x", "tag_list": []
    }))
    .unwrap();
    let project: Project = serde_json::from_value(serde_json::json!({
        "id": 1, "name": "p", "path_with_namespace": "g/p"
    }))
    .unwrap();
    app.selected_project = Some(project);
    app.jobs = vec![job];
    app.jobs_state.select(Some(0));

    let t = Instant::now();
    app.open_selected_job().await;
    assert!(t.elapsed() < BOUND, "open took {:?}", t.elapsed());
    assert_eq!(app.view_mode, ViewMode::JobLog);

    let start = Instant::now();
    while app.job_log.is_empty() && start.elapsed() < Duration::from_secs(10) {
        app.process_updates(&mut rx).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(app.job_log.contains("hello log"));
}

fn pipeline(id: u64) -> Pipeline {
    serde_json::from_value(serde_json::json!({
        "id": id, "project_id": 1, "status": "success", "source": "push",
        "ref": "main", "web_url": "http://x", "created_at": "2026-01-01T00:00:00Z"
    }))
    .unwrap()
}

fn job() -> Job {
    serde_json::from_value(serde_json::json!({
        "id": 7, "name": "build", "stage": "test", "status": "success",
        "created_at": "2026-01-01T00:00:00Z", "allow_failure": false,
        "web_url": "http://x", "tag_list": []
    }))
    .unwrap()
}

#[tokio::test]
async fn pending_job_open_only_fires_for_its_own_pipeline() {
    let (_server, mut app, _rx) = setup().await;
    app.active_selection = MenuSelection::Pipelines;
    app.view_mode = ViewMode::List;
    app.selected_pipeline = Some(pipeline(6));
    app.open_job_when_loaded = Some((1, 5));

    // Jobs for a different pipeline must not open a log.
    app.process_update_single(DataUpdate::Jobs {
        project_id: 1,
        pipeline_id: 6,
        data: vec![job()],
    })
    .await;
    assert_eq!(app.view_mode, ViewMode::List);
    assert_eq!(app.open_job_when_loaded, None);

    // go_back cancels a pending open.
    app.selected_pipeline = Some(pipeline(5));
    app.open_job_when_loaded = Some((1, 5));
    app.go_back().await;
    app.process_update_single(DataUpdate::Jobs {
        project_id: 1,
        pipeline_id: 5,
        data: vec![job()],
    })
    .await;
    assert_eq!(app.view_mode, ViewMode::List);
}
