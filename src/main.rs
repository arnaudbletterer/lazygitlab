use std::{io, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::prelude::*;
use gitlab::GitlabBuilder;
use dotenvy::dotenv;
use lazygitlab::*;
use lazygitlab::ui::ui;
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

        // Auto-refresh for Pipelines and JobLog
        let is_live_view = (app.view_mode == ViewMode::List && app.active_selection == MenuSelection::Pipelines) 
            || app.view_mode == ViewMode::JobLog
            || (app.view_mode == ViewMode::Detail && app.selected_pipeline.is_some());

        let refresh_interval = if is_live_view { Duration::from_secs(2) } else { Duration::from_secs(10) };

        if app.last_refresh.elapsed() >= refresh_interval && !app.is_loading {
            app.refresh().await;
        }

        if event::poll(Duration::from_millis(100))? {

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.show_pipeline_popup {
                        if app.input_mode {
                            match key.code {
                                KeyCode::Enter => app.enter().await,
                                KeyCode::Esc => app.back().await,
                                KeyCode::Tab => {
                                    app.show_branches_dropdown = false;
                                    match app.input_target {
                                        InputTarget::PipelineRef => {
                                            app.input_target = InputTarget::PipelineVariableKey;
                                            app.cursor_index = app.pipeline_variable_key_input.chars().count();
                                        }
                                        InputTarget::PipelineVariableKey => {
                                            app.input_target = InputTarget::PipelineVariableValue;
                                            app.cursor_index = app.pipeline_variable_value_input.chars().count();
                                        }
                                        InputTarget::PipelineVariableValue => {
                                            app.input_target = InputTarget::PipelineVariableList;
                                            if !app.pipeline_variables.is_empty() && app.pipeline_variables_state.selected().is_none() {
                                                app.pipeline_variables_state.select(Some(0));
                                            }
                                        }
                                        InputTarget::PipelineVariableList => {
                                            app.input_target = InputTarget::PipelineRef;
                                            app.cursor_index = app.pipeline_ref_input.chars().count();
                                        }
                                        _ => {
                                            app.input_target = InputTarget::PipelineRef;
                                            app.cursor_index = app.pipeline_ref_input.chars().count();
                                        }
                                    }
                                }
                                KeyCode::Left => { app.cursor_index = app.cursor_index.saturating_sub(1); }
                                KeyCode::Right => {
                                    let len = app.get_input_buffer().chars().count();
                                    app.cursor_index = (app.cursor_index + 1).min(len);
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if app.input_target == InputTarget::PipelineRef {
                                        app.show_branches_dropdown = true;
                                        app.move_down();
                                    } else if app.input_target == InputTarget::PipelineVariableList {
                                        app.move_down();
                                    }
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if app.input_target == InputTarget::PipelineRef {
                                        app.show_branches_dropdown = true;
                                        app.move_up();
                                    } else if app.input_target == InputTarget::PipelineVariableList {
                                        app.move_up();
                                    }
                                }
                                KeyCode::Delete | KeyCode::Backspace if app.input_target == InputTarget::PipelineVariableList => {
                                    app.remove_pipeline_variable();
                                }
                                KeyCode::Char(c) => {
                                    let cursor = app.cursor_index;
                                    let input = app.get_input_buffer_mut();
                                    let mut chars: Vec<char> = input.chars().collect();
                                    chars.insert(cursor, c);
                                    *input = chars.into_iter().collect();
                                    app.cursor_index += 1;
                                    if app.input_target == InputTarget::PipelineRef {
                                        app.filter_branches();
                                        app.show_branches_dropdown = true;
                                    }
                                }
                                KeyCode::Backspace => {
                                    if app.cursor_index > 0 {
                                        let cursor = app.cursor_index;
                                        let input = app.get_input_buffer_mut();
                                        let mut chars: Vec<char> = input.chars().collect();
                                        chars.remove(cursor - 1);
                                        *input = chars.into_iter().collect();
                                        app.cursor_index -= 1;
                                        if app.input_target == InputTarget::PipelineRef {
                                            app.filter_branches();
                                            app.show_branches_dropdown = true;
                                        }
                                    }
                                }
                                KeyCode::Home => { app.cursor_index = 0; }
                                KeyCode::End => { app.cursor_index = app.get_input_buffer().chars().count(); }
                                KeyCode::Delete => {
                                    let cursor = app.cursor_index;
                                    let input = app.get_input_buffer_mut();
                                    let mut chars: Vec<char> = input.chars().collect();
                                    if cursor < chars.len() {
                                        chars.remove(cursor);
                                        *input = chars.into_iter().collect();
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            match key.code {
                                KeyCode::Enter => app.enter().await,
                                KeyCode::Esc => app.back().await,
                                KeyCode::Char('p') => app.submit_pipeline_launch().await,
                                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                                KeyCode::Delete | KeyCode::Backspace => app.remove_pipeline_variable(),
                                KeyCode::Char('i') | KeyCode::Tab => {
                                    app.input_mode = true;
                                    app.input_target = InputTarget::PipelineRef;
                                    app.cursor_index = app.pipeline_ref_input.chars().count();
                                }
                                _ => {}
                            }
                        }
                    } else if app.show_label_popup {
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
                            KeyCode::Esc => app.back().await,
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.move_down();
                                if app.view_mode == ViewMode::List && app.focus_on_menu {
                                    app.refresh().await;
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.move_up();
                                if app.view_mode == ViewMode::List && app.focus_on_menu {
                                    app.refresh().await;
                                }
                            }
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
                            KeyCode::Esc => app.back().await,
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.move_down();
                                if app.view_mode == ViewMode::List && app.focus_on_menu {
                                    app.refresh().await;
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.move_up();
                                if app.view_mode == ViewMode::List && app.focus_on_menu {
                                    app.refresh().await;
                                }
                            }
                            _ => {}
                        }
                    } else if app.show_milestone_popup || app.show_weight_popup {
                        match key.code {
                            KeyCode::Enter => app.enter().await,
                            KeyCode::Esc => app.back().await,
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.move_down();
                                if app.view_mode == ViewMode::List && app.focus_on_menu {
                                    app.refresh().await;
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.move_up();
                                if app.view_mode == ViewMode::List && app.focus_on_menu {
                                    app.refresh().await;
                                }
                            }
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
                            KeyCode::Esc => app.back().await,
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
                            KeyCode::Esc => app.back().await,
                            KeyCode::Left => { app.cursor_index = app.cursor_index.saturating_sub(1); }
                            KeyCode::Right => {
                                let len = app.get_input_buffer().chars().count();
                                app.cursor_index = (app.cursor_index + 1).min(len);
                            }
                            KeyCode::Home => { app.cursor_index = 0; }
                            KeyCode::End => { app.cursor_index = app.get_input_buffer().chars().count(); }
                            KeyCode::Char(c) => {
                                let cursor = app.cursor_index;
                                let input = app.get_input_buffer_mut();
                                let mut chars: Vec<char> = input.chars().collect();
                                chars.insert(cursor, c);
                                *input = chars.into_iter().collect();
                                app.cursor_index += 1;
                            }
                            KeyCode::Backspace => {
                                if app.cursor_index > 0 {
                                    let cursor = app.cursor_index;
                                    let input = app.get_input_buffer_mut();
                                    let mut chars: Vec<char> = input.chars().collect();
                                    chars.remove(cursor - 1);
                                    *input = chars.into_iter().collect();
                                    app.cursor_index -= 1;
                                }
                            }
                            KeyCode::Delete => {
                                let cursor = app.cursor_index;
                                let input = app.get_input_buffer_mut();
                                let mut chars: Vec<char> = input.chars().collect();
                                if cursor < chars.len() {
                                    chars.remove(cursor);
                                    *input = chars.into_iter().collect();
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
                            KeyCode::PageDown => app.page_down(),
                            KeyCode::PageUp => app.page_up(),
                            KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab if app.view_mode != ViewMode::JobLog => app.toggle_focus(),
                            KeyCode::Tab if app.view_mode == ViewMode::JobLog => app.toggle_job_section(),
                            KeyCode::Char('s') if app.view_mode == ViewMode::Detail && app.selected_mr.is_some() => { let _ = app.resolve_discussion().await; }
                            KeyCode::Char('c') if app.view_mode == ViewMode::Detail && app.selected_issue.is_some() => { let _ = app.toggle_issue_state().await; }
                            KeyCode::Char('n') if app.view_mode == ViewMode::Detail && (app.selected_issue.is_some() || app.selected_mr.is_some()) => {
                                app.input_mode = true;
                                app.input_target = InputTarget::NewComment;
                                app.reply_input.clear();
                                app.cursor_index = 0;
                            }
                            KeyCode::Char('p') if app.view_mode == ViewMode::List && app.active_selection == MenuSelection::Pipelines => {
                                app.show_pipeline_launch_popup().await;
                            }
                            KeyCode::Char('p') if app.view_mode == ViewMode::Detail && app.selected_pipeline.is_some() => {
                                if let Some(proj) = &app.selected_project {
                                    if let Some(i) = app.jobs_state.selected() {
                                        if let Some(job) = app.jobs.get(i) {
                                            if job.status == "manual" {
                                                let _ = app.play_job(proj.id, job.id).await;
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('r') if app.view_mode == ViewMode::Detail && app.selected_pipeline.is_some() => {
                                if let Some(proj) = &app.selected_project {
                                    if let Some(i) = app.jobs_state.selected() {
                                        if let Some(job) = app.jobs.get(i) {
                                            let _ = app.retry_job(proj.id, job.id).await;
                                        }
                                    } else if let Some(pipeline) = &app.selected_pipeline {
                                        let _ = app.retry_pipeline(proj.id, pipeline.id).await;
                                    }
                                }
                            }
                            KeyCode::Char('c') if app.view_mode == ViewMode::List && app.active_selection == MenuSelection::Pipelines => {
                                if let Some(proj) = &app.selected_project {
                                    if let Some(i) = app.pipelines_state.selected() {
                                        if let Some(pipeline) = app.pipelines.get(i) {
                                            let _ = app.cancel_pipeline(proj.id, pipeline.id).await;
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('c') if app.view_mode == ViewMode::Detail && app.selected_pipeline.is_some() => {
                                if let Some(proj) = &app.selected_project {
                                    if let Some(i) = app.jobs_state.selected() {
                                        if let Some(job) = app.jobs.get(i) {
                                            let _ = app.cancel_job(proj.id, job.id).await;
                                        }
                                    } else if let Some(pipeline) = &app.selected_pipeline {
                                        let _ = app.cancel_pipeline(proj.id, pipeline.id).await;
                                    }
                                }
                            }
                            KeyCode::Char(' ') => app.toggle_discussion(),
                            KeyCode::Enter => app.enter().await,
                            KeyCode::Backspace | KeyCode::Esc => app.back().await,
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
