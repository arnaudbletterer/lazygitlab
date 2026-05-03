use ratatui::{prelude::*, widgets::*};
use crate::models::{App, Panel, GroupedItem, MenuSelection, TodoFilter, IssueFilter, MergeRequestFilter};
use crate::ui::widgets::get_status_style;
use crate::ui::detail::render_detail_view;

pub fn render_list_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let side_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Length(6), Constraint::Length(10), Constraint::Min(0)])
        .split(chunks[0]);

    // Dashboard Panel
    let is_dash_focused = app.active_panel == Panel::Dashboard;
    let dash_block = Block::default().borders(Borders::ALL).title(" Dashboard ").border_style(if is_dash_focused { Style::default().fg(app.theme.border) } else { Style::default() });
    let dash_items = vec![
        ListItem::new("  Todos"),
        ListItem::new("  Merge Requests"),
        ListItem::new("  Issues"),
    ];
    f.render_stateful_widget(List::new(dash_items).block(dash_block).highlight_style(if is_dash_focused { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() }).highlight_symbol(if is_dash_focused { "> " } else { "  " }), side_chunks[0], &mut app.dashboard_menu_state);

    // Explore Panel
    let is_explore_focused = app.active_panel == Panel::Explore;
    let explore_block = Block::default().borders(Borders::ALL).title(" Explore ").border_style(if is_explore_focused { Style::default().fg(app.theme.border) } else { Style::default() });
    let explore_items = vec![
        ListItem::new("  Projects"),
        ListItem::new("  Groups"),
    ];
    f.render_stateful_widget(List::new(explore_items).block(explore_block).highlight_style(if is_explore_focused { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() }).highlight_symbol(if is_explore_focused { "> " } else { "  " }), side_chunks[1], &mut app.explore_menu_state);

    // Project Panel
    let is_proj_focused = app.active_panel == Panel::Project;
    let proj_block = Block::default().borders(Borders::ALL).title(" Project ").border_style(if is_proj_focused { Style::default().fg(app.theme.border) } else { Style::default() });
    let proj_items = vec![
        ListItem::new("  Merge Requests"),
        ListItem::new("  Issues"),
        ListItem::new("  Pipelines"),
    ];
    f.render_stateful_widget(List::new(proj_items).block(proj_block).highlight_style(if is_proj_focused { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() }).highlight_symbol(if is_proj_focused { "> " } else { "  " }), side_chunks[2], &mut app.project_menu_state);

    // Main List (Middle)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Percentage(40)])
        .split(chunks[1]);

    let is_main_focused = app.active_panel == Panel::Main;
    let list_block = Block::default().borders(Borders::ALL).border_style(if is_main_focused { Style::default().fg(app.theme.border) } else { Style::default() });
    let highlight_style = Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD);
    let no_highlight = Style::default();

    // Render Tabs for Filtering (lazygit style)
    let (titles, selected) = match app.active_selection {
        MenuSelection::Todos => (vec![" Pending ", " Snoozed ", " Done "], match app.todo_filter { TodoFilter::Pending => 0, TodoFilter::Snoozed => 1, TodoFilter::Done => 2 }),
        MenuSelection::MergeRequests => (vec![" Opened ", " Merged ", " Closed ", " All "], match app.mr_filter { MergeRequestFilter::Opened => 0, MergeRequestFilter::Merged => 1, MergeRequestFilter::Closed => 2, MergeRequestFilter::All => 3 }),
        MenuSelection::Issues => (vec![" Opened ", " Closed ", " All "], match app.issue_filter { IssueFilter::Opened => 0, IssueFilter::Closed => 1, IssueFilter::All => 2 }),
        _ => (vec![" List "], 0),
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .divider(Span::raw(" "))
        .style(Style::default().fg(app.theme.dim))
        .highlight_style(Style::default().fg(app.theme.border).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED));
    
    f.render_widget(tabs, main_chunks[0]);

    match app.active_selection {
        MenuSelection::Todos => {
            let items: Vec<ListItem> = app.filtered_todos.iter().map(|item| {
                match item {
                    GroupedItem::Item(t) => ListItem::new(format!("{} - {}", t.action_name, t.body)),
                    GroupedItem::Header(h) => ListItem::new(Line::from(vec![Span::styled(format!("📁 {}", h), Style::default().fg(Color::Blue))])),
                }
            }).collect();
            f.render_stateful_widget(List::new(items).block(list_block).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), main_chunks[1], &mut app.todos_state);
        }
        MenuSelection::MergeRequests => {
            use crate::ui::widgets::hex_to_color;
            let items: Vec<ListItem> = app.filtered_mrs.iter().map(|item| {
                match item {
                    GroupedItem::Item(mr) => {
                        let mut spans = vec![
                            Span::raw(format!("!{} {}", mr.iid, mr.title)),
                            Span::raw(" "),
                        ];
                        for label in &mr.labels {
                            let color = hex_to_color(label.color());
                            let text_color = hex_to_color(label.text_color());
                            spans.push(Span::styled(format!(" {} ", label.name()), Style::default().bg(color).fg(text_color)));
                            spans.push(Span::raw(" "));
                        }
                        ListItem::new(Line::from(spans))
                    }
                    GroupedItem::Header(h) => ListItem::new(Line::from(vec![Span::styled(format!("📁 {}", h), Style::default().fg(Color::Blue))])),
                }
            }).collect();
            f.render_stateful_widget(List::new(items).block(list_block).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), main_chunks[1], &mut app.mrs_state);
        }
        MenuSelection::Issues => {
            use crate::ui::widgets::hex_to_color;
            let items: Vec<ListItem> = app.filtered_issues.iter().map(|item| {
                match item {
                    GroupedItem::Item(i) => {
                        let mut spans = vec![
                            Span::raw(format!("#{} {}", i.iid, i.title)),
                            Span::raw(" "),
                        ];
                        for label in &i.labels {
                            let color = hex_to_color(label.color());
                            let text_color = hex_to_color(label.text_color());
                            spans.push(Span::styled(format!(" {} ", label.name()), Style::default().bg(color).fg(text_color)));
                            spans.push(Span::raw(" "));
                        }
                        ListItem::new(Line::from(spans))
                    }
                    GroupedItem::Header(h) => ListItem::new(Line::from(vec![Span::styled(format!("📁 {}", h), Style::default().fg(Color::Blue))])),
                }
            }).collect();
            f.render_stateful_widget(List::new(items).block(list_block).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), main_chunks[1], &mut app.issues_state);
        }
        MenuSelection::Pipelines => {
            let items: Vec<ListItem> = app.filtered_pipelines.iter().map(|p| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", p.status), get_status_style(&p.status, &app.theme)),
                    Span::raw(format!(" {} - {}", p.ref_name, p.source)),
                ]))
            }).collect();
            f.render_stateful_widget(List::new(items).block(list_block).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), main_chunks[1], &mut app.pipelines_state);
        }
        MenuSelection::Projects => {
            let items: Vec<ListItem> = app.filtered_projects.iter().map(|p| ListItem::new(format!("  {}", p.name))).collect();
            f.render_stateful_widget(List::new(items).block(list_block).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), main_chunks[1], &mut app.projects_state);
        }
        MenuSelection::Groups => {
            let items: Vec<ListItem> = app.filtered_groups.iter().map(|g| ListItem::new(format!("  {}", g.name))).collect();
            f.render_stateful_widget(List::new(items).block(list_block).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), main_chunks[1], &mut app.groups_state);
        }
    }

    // Detail Preview (Bottom-Right)
    render_detail_view(f, app, main_chunks[2]);
}
