use ratatui::{prelude::*, widgets::*};
use crate::models::{App, InputTarget};
use crate::ui::markdown::render_markdown;

pub fn render_detail_view(f: &mut Frame, app: &mut App, area: Rect) {
    let is_editing_desc = app.input_mode && app.input_target == InputTarget::EditDescription;
    let is_desc_focused = app.active_panel == crate::models::Panel::MainDetail && app.detail_section == crate::models::DetailSection::Description;
    let is_disc_focused = app.active_panel == crate::models::Panel::MainDetail && app.detail_section == crate::models::DetailSection::Discussions;
    
    let title = if is_disc_focused { " Discussion Thread " } else { " Description / Preview " };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if is_editing_desc || is_desc_focused || is_disc_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });

    let inner = block.inner(area);
    f.render_widget(block, area);

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
                    content.extend(render_markdown(&note.body, &app.theme));
                    content.push(Line::from(""));
                }
                if disc.notes.is_empty() { content.push(Line::from(" No notes in this discussion.")); }
            }
        } else {
            content.push(Line::from(" Select a discussion to view the full thread."));
        }
    } else {
        if let Some(mr) = &app.selected_mr {
            if is_editing_desc {
                f.render_widget(&app.textarea, inner);
                return;
            } else {
                content.extend(render_markdown(mr.description.as_deref().unwrap_or("No description provided."), &app.theme));
            }
        } else if let Some(issue) = &app.selected_issue {
            if is_editing_desc {
                f.render_widget(&app.textarea, inner);
                return;
            } else {
                content.extend(render_markdown(issue.description.as_deref().unwrap_or("No description provided."), &app.theme));
            }
        } else {
            content.push(Line::from(" No item selected."));
        }
    }

    f.render_widget(Paragraph::new(content).wrap(Wrap { trim: false }), inner);
}
