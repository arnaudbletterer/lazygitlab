use lazygitlab::models::{App, MenuSelection, Project, Issue, GroupedItem, Panel, Context, HelpAction, Author, Todo};
use gitlab::GitlabBuilder;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use tokio::sync::mpsc;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::path;

async fn setup_app() -> (MockServer, App) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    let app = App::new(client);
    (server, app)
}

fn dummy_author() -> Author {
    Author { id: 1, username: "u".into() }
}

fn dummy_issue(iid: u64, title: &str) -> Issue {
    Issue {
        iid, project_id: 1, title: title.into(), description: None, author: dummy_author(),
        assignees: vec![], state: "opened".into(), web_url: "u".into(), references: None,
        issue_type: None, labels: vec![], milestone: None, due_date: None, weight: None,
        task_completion_status: None, confidential: false, created_at: "t".into(),
        upvotes: Some(0), downvotes: Some(0), reactions: vec![]
    }
}

fn dummy_todo(id: u64, body: &str) -> Todo {
    Todo {
        id, project: Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()) },
        action_name: "marked".into(), target_type: "Issue".into(),
        target_url: format!("u/{}", id), body: body.into(), state: "pending".into(), created_at: "t".into()
    }
}

#[tokio::test]
async fn test_boundary_looping_and_context() {
    let (_server, mut app) = setup_app().await;
    let (tx, _) = mpsc::unbounded_channel();
    app.tx = tx;

    // 1. Test Sidebar Dashboard -> Explore transition (Down at bottom)
    app.active_panel = Panel::Dashboard;
    app.dashboard_menu_state.select(Some(2)); // Last item (Issues)
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    assert_eq!(app.active_panel, Panel::Explore); // Moves to Explore
    assert_eq!(app.explore_menu_state.selected(), Some(0));

    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)).await;
    assert_eq!(app.active_panel, Panel::Dashboard); // Moves back to Dashboard
    assert_eq!(app.dashboard_menu_state.selected(), Some(2));

    // 2. Test Main List Looping (Main list STILL loops)
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Issues;
    app.filtered_issues = vec![
        GroupedItem::Item(dummy_issue(1, "I1")),
        GroupedItem::Item(dummy_issue(2, "I2")),
    ];
    app.issues_state.select(Some(1)); // Last item
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    assert_eq!(app.issues_state.selected(), Some(0)); // Loop to I1

    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)).await;
    assert_eq!(app.issues_state.selected(), Some(1)); // Loop back to I2

    // 3. Test Contextual Back Navigation (Esc/Left)
    app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()) });
    app.active_panel = Panel::Project; // Focused Project Sidebar
    app.update_preview().await;
    assert_eq!(app.active_context, Context::Project);

    app.active_panel = Panel::Main; // Focused List
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)).await;
    assert_eq!(app.active_panel, Panel::Project); // Should return to Project, NOT Dashboard

    app.active_panel = Panel::Main;
    app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)).await;
    assert_eq!(app.active_panel, Panel::Project);
}

#[tokio::test]
async fn test_header_selection_stability() {
    let (_server, mut app) = setup_app().await;
    let (tx, _) = mpsc::unbounded_channel();
    app.tx = tx;

    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Todos;
    app.filtered_todos = vec![
        GroupedItem::Header("Today".into()),
        GroupedItem::Item(dummy_todo(1, "T1")),
    ];
    
    // Select Header
    app.todos_state.select(Some(0));
    app.selected_issue = Some(dummy_issue(99, "Stale")); 
    app.update_preview().await;
    
    assert!(app.selected_issue.is_none()); // Preview should be cleared for headers
}

#[tokio::test]
async fn test_keyboard_exhaustive() {
    let (_server, mut app) = setup_app().await;
    let (tx, _) = mpsc::unbounded_channel();
    app.tx = tx;

    // Test Tab cycling (Main -> Sidebar -> Detail -> Main)
    app.active_panel = Panel::Main;
    app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)).await;
    assert_eq!(app.active_panel, Panel::Dashboard);
    app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)).await;
    assert_eq!(app.active_panel, Panel::MainDetail);
    app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)).await;
    assert_eq!(app.active_panel, Panel::Main);

    // Test Enter zoom (Sidebar -> Main -> Detail)
    app.active_panel = Panel::Dashboard;
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;
    assert_eq!(app.active_panel, Panel::Main);
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;
    assert_eq!(app.active_panel, Panel::MainDetail);
}

#[tokio::test]
async fn test_help_actions_exhaustive() {
    let (_server, mut app) = setup_app().await;
    let (tx, _) = mpsc::unbounded_channel();
    app.tx = tx;

    app.execute_help_action(HelpAction::ToggleFocus).await;
    assert_eq!(app.active_panel, Panel::Main);
    app.execute_help_action(HelpAction::ToggleFocus).await;
    assert_eq!(app.active_panel, Panel::Dashboard);

    app.execute_help_action(HelpAction::Quit).await;
    assert!(app.should_quit);
}

#[tokio::test]
async fn test_empty_popup_navigation_safety() {
    let (_server, mut app) = setup_app().await;
    let (tx, _) = mpsc::unbounded_channel();
    app.tx = tx;

    // 1. Label Popup (Empty)
    app.project_labels = Vec::new();
    app.show_label_popup = true;
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)).await;
    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)).await;
    assert!(app.show_label_popup);

    // 2. Assignee Popup (Empty)
    app.project_members = Vec::new();
    app.show_assignee_popup = true;
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)).await;
    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)).await;
    assert!(app.show_assignee_popup);

    // 3. Milestone Popup (Empty)
    app.project_milestones = Vec::new();
    app.show_milestone_popup = true;
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)).await;
    assert!(app.show_milestone_popup);

    // 4. Project Popup (Empty)
    app.projects = Vec::new();
    app.show_project_popup = true;
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)).await;
    assert!(app.show_project_popup);
}
