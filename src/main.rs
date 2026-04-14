mod models;
mod app;
mod ui;
mod api;

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
use chrono::Days;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let url = std::env::var("GITLAB_URL").unwrap_or_else(|_| "gitlab.com".to_string());
    let token = std::env::var("GITLAB_TOKEN").expect("GITLAB_TOKEN must be set");

    let client = GitlabBuilder::new(url, token)
        .build_async()
        .await?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(client);
    let _ = app.fetch_projects().await;

    let res = run_app(&mut terminal, &mut app).await;

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
                    if app.show_label_popup {
                        match key.code {
                            KeyCode::Char(' ') => {
                                if let Some(i) = app.label_state.selected() {
                                    let id = app.project_labels[i].id;
                                    if let Some(pos) = app.selected_label_ids.iter().position(|&x| x == id) {
                                        app.selected_label_ids.remove(pos);
                                    } else {
                                        app.selected_label_ids.push(id);
                                    }
                                }
                            }
                            KeyCode::Enter => app.enter().await,
                            KeyCode::Esc => app.back(),
                            KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                            KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                            _ => {}
                        }
                    } else if app.show_assignee_popup {
                        match key.code {
                            KeyCode::Char(' ') => {
                                if let Some(i) = app.assignee_state.selected() {
                                    let id = app.project_members[i].id;
                                    if let Some(pos) = app.selected_assignee_ids.iter().position(|&x| x == id) {
                                        app.selected_assignee_ids.remove(pos);
                                    } else {
                                        app.selected_assignee_ids.push(id);
                                    }
                                }
                            }
                            KeyCode::Enter => app.enter().await,
                            KeyCode::Esc => app.back(),
                            KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                            KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                            _ => {}
                        }
                    } else if app.show_milestone_popup || app.show_weight_popup {
                        match key.code {
                            KeyCode::Enter => app.enter().await,
                            KeyCode::Esc => app.back(),
                            KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                            KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                            _ => {}
                        }
                    } else if app.show_calendar_popup {
                        match key.code {
                            KeyCode::Left => { app.calendar_date = app.calendar_date.checked_sub_days(Days::new(1)).unwrap(); }
                            KeyCode::Right => { app.calendar_date = app.calendar_date.checked_add_days(Days::new(1)).unwrap(); }
                            KeyCode::Up => { app.calendar_date = app.calendar_date.checked_sub_days(Days::new(7)).unwrap(); }
                            KeyCode::Down => { app.calendar_date = app.calendar_date.checked_add_days(Days::new(7)).unwrap(); }
                            KeyCode::Enter => app.enter().await,
                            KeyCode::Delete | KeyCode::Backspace => { let _ = app.clear_due_date().await; }
                            KeyCode::Esc => app.back(),
                            _ => {}
                        }
                    } else if app.input_mode {
                        match key.code {
                            KeyCode::Enter if key.modifiers.contains(event::KeyModifiers::SHIFT) => {
                                if app.input_target == InputTarget::EditDescription || app.input_target == InputTarget::Reply || app.input_target == InputTarget::NewComment {
                                    let mut chars: Vec<char> = app.reply_input.chars().collect();
                                    chars.insert(app.cursor_index, '\n');
                                    app.reply_input = chars.into_iter().collect();
                                    app.cursor_index += 1;
                                }
                            }
                            KeyCode::Enter => { app.enter().await; }
                            KeyCode::Esc => app.back(),
                            KeyCode::Left => { app.cursor_index = app.cursor_index.saturating_sub(1); }
                            KeyCode::Right => { app.cursor_index = (app.cursor_index + 1).min(app.reply_input.chars().count()); }
                            KeyCode::Home => { app.cursor_index = 0; }
                            KeyCode::End => { app.cursor_index = app.reply_input.chars().count(); }
                            KeyCode::Char(c) => {
                                let mut chars: Vec<char> = app.reply_input.chars().collect();
                                chars.insert(app.cursor_index, c);
                                app.reply_input = chars.into_iter().collect();
                                app.cursor_index += 1;
                            }
                            KeyCode::Backspace => {
                                if app.cursor_index > 0 {
                                    let mut chars: Vec<char> = app.reply_input.chars().collect();
                                    chars.remove(app.cursor_index - 1);
                                    app.reply_input = chars.into_iter().collect();
                                    app.cursor_index -= 1;
                                }
                            }
                            KeyCode::Delete => {
                                let mut chars: Vec<char> = app.reply_input.chars().collect();
                                if app.cursor_index < chars.len() {
                                    chars.remove(app.cursor_index);
                                    app.reply_input = chars.into_iter().collect();
                                }
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => app.should_quit = true,
                            KeyCode::Char('r') if key.modifiers.contains(event::KeyModifiers::CONTROL) => app.switch_project(),
                            KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                            KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                            KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => app.toggle_focus(),
                            KeyCode::Char('r') => app.refresh().await,
                            KeyCode::Char('s') => { let _ = app.resolve_discussion().await; }
                            KeyCode::Char('c') => { let _ = app.toggle_issue_state().await; }
                            KeyCode::Char('n') => {
                                app.input_mode = true;
                                app.input_target = InputTarget::NewComment;
                                app.reply_input.clear();
                                app.cursor_index = 0;
                            }
                            KeyCode::Char(' ') => app.toggle_discussion(),
                            KeyCode::Enter => app.enter().await,
                            KeyCode::Backspace | KeyCode::Esc => app.back(),
                            KeyCode::Char('v') | KeyCode::Char('o') => app.open_in_browser(),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
