use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gitlab::GitlabBuilder;
use lazygitlab::models::{App, Board, Context, DataUpdate, Namespace, Panel, Project, ViewMode};
use lazygitlab::ui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use tokio::sync::mpsc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_app() -> (MockServer, App, mpsc::UnboundedReceiver<DataUpdate>) {
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

#[tokio::test]
async fn test_fetch_boards() {
    let (server, mut app, mut rx) = setup_app().await;
    let project_id = 123;
    app.selected_project = Some(Project {
        id: project_id,
        name: "p".into(),
        path_with_namespace: Some("n/p".into()),
        web_url: Some("u".into()),
        default_branch: None,
        namespace: None,
    });

    Mock::given(method("GET"))
        .and(path(format!("/api/v4/projects/{}/boards", project_id)))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([{"id": 1, "name": "Main Board"}])),
        )
        .mount(&server)
        .await;

    app.fetch_boards().await.unwrap();
    let _ = rx.recv().await.unwrap(); // Loading(true)
    let update = rx.recv().await.unwrap();
    if let DataUpdate::Boards {
        project_id: pid,
        data,
    } = update
    {
        assert_eq!(pid, project_id);
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].name, "Main Board");
    } else {
        panic!("Expected DataUpdate::Boards");
    }
}

#[tokio::test]
async fn test_fetch_board_lists_and_issues() {
    let (server, mut app, mut rx) = setup_app().await;
    let project_id = 123;
    let board_id = 1;
    app.selected_project = Some(Project {
        id: project_id,
        name: "p".into(),
        path_with_namespace: Some("n/p".into()),
        web_url: Some("u".into()),
        default_branch: None,
        namespace: None,
    });

    Mock::given(method("GET"))
        .and(path(format!("/api/v4/projects/{}/boards/{}/lists", project_id, board_id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([{"id": 10, "label": {"id": 100, "name": "todo", "color": "#ff0000"}, "position": 1, "list_type": "label"}])))
        .mount(&server)
        .await;

    app.fetch_board_lists(board_id).await.unwrap();
    let _ = rx.recv().await.unwrap(); // Loading(true)
    let update = rx.recv().await.unwrap();
    if let DataUpdate::BoardLists { data, .. } = update {
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].id, 10);
    } else {
        panic!("Expected DataUpdate::BoardLists");
    }
}

#[tokio::test]
async fn test_fetch_board_list_issues() {
    let (server, mut app, mut rx) = setup_app().await;
    let project_id = 123;
    let board_id = 1;
    let list_id = 10;
    app.selected_project = Some(Project {
        id: project_id,
        name: "p".into(),
        path_with_namespace: Some("n/p".into()),
        web_url: Some("u".into()),
        default_branch: None,
        namespace: None,
    });

    Mock::given(method("GET"))
        .and(path(format!("/api/v4/projects/{}/boards/{}/lists/{}/issues", project_id, board_id, list_id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([{"iid": 1, "project_id": project_id, "title": "I1", "state": "opened", "web_url": "u", "author": {"id": 1, "username": "u1"}, "assignees": [], "labels": []}])))
        .mount(&server)
        .await;

    app.fetch_board_list_issues(board_id, list_id)
        .await
        .unwrap();
    let _ = rx.recv().await.unwrap(); // Loading(true)
    let update = rx.recv().await.unwrap();
    if let DataUpdate::BoardIssues {
        list_id: lid, data, ..
    } = update
    {
        assert_eq!(lid, list_id);
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].title, "I1");
    } else {
        panic!("Expected DataUpdate::BoardIssues");
    }
}

#[tokio::test]
async fn test_ui_board_navigation() {
    let (server, mut app, mut rx) = setup_app().await;
    let backend = TestBackend::new(100, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let project_id = 123;
    app.selected_project = Some(Project {
        id: project_id,
        name: "p".into(),
        path_with_namespace: Some("n/p".into()),
        web_url: Some("u".into()),
        default_branch: None,
        namespace: None,
    });
    app.active_context = Context::Project;
    app.active_panel = Panel::Project;
    app.project_menu_state.select(Some(6)); // Boards

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .await;
    app.update_preview().await;

    app.project_boards = vec![Board {
        id: 1,
        name: "B1".into(),
        lists: None,
    }];
    app.project_boards_state.select(Some(0));

    Mock::given(method("GET"))
        .and(path(format!("/api/v4/projects/{}/boards/1/lists", project_id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([{"id": 10, "label": {"id": 100, "name": "todo", "color": "#ff0000"}, "position": 1, "list_type": "label"}])))
        .mount(&server)
        .await;

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .await;

    let start = std::time::Instant::now();
    while app.view_mode != ViewMode::Board && start.elapsed().as_secs() < 5 {
        app.process_updates(&mut rx).await;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    assert_eq!(app.view_mode, ViewMode::Board);
    terminal.draw(|f| ui(f, &mut app)).unwrap();
}

#[tokio::test]
async fn test_ui_epics_view() {
    let (server, mut app, mut rx) = setup_app().await;
    let backend = TestBackend::new(100, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let group_id = 456;
    app.selected_project = Some(Project {
        id: 123,
        name: "p".into(),
        path_with_namespace: Some("n/p".into()),
        web_url: Some("u".into()),
        default_branch: None,
        namespace: Some(Namespace {
            id: group_id,
            name: "g".into(),
            path: "g".into(),
            kind: "group".into(),
            full_path: "g".into(),
        }),
    });
    app.active_context = Context::Project;
    app.active_panel = Panel::Project;
    app.project_menu_state.select(Some(7)); // Epics

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .await;
    app.update_preview().await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v4/groups/{}/epics", group_id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([{"id": 1, "iid": 1, "title": "Epic 1", "state": "opened", "web_url": "url"}])))
        .mount(&server)
        .await;

    let start = std::time::Instant::now();
    while app.epics.is_empty() && start.elapsed().as_secs() < 5 {
        app.process_updates(&mut rx).await;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    assert!(!app.epics.is_empty());
    terminal.draw(|f| ui(f, &mut app)).unwrap();
}
