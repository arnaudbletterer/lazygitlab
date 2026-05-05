use std::{io, time::Duration};
use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::prelude::*;
use gitlab::GitlabBuilder;
use dotenvy::dotenv;
use lazygitlab::*;
use lazygitlab::ui::ui;

#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let config = Config::from_env();
    let client = GitlabBuilder::new(config.url, config.token).build_async().await?;
    let mut app = App::new(client);
    
    // Start TUI first
    run_with_terminal(&mut app).await
}

struct Config { url: String, token: String }
impl Config {
    fn from_env() -> Self {
        let url = std::env::var("GITLAB_URL").unwrap_or_else(|_| "gitlab.com".to_string());
        let token = std::env::var("GITLAB_TOKEN").unwrap_or_default();
        if token.is_empty() {
            panic!("GITLAB_TOKEN must be set");
        }
        Self { url, token }
    }
}

#[async_trait::async_trait]
pub trait EventSource {
    async fn next_event(&self, timeout: Duration) -> io::Result<Option<Event>>;
}

pub struct RealEventSource;

#[async_trait::async_trait]
#[cfg(not(tarpaulin_include))]
impl EventSource for RealEventSource {
    async fn next_event(&self, timeout: Duration) -> io::Result<Option<Event>> {
        if event::poll(timeout)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(not(tarpaulin_include))]
pub async fn run_with_terminal(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture, event::EnableFocusChange)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let event_source = RealEventSource;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    app.tx = tx;

    let res = run_app(&mut terminal, app, &event_source, &mut rx).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, event::DisableMouseCapture, event::DisableFocusChange)?;
    terminal.show_cursor()?;
    if let Err(err) = res { eprintln!("Application Error: {err}"); }
    Ok(())
}

async fn run_app<B: Backend, E: EventSource>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_source: &E,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
) -> Result<(), Box<dyn std::error::Error>> 
where 
    <B as Backend>::Error: 'static 
{
    // Initial data load
    let _ = app.initial_setup().await;
    let _ = app.refresh().await;

    while !app.should_quit {
        app.process_updates(rx).await;
        terminal.draw(|f| ui(f, app))?;

        let is_live_view = (app.view_mode == ViewMode::List && app.active_selection == MenuSelection::Pipelines) 
            || app.view_mode == ViewMode::JobLog
            || (app.view_mode == ViewMode::Detail && app.selected_pipeline.is_some());

        let refresh_interval = if is_live_view { Duration::from_secs(2) } else { Duration::from_secs(10) };

        if app.last_refresh.elapsed() >= refresh_interval {
            app.refresh().await;
        }

        if let Some(event) = event_source.next_event(Duration::from_millis(100)).await? {
            match event {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        app.handle_key(key).await;
                    }
                }
                Event::Mouse(mouse) => {
                    app.handle_mouse(mouse).await;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use std::sync::Mutex;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::path;

    struct MockEventSource {
        events: Mutex<Vec<Event>>,
    }

    #[async_trait::async_trait]
    impl EventSource for MockEventSource {
        async fn next_event(&self, _timeout: Duration) -> io::Result<Option<Event>> {
            let mut events = self.events.lock().unwrap();
            if events.is_empty() {
                Ok(None)
            } else {
                Ok(Some(events.remove(0)))
            }
        }
    }

    #[tokio::test]
    async fn test_run_app_full() {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        let mut app = App::new(client);
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        let event_source = MockEventSource {
            events: Mutex::new(vec![
                Event::Key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Char('q'), crossterm::event::KeyModifiers::NONE)),
            ]),
        };

        let (_tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        assert!(run_app(&mut terminal, &mut app, &event_source, &mut rx).await.is_ok());
    }

    #[tokio::test]
    async fn test_run_app_mouse_and_refresh() {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        let mut app = App::new(client);
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        // Trigger a quick refresh interval
        app.view_mode = ViewMode::JobLog; 
        app.last_refresh = std::time::Instant::now() - Duration::from_secs(5);

        let event_source = MockEventSource {
            events: Mutex::new(vec![
                Event::Mouse(crossterm::event::MouseEvent {
                    kind: crossterm::event::MouseEventKind::ScrollDown,
                    column: 0, row: 0, modifiers: crossterm::event::KeyModifiers::NONE,
                }),
                Event::Key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Char('q'), crossterm::event::KeyModifiers::NONE)),
            ]),
        };

        let (_tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        assert!(run_app(&mut terminal, &mut app, &event_source, &mut rx).await.is_ok());
    }

    static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    #[should_panic(expected = "GITLAB_TOKEN must be set")]
    fn test_config_from_env_panic() {
        let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::remove_var("GITLAB_TOKEN"); }
        Config::from_env();
    }

    #[test]
    fn test_config_from_env_success() {
        let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("GITLAB_TOKEN", "test_token");
            std::env::set_var("GITLAB_URL", "https://test.gitlab.com");
        }
        let config = Config::from_env();
        assert_eq!(config.token, "test_token");
        assert_eq!(config.url, "https://test.gitlab.com");
    }
}
