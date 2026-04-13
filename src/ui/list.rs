use ratatui::{prelude::*, widgets::*};
use crate::models::{App, MenuSelection};
use crate::ui::widgets::{render_shortcuts, get_status_style, hex_to_color};

pub fn render_list_view(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(chunks[0]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(main_chunks[0]);

    let proj_name = app.selected_project.as_ref().map(|p| p.name.as_str()).unwrap_or("None");
    let status_info = Paragraph::new(format!("Project: {}", proj_name))
        .block(Block::default().borders(Borders::ALL).title(" Status ").border_style(Style::default().fg(app.theme.border)));
    f.render_widget(status_info, left_chunks[0]);

    let items: Vec<ListItem> = app.menu_items.iter().map(|(name, _)| ListItem::new(name.as_str())).collect();
    let sidebar_style = if app.focus_on_menu { Style::default().fg(app.theme.border) } else { Style::default() };
    let sidebar_highlight_style = if app.focus_on_menu { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Menu ").border_style(sidebar_style))
        .highlight_style(sidebar_highlight_style)
        .highlight_symbol(if app.focus_on_menu { ">> " } else { "   " });
    f.render_stateful_widget(list, left_chunks[1], &mut app.menu_state);

    let content_style = if !app.focus_on_menu { Style::default().fg(app.theme.border) } else { Style::default() };
    let content_highlight_style = if !app.focus_on_menu { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() };

    if app.selected_project.is_none() {
        f.render_widget(Paragraph::new("\n\n  No project selected.\n  Press Ctrl+R to select a project.").block(Block::default().borders(Borders::ALL).title(" Content ").border_style(content_style)), main_chunks[1]);
    } else {
        match app.active_selection {
            MenuSelection::MergeRequests => {
                let mrs: Vec<ListItem> = app.merge_requests.iter().map(|mr| {
                    let state_style = get_status_style(&mr.state, &app.theme);
                    ListItem::from(Line::from(vec![
                        Span::styled(format!("[{}]", mr.state), state_style),
                        Span::raw(format!(" {} - {} by ", mr.references.full, mr.title)),
                        Span::styled(format!("@{}", mr.author.username), Style::default().fg(app.theme.author)),
                    ]))
                }).collect();
                f.render_stateful_widget(List::new(mrs).block(Block::default().borders(Borders::ALL).title(format!(" MRs for {} ", proj_name)).border_style(content_style)).highlight_style(content_highlight_style).highlight_symbol(if !app.focus_on_menu { "> " } else { "  " }), main_chunks[1], &mut app.mrs_state);
            }
            MenuSelection::Issues => {
                let issues: Vec<ListItem> = app.issues.iter().map(|issue| {
                    let state_style = get_status_style(&issue.state, &app.theme);
                    let mut spans = vec![
                        Span::styled(format!("[{}]", issue.state), state_style),
                        Span::styled(format!(" [{}]", issue.issue_type.as_deref().unwrap_or("issue")), Style::default().fg(app.theme.dim)),
                        Span::raw(format!(" #{} - {} by ", issue.iid, issue.title)),
                        Span::styled(format!("@{}", issue.author.username), Style::default().fg(app.theme.author)),
                    ];
                    for label in &issue.labels {
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(format!(" {} ", label.name), Style::default().bg(hex_to_color(&label.color)).fg(hex_to_color(&label.text_color)).add_modifier(Modifier::BOLD)));
                    }
                    ListItem::from(Line::from(spans))
                }).collect();
                f.render_stateful_widget(List::new(issues).block(Block::default().borders(Borders::ALL).title(format!(" Issues for {} ", proj_name)).border_style(content_style)).highlight_style(content_highlight_style).highlight_symbol(if !app.focus_on_menu { "> " } else { "  " }), main_chunks[1], &mut app.issues_state);
            }
            _ => { f.render_widget(Paragraph::new(format!("Content for {:?}", app.active_selection)).block(Block::default().borders(Borders::ALL).title(" Content ").border_style(content_style)), main_chunks[1]); }
        }
    }

    let shortcuts = vec![("j/k", "nav"), ("tab", "switch"), ("ctrl+r", "switch project"), ("enter", "select"), ("r", "refresh"), ("q", "quit")];
    render_shortcuts(f, chunks[1], shortcuts, &app.theme);
}
