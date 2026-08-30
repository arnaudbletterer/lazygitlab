pub mod boards;
pub mod detail;
pub mod diff;
pub mod list;
pub mod markdown;
pub mod popup;
pub mod widgets;

use crate::models::{App, ViewMode};
use crate::ui::widgets::{format_relative_time, format_time_only};
use ansi_to_tui::IntoText;
use boards::{render_boards_view, render_epics_view};
use detail::render_detail_view;
use diff::render_diff_view;
use list::render_list_view;
use popup::*;
use ratatui::prelude::*;
use ratatui::widgets::*;
pub use widgets::Theme;

pub fn ui(f: &mut Frame, app: &mut App) {
    let job_log_searching =
        app.input_mode && app.input_target == crate::models::InputTarget::JobLogSearch;
    let has_filter = app.show_filter_input || !app.filter_input.is_empty() || job_log_searching;
    let footer_height = if has_filter { 2 } else { 1 };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(footer_height),
        ])
        .split(f.area());

    let mut header_spans = vec![
        Span::styled(
            " 🦊 GitLab ",
            Style::default()
                .bg(app.theme.accent)
                .fg(widgets::get_contrast_color(app.theme.accent))
                .add_modifier(Modifier::BOLD),
        ),
    ];
    if let Some(proj) = &app.selected_project {
        header_spans.push(Span::styled(" ❯ ", Style::default().fg(app.theme.dim)));
        header_spans.push(Span::styled(
            proj.path_with_namespace.as_deref().unwrap_or(&proj.name),
            Style::default()
                .fg(app.theme.project)
                .add_modifier(Modifier::BOLD),
        ));
        if let Some(ref_name) = &app.repository_ref {
            header_spans.push(Span::styled(" ❯ ", Style::default().fg(app.theme.dim)));
            header_spans.push(Span::styled("🌿 ", Style::default().fg(app.theme.accent)));
            header_spans.push(Span::styled(ref_name, Style::default().fg(app.theme.accent)));
        }
    } else {
        header_spans.push(Span::styled(" ❯ ", Style::default().fg(app.theme.dim)));
        header_spans.push(Span::styled(
            "Personal Dashboard",
            Style::default()
                .fg(app.theme.title_fg)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if app.is_loading {
        header_spans.push(Span::styled(
            "  ● LOADING",
            Style::default()
                .fg(app.theme.pending)
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        header_spans.push(Span::styled(
            "  ● LIVE",
            Style::default()
                .fg(app.theme.status_success)
                .add_modifier(Modifier::BOLD),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(header_spans)), main_chunks[0]);

    match app.view_mode {
        ViewMode::List => render_list_view(f, app, main_chunks[1]),
        ViewMode::Detail => render_detail_view(f, app, main_chunks[1]),
        ViewMode::JobLog => render_job_log_view(f, app, main_chunks[1]),
        ViewMode::Diff => render_diff_view(f, app, main_chunks[1]),
        ViewMode::Tree => render_tree_view(f, app, main_chunks[1]),
        ViewMode::Board => render_boards_view(f, app, main_chunks[1]),
        ViewMode::Epics => render_epics_view(f, app, main_chunks[1]),
    }

    let (filter_rect, footer_rect) = if has_filter {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(main_chunks[2]);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, main_chunks[2])
    };

    if let Some(rect) = filter_rect {
        let mut filter_spans = vec![
            Span::styled(" [ FILTER ] ", Style::default().bg(app.theme.highlight_bg).fg(widgets::get_contrast_color(app.theme.highlight_bg)).add_modifier(Modifier::BOLD)),
        ];
        if job_log_searching {
            filter_spans.push(Span::styled(" Search log: ", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)));
            filter_spans.push(Span::raw(&app.job_log_search));
        } else {
            filter_spans.push(Span::styled(" Filter: ", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)));
            filter_spans.push(Span::raw(&app.filter_input));
        }
        if job_log_searching || app.show_filter_input {
            filter_spans.push(Span::styled("█", Style::default().fg(app.theme.pending)));
        } else {
            filter_spans.push(Span::styled(" (Esc to clear)", Style::default().fg(app.theme.dim)));
        }
        f.render_widget(Paragraph::new(Line::from(filter_spans)), rect);
    }

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(30)])
        .split(footer_rect);

    let mode_str = match app.view_mode {
        ViewMode::List => " LIST ",
        ViewMode::Detail => " DETAIL ",
        ViewMode::JobLog => " JOB LOG ",
        ViewMode::Diff => " DIFF ",
        ViewMode::Tree => " TREE ",
        ViewMode::Board => " BOARD ",
        ViewMode::Epics => " EPICS ",
    };

    let mut footer_spans = vec![
        Span::styled(mode_str, Style::default().bg(app.theme.shortcut_key).fg(widgets::get_contrast_color(app.theme.shortcut_key)).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
    ];

    let count = match app.view_mode {
        ViewMode::List => match app.active_selection {
            crate::models::MenuSelection::MergeRequests => app.filtered_mrs.len(),
            crate::models::MenuSelection::Issues => app.filtered_issues.len(),
            crate::models::MenuSelection::Pipelines => app.filtered_pipelines.len(),
            crate::models::MenuSelection::Todos => app.filtered_todos.len(),
            crate::models::MenuSelection::Projects => app.filtered_projects.len(),
            crate::models::MenuSelection::Branches => app.filtered_branches.len(),
            _ => 0,
        },
        _ => 0,
    };
    if count > 0 {
        footer_spans.push(Span::styled(format!(" [{}] ", count), Style::default().bg(app.theme.chip_bg).fg(app.theme.chip_fg).bold()));
        footer_spans.push(Span::raw(" "));
    }

    footer_spans.push(Span::styled("?", Style::default().fg(app.theme.accent).bold()));
    footer_spans.push(Span::raw(" help  "));
    footer_spans.push(Span::styled("q", Style::default().fg(app.theme.accent).bold()));
    footer_spans.push(Span::raw(" quit"));

    if app.pending_g_prefix {
        footer_spans.push(Span::raw("   "));
        footer_spans.push(Span::styled(
            " g- [t:todos, m:mrs, i:issues, p:pipelines, r:repo, b:branches] ",
            Style::default()
                .bg(app.theme.accent)
                .fg(widgets::get_contrast_color(app.theme.accent))
                .add_modifier(Modifier::BOLD),
        ));
    }

    if let Some(err) = &app.last_error {
        footer_spans.push(Span::raw("   "));
        footer_spans.push(Span::styled(
            format!(" Error: {} ", err),
            Style::default().bg(app.theme.failed).fg(widgets::get_contrast_color(app.theme.failed)).add_modifier(Modifier::BOLD),
        ));
    }
    if app.is_loading || app.status_message.is_some() {
        let msg = app.status_message.as_deref().unwrap_or("Loading");
        let lower = msg.to_lowercase();
        let bg = if lower.contains("failed") || lower.contains("timed out") {
            app.theme.failed
        } else if lower.starts_with("copied")
            || lower.contains("cancelled")
            || lower.contains("canceled")
            || lower.contains("created")
            || lower.contains("merged")
            || lower.contains("launched")
            || lower.contains("approved")
            || lower.contains("resolved")
        {
            app.theme.success
        } else {
            app.theme.pending
        };
        footer_spans.push(Span::raw("   "));
        footer_spans.push(Span::styled(
            format!(" {} ", msg),
            Style::default().bg(bg).fg(widgets::get_contrast_color(bg)).add_modifier(Modifier::BOLD),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(footer_spans)), footer_chunks[0]);

    let version_info = env!("LAZYGITLAB_VERSION");
    f.render_widget(
        Paragraph::new(version_info)
            .alignment(Alignment::Right)
            .style(Style::default().fg(app.theme.dim)),
        footer_chunks[1],
    );

    // Contextual Popups
    if app.show_project_popup {
        render_project_popup(f, app);
    }
    if app.show_label_popup {
        render_label_popup(f, app);
    }
    if app.show_assignee_popup {
        render_assignee_popup(f, app);
    }
    if app.show_reviewer_popup {
        render_reviewer_popup(f, app);
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
    if app.show_target_branch_popup {
        render_target_branch_popup(f, app);
    }
    if app.show_tree_branch_popup {
        render_tree_branch_popup(f, app);
    }
    if app.show_pipeline_popup {
        render_pipeline_popup(f, app);
    }
    if app.show_schedules_popup {
        render_schedules_popup(f, app);
    }
    if app.show_ci_vars_popup {
        render_ci_vars_popup(f, app);
    }
    if app.show_create_issue_popup {
        render_create_issue_popup(f, app);
    }
    if app.show_create_mr_popup {
        render_create_mr_popup(f, app);
    }
    if app.show_create_branch_popup {
        render_create_branch_popup(f, app);
    }
    if app.show_snooze_popup {
        render_snooze_popup(f, app);
    }
    if app.show_search {
        render_search_popup(f, app);
    }
    if app.show_help_popup {
        render_help_popup(f, app);
    }
    if app.show_confirm_merge_popup {
        render_confirm_merge_popup(f, app);
    }
    if app.show_confirm_rebase_popup {
        render_confirm_rebase_popup(f, app);
    }
    if app.show_confirm_delete_branch_popup {
        render_confirm_delete_branch_popup(f, app);
    }

    let is_complex_popup_visible = app.show_pipeline_popup
        || app.show_schedules_popup
        || app.show_ci_vars_popup
        || app.show_create_issue_popup
        || app.show_create_mr_popup
        || app.show_create_branch_popup
        || app.show_search
        || app.show_snooze_popup
        || app.show_calendar_popup
        || app.show_help_popup
        || app.show_project_popup
        || app.show_label_popup
        || app.show_assignee_popup
        || app.show_reviewer_popup
        || app.show_milestone_popup
        || app.show_weight_popup
        || app.show_target_branch_popup
        || app.show_tree_branch_popup
        || app.show_confirm_merge_popup
        || app.show_confirm_rebase_popup
        || app.show_confirm_delete_branch_popup;

    if app.input_mode
        && !is_complex_popup_visible
        && !matches!(
            app.input_target,
            crate::models::InputTarget::Filter
                | crate::models::InputTarget::Search
                | crate::models::InputTarget::JobLogSearch
        )
    {
        let title = match app.input_target {
            crate::models::InputTarget::Reply => " Reply (Ctrl+Enter to send, Esc to cancel) ",
            crate::models::InputTarget::NewComment => {
                " New Comment (Ctrl+Enter to send, Esc to cancel) "
            }
            crate::models::InputTarget::NewFileComment => {
                " New File Comment (Ctrl+Enter to send, Esc to cancel) "
            }
            crate::models::InputTarget::EditTitle => " Edit Title (Enter to save, Esc to cancel) ",
            crate::models::InputTarget::EditDescription => {
                " Edit Description (Ctrl+Enter to save, Esc to cancel) "
            }
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
                status_line.push(Span::styled(
                    crate::ui::widgets::get_elapsed_time(started),
                    Style::default().fg(app.theme.pending),
                ));
            }
        } else if let Some(duration) = job.duration {
            status_line.push(Span::raw(" | Duration: "));
            status_line.push(Span::raw(crate::ui::widgets::format_duration(duration)));
        }

        status_line.push(Span::raw(" | Stage: "));
        status_line.push(Span::styled(&job.stage, Style::default().fg(app.theme.dim)));

        let mut header_lines = vec![Line::from(status_line)];

        let mut second_line = vec![
            Span::raw("Created: "),
            Span::raw(format_relative_time(&job.created_at)),
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
            third_line.push(Span::styled(
                format!("@{}", user.username),
                Style::default().fg(app.theme.author),
            ));
            third_line.push(Span::raw(" | "));
        }
        if let Some(runner) = &job.runner {
            third_line.push(Span::raw("Runner: "));
            third_line.push(Span::raw(
                runner.description.as_deref().unwrap_or("Unknown"),
            ));
            third_line.push(Span::raw(" | "));
        }
        if job.allow_failure {
            third_line.push(Span::styled(
                "Allow Failure",
                Style::default().fg(app.theme.pending),
            ));
        }
        header_lines.push(Line::from(third_line));

        if !job.tag_list.is_empty() {
            header_lines.push(Line::from(vec![
                Span::raw("Tags: "),
                Span::raw(job.tag_list.join(", ")),
            ]));
        }

        header_lines.push(Line::from(vec![
            Span::raw("URL: "),
            Span::raw(&job.web_url),
        ]));

        let title = if app.is_loading {
            " Job Info (Refreshing...) "
        } else {
            " Job Info [LIVE] "
        };
        let header = Paragraph::new(header_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(if app.is_loading {
                    app.theme.pending
                } else {
                    app.theme.success
                })),
        );
        f.render_widget(header, chunks[0]);
    } else {
        let header = Paragraph::new("No job selected").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Job Info ")
                .border_style(Style::default().fg(app.theme.border)),
        );
        f.render_widget(header, chunks[0]);
    }

    app.set_rect("job_log", chunks[1]);

    let inner_height = chunks[1].height.saturating_sub(2);
    app.job_log_viewport_height = inner_height;

    let is_visible = |app: &App, line: &crate::models::LogLine| -> bool {
        let is_collapsed = line
            .section
            .as_ref()
            .map(|s| app.collapsed_sections.contains(s))
            .unwrap_or(false);
        if is_collapsed && !line.is_section_start {
            return false;
        }
        if line.is_section_end && line.content.trim().is_empty() {
            return false;
        }
        true
    };

    let epoch = app.job_log_epoch;
    if app.job_log_display_cache.0 != epoch {
        let count = app
            .job_log_lines
            .iter()
            .filter(|l| is_visible(app, l))
            .count();
        app.job_log_display_cache = (epoch, count);
    }
    let display_count = app.job_log_display_cache.1;

    if app.follow_log {
        app.job_log_scroll = (display_count as u16).saturating_sub(inner_height);
    }

    let scroll = app.job_log_scroll as usize;
    let cursor_line_idx = app.job_log_cursor;
    let selection_range = app.job_log_selection_start.map(|start| {
        if start <= app.job_log_cursor {
            (start, app.job_log_cursor)
        } else {
            (app.job_log_cursor, start)
        }
    });

    let mut log_lines = Vec::new();
    let mut display_idx = 0usize;
    for (idx, line) in app.job_log_lines.iter().enumerate() {
        if !is_visible(app, line) {
            continue;
        }
        if display_idx < scroll {
            display_idx += 1;
            continue;
        }
        if log_lines.len() >= inner_height as usize {
            break;
        }

        let mut line_style = Style::default();
        let is_selected = selection_range
            .map(|(s, e)| idx >= s && idx <= e)
            .unwrap_or(false);
        if is_selected {
            line_style = line_style.bg(app.theme.highlight_bg);
        }
        if idx == cursor_line_idx {
            line_style = line_style.add_modifier(Modifier::UNDERLINED);
        }

        let num_style = if is_selected {
            Style::default().fg(app.theme.dim).bg(app.theme.highlight_bg)
        } else {
            Style::default().fg(app.theme.dim)
        };
        let mut spans = vec![Span::styled(
            format!("{:4} ", idx + 1),
            num_style,
        )];

        if !line.timestamp.is_empty() {
            let ts_style = if is_selected {
                Style::default().fg(app.theme.dim).bg(app.theme.highlight_bg)
            } else {
                Style::default().fg(app.theme.dim)
            };
            spans.push(Span::styled(
                format!("{} ", format_time_only(&line.timestamp)),
                ts_style,
            ));
        }

        if line.is_section_start {
            let symbol = if line
                .section
                .as_ref()
                .map(|s| app.collapsed_sections.contains(s))
                .unwrap_or(false)
            {
                "[+] "
            } else {
                "[-] "
            };
            let sym_style = if is_selected {
                Style::default()
                    .fg(app.theme.shortcut_key)
                    .bg(app.theme.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(app.theme.shortcut_key)
                    .add_modifier(Modifier::BOLD)
            };
            spans.push(Span::styled(
                symbol,
                sym_style,
            ));
        } else if line.section.is_some() {
            let indent_style = if is_selected {
                Style::default().bg(app.theme.highlight_bg)
            } else {
                Style::default()
            };
            spans.push(Span::styled("    ", indent_style));
        }

        let content_text = line
            .content
            .as_bytes()
            .into_text()
            .unwrap_or_else(|_| Text::from(line.content.as_str()));
        for content_line in content_text.lines {
            let mut final_spans = spans.clone();
            for mut s in content_line.spans {
                s.style = s.style.patch(line_style);
                final_spans.push(s);
            }
            log_lines.push(Line::from(final_spans));
        }
        display_idx += 1;
    }

    if app
        .selected_job
        .as_ref()
        .is_some_and(|j| j.status == "running")
        && display_count <= scroll + (inner_height as usize)
    {
        let dots = match (chrono::Utc::now().timestamp_millis() / 500) % 4 {
            1 => ".",
            2 => "..",
            3 => "...",
            _ => "",
        };
        log_lines.push(Line::from(vec![
            Span::styled("     ", Style::default().fg(app.theme.dim)),
            Span::styled(dots, Style::default().fg(app.theme.pending)),
        ]));
    }

    let log_title = if app.follow_log {
        " Log Output [FOLLOWING] "
    } else {
        " Log Output "
    };
    let log = Paragraph::new(log_lines)
        .block(Block::default().borders(Borders::ALL).title(log_title))
        .scroll((0, 0));
    f.render_widget(log, chunks[1]);
}

fn render_tree_view(f: &mut Frame, app: &mut App, area: Rect) {
    app.file_viewport_height = area.height.saturating_sub(2);
    let ref_str = app.repository_ref.as_deref().unwrap_or("default");
    let file_path = app
        .selected_file_content
        .as_ref()
        .map(|f| f.file_path.as_str())
        .unwrap_or("");
    let block = Block::default()
        .title(Span::styled(
            format!(" 📄 File: {} (🌿 {}) ", file_path, ref_str),
            Style::default().fg(app.theme.title_fg).bold(),
        ))
        .title_bottom(Line::from(vec![
            Span::styled(" b ", Style::default().fg(app.theme.author).bold()),
            Span::styled("switch branch  ", Style::default().fg(app.theme.dim)),
            Span::styled("y ", Style::default().fg(app.theme.author).bold()),
            Span::styled("copy content  ", Style::default().fg(app.theme.dim)),
            Span::styled("Esc/h ", Style::default().fg(app.theme.author).bold()),
            Span::styled("back to tree ", Style::default().fg(app.theme.dim)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.active_border));

    if let Some(file) = &app.selected_file_content {
        let cached_hit = app
            .file_highlight_cache
            .as_ref()
            .map(|(path, blob, _)| path == &file.file_path && blob == &file.blob_id)
            .unwrap_or(false);
        if !cached_hit {
            use base64::{Engine as _, engine::general_purpose};
            let clean_content = file.content.replace("\n", "").replace("\r", "");
            if let Ok(decoded) = general_purpose::STANDARD.decode(&clean_content)
                && let Ok(text) = String::from_utf8(decoded)
            {
                let lines = highlight_and_render(
                    &app.syntax_set,
                    &app.theme_set,
                    file.file_path.as_str(),
                    &text,
                    app.theme.dim,
                );
                app.file_highlight_cache =
                    Some((file.file_path.clone(), file.blob_id.clone(), lines));
            }
        }
        match &app.file_highlight_cache {
            Some((_, _, lines)) => {
                let p = Paragraph::new(lines.clone())
                    .block(block)
                    .scroll((app.file_scroll, 0));
                f.render_widget(p, area);
            }
            None => {
                let p = Paragraph::new("Binary or non-UTF8 content").block(block);
                f.render_widget(p, area);
            }
        }
    } else {
        let p = Paragraph::new("Loading...").block(block);
        f.render_widget(p, area);
    }
}

fn highlight_and_render(
    syntax_set: &syntect::parsing::SyntaxSet,
    theme_set: &syntect::highlighting::ThemeSet,
    file_path: &str,
    text: &str,
    line_number_color: ratatui::style::Color,
) -> Vec<Line<'static>> {
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let syntax = syntax_set
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text())
        .clone();
    let theme = theme_set.themes.values().next().cloned().unwrap();
    let mut highlighter = syntect::easy::HighlightLines::new(&syntax, &theme);
    let mut out = Vec::new();
    let mut line_number = 1;
    for line in syntect::util::LinesWithEndings::from(text) {
        let ranges = highlighter
            .highlight_line(line, syntax_set)
            .unwrap_or_default();
        let mut spans = vec![Span::styled(
            format!("{:4} ", line_number),
            Style::default().fg(line_number_color),
        )];
        for (style, content) in ranges {
            let fg = syntect_color_to_ratatui(style.foreground);
            spans.push(Span::styled(content.to_string(), Style::default().fg(fg)));
        }
        if spans.len() == 1 {
            spans.push(Span::raw(line.trim_end_matches(['\n', '\r']).to_string()));
        }
        out.push(Line::from(spans));
        line_number += 1;
    }
    if out.is_empty() {
        for (i, line) in text.lines().enumerate() {
            out.push(Line::from(vec![
                Span::styled(
                    format!("{:4} ", i + 1),
                    Style::default().fg(line_number_color),
                ),
                Span::raw(line.to_string()),
            ]));
        }
    }
    out
}

fn syntect_color_to_ratatui(c: syntect::highlighting::Color) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(c.r, c.g, c.b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use wiremock::matchers::path;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_app() -> (MockServer, App) {
        use gitlab::GitlabBuilder;
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
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        (server, App::new(client, tx, None))
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
            |a: &mut App| a.show_reviewer_popup = true,
            |a: &mut App| a.show_milestone_popup = true,
            |a: &mut App| a.show_weight_popup = true,
            |a: &mut App| a.show_pipeline_popup = true,
            |a: &mut App| a.show_calendar_popup = true,
            |a: &mut App| a.show_confirm_merge_popup = true,
            |a: &mut App| a.show_confirm_rebase_popup = true,
        ];

        for setup in popups {
            app.show_project_popup = false;
            app.show_label_popup = false;
            app.show_assignee_popup = false;
            app.show_reviewer_popup = false;
            app.show_milestone_popup = false;
            app.show_weight_popup = false;
            app.show_pipeline_popup = false;
            app.show_calendar_popup = false;
            app.show_confirm_merge_popup = false;
            app.show_confirm_rebase_popup = false;
            setup(&mut app);
            terminal.draw(|f| ui(f, &mut app)).unwrap();
        }
    }

    #[test]
    fn test_render_discussions_styling_and_excerpt() {
        let (_server, mut app) = tokio_test::block_on(setup_app());
        let backend = TestBackend::new(120, 60);
        let mut terminal = Terminal::new(backend).unwrap();

        let disc1 = Discussion {
            id: "1".into(),
            notes: vec![Note {
                body: "Unresolved comment".into(),
                author: Author::new(1, "alice".into()),
                system: false,
                position: Some(crate::models::Position {
                    new_path: Some("src/main.rs".into()),
                    old_path: None,
                    new_line: Some(10),
                    old_line: None,
                    line_range: None,
                }),
                context: Some(vec![
                    (9, "fn main() {".into(), LineType::Unchanged, false),
                    (
                        10,
                        "    println!(\"Hello\");".into(),
                        LineType::Addition,
                        true,
                    ),
                    (11, "}".into(), LineType::Unchanged, false),
                ]),
            }],
            individual_note: false,
            collapsed: false,
            resolved: Some(false),
        };

        let disc2 = Discussion {
            id: "2".into(),
            notes: vec![Note {
                body: "Resolved comment".into(),
                author: Author::new(2, "bob".into()),
                system: false,
                position: None,
                context: None,
            }],
            individual_note: false,
            collapsed: false,
            resolved: Some(true),
        };

        let disc3 = Discussion {
            id: "3".into(),
            notes: vec![Note {
                body: "Activity note".into(),
                author: Author::new(3, "system".into()),
                system: true,
                position: None,
                context: None,
            }],
            individual_note: false,
            collapsed: false,
            resolved: None,
        };

        app.discussions = vec![disc1, disc2, disc3];
        app.discussion_state.select(Some(0));
        app.view_mode = ViewMode::Detail;
        app.detail_section = DetailSection::Discussions;

        // Render UI with first discussion (has context/excerpt, unresolved)
        terminal.draw(|f| ui(f, &mut app)).unwrap();

        // Render UI with second discussion (resolved)
        app.discussion_state.select(Some(1));
        terminal.draw(|f| ui(f, &mut app)).unwrap();

        // Render UI with third discussion (activity)
        app.discussion_state.select(Some(2));
        terminal.draw(|f| ui(f, &mut app)).unwrap();
    }
}
