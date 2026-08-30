use crate::models::App;
use crate::ui::widgets::centered_rect;
use ratatui::{prelude::*, widgets::*};

pub fn render_project_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 40, f.area());
    app.set_rect("popup_project", area);
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);

    let projects: Vec<ListItem> = app
        .projects
        .iter()
        .map(|p| {
            ListItem::new(Line::from(vec![
                Span::styled(" 📂 ", Style::default().fg(app.theme.project)),
                Span::styled(format!("{} ", p.name), Style::default().fg(app.theme.title_fg).bold()),
                Span::styled(format!("({})", p.path_with_namespace.as_deref().unwrap_or("")), Style::default().fg(app.theme.dim)),
            ]))
        })
        .collect();

    let list = List::new(projects)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    " 📂 Select Project (Enter to switch, Esc to cancel) ",
                    Style::default().fg(app.theme.accent).bold(),
                ))
                .border_style(Style::default().fg(app.theme.active_border)),
        )
        .highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▎ ");

    f.render_stateful_widget(list, area, &mut app.projects_state);
}
