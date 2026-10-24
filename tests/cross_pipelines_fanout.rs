use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

/// Records the server-side interval [arrival, arrival + delay) of every
/// request; concurrency is computed afterwards from the intervals, so it does
/// not depend on timer jitter.
struct Tracked {
    spans: Arc<Mutex<Vec<(Instant, Instant)>>>,
    delay: Duration,
    /// Projects with id <= fast_ids answer immediately.
    fast_ids: u64,
}

impl Respond for Tracked {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        let pid: u64 = req
            .url
            .path()
            .split('/')
            .nth(4)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let delay = if pid <= self.fast_ids {
            Duration::ZERO
        } else {
            self.delay
        };
        let now = Instant::now();
        self.spans.lock().unwrap().push((now, now + delay));
        ResponseTemplate::new(200)
            .set_delay(delay)
            .set_body_json(serde_json::json!([{
                "id": pid, "project_id": pid, "status": "failed", "source": "push",
                "ref": "main", "web_url": "u",
                "created_at": format!("2026-01-01T00:00:{:02}Z", pid % 60)
            }]))
    }
}

fn max_overlap(spans: &[(Instant, Instant)]) -> usize {
    let mut events: Vec<(Instant, i32)> = Vec::new();
    for (s, e) in spans {
        events.push((*s, 1));
        events.push((*e, -1));
    }
    // Ends sort before starts at the same instant (half-open intervals).
    events.sort_by_key(|(t, d)| (*t, *d));
    let (mut cur, mut max) = (0i32, 0i32);
    for (_, d) in events {
        cur += d;
        max = max.max(cur);
    }
    max as usize
}

struct Setup {
    _server: MockServer,
    app: App,
    rx: tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
    spans: Arc<Mutex<Vec<(Instant, Instant)>>>,
}

async fn setup(n: u64, delay_ms: u64) -> Setup {
    setup_with(n, delay_ms, 0).await
}

async fn setup_with(n: u64, delay_ms: u64, fast_ids: u64) -> Setup {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"})),
        )
        .mount(&server)
        .await;
    let spans = Arc::new(Mutex::new(Vec::new()));
    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v4/projects/\d+/pipelines$"))
        .respond_with(Tracked {
            spans: spans.clone(),
            delay: Duration::from_millis(delay_ms),
            fast_ids,
        })
        .mount(&server)
        .await;
    let client = GitlabBuilder::new(server.uri().replace("http://", ""), "t")
        .insecure()
        .build_async()
        .await
        .unwrap();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.member_project_ids = (1..=n).collect();
    Setup {
        _server: server,
        app,
        rx,
        spans,
    }
}

/// Waits for the next `Loading(false)` and returns the last
/// `GlobalPipelines` payload seen up to and including that point.
async fn next_done(rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) -> Vec<Pipeline> {
    let mut last = Vec::new();
    loop {
        match tokio::time::timeout(Duration::from_secs(60), rx.recv())
            .await
            .expect("timeout")
            .expect("closed")
        {
            DataUpdate::GlobalPipelines { data, .. } => last = data,
            DataUpdate::Loading(false) => return last,
            _ => continue,
        }
    }
}

#[tokio::test]
async fn fanout_is_bounded_and_refreshes_do_not_overlap() {
    let mut s = setup(200, 200).await;
    s.app.fetch_cross_pipelines().await.unwrap();
    // Further refreshes while the first is in flight must send nothing.
    tokio::time::sleep(Duration::from_millis(100)).await;
    s.app.fetch_cross_pipelines().await.unwrap();
    s.app.fetch_cross_pipelines().await.unwrap();
    let data = next_done(&mut s.rx).await;
    assert_eq!(data.len(), 50);
    assert_eq!(s.spans.lock().unwrap().len(), 200);
    let max = max_overlap(&s.spans.lock().unwrap());
    println!("max concurrency observed: {max}");
    assert!(max <= 8, "max concurrency {max}");
    assert!(max >= 2, "expected some parallelism, got {max}");
    // Once finished (Loading(false) seen), a new refresh is allowed again.
    s.app.fetch_cross_pipelines().await.unwrap();
    let _ = next_done(&mut s.rx).await;
    assert_eq!(s.spans.lock().unwrap().len(), 400);
}

#[tokio::test]
async fn five_projects_render_same_list() {
    use lazygitlab::ui::ui;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    let mut s = setup(5, 0).await;
    s.app.active_context = Context::Dashboard;
    s.app.active_selection = MenuSelection::Pipelines;
    s.app.fetch_cross_pipelines().await.unwrap();
    let data = next_done(&mut s.rx).await;
    let ids: Vec<u64> = data.iter().map(|p| p.id).collect();
    // Newest created_at first (seconds = id % 60).
    assert_eq!(ids, vec![5, 4, 3, 2, 1]);
    s.app
        .process_update_single(DataUpdate::GlobalPipelines {
            data,
            page: 1,
            has_more: false,
        })
        .await;
    assert_eq!(s.app.filtered_pipelines.len(), 5);
    s.app.pipeline_filter = PipelineFilter::Failed;
    s.app.filter_lists();
    assert_eq!(s.app.filtered_pipelines.len(), 5);
    s.app.pipeline_filter = PipelineFilter::Running;
    s.app.filter_lists();
    assert!(s.app.filtered_pipelines.is_empty());
    s.app.pipeline_filter = PipelineFilter::All;
    s.app.filter_lists();

    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal.draw(|f| ui(f, &mut s.app)).unwrap();
    let buf = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            out.push(buf[(x, y)].symbol().chars().next().unwrap_or(' '));
        }
        out.push('\n');
    }
    for id in 1..=5 {
        assert!(out.contains(&id.to_string()), "pipeline {id} not rendered");
    }
}

#[tokio::test]
async fn manual_refresh_during_flight_runs_exactly_one_more() {
    let mut s = setup(20, 100).await;
    s.app.fetch_cross_pipelines().await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;
    // Several manual refreshes while in flight collapse into one rerun.
    s.app.fetch_cross_pipelines_manual().await.unwrap();
    s.app.fetch_cross_pipelines_manual().await.unwrap();
    assert!(
        s.app
            .status_message
            .as_deref()
            .is_some_and(|m| m.contains("will refresh again"))
    );
    let data = next_done(&mut s.rx).await;
    assert_eq!(data.len(), 20);
    assert_eq!(s.spans.lock().unwrap().len(), 40);
    assert!(max_overlap(&s.spans.lock().unwrap()) <= 8);
}

#[tokio::test]
async fn deadline_abandons_slow_projects_and_releases_guard() {
    // Projects 1..=3 answer at once, the rest hang for 10 s.
    let mut s = setup_with(20, 10_000, 3).await;
    s.app.cross_pipelines_deadline = Duration::from_millis(500);
    let t0 = Instant::now();
    s.app.fetch_cross_pipelines().await.unwrap();
    let data = next_done(&mut s.rx).await;
    assert!(
        t0.elapsed() < Duration::from_secs(5),
        "deadline not applied"
    );
    let mut ids: Vec<u64> = data.iter().map(|p| p.id).collect();
    ids.sort_unstable();
    assert_eq!(ids, vec![1, 2, 3]);
    // Guard released: a new refresh is accepted.
    let before = s.spans.lock().unwrap().len();
    s.app.fetch_cross_pipelines().await.unwrap();
    let _ = next_done(&mut s.rx).await;
    assert!(s.spans.lock().unwrap().len() > before);
}
