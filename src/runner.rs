use std::time::Duration;

use ratatui::backend::Backend;
use ratatui::Terminal;
use tokio::sync::mpsc;

use crate::events::EventSource;
use crate::models::App;
use crate::models::DataUpdate;
use crate::ui::ui;

pub type FrameCaptureFn<B> = Box<dyn for<'a> FnMut(&'a Terminal<B>)>;

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
    if let Err(e) = app.initial_setup().await {
        app.last_error = Some(format!("Startup failed: {}", e));
        app.last_error_since = Some(std::time::Instant::now());
    }
    app.refresh().await;

    while !app.should_quit {
        app.tick_status();
        let mut got_update = false;
        while let Ok(update) = rx.try_recv() {
            got_update = true;
            if let DataUpdate::TerminalEscape(esc) = update {
                use std::io::Write;
                let mut stdout = std::io::stdout();
                let _ = stdout.write_all(esc.as_bytes());
                let _ = stdout.flush();
            } else {
                app.process_update_single(update).await;
            }
        }
        if got_update {
            app.update_preview().await;
        }

        terminal.draw(|f| ui(f, app))?;
        if let Some(capture) = frame_capture.as_deref_mut() {
            capture(terminal);
        }

        let is_live_view = (app.view_mode == crate::models::ViewMode::List
                && app.active_selection == crate::models::MenuSelection::Pipelines
                && app.pipelines.iter().any(|p| !p.is_finished()))
            || (app.view_mode == crate::models::ViewMode::JobLog
                && app.selected_job.as_ref().is_some_and(|j| !j.is_finished()))
            || (app.view_mode == crate::models::ViewMode::Detail
                && app.selected_pipeline.as_ref().is_some_and(|p| !p.is_finished()));

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

        if let Some(event) = event_source.next_event(Duration::from_millis(100)).await? {
            match event {
                crossterm::event::Event::Key(key) => {
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        app.handle_key(key).await;
                    }
                }
                crossterm::event::Event::Mouse(mouse) => {
                    app.handle_mouse(mouse).await;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
