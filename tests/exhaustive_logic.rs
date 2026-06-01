use lazygitlab::models::{App, MenuSelection, Project, Issue, GroupedItem, Panel, Context, Author, Todo, References};
use gitlab::GitlabBuilder;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use tokio::sync::mpsc;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{path};

fn dummy_project(id: u64, name: &str) -> Project {
    Project {
        id,
        name: name.into(),
        path_with_namespace: Some(format!("n/{}", name)),
        web_url: Some("u".into()),
        default_branch: None,
        namespace: None,
    }
}

fn dummy_issue(iid: u64, title: &str) -> Issue {
    Issue {
        iid, project_id: 1, title: title.into(), description: None, author: Author::new(1, "u".into()),
        assignees: vec![], state: "opened".into(), web_url: "u".into(), references: Some(References { full: "r".into() }),
        issue_type: None, labels: vec![], milestone: None, due_date: None, start_date: None, weight: None,
        task_completion_status: None, confidential: false, created_at: Some("t".into()),
        upvotes: Some(0), downvotes: Some(0), reactions: vec![], participants: vec![]
    }
}

fn dummy_todo(id: u64, body: &str) -> Todo {
    Todo {
        id, project: Some(dummy_project(1, "p")),
        action_name: "assigned".into(), target_type: "Issue".into(),
        target_url: "u".into(), body: body.into(), state: "pending".into(),
        created_at: "t".into(), group: None,
    }
}

async fn setup_app() -> (MockServer, App, mpsc::UnboundedReceiver<lazygitlab::models::DataUpdate>) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    let (tx, rx) = mpsc::unbounded_channel();
    let app = App::new(client, tx);
    (server, app, rx)
}

#[tokio::test]
async fn test_filtering_logic() {
    let (_server, mut app, _rx) = setup_app().await;
    app.projects = vec![dummy_project(1, "Alpha"), dummy_project(2, "Beta")];
    app.issues = vec![dummy_issue(1, "Bug fix"), dummy_issue(2, "New feature")];
    app.todos = vec![dummy_todo(1, "Review MR"), dummy_todo(2, "Comment on Issue")];

    app.filter_input = "alpha".into();
    app.filter_lists();
    assert_eq!(app.filtered_projects.len(), 1);
    assert_eq!(app.filtered_projects[0].name, "Alpha");

    app.active_context = Context::Project;
    app.filter_input = "bug".into();
    app.filter_lists();
    assert_eq!(app.filtered_issues.len(), 1);
    if let GroupedItem::Item(issue) = &app.filtered_issues[0] {
        assert_eq!(issue.title, "Bug fix");
    }

    app.filter_input = "review".into();
    app.filter_lists();
    assert_eq!(app.filtered_todos.len(), 2); // 1 Header + 1 Item
    if let GroupedItem::Item(todo) = &app.filtered_todos[1] {
        assert_eq!(todo.body, "Review MR");
    }
}

#[tokio::test]
async fn test_navigation_logic() {
    let (_server, mut app, _rx) = setup_app().await;
    app.active_panel = Panel::Dashboard;
    app.move_down();
    assert_eq!(app.dashboard_menu_state.selected(), Some(1));
    app.move_up();
    assert_eq!(app.dashboard_menu_state.selected(), Some(0));

    app.active_panel = Panel::Main;
    app.projects = vec![dummy_project(1, "P1"), dummy_project(2, "P2")];
    app.active_selection = MenuSelection::Projects;
    app.filter_lists();
    app.projects_state.select(Some(0));
    app.move_down();
    assert_eq!(app.projects_state.selected(), Some(1));
}

#[tokio::test]
async fn test_context_switching() {
    let (_server, mut app, _rx) = setup_app().await;
    app.selected_project = Some(dummy_project(1, "p"));
    app.active_panel = Panel::Dashboard;
    app.dashboard_menu_state.select(Some(0));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;
    assert_eq!(app.active_context, Context::Dashboard);
    assert_eq!(app.active_panel, Panel::Main);

    app.active_panel = Panel::Project;
    app.project_menu_state.select(Some(0));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;
    assert_eq!(app.active_context, Context::Project);
    assert_eq!(app.active_panel, Panel::Main);
}

#[tokio::test]
async fn test_project_popup_handling() {
    let (_server, mut app, _rx) = setup_app().await;
    app.projects = Vec::new();
    app.show_project_popup = true;
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)).await;
    assert!(app.show_project_popup);
}

#[tokio::test]
async fn test_involved_filtering() {
    let (_server, mut app, _rx) = setup_app().await;
    app.current_user = Some(Author::new(1, "me".into()));

    let mut issue_1 = dummy_issue(1, "Authored by me");
    issue_1.author = Author::new(1, "me".into());

    let mut issue_2 = dummy_issue(2, "Assigned to me");
    issue_2.author = Author::new(99, "other".into());
    issue_2.assignees = vec![Author::new(1, "me".into())];

    let mut issue_3 = dummy_issue(3, "Participated by me");
    issue_3.author = Author::new(99, "other".into());
    issue_3.participants = vec![Author::new(1, "me".into())];

    let mut issue_4 = dummy_issue(4, "Not involved");
    issue_4.author = Author::new(99, "other".into());

    app.issues = vec![issue_1, issue_2, issue_3, issue_4];

    app.show_involved_only = false;
    app.filter_lists();
    assert_eq!(app.filtered_issues.len(), 5); // 1 Header + 4 Items

    app.show_involved_only = true;
    app.filter_lists();
    assert_eq!(app.filtered_issues.len(), 4); // 1 Header + 3 Items

    use lazygitlab::models::MergeRequest;
    let mr_1 = MergeRequest {
        iid: 1, project_id: 1, title: "Authored by me".into(), description: None,
        author: Author::new(1, "me".into()), web_url: "u".into(), state: "opened".into(),
        draft: Some(false), references: None, upvotes: Some(0), downvotes: Some(0),
        labels: vec![], milestone: None, assignees: vec![], reviewers: vec![],
        squash: Some(false), created_at: Some("t".into()), diff_refs: None,
        reactions: vec![], merge_status: None, detailed_merge_status: None,
        has_conflicts: None, head_pipeline: None, source_branch: "".into(),
        target_branch: "".into(), participants: vec![]
    };
    let mr_2 = MergeRequest {
        iid: 2, project_id: 1, title: "Reviewed by me".into(), description: None,
        author: Author::new(99, "other".into()), web_url: "u".into(), state: "opened".into(),
        draft: Some(false), references: None, upvotes: Some(0), downvotes: Some(0),
        labels: vec![], milestone: None, assignees: vec![], reviewers: vec![Author::new(1, "me".into())],
        squash: Some(false), created_at: Some("t".into()), diff_refs: None,
        reactions: vec![], merge_status: None, detailed_merge_status: None,
        has_conflicts: None, head_pipeline: None, source_branch: "".into(),
        target_branch: "".into(), participants: vec![]
    };
    let mr_3 = MergeRequest {
        iid: 3, project_id: 1, title: "Not involved".into(), description: None,
        author: Author::new(99, "other".into()), web_url: "u".into(), state: "opened".into(),
        draft: Some(false), references: None, upvotes: Some(0), downvotes: Some(0),
        labels: vec![], milestone: None, assignees: vec![], reviewers: vec![],
        squash: Some(false), created_at: Some("t".into()), diff_refs: None,
        reactions: vec![], merge_status: None, detailed_merge_status: None,
        has_conflicts: None, head_pipeline: None, source_branch: "".into(),
        target_branch: "".into(), participants: vec![]
    };

    app.merge_requests = vec![mr_1, mr_2, mr_3];

    app.show_involved_only = false;
    app.filter_lists();
    assert_eq!(app.filtered_mrs.len(), 4); // 1 Header + 3 Items

    app.show_involved_only = true;
    app.filter_lists();
    assert_eq!(app.filtered_mrs.len(), 3); // 1 Header + 2 Items
}

#[tokio::test]
async fn test_dashboard_membership_filtering() {
    let (_server, mut app, _rx) = setup_app().await;
    app.active_context = Context::Dashboard;
    
    app.membership_sync_attempted = true;
    app.member_project_ids = vec![1].into_iter().collect();

    let issue_1 = dummy_issue(1, "In project 1");
    let mut issue_2 = dummy_issue(2, "In project 2");
    issue_2.project_id = 2;

    app.issues = vec![issue_1, issue_2];

    app.filter_lists();
    assert_eq!(app.filtered_issues.len(), 2);
    if let GroupedItem::Item(issue) = &app.filtered_issues[1] {
        assert_eq!(issue.title, "In project 1");
    }

    app.active_context = Context::Project;
    app.filter_lists();
    assert_eq!(app.filtered_issues.len(), 2);
}
