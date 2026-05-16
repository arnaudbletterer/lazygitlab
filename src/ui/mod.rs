pub mod markdown;
pub mod widgets;
pub mod list;
pub mod detail;
pub mod popup;
pub mod diff;
pub mod boards;

pub use widgets::Theme;
use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::models::{App, ViewMode};
use list::render_list_view;
use detail::render_detail_view;
use diff::render_diff_view;
use boards::{render_boards_view, render_epics_view};
use popup::*;
use ansi_to_tui::IntoText;
use crate::ui::widgets::{format_relative_time, format_time_only};

pub fn ui(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    match app.view_mode {
        ViewMode::List => render_list_view(f, app, main_chunks[0]),
        ViewMode::Detail => render_detail_view(f, app, main_chunks[0]),
        ViewMode::JobLog => render_job_log_view(f, app, main_chunks[0]),
        ViewMode::Diff => render_diff_view(f, app, main_chunks[0]),
        ViewMode::Tree => render_tree_view(f, app, main_chunks[0]),
        ViewMode::Board => render_boards_view(f, app, main_chunks[0]),
        ViewMode::Epics => render_epics_view(f, app, main_chunks[0]),
    }

    // Footer with left shortcuts and right version info
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(30)])
        .split(main_chunks[1]);

    let mut footer_spans = vec![
        Span::styled("?", Style::default().fg(app.theme.shortcut_key)),
        Span::raw(" help  "),
        Span::styled("q", Style::default().fg(app.theme.shortcut_key)),
        Span::raw(" quit"),
    ];

    if let Some(err) = &app.last_error {
        footer_spans.push(Span::raw("   "));
        footer_spans.push(Span::styled(format!(" Error: {} ", err), Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)));
    }
    if app.is_loading || app.status_message.is_some() {
        let msg = app.status_message.as_deref().unwrap_or("Loading");
        footer_spans.push(Span::raw("   "));
        footer_spans.push(Span::styled(format!(" {} ", msg), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    }

    f.render_widget(Paragraph::new(Line::from(footer_spans)), footer_chunks[0]);

    // Bottom right version info
    let version_info = format!("v{} ({}{})", env!("CARGO_PKG_VERSION"), env!("GIT_HASH"), env!("GIT_DIRTY"));
    f.render_widget(Paragraph::new(version_info).alignment(Alignment::Right).style(Style::default().fg(app.theme.dim)), footer_chunks[1]);

    // Contextual Popups
    if app.show_project_popup { render_project_popup(f, app); }
    if app.show_label_popup { render_label_popup(f, app); }
    if app.show_assignee_popup { render_assignee_popup(f, app); }
    if app.show_milestone_popup { render_milestone_popup(f, app); }
    if app.show_calendar_popup { render_calendar_popup(f, app); }
    if app.show_weight_popup { render_weight_popup(f, app); }
    if app.show_pipeline_popup { render_pipeline_popup(f, app); }
    if app.show_create_issue_popup { render_create_issue_popup(f, app); }
    if app.show_create_mr_popup { render_create_mr_popup(f, app); }
    if app.show_snooze_popup { render_snooze_popup(f, app); }
    if app.show_search { render_search_popup(f, app); }
    if app.show_help_popup { render_help_popup(f, app); }

    let is_complex_popup_visible = app.show_pipeline_popup || 
                                   app.show_create_issue_popup || 
                                   app.show_create_mr_popup || 
                                   app.show_search || 
                                   app.show_snooze_popup || 
                                   app.show_calendar_popup ||
                                   app.show_help_popup ||
                                   app.show_project_popup ||
                                   app.show_label_popup ||
                                   app.show_assignee_popup ||
                                   app.show_milestone_popup ||
                                   app.show_weight_popup;

    if app.input_mode && !is_complex_popup_visible && !matches!(app.input_target, crate::models::InputTarget::Filter | crate::models::InputTarget::Search) {
        let title = match app.input_target {
            crate::models::InputTarget::Reply => " Reply (Ctrl+Enter to send, Esc to cancel) ",
            crate::models::InputTarget::NewComment => " New Comment (Ctrl+Enter to send, Esc to cancel) ",
            crate::models::InputTarget::EditTitle => " Edit Title (Enter to save, Esc to cancel) ",
            crate::models::InputTarget::EditDescription => " Edit Description (Ctrl+Enter to save, Esc to cancel) ",
            crate::models::InputTarget::SnoozeUntil => " Snooze Until (Enter to confirm) ",
            _ => " Input (Ctrl+Enter to save, Esc to cancel) ",
        };
        let area = widgets::centered_rect(70, 40, f.area());
        f.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(app.theme.border));
        app.textarea.set_block(block);
        f.render_widget(&app.textarea, area);
    }
}



pub fn render_job_log_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    if let Some(job) = &app.selected_job {
        let status_style = widgets::get_status_style(&job.status, &app.theme);
        
        let mut status_line = vec![
            Span::raw("Job: "), 
            Span::styled(format!("#{} ", job.id), Style::default().fg(app.theme.dim)),
            Span::styled(&job.name, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" | Status: "),
            Span::styled(&job.status, status_style),
        ];

        if job.status == "running" {
            if let Some(started) = &job.started_at {
                status_line.push(Span::raw(" | Elapsed: "));
                status_line.push(Span::styled(crate::ui::widgets::get_elapsed_time(started), Style::default().fg(Color::Yellow)));
            }
        } else if let Some(duration) = job.duration {
            status_line.push(Span::raw(" | Duration: "));
            status_line.push(Span::raw(crate::ui::widgets::format_duration(duration)));
        }

        status_line.push(Span::raw(" | Stage: "));
        status_line.push(Span::styled(&job.stage, Style::default().fg(app.theme.dim)));

        let mut header_lines = vec![Line::from(status_line)];

        let mut second_line = vec![
            Span::raw("Created: "), Span::raw(format_relative_time(&job.created_at)),
        ];
        if let Some(started) = &job.started_at {
            second_line.push(Span::raw(" | Started: "));
            second_line.push(Span::raw(format_relative_time(started)));
        }
        if let Some(finished) = &job.finished_at {
            second_line.push(Span::raw(" | Finished: "));
            second_line.push(Span::raw(format_relative_time(finished)));
        }
        header_lines.push(Line::from(second_line));

        let mut third_line = vec![];
        if let Some(duration) = job.duration {
            third_line.push(Span::raw(format!("Duration: {:.2}s", duration)));
            third_line.push(Span::raw(" | "));
        }
        if let Some(user) = &job.user {
            third_line.push(Span::raw("User: "));
            third_line.push(Span::styled(format!("@{}", user.username), Style::default().fg(app.theme.author)));
            third_line.push(Span::raw(" | "));
        }
        if let Some(runner) = &job.runner {
            third_line.push(Span::raw("Runner: "));
            third_line.push(Span::raw(runner.description.as_deref().unwrap_or("Unknown")));
            third_line.push(Span::raw(" | "));
        }
        if job.allow_failure {
            third_line.push(Span::styled("Allow Failure", Style::default().fg(Color::Yellow)));
        }
        header_lines.push(Line::from(third_line));

        if !job.tag_list.is_empty() {
            header_lines.push(Line::from(vec![
                Span::raw("Tags: "),
                Span::raw(job.tag_list.join(", ")),
            ]));
        }

        header_lines.push(Line::from(vec![Span::raw("URL: "), Span::raw(&job.web_url)]));

        let title = if app.is_loading { " Job Info (Refreshing...) " } else { " Job Info [LIVE] " };
        let header = Paragraph::new(header_lines)
            .block(Block::default().borders(Borders::ALL).title(title).border_style(Style::default().fg(if app.is_loading { Color::Yellow } else { Color::Green })));
        f.render_widget(header, chunks[0]);
    } else {
        let header = Paragraph::new("No job selected")
            .block(Block::default().borders(Borders::ALL).title(" Job Info ").border_style(Style::default().fg(app.theme.border)));
        f.render_widget(header, chunks[0]);
    }

    app.set_rect("job_log", chunks[1]);

    let mut log_lines = Vec::new();
    for (idx, line) in app.job_log_lines.iter().enumerate() {
        let is_collapsed = line.section.as_ref().map(|s| app.collapsed_sections.contains(s)).unwrap_or(false);
        
        if is_collapsed && !line.is_section_start {
            continue;
        }

        if line.is_section_end && line.content.trim().is_empty() {
             continue;
        }

        let mut spans = vec![
            Span::styled(format!("{:4} ", idx + 1), Style::default().fg(app.theme.dim)),
        ];

        if !line.timestamp.is_empty() {
            spans.push(Span::styled(format!("{} ", format_time_only(&line.timestamp)), Style::default().fg(app.theme.dim)));
        }

        if line.is_section_start {
            let symbol = if is_collapsed { "[+] " } else { "[-] " };
            spans.push(Span::styled(symbol, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
        } else if line.section.is_some() {
            spans.push(Span::raw("    "));
        }

        let mut line_style = Style::default();
        let is_selected = if let Some(start) = app.job_log_selection_start {
            let (s, e) = if start <= app.job_log_cursor { (start, app.job_log_cursor) } else { (app.job_log_cursor, start) };
            idx >= s && idx <= e
        } else {
            false
        };

        if is_selected {
            line_style = line_style.bg(app.theme.highlight_bg);
        }
        if idx == app.job_log_cursor {
            line_style = line_style.add_modifier(Modifier::UNDERLINED);
        }

        let content_text = line.content.as_bytes().into_text().unwrap_or_else(|_| Text::from(line.content.as_str()));
        for content_line in content_text.lines {
            let mut final_spans = spans.clone();
            for mut s in content_line.spans {
                s.style = s.style.patch(line_style);
                final_spans.push(s);
            }
            log_lines.push(Line::from(final_spans));
        }
    }

    if let Some(job) = &app.selected_job {
        if job.status == "running" {
            let dots = match (chrono::Utc::now().timestamp_millis() / 500) % 4 {
                1 => ".",
                2 => "..",
                3 => "...",
                _ => "",
            };
            log_lines.push(Line::from(vec![
                Span::styled("     ", Style::default().fg(app.theme.dim)),
                Span::styled(dots, Style::default().fg(Color::Yellow)),
            ]));
        }
    }

    let inner_height = chunks[1].height.saturating_sub(2);
    app.job_log_viewport_height = inner_height;

    if app.follow_log {
        app.job_log_scroll = (log_lines.len() as u16).saturating_sub(inner_height);
    }

    let log_title = if app.follow_log { " Log Output [FOLLOWING] " } else { " Log Output " };
    let log = Paragraph::new(log_lines)
        .block(Block::default().borders(Borders::ALL).title(log_title))
        .scroll((app.job_log_scroll, 0));
    f.render_widget(log, chunks[1]);
}

fn render_tree_view(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(format!(" File Content: {} ", app.selected_file_content.as_ref().map(|f| f.file_path.as_str()).unwrap_or("")))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border));
        
    if let Some(file) = &app.selected_file_content {
        use base64::{Engine as _, engine::general_purpose};
        let clean_content = file.content.replace("\n", "").replace("\r", "");
        if let Ok(decoded) = general_purpose::STANDARD.decode(&clean_content) {
            if let Ok(text) = String::from_utf8(decoded) {
                let mut lines = Vec::new();
                for (i, line) in text.lines().enumerate() {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{:4} ", i + 1), Style::default().fg(app.theme.dim)),
                        Span::raw(line),
                    ]));
                }
                let p = Paragraph::new(lines).block(block).scroll((app.job_log_scroll, 0));
                f.render_widget(p, area);
            } else {
                let p = Paragraph::new("Binary or non-UTF8 content").block(block);
                f.render_widget(p, area);
            }
        } else {
            let p = Paragraph::new("Failed to decode base64").block(block);
            f.render_widget(p, area);
        }
    } else {
        let p = Paragraph::new("Loading...").block(block);
        f.render_widget(p, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::models::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::path;

    async fn setup_app() -> (MockServer, App) {
        use gitlab::GitlabBuilder;
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        (server, App::new(client))
    }

    #[test]
    fn test_render_main_ui_all_states() {
        let (_server, mut app) = tokio_test::block_on(setup_app());
        let backend = TestBackend::new(120, 60);
        let mut terminal = Terminal::new(backend).unwrap();

        for mode in [ViewMode::List, ViewMode::Detail, ViewMode::JobLog] {
            app.view_mode = mode;
            terminal.draw(|f| ui(f, &mut app)).unwrap();
        }

        let popups = [
            |a: &mut App| a.show_project_popup = true,
            |a: &mut App| a.show_label_popup = true,
            |a: &mut App| a.show_assignee_popup = true,
            |a: &mut App| a.show_milestone_popup = true,
            |a: &mut App| a.show_weight_popup = true,
            |a: &mut App| a.show_pipeline_popup = true,
            |a: &mut App| a.show_calendar_popup = true,
        ];

        for setup in popups {
            app.show_project_popup = false; app.show_label_popup = false; app.show_assignee_popup = false;
            app.show_milestone_popup = false; app.show_weight_popup = false; app.show_pipeline_popup = false;
            app.show_calendar_popup = false;
            setup(&mut app);
            terminal.draw(|f| ui(f, &mut app)).unwrap();
        }
    }
}
