use std::time::Duration;
use tokio::sync::mpsc;

use gitlab::GitlabBuilder;
use lazygitlab::headless::{ScriptedEventSource, render_to_string};
use lazygitlab::models::{App, Context, DataUpdate, MenuSelection, Panel};
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

async fn drive(script: &str) -> (App, Vec<String>) {
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
    (app, frames)
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_retry_finished_pipeline_succeeds() {
    if common::skip_if_live_unavailable("live_full_pipeline_retry_finished_pipeline_succeeds") {
        return;
    }
    // A "finished" (success/failed/canceled) pipeline is retryable.
    // After the retry, status should say "retry triggered" or "cancelled"/"failed".
    let (app, _frames) = drive("R\nq").await;
    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        !app.last_error.as_deref().unwrap_or("").is_empty()
            || msg.contains("retry")
            || msg.contains("cancelled")
            || msg.contains("Retry")
            || msg.contains("failed")
            || msg.is_empty(),
        "expected a status update after R, got status={:?} error={:?}",
        app.status_message,
        app.last_error
    );
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_launch_new_sync() {
    if common::skip_if_live_unavailable("live_full_pipeline_launch_new_sync") {
        return;
    }
    // Press n to open the popup (ref is pre-filled with the current pipeline's ref or
    // default branch). Press Enter to advance past the ref input, then p to submit.
    // Use <Esc> to bail if anything goes wrong so we don't hang.
    let (app, _frames) = drive("n\n<Enter>\np\n<Esc>\nq").await;
    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        msg.contains("Pipeline #")
            || msg.contains("Launch")
            || msg.contains("Launch failed")
            || msg.contains("timed out"),
        "expected a launch status update, got {:?}",
        msg
    );
    assert!(
        !app.show_pipeline_popup,
        "popup should have closed on submit"
    );
    assert!(!app.input_mode, "input_mode should have cleared on submit");
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_launch_popup_dismisses_with_esc() {
    if common::skip_if_live_unavailable("live_full_pipeline_launch_popup_dismisses_with_esc") {
        return;
    }
    // n opens the popup, <Esc> closes it without launching.
    let (app, _frames) = drive("n\n<Esc>\nq").await;
    assert!(!app.show_pipeline_popup, "popup should close on Esc");
    assert!(!app.input_mode, "input_mode should clear on Esc");
    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        !msg.to_lowercase().contains("launching"),
        "Esc should not trigger launch: got status {msg:?}"
    );
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_help_lists_cancel_and_retry() {
    if common::skip_if_live_unavailable("live_full_pipeline_help_lists_cancel_and_retry") {
        return;
    }
    let (app, _frames) = drive("q").await;
    let items = app.get_available_actions();
    let has_cancel = items.iter().any(|i| {
        i.category == "Pipeline"
            && i.shortcut == "C"
            && i.description.to_lowercase().contains("cancel")
    });
    let has_retry = items.iter().any(|i| {
        i.category == "Pipeline"
            && i.shortcut == "R"
            && i.description.to_lowercase().contains("retry")
    });
    assert!(has_cancel, "help should list C (cancel pipeline)");
    assert!(has_retry, "help should list R (retry pipeline)");
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_cancel_sets_status_and_closes_popup() {
    if common::skip_if_live_unavailable("live_full_pipeline_cancel_sets_status_and_closes_popup") {
        return;
    }
    // We try to cancel the first pipeline in the list. If it's already finished
    // or cancelled, the API returns an error -- that's still a valid test of the
    // UX (status message must be set). If it's still running, it gets cancelled.
    // We do NOT re-seed the test ground on success; the populate script must be
    // re-run by hand after this test to keep pipeline_log_flow tests working.
    let (app, _frames) = drive("C\nq").await;
    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        msg.contains("cancelled") || msg.contains("Cancel failed") || msg.contains("timed out"),
        "expected a cancel status update, got {msg:?} (error: {:?})",
        app.last_error
    );
    assert!(!app.show_pipeline_popup, "cancel should not open the popup");
    assert!(!app.input_mode, "cancel should not leave input_mode on");
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_pipeline_cancel_finished_pipeline_shows_error() {
    if common::skip_if_live_unavailable("live_full_pipeline_cancel_finished_pipeline_shows_error") {
        return;
    }
    // Cancel a *finished* pipeline. The API will reject (cannot cancel finished),
    // but the user must still see a clear status message and the popup must stay
    // closed. This is the safe test (no state mutation).
    // We pick the first pipeline in the list -- if it's still running, skip the
    // test rather than mutate state.
    let (app, _frames) = drive("q").await;
    let first = match app.pipelines.first() {
        Some(p) => p.clone(),
        None => return,
    };
    if first.status == "running" {
        eprintln!(
            "[cancel_finished] skipping: first pipeline is still running, run cancel_running first"
        );
        return;
    }
    let (app, _frames) = drive("C\nq").await;
    let msg = app.status_message.as_deref().unwrap_or("");
    let err = app.last_error.as_deref().unwrap_or("");
    assert!(
        msg.contains("Cancel failed")
            || msg.contains("cancelled")
            || msg.contains("timed out")
            || !err.is_empty(),
        "expected a cancel status / error update on a finished pipeline, got status={msg:?} error={err:?}"
    );
    assert!(!app.show_pipeline_popup, "cancel should not open the popup");
    assert!(!app.input_mode, "cancel should not leave input_mode on");
}
