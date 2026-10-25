use lazygitlab::client::builder;
use lazygitlab::config::Transport;
use lazygitlab::graphql::{
    GraphqlError, current_batch_size, fetch_issues, fetch_merge_requests, is_marked_unsupported,
    last_complexity,
};
use serde_json::{Value, json};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

const TOKEN: &str = "glpat-graphqlToken";

async fn client(server: &MockServer) -> gitlab::AsyncGitlab {
    Mock::given(path("/api/v4/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":1,"username":"u"})))
        .mount(server)
        .await;
    builder(
        server.uri().replace("http://", ""),
        TOKEN.into(),
        Transport::PlainHttp,
    )
    .build_async()
    .await
    .expect("client")
}

fn body(req: &Request) -> Value {
    serde_json::from_slice(&req.body).unwrap()
}

fn ids_of(req: &Request) -> Vec<u64> {
    body(req)["variables"]["ids"]
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
        .collect()
}

fn mr_node(pid: u64) -> Value {
    json!({
        "iid": "7", "title": format!("mr {pid}"), "state": "opened", "draft": true,
        "webUrl": format!("https://gl/g/p{pid}/-/merge_requests/7"),
        "createdAt": "2026-01-01T00:00:00Z", "updatedAt": "2026-01-02T00:00:00Z",
        "detailedMergeStatus": "NOT_APPROVED", "conflicts": true,
        "sourceBranch": "feat", "targetBranch": "main",
        "author": {"id": "gid://gitlab/User/11", "username": "alice", "name": "Alice"},
        "assignees": {"nodes": [{"id": "gid://gitlab/User/12", "username": "bob"}]},
        "reviewers": {"nodes": [{"id": "gid://gitlab/User/13", "username": "carol"}]},
        "labels": {"nodes": [{"title": "bug", "color": "#ff0000", "textColor": "#ffffff"}]},
        "milestone": {"title": "v1"},
        "headPipeline": {"status": "SUCCESS"},
        "approved": false, "approvalsLeft": 2,
        "resolvableDiscussionsCount": 5, "resolvedDiscussionsCount": 3
    })
}

fn issue_node(pid: u64) -> Value {
    json!({
        "iid": "3", "title": format!("issue {pid}"), "state": "opened",
        "webUrl": format!("https://gl/g/p{pid}/-/issues/3"),
        "createdAt": "2026-02-01T00:00:00Z", "updatedAt": "2026-02-02T00:00:00Z",
        "dueDate": "2026-03-01", "userNotesCount": 4,
        "author": {"id": "gid://gitlab/User/21", "username": "dave", "name": "Dave"},
        "assignees": {"nodes": [{"id": "gid://gitlab/User/22", "username": "erin"}]},
        "labels": {"nodes": [{"title": "ux", "color": "#00ff00", "textColor": "#000000"}]},
        "milestone": {"title": "v2"}
    })
}

fn project(pid: u64, field: &str, nodes: Vec<Value>, next: Option<&str>) -> Value {
    json!({
        "id": format!("gid://gitlab/Project/{pid}"),
        "fullPath": format!("g/p{pid}"),
        field: {
            "pageInfo": {"hasNextPage": next.is_some(), "endCursor": next},
            "nodes": nodes
        }
    })
}

fn envelope(projects: Vec<Value>, score: u64) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "data": {
            "projects": {"nodes": projects},
            "queryComplexity": {"score": score, "limit": 300}
        }
    }))
}

struct PerProject {
    field: &'static str,
    score: u64,
}

impl Respond for PerProject {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        let projects = ids_of(req)
            .into_iter()
            .map(|pid| {
                let node = if self.field == "issues" {
                    issue_node(pid)
                } else {
                    mr_node(pid)
                };
                project(pid, self.field, vec![node], None)
            })
            .collect();
        envelope(projects, self.score)
    }
}

async fn mount_graphql(server: &MockServer, r: impl Respond + 'static) {
    Mock::given(method("POST"))
        .and(path("/api/graphql"))
        .respond_with(r)
        .mount(server)
        .await;
}

async fn graphql_requests(server: &MockServer) -> Vec<Request> {
    server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|r| r.url.path() == "/api/graphql")
        .collect()
}

#[tokio::test]
async fn forty_five_projects_use_three_requests_and_map_every_field() {
    let server = MockServer::start().await;
    mount_graphql(
        &server,
        PerProject {
            field: "mergeRequests",
            score: 100,
        },
    )
    .await;
    let c = client(&server).await;
    let ids: Vec<u64> = (1..=45).collect();

    let res = fetch_merge_requests(&c, &ids, "opened", Some("2026-01-01T00:00:00Z"))
        .await
        .unwrap();

    assert!(res.complete);
    assert_eq!(res.rows.len(), 45);
    assert_eq!(graphql_requests(&server).await.len(), 3);
    let row = res.rows.iter().find(|r| r.mr.project_id == 30).unwrap();
    assert_eq!(row.path_with_namespace, "g/p30");
    assert_eq!(row.mr.iid, 7);
    assert_eq!(row.mr.title, "mr 30");
    assert_eq!(row.mr.state, "opened");
    assert_eq!(row.mr.draft, Some(true));
    assert_eq!(row.mr.web_url, "https://gl/g/p30/-/merge_requests/7");
    assert_eq!(row.mr.created_at.as_deref(), Some("2026-01-01T00:00:00Z"));
    assert_eq!(row.updated_at.as_deref(), Some("2026-01-02T00:00:00Z"));
    assert_eq!(
        row.mr.detailed_merge_status.as_deref(),
        Some("not_approved")
    );
    assert_eq!(row.mr.has_conflicts, Some(true));
    assert_eq!(row.mr.source_branch, "feat");
    assert_eq!(row.mr.target_branch, "main");
    assert_eq!(row.mr.author.id, 11);
    assert_eq!(row.mr.author.username, "alice");
    assert_eq!(row.mr.author.name.as_deref(), Some("Alice"));
    assert_eq!(row.mr.assignees.len(), 1);
    assert_eq!(row.mr.assignees[0].id, 12);
    assert_eq!(row.mr.assignees[0].username, "bob");
    assert_eq!(row.mr.reviewers[0].id, 13);
    assert_eq!(row.mr.reviewers[0].username, "carol");
    assert_eq!(row.mr.labels[0].name, "bug");
    assert_eq!(row.mr.labels[0].color, "#ff0000");
    assert_eq!(row.mr.labels[0].text_color.as_deref(), Some("#ffffff"));
    assert_eq!(row.mr.milestone.as_ref().unwrap().title, "v1");
    assert_eq!(row.mr.head_pipeline.as_ref().unwrap().status, "success");
    assert_eq!(row.mr.references.as_ref().unwrap().full, "g/p30!7");
    assert_eq!(row.approved, Some(false));
    assert_eq!(row.approvals_left, Some(2));
    assert_eq!(row.resolvable_discussions, Some(5));
    assert_eq!(row.resolved_discussions, Some(3));
    assert_eq!(row.unresolved_discussions(), Some(2));
    let c = last_complexity(&c).unwrap();
    assert_eq!((c.score, c.limit), (100, 300));
}

#[tokio::test]
async fn issues_map_every_field() {
    let server = MockServer::start().await;
    mount_graphql(
        &server,
        PerProject {
            field: "issues",
            score: 50,
        },
    )
    .await;
    let c = client(&server).await;

    let res = fetch_issues(&c, &[9], "opened", None).await.unwrap();

    assert!(res.complete);
    let row = &res.rows[0];
    assert_eq!(row.path_with_namespace, "g/p9");
    assert_eq!(row.issue.project_id, 9);
    assert_eq!(row.issue.iid, 3);
    assert_eq!(row.issue.title, "issue 9");
    assert_eq!(row.issue.state, "opened");
    assert_eq!(row.issue.web_url, "https://gl/g/p9/-/issues/3");
    assert_eq!(
        row.issue.created_at.as_deref(),
        Some("2026-02-01T00:00:00Z")
    );
    assert_eq!(row.updated_at.as_deref(), Some("2026-02-02T00:00:00Z"));
    assert_eq!(row.issue.due_date.as_deref(), Some("2026-03-01"));
    assert_eq!(row.user_notes_count, Some(4));
    assert_eq!(row.issue.author.id, 21);
    assert_eq!(row.issue.author.username, "dave");
    assert_eq!(row.issue.assignees[0].id, 22);
    assert_eq!(row.issue.labels[0].name, "ux");
    assert_eq!(row.issue.milestone.as_ref().unwrap().title, "v2");
    assert_eq!(row.issue.references.as_ref().unwrap().full, "g/p9#3");
}

struct Paged;

impl Respond for Paged {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        match body(req)["variables"]["after"].as_str() {
            None => envelope(
                vec![project(1, "mergeRequests", vec![mr_node(1)], Some("CUR1"))],
                10,
            ),
            Some("CUR1") => {
                let mut second = mr_node(1);
                second["iid"] = json!("8");
                envelope(vec![project(1, "mergeRequests", vec![second], None)], 10)
            }
            Some(other) => panic!("unexpected cursor {other}"),
        }
    }
}

#[tokio::test]
async fn has_next_page_follows_the_cursor() {
    let server = MockServer::start().await;
    mount_graphql(&server, Paged).await;
    let c = client(&server).await;

    let res = fetch_merge_requests(&c, &[1], "opened", None)
        .await
        .unwrap();

    let mut iids: Vec<u64> = res.rows.iter().map(|r| r.mr.iid).collect();
    iids.sort();
    assert_eq!(iids, vec![7, 8]);
    assert!(res.complete);
    assert_eq!(graphql_requests(&server).await.len(), 2);
}

#[tokio::test]
async fn unknown_field_is_unsupported_and_remembered() {
    let server = MockServer::start().await;
    mount_graphql(
        &server,
        ResponseTemplate::new(200).set_body_json(json!({
            "errors": [{"message": "Field 'approvalsLeft' doesn't exist on type 'MergeRequest'"}]
        })),
    )
    .await;
    let c = client(&server).await;

    let first = fetch_merge_requests(&c, &[1, 2], "opened", None).await;
    assert_eq!(first.unwrap_err(), GraphqlError::Unsupported);
    assert!(is_marked_unsupported(&c));
    let before = graphql_requests(&server).await.len();
    let second = fetch_issues(&c, &[1, 2], "opened", None).await;
    assert_eq!(second.unwrap_err(), GraphqlError::Unsupported);
    assert_eq!(graphql_requests(&server).await.len(), before);
}

#[tokio::test]
async fn http_404_is_unsupported() {
    let server = MockServer::start().await;
    mount_graphql(&server, ResponseTemplate::new(404)).await;
    let c = client(&server).await;
    let r = fetch_merge_requests(&c, &[1], "opened", None).await;
    assert_eq!(r.unwrap_err(), GraphqlError::Unsupported);
}

struct FailOne;

impl Respond for FailOne {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        if ids_of(req).contains(&25) {
            return ResponseTemplate::new(500);
        }
        PerProject {
            field: "mergeRequests",
            score: 10,
        }
        .respond(req)
    }
}

#[tokio::test]
async fn partial_failure_is_incomplete_but_keeps_other_rows() {
    let server = MockServer::start().await;
    mount_graphql(&server, FailOne).await;
    let c = client(&server).await;
    let ids: Vec<u64> = (1..=45).collect();

    let res = fetch_merge_requests(&c, &ids, "opened", None)
        .await
        .unwrap();

    assert!(!res.complete);
    assert_eq!(res.rows.len(), 25);
}

#[tokio::test]
async fn graphql_errors_alongside_data_are_incomplete() {
    let server = MockServer::start().await;
    mount_graphql(
        &server,
        ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "projects": {"nodes": [project(1, "mergeRequests", vec![mr_node(1)], None)]},
                "queryComplexity": {"score": 1, "limit": 300}
            },
            "errors": [{"message": "something transient"}]
        })),
    )
    .await;
    let c = client(&server).await;
    let res = fetch_merge_requests(&c, &[1], "opened", None)
        .await
        .unwrap();
    assert!(!res.complete);
    assert_eq!(res.rows.len(), 1);
}

#[tokio::test]
async fn high_complexity_shrinks_the_next_batch() {
    let server = MockServer::start().await;
    mount_graphql(
        &server,
        PerProject {
            field: "mergeRequests",
            score: 250,
        },
    )
    .await;
    let c = client(&server).await;
    let ids: Vec<u64> = (1..=45).collect();
    assert_eq!(current_batch_size(&c), 20);

    fetch_merge_requests(&c, &ids, "opened", None)
        .await
        .unwrap();
    assert_eq!(graphql_requests(&server).await.len(), 3);
    assert_eq!(current_batch_size(&c), 10);

    fetch_merge_requests(&c, &ids, "opened", None)
        .await
        .unwrap();
    assert_eq!(graphql_requests(&server).await.len(), 3 + 5);
    assert_eq!(current_batch_size(&c), 5);
}

#[tokio::test]
async fn too_many_requests_is_rate_limited_without_retry() {
    let server = MockServer::start().await;
    mount_graphql(&server, ResponseTemplate::new(429)).await;
    let c = client(&server).await;
    let r = fetch_merge_requests(&c, &[1], "opened", None).await;
    assert_eq!(r.unwrap_err(), GraphqlError::RateLimited);
    assert_eq!(graphql_requests(&server).await.len(), 1);
    assert!(!is_marked_unsupported(&c));
}

#[tokio::test]
async fn bearer_header_sent_and_query_has_no_interpolated_ids() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/graphql"))
        .and(header("authorization", format!("Bearer {TOKEN}").as_str()))
        .respond_with(PerProject {
            field: "mergeRequests",
            score: 1,
        })
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server).await;

    fetch_merge_requests(&c, &[424242], "opened", Some("2026-01-01T00:00:00Z"))
        .await
        .unwrap();

    let reqs = graphql_requests(&server).await;
    let b = body(&reqs[0]);
    let query = b["query"].as_str().unwrap();
    assert!(!query.contains("424242"));
    assert!(!query.contains("gid://"));
    assert!(!query.contains("2026-01-01"));
    assert!(!query.contains("opened"));
    assert_eq!(
        b["variables"]["ids"],
        json!(["gid://gitlab/Project/424242"])
    );
    assert_eq!(b["variables"]["state"], "opened");
    assert!(!reqs[0].headers.contains_key("private-token"));
}
