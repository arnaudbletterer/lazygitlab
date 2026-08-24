use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_app() -> (
    MockServer,
    App,
    tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
) {
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
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    (server, App::new(client, tx), rx)
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn key_shift(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::SHIFT,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn make_log_line(content: &str) -> LogLine {
    LogLine {
        timestamp: "t".into(),
        content: content.into(),
        section: None,
        is_section_start: false,
        is_section_end: false,
    }
}

#[tokio::test]
async fn test_job_log_copy_current_line() {
    let (_server, mut app, mut rx) = setup_app().await;
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Pipelines;
    app.selected_project = Some(Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    });
    app.selected_pipeline = Some(Pipeline {
        id: 100,
        project_id: 1,
        status: "success".into(),
        source: "trigger".into(),
        ref_name: "main".into(),
        web_url: "u".into(),
        created_at: "t".into(),
        started_at: None,
        duration: None,
    });
    app.selected_job = Some(Job {
        id: 1,
        name: "build".into(),
        stage: "build".into(),
        status: "success".into(),
        created_at: "t".into(),
        started_at: None,
        finished_at: None,
        duration: None,
        user: None,
        runner: None,
        allow_failure: false,
        web_url: "u".into(),
        tag_list: vec![],
    });
    app.job_log_lines = vec![make_log_line("hello world")];
    app.view_mode = ViewMode::JobLog;
    app.job_log_cursor = 0;
    app.job_log_selection_start = None;

    app.handle_key(key(KeyCode::Char('y'))).await;

    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        msg.contains("Copied"),
        "expected status message to confirm copy, got {msg:?}"
    );

    let mut got_osc = false;
    while let Ok(update) = rx.try_recv() {
        if let DataUpdate::TerminalEscape(seq) = update {
            assert!(
                seq.contains("\x1b]52;c;"),
                "expected OSC 52 escape, got {seq:?}"
            );
            got_osc = true;
        }
    }
    assert!(got_osc, "expected OSC 52 escape to be sent on copy");
}

#[tokio::test]
async fn test_job_log_copy_selection() {
    let (_server, mut app, _rx) = setup_app().await;
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Pipelines;
    app.selected_project = Some(Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    });
    app.selected_pipeline = Some(Pipeline {
        id: 100,
        project_id: 1,
        status: "success".into(),
        source: "trigger".into(),
        ref_name: "main".into(),
        web_url: "u".into(),
        created_at: "t".into(),
        started_at: None,
        duration: None,
    });
    app.selected_job = Some(Job {
        id: 1,
        name: "build".into(),
        stage: "build".into(),
        status: "success".into(),
        created_at: "t".into(),
        started_at: None,
        finished_at: None,
        duration: None,
        user: None,
        runner: None,
        allow_failure: false,
        web_url: "u".into(),
        tag_list: vec![],
    });
    app.job_log_lines = vec![
        make_log_line("line 1"),
        make_log_line("line 2"),
        make_log_line("line 3"),
    ];
    app.view_mode = ViewMode::JobLog;
    app.job_log_cursor = 0;
    app.job_log_selection_start = None;

    app.handle_key(key_shift(KeyCode::Down)).await;
    app.handle_key(key_shift(KeyCode::Down)).await;
    assert!(
        app.job_log_selection_start.is_some(),
        "Shift+Down should start a selection"
    );

    app.handle_key(key(KeyCode::Char('y'))).await;

    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        msg.contains("Copied 3 lines"),
        "expected status message to mention 3 lines, got {msg:?}"
    );
}

#[tokio::test]
async fn test_job_log_help_popup_lists_copy_shortcut() {
    let (_server, mut app, _rx) = setup_app().await;
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Pipelines;
    app.selected_project = Some(Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    });
    app.selected_pipeline = Some(Pipeline {
        id: 100,
        project_id: 1,
        status: "success".into(),
        source: "trigger".into(),
        ref_name: "main".into(),
        web_url: "u".into(),
        created_at: "t".into(),
        started_at: None,
        duration: None,
    });
    app.selected_job = Some(Job {
        id: 1,
        name: "build".into(),
        stage: "build".into(),
        status: "success".into(),
        created_at: "t".into(),
        started_at: None,
        finished_at: None,
        duration: None,
        user: None,
        runner: None,
        allow_failure: false,
        web_url: "u".into(),
        tag_list: vec![],
    });
    app.view_mode = ViewMode::JobLog;

    let items = app.get_available_actions();
    let has_copy = items.iter().any(|i| {
        i.category == "Job Log"
            && i.shortcut == "y"
            && i.description.to_lowercase().contains("copy")
    });
    let has_extend = items
        .iter()
        .any(|i| i.category == "Job Log" && i.shortcut.to_lowercase().contains("shift"));
    let has_toggle = items
        .iter()
        .any(|i| i.category == "Job Log" && i.shortcut.to_lowercase().contains("tab"));
    assert!(has_copy, "JobLog help should list the y (copy) shortcut");
    assert!(
        has_extend,
        "JobLog help should list the Shift+jk (extend selection) shortcut"
    );
    assert!(
        has_toggle,
        "JobLog help should list the Tab (toggle section) shortcut"
    );
}

#[tokio::test]
async fn test_pipeline_help_lists_launch_shortcut() {
    let (_server, mut app, _rx) = setup_app().await;
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Pipelines;
    app.selected_project = Some(Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    });
    app.selected_pipeline = Some(Pipeline {
        id: 100,
        project_id: 1,
        status: "success".into(),
        source: "trigger".into(),
        ref_name: "main".into(),
        web_url: "u".into(),
        created_at: "t".into(),
        started_at: None,
        duration: None,
    });

    let items = app.get_available_actions();
    let has_launch = items.iter().any(|i| {
        i.category == "Pipeline"
            && i.shortcut == "n"
            && i.description.to_lowercase().contains("launch")
    });
    assert!(
        has_launch,
        "Pipeline help should list the n (launch new pipeline) shortcut"
    );
}

#[tokio::test]
async fn test_job_log_help_lists_retry_for_failed_job() {
    let (_server, mut app, _rx) = setup_app().await;
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Pipelines;
    app.selected_project = Some(Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    });
    app.selected_pipeline = Some(Pipeline {
        id: 100,
        project_id: 1,
        status: "failed".into(),
        source: "trigger".into(),
        ref_name: "main".into(),
        web_url: "u".into(),
        created_at: "t".into(),
        started_at: None,
        duration: None,
    });
    app.selected_job = Some(Job {
        id: 1,
        name: "build".into(),
        stage: "build".into(),
        status: "failed".into(),
        created_at: "t".into(),
        started_at: None,
        finished_at: None,
        duration: None,
        user: None,
        runner: None,
        allow_failure: false,
        web_url: "u".into(),
        tag_list: vec![],
    });
    app.view_mode = ViewMode::JobLog;

    let items = app.get_available_actions();
    let has_retry = items
        .iter()
        .any(|i| i.category == "Job Log" && i.description.to_lowercase().contains("retry"));
    assert!(
        has_retry,
        "JobLog help should list retry shortcut when job is failed"
    );
}
