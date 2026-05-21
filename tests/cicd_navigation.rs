use lazygitlab::models::*;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::path;
use gitlab::GitlabBuilder;

async fn setup_app() -> (MockServer, App) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    (server, App::new(client, tx))
}

#[tokio::test]
async fn test_pipeline_to_jobs_navigation() {
    let (_server, mut app) = setup_app().await;
    app.active_context = Context::Project;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Pipelines;
    app.pipelines = vec![Pipeline { id: 123, project_id: 1, status: "success".into(), source: "trigger".into(), ref_name: "master".into(), web_url: "u".into(), created_at: "t".into(), started_at: None, duration: None }];
    app.filtered_pipelines = app.pipelines.clone();
    app.pipelines_state.select(Some(0));

    let enter = KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };
    
    // Pressing Enter on a Pipeline should move focus to MainDetail and select the pipeline
    app.handle_key(enter).await;
    
    assert_eq!(app.active_panel, Panel::MainDetail);
    assert!(app.selected_pipeline.is_some());
    assert_eq!(app.selected_pipeline.as_ref().unwrap().id, 123);
}

#[tokio::test]
async fn test_job_to_log_navigation() {
    let (_server, mut app) = setup_app().await;
    app.active_context = Context::Project;
    app.active_panel = Panel::MainDetail;
    app.active_selection = MenuSelection::Pipelines;
    app.selected_project = Some(Project { id: 1, name: "p".into(), web_url: Some("u".into()), path_with_namespace: Some("p".into()), default_branch: Some("master".into()), namespace: None });
    app.selected_pipeline = Some(Pipeline { id: 123, project_id: 1, status: "success".into(), source: "trigger".into(), ref_name: "master".into(), web_url: "u".into(), created_at: "t".into(), started_at: None, duration: None });
    app.jobs = vec![Job { id: 456, name: "build".into(), stage: "build".into(), status: "success".into(), created_at: "t".into(), started_at: None, finished_at: None, duration: None, user: None, runner: None, allow_failure: false, web_url: "u".into(), tag_list: vec![] }];
    app.jobs_state.select(Some(0));

    let enter = KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE };
    
    // Pressing Enter on a Job in MainDetail should open the JobLog view
    app.handle_key(enter).await;
    
    assert_eq!(app.view_mode, ViewMode::JobLog);
    assert!(app.selected_job.is_some());
    assert_eq!(app.selected_job.as_ref().unwrap().id, 456);
}

#[tokio::test]
async fn test_no_overlapping_popups_on_pipeline_launch() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use lazygitlab::ui::ui;

    let (_server, mut app) = setup_app().await;
    app.active_context = Context::Project;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Pipelines;
    
    // Simulate 'n' key press logic
    app.show_pipeline_popup = true;
    app.input_mode = true;
    app.input_target = InputTarget::PipelineRef;

    let backend = TestBackend::new(100, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    
    let buffer = terminal.backend().buffer();
    let mut output = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            output.push(buffer[(x, y)].symbol().chars().next().unwrap_or(' '));
        }
        output.push('\n');
    }

    // Should contain the pipeline launch title
    assert!(output.contains("Launch Pipeline"));
    // Should NOT contain the generic input title
    assert!(!output.contains("Input (Ctrl+Enter to save, Esc to cancel)"));
}
