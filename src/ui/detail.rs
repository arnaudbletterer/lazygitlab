use ratatui::{prelude::*, widgets::*};
use crate::models::{App, InputTarget};
use crate::ui::markdown::render_markdown;

pub fn render_detail_view(f: &mut Frame, app: &mut App, area: Rect) {
    let is_editing_desc = app.input_mode && app.input_target == InputTarget::EditDescription;
    let is_desc_focused = app.active_panel == crate::models::Panel::MainDetail && app.detail_section == crate::models::DetailSection::Description;
    let is_disc_focused = app.active_panel == crate::models::Panel::MainDetail && app.detail_section == crate::models::DetailSection::Discussions;
    let is_changes_focused = app.active_panel == crate::models::Panel::MainDetail && app.detail_section == crate::models::DetailSection::Changes;
    
    let title = if is_disc_focused { " Discussion Thread " } 
                else if is_changes_focused { " Changes / Files " }
                else { " Description / Preview " };
                
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if is_editing_desc || is_desc_focused || is_disc_focused || is_changes_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });

    let inner = block.inner(area);
    f.render_widget(block, area);

    if is_changes_focused {
        if app.mr_diffs.is_empty() {
            f.render_widget(Paragraph::new(" No changes found or still loading...").fg(app.theme.dim), inner);
        } else {
            let files: Vec<ListItem> = app.mr_diffs.iter().map(|d| {
                ListItem::new(Line::from(vec![
                    Span::styled(" ● ", Style::default().fg(app.theme.border)),
                    Span::raw(format!(" {} -> {}", d.old_path, d.new_path)),
                ]))
            }).collect();
            f.render_stateful_widget(
                List::new(files)
                    .highlight_style(if is_changes_focused { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() })
                    .highlight_symbol(if is_changes_focused { "> " } else { "  " }), 
                inner,
                &mut app.mr_diff_state
            );
            
            // Helpful hint at the bottom
            let hint_area = Rect::new(inner.x, inner.y + inner.height.saturating_sub(1), inner.width, 1);
            f.render_widget(Paragraph::new(" Press [Enter] or [v] to view full diff ").fg(app.theme.dim).alignment(Alignment::Right), hint_area);
        }
        return;
    }

    let mut content = Vec::new();

    if is_disc_focused {
        if let Some(idx) = app.discussion_state.selected() {
            if let Some(disc) = app.discussions.get(idx) {
                for note in &disc.notes {
                    let mut header = vec![
                        Span::styled(format!("@{}", note.author.username), Style::default().fg(app.theme.author).add_modifier(Modifier::BOLD)),
                        Span::raw(" "),
                    ];
                    if note.system { header.push(Span::styled(" [SYSTEM]", Style::default().fg(app.theme.dim))); }
                    content.push(Line::from(header));
                    content.extend(render_markdown(&note.body, &app.theme, &app.syntax_set, &app.theme_set));
                    content.push(Line::from(""));
                }
                if disc.notes.is_empty() { content.push(Line::from(" No notes in this discussion.")); }
            }
        } else {
            content.push(Line::from(" Select a discussion thread to view full conversation."));
        }
    } else {
        if let Some(mr) = &app.selected_mr {
            if is_editing_desc {
                f.render_widget(&app.textarea, inner);
                return;
            } else {
                content.extend(render_markdown(mr.description.as_deref().unwrap_or("No description provided."), &app.theme, &app.syntax_set, &app.theme_set));
            }
        } else if let Some(issue) = &app.selected_issue {
            if is_editing_desc {
                f.render_widget(&app.textarea, inner);
                return;
            } else {
                content.extend(render_markdown(issue.description.as_deref().unwrap_or("No description provided."), &app.theme, &app.syntax_set, &app.theme_set));
            }
        } else {
            content.push(Line::from(" No item selected."));
        }
    }

    f.render_widget(Paragraph::new(content).wrap(Wrap { trim: false }), inner);
}
