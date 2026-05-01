use lazygitlab::models::*;
use lazygitlab::ui::ui;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use gitlab::GitlabBuilder;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

struct PathContains(&'static str);
impl wiremock::Match for PathContains {
    fn matches(&self, request: &wiremock::Request) -> bool {
        request.url.path().contains(self.0)
    }
}

async fn setup_app() -> (MockServer, App) {
    let server = MockServer::start().await;
    
    // Default success mocks
    Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;

    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    let mut app = App::new(client);
    app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: "n/p".into(), web_url: "u".into() });
    app.show_project_popup = false;
    (server, app)
}

fn assert_buffer_contains(terminal: &Terminal<TestBackend>, expected: &str) {
    let buffer = terminal.backend().buffer();
    let mut content = String::new();
    for y in 0..buffer.area().height {
        for x in 0..buffer.area().width {
            content.push(buffer[(x, y)].symbol().chars().next().unwrap_or(' '));
        }
        content.push('\n');
    }
    assert!(content.contains(expected), "Buffer does not contain: '{}'\nBuffer content:\n{}", expected, content);
}

#[tokio::test]
async fn test_mr_visual_completeness() {
    let (server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let mr_json = r##"{"iid": 123, "project_id": 1, "title": "Implement Feature X", "description": "This is a detailed description.", "author": {"id": 1, "username": "alice"}, "web_url": "u", "state": "opened", "references": {"full": "n/p!123"}, "upvotes": 5, "downvotes": 0, "labels": [{"id":1,"name":"bug","color":"#ff0000","text_color":"#ffffff"}], "milestone": {"id":1,"title":"v1.0"}, "assignees": [{"id":2,"username":"bob"}], "created_at": "2026-04-25T12:00:00Z"}"##;
    let disc_json = r##"{"id": "1", "notes": [{"id":1,"body":"Please review this part.","author":{"id":2,"username":"bob"},"system":false,"created_at":"2026-04-25T12:30:00Z"}]}"##;

    Mock::given(path("/api/v4/projects/1/merge_requests/123")).and(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<MergeRequest>(mr_json).unwrap())).mount(&server).await;
    Mock::given(PathContains("award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([{"emoji":"rocket"}]))).mount(&server).await;
    Mock::given(PathContains("discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([serde_json::from_str::<Discussion>(disc_json).unwrap()]))).mount(&server).await;

    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());
    app.fetch_discussions(1, 123).await.unwrap();
    app.view_mode = ViewMode::Detail;
    app.discussion_state.select(Some(0));
    
    // Simulate Space to expand
    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    app.handle_key(KeyEvent {
        code: KeyCode::Char(' '),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }).await;

    terminal.draw(|f| ui(f, &mut app)).unwrap();

    assert_buffer_contains(&terminal, "MR: n/p!123");
    assert_buffer_contains(&terminal, "Implement Feature X");
    assert_buffer_contains(&terminal, "Author: @alice");
    assert_buffer_contains(&terminal, "Assignees: @bob");
    assert_buffer_contains(&terminal, "Milestone: v1.0");
    assert_buffer_contains(&terminal, "bug");
    assert_buffer_contains(&terminal, "Reactions: 👍  5 🚀  1");
    assert_buffer_contains(&terminal, "@bob says:");
    assert_buffer_contains(&terminal, "Please review this part.");
}

#[tokio::test]
async fn test_navigation_and_focus() {
    let (_server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    app.projects = vec![
        Project { id: 1, name: "Proj 1".into(), path_with_namespace: "n/p1".into(), web_url: "u".into() },
        Project { id: 2, name: "Proj 2".into(), path_with_namespace: "n/p2".into(), web_url: "u".into() },
    ];
    app.show_project_popup = true;
    app.projects_state.select(Some(0));

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "> Proj 1 (n/p1)");
    assert_buffer_contains(&terminal, "  Proj 2 (n/p2)");

    // Move down
    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    let down = KeyEvent { code: KeyCode::Char('j'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };
    app.handle_key(down).await;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "  Proj 1 (n/p1)");
    assert_buffer_contains(&terminal, "> Proj 2 (n/p2)");

    // Select and go to list
    let enter = KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };
    app.handle_key(enter).await;

    app.merge_requests = vec![
        serde_json::from_str(r##"{"iid": 1, "project_id": 1, "title": "MR 1", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r1"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z"}"##).unwrap(),
    ];
    app.filter_lists();
    app.mrs_state.select(Some(0));
    app.view_mode = ViewMode::List;
    app.active_selection = MenuSelection::MergeRequests;
    app.focus_on_menu = false;

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "> [opened] r1 - MR 1 by @u");

    // Toggle focus to menu
    let tab = KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };
    app.handle_key(tab).await;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    // In menu, we expect highlighting. Menu items: Merge Requests, Issues, Pipelines
    assert_buffer_contains(&terminal, " Merge Requests "); 
}

#[tokio::test]
async fn test_pipeline_visual_completeness() {
    let (_server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let pipeline_json = r##"{"id": 789, "project_id": 1, "status": "running", "source": "trigger", "ref": "master", "web_url": "u", "created_at": "2026-04-25T12:00:00Z", "started_at": "2026-04-25T12:00:00Z"}"##;
    app.selected_pipeline = Some(serde_json::from_str(pipeline_json).unwrap());
    
    let job_json = r##"{"id": 1, "name": "build-job", "stage": "build", "status": "running", "created_at": "t", "started_at": "2026-04-25T12:00:00Z", "web_url": "u", "tag_list": ["docker"], "allow_failure": false}"##;
    app.jobs = vec![serde_json::from_str(job_json).unwrap()];
    app.view_mode = ViewMode::Detail;

    terminal.draw(|f| ui(f, &mut app)).unwrap();

    assert_buffer_contains(&terminal, "Pipeline: #789");
    assert_buffer_contains(&terminal, "Status: running");
    assert_buffer_contains(&terminal, "Source: trigger");
    assert_buffer_contains(&terminal, "Ref: master");
    assert_buffer_contains(&terminal, "Elapsed:");
    assert_buffer_contains(&terminal, "[running] build-job - Stage: build");
}

#[tokio::test]
async fn test_issue_visual_completeness() {
    let (_server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let issue_json = r##"{"iid": 456, "project_id": 1, "title": "Fix Critical Bug", "description": "Critical bug in engine.", "author": {"id": 1, "username": "alice"}, "assignees": [{"id":2,"username":"bob"}], "state": "opened", "web_url": "u", "labels": [], "milestone": null, "due_date": "2026-05-20", "weight": 9, "confidential": true, "created_at": "2026-04-25T12:00:00Z", "upvotes": 0, "downvotes": 0, "task_completion_status": {"count": 5, "completed_count": 2}}"##;
    app.selected_issue = Some(serde_json::from_str(issue_json).unwrap());
    app.view_mode = ViewMode::Detail;

    terminal.draw(|f| ui(f, &mut app)).unwrap();

    assert_buffer_contains(&terminal, "Issue: #456");
    assert_buffer_contains(&terminal, "Fix Critical Bug");
    assert_buffer_contains(&terminal, "Weight: 9");
    assert_buffer_contains(&terminal, "Due Date: 2026-05-20");
    assert_buffer_contains(&terminal, "Confidential: Yes");
    assert_buffer_contains(&terminal, "Tasks: 2 of 5 completed");
}

#[tokio::test]
async fn test_interaction_completeness() {
    let (server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let mr_json = r##"{"iid": 1, "project_id": 1, "title": "Old Title", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z"}"##;
    let disc_resolved_json = r##"{"id": "1", "notes": [{"id":1,"body":"resolved note","author":{"id":1,"username":"u"},"system":false,"created_at":"t"}], "resolved": true}"##;

    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());
    app.discussions = vec![serde_json::from_str(disc_resolved_json).unwrap()];
    app.view_mode = ViewMode::Detail;
    app.detail_section = DetailSection::Discussions;

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "[RESOLVED]");

    // Test Title Edit visual
    app.detail_section = DetailSection::Metadata;
    app.selected_metadata_field = MetadataField::Title;
    app.input_mode = true;
    app.input_target = InputTarget::EditTitle;
    app.reply_input = "New Amazing Title".into();
    app.cursor_index = app.reply_input.len();

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "Title: New Amazing Title");

    // Mock successful save
    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"title":"New Amazing Title"}))).mount(&server).await;
    
    // Simulate Enter to save
    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    let enter = KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };
    app.handle_key(enter).await;
    
    // Title in model should still be "Old Title" until refresh or manual update in mock environment
    // But our UI should have cleared input mode
    assert_eq!(app.input_mode, false);
}
