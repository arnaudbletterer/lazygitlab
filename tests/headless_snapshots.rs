use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

use gitlab::GitlabBuilder;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

use lazygitlab::headless::{render_to_string, ScriptedEventSource};
use lazygitlab::models::{App, DataUpdate};
use lazygitlab::run_app;

async fn wait_for_loading_finish(app: &mut App, rx: &mut mpsc::UnboundedReceiver<DataUpdate>) {
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        app.process_updates(rx).await;
        if !app.is_loading {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn test_headless_dashboard_initial() {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1, "username": "u"})))
        .mount(&server)
        .await;

    let client = GitlabBuilder::new(server.uri().replace("http://", ""), "t")
        .insecure()
        .build_async()
        .await
        .unwrap();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx);

    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let event_source = ScriptedEventSource::from_text("q").unwrap();
    let frames: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let frames_for_capture = Arc::clone(&frames);
    let mut capture: Option<lazygitlab::runner::FrameCaptureFn<TestBackend>> = Some(Box::new(move |t| {
        frames_for_capture.lock().unwrap().push(render_to_string(t));
    }));

    let _ = run_app(&mut terminal, &mut app, &event_source, &mut rx, &mut capture).await;
    wait_for_loading_finish(&mut app, &mut rx).await;

    let frames = frames.lock().unwrap();
    assert!(!frames.is_empty(), "expected at least one captured frame");
    let frame = frames.last().unwrap();
    insta::assert_snapshot!("headless_dashboard_initial", frame);
}

#[tokio::test]
async fn test_headless_dashboard_after_navigation() {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1, "username": "u"})))
        .mount(&server)
        .await;

    let client = GitlabBuilder::new(server.uri().replace("http://", ""), "t")
        .insecure()
        .build_async()
        .await
        .unwrap();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx);

    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let event_source = ScriptedEventSource::from_text("g\nm\nq").unwrap();
    let frames: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let frames_for_capture = Arc::clone(&frames);
    let mut capture: Option<lazygitlab::runner::FrameCaptureFn<TestBackend>> = Some(Box::new(move |t| {
        frames_for_capture.lock().unwrap().push(render_to_string(t));
    }));

    let _ = run_app(&mut terminal, &mut app, &event_source, &mut rx, &mut capture).await;

    let frames = frames.lock().unwrap();
    assert!(!frames.is_empty(), "expected at least one captured frame");
    let last_frame = frames.last().unwrap();
    insta::assert_snapshot!("headless_dashboard_after_navigation", last_frame);
}
