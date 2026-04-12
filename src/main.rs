mod models;
mod app;
mod ui;

use std::{io, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::prelude::*;
use gitlab::GitlabBuilder;
use dotenvy::dotenv;
use models::*;
use ui::ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let url = std::env::var("GITLAB_URL").unwrap_or_else(|_| "gitlab.com".to_string());
    let token = std::env::var("GITLAB_TOKEN").expect("GITLAB_TOKEN must be set");

    let client = GitlabBuilder::new(url, token)
        .build_async()
        .await?;

    // --- SETUP ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(client);
    
    // Initial fetch
    let _ = app.fetch_projects().await;
    let _ = app.fetch_merge_requests().await;

    // Pass execution to the app loop
    let res = run_app(&mut terminal, &mut app).await;

    // --- RESTORE ---
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Application Error: {err}");
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn std::error::Error>> 
where 
    <B as Backend>::Error: 'static 
{
    while !app.should_quit {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.input_mode {
                        match key.code {
                            KeyCode::Enter => app.enter().await,
                            KeyCode::Esc => app.back(),
                            KeyCode::Char(c) => app.reply_input.push(c),
                            KeyCode::Backspace => { app.reply_input.pop(); }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => app.should_quit = true,
                            KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                            KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                            KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => app.toggle_focus(),
                            KeyCode::Char('r') => app.refresh().await,
                            KeyCode::Char('s') => { let _ = app.resolve_discussion().await; }
                            KeyCode::Char(' ') => app.toggle_discussion(),
                            KeyCode::Enter => app.enter().await,
                            KeyCode::Backspace | KeyCode::Esc => app.back(),
                            KeyCode::Char('v') => app.open_in_vscode(),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
