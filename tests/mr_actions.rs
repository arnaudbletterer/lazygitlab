use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gitlab::GitlabBuilder;
use lazygitlab::models::{App, Author, MergeRequest, Panel};
use tokio::sync::mpsc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_app() -> (
    MockServer,
    App,
    mpsc::UnboundedReceiver<lazygitlab::models::DataUpdate>,
) {
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
    let app = App::new(client, tx);
    (server, app, rx)
}

fn dummy_mr(iid: u64, title: &str) -> MergeRequest {
    MergeRequest {
        iid,
        project_id: 1,
        title: title.into(),
        description: None,
        author: Author::new(1, "u".into()),
        web_url: "u".into(),
        state: "opened".into(),
        references: None,
        upvotes: Some(0),
        downvotes: Some(0),
        labels: vec![],
        milestone: None,
        assignees: vec![],
        reviewers: vec![],
        squash: Some(false),
        created_at: Some("t".into()),
        draft: Some(false),
        diff_refs: None,
        reactions: vec![],
        merge_status: None,
        detailed_merge_status: None,
        has_conflicts: None,
        head_pipeline: None,
        source_branch: "source".into(),
        target_branch: "target".into(),
        participants: vec![],
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
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/1/merge_requests/1/approve"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    app.handle_key(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT))
        .await;

    // Test 'R' shortcut (Unapprove)
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/1/merge_requests/1/unapprove"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    app.handle_key(KeyEvent::new(KeyCode::Char('R'), KeyModifiers::SHIFT))
        .await;

    // Test 'D' shortcut (Draft)
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/merge_requests/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"draft":true})))
        .mount(&server)
        .await;
    app.handle_key(KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT))
        .await;
    assert!(app.selected_mr.as_ref().unwrap().draft.unwrap());

    // Test 'M' shortcut (Merge confirmation popup)
    app.show_confirm_merge_popup = false;
    app.handle_key(KeyEvent::new(KeyCode::Char('M'), KeyModifiers::SHIFT))
        .await;
    assert!(app.show_confirm_merge_popup);

    // Press 'Esc' to cancel merge popup
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
        .await;
    assert!(!app.show_confirm_merge_popup);

    // Press 'M' again, then Enter to confirm merge
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/merge_requests/1/merge"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    app.handle_key(KeyEvent::new(KeyCode::Char('M'), KeyModifiers::SHIFT))
        .await;
    assert!(app.show_confirm_merge_popup);
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .await;
    assert!(!app.show_confirm_merge_popup);

    // Test 'E' shortcut (Rebase confirmation popup)
    app.show_confirm_rebase_popup = false;
    app.handle_key(KeyEvent::new(KeyCode::Char('E'), KeyModifiers::SHIFT))
        .await;
    assert!(app.show_confirm_rebase_popup);

    // Press 'Esc' to cancel rebase popup
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
        .await;
    assert!(!app.show_confirm_rebase_popup);

    // Press 'E' again, then Enter to confirm rebase
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/merge_requests/1/rebase"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    app.handle_key(KeyEvent::new(KeyCode::Char('E'), KeyModifiers::SHIFT))
        .await;
    assert!(app.show_confirm_rebase_popup);
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .await;
    assert!(!app.show_confirm_rebase_popup);

    // Test 'S' shortcut (Squash toggle)
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/merge_requests/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"squash":true})))
        .mount(&server)
        .await;
    app.handle_key(KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT))
        .await;
    assert!(app.selected_mr.as_ref().unwrap().squash.unwrap());

    // Consume updates to ensure no panic and that it triggered refreshes
    let _ = tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while let Ok(update) = rx.try_recv() {
            if let lazygitlab::models::DataUpdate::TriggerRefresh = update {
                break;
            }
        }
    })
    .await;
}
