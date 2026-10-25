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

fn ctrl_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::CONTROL,
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
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"id":1,"username":"devops"})),
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

fn sample_project() -> Project {
    Project {
        id: 42,
        name: "ci-project".into(),
        web_url: Some("https://gitlab.example.com/group/ci-project".into()),
        path_with_namespace: Some("group/ci-project".into()),
        default_branch: Some("main".into()),
        namespace: None,
    }
}

fn sample_pipeline(id: u64, status: &str) -> Pipeline {
    Pipeline {
        id,
        project_id: 42,
        status: status.into(),
        source: "push".into(),
        ref_name: "main".into(),
        web_url: format!("https://gitlab.example.com/pipelines/{}", id),
        created_at: "2026-09-01T12:00:00Z".into(),
        started_at: Some("2026-09-01T12:01:00Z".into()),
        duration: Some(120.0),
    }
}

fn sample_job(id: u64, name: &str, stage: &str, status: &str) -> Job {
    Job {
        id,
        name: name.into(),
        stage: stage.into(),
        status: status.into(),
        created_at: "2026-09-01T12:01:00Z".into(),
        started_at: Some("2026-09-01T12:01:10Z".into()),
        finished_at: None,
        duration: Some(60.0),
        user: None,
        runner: None,
        tag_list: Vec::new(),
        web_url: format!("https://gitlab.example.com/jobs/{}", id),
        allow_failure: false,
    }
}

#[tokio::test]
async fn test_pipeline_overview_status_pills_and_stage_drill_in() {
    let (server, mut app, _rx) = setup_app().await;
    app.selected_project = Some(sample_project());
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Pipelines;
    app.project_menu_state.select(Some(2));
    app.active_panel = Panel::Main;

    app.pipelines = vec![
        sample_pipeline(101, "success"),
        sample_pipeline(102, "running"),
        sample_pipeline(103, "failed"),
    ];
    app.filter_lists();
    app.pipelines_state.select(Some(0));

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/42/pipelines/101/jobs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": 201, "name": "build-bin", "stage": "build", "status": "success", "allow_failure": false, "created_at": "2026-09-01T12:00:00Z", "web_url": "u", "tag_list": []},
            {"id": 202, "name": "test-unit", "stage": "test", "status": "running", "allow_failure": false, "created_at": "2026-09-01T12:00:00Z", "web_url": "u", "tag_list": []}
        ])))
        .mount(&server)
        .await;

    // Drill into pipeline jobs with Enter
    app.jobs = vec![
        sample_job(201, "build-bin", "build", "success"),
        sample_job(202, "test-unit", "test", "running"),
    ];
    app.handle_key(key(KeyCode::Enter)).await;
    assert_eq!(app.active_panel, Panel::MainDetail);
    app.jobs = vec![
        sample_job(201, "build-bin", "build", "success"),
        sample_job(202, "test-unit", "test", "running"),
    ];

    // Render frame to verify status pills render properly
    let backend = ratatui::backend::TestBackend::new(140, 40);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|f| lazygitlab::ui::ui(f, &mut app)).unwrap();
    let frame = lazygitlab::headless::render_to_string(&terminal);

    assert!(frame.contains("ci-project"));
    assert!(frame.contains("build-bin"));
    assert!(frame.contains("test-unit"));
}

async fn wait_for_status(
    app: &mut App,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
    needle: &str,
) {
    for _ in 0..100 {
        while let Ok(update) = rx.try_recv() {
            app.process_update_single(update).await;
        }
        if app
            .status_message
            .as_deref()
            .is_some_and(|m| m.contains(needle))
        {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn test_pipeline_retry_and_cancel_actions() {
    let (server, mut app, mut rx) = setup_app().await;
    app.selected_project = Some(sample_project());
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Pipelines;
    app.project_menu_state.select(Some(2));
    app.active_panel = Panel::Main;

    app.pipelines = vec![
        sample_pipeline(101, "failed"),
        sample_pipeline(102, "running"),
    ];
    app.filter_lists();
    app.pipelines_state.select(Some(0));

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/42/pipelines/101/retry"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 101, "project_id": 42, "status": "running", "source": "push", "ref": "main", "web_url": "u", "created_at": "2026-09-01T12:00:00Z"
        })))
        .mount(&server)
        .await;

    // Retry pipeline with 'R'
    app.handle_key(key(KeyCode::Char('R'))).await;
    wait_for_status(&mut app, &mut rx, "retry triggered").await;
    assert_eq!(
        app.status_message.as_deref(),
        Some("Pipeline #101 retry triggered")
    );

    // Select running pipeline and cancel with 'C'
    app.pipelines_state.select(Some(1));
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/42/pipelines/102/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 102, "project_id": 42, "status": "canceled", "source": "push", "ref": "main", "web_url": "u", "created_at": "2026-09-01T12:00:00Z"
        })))
        .mount(&server)
        .await;

    app.handle_key(key(KeyCode::Char('C'))).await;
    wait_for_status(&mut app, &mut rx, "cancelled").await;
    let cancel_msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        cancel_msg.contains("102")
            && (cancel_msg.contains("cancel") || cancel_msg.contains("Canceling"))
    );
}

#[tokio::test]
async fn test_pipeline_launch_popup_typing_p_does_not_launch() {
    let (_server, mut app, _rx) = setup_app().await;
    app.selected_project = Some(sample_project());
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Pipelines;
    app.project_menu_state.select(Some(2));
    app.active_panel = Panel::Main;

    // Press 'n' to open Launch Pipeline popup
    app.handle_key(key(KeyCode::Char('n'))).await;
    assert!(app.show_pipeline_popup);
    assert_eq!(app.input_target, InputTarget::PipelineRef);

    app.pipeline_ref_input.clear();

    // Type a branch name that contains 'p', e.g. "patch-p1"
    for ch in "patch-p1".chars() {
        app.handle_key(key(KeyCode::Char(ch))).await;
    }

    // Popup MUST still be open and ref input must accurately contain "patch-p1"
    assert!(
        app.show_pipeline_popup,
        "Popup must NOT close or launch on typing 'p'"
    );
    assert_eq!(app.pipeline_ref_input, "patch-p1");
}

#[tokio::test]
async fn test_job_log_viewer_search_next_prev_and_auto_follow() {
    let (_server, mut app, _rx) = setup_app().await;
    app.view_mode = ViewMode::JobLog;
    app.job_log_viewport_height = 10;
    app.job_log_lines = vec![
        LogLine {
            timestamp: String::new(),
            content: "Starting job runner".into(),
            section: None,
            is_section_start: false,
            is_section_end: false,
        },
        LogLine {
            timestamp: String::new(),
            content: "ERROR: dependency missing".into(),
            section: None,
            is_section_start: false,
            is_section_end: false,
        },
        LogLine {
            timestamp: String::new(),
            content: "Retrying step 1".into(),
            section: None,
            is_section_start: false,
            is_section_end: false,
        },
        LogLine {
            timestamp: String::new(),
            content: "ERROR: build failed".into(),
            section: None,
            is_section_start: false,
            is_section_end: false,
        },
    ];

    // Set search query
    app.job_log_search = "error".into();
    app.job_log_cursor = 0;

    // Press 'n' -> jumps forward to line 1
    app.handle_key(key(KeyCode::Char('n'))).await;
    assert_eq!(app.job_log_cursor, 1);

    // Press 'n' again -> jumps forward to line 3
    app.handle_key(key(KeyCode::Char('n'))).await;
    assert_eq!(app.job_log_cursor, 3);

    // Press 'N' -> jumps backward to line 1
    app.handle_key(key(KeyCode::Char('N'))).await;
    assert_eq!(app.job_log_cursor, 1);

    // Test auto-follow toggle with 'f'
    assert!(app.follow_log);
    app.handle_key(key(KeyCode::Char('f'))).await;
    assert!(!app.follow_log);
    assert_eq!(app.status_message.as_deref(), Some("Auto-follow disabled"));

    app.handle_key(key(KeyCode::Char('f'))).await;
    assert!(app.follow_log);
    assert_eq!(app.status_message.as_deref(), Some("Auto-follow enabled"));

    // Scrolling up with 'k' turns off auto-follow
    app.handle_key(key(KeyCode::Char('k'))).await;
    assert!(!app.follow_log, "Scrolling up must disable auto-follow");
}

#[tokio::test]
async fn test_job_log_section_collapsing_with_space_and_tab() {
    let (_server, mut app, _rx) = setup_app().await;
    app.view_mode = ViewMode::JobLog;
    app.job_log_viewport_height = 10;
    app.job_log_lines = vec![
        LogLine {
            timestamp: String::new(),
            content: "section_start:step_build[collapsed=true]".into(),
            section: Some("step_build".into()),
            is_section_start: true,
            is_section_end: false,
        },
        LogLine {
            timestamp: String::new(),
            content: "compiling src/main.rs".into(),
            section: Some("step_build".into()),
            is_section_start: false,
            is_section_end: false,
        },
        LogLine {
            timestamp: String::new(),
            content: "section_end:step_build".into(),
            section: Some("step_build".into()),
            is_section_start: false,
            is_section_end: true,
        },
    ];

    app.job_log_cursor = 0;
    assert!(!app.collapsed_sections.contains("step_build"));

    // Press Space to collapse section
    app.handle_key(key(KeyCode::Char(' '))).await;
    assert!(app.collapsed_sections.contains("step_build"));

    // Press Tab to expand section
    app.handle_key(key(KeyCode::Tab)).await;
    assert!(!app.collapsed_sections.contains("step_build"));
}

#[tokio::test]
async fn test_ci_variables_editor_live_typing_and_editing() {
    let (server, mut app, mut rx) = setup_app().await;
    app.selected_project = Some(sample_project());
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Pipelines;

    // Open CI vars popup with 'v'
    app.handle_key(key(KeyCode::Char('v'))).await;
    assert!(app.show_ci_vars_popup);

    // Press 'n' to create a new variable
    app.handle_key(key(KeyCode::Char('n'))).await;
    assert!(app.ci_var_editing.is_some());
    assert_eq!(app.input_target, InputTarget::PipelineVariableKey);

    // Type KEY_A
    for ch in "KEY_A".chars() {
        app.handle_key(key(KeyCode::Char(ch))).await;
    }
    assert_eq!(app.pipeline_variable_key_input, "KEY_A");

    // Press Tab to switch to value
    app.handle_key(key(KeyCode::Tab)).await;
    assert_eq!(app.input_target, InputTarget::PipelineVariableValue);

    // Type VAL_A
    for ch in "VAL_A".chars() {
        app.handle_key(key(KeyCode::Char(ch))).await;
    }
    assert_eq!(app.pipeline_variable_value_input, "VAL_A");

    Mock::given(method("POST"))
        .and(path("/api/v4/projects/42/variables"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "key": "KEY_A", "value": "VAL_A", "variable_type": "env_var", "protected": false, "masked": false, "environment_scope": "*"
        })))
        .mount(&server)
        .await;

    // Save with Ctrl+Enter
    app.handle_key(ctrl_key(KeyCode::Enter)).await;
    assert!(app.ci_var_editing.is_none());

    // Drain background tasks
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    while let Ok(update) = rx.try_recv() {
        app.process_update_single(update).await;
    }

    assert!(
        app.project_variables
            .iter()
            .any(|v| v.key == "KEY_A" && v.value == "VAL_A")
    );
}
