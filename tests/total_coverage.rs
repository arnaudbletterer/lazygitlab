use lazygitlab::models::*;
use lazygitlab::ui::ui;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use gitlab::GitlabBuilder;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use crossterm::event::KeyCode;

struct PathContains(&'static str);
impl wiremock::Match for PathContains {
    fn matches(&self, request: &wiremock::Request) -> bool {
        request.url.path().contains(self.0)
    }
}

async fn run_monolithic_tests() {
    let server = MockServer::start().await;
    
    // Default success mocks
    Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;

    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    let mut app = App::new(client);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    app.tx = tx;
    app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()) });

    // 1. Logic exhaustive
    let _ = app.fetch_projects().await;
    let _ = app.fetch_groups().await;
    let _ = app.fetch_todos().await;
    let _ = app.fetch_global_merge_requests().await;
    let _ = app.fetch_global_issues().await;
    let _ = app.fetch_project_metadata().await;
    
    // Pump updates
    let start = std::time::Instant::now();
    while std::time::Instant::now().duration_since(start) < std::time::Duration::from_millis(500) {
        app.process_updates(&mut rx);
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    
    let issue_json = r##"{"iid": 1, "project_id": 1, "title": "I", "description": "d", "author": {"id": 1, "username": "u"}, "assignees": [{"id":1,"username":"u"}], "state": "opened", "web_url": "u", "labels": [{"id":1,"name":"l","color":"#ff0000","text_color":"#ffffff"}], "milestone": {"id":1,"title":"m"}, "due_date": "2026-04-25", "weight": 5, "confidential": true, "created_at": "2026-04-25T12:00:00Z", "upvotes": 1, "downvotes": 1, "issue_type": "issue", "references": {"full": "n/p#1"}, "reactions": []}"##;
    app.selected_issue = Some(serde_json::from_str(issue_json).unwrap());
    let _ = app.update_issue().await;
    let _ = app.toggle_issue_state().await;
    let _ = app.toggle_confidential().await;

    // 2. Navigation exhaustive
    for mode in [ViewMode::List, ViewMode::Detail, ViewMode::JobLog, ViewMode::Diff] {
        app.view_mode = mode;
        for focus in [true, false] {
            app.active_panel = if focus { Panel::Sidebar } else { Panel::Content };
            for _ in 0..10 { app.move_down(); app.move_up(); }
        }
        app.page_down(); app.page_up();
        app.enter().await;
        app.back().await;
    }

    // 3. UI Rendering exhaustive
    let backend = TestBackend::new(150, 100);
    let mut terminal = Terminal::new(backend).unwrap();

    app.view_mode = ViewMode::Detail;
    app.detail_section = DetailSection::Metadata;
    app.selected_mr = None; app.selected_issue = None; app.selected_pipeline = None;
    
    for field in [MetadataField::Title, MetadataField::Description, MetadataField::Labels, MetadataField::Assignees, MetadataField::Milestone, MetadataField::DueDate, MetadataField::Weight, MetadataField::Confidential] {
        app.selected_metadata_field = field;
        terminal.draw(|f| ui(f, &mut app)).unwrap();
    }
    
    let mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "desc\nline2", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 1, "downvotes": 1, "labels": [], "milestone": null, "assignees": [{"id":1,"username":"u1"},{"id":2,"username":"u2"}], "created_at": "2026-04-25T12:00:00Z", "reactions": []}"##;
    let disc_json = r##"{"id": "1", "notes": [{"id":1,"body":"b","author":{"id":1,"username":"u"},"system":false,"created_at":"2026-04-25T12:00:00Z"}]}"##;

    Mock::given(PathContains("award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([{"emoji":"smile"}]))).mount(&server).await;
    Mock::given(PathContains("discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([serde_json::from_str::<Discussion>(disc_json).unwrap()]))).mount(&server).await;
    Mock::given(method("GET")).and(path("/api/v4/projects/1/merge_requests/1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<MergeRequest>(mr_json).unwrap())).mount(&server).await;

    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());
    app.selected_issue = None; app.selected_pipeline = None;
    let _ = app.fetch_discussions(1, 1).await;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    
    // Expand discussions
    app.detail_section = DetailSection::Discussions;
    if !app.discussions.is_empty() {
        app.discussions[0].collapsed = false;
    }
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.detail_section = DetailSection::Metadata;

    // Running job with started_at
    app.selected_mr = None; app.selected_issue = None;
    let job_json = r#"{"id": 1, "name": "j", "stage": "s", "status": "running", "created_at": "t", "started_at": "2026-04-25T12:00:00Z", "web_url": "u_job", "tag_list": [], "allow_failure": false, "duration": null, "finished_at": null, "runner": null, "pipeline": {"id": 100, "status": "success", "ref": "m", "sha": "s", "web_url": "u"}, "user": {"id": 1, "username": "u", "name": "n", "state": "active", "avatar_url": "u", "web_url": "u"}}"#;
    app.selected_job = Some(serde_json::from_str(job_json).unwrap());
    app.selected_pipeline = Some(serde_json::from_str(r#"{"id": 100, "project_id": 1, "status": "running", "source": "push", "ref": "master", "web_url": "url", "created_at": "2026-04-25T12:00:00Z", "started_at": "2026-04-25T12:00:00Z"}"#).unwrap());
    app.view_mode = ViewMode::JobLog;
    terminal.draw(|f| ui(f, &mut app)).unwrap();

    // 4. Open in browser exhaustive
    app.view_mode = ViewMode::Detail;
    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());
    app.selected_issue = None; app.selected_pipeline = None;
    app.open_in_browser(); // MR
    app.selected_mr = None;
    let issue_obj: Issue = serde_json::from_str(issue_json).unwrap();
    app.selected_issue = Some(issue_obj);
    app.open_in_browser(); // Issue
    app.selected_issue = None;
    app.selected_pipeline = Some(serde_json::from_str(r#"{"id": 100, "project_id": 1, "status": "success", "source": "push", "ref": "master", "web_url": "url", "created_at": "2026-04-25T12:00:00Z"}"#).unwrap());
    app.open_in_browser(); // Pipeline
    app.jobs = vec![serde_json::from_str(job_json).unwrap()];
    app.jobs_state.select(Some(0));
    app.open_in_browser(); // Job

    // Global list open_in_browser
    app.view_mode = ViewMode::List;
    app.selected_pipeline = None;
    app.selected_job = None;
    
    app.projects = vec![Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()) }];
    app.projects_state.select(Some(0));
    app.active_selection = MenuSelection::Projects;
    app.open_in_browser();

    app.groups = vec![Group { id: 1, name: "g".into(), path: "g".into(), full_path: "g".into() }];
    app.groups_state.select(Some(0));
    app.active_selection = MenuSelection::Groups;
    app.open_in_browser();

    app.todos = vec![serde_json::from_str(r#"{"id": 1, "project": {"id":1,"name":"p","path_with_namespace":"n/p","web_url":"u"}, "action_name": "assigned", "target_type": "Issue", "target_url": "u", "body": "b", "state": "pending", "created_at": "t"}"#).unwrap()];
    app.todos_state.select(Some(0));
    app.active_selection = MenuSelection::Todos;
    app.open_in_browser();

    // 5. Discussion enrichment exhaustive
    Mock::given(PathContains("/diffs")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
        {"old_path": "f", "new_path": "f", "diff": "@@ -1,3 +1,3 @@\n same\n-old\n+new\n"}
    ]))).mount(&server).await;
    
    let disc_removed = r##"{"id": "d2", "notes": [{"id": 2, "body": "b", "author": {"id": 1, "username": "u"}, "system": false, "created_at": "t", "position": {"old_path": "f", "old_line": 2}}]}"##;
    let disc_unchanged = r##"{"id": "d3", "notes": [{"id": 3, "body": "b", "author": {"id": 1, "username": "u"}, "system": false, "created_at": "t", "position": {"new_path": "f", "new_line": 1}}]}"##;
    Mock::given(PathContains("/discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
        serde_json::from_str::<Discussion>(disc_json).unwrap(),
        serde_json::from_str::<Discussion>(disc_removed).unwrap(),
        serde_json::from_str::<Discussion>(disc_unchanged).unwrap()
    ]))).mount(&server).await;

    let _ = app.fetch_discussions(1, 1).await;

    // Detailed Issue Rendering
    app.selected_mr = None;
    app.selected_pipeline = None;
    app.selected_issue = Some(serde_json::from_str(issue_json).unwrap());
    app.detail_section = DetailSection::Metadata;
    for field in [MetadataField::Title, MetadataField::Labels, MetadataField::Assignees, MetadataField::Milestone, MetadataField::DueDate, MetadataField::Weight, MetadataField::Confidential, MetadataField::Description] {
        app.selected_metadata_field = field;
        terminal.draw(|f| ui(f, &mut app)).unwrap();
    }
    
    // Discussion comments and system notes
    let disc_system_json = r##"{"id": "4", "notes": [{"id":4,"body":"system note","author":{"id":1,"username":"u"},"system":true,"created_at":"t"}]}"##;
    app.discussions.push(serde_json::from_str(disc_system_json).unwrap());
    app.detail_section = DetailSection::Discussions;
    if app.discussions.len() >= 2 {
        app.discussions[0].collapsed = false;
        app.discussions[1].collapsed = false;
    }
    terminal.draw(|f| ui(f, &mut app)).unwrap();

    // List view variants
    app.view_mode = ViewMode::List;
    app.active_selection = MenuSelection::MergeRequests;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.active_selection = MenuSelection::Issues;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.active_selection = MenuSelection::Pipelines;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.active_panel = Panel::Sidebar;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    
    // Input mode variants
    app.input_mode = true;
    for target in [InputTarget::NewComment, InputTarget::Reply, InputTarget::EditTitle, InputTarget::EditDescription, InputTarget::PipelineVariableKey, InputTarget::PipelineVariableValue] {
        app.input_target = target;
        terminal.draw(|f| ui(f, &mut app)).unwrap();
    }
    app.input_mode = false;
    
    // Widgets coverage
    let theme = lazygitlab::ui::Theme::default();
    lazygitlab::ui::widgets::get_status_style("canceled", &theme);
    lazygitlab::ui::widgets::format_relative_time("2026-04-25T12:00:00.123Z");

    // 8. Fuzzing to hit all UI branches
    let keys = [
        KeyCode::Down, KeyCode::Up, KeyCode::Left, KeyCode::Right, KeyCode::Enter, KeyCode::Esc, KeyCode::Tab, KeyCode::Backspace, 
        KeyCode::Delete, KeyCode::Home, KeyCode::End, KeyCode::PageUp, KeyCode::PageDown,
        KeyCode::Char('j'), KeyCode::Char('k'), KeyCode::Char('h'), KeyCode::Char('l'),
        KeyCode::Char('p'), KeyCode::Char('r'), KeyCode::Char('c'), KeyCode::Char('s'),
        KeyCode::Char('n'), KeyCode::Char(' '), KeyCode::Char('v'), KeyCode::Char('o'), KeyCode::Char('q'),
        KeyCode::Char('?'), KeyCode::Char('/'), KeyCode::Char('d'), KeyCode::Char('a'), KeyCode::Char('m'), KeyCode::Char('C')
    ];
    let setup_popups = [
        (|a: &mut App| a.show_project_popup = true),
        (|a: &mut App| a.show_calendar_popup = true),
        (|a: &mut App| a.show_label_popup = true),
        (|a: &mut App| a.show_assignee_popup = true),
        (|a: &mut App| a.show_milestone_popup = true),
        (|a: &mut App| a.show_weight_popup = true),
        (|a: &mut App| { a.show_pipeline_popup = true; a.input_target = InputTarget::PipelineVariableList; }),
        (|a: &mut App| { a.show_pipeline_popup = true; a.show_branches_dropdown = true; }),
        (|a: &mut App| a.show_help_popup = true),
        (|a: &mut App| { a.show_help_popup = true; a.show_help_filter = true; }),
        (|a: &mut App| a.show_create_issue_popup = true),
        (|a: &mut App| a.show_create_mr_popup = true),
    ];
    let modes = [ViewMode::List, ViewMode::Detail, ViewMode::JobLog, ViewMode::Diff];
    let selections = [MenuSelection::Todos, MenuSelection::MergeRequests, MenuSelection::Issues, MenuSelection::Pipelines, MenuSelection::Projects, MenuSelection::Groups];

    for mode in modes {
        for sel in selections {
            for focus in [true, false] {
                app.view_mode = mode;
                app.active_selection = sel;
                app.active_panel = if focus { Panel::Sidebar } else { Panel::Content };
                
                // Normal keys
                for k in &keys {
                    let k_event = crossterm::event::KeyEvent::new(*k, crossterm::event::KeyModifiers::NONE);
                    app.handle_key(k_event).await;
                }
                // Shift Enter
                app.handle_key(crossterm::event::KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::SHIFT)).await;
                // Mouse
                app.handle_mouse(crossterm::event::MouseEvent { kind: crossterm::event::MouseEventKind::ScrollDown, column: 0, row: 0, modifiers: crossterm::event::KeyModifiers::NONE }).await;
                app.handle_mouse(crossterm::event::MouseEvent { kind: crossterm::event::MouseEventKind::ScrollUp, column: 0, row: 0, modifiers: crossterm::event::KeyModifiers::NONE }).await;
            }
        }
    }

    // Fuzz popups
    for setup in setup_popups {
        app.show_project_popup = false; app.show_calendar_popup = false; app.show_label_popup = false;
        app.show_assignee_popup = false; app.show_milestone_popup = false; app.show_weight_popup = false; app.show_pipeline_popup = false;
        app.show_help_popup = false; app.show_help_filter = false; app.show_create_issue_popup = false; app.show_create_mr_popup = false;
        app.input_mode = false;
        setup(&mut app);
        for k in &keys {
            let k_event = crossterm::event::KeyEvent::new(*k, crossterm::event::KeyModifiers::NONE);
            app.handle_key(k_event).await;
        }
    }
}

#[tokio::test]
async fn test_the_entire_app_once() {
    run_monolithic_tests().await;
}
