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

async fn setup_app() -> (
    MockServer,
    App,
    tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
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
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    (server, App::new(client, tx, None), rx)
}

async fn drain(app: &mut App, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
    while let Ok(update) = rx.try_recv() {
        app.process_update_single(update).await;
    }
}

fn branch(name: &str) -> serde_json::Value {
    serde_json::json!({"name": name, "merged": false, "protected": false, "default": name == "main", "web_url": "u"})
}

fn tree_item(path: &str) -> serde_json::Value {
    serde_json::json!({"id": "1", "name": path, "type": "blob", "path": path, "mode": "100644"})
}

#[tokio::test]
async fn test_b_key_fetches_branches_and_opens_popup() {
    let (server, mut app, mut rx) = setup_app().await;
    Mock::given(path("/api/v4/projects/1/repository/branches"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(vec![branch("main"), branch("feature-x")]),
        )
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
    app.view_mode = ViewMode::Tree;

    app.handle_key(key(KeyCode::Char('b'))).await;

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    drain(&mut app, &mut rx).await;
    assert!(
        app.show_tree_branch_popup,
        "'b' should open the tree branch popup"
    );
    assert_eq!(
        app.project_branches.len(),
        2,
        "branches should have been fetched into project_branches"
    );
    assert_eq!(app.tree_branch_state.selected(), Some(0));
}

#[tokio::test]
async fn test_popup_navigation_and_esc() {
    let (_server, mut app, _rx) = setup_app().await;
    app.view_mode = ViewMode::Tree;
    app.project_branches = vec![
        Branch {
            name: "main".into(),
            merged: None,
            protected: None,
            default: Some(true),
            web_url: None,
        },
        Branch {
            name: "feature-x".into(),
            merged: None,
            protected: None,
            default: None,
            web_url: None,
        },
    ];
    app.handle_key(key(KeyCode::Char('b'))).await;
    assert!(app.show_tree_branch_popup);

    app.handle_key(key(KeyCode::Char('j'))).await;
    assert_eq!(app.tree_branch_state.selected(), Some(1), "j moves down");
    app.handle_key(key(KeyCode::Char('k'))).await;
    assert_eq!(app.tree_branch_state.selected(), Some(0), "k moves back up");

    app.handle_key(key(KeyCode::Esc)).await;
    assert!(!app.show_tree_branch_popup, "Esc closes the popup");
    assert_eq!(app.repository_ref, None, "Esc must not change the ref");
}

#[tokio::test]
async fn test_enter_selects_ref_and_refetches_tree() {
    let (server, mut app, _rx) = setup_app().await;
    Mock::given(path("/api/v4/projects/1/repository/tree"))
        .and(query_param("ref", "feature-x"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![tree_item("f.txt")]))
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
    app.repository_tree = vec![];
    app.view_mode = ViewMode::Tree;
    app.project_branches = vec![
        Branch {
            name: "main".into(),
            merged: None,
            protected: None,
            default: Some(true),
            web_url: None,
        },
        Branch {
            name: "feature-x".into(),
            merged: None,
            protected: None,
            default: None,
            web_url: None,
        },
    ];
    app.handle_key(key(KeyCode::Char('b'))).await;
    app.handle_key(key(KeyCode::Char('j'))).await;

    app.handle_key(key(KeyCode::Enter)).await;

    assert!(!app.show_tree_branch_popup, "Enter closes the popup");
    assert_eq!(app.repository_ref, Some("feature-x".to_string()));
    assert_eq!(
        app.status_message.as_deref(),
        Some("Browsing tree at 'feature-x'")
    );
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let requests = server.received_requests().await.unwrap();
    let urls: Vec<String> = requests
        .iter()
        .map(|r| format!("{}?{}", r.url.path(), r.url.query().unwrap_or("")))
        .collect();
    assert!(
        urls.iter()
            .any(|u| u.contains("/projects/1/repository/tree") && u.contains("ref=feature-x")),
        "tree fetch must be scoped to the selected ref, got: {:?}",
        urls
    );
}

#[tokio::test]
async fn test_file_fetch_uses_selected_ref() {
    let (server, mut app, _rx) = setup_app().await;
    Mock::given(path("/api/v4/projects/1/repository/files/f.txt"))
        .and(query_param("ref", "feature-x"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "file_name": "f.txt", "file_path": "f.txt", "size": 5, "encoding": "base64",
            "content_sha256": "x", "ref": "feature-x", "blob_id": "b", "commit_id": "c",
            "last_commit_id": "c", "content": "aGVsbG8K"
        })))
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
    app.repository_ref = Some("feature-x".to_string());

    app.fetch_repository_file("f.txt".into()).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let requests = server.received_requests().await.unwrap();
    let urls: Vec<String> = requests
        .iter()
        .map(|r| format!("{}?{}", r.url.path(), r.url.query().unwrap_or("")))
        .collect();
    assert!(
        urls.iter()
            .any(|u| u.contains("/repository/files/f.txt") && u.contains("ref=feature-x")),
        "file fetch must use the browsed ref, got: {:?}",
        urls
    );
}
