use std::time::Duration;

use ratatui::Terminal;
use ratatui::backend::Backend;
use tokio::sync::mpsc;

use crate::events::EventSource;
use crate::models::App;
use crate::models::DataUpdate;
use crate::ui::ui;

pub type FrameCaptureFn<B> = Box<dyn for<'a> FnMut(&'a Terminal<B>)>;

const INPUT_POLL: Duration = Duration::from_millis(100);

fn draw_frame<B>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    frame_capture: &mut Option<FrameCaptureFn<B>>,
) -> Result<(), Box<dyn std::error::Error>>
where
    B: Backend,
    B::Error: 'static,
{
    let completed = terminal.draw(|f| ui(f, app))?;
    if crate::headless::observing() {
        crate::headless::observe(completed.buffer, app);
    }
    if let Some(capture) = frame_capture.as_deref_mut() {
        capture(terminal);
    }
    Ok(())
}

async fn apply_update(app: &mut App, update: DataUpdate) {
    if let DataUpdate::TerminalEscape(esc) = update {
        use std::io::Write;
        let mut stdout = std::io::stdout();
        let _ = stdout.write_all(esc.as_bytes());
        let _ = stdout.flush();
    } else {
        app.process_update_single(update).await;
    }
}

#[allow(clippy::type_complexity)]
pub async fn run_app<B, E>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_source: &E,
    rx: &mut mpsc::UnboundedReceiver<DataUpdate>,
    frame_capture: &mut Option<FrameCaptureFn<B>>,
) -> Result<(), Box<dyn std::error::Error>>
where
    B: Backend,
    B::Error: 'static,
    E: EventSource,
{
    app.is_loading = true;
    draw_frame(terminal, app, frame_capture)?;
    let mut last_draw = std::time::Instant::now();

    if let Some(spec) = app.preselect_spec.take()
        && let Err(e) = app.preselect_project(&spec).await
    {
        app.last_error = Some(e);
        app.last_error_since = Some(std::time::Instant::now());
    }
    if let Err(e) = app.initial_setup().await {
        app.last_error = Some(format!("Startup failed: {}", e));
        app.last_error_since = Some(std::time::Instant::now());
    }
    app.refresh().await;

    let mut needs_draw = true;
    let mut pending = Box::pin(event_source.next_event(INPUT_POLL));

    while !app.should_quit {
        let status_before = app.status_message.clone();
        app.tick_status();
        if status_before != app.status_message {
            needs_draw = true;
        }

        let mut got_update = false;
        while let Ok(update) = rx.try_recv() {
            got_update = true;
            apply_update(app, update).await;
        }
        if got_update {
            app.update_preview().await;
            needs_draw = true;
        }

        // Redraw only when something changed, or at spinner cadence while loading.
        let animate = app.is_loading && last_draw.elapsed() >= Duration::from_millis(150);
        if needs_draw || animate {
            draw_frame(terminal, app, frame_capture)?;
            needs_draw = false;
            last_draw = std::time::Instant::now();
        }

        let is_live_view = (app.view_mode == crate::models::ViewMode::List
            && app.active_selection == crate::models::MenuSelection::Pipelines
            && app.pipelines.iter().any(|p| !p.is_finished()))
            || (app.view_mode == crate::models::ViewMode::JobLog
                && app.selected_job.as_ref().is_some_and(|j| !j.is_finished()))
            || (app.view_mode == crate::models::ViewMode::Detail
                && app
                    .selected_pipeline
                    .as_ref()
                    .is_some_and(|p| !p.is_finished()));

        let idle_threshold = Duration::from_secs(60);
        let is_idle = app.last_input.elapsed() >= idle_threshold;

        let refresh_interval = if is_idle {
            Duration::from_secs(30)
        } else if is_live_view {
            Duration::from_secs(2)
        } else {
            Duration::from_secs(5)
        };

        if app.last_refresh.elapsed() >= refresh_interval {
            app.refresh_background().await;
        }
        app.run_deferred().await;

        // Wake on whichever comes first: a network result or an input event. The
        // input future is kept across wake-ups so that no event is lost.
        let event = tokio::select! {
            biased;
            update = rx.recv() => {
                if let Some(update) = update {
                    apply_update(app, update).await;
                    app.update_preview().await;
                    needs_draw = true;
                }
                None
            }
            res = &mut pending => {
                pending = Box::pin(event_source.next_event(INPUT_POLL));
                res?
            }
        };

        if let Some(event) = event {
            match event {
                crossterm::event::Event::Key(key) => {
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        app.handle_key(key).await;
                        needs_draw = true;
                    }
                }
                crossterm::event::Event::Mouse(mouse) => {
                    app.handle_mouse(mouse).await;
                    needs_draw = true;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
