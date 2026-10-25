use lazygitlab::cache::{ListCache, query_key};
use lazygitlab::client::builder;
use lazygitlab::config::Transport;
use lazygitlab::graphql::{GraphqlState, fetch_merge_requests_via};
use lazygitlab::http::{HttpLayer, Outcome};
use lazygitlab::models::*;
use serde_json::{Value, json};
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

const HOST: &str = "https://gitlab.completeness.test";

#[derive(Clone)]
struct Row {
    pid: u64,
    iid: u64,
    updated: String,
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

fn dataset(projects: u64, per_project: u64, state: &str) -> Data {
    let mut rows = Vec::new();
    for pid in 1..=projects {
        for iid in 1..=per_project {
            rows.push(Row {
                pid,
                iid,
                updated: stamp(iid),
                state: state.into(),
                reviewers: Vec::new(),
            });
        }
    }
    Arc::new(Mutex::new(rows))
}

fn gql_node(r: &Row) -> Value {
    json!({
        "iid": r.iid.to_string(), "title": format!("mr {}-{}", r.pid, r.iid), "state": r.state, "draft": false,
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

fn rest_mr(pid: u64, iid: u64, state: &str, updated: &str) -> Value {
    json!({
        "iid": iid, "project_id": pid, "title": format!("mr {pid}-{iid}"),
        "author": {"id": 1, "username": "u"}, "web_url": "https://gl/x", "state": state,
        "source_branch": "a", "target_branch": "b", "draft": false,
        "created_at": "2026-01-01T00:00:00Z", "updated_at": updated,
        "references": {"full": format!("g/p{pid}!{iid}")}
    })
}

/// Cursor paginated GraphQL answer: the cursor is the offset, newest rows first.
struct Paged {
    data: Data,
    mode: Arc<Mutex<u16>>,
}

impl Respond for Paged {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        let mode = *self.mode.lock().unwrap();
        if mode != 200 {
            return ResponseTemplate::new(mode).insert_header("retry-after", "2");
        }
        let body: Value = serde_json::from_slice(&req.body).unwrap();
        let v = &body["variables"];
        let since = v["since"].as_str().map(str::to_string);
        let state = v["state"].as_str().unwrap_or("all").to_string();
        let after: usize = v["after"]
            .as_str()
            .and_then(|a| a.parse().ok())
            .unwrap_or(0);
        let first = v["first"].as_u64().unwrap_or(20) as usize;
        let ids: Vec<u64> = v["ids"]
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
                let mut mine: Vec<&Row> = rows
                    .iter()
                    .filter(|r| r.pid == pid)
                    .filter(|r| state == "all" || r.state == state)
                    .filter(|r| since.as_ref().is_none_or(|s| r.updated >= *s))
                    .collect();
                mine.sort_by(|a, b| b.updated.cmp(&a.updated));
                let end = (after + first).min(mine.len());
                let nodes: Vec<Value> = mine[after.min(end)..end].iter().map(|r| gql_node(r)).collect();
                let has_next = end < mine.len();
                json!({
                    "id": format!("gid://gitlab/Project/{pid}"),
                    "fullPath": format!("g/p{pid}"),
                    "mergeRequests": {"pageInfo": {"hasNextPage": has_next, "endCursor": has_next.then(|| end.to_string())}, "nodes": nodes}
                })
            })
            .collect();
        ResponseTemplate::new(200).set_body_json(json!({"data": {
            "projects": {"nodes": projects},
            "queryComplexity": {"score": 50, "limit": 300}
        }}))
    }
}

async fn mount_user(server: &MockServer) {
    Mock::given(path("/api/v4/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":1,"username":"u"})))
        .mount(server)
        .await;
}

async fn mount_graphql(server: &MockServer, data: &Data) -> Arc<Mutex<u16>> {
    let mode = Arc::new(Mutex::new(200));
    Mock::given(method("POST"))
        .and(path("/api/graphql"))
        .respond_with(Paged {
            data: data.clone(),
            mode: mode.clone(),
        })
        .mount(server)
        .await;
    mode
}

async fn client(server: &MockServer) -> gitlab::AsyncGitlab {
    builder(
        server.uri().replace("http://", ""),
        "t".into(),
        Transport::PlainHttp,
    )
    .build_async()
    .await
    .unwrap()
}

type Rx = tokio::sync::mpsc::UnboundedReceiver<DataUpdate>;

async fn app_for(server: &MockServer, projects: u64) -> (App, Rx) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client(server).await, tx, None);
    app.active_context = Context::Dashboard;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::MergeRequests;
    app.member_project_ids = (1..=projects).collect();
    app.membership_sync_attempted = true;
    (app, rx)
}

async fn settle(app: &mut App, rx: &mut Rx) {
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

async fn wait_until(app: &mut App, rx: &mut Rx, what: &str, cond: impl Fn(&App) -> bool) {
    let start = Instant::now();
    loop {
        app.process_updates(rx).await;
        if cond(app) {
            return;
        }
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "timed out: {what}"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn unique_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lazygitlab-complete-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn iids(app: &App) -> Vec<u64> {
    let mut v: Vec<u64> = app.merge_requests.iter().map(|m| m.iid).collect();
    v.sort_unstable();
    v
}

#[tokio::test]
async fn merged_filter_over_the_followup_cap_is_a_complete_window() {
    let server = MockServer::start().await;
    mount_user(&server).await;
    let data = dataset(1, 300, "merged");
    mount_graphql(&server, &data).await;
    let (mut app, mut rx) = app_for(&server, 1).await;
    let dir = unique_dir("window");
    app.enable_cache(&dir, HOST);
    app.process_update_single(DataUpdate::CurrentUser(Author::new(1, "u".into())))
        .await;
    app.mr_filter = MergeRequestFilter::Merged;

    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;

    assert_eq!(
        app.merge_requests.len(),
        80,
        "20 first page plus 3 follow-ups"
    );
    assert!(
        !app.status_message
            .as_deref()
            .unwrap_or("")
            .contains("refresh failed"),
        "{:?}",
        app.status_message
    );
    assert!(!app.mrs_stale);
    assert!(app.mrs_sync.cursor.is_some());
    assert!(app.mrs_sync.last_full.is_some());
    app.flush_cache();
    let cache = ListCache::load(&dir, HOST, NonZeroU64::new(1));
    let entry = cache
        .get(&query_key("mr", "merged", "merged", "global"))
        .expect("cache entry written");
    assert_eq!(entry.items.len(), 80);

    {
        let mut d = data.lock().unwrap();
        let row = d.iter_mut().find(|r| r.iid == 300).unwrap();
        row.state = "opened".into();
        row.updated = "2026-04-01T00:00:00Z".into();
    }
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 79);
    assert!(!app.merge_requests.iter().any(|m| m.iid == 300));
}

#[tokio::test]
async fn full_pass_over_a_window_removes_inside_and_keeps_older_rows() {
    let server = MockServer::start().await;
    mount_user(&server).await;
    let data = dataset(1, 300, "merged");
    mount_graphql(&server, &data).await;
    let (mut app, mut rx) = app_for(&server, 1).await;
    app.mr_filter = MergeRequestFilter::Merged;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 80);

    let old: MergeRequest = serde_json::from_value(rest_mr(1, 1, "merged", &stamp(1))).unwrap();
    app.merge_requests.push(old);
    data.lock().unwrap().retain(|r| r.iid != 250);
    app.mrs_force_full = true;
    app.fetch_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;

    let ids = iids(&app);
    assert!(!ids.contains(&250), "removed inside the window");
    assert!(ids.contains(&1), "older than the window stays");
    assert!(ids.contains(&220), "window slid down by one");
    assert_eq!(ids.len(), 81);
}

#[tokio::test]
async fn failed_batch_marks_only_its_own_projects() {
    let server = MockServer::start().await;
    mount_user(&server).await;
    let data = dataset(25, 1, "opened");
    Mock::given(method("POST"))
        .and(path("/api/graphql"))
        .respond_with(move |req: &Request| {
            let body: Value = serde_json::from_slice(&req.body).unwrap();
            let ids: Vec<u64> = body["variables"]["ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|g| g.as_str().unwrap().rsplit('/').next().unwrap().parse().unwrap())
                .collect();
            if ids.contains(&1) {
                return ResponseTemplate::new(500);
            }
            let nodes: Vec<Value> = ids
                .iter()
                .map(|pid| {
                    let rows: Vec<Value> = data
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|r| r.pid == *pid)
                        .map(gql_node)
                        .collect();
                    json!({
                        "id": format!("gid://gitlab/Project/{pid}"), "fullPath": format!("g/p{pid}"),
                        "mergeRequests": {"pageInfo": {"hasNextPage": false, "endCursor": null}, "nodes": rows}
                    })
                })
                .collect();
            ResponseTemplate::new(200).set_body_json(json!({"data": {"projects": {"nodes": nodes}}}))
        })
        .mount(&server)
        .await;
    let c = client(&server).await;
    let http = HttpLayer::new(c.clone());
    let state = GraphqlState::new();
    let ids: Vec<u64> = (1..=25).collect();
    let res = fetch_merge_requests_via(&c, Some(&http), &state, &ids, "opened", None)
        .await
        .unwrap();
    assert!(!res.complete);
    assert_eq!(res.failed_projects, (1..=20).collect::<Vec<u64>>());
    assert_eq!(res.rows.len(), 5);
}

#[tokio::test]
async fn cold_start_on_the_review_filter_fetches_when_the_user_arrives() {
    let server = MockServer::start().await;
    mount_user(&server).await;
    let data = dataset(2, 2, "opened");
    for r in data.lock().unwrap().iter_mut() {
        r.reviewers = vec![1];
    }
    mount_graphql(&server, &data).await;
    let (mut app, mut rx) = app_for(&server, 2).await;
    app.mr_filter = MergeRequestFilter::Review;
    assert!(app.current_user.is_none());

    app.refresh_background().await;
    wait_until(&mut app, &mut rx, "rows after /user answered", |a| {
        !a.merge_requests.is_empty()
    })
    .await;
    assert_eq!(app.merge_requests.len(), 4);
}

fn issue(project: u64, iid: u64, title: &str) -> Value {
    json!({"iid": iid, "project_id": project, "title": title, "author": {"id": 1, "username": "u"},
        "web_url": "u", "state": "opened", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"})
}

fn marker(app: &App) -> Vec<(String, Instant)> {
    let mut v: Vec<_> = app
        .fanouts_started
        .iter()
        .map(|(k, t)| (k.clone(), *t))
        .collect();
    v.sort();
    v
}

#[tokio::test]
async fn project_issue_failure_does_not_end_the_global_fanout() {
    let server = MockServer::start().await;
    mount_user(&server).await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v4/projects/[12]/issues$"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!([]))
                .set_delay(Duration::from_millis(800)),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/9/issues"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let (mut app, mut rx) = app_for(&server, 2).await;
    app.active_selection = MenuSelection::Issues;
    app.refresh_background().await;
    let before = marker(&app);
    assert_eq!(before.len(), 1, "global fan-out in flight");

    app.selected_project = Some(Project {
        id: 9,
        name: "p".into(),
        web_url: None,
        path_with_namespace: Some("g/p".into()),
        default_branch: None,
        namespace: None,
    });
    app.fetch_issues().await.unwrap();
    wait_until(&mut app, &mut rx, "project issues failure", |a| {
        a.status_message
            .as_deref()
            .is_some_and(|m| m.contains("refresh failed"))
    })
    .await;

    assert_eq!(marker(&app), before, "the global marker was not touched");
    settle(&mut app, &mut rx).await;
    assert!(app.fanouts_started.is_empty());
}

#[tokio::test]
async fn stale_issue_generation_is_dropped() {
    let server = MockServer::start().await;
    mount_user(&server).await;
    let (mut app, _rx) = app_for(&server, 1).await;
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Issues;
    app.selected_project = Some(Project {
        id: 7,
        name: "p".into(),
        web_url: None,
        path_with_namespace: Some("g/p".into()),
        default_branch: None,
        namespace: None,
    });
    app.issues_fetch_gen = 5;
    let result = |generation: u64, title: &str| {
        let data: Vec<Issue> = vec![serde_json::from_value(issue(7, 1, title)).unwrap()];
        DataUpdate::ScopedIssue(
            IssueScope {
                filter: IssueFilter::Opened,
                project: Some(7),
                generation,
                fanout_id: 0,
            },
            Box::new(DataUpdate::ProjectIssues {
                project_id: 7,
                data,
                page: 1,
                has_more: false,
            }),
        )
    };
    app.process_update_single(result(5, "fresh")).await;
    app.process_update_single(result(4, "stale")).await;
    let titles: Vec<&str> = app.issues.iter().map(|i| i.title.as_str()).collect();
    assert_eq!(titles, vec!["fresh"]);
}

#[tokio::test]
async fn fanout_end_messages_end_it_exactly_once() {
    let server = MockServer::start().await;
    mount_user(&server).await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v4/projects/\d+/issues$"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!([]))
                .set_delay(Duration::from_millis(800)),
        )
        .mount(&server)
        .await;
    let (mut app, mut rx) = app_for(&server, 1).await;
    app.active_selection = MenuSelection::Issues;
    app.refresh_background().await;
    let (key, started) = marker(&app).remove(0);
    let id = app.fanout_ids[&key];
    let generation = app.issues_fetch_gen;
    let scope = |fanout_id| IssueScope {
        filter: IssueFilter::Opened,
        project: None,
        generation,
        fanout_id,
    };

    app.process_update_single(DataUpdate::ScopedIssue(
        scope(id + 100),
        Box::new(DataUpdate::RefreshFailed),
    ))
    .await;
    assert_eq!(
        marker(&app),
        vec![(key.clone(), started)],
        "a foreign id ends nothing"
    );
    app.process_update_single(DataUpdate::ScopedIssue(
        scope(id),
        Box::new(DataUpdate::RefreshFailed),
    ))
    .await;
    assert!(
        app.fanouts_started.is_empty(),
        "the first end message ends it"
    );
    app.process_update_single(DataUpdate::ScopedIssue(
        scope(id),
        Box::new(DataUpdate::RefreshFailed),
    ))
    .await;
    assert!(
        app.fanouts_started.is_empty(),
        "a second end message starts nothing"
    );
    settle(&mut app, &mut rx).await;
}

#[tokio::test]
async fn membership_timeout_releases_the_flag_and_the_next_tick_recovers() {
    let server = MockServer::start().await;
    mount_user(&server).await;
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    Mock::given(method("GET"))
        .and(path("/api/v4/projects"))
        .respond_with(move |_: &Request| {
            let n = seen.fetch_add(1, Ordering::SeqCst);
            let t = ResponseTemplate::new(200)
                .set_body_json(json!([{"id": 1, "name": "p", "path_with_namespace": "g/p"}]));
            if n == 0 {
                t.set_delay(Duration::from_secs(5))
            } else {
                t
            }
        })
        .mount(&server)
        .await;
    let (mut app, mut rx) = app_for(&server, 0).await;
    app.membership_sync_attempted = false;
    app.membership_timeout = Duration::from_millis(200);

    app.fetch_membership_project_ids().await.unwrap();
    assert!(app.is_syncing_membership);
    wait_until(&mut app, &mut rx, "flag released", |a| {
        !a.is_syncing_membership
    })
    .await;
    assert!(app.member_project_ids.is_empty());

    app.refresh_background().await;
    wait_until(&mut app, &mut rx, "membership recovered", |a| {
        a.member_project_ids.contains(&1)
    })
    .await;
    assert!(calls.load(Ordering::SeqCst) >= 2);
}

#[tokio::test]
async fn graphql_429_pauses_a_concurrent_rest_request() {
    let server = MockServer::start().await;
    mount_user(&server).await;
    let data = dataset(1, 1, "opened");
    let mode = mount_graphql(&server, &data).await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/ping"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;
    *mode.lock().unwrap() = 429;
    let c = client(&server).await;
    let http = HttpLayer::new(c.clone());
    let state = GraphqlState::new();
    let res = fetch_merge_requests_via(&c, Some(&http), &state, &[1], "opened", None).await;
    assert!(res.is_err(), "graphql answered 429");

    let paused = tokio::time::timeout(
        Duration::from_millis(400),
        http.get("projects/1/ping", None),
    )
    .await;
    assert!(paused.is_err(), "the REST request waits for the gate");
    let pings = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path().ends_with("/ping"))
        .count();
    assert_eq!(pings, 0);
}

#[tokio::test]
async fn aborting_waiters_releases_the_request_budget() {
    let server = MockServer::start().await;
    mount_user(&server).await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/api/v4/projects/\d+/slow$"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!([]))
                .set_delay(Duration::from_secs(30)),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/99/fast"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;
    let http = HttpLayer::new(client(&server).await);
    let tasks: Vec<_> = (1..=6)
        .map(|i| {
            let h = http.clone();
            tokio::spawn(async move { h.get(&format!("projects/{i}/slow"), None).await })
        })
        .collect();
    let start = Instant::now();
    loop {
        let n = server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .filter(|r| r.url.path().ends_with("/slow"))
            .count();
        if n == 6 {
            break;
        }
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "requests never started"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    for t in &tasks {
        t.abort();
    }
    let fast =
        tokio::time::timeout(Duration::from_secs(8), http.get("projects/99/fast", None)).await;
    assert!(
        matches!(fast, Ok(Outcome::Ok(_))),
        "budget still held by aborted requests"
    );
}
