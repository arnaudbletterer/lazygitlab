use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use wiremock::matchers::{path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

async fn setup_app() -> (MockServer, App) {
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
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    (server, App::new(client, tx, None))
}

fn mr(iid: u64) -> serde_json::Value {
    serde_json::json!({"iid": iid, "project_id": 1, "title": format!("MR {}", iid), "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "source_branch": "a", "target_branch": "b", "draft": false})
}

#[tokio::test]
async fn test_bracket_cycles_through_review_and_assigned() {
    let (_server, mut app) = setup_app().await;
    app.active_context = Context::Dashboard;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::MergeRequests;
    app.mr_filter = MergeRequestFilter::Opened;

    app.handle_key(key(KeyCode::Char(']'))).await;
    assert_eq!(app.mr_filter, MergeRequestFilter::Review);
    app.handle_key(key(KeyCode::Char(']'))).await;
    assert_eq!(app.mr_filter, MergeRequestFilter::Assigned);
    app.handle_key(key(KeyCode::Char('['))).await;
    assert_eq!(app.mr_filter, MergeRequestFilter::Review);
    app.handle_key(key(KeyCode::Char('['))).await;
    assert_eq!(app.mr_filter, MergeRequestFilter::Opened);
}

#[tokio::test]
async fn test_assigned_filter_sends_assignee_id() {
    let (server, mut app) = setup_app().await;
    Mock::given(path("/api/v4/projects/1/merge_requests"))
        .and(query_param("assignee_id", "42"))
        .and(query_param("state", "opened"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![mr(10)]))
        .mount(&server)
        .await;
    Mock::given(path("/api/v4/projects/1/merge_requests"))
        .and(query_param("reviewer_id", "42"))
        .and(query_param("state", "opened"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![mr(11)]))
        .mount(&server)
        .await;
    app.active_context = Context::Project;
    app.selected_project = Some(Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    });
    app.current_user = Some(Author::new(42, "me".into()));

    app.mr_filter = MergeRequestFilter::Assigned;
    app.fetch_merge_requests().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    app.mr_filter = MergeRequestFilter::Review;
    app.fetch_merge_requests().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let requests = server.received_requests().await.unwrap();
    let urls: Vec<String> = requests
        .iter()
        .filter(|r| r.url.path().contains("merge_requests"))
        .map(|r| format!("{}?{}", r.url.path(), r.url.query().unwrap_or("")))
        .collect();
    assert!(
        urls.iter()
            .any(|u| u.contains("assignee_id=42") && u.contains("state=opened")),
        "assigned segment queries by assignee_id, got: {:?}",
        urls
    );
    assert!(
        urls.iter()
            .any(|u| u.contains("reviewer_id=42") && u.contains("state=opened")),
        "review segment queries by reviewer_id, got: {:?}",
        urls
    );
}
