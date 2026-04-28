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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let config = Config::from_env();
    let client = GitlabBuilder::new(config.url, config.token).build_async().await?;
    let mut app = App::new(client);
    let _ = app.fetch_projects().await;
    run_with_terminal(&mut app).await
}

struct Config { url: String, token: String }
impl Config {
    fn from_env() -> Self {
        Self {
            url: std::env::var("GITLAB_URL").unwrap_or_else(|_| "gitlab.com".to_string()),
            token: std::env::var("GITLAB_TOKEN").expect("GITLAB_TOKEN must be set"),
        }
    }
}

pub async fn run_with_terminal(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, event::DisableMouseCapture)?;
    terminal.show_cursor()?;
    if let Err(err) = res { eprintln!("Application Error: {err}"); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[tokio::test]
    async fn test_run_app_quit() {
        use wiremock::{MockServer, Mock, ResponseTemplate};
        use wiremock::matchers::path;
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        let mut app = App::new(client);
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        app.should_quit = true;
        assert!(run_app(&mut terminal, &mut app).await.is_ok());
    }

    #[test]
    #[should_panic(expected = "GITLAB_TOKEN must be set")]
    fn test_config_from_env_panic() {
        unsafe { std::env::remove_var("GITLAB_TOKEN"); }
        Config::from_env();
    }
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn std::error::Error>> 
where 
    <B as Backend>::Error: 'static 
{
    while !app.should_quit {
        terminal.draw(|f| ui(f, app))?;

        let is_live_view = (app.view_mode == ViewMode::List && app.active_selection == MenuSelection::Pipelines) 
            || app.view_mode == ViewMode::JobLog
            || (app.view_mode == ViewMode::Detail && app.selected_pipeline.is_some());

        let refresh_interval = if is_live_view { Duration::from_secs(2) } else { Duration::from_secs(10) };

        if app.last_refresh.elapsed() >= refresh_interval && !app.is_loading {
            app.refresh().await;
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
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
