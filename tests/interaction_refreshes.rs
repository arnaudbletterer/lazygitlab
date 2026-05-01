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
async fn test_mr_title_update_refresh() {
    let (server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let mr_json = r##"{"iid": 1, "project_id": 1, "title": "Old Title", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;
    let updated_mr_json = r##"{"iid": 1, "project_id": 1, "title": "New Updated Title", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;

    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());
    app.view_mode = ViewMode::Detail;
    app.detail_section = DetailSection::Metadata;
    app.selected_metadata_field = MetadataField::Title;
    
    // Simulate Edit
    app.input_mode = true;
    app.input_target = InputTarget::EditTitle;
    app.reply_input = "New Updated Title".into();

    // Mock the PUT update AND the subsequent GET refreshes
    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"title":"New Updated Title"}))).mount(&server).await;
    // GitLab API Refresh mocks
    Mock::given(path("/api/v4/projects/1/merge_requests/1")).and(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<MergeRequest>(updated_mr_json).unwrap())).mount(&server).await;
    Mock::given(PathContains("award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
    Mock::given(PathContains("discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
    // List refresh mock
    Mock::given(path("/api/v4/projects/1/merge_requests")).and(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([serde_json::from_str::<MergeRequest>(updated_mr_json).unwrap()]))).mount(&server).await;

    // Act: Press Enter to save
    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    app.handle_key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }).await;

    // Render and Assert
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "Title: New Updated Title");
    
    // Verify internal state matches updated object
    assert_eq!(app.selected_mr.as_ref().unwrap().title, "New Updated Title");
}

#[tokio::test]
async fn test_issue_weight_update_refresh() {
    let (server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let issue_json = r##"{"iid": 1, "project_id": 1, "title": "I", "description": "d", "author": {"id": 1, "username": "u"}, "assignees": [], "state": "opened", "web_url": "u", "labels": [], "milestone": null, "due_date": null, "weight": 1, "confidential": false, "created_at": "2026-04-25T12:00:00Z", "upvotes": 0, "downvotes": 0, "references": {"full": "r"}}"##;
    let updated_issue_json = r##"{"iid": 1, "project_id": 1, "title": "I", "description": "d", "author": {"id": 1, "username": "u"}, "assignees": [], "state": "opened", "web_url": "u", "labels": [], "milestone": null, "due_date": null, "weight": 5, "confidential": false, "created_at": "2026-04-25T12:00:00Z", "upvotes": 0, "downvotes": 0, "references": {"full": "r"}}"##;

    app.selected_issue = Some(serde_json::from_str(issue_json).unwrap());
    app.view_mode = ViewMode::Detail;
    app.detail_section = DetailSection::Metadata;
    app.selected_metadata_field = MetadataField::Weight;
    app.show_weight_popup = true;
    app.weight_state.select(Some(5)); // Select weight 5 (Assuming the list logic works)

    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"weight":5}))).mount(&server).await;
    Mock::given(path("/api/v4/projects/1/issues")).and(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([serde_json::from_str::<Issue>(updated_issue_json).unwrap()]))).mount(&server).await;
    Mock::given(PathContains("award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
    Mock::given(PathContains("discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;

    // Press Enter in weight popup
    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    app.handle_key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }).await;

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "Weight: 5");
    assert_eq!(app.selected_issue.as_ref().unwrap().weight, Some(5));
}

#[tokio::test]
async fn test_mr_label_update_refresh() {
    let (server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;
    let updated_mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [{"id":1,"name":"feature","color":"#00ff00","text_color":"#000000"}], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;

    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());
    app.project_labels = vec![Label { id: 1, name: "feature".into(), color: "#00ff00".into(), text_color: "#000000".into() }];
    app.view_mode = ViewMode::Detail;
    app.show_label_popup = true;
    app.selected_label_ids.push(1);

    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"labels":"feature"}))).mount(&server).await;
    Mock::given(path("/api/v4/projects/1/merge_requests/1")).and(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<MergeRequest>(updated_mr_json).unwrap())).mount(&server).await;
    Mock::given(PathContains("award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
    Mock::given(PathContains("discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;

    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    app.handle_key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }).await;

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "feature");
    assert_eq!(app.selected_mr.as_ref().unwrap().labels.len(), 1);
}

#[tokio::test]
async fn test_discussion_resolve_refresh() {
    let (server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;
    let disc_json = r##"{"id": "1", "notes": [{"id":1,"body":"b","author":{"id":1,"username":"u"},"system":false,"created_at":"t"}], "resolved": false}"##;
    let updated_disc_json = r##"{"id": "1", "notes": [{"id":1,"body":"b","author":{"id":1,"username":"u"},"system":false,"created_at":"t"}], "resolved": true}"##;

    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());
    app.discussions = vec![serde_json::from_str(disc_json).unwrap()];
    app.view_mode = ViewMode::Detail;
    app.detail_section = DetailSection::Discussions;
    app.discussion_state.select(Some(0));

    Mock::given(method("PUT")).and(PathContains("discussions/1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"resolved":true}))).mount(&server).await;
    Mock::given(path("/api/v4/projects/1/merge_requests/1")).and(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<MergeRequest>(mr_json).unwrap())).mount(&server).await;
    Mock::given(PathContains("award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
    Mock::given(PathContains("discussions")).and(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([serde_json::from_str::<Discussion>(updated_disc_json).unwrap()]))).mount(&server).await;
    Mock::given(PathContains("diffs")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;

    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    app.handle_key(KeyEvent { code: KeyCode::Char('s'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }).await;

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "[RESOLVED]");
    assert_eq!(app.discussions[0].resolved, Some(true));
}

#[tokio::test]
async fn test_mr_milestone_update_refresh() {
    let (server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;
    let updated_mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": {"id":1,"title":"v2.0"}, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;

    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());
    app.project_milestones = vec![Milestone { id: 1, title: "v2.0".into() }];
    app.view_mode = ViewMode::Detail;
    app.show_milestone_popup = true;
    app.milestone_state.select(Some(1)); // First real milestone (0 is None)

    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"milestone_id":1}))).mount(&server).await;
    Mock::given(path("/api/v4/projects/1/merge_requests/1")).and(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<MergeRequest>(updated_mr_json).unwrap())).mount(&server).await;
    Mock::given(PathContains("award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
    Mock::given(PathContains("discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;

    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    app.handle_key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }).await;

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "Milestone: v2.0");
    assert_eq!(app.selected_mr.as_ref().unwrap().milestone.as_ref().unwrap().title, "v2.0");
}

#[tokio::test]
async fn test_ui_state_toggle_refresh() {
    let (_server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;
    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());
    let disc_json = r##"{"id": "1", "notes": [{"id":1,"body":"b","author":{"id":1,"username":"u"},"system":false,"created_at":"t"}], "resolved": false}"##;
    app.discussions = vec![serde_json::from_str(disc_json).unwrap()];
    app.discussions[0].collapsed = true;
    app.view_mode = ViewMode::Detail;
    app.detail_section = DetailSection::Discussions;
    app.discussion_state.select(None);

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "[+] @u"); // Collapsed indicator

    // Toggle expansion
    app.discussion_state.select(Some(0));
    app.toggle_discussion();
    assert_eq!(app.discussions[0].collapsed, false);

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "@u says:"); // Expanded indicator
}

#[tokio::test]
async fn test_pipeline_variable_refresh() {
    let (_server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    app.view_mode = ViewMode::List;
    app.active_selection = MenuSelection::Pipelines;
    app.show_pipeline_popup = true;
    app.input_target = InputTarget::PipelineVariableList;

    app.pipeline_variable_key_input = "DEBUG".into();
    app.pipeline_variable_value_input = "true".into();
    app.add_pipeline_variable();

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "DEBUG: true");

    app.pipeline_variables_state.select(Some(0));
    app.remove_pipeline_variable();
    
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    let buffer = terminal.backend().buffer();
    let mut content = String::new();
    for y in 0..buffer.area().height {
        for x in 0..buffer.area().width {
            content.push(buffer[(x, y)].symbol().chars().next().unwrap_or(' '));
        }
        content.push('\n');
    }
    assert!(!content.contains("DEBUG: true"), "Variable was not removed visually");
}

#[tokio::test]
async fn test_job_log_and_back_navigation_refresh() {
    let (_server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;
    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());

    let job_json = r##"{"id": 1, "name": "j", "stage": "s", "status": "success", "created_at": "t", "web_url": "u", "tag_list": [], "allow_failure": false}"##;
    app.selected_job = Some(serde_json::from_str(job_json).unwrap());
    app.job_log_lines = vec![
        LogLine { timestamp: "t".into(), content: "section start".into(), section: Some("s1".into()), is_section_start: true, is_section_end: false },
        LogLine { timestamp: "t".into(), content: "inner content".into(), section: Some("s1".into()), is_section_start: false, is_section_end: false },
    ];
    app.view_mode = ViewMode::JobLog;

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "inner content");

    // Toggle section
    app.job_log_scroll = 0;
    app.toggle_job_section();
    assert!(app.collapsed_sections.contains("s1"));

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    let mut content = String::new();
    let buffer = terminal.backend().buffer();
    for y in 0..buffer.area().height {
        for x in 0..buffer.area().width {
            content.push(buffer[(x, y)].symbol().chars().next().unwrap_or(' '));
        }
        content.push('\n');
    }
    // "inner content" should be gone because it's in a collapsed section
    assert!(!content.contains("inner content"));

    // Test Back refresh
    app.back().await;
    assert_eq!(app.view_mode, ViewMode::Detail);
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    // Should be back to Detail view (MR title)
    assert_buffer_contains(&terminal, "Title: M");
}
