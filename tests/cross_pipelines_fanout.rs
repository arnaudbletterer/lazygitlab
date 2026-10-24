use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

struct Tracked {
    current: Arc<AtomicUsize>,
    max: Arc<AtomicUsize>,
    total: Arc<AtomicUsize>,
    delay: Duration,
}

impl Respond for Tracked {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        // respond() runs when the request arrives; the response is held back
        // by `delay`, so decrement the in-flight counter after that delay.
        let now = self.current.fetch_add(1, Ordering::SeqCst) + 1;
        self.max.fetch_max(now, Ordering::SeqCst);
        self.total.fetch_add(1, Ordering::SeqCst);
        let cur = self.current.clone();
        // Release slightly early to absorb timer jitter between server and client.
        let d = self.delay.saturating_sub(Duration::from_millis(40));
        tokio::spawn(async move {
            tokio::time::sleep(d).await;
            cur.fetch_sub(1, Ordering::SeqCst);
        });
        let pid: u64 = req
            .url
            .path()
            .split('/')
            .nth(4)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        ResponseTemplate::new(200)
            .set_delay(self.delay)
            .set_body_json(serde_json::json!([{
                "id": pid, "project_id": pid, "status": "failed", "source": "push",
                "ref": "main", "web_url": "u",
                "created_at": format!("2026-01-01T00:00:{:02}Z", pid % 60)
            }]))
    }
}

struct Setup {
    _server: MockServer,
    app: App,
    rx: tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
    max: Arc<AtomicUsize>,
    total: Arc<AtomicUsize>,
}

async fn setup(n: u64, delay_ms: u64) -> Setup {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"})),
        )
        .mount(&server)
        .await;
    let max = Arc::new(AtomicUsize::new(0));
    let total = Arc::new(AtomicUsize::new(0));
    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v4/projects/\d+/pipelines$"))
        .respond_with(Tracked {
            current: Arc::new(AtomicUsize::new(0)),
            max: max.clone(),
            total: total.clone(),
            delay: Duration::from_millis(delay_ms),
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
        max,
        total,
    }
}

async fn next_global(rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) -> Vec<Pipeline> {
    loop {
        match tokio::time::timeout(Duration::from_secs(60), rx.recv())
            .await
            .expect("timeout")
            .expect("closed")
        {
            DataUpdate::GlobalPipelines { data, .. } => return data,
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
    let data = next_global(&mut s.rx).await;
    assert_eq!(data.len(), 50);
    assert_eq!(s.total.load(Ordering::SeqCst), 200);
    let max = s.max.load(Ordering::SeqCst);
    println!("max concurrency observed: {max}");
    assert!(max <= 8, "max concurrency {max}");
    assert!(max >= 2, "expected some parallelism, got {max}");
    // Once finished, a new refresh is allowed again.
    tokio::time::sleep(Duration::from_millis(50)).await;
    s.app.fetch_cross_pipelines().await.unwrap();
    let _ = next_global(&mut s.rx).await;
    assert_eq!(s.total.load(Ordering::SeqCst), 400);
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
    let data = next_global(&mut s.rx).await;
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
    let _ = (&s.max, &s.total);
}
