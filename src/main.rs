use std::{io, time::Duration};
use crossterm::{
    event::{self, Event},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::prelude::*;
use gitlab::GitlabBuilder;
use dotenvy::dotenv;
use lazygitlab::*;
use lazygitlab::headless::{self, HeadlessOptions};

#[cfg(not(tarpaulin_include))]
fn is_headless_requested() -> bool {
    std::env::args().any(|a| a == "--headless" || a.starts_with("--headless="))
}

#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let _guard = crate::logger::init();
    let config = Config::from_env();
    let mut builder = GitlabBuilder::new(config.url, config.token);
    if config.insecure {
        builder.insecure();
    }
    let client = builder.build_async().await?;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client, tx);

    if is_headless_requested() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let opts = headless::parse_options(&args).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        run_headless(&mut app, &mut rx, opts).await
    } else {
        run_with_terminal(&mut app, &mut rx).await
    }
}

#[cfg(not(tarpaulin_include))]
pub async fn run_headless(
    app: &mut App,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
    opts: HeadlessOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    use ratatui::backend::TestBackend;
    use lazygitlab::headless::{headless_terminal, FrameCapture, ScriptedEventSource, load_script};

    let mut terminal: ratatui::Terminal<TestBackend> = headless_terminal(opts.width, opts.height);
    let capture = match &opts.frames_dir {
        Some(dir) => FrameCapture::new(dir)?,
        None => FrameCapture::disabled(),
    };

    let event_source: ScriptedEventSource = match &opts.script_path {
        Some(path) => ScriptedEventSource::from_text(&load_script(path)?)?,
        None => ScriptedEventSource::empty(),
    };

    let mut capture_opt: Option<lazygitlab::runner::FrameCaptureFn<TestBackend>> = if capture.is_enabled() {
        let mut cap = capture;
        Some(Box::new(move |t: &ratatui::Terminal<TestBackend>| {
            let _ = cap.record(t);
        }))
    } else {
        None
    };

    run_app(&mut terminal, app, &event_source, rx, &mut capture_opt).await?;

    Ok(())
}

struct Config { url: String, token: String, insecure: bool }
impl Config {
    fn from_env() -> Self {
        let mut url = std::env::var("GITLAB_URL").unwrap_or_else(|_| "gitlab.com".to_string());
        let mut insecure = false;
        if url.starts_with("http://") {
            insecure = true;
            url = url.strip_prefix("http://").unwrap().to_string();
        } else if url.starts_with("https://") {
            url = url.strip_prefix("https://").unwrap().to_string();
        }
        
        while url.ends_with('/') {
            url.pop();
        }

        let token = std::env::var("GITLAB_TOKEN").unwrap_or_default();
        if token.is_empty() {
            panic!("GITLAB_TOKEN must be set");
        }
        Self { url, token, insecure }
    }
}

pub use lazygitlab::EventSource;

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
pub async fn run_with_terminal(app: &mut App, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture, event::EnableFocusChange)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let event_source = RealEventSource;
    let res = run_app(&mut terminal, app, &event_source, rx, &mut None).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, event::DisableMouseCapture, event::DisableFocusChange)?;
    terminal.show_cursor()?;
    if let Err(err) = res { eprintln!("Application Error: {err}"); }
    Ok(())
}

#[allow(clippy::type_complexity)]
async fn run_app<B, E>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_source: &E,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
    frame_capture: &mut Option<Box<dyn for<'a> FnMut(&'a Terminal<B>)>>,
) -> Result<(), Box<dyn std::error::Error>> 
where 
    B: Backend,
    B::Error: 'static,
    E: EventSource,
{
    lazygitlab::run_app(terminal, app, event_source, rx, frame_capture).await
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
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(client, tx);
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        let event_source = MockEventSource {
            events: Mutex::new(vec![
                Event::Key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Char('q'), crossterm::event::KeyModifiers::NONE)),
            ]),
        };

        let mut noop: Option<lazygitlab::runner::FrameCaptureFn<TestBackend>> = None;
        assert!(run_app(&mut terminal, &mut app, &event_source, &mut rx, &mut noop).await.is_ok());
    }

    #[tokio::test]
    async fn test_run_app_mouse_and_refresh() {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(client, tx);
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

        assert!(run_app(&mut terminal, &mut app, &event_source, &mut rx, &mut None).await.is_ok());
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
        assert_eq!(config.url, "test.gitlab.com");
        assert!(!config.insecure);

        unsafe {
            std::env::set_var("GITLAB_URL", "http://test.gitlab.com/");
        }
        let config = Config::from_env();
        assert_eq!(config.url, "test.gitlab.com");
        assert!(config.insecure);
    }
}
