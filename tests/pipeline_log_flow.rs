use std::time::Duration;
use tokio::sync::mpsc;

use gitlab::GitlabBuilder;
use lazygitlab::headless::{render_to_string, ScriptedEventSource};
use lazygitlab::models::{App, DataUpdate, MenuSelection, Panel, Context, ViewMode};
use lazygitlab::run_app;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

mod common;

async fn try_build_client() -> Option<gitlab::AsyncGitlab> {
    let _ = dotenvy::dotenv();
    let token = match std::env::var("GITLAB_TOKEN") {
        Ok(t) if !t.trim().is_empty() => t,
        _ => return None,
    };
    let url = std::env::var("GITLAB_URL").unwrap_or_else(|_| "https://gitlab.com".to_string());
    match GitlabBuilder::new(url.replace("https://", "").replace("http://", ""), token)
        .build_async()
        .await
    {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("Skipping test: Failed to create GitLab client: {}", e);
            None
        }
    }
}

async fn wait_for_loading_finish(app: &mut App, rx: &mut mpsc::UnboundedReceiver<DataUpdate>) {
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(20) {
        app.process_updates(rx).await;
        if !app.is_loading {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn drive(script: &str) -> (App, mpsc::UnboundedReceiver<DataUpdate>, Vec<String>) {
    let Some(client) = try_build_client().await else {
        panic!("test pre-condition: GITLAB_TOKEN must be set");
    };
    let ground = common::test_ground();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx);
    app.initial_setup().await.expect("initial setup");
    wait_for_loading_finish(&mut app, &mut rx).await;

    let project = app
        .projects
        .iter()
        .find(|p| p.path_with_namespace.as_deref() == Some(ground.project.path_with_namespace.as_str()))
        .cloned()
        .expect("test ground project not found");
    app.selected_project = Some(project.clone());
    app.active_context = Context::Project;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Pipelines;
    app.project_menu_state.select(Some(2));
    app.refresh().await;
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(!app.pipelines.is_empty(), "expected at least one pipeline in test ground");

    let backend = TestBackend::new(180, 60);
    let mut terminal = Terminal::new(backend).unwrap();
    let frames: std::sync::Arc<std::sync::Mutex<Vec<String>>> = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let fc = std::sync::Arc::clone(&frames);
    let mut capture: Option<lazygitlab::runner::FrameCaptureFn<TestBackend>> = Some(Box::new(move |t| {
        fc.lock().unwrap().push(render_to_string(t));
    }));
    let src = ScriptedEventSource::from_text(script).unwrap();
    let _ = run_app(&mut terminal, &mut app, &src, &mut rx, &mut capture).await;
    let frames = frames.lock().unwrap().clone();
    (app, rx, frames)
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_open_then_open_job_log() {
    if common::skip_if_live_unavailable("live_full_pipeline_open_then_open_job_log") {
        return;
    }
    let (app, _rx, _frames) = drive("Enter\nEnter\nq").await;
    assert_eq!(app.view_mode, ViewMode::JobLog, "expected JobLog after two Enters on a pipeline");
    assert!(app.selected_job.is_some(), "expected a selected job");
    assert!(!app.job_log_lines.is_empty(), "expected the job log to be loaded synchronously, got {} lines", app.job_log_lines.len());
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_log_navigate_and_copy_current_line() {
    if common::skip_if_live_unavailable("live_full_pipeline_log_navigate_and_copy_current_line") {
        return;
    }
    let (app, _rx, _frames) = drive("Enter\nEnter\nj\nj\ny\nq").await;
    assert_eq!(app.view_mode, ViewMode::JobLog);
    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(msg.contains("Copied"), "expected 'Copied to clipboard' status, got {msg:?}");
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_log_select_and_copy_multiline() {
    if common::skip_if_live_unavailable("live_full_pipeline_log_select_and_copy_multiline") {
        return;
    }
    let (app, _rx, _frames) = drive("Enter\nEnter\nj\nj\n<S-j>\n<S-j>\ny\nq").await;
    assert_eq!(app.view_mode, ViewMode::JobLog);
    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(msg.contains("Copied 3 lines"), "expected multi-line copy, got {msg:?}");
    assert!(app.job_log_selection_start.is_none(), "selection should be cleared after copy");
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_log_section_toggle() {
    if common::skip_if_live_unavailable("live_full_pipeline_log_section_toggle") {
        return;
    }
    let (app, _rx, _frames) = drive("Enter\nEnter\nj\nj\nj\nj\nTab\nq").await;
    assert_eq!(app.view_mode, ViewMode::JobLog);
    assert!(!app.collapsed_sections.is_empty(), "Tab should have collapsed at least one section");
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_open_navigate_works_with_no_prefetch() {
    if common::skip_if_live_unavailable("live_full_pipeline_open_navigate_works_with_no_prefetch") {
        return;
    }
    // Regression: previously, pressing Enter on MainDetail with Pipelines did
    // nothing if the jobs had not yet been fetched. The user had to press Enter
    // twice (once to drill in, once again after the fetch completed) and often
    // thought the interaction was broken. After the fix, a single Enter sequence
    // should open the job log.
    let (app, _rx, _frames) = drive("Enter\nEnter\nq").await;
    assert!(app.selected_job.is_some(), "single Enter sequence should have opened a job");
    assert!(!app.job_log_lines.is_empty(), "log must be loaded after a single Enter sequence");
}
