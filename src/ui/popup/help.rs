use crate::models::App;
use crate::ui::widgets::centered_rect;
use ratatui::{prelude::*, widgets::*};

pub fn render_help_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(65, 75, f.area());
    app.set_rect("popup_help", area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .bg(Color::Black)
        .borders(Borders::ALL)
        .title(" Help & Shortcuts (?: close, /: search, Enter: run) ")
        .border_style(Style::default().fg(app.theme.border));
    f.render_widget(block, area);

    let inner = area.inner(Margin {
        horizontal: 2,
        vertical: 2,
    });
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if app.show_help_filter { 3 } else { 0 }),
            Constraint::Min(0),
        ])
        .split(inner);

    if app.show_help_filter {
        let filter_block = Block::default()
            .borders(Borders::ALL)
            .title(" Search Command ")
            .border_style(Style::default().fg(app.theme.highlight_bg));
        let filter_paragraph = Paragraph::new(app.help_filter_input.as_str()).block(filter_block);
        f.render_widget(filter_paragraph, chunks[0]);

        f.set_cursor_position(Position::new(
            chunks[0].x + app.help_filter_input.chars().count() as u16 + 1,
            chunks[0].y + 1,
        ));
    }

    let items: Vec<ListItem> = app
        .filtered_help
        .iter()
        .map(|h| {
            let line = Line::from(vec![
                Span::styled(
                    format!("{:<15}", h.shortcut),
                    Style::default()
                        .fg(app.theme.shortcut_key)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<35}", h.description),
                    Style::default().fg(app.theme.shortcut_desc),
                ),
                Span::styled(
                    format!("[{}]", h.category),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().bg(app.theme.highlight_bg))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[1], &mut app.help_state);
}
