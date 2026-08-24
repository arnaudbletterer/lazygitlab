use crate::models::App;
use crate::ui::widgets::centered_rect;
use ratatui::{prelude::*, widgets::*};

pub fn render_tree_branch_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(40, 40, f.area());
    app.set_rect("popup_tree_branch", area);
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);

    let current = app.repository_ref.as_deref().unwrap_or("default");
    let branches: Vec<ListItem> = app
        .project_branches
        .iter()
        .map(|b| ListItem::new(b.name.clone()))
        .collect();

    let list = List::new(branches)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Select Branch [current: {}] ", current)),
        )
        .highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, area, &mut app.tree_branch_state);
}
