use crate::models::App;
use crate::ui::widgets::centered_rect;
use ratatui::{prelude::*, widgets::*};

pub fn render_tree_branch_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 45, f.area());
    app.set_rect("popup_tree_branch", area);
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(app.theme.code_bg), area);

    let current = app.repository_ref.as_deref().unwrap_or("default");
    let branches: Vec<ListItem> = app
        .project_branches
        .iter()
        .map(|b| {
            let is_current = app.repository_ref.as_deref() == Some(&b.name)
                || (app.repository_ref.is_none() && b.default.unwrap_or(false));
            let mut spans = vec![
                Span::styled(" 🌿 ", Style::default().fg(app.theme.author)),
                Span::styled(
                    b.name.clone(),
                    if is_current {
                        Style::default().fg(app.theme.title_fg).bold()
                    } else {
                        Style::default().fg(app.theme.title_fg)
                    },
                ),
            ];
            if b.default.unwrap_or(false) {
                spans.push(Span::styled(
                    " [Default]",
                    Style::default().fg(app.theme.status_success).bold(),
                ));
            }
            if is_current {
                spans.push(Span::styled(
                    " (active)",
                    Style::default().fg(app.theme.author).bold(),
                ));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(branches)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(app.theme.active_border))
                .title(Span::styled(
                    format!(" 🌿 Switch Branch (Active: {}) ", current),
                    Style::default().fg(app.theme.title_fg).bold(),
                ))
                .title_bottom(Line::from(vec![
                    Span::styled(" Enter ", Style::default().fg(app.theme.author).bold()),
                    Span::styled("switch branch  ", Style::default().fg(app.theme.dim)),
                    Span::styled("Esc ", Style::default().fg(app.theme.author).bold()),
                    Span::styled("cancel ", Style::default().fg(app.theme.dim)),
                ])),
        )
        .highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .fg(app.theme.title_fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▎ ");

    f.render_stateful_widget(list, area, &mut app.tree_branch_state);
}
