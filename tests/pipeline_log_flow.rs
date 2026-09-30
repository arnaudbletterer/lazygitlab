use std::time::Duration;
use tokio::sync::mpsc;

use gitlab::GitlabBuilder;
use lazygitlab::headless::{ScriptedEventSource, render_to_string};
use lazygitlab::models::{App, Context, DataUpdate, MenuSelection, Panel, ViewMode};
use lazygitlab::run_app;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

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
    let mut app = App::new(client, tx, None);
    app.initial_setup().await.expect("initial setup");
    wait_for_loading_finish(&mut app, &mut rx).await;

    let project = app
        .projects
        .iter()
        .find(|p| {
            p.path_with_namespace.as_deref() == Some(ground.project.path_with_namespace.as_str())
        })
        .cloned()
        .expect("test ground project not found");
    app.selected_project = Some(project.clone());
    app.active_context = Context::Project;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Pipelines;
    app.project_menu_state.select(Some(2));
    app.refresh().await;
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(
        !app.pipelines.is_empty(),
        "expected at least one pipeline in test ground"
    );

    let backend = TestBackend::new(180, 60);
    let mut terminal = Terminal::new(backend).unwrap();
    let frames: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let fc = std::sync::Arc::clone(&frames);
    let mut capture: Option<lazygitlab::runner::FrameCaptureFn<TestBackend>> =
        Some(Box::new(move |t| {
            fc.lock().unwrap().push(render_to_string(t));
        }));
    let src = ScriptedEventSource::from_text(script).unwrap();
    let _ = run_app(&mut terminal, &mut app, &src, &mut rx, &mut capture).await;
    let frames = frames.lock().unwrap().clone();
    (app, rx, frames)
}

fn pick_first_running_index(pipelines: &[lazygitlab::models::Pipeline]) -> Option<usize> {
    pipelines.iter().position(|p| p.status == "running")
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_open_then_open_job_log() {
    if common::skip_if_live_unavailable("live_full_pipeline_open_then_open_job_log") {
        return;
    }
    // Verifies the drill-in path doesn't get stuck on the first Enter even if the
    // pipeline happens to have no jobs (e.g. just created). With the sync-fetch
    // fix, the second Enter still gets us to JobLog (selected_job set, view_mode
    // changed) -- the log may be empty if there really are no jobs, but the user
    // reaches JobLog and can press Esc back.
    let (app, _rx, _frames) = drive("Enter\nEnter\nq").await;
    assert_eq!(
        app.view_mode,
        lazygitlab::models::ViewMode::JobLog,
        "expected JobLog after two Enters on a pipeline"
    );
    assert!(app.selected_job.is_some(), "expected a selected job");
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
    assert!(
        app.selected_job.is_some(),
        "single Enter sequence should have opened a job"
    );
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_log_flow_smoke() {
    if common::skip_if_live_unavailable("live_full_pipeline_log_flow_smoke") {
        return;
    }
    // Locates a *running* pipeline in the test ground (the only one that has a
    // live job log) and exercises the full flow on it. If none is running (e.g.
    // the cancel test ran and nothing is running), this test is skipped.
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.initial_setup().await.expect("initial setup");
    wait_for_loading_finish(&mut app, &mut rx).await;

    let project = app
        .projects
        .iter()
        .find(|p| {
            p.path_with_namespace.as_deref() == Some(ground.project.path_with_namespace.as_str())
        })
        .cloned()
        .expect("test ground project not found");
    app.selected_project = Some(project.clone());
    app.active_context = Context::Project;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Pipelines;
    app.project_menu_state.select(Some(2));
    app.refresh().await;
    wait_for_loading_finish(&mut app, &mut rx).await;

    let running_idx = match pick_first_running_index(&app.pipelines) {
        Some(i) => i,
        None => {
            eprintln!(
                "[log_flow_smoke] skipping: no running pipeline in test ground (re-run populate)"
            );
            return;
        }
    };
    app.pipelines_state.select(Some(running_idx));
    let target_pipeline = app.pipelines[running_idx].clone();
    eprintln!(
        "[log_flow_smoke] using pipeline {} (status {})",
        target_pipeline.id, target_pipeline.status
    );

    let backend = TestBackend::new(180, 60);
    let mut terminal = Terminal::new(backend).unwrap();
    let frames: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let fc = std::sync::Arc::clone(&frames);
    let mut capture: Option<lazygitlab::runner::FrameCaptureFn<TestBackend>> =
        Some(Box::new(move |t| {
            fc.lock().unwrap().push(render_to_string(t));
        }));

    let script = "Enter\nEnter\nj\nj\ny\nq";
    let src = ScriptedEventSource::from_text(script).unwrap();
    let _ = run_app(&mut terminal, &mut app, &src, &mut rx, &mut capture).await;

    assert_eq!(
        app.view_mode,
        ViewMode::JobLog,
        "expected JobLog after Enter Enter on the running pipeline"
    );
    assert!(app.selected_job.is_some(), "expected a selected job");
    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        msg.contains("Copied"),
        "expected 'Copied' status after y, got {msg:?}"
    );
}
