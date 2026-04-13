use ratatui::{prelude::*, widgets::*};
use crate::models::{App, InputTarget, DetailSection, MetadataField};
use crate::ui::markdown::render_markdown;
use crate::ui::widgets::{render_shortcuts, get_status_style, hex_to_color};

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
