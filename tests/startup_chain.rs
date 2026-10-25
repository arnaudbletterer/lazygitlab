use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent};
use lazygitlab::client::build_deferred;
use lazygitlab::config::Transport;
use lazygitlab::models::*;
use lazygitlab::runner::{FrameCaptureFn, run_app};
use lazygitlab::{EventSource, cache};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use tokio::sync::mpsc;
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const LATENCY: Duration = Duration::from_millis(100);
const HOST: &str = "gitlab.startup-chain.test";

type Log = Arc<Mutex<Vec<(String, Instant)>>>;

struct Fixture {
    server: MockServer,
    log: Log,
}

fn issue(project: u64) -> serde_json::Value {
    serde_json::json!({"iid": 1, "project_id": project, "title": "Startup chain issue", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"})
}

async fn fixture(member_ids: &'static [u64], list_latency: Duration) -> Fixture {
    let server = MockServer::start().await;
    let log: Log = Arc::default();
    let seen = log.clone();
    Mock::given(wiremock::matchers::any())
        .respond_with(move |req: &Request| {
            let path = req.url.path().to_string();
            seen.lock().unwrap().push((path.clone(), Instant::now()));
            let (body, delay): (serde_json::Value, Duration) = if path == "/api/v4/user" {
                (serde_json::json!({"id": 1, "username": "u"}), LATENCY)
            } else if path == "/api/v4/projects" {
                let ids: Vec<_> = member_ids.iter().map(|id| serde_json::json!({"id": id, "name": "p", "path_with_namespace": "g/p"})).collect();
                (serde_json::Value::Array(ids), LATENCY)
            } else if path.ends_with("/issues") {
                let id = path.split('/').nth(4).and_then(|s| s.parse().ok()).unwrap_or(0);
                (serde_json::json!([issue(id)]), list_latency)
            } else {
                (serde_json::json!([]), LATENCY)
            };
            ResponseTemplate::new(200).set_body_json(body).set_delay(delay)
        })
        .mount(&server)
        .await;
    Fixture { server, log }
}

impl Fixture {
    fn first(&self, pred: impl Fn(&str) -> bool) -> Option<Instant> {
        self.log
            .lock()
            .unwrap()
            .iter()
            .find(|(p, _)| pred(p))
            .map(|(_, t)| *t)
    }

    fn count(&self, pred: impl Fn(&str) -> bool) -> usize {
        self.log
            .lock()
            .unwrap()
            .iter()
            .filter(|(p, _)| pred(p))
            .count()
    }
}

fn is_user(p: &str) -> bool {
    p == "/api/v4/user"
}
fn is_membership(p: &str) -> bool {
    p == "/api/v4/projects"
}
fn is_project_issues(p: &str) -> bool {
    p.starts_with("/api/v4/projects/") && p.ends_with("/issues")
}
fn is_project_pipelines(p: &str) -> bool {
    p.starts_with("/api/v4/projects/") && p.ends_with("/pipelines")
}

async fn new_app(fx: &Fixture) -> (App, mpsc::UnboundedReceiver<DataUpdate>) {
    let client = build_deferred(
        fx.server.uri().replace("http://", ""),
        "t",
        Transport::PlainHttp,
    )
    .await
    .unwrap();
    let (tx, rx) = mpsc::unbounded_channel();
    (App::new(client, tx, None), rx)
}

struct QuitAfter {
    until: Instant,
}

impl EventSource for QuitAfter {
    async fn next_event(&self, timeout: Duration) -> std::io::Result<Option<Event>> {
        let left = self.until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Ok(Some(Event::Key(KeyEvent::from(KeyCode::Char('q')))));
        }
        tokio::time::sleep(timeout.min(left)).await;
        Ok(None)
    }
}

#[derive(Default)]
struct Frames {
    times: Vec<(Instant, String)>,
}

async fn run(
    app: &mut App,
    rx: &mut mpsc::UnboundedReceiver<DataUpdate>,
    run_for: Duration,
) -> Frames {
    let frames = Arc::new(Mutex::new(Frames::default()));
    let sink = frames.clone();
    let mut capture: Option<FrameCaptureFn<TestBackend>> = Some(Box::new(move |t| {
        sink.lock()
            .unwrap()
            .times
            .push((Instant::now(), lazygitlab::headless::render_to_string(t)));
    }));
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    let source = QuitAfter {
        until: Instant::now() + run_for,
    };
    run_app(&mut terminal, app, &source, rx, &mut capture)
        .await
        .unwrap();
    drop(capture);
    Arc::try_unwrap(frames).ok().unwrap().into_inner().unwrap()
}

fn open_dashboard(app: &mut App, selection: MenuSelection) {
    app.active_context = Context::Dashboard;
    app.active_selection = selection;
    app.view_mode = ViewMode::List;
    app.active_panel = Panel::Main;
}

#[tokio::test]
async fn first_frame_precedes_every_request_and_user_is_fetched_once() {
    let fx = fixture(&[1, 2], LATENCY).await;
    let (mut app, mut rx) = new_app(&fx).await;
    let frames = run(&mut app, &mut rx, Duration::from_millis(900)).await;

    let first_request = fx
        .log
        .lock()
        .unwrap()
        .iter()
        .map(|(_, t)| *t)
        .min()
        .unwrap();
    assert!(
        frames.times[0].0 < first_request,
        "first frame must precede the first request"
    );
    assert_eq!(fx.count(is_user), 1, "/user must be requested exactly once");

    let user = fx.first(is_user).unwrap();
    let membership = fx.first(is_membership).unwrap();
    let gap = if user > membership {
        user - membership
    } else {
        membership - user
    };
    assert!(
        gap < Duration::from_millis(20),
        "user and membership started {gap:?} apart"
    );
}

#[tokio::test]
async fn warm_cache_starts_the_first_list_fetch_before_membership_answers() {
    let dir = std::env::temp_dir().join(format!("lgl-startup-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    cache::save_membership(&dir, HOST, 1, &[1u64, 2].into_iter().collect()).unwrap();
    let fx = fixture(&[1, 2], LATENCY).await;
    let (mut app, mut rx) = new_app(&fx).await;
    app.enable_membership_cache(dir.clone(), HOST);
    open_dashboard(&mut app, MenuSelection::Issues);
    run(&mut app, &mut rx, Duration::from_millis(600)).await;

    let membership_asked = fx.first(is_membership).unwrap();
    let list = fx.first(is_project_issues).expect("issues were fetched");
    assert!(
        list < membership_asked + LATENCY,
        "the list fetch must not wait for the membership response"
    );
}

#[tokio::test]
async fn screen_opened_before_ids_arrive_fetches_when_they_do() {
    let fx = fixture(&[1, 2], LATENCY).await;
    let (mut app, mut rx) = new_app(&fx).await;
    open_dashboard(&mut app, MenuSelection::Pipelines);
    let started = Instant::now();
    run(&mut app, &mut rx, Duration::from_millis(900)).await;

    let ids_arrive = fx.first(is_membership).unwrap() + LATENCY;
    let fetch = fx
        .first(is_project_pipelines)
        .expect("pipelines were fetched");
    assert!(fetch >= ids_arrive - Duration::from_millis(5));
    assert!(
        fetch - ids_arrive < Duration::from_millis(100),
        "pipelines fetch started {:?} after the ids",
        fetch - ids_arrive
    );
    assert!(started.elapsed() < Duration::from_secs(4));
}

#[tokio::test]
async fn results_are_applied_as_soon_as_they_arrive() {
    let fx = fixture(&[1], LATENCY).await;
    let (mut app, mut rx) = new_app(&fx).await;
    open_dashboard(&mut app, MenuSelection::Issues);
    let frames = run(&mut app, &mut rx, Duration::from_millis(900)).await;

    let asked = fx.first(is_project_issues).expect("issues were fetched");
    let shown = frames
        .times
        .iter()
        .find(|(_, text)| text.contains("Startup chain issue"))
        .map(|(t, _)| *t)
        .expect("the issue was drawn");
    let lag = shown.saturating_duration_since(asked + LATENCY);
    assert!(
        lag < Duration::from_millis(20),
        "result drawn {lag:?} after it arrived"
    );
}

#[tokio::test]
async fn refreshes_never_overlap_for_the_same_list() {
    let fx = fixture(&[1, 2, 3], Duration::from_millis(400)).await;
    let (mut app, _rx) = new_app(&fx).await;
    app.member_project_ids = [1, 2, 3].into_iter().collect();
    open_dashboard(&mut app, MenuSelection::Issues);

    app.refresh().await;
    app.refresh_background().await;
    app.refresh().await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert_eq!(
        fx.count(is_project_issues),
        3,
        "one request per project, not one per refresh"
    );
}

#[test]
fn membership_cache_round_trips_and_ignores_other_hosts() {
    let dir = std::env::temp_dir().join(format!("lgl-membership-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    assert!(cache::load_membership(&dir, HOST).is_none());
    cache::save_membership(&dir, HOST, 7, &[3u64, 1].into_iter().collect()).unwrap();
    let record = cache::load_membership(&dir, HOST).unwrap();
    assert_eq!((record.user_id, record.ids), (7, vec![1, 3]));
    assert!(cache::load_membership(&dir, "other.example").is_none());
    let _ = std::fs::remove_dir_all(&dir);
}
