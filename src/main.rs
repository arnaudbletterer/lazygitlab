use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, enable_raw_mode},
};
use gitlab::GitlabBuilder;
use lazygitlab::headless::HeadlessOptions;
use lazygitlab::*;
use ratatui::prelude::*;
use std::{io, time::Duration};

#[cfg(not(tarpaulin_include))]
fn fail(code: i32, message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(code);
}

#[cfg(not(tarpaulin_include))]
fn resolve_settings(cli: &cli::CliOptions) -> config::Settings {
    let mut cli = cli.clone();
    if cli.token.as_deref() == Some("-") {
        let token =
            cli::read_token_line(&mut std::io::stdin().lock()).unwrap_or_else(|e| fail(1, &e));
        cli.token = Some(token);
    }
    let cli = &cli;
    let explicit = cli.config.is_some();
    let path = cli
        .config
        .as_ref()
        .map(std::path::PathBuf::from)
        .or_else(config::default_config_path);
    let file = match &path {
        Some(p) => config::load_file_config(p, explicit).unwrap_or_else(|e| fail(1, &e)),
        None => config::FileConfig::default(),
    };
    let mut settings = config::Settings::from_env(cli, &file);
    if settings.token.is_some() {
        return settings;
    }

    use std::io::IsTerminal;
    if !(std::io::stdin().is_terminal() && std::io::stdout().is_terminal()) {
        fail(
            1,
            "No GitLab token found.\n\
Provide one with the GITLAB_TOKEN environment variable, \
the --token flag, or a [gitlab] table with `token` in ~/.config/lazygitlab/config.toml \
(or the file given by --config).",
        );
    }
    let stdin = std::io::stdin();
    let outcome = setup::run_wizard(
        &mut stdin.lock(),
        &mut std::io::stdout(),
        &settings.url,
        true,
    )
    .unwrap_or_else(|e| fail(1, &format!("Setup aborted: {e}")));
    let Some(outcome) = outcome else {
        fail(1, "Setup aborted: no token provided.");
    };
    let (url, http) = config::normalize_url(&outcome.url);
    let insecure = settings.insecure || http;
    settings.plain_http = settings.plain_http || http;
    if outcome.save {
        let saved_url = if settings.plain_http {
            format!("http://{url}")
        } else {
            url.clone()
        };
        match path {
            Some(p) => match config::save_gitlab_section(&p, &saved_url, &outcome.token) {
                Ok(()) => println!("Saved to {}", p.display()),
                Err(e) => eprintln!("Could not save configuration: {e}"),
            },
            None => eprintln!("Could not save configuration: no config directory found"),
        }
    }
    if !url.is_empty() {
        settings.url = url;
    }
    settings.insecure = insecure;
    settings.token = Some(outcome.token);
    settings
}

#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = match cli::parse_args(&args) {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("error: {e}\n\n{}", cli::usage());
            std::process::exit(2);
        }
    };
    if cli.help {
        print!("{}", cli::usage());
        return Ok(());
    }
    if cli.version {
        println!("{}", cli::version_line());
        return Ok(());
    }

    let _guard = crate::logger::init();
    let settings = resolve_settings(&cli);
    let token = settings.token.clone().unwrap_or_default();
    lazygitlab::redaction::register_secret(&token);
    let transport = settings.transport();
    if transport == config::Transport::HttpsSkipVerify {
        eprintln!("{}", config::SKIP_VERIFY_WARNING);
    }
    let mut builder = GitlabBuilder::new(settings.url, token);
    match transport {
        config::Transport::Https => {}
        config::Transport::PlainHttp => {
            builder.insecure();
        }
        config::Transport::HttpsSkipVerify => {
            builder.cert_insecure();
        }
    }
    let client = match builder.build_async().await {
        Ok(client) => client,
        Err(e) => fail(
            1,
            &format!(
                "Could not connect to GitLab: {}",
                errors::friendly_error(&e.to_string())
            ),
        ),
    };
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client, tx, settings.project.clone());
    app.skip_tls_verify = transport == config::Transport::HttpsSkipVerify;
    if let Some(spec) = &settings.project
        && let Err(e) = app.preselect_project(spec).await
    {
        app.last_error = Some(e);
        app.last_error_since = Some(std::time::Instant::now());
    }

    if cli.headless {
        run_headless(&mut app, &mut rx, cli.headless_options()).await
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
    use lazygitlab::headless::{FrameCapture, ScriptedEventSource, headless_terminal, load_script};
    use ratatui::backend::TestBackend;

    let mut terminal: ratatui::Terminal<TestBackend> = headless_terminal(opts.width, opts.height);
    let capture = match &opts.frames_dir {
        Some(dir) => FrameCapture::new(dir)?,
        None => FrameCapture::disabled(),
    };

    let event_source: ScriptedEventSource = match &opts.script_path {
        Some(path) => ScriptedEventSource::from_text(&load_script(path)?)?,
        None => ScriptedEventSource::empty(),
    };

    let mut capture_opt: Option<lazygitlab::runner::FrameCaptureFn<TestBackend>> =
        if capture.is_enabled() {
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

pub use lazygitlab::EventSource;

pub struct RealEventSource;

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
pub async fn run_with_terminal(
    app: &mut App,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal::install_panic_hook();
    let _restore = terminal::RestoreOnDrop;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        event::EnableMouseCapture,
        event::EnableFocusChange
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let event_source = RealEventSource;
    let res = run_app(&mut terminal, app, &event_source, rx, &mut None).await;

    terminal::restore();
    if let Err(err) = res {
        eprintln!(
            "Application Error: {}",
            lazygitlab::redaction::redact_secrets(&err.to_string())
        );
    }
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
    use wiremock::matchers::path;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    struct MockEventSource {
        events: Mutex<Vec<Event>>,
    }

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
        Mock::given(path("/api/v4/user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id":1,"username":"u"})),
            )
            .mount(&server)
            .await;

        let client = GitlabBuilder::new(server.uri().replace("http://", ""), "t")
            .insecure()
            .build_async()
            .await
            .unwrap();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(client, tx, None);
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        let event_source = MockEventSource {
            events: Mutex::new(vec![Event::Key(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('q'),
                crossterm::event::KeyModifiers::NONE,
            ))]),
        };

        let mut noop: Option<lazygitlab::runner::FrameCaptureFn<TestBackend>> = None;
        assert!(
            run_app(&mut terminal, &mut app, &event_source, &mut rx, &mut noop)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_run_app_mouse_and_refresh() {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id":1,"username":"u"})),
            )
            .mount(&server)
            .await;

        let client = GitlabBuilder::new(server.uri().replace("http://", ""), "t")
            .insecure()
            .build_async()
            .await
            .unwrap();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(client, tx, None);
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        // Trigger a quick refresh interval
        app.view_mode = ViewMode::JobLog;
        app.last_refresh = std::time::Instant::now() - Duration::from_secs(5);

        let event_source = MockEventSource {
            events: Mutex::new(vec![
                Event::Mouse(crossterm::event::MouseEvent {
                    kind: crossterm::event::MouseEventKind::ScrollDown,
                    column: 0,
                    row: 0,
                    modifiers: crossterm::event::KeyModifiers::NONE,
                }),
                Event::Key(crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char('q'),
                    crossterm::event::KeyModifiers::NONE,
                )),
            ]),
        };

        assert!(
            run_app(&mut terminal, &mut app, &event_source, &mut rx, &mut None)
                .await
                .is_ok()
        );
    }
}
