pub mod markdown;
pub mod widgets;
pub mod list;
pub mod detail;
pub mod popup;

pub use widgets::Theme;
use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::models::{App, ViewMode};
use list::render_list_view;
use detail::render_detail_view;
use popup::*;
use ansi_to_tui::IntoText;
use crate::ui::widgets::{format_relative_time, format_time_only};

pub fn ui(f: &mut Frame, app: &mut App) {
    match app.view_mode {
        ViewMode::List => {
            render_list_view(f, app);
        }
        ViewMode::Detail => {
            render_detail_view(f, app);
        }
        ViewMode::JobLog => {
            render_job_log_view(f, app);
        }
    }

    if app.show_project_popup {
        render_project_popup(f, app);
    }
    if app.show_label_popup {
        render_label_popup(f, app);
    }
    if app.show_assignee_popup {
        render_assignee_popup(f, app);
    }
    if app.show_milestone_popup {
        render_milestone_popup(f, app);
    }
    if app.show_calendar_popup {
        render_calendar_popup(f, app);
    }
    if app.show_weight_popup {
        render_weight_popup(f, app);
    }
    if app.show_pipeline_popup {
        render_pipeline_popup(f, app);
    }
}

pub fn render_job_log_view(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

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

    let mut log_lines = Vec::new();
    let mut line_num = 1;
    for line in &app.job_log_lines {
        let is_collapsed = line.section.as_ref().map(|s| app.collapsed_sections.contains(s)).unwrap_or(false);
        
        // Skip lines that are inside a collapsed section, but keep the start line
        if is_collapsed && !line.is_section_start {
            line_num += 1;
            continue;
        }

        if line.is_section_end && line.content.trim().is_empty() {
             line_num += 1;
             continue;
        }

        let mut spans = vec![
            Span::styled(format!("{:4} ", line_num), Style::default().fg(app.theme.dim)),
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

        let content_text = line.content.as_bytes().into_text().unwrap_or_else(|_| Text::from(line.content.as_str()));
        for content_line in content_text.lines {
            let mut final_spans = spans.clone();
            final_spans.extend(content_line.spans);
            log_lines.push(Line::from(final_spans));
        }

        line_num += 1;
    }

    // Add animated dots if the job is running
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

    widgets::render_shortcuts(f, chunks[2], vec![("tab", "toggle section"), ("esc", "back"), ("o", "open"), ("q", "quit")], &app.theme);
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

        // Test base view modes
        for mode in [ViewMode::List, ViewMode::Detail, ViewMode::JobLog] {
            app.view_mode = mode;
            terminal.draw(|f| ui(f, &mut app)).unwrap();
        }

        // Test popups
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

        // Test input modes
        app.show_project_popup = false; // clear popups
        app.input_mode = true;
        for target in [InputTarget::PipelineRef, InputTarget::PipelineVariableKey, InputTarget::PipelineVariableValue, InputTarget::Reply] {
            app.input_target = target;
            app.show_pipeline_popup = true; // some inputs need pipeline popup context
            terminal.draw(|f| ui(f, &mut app)).unwrap();
            app.show_pipeline_popup = false;
            terminal.draw(|f| ui(f, &mut app)).unwrap();
        }
    }

    #[test]
    fn test_render_job_log_exhaustive() {
        let (_server, mut app) = tokio_test::block_on(setup_app());
        let backend = TestBackend::new(120, 60);
        let mut terminal = Terminal::new(backend).unwrap();

        app.view_mode = ViewMode::JobLog;

        // No job selected
        terminal.draw(|f| render_job_log_view(f, &mut app)).unwrap();

        // Job selected, various states
        let job_json = r#"{"id": 1, "name": "j", "stage": "s", "status": "running", "created_at": "2026-04-25T12:00:00Z", "started_at": "2026-04-25T12:00:00Z", "duration": null, "web_url": "u", "tag_list": ["t1", "t2"], "allow_failure": true, "user": {"id": 1, "username": "u", "name": "n", "state": "active", "avatar_url": "u", "web_url": "u"}, "runner": {"id": 1, "description": "runner", "active": true, "is_shared": true}}"#;
        app.selected_job = Some(serde_json::from_str(job_json).unwrap());
        
        // With log lines
        app.job_log_lines = vec![
            LogLine { timestamp: "2026-04-25T12:00:00Z".into(), content: "start".into(), section: Some("s1".into()), is_section_start: true, is_section_end: false },
            LogLine { timestamp: "".into(), content: "middle".into(), section: Some("s1".into()), is_section_start: false, is_section_end: false },
            LogLine { timestamp: "".into(), content: "".into(), section: Some("s1".into()), is_section_start: false, is_section_end: true },
        ];
        
        // Running state
        app.is_loading = false;
        terminal.draw(|f| render_job_log_view(f, &mut app)).unwrap();

        // Loading state
        app.is_loading = true;
        terminal.draw(|f| render_job_log_view(f, &mut app)).unwrap();

        // Finished state with duration
        app.is_loading = false;
        let job_finished_json = r#"{"id": 1, "name": "j", "stage": "s", "status": "success", "created_at": "2026-04-25T12:00:00Z", "started_at": "2026-04-25T12:00:00Z", "finished_at": "2026-04-25T12:01:00Z", "duration": 60.0, "web_url": "u", "tag_list": [], "allow_failure": false}"#;
        app.selected_job = Some(serde_json::from_str(job_finished_json).unwrap());
        
        // Collapsed section
        app.collapsed_sections.insert("s1".into());
        app.follow_log = false;
        terminal.draw(|f| render_job_log_view(f, &mut app)).unwrap();
    }
}
