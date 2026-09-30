use crate::models::App;
use crate::ui::widgets::centered_rect;
use ratatui::{prelude::*, widgets::*};

pub fn render_snooze_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(40, 30, f.area());
    app.set_rect("popup_snooze", area);
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default()
            .bg(app.theme.code_bg)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " Snooze Todo ",
                Style::default().fg(app.theme.title_fg).bold(),
            ))
            .border_style(Style::default().fg(app.theme.active_border)),
        area,
    );

    let suggestions = [
        "In 1 hour",
        "Tomorrow",
        "In 3 days",
        "Next week",
        "Custom date/time...",
    ];

    let items: Vec<ListItem> = suggestions
        .iter()
        .map(|s| ListItem::new(Span::styled(*s, Style::default().fg(app.theme.title_fg))))
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .fg(app.theme.title_fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(
        list,
        area.inner(Margin {
            horizontal: 2,
            vertical: 2,
        }),
        &mut app.snooze_state,
    );
}
