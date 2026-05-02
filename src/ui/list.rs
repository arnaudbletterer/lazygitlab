use ratatui::{prelude::*, widgets::*};
use crate::models::{App, Panel, GroupedItem, MenuSelection};
use crate::ui::widgets::get_status_style;
use crate::ui::detail::render_detail_view;

pub fn render_list_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ])
        .split(area);

    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Dashboard
            Constraint::Length(4), // Explore
            Constraint::Min(0),    // Project
        ])
        .split(chunks[0]);

    let highlight_style = Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD);
    let no_highlight = Style::default();

    // --- Sidebar Panels ---
    
    // 1. Dashboard
    let dashboard_items = vec![
        ListItem::new(" Todos"),
        ListItem::new(" Merge Requests"),
        ListItem::new(" Issues"),
    ];
    let is_dash_focused = app.active_panel == Panel::Dashboard;
    let dashboard_block = Block::default().borders(Borders::ALL).title(" Dashboard ")
        .border_style(if is_dash_focused { Style::default().fg(app.theme.border).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.dim) });
    
    f.render_stateful_widget(List::new(dashboard_items).block(dashboard_block).highlight_style(if is_dash_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_dash_focused { "> " } else { "  " }), sidebar_chunks[0], &mut app.dashboard_menu_state);

    // 2. Explore
    let explore_items = vec![
        ListItem::new(" Projects"),
        ListItem::new(" Groups"),
    ];
    let is_explore_focused = app.active_panel == Panel::Explore;
    let explore_block = Block::default().borders(Borders::ALL).title(" Explore ")
        .border_style(if is_explore_focused { Style::default().fg(app.theme.border).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.dim) });
    
    f.render_stateful_widget(List::new(explore_items).block(explore_block).highlight_style(if is_explore_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_explore_focused { "> " } else { "  " }), sidebar_chunks[1], &mut app.explore_menu_state);

    // 3. Project
    let proj_name = app.selected_project.as_ref().map_or("None".to_string(), |p| p.name.clone());
    let project_items = vec![
        ListItem::new(" Pipelines"),
        ListItem::new(" Merge Requests"),
        ListItem::new(" Issues"),
    ];
    let is_project_focused = app.active_panel == Panel::Project;
    let project_block = Block::default().borders(Borders::ALL).title(format!(" Project: {} ", proj_name))
        .border_style(if is_project_focused { Style::default().fg(app.theme.border).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.dim) });
    
    f.render_stateful_widget(List::new(project_items).block(project_block).highlight_style(if is_project_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_project_focused { "> " } else { "  " }), sidebar_chunks[2], &mut app.project_menu_state);

    // --- Main Area (Right) ---
    
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // List
            Constraint::Percentage(60), // Preview/Detail
        ])
        .split(chunks[1]);

    let is_main_focused = app.active_panel == Panel::Main;
    let list_title = format!(" {:?} ", app.active_selection);
    let list_block = Block::default().borders(Borders::ALL).title(list_title)
        .border_style(if is_main_focused { Style::default().fg(app.theme.border).add_modifier(Modifier::BOLD) } else { Style::default().fg(app.theme.dim) });

    match app.active_selection {
        MenuSelection::Todos => {
            let items: Vec<ListItem> = app.filtered_todos.iter().map(|item| {
                match item {
                    GroupedItem::Item(t) => ListItem::new(format!("{} {}", match t.target_type.as_str() { "MergeRequest" => "🔀", "Issue" => "🎫", _ => "📋" }, t.body)),
                    GroupedItem::Header(h) => ListItem::new(Line::from(vec![Span::styled(format!("📁 {}", h), Style::default().fg(Color::Blue))])),
                }
            }).collect();
            f.render_stateful_widget(List::new(items).block(list_block).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), main_chunks[0], &mut app.todos_state);
        }
        MenuSelection::MergeRequests => {
            let items: Vec<ListItem> = app.filtered_mrs.iter().map(|item| {
                match item {
                    GroupedItem::Item(mr) => ListItem::new(format!("!{} {}", mr.iid, mr.title)),
                    GroupedItem::Header(h) => ListItem::new(Line::from(vec![Span::styled(format!("📁 {}", h), Style::default().fg(Color::Blue))])),
                }
            }).collect();
            f.render_stateful_widget(List::new(items).block(list_block).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), main_chunks[0], &mut app.mrs_state);
        }
        MenuSelection::Issues => {
            let items: Vec<ListItem> = app.filtered_issues.iter().map(|item| {
                match item {
                    GroupedItem::Item(i) => ListItem::new(format!("#{} {}", i.iid, i.title)),
                    GroupedItem::Header(h) => ListItem::new(Line::from(vec![Span::styled(format!("📁 {}", h), Style::default().fg(Color::Blue))])),
                }
            }).collect();
            f.render_stateful_widget(List::new(items).block(list_block).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), main_chunks[0], &mut app.issues_state);
        }
        MenuSelection::Pipelines => {
            let items: Vec<ListItem> = app.filtered_pipelines.iter().map(|p| {
                let status_style = get_status_style(&p.status, &app.theme);
                ListItem::new(Line::from(vec![Span::styled(&p.status, status_style), Span::raw(format!(" #{} {}", p.id, p.ref_name))]))
            }).collect();
            f.render_stateful_widget(List::new(items).block(list_block).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), main_chunks[0], &mut app.pipelines_state);
        }
        MenuSelection::Projects => {
            let items: Vec<ListItem> = app.filtered_projects.iter().map(|p| ListItem::new(p.name.as_str())).collect();
            f.render_stateful_widget(List::new(items).block(list_block).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), main_chunks[0], &mut app.projects_state);
        }
        MenuSelection::Groups => {
            let items: Vec<ListItem> = app.filtered_groups.iter().map(|g| ListItem::new(g.name.as_str())).collect();
            f.render_stateful_widget(List::new(items).block(list_block).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), main_chunks[0], &mut app.groups_state);
        }
    }

    // Detail Preview (Bottom-Right)
    render_detail_view(f, app, main_chunks[1]);
}
