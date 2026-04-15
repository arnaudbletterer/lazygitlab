use ratatui::{prelude::*, widgets::*};
use crate::models::{App, InputTarget, DetailSection, MetadataField};
use crate::ui::markdown::render_markdown;
use crate::ui::widgets::{render_shortcuts, get_status_style, hex_to_color, format_relative_time};

pub fn render_detail_view(f: &mut Frame, app: &mut App) {
    if let Some(mr) = &app.selected_mr {
        let is_metadata_focused = app.detail_section == DetailSection::Metadata;
        let is_editing_title = app.input_mode && app.input_target == InputTarget::EditTitle;
        let is_editing_desc = app.input_mode && app.input_target == InputTarget::EditDescription;
        let metadata_height = if is_editing_desc || (is_metadata_focused && app.selected_metadata_field == MetadataField::Description) { 15.min(f.area().height - 10) } else { 12 };
        let chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(metadata_height), Constraint::Min(0), Constraint::Length(if app.input_mode && !is_editing_title && !is_editing_desc { 5 } else { 0 }), Constraint::Length(1)]).split(f.area());

        let metadata_block = Block::default().borders(Borders::ALL).title(format!(" MR: {} ", mr.references.full)).border_style(if is_metadata_focused { Style::default().fg(app.theme.border) } else { Style::default() });
        let inner_area = metadata_block.inner(chunks[0]);
        f.render_widget(metadata_block, chunks[0]);

        let metadata_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(metadata_height.saturating_sub(2)), Constraint::Length(1), Constraint::Min(0)]).split(inner_area);

        let highlight = Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD);
        let field = |f| if is_metadata_focused && app.selected_metadata_field == f { highlight } else { Style::default() };

        let title_line = vec![Span::raw("Title: "), Span::styled(if is_editing_title { &app.reply_input } else { &mr.title }, field(MetadataField::Title))];
        
        let mut labels_line = vec![Span::styled("Labels: ", field(MetadataField::Labels))];
        for label in &mr.labels {
            labels_line.push(Span::raw(" "));
            labels_line.push(Span::styled(format!(" {} ", label.name), Style::default().bg(hex_to_color(&label.color)).fg(hex_to_color(&label.text_color)).add_modifier(Modifier::BOLD)));
        }

        let mut header_lines = vec![
            Line::from(title_line),
            Line::from(labels_line),
            Line::from(vec![
                Span::raw("Author: "), Span::styled(format!("@{}", mr.author.username), Style::default().fg(app.theme.author)), 
                Span::raw(" | Assignees: "), Span::styled(if mr.assignees.is_empty() { "None".to_string() } else { mr.assignees.iter().map(|a| format!("@{}", a.username)).collect::<Vec<_>>().join(", ") }, field(MetadataField::Assignees)),
            ]),
            Line::from(vec![
                Span::raw("State: "), Span::styled(&mr.state, get_status_style(&mr.state, &app.theme)), 
                Span::raw(" | Milestone: "), Span::styled(mr.milestone.as_ref().map(|m| m.title.as_str()).unwrap_or("None"), field(MetadataField::Milestone)), 
                Span::raw(" | Created: "), Span::styled(format_relative_time(&mr.created_at), Style::default().fg(app.theme.dim)),
            ]),
            Line::from(vec![Span::raw("URL: "), Span::raw(&mr.web_url)]),
        ];

        // Reactions line
        if !mr.reactions.is_empty() || mr.upvotes > 0 || mr.downvotes > 0 {
            let mut reaction_spans = vec![Span::raw("Reactions: ")];
            if mr.upvotes > 0 {
                reaction_spans.push(Span::styled(format!("👍 {} ", mr.upvotes), Style::default().fg(Color::Green)));
            }
            if mr.downvotes > 0 {
                reaction_spans.push(Span::styled(format!("👎 {} ", mr.downvotes), Style::default().fg(Color::Red)));
            }
            
            // Other reactions
            let mut counts = std::collections::HashMap::new();
            for r in &mr.reactions {
                let clean_name = r.name.trim_matches(':');
                if clean_name != "thumbsup" && clean_name != "thumbsdown" && clean_name != "+1" && clean_name != "-1" {
                    *counts.entry(clean_name).or_insert(0) += 1;
                }
            }
            let mut reaction_keys: Vec<_> = counts.keys().collect();
            reaction_keys.sort();
            for name in reaction_keys {
                let count = counts.get(name).unwrap();
                let emoji = if let Some(e) = emojis::get_by_shortcode(name) {
                    e.as_str()
                } else {
                    name
                };
                reaction_spans.push(Span::raw(format!("{} {} ", emoji, count)));
            }
            header_lines.push(Line::from(reaction_spans));
        }

        f.render_widget(Paragraph::new(header_lines).wrap(Wrap { trim: true }), metadata_chunks[0]);
        f.render_widget(Paragraph::new(vec![Line::from(""), Line::from(vec![Span::styled("Description: ", field(MetadataField::Description))])]), metadata_chunks[1]);

        let mut desc_lines = Vec::new();
        let is_desc_active = is_metadata_focused && app.selected_metadata_field == MetadataField::Description;
        let desc_style = if is_desc_active { Style::default().bg(app.theme.highlight_bg) } else { Style::default() };
        if is_editing_desc {
            for line in app.reply_input.split('\n') { desc_lines.push(Line::from(vec![Span::styled(line, desc_style.add_modifier(Modifier::BOLD))])); }
        } else {
            let markdown = render_markdown(mr.description.as_deref().unwrap_or("No description provided."), &app.theme);
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
            if is_editing_title { f.set_cursor_position((metadata_chunks[0].x + 7 + (app.cursor_index as u16), metadata_chunks[0].y)); }
            else if is_editing_desc {
                let chars: Vec<char> = app.reply_input.chars().collect();
                let before_cursor = &chars[..app.cursor_index.min(chars.len())];
                let s_before: String = before_cursor.iter().collect();
                let lines_before: Vec<&str> = s_before.split('\n').collect();
                let cursor_line = lines_before.len() - 1;
                let cursor_col = lines_before.last().unwrap_or(&"").chars().count();

                let visual_line = (cursor_line) as i32 - (scroll as i32);
                if visual_line >= 0 && visual_line < metadata_chunks[2].height as i32 { 
                    f.set_cursor_position((metadata_chunks[2].x + (cursor_col as u16), metadata_chunks[2].y + visual_line as u16)); 
                }
            } else if !is_editing_title && !is_editing_desc {
                f.render_widget(Paragraph::new(app.reply_input.as_str()).block(Block::default().borders(Borders::ALL).title(" Input ")), chunks[2]);
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
        render_shortcuts(f, chunks[chunks.len() - 1], vec![("j/k", "nav"), ("space", "expand"), ("s", "resolve"), ("enter", "edit/save"), ("shift+enter", "newline"), ("esc", "back"), ("o", "open"), ("r", "refresh"), ("q", "quit")], &app.theme);
    } else if let Some(issue) = &app.selected_issue {
        let is_metadata_focused = app.detail_section == DetailSection::Metadata;
        let is_editing_title = app.input_mode && app.input_target == InputTarget::EditTitle;
        let is_editing_desc = app.input_mode && app.input_target == InputTarget::EditDescription;
        let metadata_height = if is_editing_desc || (is_metadata_focused && app.selected_metadata_field == MetadataField::Description) { 15.min(f.area().height - 10) } else { 12 };
        let chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(metadata_height), Constraint::Min(0), Constraint::Length(1)]).split(f.area());
        
        let metadata_block = Block::default().borders(Borders::ALL).title(format!(" Issue: #{} ", issue.iid)).border_style(if is_metadata_focused { Style::default().fg(app.theme.border) } else { Style::default() });
        let inner_area = metadata_block.inner(chunks[0]);
        f.render_widget(metadata_block, chunks[0]);

        let metadata_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(metadata_height.saturating_sub(2)), Constraint::Length(1), Constraint::Min(0)]).split(inner_area);
        
        // --- Highlighting Logic ---
        let highlight = Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD);
        let field = |f| if is_metadata_focused && app.selected_metadata_field == f { highlight } else { Style::default() };

        let title_line = vec![Span::raw("Title: "), Span::styled(if is_editing_title { &app.reply_input } else { &issue.title }, field(MetadataField::Title))];
        
        let mut labels_line = vec![Span::styled("Labels: ", field(MetadataField::Labels))];
        for label in &issue.labels {
            labels_line.push(Span::raw(" "));
            labels_line.push(Span::styled(format!(" {} ", label.name), Style::default().bg(hex_to_color(&label.color)).fg(hex_to_color(&label.text_color)).add_modifier(Modifier::BOLD)));
        }

        let mut header_lines = vec![
            Line::from(title_line),
            Line::from(labels_line),
            Line::from(vec![
                Span::raw("Author: "), Span::styled(format!("@{}", issue.author.username), Style::default().fg(app.theme.author)), 
                Span::raw(" | Assignees: "), Span::styled(if issue.assignees.is_empty() { "None".to_string() } else { issue.assignees.iter().map(|a| format!("@{}", a.username)).collect::<Vec<_>>().join(", ") }, field(MetadataField::Assignees)),
            ]),
            Line::from(vec![
                Span::raw("State: "), Span::styled(&issue.state, get_status_style(&issue.state, &app.theme)), 
                Span::raw(" | Milestone: "), Span::styled(issue.milestone.as_ref().map(|m| m.title.as_str()).unwrap_or("None"), field(MetadataField::Milestone)), 
                Span::raw(" | Due Date: "), Span::styled(issue.due_date.as_deref().unwrap_or("None"), field(MetadataField::DueDate))
            ]),
        ];

        // Reactions line
        if !issue.reactions.is_empty() || issue.upvotes > 0 || issue.downvotes > 0 {
            let mut reaction_spans = vec![Span::raw("Reactions: ")];
            if issue.upvotes > 0 {
                reaction_spans.push(Span::styled(format!("👍 {} ", issue.upvotes), Style::default().fg(Color::Green)));
            }
            if issue.downvotes > 0 {
                reaction_spans.push(Span::styled(format!("👎 {} ", issue.downvotes), Style::default().fg(Color::Red)));
            }
            
            // Other reactions
            let mut counts = std::collections::HashMap::new();
            for r in &issue.reactions {
                let clean_name = r.name.trim_matches(':');
                if clean_name != "thumbsup" && clean_name != "thumbsdown" && clean_name != "+1" && clean_name != "-1" {
                    *counts.entry(clean_name).or_insert(0) += 1;
                }
            }
            let mut reaction_keys: Vec<_> = counts.keys().collect();
            reaction_keys.sort();
            for name in reaction_keys {
                let count = counts.get(name).unwrap();
                let emoji = if let Some(e) = emojis::get_by_shortcode(name) {
                    e.as_str()
                } else {
                    name
                };
                reaction_spans.push(Span::raw(format!("{} {} ", emoji, count)));
            }
            header_lines.push(Line::from(reaction_spans));
        }

        let mut line4 = vec![];
        if issue.weight.is_some() {
            line4.push(Span::raw("Weight: "));
            line4.push(Span::styled(issue.weight.map(|w| w.to_string()).unwrap_or_else(|| "None".to_string()), field(MetadataField::Weight)));
            line4.push(Span::raw(" | "));
        }
        line4.push(Span::raw("Confidential: "));
        line4.push(Span::styled(if issue.confidential { "Yes" } else { "No" }, field(MetadataField::Confidential)));
        line4.push(Span::raw(" | Created: "));
        line4.push(Span::styled(format_relative_time(&issue.created_at), Style::default().fg(app.theme.dim)));
        header_lines.push(Line::from(line4));

        header_lines.push(Line::from(vec![Span::raw("URL: "), Span::raw(&issue.web_url)]));

        if let Some(ts) = &issue.task_completion_status {
            if ts.count > 0 {
                header_lines.push(Line::from(vec![Span::raw("Tasks: "), Span::styled(format!("{} of {} completed", ts.completed_count, ts.count), Style::default().fg(app.theme.border))]));
            }
        }
        f.render_widget(Paragraph::new(header_lines).wrap(Wrap { trim: true }), metadata_chunks[0]);
        f.render_widget(Paragraph::new(vec![Line::from(""), Line::from(vec![Span::styled("Description: ", field(MetadataField::Description))])]), metadata_chunks[1]);

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
            if is_editing_title { f.set_cursor_position((metadata_chunks[0].x + 7 + (app.cursor_index as u16), metadata_chunks[0].y)); }
            else if is_editing_desc {
                let chars: Vec<char> = app.reply_input.chars().collect();
                let before_cursor = &chars[..app.cursor_index.min(chars.len())];
                let s_before: String = before_cursor.iter().collect();
                let lines_before: Vec<&str> = s_before.split('\n').collect();
                let cursor_line = lines_before.len() - 1;
                let cursor_col = lines_before.last().unwrap_or(&"").chars().count();

                let visual_line = (cursor_line) as i32 - (scroll as i32);
                if visual_line >= 0 && visual_line < metadata_chunks[2].height as i32 { 
                    f.set_cursor_position((metadata_chunks[2].x + (cursor_col as u16), metadata_chunks[2].y + visual_line as u16)); 
                }
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
        render_shortcuts(f, chunks[2], vec![("j/k", "nav"), ("space", "expand"), ("c", "close/reopen"), ("n", "comment"), ("enter", "edit/save"), ("shift+enter", "newline"), ("esc", "back"), ("o", "open"), ("r", "refresh"), ("q", "quit")], &app.theme);
    } else if let Some(pipeline) = &app.selected_pipeline {
        let chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(8), Constraint::Min(0), Constraint::Length(1)]).split(f.area());
        
        let header_title = if app.is_loading { format!(" Pipeline: #{} (Refreshing...) ", pipeline.id) } else { format!(" Pipeline: #{} [LIVE] ", pipeline.id) };
        let header_style = if app.is_loading { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Green) };
        
        let header_block = Block::default().borders(Borders::ALL).title(header_title).border_style(header_style);

        let mut status_line = vec![Span::raw("Status: "), Span::styled(&pipeline.status, get_status_style(&pipeline.status, &app.theme))];
        if pipeline.status == "running" {
            if let Some(started) = &pipeline.started_at {
                status_line.push(Span::raw(" | Elapsed: "));
                status_line.push(Span::styled(crate::ui::widgets::get_elapsed_time(started), Style::default().fg(Color::Yellow)));
            }
        } else if let Some(duration) = pipeline.duration {
            status_line.push(Span::raw(" | Duration: "));
            status_line.push(Span::raw(crate::ui::widgets::format_duration(duration)));
        }

        let header_lines = vec![
            Line::from(status_line),
            Line::from(vec![Span::raw("Ref: "), Span::styled(&pipeline.ref_name, Style::default().add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::raw("Source: "), Span::raw(&pipeline.source)]),
            Line::from(vec![Span::raw("Created: "), Span::raw(format_relative_time(&pipeline.created_at))]),
            Line::from(vec![Span::raw("URL: "), Span::raw(&pipeline.web_url)]),
        ];
        f.render_widget(Paragraph::new(header_lines).block(header_block), chunks[0]);

        let jobs: Vec<ListItem> = app.jobs.iter().map(|job| {
            let state_style = get_status_style(&job.status, &app.theme);
            let mut spans = vec![
                Span::styled(format!("[{}]", job.status), state_style),
                Span::raw(format!(" {} - Stage: ", job.name)),
                Span::styled(&job.stage, Style::default().fg(app.theme.dim)),
            ];

            if job.status == "running" {
                if let Some(started) = &job.started_at {
                    spans.push(Span::raw(" | "));
                    spans.push(Span::styled(crate::ui::widgets::get_elapsed_time(started), Style::default().fg(Color::Yellow)));
                }
            } else if let Some(duration) = job.duration {
                spans.push(Span::raw(" | "));
                spans.push(Span::styled(crate::ui::widgets::format_duration(duration), Style::default().fg(app.theme.dim)));
            }

            ListItem::from(Line::from(spans))
        }).collect();
        
        let jobs_list = List::new(jobs)
            .block(Block::default().borders(Borders::ALL).title(" Jobs "))
            .highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        f.render_stateful_widget(jobs_list, chunks[1], &mut app.jobs_state);
        
        render_shortcuts(f, chunks[2], vec![("j/k", "nav"), ("esc", "back"), ("o", "open"), ("r", "refresh"), ("q", "quit")], &app.theme);
    }
}
