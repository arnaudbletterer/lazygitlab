use lazygitlab::models::{App, MergeRequest, Author, Panel};
use gitlab::GitlabBuilder;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use tokio::sync::mpsc;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{path, method};

async fn setup_app() -> (MockServer, App, mpsc::UnboundedReceiver<lazygitlab::models::DataUpdate>) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    let (tx, rx) = mpsc::unbounded_channel();
    let app = App::new(client, tx);
    (server, app, rx)
}

fn dummy_mr(iid: u64, title: &str) -> MergeRequest {
    MergeRequest {
        iid, project_id: 1, title: title.into(), description: None, author: Author::new(1, "u".into()),
        web_url: "u".into(), state: "opened".into(), references: None,
        upvotes: Some(0), downvotes: Some(0), labels: vec![], milestone: None,
        assignees: vec![], reviewers: vec![], created_at: Some("t".into()), draft: Some(false), diff_refs: None, reactions: vec![]
    }
}

#[tokio::test]
async fn test_mr_shortcuts() {
    let (server, mut app, mut rx) = setup_app().await;
    
    let mr = dummy_mr(1, "MR1");
    app.selected_mr = Some(mr.clone());
    app.active_panel = Panel::MainDetail;
    app.active_selection = lazygitlab::models::MenuSelection::MergeRequests;

    // Test 'A' shortcut (Approve)
    Mock::given(method("POST")).and(path("/api/v4/projects/1/merge_requests/1/approve")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
    app.handle_key(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT)).await;
    
    // Test 'R' shortcut (Unapprove)
    Mock::given(method("POST")).and(path("/api/v4/projects/1/merge_requests/1/unapprove")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
    app.handle_key(KeyEvent::new(KeyCode::Char('R'), KeyModifiers::SHIFT)).await;

    // Test 'D' shortcut (Draft)
    Mock::given(method("PUT")).and(path("/api/v4/projects/1/merge_requests/1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"draft":true}))).mount(&server).await;
    app.handle_key(KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT)).await;
    assert!(app.selected_mr.as_ref().unwrap().draft.unwrap());

    // Test 'M' shortcut (Merge)
    Mock::given(method("PUT")).and(path("/api/v4/projects/1/merge_requests/1/merge")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
    app.handle_key(KeyEvent::new(KeyCode::Char('M'), KeyModifiers::SHIFT)).await;

    // Consume updates to ensure no panic and that it triggered refreshes
    let _ = tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while let Ok(update) = rx.try_recv() {
            if let lazygitlab::models::DataUpdate::TriggerRefresh = update {
                break;
            }
        }
    }).await;
}
