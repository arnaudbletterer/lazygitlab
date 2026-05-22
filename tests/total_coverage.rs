use lazygitlab::models::*;
use lazygitlab::ui::ui;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use gitlab::GitlabBuilder;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use crossterm::event::KeyCode;

// Removed PathContains struct and its impl as it was unused

async fn wait_for_async(app: &mut App, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
    let start = std::time::Instant::now();
    while std::time::Instant::now().duration_since(start) < std::time::Duration::from_millis(200) {
        app.process_updates(rx).await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

async fn run_monolithic_tests() {
    let server = MockServer::start().await;
    
    // Default success mocks
    Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;

    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client, tx);
    app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()), default_branch: None, namespace: None });

    // 1. Initial data load
    let _ = app.initial_setup().await;
    wait_for_async(&mut app, &mut rx).await;

    // 2. Navigation exhaustive
    for mode in [ViewMode::List, ViewMode::Detail, ViewMode::JobLog, ViewMode::Diff] {
        app.view_mode = mode;
        for focus in [true, false] {
            app.active_panel = if focus { Panel::Dashboard } else { Panel::Main };
            for _ in 0..5 { app.move_down(); app.move_up(); }
        }
        app.enter().await;
        app.back().await;
    }

    // 3. UI Rendering exhaustive
    let backend = TestBackend::new(150, 100);
    let mut terminal = Terminal::new(backend).unwrap();

    app.view_mode = ViewMode::Detail;
    app.detail_section = DetailSection::Metadata;
    
    let mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "desc\nline2", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 1, "downvotes": 1, "labels": [], "milestone": null, "assignees": [{"id":1,"username":"u1"}], "created_at": "2026-04-25T12:00:00Z", "reactions": [], "draft": false}"##;
    app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());

    for field in [MetadataField::Title, MetadataField::Description, MetadataField::Labels, MetadataField::Assignees, MetadataField::Reviewers, MetadataField::Milestone, MetadataField::DueDate, MetadataField::Weight, MetadataField::Confidential] {
        app.selected_metadata_field = field;
        terminal.draw(|f| ui(f, &mut app)).unwrap();
    }
    
    // 4. Input target exhaustive
    app.input_mode = true;
    for target in [
        InputTarget::Reply, InputTarget::EditTitle, InputTarget::EditDescription,
        InputTarget::Filter, InputTarget::PipelineRef, InputTarget::PipelineVariableKey, InputTarget::PipelineVariableValue,
        InputTarget::CreateIssueTitle, InputTarget::CreateMRTitle,
    ] {
        app.input_target = target;
        terminal.draw(|f| ui(f, &mut app)).unwrap();
    }
    app.input_mode = false;

    // 5. Global fetches
    let _ = app.fetch_todos().await;
    let _ = app.fetch_global_merge_requests().await;
    let _ = app.fetch_global_issues().await;
    wait_for_async(&mut app, &mut rx).await;

    // 6. Action methods
    let _ = app.post_reply().await;
    let _ = app.toggle_mr_state().await;
    wait_for_async(&mut app, &mut rx).await;

    // 7. Fuzzing keys in all modes
    let keys = [
        KeyCode::Down, KeyCode::Up, KeyCode::Left, KeyCode::Right, KeyCode::Enter, KeyCode::Esc, KeyCode::Tab, KeyCode::Backspace, 
        KeyCode::Delete, KeyCode::Home, KeyCode::End, KeyCode::PageUp, KeyCode::PageDown,
        KeyCode::Char('j'), KeyCode::Char('k'), KeyCode::Char('h'), KeyCode::Char('l'),
        KeyCode::Char('p'), KeyCode::Char('r'), KeyCode::Char('c'), KeyCode::Char('s'),
        KeyCode::Char('n'), KeyCode::Char(' '), KeyCode::Char('v'), KeyCode::Char('o'), KeyCode::Char('q'),
        KeyCode::Char('?'), KeyCode::Char('/'), KeyCode::Char('d'), KeyCode::Char('a'), KeyCode::Char('m')
    ];

    for mode in [ViewMode::List, ViewMode::Detail] {
        app.view_mode = mode;
        for sel in [MenuSelection::Todos, MenuSelection::MergeRequests, MenuSelection::Issues] {
            app.active_selection = sel;
            for k in &keys {
                let k_event = crossterm::event::KeyEvent::new(*k, crossterm::event::KeyModifiers::NONE);
                app.handle_key(k_event).await;
            }
        }
    }
}

#[tokio::test]
async fn test_the_entire_app_once() {
    run_monolithic_tests().await;
}
