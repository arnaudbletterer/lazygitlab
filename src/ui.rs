use ratatui::{prelude::*, widgets::*};
use crate::models::{App, ViewMode, MenuSelection, InputTarget, DetailSection, MetadataField};

pub fn render_markdown<'a>(text: &'a str, theme: &Theme) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let clean_text = text.replace("\r\n", "\n");
    for line in clean_text.split('\n') {
        let mut spans = Vec::new();
        let mut current_pos = 0;
        let line_chars: Vec<char> = line.chars().collect();

        if line_chars.is_empty() {
            lines.push(Line::from(""));
            continue;
        }

        while current_pos < line_chars.len() {
            if line_chars[current_pos..].starts_with(&['*', '*']) || line_chars[current_pos..].starts_with(&['_', '_']) {
                let marker = if line_chars[current_pos] == '*' { &['*', '*'] } else { &['_', '_'] };
                let start = current_pos + 2;
                let mut end = start;
                while end < line_chars.len() && !line_chars[end..].starts_with(marker) {
                    end += 1;
                }
                
                if end < line_chars.len() {
                    let content: String = line_chars[start..end].iter().collect();
                    spans.push(Span::styled(content, Style::default().add_modifier(Modifier::BOLD)));
                    current_pos = end + 2;
                } else {
                    spans.push(Span::raw(line_chars[current_pos].to_string()));
                    current_pos += 1;
                }
            } else if line_chars[current_pos] == '`' {
                let start = current_pos + 1;
                let mut end = start;
                while end < line_chars.len() && line_chars[end] != '`' {
                    end += 1;
                }
                
                if end < line_chars.len() {
                    let content: String = line_chars[start..end].iter().collect();
                    spans.push(Span::styled(content, Style::default().fg(theme.shortcut_key).bg(Color::Rgb(30, 30, 30))));
                    current_pos = end + 1;
                } else {
                    spans.push(Span::raw("`"));
                    current_pos += 1;
                }
            } else {
                let start = current_pos;
                let mut end = start;
                while end < line_chars.len() && line_chars[end] != '*' && line_chars[end] != '_' && line_chars[end] != '`' {
                    end += 1;
                }
                
                if start == end {
                    spans.push(Span::raw(line_chars[current_pos].to_string()));
                    current_pos += 1;
                } else {
                    let content: String = line_chars[start..end].iter().collect();
                    spans.push(Span::raw(content));
                    current_pos = end;
                }
            }
        }
        lines.push(Line::from(spans));
    }
    lines
}

pub struct Theme {
    pub border: Color,
    pub highlight_bg: Color,
    pub author: Color,
    pub dim: Color,
    pub opened: Color,
    pub merged: Color,
    pub closed: Color,
    pub shortcut_key: Color,
    pub shortcut_desc: Color,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            border: Color::Green,
            highlight_bg: Color::Blue,
            author: Color::Magenta,
            dim: Color::Rgb(100, 100, 100),
            opened: Color::Red,
            merged: Color::Blue,
            closed: Color::Rgb(150, 150, 150),
            shortcut_key: Color::Cyan,
            shortcut_desc: Color::White,
        }
    }
}

pub fn get_status_style(status: &str, theme: &Theme) -> Style {
    match status {
        "opened" | "open" => Style::default().fg(theme.opened),
        "merged" | "resolved" => Style::default().fg(theme.merged),
        "closed" => Style::default().fg(theme.closed),
        _ => Style::default(),
    }
}

fn render_shortcuts(f: &mut Frame, area: Rect, shortcuts: Vec<(&str, &str)>, theme: &Theme) {
    let mut spans = Vec::new();
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(", ", Style::default().fg(theme.shortcut_desc)));
        }
        spans.push(Span::styled(*key, Style::default().fg(theme.shortcut_key)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*desc, Style::default().fg(theme.shortcut_desc)));
    }
    let p = Paragraph::new(Line::from(spans));
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Color::Rgb(r, g, b)
    } else {
        Color::Gray
    }
}

pub fn ui(f: &mut Frame, app: &mut App) {
    match app.view_mode {
        ViewMode::List => {
            render_list_view(f, app);
        }
        ViewMode::Detail => {
            render_detail_view(f, app);
        }
    }

    if app.show_project_popup {
        render_project_popup(f, app);
    }
}

fn render_project_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);

    let projects: Vec<ListItem> = app.projects
        .iter()
        .map(|p| ListItem::new(format!("{} ({})", p.name, p.path_with_namespace)))
        .collect();

    let list = List::new(projects)
        .block(Block::default().borders(Borders::ALL).title(" Select Project ").border_style(Style::default().fg(app.theme.border)))
        .highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.projects_state);
}

fn render_list_view(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(chunks[0]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(main_chunks[0]);

    let proj_name = app.selected_project.as_ref().map(|p| p.name.as_str()).unwrap_or("None");
    let status_info = Paragraph::new(format!("Project: {}", proj_name))
        .block(Block::default().borders(Borders::ALL).title(" Status ").border_style(Style::default().fg(app.theme.border)));
    f.render_widget(status_info, left_chunks[0]);

    let items: Vec<ListItem> = app.menu_items.iter().map(|(name, _)| ListItem::new(name.as_str())).collect();
    let sidebar_style = if app.focus_on_menu { Style::default().fg(app.theme.border) } else { Style::default() };
    let sidebar_highlight_style = if app.focus_on_menu { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Menu ").border_style(sidebar_style))
        .highlight_style(sidebar_highlight_style)
        .highlight_symbol(if app.focus_on_menu { ">> " } else { "   " });
    f.render_stateful_widget(list, left_chunks[1], &mut app.menu_state);

    let content_style = if !app.focus_on_menu { Style::default().fg(app.theme.border) } else { Style::default() };
    let content_highlight_style = if !app.focus_on_menu { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() };

    if app.selected_project.is_none() {
        f.render_widget(Paragraph::new("\n\n  No project selected.\n  Press Ctrl+R to select a project.").block(Block::default().borders(Borders::ALL).title(" Content ").border_style(content_style)), main_chunks[1]);
    } else {
        match app.active_selection {
            MenuSelection::MergeRequests => {
                let mrs: Vec<ListItem> = app.merge_requests.iter().map(|mr| {
                    let state_style = get_status_style(&mr.state, &app.theme);
                    ListItem::from(Line::from(vec![
                        Span::styled(format!("[{}]", mr.state), state_style),
                        Span::raw(format!(" {} - {} by ", mr.references.full, mr.title)),
                        Span::styled(format!("@{}", mr.author.username), Style::default().fg(app.theme.author)),
                    ]))
                }).collect();
                f.render_stateful_widget(List::new(mrs).block(Block::default().borders(Borders::ALL).title(format!(" MRs for {} ", proj_name)).border_style(content_style)).highlight_style(content_highlight_style).highlight_symbol(if !app.focus_on_menu { "> " } else { "  " }), main_chunks[1], &mut app.mrs_state);
            }
            MenuSelection::Issues => {
                let issues: Vec<ListItem> = app.issues.iter().map(|issue| {
                    let state_style = get_status_style(&issue.state, &app.theme);
                    let mut spans = vec![
                        Span::styled(format!("[{}]", issue.state), state_style),
                        Span::styled(format!(" [{}]", issue.issue_type.as_deref().unwrap_or("issue")), Style::default().fg(app.theme.dim)),
                        Span::raw(format!(" #{} - {} by ", issue.iid, issue.title)),
                        Span::styled(format!("@{}", issue.author.username), Style::default().fg(app.theme.author)),
                    ];
                    for label in &issue.labels {
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(format!(" {} ", label.name), Style::default().bg(hex_to_color(&label.color)).fg(hex_to_color(&label.text_color)).add_modifier(Modifier::BOLD)));
                    }
                    ListItem::from(Line::from(spans))
                }).collect();
                f.render_stateful_widget(List::new(issues).block(Block::default().borders(Borders::ALL).title(format!(" Issues for {} ", proj_name)).border_style(content_style)).highlight_style(content_highlight_style).highlight_symbol(if !app.focus_on_menu { "> " } else { "  " }), main_chunks[1], &mut app.issues_state);
            }
            _ => { f.render_widget(Paragraph::new(format!("Content for {:?}", app.active_selection)).block(Block::default().borders(Borders::ALL).title(" Content ").border_style(content_style)), main_chunks[1]); }
        }
    }

    let shortcuts = vec![("j/k", "nav"), ("tab", "switch"), ("ctrl+r", "switch project"), ("enter", "select"), ("r", "refresh"), ("q", "quit")];
    render_shortcuts(f, chunks[1], shortcuts, &app.theme);
}

pub fn render_detail_view(f: &mut Frame, app: &mut App) {
    if let Some(mr) = &app.selected_mr {
        let chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(10), Constraint::Min(0), Constraint::Length(if app.input_mode { 5 } else { 0 }), Constraint::Length(1)]).split(f.area());
        let mut info_lines = vec![
            Line::from(vec![Span::raw("Title: "), Span::raw(&mr.title)]),
            Line::from(vec![Span::raw("Author: "), Span::styled(format!("@{}", mr.author.username), Style::default().fg(app.theme.author))]),
            Line::from(vec![Span::raw("State: "), Span::styled(&mr.state, get_status_style(&mr.state, &app.theme))]),
            Line::from(vec![Span::raw("URL: "), Span::raw(&mr.web_url)]),
            Line::from(""),
            Line::from(vec![Span::raw("Description: ")]),
        ];
        info_lines.extend(render_markdown(mr.description.as_deref().unwrap_or("No description provided."), &app.theme));
        f.render_widget(Paragraph::new(info_lines).block(Block::default().borders(Borders::ALL).title(format!(" MR: {} ", mr.references.full))).wrap(Wrap { trim: true }), chunks[0]);

        let comments: Vec<ListItem> = app.discussions.iter().enumerate().map(|(idx, d)| {
            let is_selected = app.discussion_state.selected() == Some(idx);
            let highlight = if is_selected && !app.input_mode { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() };
            let mut lines = Vec::new();
            if d.collapsed {
                if let Some(note) = d.notes.first() {
                    lines.push(Line::from(vec![Span::styled(if is_selected { "> " } else { "[+] " }, highlight), Span::styled(format!("@{}", note.author.username), highlight.fg(app.theme.author)), Span::styled(": ", highlight), Span::styled(note.body.chars().take(50).collect::<String>(), highlight)]));
                }
            } else {
                for note in &d.notes {
                    lines.push(Line::from(vec![Span::styled(format!("@{}", note.author.username), Style::default().fg(app.theme.author).add_modifier(Modifier::BOLD)), Span::raw(" says:")]));
                    lines.extend(render_markdown(&note.body, &app.theme).into_iter().map(|l| {
                        let mut spans = vec![Span::raw("  ")];
                        spans.extend(l.spans);
                        Line::from(spans)
                    }));
                    lines.push(Line::from(""));
                }
            }
            ListItem::new(lines)
        }).collect();
        f.render_stateful_widget(List::new(comments).block(Block::default().borders(Borders::ALL).title(" Discussions ")), chunks[1], &mut app.discussion_state);
        
        if app.input_mode {
            f.render_widget(Paragraph::new(app.reply_input.as_str()).block(Block::default().borders(Borders::ALL).title(" Input ")), chunks[2]);
        }
        render_shortcuts(f, chunks[3], vec![("j/k", "nav"), ("space", "expand"), ("s", "resolve"), ("enter", "reply"), ("esc", "back"), ("v", "vscode"), ("r", "refresh"), ("q", "quit")], &app.theme);
    } else if let Some(issue) = &app.selected_issue {
        let is_metadata_focused = app.detail_section == DetailSection::Metadata;
        let is_editing_title = app.input_mode && app.input_target == InputTarget::EditTitle;
        let is_editing_desc = app.input_mode && app.input_target == InputTarget::EditDescription;
        let metadata_height = if is_editing_desc || (is_metadata_focused && app.selected_metadata_field == MetadataField::Description) { 15.min(f.area().height - 10) } else { 10 };
        let chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(metadata_height), Constraint::Min(0), Constraint::Length(1)]).split(f.area());
        
        let metadata_block = Block::default().borders(Borders::ALL).title(format!(" Issue: #{} ", issue.iid)).border_style(if is_metadata_focused { Style::default().fg(app.theme.border) } else { Style::default() });
        let inner_area = metadata_block.inner(chunks[0]);
        f.render_widget(metadata_block, chunks[0]);

        let metadata_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(6), Constraint::Length(1), Constraint::Min(0)]).split(inner_area);
        
        let mut title_line = vec![Span::raw("Title: "), Span::styled(if is_editing_title { &app.reply_input } else { &issue.title }, if is_metadata_focused && app.selected_metadata_field == MetadataField::Title { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() })];
        for label in &issue.labels {
            title_line.push(Span::raw(" "));
            title_line.push(Span::styled(format!(" {} ", label.name), Style::default().bg(hex_to_color(&label.color)).fg(hex_to_color(&label.text_color)).add_modifier(Modifier::BOLD)));
        }

        let mut header_lines = vec![
            Line::from(title_line),
            Line::from(vec![Span::raw("Author: "), Span::styled(format!("@{}", issue.author.username), Style::default().fg(app.theme.author)), Span::raw(" | Assignees: "), Span::styled(if issue.assignees.is_empty() { "None".to_string() } else { issue.assignees.iter().map(|a| format!("@{}", a.username)).collect::<Vec<_>>().join(", ") }, Style::default().fg(app.theme.author))]),
            Line::from(vec![Span::raw("State: "), Span::styled(&issue.state, get_status_style(&issue.state, &app.theme)), Span::raw(" | Milestone: "), Span::styled(issue.milestone.as_ref().map(|m| m.title.as_str()).unwrap_or("None"), Style::default().fg(app.theme.merged)), Span::raw(" | Due Date: "), Span::styled(issue.due_date.as_deref().unwrap_or("None"), Style::default().fg(app.theme.opened))]),
            Line::from(vec![Span::raw("Weight: "), Span::raw(issue.weight.map(|w| w.to_string()).unwrap_or_else(|| "None".to_string())), Span::raw(" | Confidential: "), Span::raw(if issue.confidential { "Yes" } else { "No" }), Span::raw(" | Created: "), Span::styled(&issue.created_at, Style::default().fg(app.theme.dim))]),
            Line::from(vec![Span::raw("URL: "), Span::raw(&issue.web_url)]),
        ];
        if let Some(ts) = &issue.task_completion_status {
            if ts.count > 0 {
                header_lines.push(Line::from(vec![Span::raw("Tasks: "), Span::styled(format!("{} of {} completed", ts.completed_count, ts.count), Style::default().fg(app.theme.border))]));
            }
        }
        f.render_widget(Paragraph::new(header_lines), metadata_chunks[0]);
        f.render_widget(Paragraph::new(vec![Line::from(""), Line::from(vec![Span::raw("Description: ")])]), metadata_chunks[1]);

        let mut desc_lines = Vec::new();
        let is_desc_active = is_metadata_focused && app.selected_metadata_field == MetadataField::Description;
        let desc_style = if is_desc_active { Style::default().bg(app.theme.highlight_bg) } else { Style::default() };
        if is_editing_desc {
            for line in app.reply_input.split('\n') { desc_lines.push(Line::from(vec![Span::styled(line, desc_style.add_modifier(Modifier::BOLD))])); }
        } else {
            let markdown = render_markdown(issue.description.as_deref().unwrap_or("No description provided."), &app.theme);
            let limit = if is_desc_active { markdown.len() } else { 3 };
            for (i, mut line) in markdown.into_iter().enumerate() {
                if i >= limit && !is_desc_active { break; }
                if is_desc_active { line = line.patch_style(desc_style); }
                desc_lines.push(line);
            }
        }

        let current_line_idx = if is_editing_desc { app.reply_input.split('\n').count().saturating_sub(1) } else { 0 };
        let scroll = if current_line_idx >= metadata_chunks[2].height as usize { (current_line_idx - (metadata_chunks[2].height as usize) + 1) as u16 } else { 0 };
        f.render_widget(Paragraph::new(desc_lines).wrap(Wrap { trim: false }).scroll((scroll, 0)), metadata_chunks[2]);

        if app.input_mode {
            if is_editing_title { f.set_cursor_position((metadata_chunks[0].x + 7 + (app.reply_input.chars().count() as u16), metadata_chunks[0].y)); }
            else if is_editing_desc {
                let lines: Vec<&str> = app.reply_input.split('\n').collect();
                let visual_line = (lines.len() - 1) as i32 - (scroll as i32);
                if visual_line >= 0 && visual_line < metadata_chunks[2].height as i32 { f.set_cursor_position((metadata_chunks[2].x + (lines.last().unwrap_or(&"").chars().count() as u16), metadata_chunks[2].y + visual_line as u16)); }
            }
        }

        let comments: Vec<ListItem> = app.discussions.iter().enumerate().map(|(idx, d)| {
            let is_selected = !is_metadata_focused && app.discussion_state.selected() == Some(idx);
            let highlight = if is_selected && !app.input_mode { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() };
            let mut lines = Vec::new();
            if d.collapsed {
                if let Some(note) = d.notes.first() {
                    lines.push(Line::from(vec![Span::styled(if is_selected { "> " } else { "[+] " }, highlight), Span::styled(format!("@{}", note.author.username), highlight.fg(app.theme.author)), Span::styled(": ", highlight), Span::styled(note.body.chars().take(80).collect::<String>(), highlight)]));
                }
            } else {
                for note in &d.notes {
                    lines.push(Line::from(vec![Span::styled(format!("@{}", note.author.username), Style::default().fg(app.theme.author).add_modifier(Modifier::BOLD)), Span::raw(" says:")]));
                    lines.extend(render_markdown(&note.body, &app.theme).into_iter().map(|l| {
                        let mut spans = vec![Span::raw("  ")];
                        spans.extend(l.spans);
                        Line::from(spans)
                    }));
                    lines.push(Line::from(""));
                }
            }
            ListItem::new(lines)
        }).collect();
        f.render_stateful_widget(List::new(comments).block(Block::default().borders(Borders::ALL).title(" Discussions ").border_style(if !is_metadata_focused && !app.input_mode { Style::default().fg(app.theme.border) } else { Style::default() })), chunks[1], &mut app.discussion_state);
        render_shortcuts(f, chunks[2], vec![("j/k", "nav"), ("space", "expand"), ("c", "close/reopen"), ("n", "comment"), ("enter", "edit/save"), ("shift+enter", "newline"), ("esc", "back"), ("r", "refresh"), ("q", "quit")], &app.theme);
    }
}
