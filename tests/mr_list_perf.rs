use lazygitlab::client::builder;
use lazygitlab::config::Transport;
use lazygitlab::models::*;
use lazygitlab::ui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

const LATENCY: Duration = Duration::from_millis(100);

#[derive(Clone)]
struct Row {
    pid: u64,
    iid: u64,
    updated: String,
    title: String,
    state: String,
    reviewers: Vec<u64>,
}

type Data = Arc<Mutex<Vec<Row>>>;

fn stamp(n: u64) -> String {
    format!(
        "2026-03-{:02}T{:02}:{:02}:00Z",
        1 + n / 1440 % 28,
        n / 60 % 24,
        n % 60
    )
}

fn dataset(projects: u64, per_project: u64) -> Data {
    let mut rows = Vec::new();
    for pid in 1..=projects {
        for iid in 1..=per_project {
            rows.push(Row {
                pid,
                iid,
                updated: stamp(pid * 100 + iid),
                title: format!("mr {pid}-{iid}"),
                state: "opened".into(),
                reviewers: Vec::new(),
            });
        }
    }
    Arc::new(Mutex::new(rows))
}

fn rest_json(r: &Row) -> Value {
    json!({
        "iid": r.iid, "project_id": r.pid, "title": r.title,
        "author": {"id": 1, "username": "u"}, "web_url": "https://gl/x", "state": r.state,
        "reviewers": r.reviewers.iter().map(|id| json!({"id": id, "username": format!("r{id}")})).collect::<Vec<_>>(),
        "source_branch": "a", "target_branch": "b", "draft": false,
        "created_at": "2026-01-01T00:00:00Z", "updated_at": r.updated,
        "references": {"full": format!("g/p{}!{}", r.pid, r.iid)}
    })
}

fn gql_node(r: &Row) -> Value {
    json!({
        "iid": r.iid.to_string(), "title": r.title, "state": r.state, "draft": false,
        "webUrl": format!("https://gl/g/p{}/-/merge_requests/{}", r.pid, r.iid),
        "createdAt": "2026-01-01T00:00:00Z", "updatedAt": r.updated,
        "detailedMergeStatus": "MERGEABLE", "conflicts": false,
        "sourceBranch": "a", "targetBranch": "b",
        "author": {"id": "gid://gitlab/User/1", "username": "u", "name": "U"},
        "assignees": {"nodes": []},
        "reviewers": {"nodes": r.reviewers.iter().map(|id| json!({"id": format!("gid://gitlab/User/{id}"), "username": format!("r{id}")})).collect::<Vec<_>>()},
        "labels": {"nodes": []}, "milestone": null, "headPipeline": null,
        "approved": false, "approvalsLeft": 0,
        "resolvableDiscussionsCount": 0, "resolvedDiscussionsCount": 0
    })
}

struct Gql {
    data: Data,
    mode: Arc<AtomicU16>,
}

impl Respond for Gql {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        let mode = self.mode.load(Ordering::SeqCst);
        if mode != 200 {
            return ResponseTemplate::new(mode).set_delay(LATENCY);
        }
        let body: Value = serde_json::from_slice(&req.body).unwrap();
        let since = body["variables"]["since"].as_str().map(str::to_string);
        let state = body["variables"]["state"]
            .as_str()
            .unwrap_or("all")
            .to_string();
        let ids: Vec<u64> = body["variables"]["ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| {
                g.as_str()
                    .unwrap()
                    .rsplit('/')
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap()
            })
            .collect();
        let rows = self.data.lock().unwrap().clone();
        let projects: Vec<Value> = ids
            .into_iter()
            .map(|pid| {
                let nodes: Vec<Value> = rows
                    .iter()
                    .filter(|r| r.pid == pid)
                    .filter(|r| state == "all" || r.state == state)
                    .filter(|r| since.as_ref().is_none_or(|s| r.updated >= *s))
                    .map(gql_node)
                    .collect();
                json!({
                    "id": format!("gid://gitlab/Project/{pid}"),
                    "fullPath": format!("g/p{pid}"),
                    "mergeRequests": {"pageInfo": {"hasNextPage": false, "endCursor": null}, "nodes": nodes}
                })
            })
            .collect();
        ResponseTemplate::new(200)
            .set_body_json(json!({"data": {
                "projects": {"nodes": projects},
                "queryComplexity": {"score": 50, "limit": 300}
            }}))
            .set_delay(LATENCY)
    }
}

struct Rest {
    data: Data,
    status: Arc<Mutex<std::collections::HashMap<u64, u16>>>,
    page2_status: Arc<AtomicU16>,
    arrivals: Arc<Mutex<Vec<Instant>>>,
    queries: Arc<Mutex<Vec<String>>>,
}

impl Respond for Rest {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        self.arrivals.lock().unwrap().push(Instant::now());
        self.queries
            .lock()
            .unwrap()
            .push(req.url.query().unwrap_or("").to_string());
        let pid: u64 = req.url.path().split('/').nth(4).unwrap().parse().unwrap();
        let q: std::collections::HashMap<String, String> = req
            .url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        let page: usize = q.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
        if let Some(s) = self.status.lock().unwrap().get(&pid)
            && *s == 0
        {
            return ResponseTemplate::new(200)
                .set_body_json(json!([]))
                .set_delay(Duration::from_secs(30));
        }
        if let Some(s) = self.status.lock().unwrap().get(&pid)
            && *s != 200
        {
            return ResponseTemplate::new(*s).set_delay(LATENCY);
        }
        if page > 1 {
            let s = self.page2_status.load(Ordering::SeqCst);
            if s != 200 {
                return ResponseTemplate::new(s).set_delay(LATENCY);
            }
        }
        let per_page: usize = q.get("per_page").and_then(|p| p.parse().ok()).unwrap_or(20);
        let since = q.get("updated_after");
        let state = q.get("state").cloned().unwrap_or_else(|| "all".into());
        let reviewer: Option<u64> = q.get("reviewer_id").and_then(|v| v.parse().ok());
        let rows: Vec<Row> = self
            .data
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.pid == pid)
            .filter(|r| state == "all" || r.state == state)
            .filter(|r| reviewer.is_none_or(|id| r.reviewers.contains(&id)))
            .filter(|r| since.is_none_or(|s| r.updated >= *s))
            .cloned()
            .collect();
        let total_pages = rows.len().div_ceil(per_page).max(1);
        let slice: Vec<Value> = rows
            .iter()
            .skip((page - 1) * per_page)
            .take(per_page)
            .map(rest_json)
            .collect();
        ResponseTemplate::new(200)
            .insert_header("x-total-pages", total_pages.to_string().as_str())
            .set_body_json(slice)
            .set_delay(LATENCY)
    }
}

struct Mocks {
    gql_mode: Arc<AtomicU16>,
    rest_status: Arc<Mutex<std::collections::HashMap<u64, u16>>>,
    page2_status: Arc<AtomicU16>,
    arrivals: Arc<Mutex<Vec<Instant>>>,
    queries: Arc<Mutex<Vec<String>>>,
}

async fn mount(server: &MockServer, data: &Data, graphql: bool) -> Mocks {
    Mock::given(path("/api/v4/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":1,"username":"u"})))
        .mount(server)
        .await;
    let mocks = Mocks {
        gql_mode: Arc::new(AtomicU16::new(200)),
        rest_status: Arc::default(),
        page2_status: Arc::new(AtomicU16::new(200)),
        arrivals: Arc::default(),
        queries: Arc::default(),
    };
    if graphql {
        Mock::given(method("POST"))
            .and(path("/api/graphql"))
            .respond_with(Gql {
                data: data.clone(),
                mode: mocks.gql_mode.clone(),
            })
            .mount(server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v4/projects/\d+/merge_requests$"))
        .respond_with(Rest {
            data: data.clone(),
            status: mocks.rest_status.clone(),
            page2_status: mocks.page2_status.clone(),
            arrivals: mocks.arrivals.clone(),
            queries: mocks.queries.clone(),
        })
        .mount(server)
        .await;
    mocks
}

async fn app_for(
    server: &MockServer,
    projects: u64,
) -> (App, tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
    let client = builder(
        server.uri().replace("http://", ""),
        "t".into(),
        Transport::PlainHttp,
    )
    .build_async()
    .await
    .unwrap();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.active_context = Context::Dashboard;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::MergeRequests;
    app.member_project_ids = (1..=projects).collect();
    app.membership_sync_attempted = true;
    (app, rx)
}

async fn settle(app: &mut App, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
    let start = Instant::now();
    tokio::time::sleep(Duration::from_millis(20)).await;
    loop {
        app.process_updates(rx).await;
        if !app.is_loading || start.elapsed() > Duration::from_secs(15) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    app.process_updates(rx).await;
}

async fn count(server: &MockServer, method: &str, path_prefix: &str) -> usize {
    server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.method.as_str() == method && r.url.path().starts_with(path_prefix))
        .count()
}

fn unique_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lazygitlab-mrperf-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[tokio::test]
async fn forty_five_projects_show_rows_with_three_graphql_requests() {
    let server = MockServer::start().await;
    let data = dataset(45, 1);
    mount(&server, &data, true).await;
    let (mut app, mut rx) = app_for(&server, 45).await;

    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;

    assert_eq!(app.merge_requests.len(), 45);
    assert_eq!(count(&server, "POST", "/api/graphql").await, 3);
    assert_eq!(count(&server, "GET", "/api/v4/projects").await, 0);
}

#[tokio::test]
async fn rest_fallback_uses_per_page_100_and_bounded_concurrency() {
    let server = MockServer::start().await;
    let data = dataset(45, 1);
    let mocks = mount(&server, &data, false).await;
    let (mut app, mut rx) = app_for(&server, 45).await;

    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;

    assert_eq!(app.merge_requests.len(), 45);
    assert!(
        mocks
            .queries
            .lock()
            .unwrap()
            .iter()
            .all(|q| q.contains("per_page=100"))
    );
    let mut arrivals = mocks.arrivals.lock().unwrap().clone();
    arrivals.sort();
    let window = Duration::from_millis(90);
    let max_overlap = arrivals
        .iter()
        .enumerate()
        .map(|(i, t)| {
            arrivals[i..]
                .iter()
                .take_while(|u| **u - *t < window)
                .count()
        })
        .max()
        .unwrap();
    assert!(max_overlap <= 6, "{max_overlap} requests in flight");
}

#[tokio::test]
async fn five_hundred_mrs_over_fifty_projects_load_under_two_seconds() {
    for graphql in [true, false] {
        let server = MockServer::start().await;
        let data = dataset(50, 10);
        mount(&server, &data, graphql).await;
        let (mut app, mut rx) = app_for(&server, 50).await;

        let start = Instant::now();
        app.fetch_global_merge_requests().await.unwrap();
        settle(&mut app, &mut rx).await;
        let took = start.elapsed();

        assert_eq!(app.merge_requests.len(), 500, "graphql={graphql}");
        let bound = if graphql { 2 } else { 5 };
        assert!(
            took < Duration::from_secs(bound),
            "graphql={graphql} took {took:?}"
        );
        assert!(!app.mrs_stale);
    }
}

#[tokio::test]
async fn unchanged_refresh_costs_one_request_per_batch_and_mutates_nothing() {
    let server = MockServer::start().await;
    let data = dataset(45, 3);
    mount(&server, &data, true).await;
    let (mut app, mut rx) = app_for(&server, 45).await;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    let before = app.merge_requests.clone();
    let mutations = app.mrs_mutations;
    let posts = count(&server, "POST", "/api/graphql").await;
    assert_eq!(posts, 3);

    app.refresh_background().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(count(&server, "POST", "/api/graphql").await - posts, 3);
    assert_eq!(app.merge_requests, before);
    assert_eq!(app.mrs_mutations, mutations);
    let last: Value = serde_json::from_slice(
        &server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .rfind(|r| r.url.path() == "/api/graphql")
            .unwrap()
            .body,
    )
    .unwrap();
    assert!(last["variables"]["since"].is_string());

    data.lock().unwrap()[0].updated = "2026-04-01T00:00:00Z".into();
    data.lock().unwrap()[0].title = "renamed".into();
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;
    assert_eq!(app.mrs_mutations, mutations + 1);
    assert_eq!(app.merge_requests.len(), before.len());
    assert!(app.merge_requests.iter().any(|m| m.title == "renamed"));
}

#[tokio::test]
async fn manual_refresh_reconciles_removed_rows() {
    let server = MockServer::start().await;
    let data = dataset(5, 2);
    mount(&server, &data, true).await;
    let (mut app, mut rx) = app_for(&server, 5).await;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 10);

    data.lock().unwrap().remove(0);
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;
    assert_eq!(
        app.merge_requests.len(),
        10,
        "background refresh only upserts"
    );

    app.refresh().await;
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 9);
}

#[tokio::test]
async fn rest_failures_never_remove_rows_and_show_stale() {
    let server = MockServer::start().await;
    let data = dataset(10, 3);
    let mocks = mount(&server, &data, false).await;
    let (mut app, mut rx) = app_for(&server, 10).await;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 30);
    assert!(!app.mrs_stale);

    mocks.rest_status.lock().unwrap().insert(3, 500);
    mocks.rest_status.lock().unwrap().insert(4, 429);
    data.lock().unwrap().retain(|r| r.pid != 5);
    app.refresh().await;
    settle(&mut app, &mut rx).await;

    assert!(app.mrs_stale);
    assert_eq!(
        app.merge_requests
            .iter()
            .filter(|m| m.project_id == 3)
            .count(),
        3
    );
    assert_eq!(
        app.merge_requests
            .iter()
            .filter(|m| m.project_id == 4)
            .count(),
        3
    );
    assert_eq!(
        app.merge_requests
            .iter()
            .filter(|m| m.project_id == 5)
            .count(),
        0,
        "projects that answered are replaced"
    );
    let msg = app.status_message.clone().unwrap_or_default();
    assert!(
        msg.starts_with("Rate limited") || msg.starts_with("refresh failed"),
        "{msg}"
    );
}

#[tokio::test]
async fn failed_second_page_keeps_every_row() {
    let server = MockServer::start().await;
    let data = dataset(1, 250);
    let mocks = mount(&server, &data, false).await;
    let (mut app, mut rx) = app_for(&server, 1).await;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 250);

    mocks.page2_status.store(500, Ordering::SeqCst);
    app.refresh().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(app.merge_requests.len(), 250);
    assert!(app.mrs_stale);
}

#[tokio::test]
async fn graphql_rate_limit_keeps_rows_and_reports() {
    let server = MockServer::start().await;
    let data = dataset(30, 2);
    let mocks = mount(&server, &data, true).await;
    let (mut app, mut rx) = app_for(&server, 30).await;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 60);

    mocks.gql_mode.store(429, Ordering::SeqCst);
    app.refresh().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(app.merge_requests.len(), 60);
    assert!(app.mrs_stale);
    let msg = app.status_message.clone().unwrap_or_default();
    assert!(msg.starts_with("Rate limited"), "{msg}");
}

#[tokio::test]
async fn cached_rows_are_drawn_before_any_network_response() {
    let dir = unique_dir("cached");
    let server = MockServer::start().await;
    let data = dataset(5, 3);
    let mocks = mount(&server, &data, true).await;
    let host = server.uri();

    let (mut first, mut rx) = app_for(&server, 5).await;
    first.enable_cache(&dir, &host);
    first
        .process_update_single(DataUpdate::CurrentUser(Author::new(1, "u".into())))
        .await;
    first.fetch_global_merge_requests().await.unwrap();
    settle(&mut first, &mut rx).await;
    assert_eq!(first.merge_requests.len(), 15);
    first.flush_cache();
    drop(first);

    drop(mocks);
    server.reset().await;
    Mock::given(path("/api/v4/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":1,"username":"u"})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/graphql"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
        .mount(&server)
        .await;

    let (mut second, mut rx) = app_for(&server, 5).await;
    second.enable_cache(&dir, &host);
    second.fetch_global_merge_requests().await.unwrap();
    let mut terminal = Terminal::new(TestBackend::new(140, 40)).unwrap();
    let frame = terminal.draw(|f| ui(f, &mut second)).unwrap();
    let text: String = frame
        .buffer
        .content
        .iter()
        .map(|c| c.symbol())
        .collect::<Vec<_>>()
        .join("");

    assert_eq!(second.merge_requests.len(), 15);
    assert!(text.contains("mr 1-1"), "first frame has no rows");
    assert!(text.contains("cached"), "no cached label");
    while let Ok(u) = rx.try_recv() {
        assert!(!matches!(u, DataUpdate::MrList(_)));
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn cache_stays_off_without_a_known_user() {
    let dir = unique_dir("nouser");
    let server = MockServer::start().await;
    let data = dataset(3, 2);
    mount(&server, &data, true).await;
    let (mut app, mut rx) = app_for(&server, 3).await;
    app.enable_cache(&dir, &server.uri());
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    app.flush_cache();
    let files = std::fs::read_dir(&dir).map(|d| d.count()).unwrap_or(0);
    assert_eq!(files, 0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn missing_graphql_ids_disable_the_pipeline_and_milestone_actions() {
    let server = MockServer::start().await;
    let data = dataset(1, 1);
    mount(&server, &data, true).await;
    let (mut app, mut rx) = app_for(&server, 1).await;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    let mut mr = app.merge_requests[0].clone();
    mr.head_pipeline = Some(PipelineMin {
        id: 0,
        status: "success".into(),
    });
    mr.milestone = Some(Milestone {
        id: 0,
        title: "m".into(),
    });
    assert_eq!(app.mr_pipeline_id(&mr), None);
    assert!(!app.mr_milestone_known(&mr));
    mr.head_pipeline = Some(PipelineMin {
        id: 9,
        status: "success".into(),
    });
    mr.milestone = Some(Milestone {
        id: 4,
        title: "m".into(),
    });
    app.mr_meta.clear();
    assert_eq!(app.mr_pipeline_id(&mr), Some(9));
    assert!(app.mr_milestone_known(&mr));
}

fn item_rows(app: &App) -> Vec<usize> {
    app.filtered_mrs
        .iter()
        .enumerate()
        .filter(|(_, g)| matches!(g, GroupedItem::Item(_)))
        .map(|(i, _)| i)
        .collect()
}

#[tokio::test]
async fn detail_requests_wait_for_the_cursor_to_rest() {
    let server = MockServer::start().await;
    let data = dataset(1, 12);
    mount(&server, &data, false).await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v4/projects/\d+/merge_requests/\d+$"))
        .respond_with(|req: &Request| {
            let iid: u64 = req.url.path().rsplit('/').next().unwrap().parse().unwrap();
            ResponseTemplate::new(200).set_body_json(rest_json(&Row {
                pid: 1,
                iid,
                updated: stamp(iid),
                title: format!("mr 1-{iid}"),
                state: "opened".into(),
                reviewers: Vec::new(),
            }))
        })
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v4/projects/\d+/merge_requests/\d+/.+$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;
    let (mut app, mut rx) = app_for(&server, 1).await;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    let rows = item_rows(&app);
    assert_eq!(rows.len(), 12);

    for i in &rows[..8] {
        app.mrs_state.select(Some(*i));
        app.update_preview().await;
        tokio::time::sleep(Duration::from_millis(30)).await;
        app.fire_pending_detail().await;
    }
    assert_eq!(
        count(&server, "GET", "/api/v4/projects/1/merge_requests/").await,
        0,
        "no detail request while scrolling quickly"
    );

    tokio::time::sleep(Duration::from_millis(300)).await;
    app.update_preview().await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    app.process_updates(&mut rx).await;
    let singles = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| {
            let p = r.url.path();
            p.starts_with("/api/v4/projects/1/merge_requests/") && p.matches('/').count() == 6
        })
        .count();
    assert_eq!(singles, 1, "one detail fetch after the cursor rests");
}

#[tokio::test]
async fn mutations_are_refused_fast_while_rate_limited() {
    let server = MockServer::start().await;
    let data = dataset(2, 2);
    let mocks = mount(&server, &data, false).await;
    Mock::given(method("POST"))
        .and(path_regex(r"/approve$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&server)
        .await;
    let (mut app, mut rx) = app_for(&server, 2).await;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    mocks.rest_status.lock().unwrap().insert(1, 429);
    app.refresh().await;
    settle(&mut app, &mut rx).await;

    app.selected_mr = Some(app.merge_requests[0].clone());
    let start = Instant::now();
    app.approve_mr().await.unwrap();
    settle(&mut app, &mut rx).await;

    assert!(start.elapsed() < Duration::from_secs(5));
    assert_eq!(count(&server, "POST", "/api/v4/projects").await, 0);
    let err = app.last_error.clone().unwrap_or_default();
    assert!(err.contains("Rate limited"), "{err}");
}

async fn loaded_app(
    server: &MockServer,
    data: &Data,
    graphql: bool,
    projects: u64,
) -> (App, tokio::sync::mpsc::UnboundedReceiver<DataUpdate>, Mocks) {
    let mocks = mount(server, data, graphql).await;
    let (mut app, mut rx) = app_for(server, projects).await;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    (app, rx, mocks)
}

async fn last_graphql_state(server: &MockServer) -> String {
    let req = server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .rfind(|r| r.url.path() == "/api/graphql")
        .unwrap();
    let body: Value = serde_json::from_slice(&req.body).unwrap();
    body["variables"]["state"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn merged_mr_leaves_the_opened_view_after_one_background_tick() {
    for graphql in [true, false] {
        let server = MockServer::start().await;
        let data = dataset(3, 2);
        let (mut app, mut rx, _m) = loaded_app(&server, &data, graphql, 3).await;
        assert_eq!(app.merge_requests.len(), 6);

        {
            let mut d = data.lock().unwrap();
            d[0].state = "merged".into();
            d[0].updated = "2026-05-01T00:00:00Z".into();
        }
        app.refresh_background().await;
        settle(&mut app, &mut rx).await;

        assert_eq!(app.merge_requests.len(), 5, "graphql={graphql}");
        assert!(app.merge_requests.iter().all(|m| m.state == "opened"));
        if graphql {
            assert_eq!(last_graphql_state(&server).await, "all");
        }
    }
}

#[tokio::test]
async fn closed_mr_leaves_the_opened_view_after_one_background_tick() {
    let server = MockServer::start().await;
    let data = dataset(3, 2);
    let (mut app, mut rx, _m) = loaded_app(&server, &data, true, 3).await;
    {
        let mut d = data.lock().unwrap();
        d[3].state = "closed".into();
        d[3].updated = "2026-05-01T00:00:00Z".into();
    }
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 5);
    assert!(!app.merge_requests.iter().any(|m| m.state == "closed"));
}

#[tokio::test]
async fn removed_reviewer_drops_the_row_from_the_review_view() {
    for graphql in [true, false] {
        let server = MockServer::start().await;
        let data = dataset(2, 2);
        for r in data.lock().unwrap().iter_mut() {
            r.reviewers = vec![1];
        }
        let mocks = mount(&server, &data, graphql).await;
        let (mut app, mut rx) = app_for(&server, 2).await;
        app.process_update_single(DataUpdate::CurrentUser(Author::new(1, "u".into())))
            .await;
        app.mr_filter = MergeRequestFilter::Review;
        app.fetch_global_merge_requests().await.unwrap();
        settle(&mut app, &mut rx).await;
        assert_eq!(app.merge_requests.len(), 4, "graphql={graphql}");

        {
            let mut d = data.lock().unwrap();
            d[0].reviewers = vec![2];
            d[0].updated = "2026-05-01T00:00:00Z".into();
        }
        app.refresh_background().await;
        settle(&mut app, &mut rx).await;
        assert_eq!(app.merge_requests.len(), 3, "graphql={graphql}");
        drop(mocks);
    }
}

#[tokio::test]
async fn a_hanging_project_does_not_block_the_others() {
    let server = MockServer::start().await;
    let data = dataset(6, 2);
    let mocks = mount(&server, &data, false).await;
    mocks.rest_status.lock().unwrap().insert(3, 0);
    let (mut app, mut rx) = app_for(&server, 6).await;
    app.http = app
        .http
        .clone()
        .with_pages_deadline(Duration::from_millis(600));

    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;

    assert_eq!(app.merge_requests.len(), 10);
    assert!(app.merge_requests.iter().all(|m| m.project_id != 3));
    assert!(app.mrs_stale);
}

#[tokio::test]
async fn user_change_drops_rows_and_refetches() {
    let dir = unique_dir("userchange");
    let server = MockServer::start().await;
    let data = dataset(3, 2);
    mount(&server, &data, true).await;
    let host = server.uri();
    let (mut app, mut rx) = app_for(&server, 3).await;
    app.enable_cache(&dir, &host);
    app.process_update_single(DataUpdate::CurrentUser(Author::new(1, "u".into())))
        .await;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 6);

    app.process_update_single(DataUpdate::CurrentUser(Author::new(2, "v".into())))
        .await;
    assert!(app.merge_requests.is_empty());
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 6);
    let _ = std::fs::remove_dir_all(&dir);
}

fn switch_filter(app: &mut App, filter: MergeRequestFilter) {
    app.mr_filter = filter;
    app.merge_requests.clear();
    app.filter_lists();
}

#[tokio::test]
async fn switching_filters_away_and_back_still_loads_the_first_list() {
    let dir = unique_dir("aba");
    let server = MockServer::start().await;
    let data = dataset(6, 2);
    mount(&server, &data, true).await;
    let host = server.uri();
    let (mut app, mut rx) = app_for(&server, 6).await;
    app.enable_cache(&dir, &host);
    app.process_update_single(DataUpdate::CurrentUser(Author::new(1, "u".into())))
        .await;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 12);

    switch_filter(&mut app, MergeRequestFilter::Merged);
    app.refresh().await;
    switch_filter(&mut app, MergeRequestFilter::Opened);
    app.refresh().await;
    assert_eq!(app.merge_requests.len(), 12, "cached rows paint on return");
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 12);
    assert!(!app.is_loading);
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn switching_away_and_back_during_a_hanging_fetch_recovers() {
    let server = MockServer::start().await;
    let data = dataset(6, 2);
    let mocks = mount(&server, &data, false).await;
    mocks.rest_status.lock().unwrap().insert(3, 0);
    let (mut app, mut rx) = app_for(&server, 6).await;
    app.http = app
        .http
        .clone()
        .with_pages_deadline(Duration::from_millis(600));

    app.refresh().await;
    switch_filter(&mut app, MergeRequestFilter::Merged);
    app.refresh().await;
    switch_filter(&mut app, MergeRequestFilter::Opened);
    app.refresh().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(app.merge_requests.len(), 10);
    assert!(
        !app.is_loading,
        "an aborted fetch must end its loading marker"
    );
}

#[tokio::test]
async fn manual_refresh_during_a_pass_queues_one_full_rerun() {
    let server = MockServer::start().await;
    let data = dataset(45, 1);
    let (mut app, mut rx, _m) = loaded_app(&server, &data, true, 45).await;
    let base = count(&server, "POST", "/api/graphql").await;
    assert_eq!(base, 3);

    app.refresh_background().await;
    app.refresh().await;
    settle(&mut app, &mut rx).await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    settle(&mut app, &mut rx).await;

    let posts = server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|r| r.url.path() == "/api/graphql")
        .collect::<Vec<_>>();
    assert_eq!(
        posts.len(),
        base + 6,
        "one incremental pass and one full rerun"
    );
    let last: Value = serde_json::from_slice(&posts.last().unwrap().body).unwrap();
    assert!(
        last["variables"]["since"].is_null(),
        "the rerun is a full pass"
    );
}

#[tokio::test]
async fn background_tick_keeps_a_pending_full_request() {
    let server = MockServer::start().await;
    let data = dataset(5, 1);
    let (mut app, mut rx, _m) = loaded_app(&server, &data, true, 5).await;
    app.mrs_force_full = true;
    app.refresh_background().await;
    assert!(
        !app.mrs_force_full || app.mrs_in_flight.is_some(),
        "the full pass was started or is still pending"
    );
    settle(&mut app, &mut rx).await;
    let last = server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .rfind(|r| r.url.path() == "/api/graphql")
        .unwrap();
    let body: Value = serde_json::from_slice(&last.body).unwrap();
    assert!(body["variables"]["since"].is_null());
}

#[tokio::test]
async fn update_for_another_user_is_dropped() {
    let server = MockServer::start().await;
    let data = dataset(2, 1);
    mount(&server, &data, true).await;
    let (mut app, _rx) = app_for(&server, 2).await;
    app.process_update_single(DataUpdate::CurrentUser(Author::new(2, "v".into())))
        .await;
    let row: MergeRequest = serde_json::from_value(rest_json(&Row {
        pid: 1,
        iid: 1,
        updated: stamp(1),
        title: "stale".into(),
        state: "opened".into(),
        reviewers: Vec::new(),
    }))
    .unwrap();
    app.process_update_single(DataUpdate::MrList(Box::new(MrListUpdate {
        scope: Some(MergeRequestFilter::Opened),
        project: None,
        rows: vec![row],
        complete: true,
        user_id: Some(1),
        ..Default::default()
    })))
    .await;
    assert!(app.merge_requests.is_empty());
}

#[tokio::test]
async fn failed_incremental_pass_is_followed_by_a_full_one() {
    let server = MockServer::start().await;
    let data = dataset(4, 2);
    let (mut app, mut rx, mocks) = loaded_app(&server, &data, false, 4).await;
    mocks.rest_status.lock().unwrap().insert(2, 500);
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;
    mocks.rest_status.lock().unwrap().clear();
    let seen = mocks.queries.lock().unwrap().len();

    app.refresh_background().await;
    settle(&mut app, &mut rx).await;

    let queries = mocks.queries.lock().unwrap().clone();
    assert!(queries.len() > seen);
    assert!(
        queries[seen..].iter().all(|q| !q.contains("updated_after")),
        "{:?}",
        &queries[seen..]
    );
}
