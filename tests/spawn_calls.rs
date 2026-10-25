use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use tokio::sync::mpsc;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

const DELAY: Duration = Duration::from_secs(2);
const BOUND: Duration = Duration::from_millis(50);

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

fn pipeline(project: u64, id: u64) -> Pipeline {
    serde_json::from_value(pipeline_json(project, id)).unwrap()
}

fn pipeline_json(project: u64, id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id, "project_id": project, "status": "running", "source": "push",
        "ref": "main", "web_url": "http://x", "created_at": "2026-01-01T00:00:00Z"
    })
}

fn job(id: u64, status: &str) -> Job {
    serde_json::from_value(serde_json::json!({
        "id": id, "name": "build", "stage": "test", "status": status,
        "created_at": "2026-01-01T00:00:00Z", "allow_failure": false,
        "web_url": "http://x", "tag_list": []
    }))
    .unwrap()
}

fn project(id: u64) -> Project {
    serde_json::from_value(serde_json::json!({"id": id, "name": "p"})).unwrap()
}

fn mr(iid: u64) -> MergeRequest {
    serde_json::from_value(serde_json::json!({
        "iid": iid, "project_id": 1, "title": "t", "author": {"id": 1, "username": "u"},
        "web_url": "u", "state": "opened", "source_branch": "a", "target_branch": "b",
        "draft": false, "created_at": "2026-01-01T00:00:00Z",
        "diff_refs": {"base_sha": "b", "head_sha": "h", "start_sha": "s"}
    }))
    .unwrap()
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

async fn pump_until(
    app: &mut App,
    rx: &mut mpsc::UnboundedReceiver<DataUpdate>,
    limit: Duration,
    mut done: impl FnMut(&App) -> bool,
) -> bool {
    let start = Instant::now();
    while start.elapsed() < limit {
        while let Ok(update) = rx.try_recv() {
            app.process_update_single(update).await;
        }
        if done(app) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    false
}

fn pipelines_screen(app: &mut App) {
    app.active_context = Context::Dashboard;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Pipelines;
    app.view_mode = ViewMode::List;
    app.filtered_pipelines = vec![pipeline(1, 9)];
    app.pipelines_state.select(Some(0));
}

async fn mock_slow_post(server: &MockServer, p: &str, body: serde_json::Value) {
    Mock::given(method("POST"))
        .and(path(p))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(DELAY)
                .set_body_json(body),
        )
        .mount(server)
        .await;
}

#[tokio::test]
async fn cancel_pipeline_key_returns_fast_and_reports_later() {
    let (server, mut app, mut rx) = setup().await;
    mock_slow_post(
        &server,
        "/api/v4/projects/1/pipelines/9/cancel",
        pipeline_json(1, 9),
    )
    .await;
    pipelines_screen(&mut app);

    let t = Instant::now();
    app.handle_key(key(KeyCode::Char('C'))).await;
    assert!(t.elapsed() < BOUND, "C blocked for {:?}", t.elapsed());

    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| a
            .status_message
            .as_deref()
            .is_some_and(|m| m.contains("cancelled")))
        .await,
        "no result, status {:?}",
        app.status_message
    );
}

#[tokio::test]
async fn retry_pipeline_key_returns_fast_and_reports_later() {
    let (server, mut app, mut rx) = setup().await;
    mock_slow_post(
        &server,
        "/api/v4/projects/1/pipelines/9/retry",
        pipeline_json(1, 9),
    )
    .await;
    pipelines_screen(&mut app);

    let t = Instant::now();
    app.handle_key(key(KeyCode::Char('R'))).await;
    assert!(t.elapsed() < BOUND, "R blocked for {:?}", t.elapsed());

    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| a
            .status_message
            .as_deref()
            .is_some_and(|m| m.contains("retry triggered")))
        .await,
        "no result, status {:?}",
        app.status_message
    );
}

#[tokio::test]
async fn failed_mutation_shows_error_and_releases_loading() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/1/pipelines/9/cancel"))
        .respond_with(
            ResponseTemplate::new(403)
                .set_delay(DELAY)
                .set_body_json(serde_json::json!({"message": "403 Forbidden"})),
        )
        .mount(&server)
        .await;
    pipelines_screen(&mut app);

    let t = Instant::now();
    app.handle_key(key(KeyCode::Char('C'))).await;
    assert!(t.elapsed() < BOUND, "C blocked for {:?}", t.elapsed());
    assert!(app.last_error.is_none());
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| a
            .last_error
            .as_deref()
            .is_some_and(|e| e.contains("Cancel failed")))
        .await,
        "no error, last_error {:?}",
        app.last_error
    );
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(2), |a| !a.is_loading).await,
        "loading leaked"
    );
    assert!(
        app.status_message
            .as_deref()
            .is_some_and(|m| m.contains("Cancel failed"))
    );
}

#[tokio::test]
async fn delete_branch_popup_confirm_returns_fast_and_refreshes_list() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v4/projects/1/repository/branches/feat"))
        .respond_with(ResponseTemplate::new(204).set_delay(DELAY))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/repository/branches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"name": "main", "merged": false, "protected": false, "default": true, "web_url": "u"}
        ])))
        .mount(&server)
        .await;
    app.selected_project = Some(project(1));
    app.project_branches = ["main", "feat"]
        .iter()
        .map(|n| Branch {
            name: n.to_string(),
            merged: None,
            protected: None,
            default: None,
            web_url: None,
        })
        .collect();
    app.show_confirm_delete_branch_popup = true;
    app.pending_delete_branch = Some((1, "feat".into()));

    let t = Instant::now();
    app.handle_key(key(KeyCode::Char('y'))).await;
    assert!(t.elapsed() < BOUND, "confirm blocked for {:?}", t.elapsed());

    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| a
            .project_branches
            .len()
            == 1)
        .await,
        "branch list not refreshed"
    );
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(2), |a| !a.is_loading).await,
        "loading leaked"
    );
}

#[tokio::test]
async fn create_branch_popup_returns_fast_and_refreshes_list() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/1/repository/branches"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_delay(DELAY)
                .set_body_json(serde_json::json!({"name": "feat"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/repository/branches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"name": "main"}, {"name": "feat"}
        ])))
        .mount(&server)
        .await;
    app.selected_project = Some(project(1));
    app.show_create_branch_popup = true;
    app.input_mode = true;
    app.input_target = InputTarget::CreateBranchRef;
    app.create_branch_name = "feat".into();
    app.textarea = ratatui_textarea::TextArea::from(["main"]);

    let t = Instant::now();
    app.handle_key(ctrl(KeyCode::Enter)).await;
    assert!(t.elapsed() < BOUND, "create blocked for {:?}", t.elapsed());

    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| a
            .project_branches
            .iter()
            .any(|b| b.name == "feat"))
        .await,
        "branch not listed"
    );
}

#[tokio::test]
async fn launch_pipeline_popup_returns_fast_and_reports_later() {
    let (server, mut app, mut rx) = setup().await;
    mock_slow_post(&server, "/api/v4/projects/1/pipeline", pipeline_json(1, 10)).await;
    app.selected_project = Some(project(1));
    app.show_pipeline_popup = true;
    app.input_mode = false;
    app.input_target = InputTarget::PipelineRef;
    app.pipeline_ref_input = "main".into();

    let t = Instant::now();
    app.handle_key(ctrl(KeyCode::Char('r'))).await;
    assert!(t.elapsed() < BOUND, "launch blocked for {:?}", t.elapsed());
    assert!(!app.show_pipeline_popup);

    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| a
            .status_message
            .as_deref()
            .is_some_and(|m| m.contains("launched")))
        .await,
        "no result, status {:?}",
        app.status_message
    );
}

async fn mock_trace(server: &MockServer, project: u64, job: u64, body: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/api/v4/projects/{project}/jobs/{job}/trace")))
        .respond_with(ResponseTemplate::new(200).set_body_string(body.to_string()))
        .mount(server)
        .await;
}

async fn trace_requests(server: &MockServer) -> usize {
    server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path().ends_with("/trace"))
        .count()
}

fn dashboard_job_screen(app: &mut App, status: &str) {
    app.active_context = Context::Dashboard;
    app.active_panel = Panel::MainDetail;
    app.active_selection = MenuSelection::Pipelines;
    app.view_mode = ViewMode::Detail;
    app.selected_project = None;
    app.selected_pipeline = Some(pipeline(5, 3));
    app.jobs = vec![job(7, status)];
    app.jobs_state.select(Some(0));
}

#[tokio::test]
async fn dashboard_pipeline_job_log_loads() {
    let (server, mut app, mut rx) = setup().await;
    mock_trace(&server, 5, 7, "hello\nworld\n").await;
    dashboard_job_screen(&mut app, "success");

    app.open_selected_job().await;
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| !a
            .job_log_lines
            .is_empty())
        .await,
        "dashboard job log never loaded"
    );
    assert!(app.job_log.contains("hello"));
}

#[tokio::test]
async fn finished_job_log_is_downloaded_once() {
    let (server, mut app, mut rx) = setup().await;
    mock_trace(&server, 5, 7, "done\n").await;
    dashboard_job_screen(&mut app, "success");
    app.open_selected_job().await;
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| !a
            .job_log_lines
            .is_empty())
        .await
    );

    for _ in 0..3 {
        app.refresh_background().await;
        pump_until(&mut app, &mut rx, Duration::from_millis(300), |_| false).await;
    }
    assert_eq!(trace_requests(&server).await, 1);
}

#[tokio::test]
async fn running_job_log_has_one_request_in_flight() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/5/jobs/7/trace"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(600))
                .set_body_string("x\n"),
        )
        .mount(&server)
        .await;
    dashboard_job_screen(&mut app, "running");
    app.open_selected_job().await;
    for _ in 0..5 {
        app.fetch_job_log(5, 7).await.unwrap();
    }
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| !a
            .job_log_lines
            .is_empty())
        .await
    );
    pump_until(&mut app, &mut rx, Duration::from_millis(900), |_| false).await;
    assert_eq!(trace_requests(&server).await, 2, "one coalesced refetch");
    app.fetch_job_log(5, 7).await.unwrap();
    pump_until(&mut app, &mut rx, Duration::from_millis(900), |_| false).await;
    assert_eq!(trace_requests(&server).await, 3);
}

#[tokio::test]
async fn overlapping_log_responses_never_shrink_the_log() {
    let (server, mut app, _rx) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/5/jobs/7/trace"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(DELAY)
                .set_body_string("x\n"),
        )
        .mount(&server)
        .await;
    dashboard_job_screen(&mut app, "running");
    app.open_selected_job().await;
    let (generation, k) = (app.job_log_gen, app.job_log_key.unwrap());
    let update = |seq: u64, n: usize| DataUpdate::JobLog {
        project_id: k.0,
        job_id: k.1,
        generation,
        seq,
        raw: "line\n".repeat(n),
        lines: lazygitlab::app::pipelines::parse_job_log(&"line\n".repeat(n)),
        dropped: 0,
    };
    app.process_update_single(update(2, 100)).await;
    app.process_update_single(update(1, 10)).await;
    assert_eq!(app.job_log_lines.len(), 100);

    app.process_update_single(DataUpdate::JobLog {
        project_id: k.0,
        job_id: k.1,
        generation: generation + 1,
        seq: 3,
        raw: "line\n".repeat(5),
        lines: lazygitlab::app::pipelines::parse_job_log(&"line\n".repeat(5)),
        dropped: 0,
    })
    .await;
    assert_eq!(app.job_log_lines.len(), 100, "stale generation applied");
}

#[test]
fn oversized_log_keeps_the_tail_with_a_notice() {
    let line = "0123456789abcdef\n";
    let data = line.repeat(lazygitlab::app::pipelines::JOB_LOG_MAX_BYTES / line.len() + 1000);
    let (kept, dropped) = lazygitlab::app::pipelines::tail_log(data.as_bytes());
    assert!(dropped > 0 && kept.contains(&format!("{dropped} earlier lines dropped")));
    assert!(kept.len() <= lazygitlab::app::pipelines::JOB_LOG_MAX_BYTES + 100);
    assert!(kept.starts_with("[log truncated"));
    assert!(kept.ends_with(line));
}

#[tokio::test]
async fn leaving_the_log_view_releases_its_memory() {
    let (server, mut app, mut rx) = setup().await;
    mock_trace(&server, 5, 7, "hello\n").await;
    dashboard_job_screen(&mut app, "success");
    app.open_selected_job().await;
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| !a
            .job_log_lines
            .is_empty())
        .await
    );
    app.view_mode = ViewMode::Detail;
    app.release_idle_view_state();
    assert!(app.job_log.is_empty() && app.job_log_lines.is_empty());
}

#[derive(Clone, Default)]
struct FileMock {
    hits: Arc<Mutex<Vec<(Instant, String)>>>,
    delay: Duration,
}

impl Respond for FileMock {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        self.hits
            .lock()
            .unwrap()
            .push((Instant::now(), req.url.path().to_string()));
        ResponseTemplate::new(200)
            .set_delay(self.delay)
            .set_body_json(serde_json::json!({
                "file_name": "f", "file_path": "f", "size": 1, "encoding": "base64",
                "content_sha256": "x", "ref": "h", "blob_id": "b", "commit_id": "c",
                "last_commit_id": "l", "content": "aGVsbG8K"
            }))
    }
}

fn diffs(n: usize) -> Vec<MRDiff> {
    (0..n)
        .map(|i| MRDiff {
            old_path: format!("f{i}.txt"),
            new_path: format!("f{i}.txt"),
            diff: String::new(),
            collapsed: true,
            display_lines: vec![],
        })
        .collect()
}

async fn diff_screen(
    n: usize,
    delay: Duration,
) -> (
    MockServer,
    App,
    mpsc::UnboundedReceiver<DataUpdate>,
    FileMock,
) {
    let (server, mut app, rx) = setup().await;
    let files = FileMock {
        delay,
        ..Default::default()
    };
    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v4/projects/1/repository/files/.*"))
        .respond_with(files.clone())
        .mount(&server)
        .await;
    app.selected_mr = Some(mr(1));
    app.view_mode = ViewMode::Diff;
    app.mr_diffs = diffs(n);
    (server, app, rx, files)
}

#[tokio::test]
async fn diff_fan_out_runs_at_most_four_requests_at_once() {
    let delay = Duration::from_millis(300);
    let (_server, mut app, mut rx, files) = diff_screen(12, delay).await;
    app.fetch_all_missing_diffs().await.unwrap();
    pump_until(&mut app, &mut rx, Duration::from_millis(2600), |_| false).await;

    let hits = files.hits.lock().unwrap().clone();
    assert_eq!(hits.len(), 24, "all 12 files fetched (old and new)");
    let max_overlap = hits
        .iter()
        .map(|(t, _)| {
            hits.iter()
                .filter(|(u, _)| *u <= *t && t.duration_since(*u) < delay)
                .count()
        })
        .max()
        .unwrap();
    assert!(max_overlap <= 4, "{max_overlap} requests in flight at once");
}

#[tokio::test]
async fn diff_fan_out_stops_when_the_selected_mr_changes() {
    let delay = Duration::from_millis(300);
    let (_server, mut app, mut rx, files) = diff_screen(40, delay).await;
    app.fetch_all_missing_diffs().await.unwrap();
    pump_until(&mut app, &mut rx, Duration::from_millis(100), |_| false).await;
    app.selected_mr = Some(mr(2));
    app.release_idle_view_state();
    pump_until(&mut app, &mut rx, Duration::from_millis(1500), |_| false).await;
    let n = files.hits.lock().unwrap().len();
    assert!(n <= 8, "{n} requests after the MR changed");
}

#[tokio::test]
async fn big_mr_fetches_at_most_the_cap_automatically() {
    let (_server, mut app, mut rx, files) = diff_screen(500, Duration::ZERO).await;
    app.fetch_all_missing_diffs().await.unwrap();
    pump_until(&mut app, &mut rx, Duration::from_millis(1500), |_| false).await;
    let paths: HashSet<String> = files
        .hits
        .lock()
        .unwrap()
        .iter()
        .map(|(_, p)| p.clone())
        .collect();
    assert!(paths.len() <= 30, "{} files fetched", paths.len());
    assert!(!paths.is_empty());
    assert!(app.mr_diffs.iter().filter(|d| !d.diff.is_empty()).count() <= 30);
}

#[tokio::test]
async fn late_diff_updates_for_another_mr_are_dropped() {
    let (_server, mut app, _rx) = setup().await;
    app.selected_mr = Some(mr(2));
    app.mr_diffs = diffs(1);
    app.process_update_single(DataUpdate::SingleDiffFetched {
        project_id: 1,
        iid: 3,
        old_path: "f0.txt".into(),
        new_path: "f0.txt".into(),
        diff: "+x".into(),
    })
    .await;
    assert!(app.mr_diffs[0].diff.is_empty());

    app.is_fetching_diffs = true;
    app.diffs_fetching_key = Some((1, 2));
    app.process_update_single(DataUpdate::Diffs {
        project_id: 1,
        iid: 3,
        data: diffs(2),
    })
    .await;
    assert!(
        app.is_fetching_diffs,
        "another MR's response cleared the flag"
    );
    assert_eq!(app.mr_diffs.len(), 1);
}

async fn count(server: &MockServer, http_method: &str, p: &str) -> usize {
    server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.method.as_str() == http_method && r.url.path() == p)
        .count()
}

#[tokio::test]
async fn pressing_cancel_twice_sends_one_request() {
    let (server, mut app, mut rx) = setup().await;
    mock_slow_post(
        &server,
        "/api/v4/projects/1/pipelines/9/cancel",
        pipeline_json(1, 9),
    )
    .await;
    pipelines_screen(&mut app);
    app.handle_key(key(KeyCode::Char('C'))).await;
    app.handle_key(key(KeyCode::Char('C'))).await;
    assert!(
        app.status_message
            .as_deref()
            .is_some_and(|m| m.contains("already in progress"))
    );
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| a
            .status_message
            .as_deref()
            .is_some_and(|m| m.contains("cancelled")))
        .await
    );
    assert_eq!(
        count(&server, "POST", "/api/v4/projects/1/pipelines/9/cancel").await,
        1
    );
    app.handle_key(key(KeyCode::Char('C'))).await;
    assert!(
        app.status_message
            .as_deref()
            .is_some_and(|m| m.contains("Cancelling"))
    );
}

#[tokio::test]
async fn rate_limited_mutation_says_it_was_not_sent() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/1/pipelines/9/cancel"))
        .respond_with(ResponseTemplate::new(429).append_header("Retry-After", "30"))
        .mount(&server)
        .await;
    pipelines_screen(&mut app);
    app.handle_key(key(KeyCode::Char('C'))).await;
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| a
            .status_message
            .as_deref()
            .is_some_and(|m| m.contains("Cancel not sent: rate limited")))
        .await,
        "status {:?}",
        app.status_message
    );
    assert!(pump_until(&mut app, &mut rx, Duration::from_secs(2), |a| !a.is_loading).await);
}

#[tokio::test]
async fn launch_sends_ref_and_variables_as_json() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/1/pipeline"))
        .and(wiremock::matchers::body_json(serde_json::json!({
            "ref": "main",
            "variables": [{"key": "K", "value": "V"}]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(pipeline_json(1, 10)))
        .mount(&server)
        .await;
    app.selected_project = Some(project(1));
    app.show_pipeline_popup = true;
    app.pipeline_ref_input = "main".into();
    app.pipeline_variables = vec![PipelineVariable {
        key: "K".into(),
        value: "V".into(),
    }];
    app.handle_key(ctrl(KeyCode::Char('r'))).await;
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| a
            .status_message
            .as_deref()
            .is_some_and(|m| m.contains("launched")))
        .await,
        "status {:?}",
        app.status_message
    );
}

#[tokio::test]
async fn create_branch_sends_branch_and_ref_query() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/1/repository/branches"))
        .and(wiremock::matchers::query_param("branch", "feat/x y"))
        .and(wiremock::matchers::query_param("ref", "main"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({"name": "feat/x y"})),
        )
        .mount(&server)
        .await;
    app.selected_project = Some(project(1));
    app.create_branch(1, "feat/x y".into(), "main".into())
        .await
        .unwrap();
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| a
            .status_message
            .as_deref()
            .is_some_and(|m| m.contains("created")))
        .await,
        "status {:?}",
        app.status_message
    );
}

#[tokio::test]
async fn delete_branch_with_slash_encodes_the_name() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v4/projects/1/repository/branches/feature%2Fx"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    app.delete_branch(1, "feature/x".into()).await.unwrap();
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| a
            .status_message
            .as_deref()
            .is_some_and(|m| m.contains("deleted")))
        .await,
        "status {:?}",
        app.status_message
    );
}

#[tokio::test]
async fn project_switch_during_mutation_keeps_the_new_project_branches() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/1/repository/branches"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_delay(Duration::from_millis(500))
                .set_body_json(serde_json::json!({"name": "feat"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/repository/branches"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([{"name": "feat"}])),
        )
        .mount(&server)
        .await;
    app.selected_project = Some(project(1));
    app.create_branch(1, "feat".into(), "main".into())
        .await
        .unwrap();
    app.selected_project = Some(project(2));
    app.project_branches = vec![Branch {
        name: "other".into(),
        merged: None,
        protected: None,
        default: None,
        web_url: None,
    }];
    pump_until(&mut app, &mut rx, Duration::from_millis(1500), |_| false).await;
    assert_eq!(app.project_branches.len(), 1);
    assert_eq!(app.project_branches[0].name, "other");
}

struct Growing(Mutex<usize>);

impl Respond for Growing {
    fn respond(&self, _: &Request) -> ResponseTemplate {
        let mut n = self.0.lock().unwrap();
        *n += 1;
        let body = if *n == 1 { "a\n" } else { "a\nfinal\n" };
        ResponseTemplate::new(200)
            .set_delay(Duration::from_millis(300))
            .set_body_string(body)
    }
}

#[tokio::test]
async fn log_is_refetched_when_the_job_finishes_during_a_fetch() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/5/jobs/7/trace"))
        .respond_with(Growing(Mutex::new(0)))
        .mount(&server)
        .await;
    dashboard_job_screen(&mut app, "running");
    app.open_selected_job().await;
    app.process_update_single(DataUpdate::Jobs {
        project_id: 5,
        pipeline_id: 3,
        data: vec![job(7, "success")],
    })
    .await;
    assert!(
        pump_until(&mut app, &mut rx, Duration::from_secs(5), |a| a
            .job_log
            .contains("final"))
        .await,
        "final output never loaded: {:?}",
        app.job_log
    );
}

#[tokio::test]
async fn changes_tab_diffs_survive_losing_panel_focus() {
    let (_server, mut app, _rx) = setup().await;
    app.selected_mr = Some(mr(1));
    app.view_mode = ViewMode::List;
    app.active_panel = Panel::Main;
    app.detail_section = DetailSection::Changes;
    app.mr_diffs = diffs(3);
    app.release_idle_view_state();
    assert_eq!(app.mr_diffs.len(), 3);
    app.detail_section = DetailSection::Metadata;
    app.release_idle_view_state();
    assert!(app.mr_diffs.is_empty());
}

#[tokio::test]
async fn aborted_diff_fan_out_releases_every_loading_count() {
    let (_server, mut app, mut rx, _files) = diff_screen(8, Duration::from_millis(300)).await;
    app.fetch_all_missing_diffs().await.unwrap();
    tokio::task::yield_now().await;
    app.selected_mr = Some(mr(2));
    app.release_idle_view_state();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let (mut on, mut off) = (0, 0);
    while let Ok(u) = rx.try_recv() {
        match u {
            DataUpdate::Loading(true) => on += 1,
            DataUpdate::Loading(false) => off += 1,
            _ => {}
        }
    }
    assert!(on > 0);
    assert_eq!(on, off, "loading leaked");
}

#[tokio::test]
async fn release_view_tasks_aborts_and_clears_keys() {
    let (_server, mut app, _rx, _files) = diff_screen(8, Duration::from_millis(300)).await;
    app.fetch_all_missing_diffs().await.unwrap();
    app.diffs_fetching_key = Some((1, 1));
    app.is_fetching_diffs = true;
    app.job_log_key = Some((1, 2));
    let gen_before = app.job_log_gen;
    app.release_view_tasks();
    assert!(app.diff_fan_task.is_none());
    assert!(app.diffs_fetching_key.is_none() && !app.is_fetching_diffs);
    assert!(app.job_log_key.is_none());
    assert!(app.job_log_gen > gen_before);
}

#[tokio::test]
async fn truncated_log_shifts_cursor_by_the_dropped_lines() {
    let (_server, mut app, _rx) = setup().await;
    dashboard_job_screen(&mut app, "running");
    app.view_mode = ViewMode::JobLog;
    app.job_log_key = Some((5, 7));
    let upd = |seq: u64, dropped: usize, n: usize| {
        let raw = "line\n".repeat(n);
        DataUpdate::JobLog {
            project_id: 5,
            job_id: 7,
            generation: 0,
            seq,
            lines: lazygitlab::app::pipelines::parse_job_log(&raw),
            raw,
            dropped,
        }
    };
    app.process_update_single(upd(1, 0, 10)).await;
    app.job_log_cursor = 5;
    app.process_update_single(upd(2, 2, 11)).await;
    assert_eq!(app.job_log_cursor, 4, "5 + notice line - 2 dropped");
    app.process_update_single(upd(3, 5, 11)).await;
    assert_eq!(app.job_log_cursor, 1, "shifted by 3 more dropped lines");
}
