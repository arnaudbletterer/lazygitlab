use ratatui::{prelude::*, widgets::*};
use crate::models::App;

pub fn render_boards_view(f: &mut Frame, app: &mut App, area: Rect) {
    if app.selected_board.is_none() {
        render_board_list(f, app, area);
    } else {
        render_kanban_board(f, app, area);
    }
}

fn render_board_list(f: &mut Frame, app: &mut App, area: Rect) {
    let is_main_focused = app.active_panel == crate::models::Panel::Main;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Select a Board ")
        .border_style(if is_main_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });
    
    let items: Vec<ListItem> = app.project_boards.iter().map(|b| {
        ListItem::new(format!("  {}", b.name))
    }).collect();

    if items.is_empty() {
        f.render_widget(Paragraph::new(" No boards found for this project.").fg(app.theme.dim).block(block), area);
    } else {
        f.render_stateful_widget(List::new(items).block(block).highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD)).highlight_symbol("> "), area, &mut app.project_boards_state);
    }
}

fn render_kanban_board(f: &mut Frame, app: &mut App, area: Rect) {
    let lists = &app.board_lists;
    
    if lists.is_empty() {
        f.render_widget(Paragraph::new(" No lists found for this board.").fg(app.theme.dim), area);
        return;
    }

    let column_count = lists.len();
    let constraints: Vec<Constraint> = (0..column_count).map(|_| Constraint::Percentage(100 / column_count as u16)).collect();
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, list) in lists.iter().enumerate() {
        let title = match &list.label {
            Some(l) => l.name.clone(),
            None => list.list_type.clone(),
        };

        let is_column_focused = app.active_panel == crate::models::Panel::Main && app.board_lists_state.selected() == Some(i);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", title))
            .border_style(if is_column_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });

        let issues = app.board_issues.get(&list.id).map(|v| v.as_slice()).unwrap_or(&[]);
        let list_items: Vec<ListItem> = issues.iter().map(|issue| {
            let mut spans = vec![Span::styled(format!("#{} ", issue.iid), Style::default().fg(app.theme.dim))];
            spans.push(Span::raw(issue.title.chars().take(20).collect::<String>()));
            if issue.title.len() > 20 { spans.push(Span::raw("...")); }
            ListItem::new(Line::from(spans))
        }).collect();

        let mut state = app.board_issue_states.get(&list.id).cloned().unwrap_or_default();
        f.render_stateful_widget(List::new(list_items).block(block).highlight_style(Style::default().bg(app.theme.highlight_bg)), chunks[i], &mut state);
    }
}

pub fn render_epics_view(f: &mut Frame, app: &mut App, area: Rect) {
    let is_main_focused = app.active_panel == crate::models::Panel::Main;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Epics ")
        .border_style(if is_main_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });
    
    let items: Vec<ListItem> = app.epics.iter().map(|e| {
        let mut spans = vec![
            Span::styled(format!("&{} ", e.iid), Style::default().fg(app.theme.dim)),
            Span::styled(e.title.clone(), Style::default().add_modifier(Modifier::BOLD)),
        ];
        if e.state == "closed" { spans.push(Span::styled(" [CLOSED]", Style::default().fg(Color::Red))); }
        ListItem::new(Line::from(spans))
    }).collect();

    if items.is_empty() {
        f.render_widget(Paragraph::new(" No epics found for this group.").fg(app.theme.dim).block(block), area);
    } else {
        f.render_stateful_widget(List::new(items).block(block).highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD)).highlight_symbol("> "), area, &mut app.epics_state);
    }
}
