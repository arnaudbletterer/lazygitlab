use ratatui::{prelude::*, widgets::*};
use crate::models::{App, InputTarget};
use crate::ui::markdown::render_markdown;

pub fn render_detail_view(f: &mut Frame, app: &mut App, area: Rect) {
    let is_editing_desc = app.input_mode && app.input_target == InputTarget::EditDescription;
    let is_focused = app.active_panel == crate::models::Panel::MainDetail && app.detail_section == crate::models::DetailSection::Description;
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Description / Preview ")
        .border_style(if is_editing_desc || is_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut content = Vec::new();

    if let Some(mr) = &app.selected_mr {
        if is_editing_desc {
            content.extend(app.reply_input.split('\n').map(|l| Line::from(l)));
        } else {
            content.extend(render_markdown(mr.description.as_deref().unwrap_or("No description provided."), &app.theme));
        }
    } else if let Some(issue) = &app.selected_issue {
        if is_editing_desc {
            content.extend(app.reply_input.split('\n').map(|l| Line::from(l)));
        } else {
            content.extend(render_markdown(issue.description.as_deref().unwrap_or("No description provided."), &app.theme));
        }
    } else {
        content.push(Line::from(" No item selected."));
    }

    f.render_widget(Paragraph::new(content).wrap(Wrap { trim: false }), inner);
}
