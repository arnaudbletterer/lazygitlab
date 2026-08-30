use crate::models::{App, InputTarget, SearchResult};
use crate::ui::widgets::{centered_rect, render_shortcuts};
use ratatui::{prelude::*, widgets::*};

pub fn render_search_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 80, f.area());
    app.set_rect("popup_search", area);
    f.render_widget(Clear, area);
    let block = Block::default()
        .bg(app.theme.code_bg)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            if app.is_loading { " 🔍 Global Search (Searching...) " } else { " 🔍 Global Search " },
            Style::default().fg(app.theme.accent).bold(),
        ))
        .border_style(Style::default().fg(app.theme.active_border));
    f.render_widget(block, area);

    let inner = area.inner(Margin {
        horizontal: 2,
        vertical: 2,
    });
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search Input
            Constraint::Min(0),    // Results
            Constraint::Length(1), // Footer
        ])
        .split(inner);

    // Search Input
    let search_style = if app.input_target == InputTarget::Search {
        Style::default().fg(app.theme.accent)
    } else {
        Style::default().fg(app.theme.inactive_border)
    };
    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(" Search Query (Esc to close) ", Style::default().fg(app.theme.title_fg)))
        .border_style(search_style);
    let search_para = Paragraph::new(app.search_query.as_str()).style(Style::default().fg(app.theme.title_fg)).block(search_block);
    f.render_widget(search_para, chunks[0]);

    if app.input_target == InputTarget::Search {
        f.set_cursor_position(Position::new(
            chunks[0].x + app.search_query.chars().count() as u16 + 1,
            chunks[0].y + 1,
        ));
    }

    // Results
    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .map(|res| match res {
            SearchResult::Project(p) => ListItem::new(Line::from(vec![
                Span::styled(" [Project] ", Style::default().fg(app.theme.project).bold()),
                Span::styled(format!(" 📂 {}", p.path_with_namespace.as_deref().unwrap_or(&p.name)), Style::default().fg(app.theme.title_fg)),
            ])),
            SearchResult::Issue(i) => ListItem::new(Line::from(vec![
                Span::styled(" [Issue]   ", Style::default().fg(app.theme.issue).bold()),
                Span::styled(format!(" #{} ", i.iid), Style::default().fg(app.theme.issue)),
                Span::styled(&i.title, Style::default().fg(app.theme.title_fg)),
            ])),
            SearchResult::MergeRequest(m) => ListItem::new(Line::from(vec![
                Span::styled(" [MR]      ", Style::default().fg(app.theme.mr).bold()),
                Span::styled(format!(" !{} ", m.iid), Style::default().fg(app.theme.mr)),
                Span::styled(&m.title, Style::default().fg(app.theme.title_fg)),
            ])),
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(app.theme.inactive_border))
                .title(Span::styled(format!(" Results ({}) ", app.search_results.len()), Style::default().fg(app.theme.title_fg))),
        )
        .highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .fg(app.theme.title_fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▎ ");
    f.render_stateful_widget(list, chunks[1], &mut app.search_state);

    // Footer
    render_shortcuts(
        f,
        chunks[2],
        vec![("arrows", "nav"), ("enter", "select"), ("esc", "close")],
        &app.theme,
    );
}
