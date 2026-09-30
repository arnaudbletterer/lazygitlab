use crate::models::App;
use crate::ui::banner::Banner;
use crate::ui::widgets::centered_rect;
use ratatui::{prelude::*, widgets::*};

pub fn render_help_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(65, 75, f.area());
    app.set_rect("popup_help", area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .bg(app.theme.code_bg)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " ❓ Help & Command Palette (?: close, /: search, Enter: run) ",
            Style::default().fg(app.theme.accent).bold(),
        ))
        .border_style(Style::default().fg(app.theme.active_border));
    f.render_widget(block, area);

    let inner = area.inner(Margin {
        horizontal: 2,
        vertical: 2,
    });

    let banner_h = Banner::height_for(inner.width, inner.height);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(banner_h),
            Constraint::Length(if app.show_help_filter { 3 } else { 0 }),
            Constraint::Min(0),
        ])
        .split(inner);

    if banner_h > 0 {
        Banner::render(f, chunks[0], &app.theme);
    }

    if app.show_help_filter {
        let filter_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " 🔍 Search Commands ",
                Style::default().fg(app.theme.title_fg),
            ))
            .border_style(Style::default().fg(app.theme.accent));
        let filter_paragraph = Paragraph::new(app.help_filter_input.as_str())
            .style(Style::default().fg(app.theme.title_fg))
            .block(filter_block);
        f.render_widget(filter_paragraph, chunks[1]);

        f.set_cursor_position(Position::new(
            chunks[1].x + app.help_filter_input.chars().count() as u16 + 1,
            chunks[1].y + 1,
        ));
    }

    let items: Vec<ListItem> = app
        .filtered_help
        .iter()
        .map(|h| {
            let line = Line::from(vec![
                Span::styled(
                    format!(" {:<15}", h.shortcut),
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
                        .fg(app.theme.dim)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .fg(app.theme.title_fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▎ ");
    f.render_stateful_widget(list, chunks[2], &mut app.help_state);
}
