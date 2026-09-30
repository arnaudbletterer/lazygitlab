use base64::{Engine as _, engine::general_purpose};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use wiremock::matchers::{method, path};
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
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"dev"})),
        )
        .mount(&server)
        .await;
    let client = GitlabBuilder::new(server.uri().replace("http://", ""), "test-token")
        .insecure()
        .build_async()
        .await
        .unwrap();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    (server, App::new(client, tx, None), rx)
}

fn sample_project(id: u64, name: &str) -> Project {
    Project {
        id,
        name: name.into(),
        web_url: Some(format!("https://gitlab.example.com/{}", name)),
        path_with_namespace: Some(format!("group/{}", name)),
        default_branch: Some("main".into()),
        namespace: None,
    }
}

#[tokio::test]
async fn test_repository_tree_traversal_and_space_expand() {
    let (_server, mut app, _rx) = setup_app().await;
    app.selected_project = Some(sample_project(1, "repo-1"));
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Repository;
    app.active_panel = Panel::Main;

    app.repository_tree = vec![
        RepositoryTreeItem {
            id: "t1".into(),
            name: "src".into(),
            r#type: "tree".into(),
            path: "src".into(),
            mode: "040000".into(),
        },
        RepositoryTreeItem {
            id: "b1".into(),
            name: "main.rs".into(),
            r#type: "blob".into(),
            path: "src/main.rs".into(),
            mode: "100644".into(),
        },
    ];
    app.repository_tree_state.select(vec!["src".into()]);

    let backend = ratatui::backend::TestBackend::new(120, 30);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|f| lazygitlab::ui::ui(f, &mut app)).unwrap();

    // Press Space on directory node in repository tree
    app.handle_key(key(KeyCode::Char(' '))).await;
    // View mode should remain list (not opening a blob)
    assert_eq!(app.view_mode, ViewMode::List);

    // Press 'j' to navigate down to blob
    app.handle_key(key(KeyCode::Char('j'))).await;
    assert_eq!(
        app.repository_tree_state.selected(),
        &["src/main.rs".to_string()]
    );
}

#[tokio::test]
async fn test_branch_switcher_from_repository_tree() {
    let (server, mut app, mut rx) = setup_app().await;
    app.selected_project = Some(sample_project(1, "repo-1"));
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Repository;
    app.active_panel = Panel::Main;
    app.view_mode = ViewMode::List;

    app.project_branches = vec![
        Branch {
            name: "main".into(),
            merged: Some(false),
            protected: Some(true),
            default: Some(true),
            web_url: Some("https://gitlab.example.com/repo-1/-/tree/main".into()),
        },
        Branch {
            name: "feature-auth".into(),
            merged: Some(false),
            protected: Some(false),
            default: Some(false),
            web_url: Some("https://gitlab.example.com/repo-1/-/tree/feature-auth".into()),
        },
    ];

    // Press 'b' while in repository tree view (ViewMode::List)
    app.handle_key(key(KeyCode::Char('b'))).await;
    assert!(
        app.show_tree_branch_popup,
        "Branch popup must open from repository list"
    );

    // Select second branch 'feature-auth' and press Enter
    app.tree_branch_state.select(Some(1));

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/repository/tree"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    app.handle_key(key(KeyCode::Enter)).await;
    assert!(!app.show_tree_branch_popup);
    assert_eq!(app.repository_ref.as_deref(), Some("feature-auth"));
    assert_eq!(app.view_mode, ViewMode::List, "Must return to tree view");

    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    while let Ok(update) = rx.try_recv() {
        app.process_update_single(update).await;
    }
}

#[tokio::test]
async fn test_empty_repository_branch_ui() {
    let (_server, mut app, _rx) = setup_app().await;
    app.selected_project = Some(sample_project(1, "repo-1"));
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Repository;
    app.active_panel = Panel::Main;
    app.repository_tree = vec![]; // Empty branch

    let backend = ratatui::backend::TestBackend::new(120, 30);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|f| lazygitlab::ui::ui(f, &mut app)).unwrap();
    let frame = lazygitlab::headless::render_to_string(&terminal);

    assert!(frame.contains("No files found on this branch or repository is empty."));
    assert!(frame.contains("switch to another branch"));
}

#[tokio::test]
async fn test_project_quick_switcher_clears_previous_project_state() {
    let (server, mut app, mut rx) = setup_app().await;
    app.selected_project = Some(sample_project(1, "repo-alpha"));
    app.repository_ref = Some("feature-legacy".into());
    app.repository_tree = vec![RepositoryTreeItem {
        id: "1".into(),
        name: "old_alpha.txt".into(),
        r#type: "blob".into(),
        path: "old_alpha.txt".into(),
        mode: "100644".into(),
    }];
    app.project_branches = vec![Branch {
        name: "feature-legacy".into(),
        merged: None,
        protected: None,
        default: Some(false),
        web_url: None,
    }];
    app.selected_file_content = Some(RepositoryFile {
        file_name: "old_alpha.txt".into(),
        file_path: "old_alpha.txt".into(),
        size: 10,
        encoding: "base64".into(),
        content_sha256: "".into(),
        ref_name: "feature-legacy".into(),
        blob_id: "b1".into(),
        commit_id: "".into(),
        last_commit_id: "".into(),
        content: general_purpose::STANDARD.encode("alpha content"),
    });

    // Press 'p' to open Project Switcher
    app.handle_key(key(KeyCode::Char('p'))).await;
    assert!(app.show_project_popup);

    // Populate projects list with repo-beta
    let project_beta = sample_project(2, "repo-beta");
    app.projects = vec![sample_project(1, "repo-alpha"), project_beta.clone()];
    app.projects_state.select(Some(1));

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/2/merge_requests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    // Hit Enter to select repo-beta
    app.handle_key(key(KeyCode::Enter)).await;
    assert_eq!(app.selected_project.as_ref().map(|p| p.id), Some(2));
    assert_eq!(
        app.repository_ref, None,
        "Previous project's branch ref must be cleared"
    );
    assert!(
        app.repository_tree.is_empty(),
        "Previous repository tree must be cleared"
    );
    assert!(
        app.project_branches.is_empty(),
        "Previous branches list must be cleared"
    );
    assert!(
        app.selected_file_content.is_none(),
        "Previous file content must be cleared"
    );

    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    while let Ok(update) = rx.try_recv() {
        app.process_update_single(update).await;
    }
}

#[tokio::test]
async fn test_large_file_safeguard_and_clamped_scrolling() {
    let (_server, mut app, _rx) = setup_app().await;
    app.view_mode = ViewMode::Tree;

    // Create a mock large file content (> 500KB)
    let large_text = "fn compute() -> i32 { 42 }\n".repeat(25_000);
    assert!(large_text.len() > 500_000);

    let file = RepositoryFile {
        file_name: "huge.rs".into(),
        file_path: "huge.rs".into(),
        size: large_text.len() as u64,
        encoding: "base64".into(),
        content_sha256: "hash".into(),
        ref_name: "main".into(),
        blob_id: "blob-huge".into(),
        commit_id: "c1".into(),
        last_commit_id: "c1".into(),
        content: general_purpose::STANDARD.encode(&large_text),
    };
    app.selected_file_content = Some(file);
    app.file_viewport_height = 20;
    app.file_scroll = 0;

    let backend = ratatui::backend::TestBackend::new(100, 30);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|f| lazygitlab::ui::ui(f, &mut app)).unwrap();
    let frame = lazygitlab::headless::render_to_string(&terminal);

    assert!(frame.contains("[Large file: syntax highlighting disabled for performance]"));

    // Test clamped scrolling
    app.file_scroll = 50_000;
    terminal.draw(|f| lazygitlab::ui::ui(f, &mut app)).unwrap();
    assert!(
        app.file_scroll <= 25_001,
        "Scroll must be clamped to maximum lines"
    );
}

#[tokio::test]
async fn test_local_git_status_rendering() {
    let (_server, mut app, _rx) = setup_app().await;
    app.selected_project = Some(sample_project(1, "repo-1"));
    app.active_selection = MenuSelection::Repository;
    app.active_context = Context::Project;
    app.project_menu_state.select(Some(3));

    // Set dirty status
    app.local_git_status = Some(LocalGitStatus {
        branch: "feature-test".into(),
        is_dirty: true,
    });

    let backend = ratatui::backend::TestBackend::new(140, 40);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|f| lazygitlab::ui::ui(f, &mut app)).unwrap();
    let frame = lazygitlab::headless::render_to_string(&terminal);

    assert!(frame.contains("● Dirty"));

    // Set clean status
    app.local_git_status = Some(LocalGitStatus {
        branch: "main".into(),
        is_dirty: false,
    });

    terminal.draw(|f| lazygitlab::ui::ui(f, &mut app)).unwrap();
    let frame_clean = lazygitlab::headless::render_to_string(&terminal);

    assert!(frame_clean.contains("● Clean"));
}
