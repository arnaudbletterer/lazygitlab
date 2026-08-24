use crate::models::{App, InputTarget, SearchResult};
use crate::ui::widgets::{centered_rect, render_shortcuts};
use ratatui::{prelude::*, widgets::*};

pub fn render_search_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 80, f.area());
    app.set_rect("popup_search", area);
    f.render_widget(Clear, area);
    let title = if app.is_loading {
        " Global Search (Searching...) "
    } else {
        " Global Search "
    };
    f.render_widget(
        Block::default()
            .bg(Color::Black)
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(app.theme.border)),
        area,
    );

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
        Style::default().fg(app.theme.highlight_bg)
    } else {
        Style::default()
    };
    let search_block = Block::default()
        .borders(Borders::ALL)
        .title(" Search Query (Esc to close) ")
        .border_style(search_style);
    let search_para = Paragraph::new(app.search_query.as_str()).block(search_block);
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
                Span::styled(" [Project] ", Style::default().fg(app.theme.project)),
                Span::raw(format!(
                    " {}",
                    p.path_with_namespace.as_deref().unwrap_or(&p.name)
                )),
            ])),
            SearchResult::Issue(i) => ListItem::new(Line::from(vec![
                Span::styled(" [Issue]   ", Style::default().fg(app.theme.issue)),
                Span::raw(format!(" #{} {}", i.iid, i.title)),
            ])),
            SearchResult::MergeRequest(m) => ListItem::new(Line::from(vec![
                Span::styled(" [MR]      ", Style::default().fg(app.theme.mr)),
                Span::raw(format!(" !{} {}", m.iid, m.title)),
            ])),
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Results ({}) ", app.search_results.len())),
        )
        .highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, chunks[1], &mut app.search_state);

    // Footer
    render_shortcuts(
        f,
        chunks[2],
        vec![("arrows", "nav"), ("enter", "select"), ("esc", "close")],
        &app.theme,
    );
}
