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

async fn setup_app() -> (MockServer, App, tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client);
    app.tx = tx;
    app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()) });
    app.show_project_popup = false;
    (server, app, rx)
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

async fn wait_for_loading_finish(app: &mut App, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
    let start = std::time::Instant::now();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    while std::time::Instant::now().duration_since(start) < std::time::Duration::from_secs(10) {
        app.process_updates(rx).await;
        if !app.is_loading && app.loading_tasks == 0 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            app.process_updates(rx).await;
            if !app.is_loading { return; }
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn test_mr_title_update_refresh() {
    let (server, mut app, mut rx) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let mr_json = r##"{"iid": 1, "project_id": 1, "title": "Old Title", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;
    let updated_mr_json = r##"{"iid": 1, "project_id": 1, "title": "New Updated Title", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;

    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());
    app.view_mode = ViewMode::Detail;
    app.detail_section = DetailSection::Metadata;
    app.selected_metadata_field = MetadataField::Title;
    
    app.input_mode = true;
    app.input_target = InputTarget::EditTitle;
    app.reply_input = "New Updated Title".into();

    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"title":"New Updated Title"}))).mount(&server).await;
    Mock::given(path("/api/v4/projects/1/merge_requests/1")).and(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<MergeRequest>(updated_mr_json).unwrap())).mount(&server).await;
    Mock::given(PathContains("award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
    Mock::given(PathContains("discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;

    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    app.handle_key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }).await;

    wait_for_loading_finish(&mut app, &mut rx).await;

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "Title: New Updated Title");
    assert_eq!(app.selected_mr.as_ref().unwrap().title, "New Updated Title");
}

#[tokio::test]
async fn test_issue_weight_update_refresh() {
    let (server, mut app, mut rx) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let issue_json = r##"{"iid": 1, "project_id": 1, "title": "I", "description": "d", "author": {"id": 1, "username": "u"}, "assignees": [], "state": "opened", "web_url": "u", "labels": [], "milestone": null, "due_date": null, "weight": 1, "confidential": false, "created_at": "2026-04-25T12:00:00Z", "upvotes": 0, "downvotes": 0, "references": {"full": "r"}}"##;
    let updated_issue_json = r##"{"iid": 1, "project_id": 1, "title": "I", "description": "d", "author": {"id": 1, "username": "u"}, "assignees": [], "state": "opened", "web_url": "u", "labels": [], "milestone": null, "due_date": null, "weight": 5, "confidential": false, "created_at": "2026-04-25T12:00:00Z", "upvotes": 0, "downvotes": 0, "references": {"full": "r"}}"##;

    app.selected_issue = Some(serde_json::from_str(issue_json).unwrap());
    app.view_mode = ViewMode::Detail;
    app.detail_section = DetailSection::Metadata;
    app.selected_metadata_field = MetadataField::Weight;
    app.show_weight_popup = true;
    app.weight_state.select(Some(5)); 

    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"weight":5}))).mount(&server).await;
    Mock::given(path("/api/v4/projects/1/issues")).and(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([serde_json::from_str::<Issue>(updated_issue_json).unwrap()]))).mount(&server).await;
    Mock::given(path("/api/v4/projects/1/issues/1")).and(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<Issue>(updated_issue_json).unwrap())).mount(&server).await;
    Mock::given(PathContains("award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
    Mock::given(PathContains("discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;

    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    app.handle_key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }).await;

    wait_for_loading_finish(&mut app, &mut rx).await;

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "Weight: 5");
    assert_eq!(app.selected_issue.as_ref().unwrap().weight, Some(5));
}

#[tokio::test]
async fn test_mr_label_update_refresh() {
    let (server, mut app, mut rx) = setup_app().await;
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

    wait_for_loading_finish(&mut app, &mut rx).await;

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "feature");
    assert_eq!(app.selected_mr.as_ref().unwrap().labels.len(), 1);
}

#[tokio::test]
async fn test_discussion_resolve_refresh() {
    let (server, mut app, mut rx) = setup_app().await;
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

    wait_for_loading_finish(&mut app, &mut rx).await;

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "[RESOLVED]");
    assert_eq!(app.discussions[0].resolved, Some(true));
}

#[tokio::test]
async fn test_mr_milestone_update_refresh() {
    let (server, mut app, mut rx) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    let mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;
    let updated_mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": {"id":1,"title":"v2.0"}, "assignees": [], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;

    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());
    app.project_milestones = vec![Milestone { id: 1, title: "v2.0".into() }];
    app.view_mode = ViewMode::Detail;
    app.show_milestone_popup = true;
    app.milestone_state.select(Some(1)); 

    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"milestone_id":1}))).mount(&server).await;
    Mock::given(path("/api/v4/projects/1/merge_requests/1")).and(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<MergeRequest>(updated_mr_json).unwrap())).mount(&server).await;
    Mock::given(PathContains("award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
    Mock::given(PathContains("discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;

    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    app.handle_key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }).await;

    wait_for_loading_finish(&mut app, &mut rx).await;

    terminal.draw(|f| ui(f, &mut app)).unwrap();
    assert_buffer_contains(&terminal, "Milestone: v2.0");
    assert_eq!(app.selected_mr.as_ref().unwrap().milestone.as_ref().unwrap().title, "v2.0");
}
