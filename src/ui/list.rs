use ratatui::{prelude::*, widgets::*};
use crate::models::{App, Panel, GroupedItem, MenuSelection, TodoFilter, IssueFilter, MergeRequestFilter, DetailSection};
use crate::ui::detail::render_detail_view;

pub fn render_list_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    // --- LEFT COLUMN: STATUS & NAVIGATION (Filling the space) ---
    let side_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Status (Top)
            Constraint::Min(0),         // Dashboard (Expands)
            Constraint::Min(0),         // Explore (Expands)
            Constraint::Min(0),         // Project (Expands)
        ])
        .split(chunks[0]);

    // 1. Status Panel
    let status_block = Block::default()
        .borders(Borders::ALL)
        .title(" Status ")
        .border_style(Style::default().fg(app.theme.dim));
    let status_text = if let Some(project) = &app.selected_project {
        format!(" Project: {}\n Context: {:?}", project.name, app.active_context)
    } else {
        format!(" No Project Selected\n Context: {:?}", app.active_context)
    };
    f.render_widget(Paragraph::new(status_text).block(status_block), side_chunks[0]);

    // 2. Navigation Panels (Filling space)
    render_nav_panel(f, app.active_panel, &app.theme, side_chunks[1], Panel::Dashboard, " Dashboard ", &mut app.dashboard_menu_state, vec!["  Todos", "  Merge Requests", "  Issues"]);
    render_nav_panel(f, app.active_panel, &app.theme, side_chunks[2], Panel::Explore, " Explore ", &mut app.explore_menu_state, vec!["  Projects", "  Groups"]);
    render_nav_panel(f, app.active_panel, &app.theme, side_chunks[3], Panel::Project, " Project ", &mut app.project_menu_state, vec!["  Merge Requests", "  Issues", "  Pipelines"]);

    // --- RIGHT COLUMN: MAIN REGION (LIST -> DESCRIPTION -> CONTEXT) ---
    let is_item_selected = app.selected_mr.is_some() || app.selected_issue.is_some() || app.selected_pipeline.is_some();
    
    let main_chunks = if is_item_selected {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(35), // Main List (Top)
                Constraint::Percentage(30), // Detail View (Description)
                Constraint::Length(6),      // Metadata
                Constraint::Min(10),        // Discussions (Bottom)
            ])
            .split(chunks[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),         // Main List (Full height when nothing selected)
            ])
            .split(chunks[1])
    };

    if is_item_selected {
        render_main_list(f, app, main_chunks[0]);
        render_detail_view(f, app, main_chunks[1]);
        render_metadata_panel(f, app, main_chunks[2]);
        render_discussions_panel(f, app, main_chunks[3]);
    } else {
        render_main_list(f, app, main_chunks[0]);
    }
}

fn render_nav_panel(f: &mut Frame, active_panel: Panel, theme: &crate::ui::Theme, area: Rect, panel: Panel, title: &str, state: &mut ListState, items: Vec<&str>) {
    let is_focused = active_panel == panel;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if is_focused { Style::default().fg(theme.border) } else { Style::default().fg(theme.dim) });
    
    let list_items: Vec<ListItem> = items.iter().map(|i| ListItem::new(*i)).collect();
    f.render_stateful_widget(
        List::new(list_items)
            .block(block)
            .highlight_style(if is_focused { Style::default().bg(theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() })
            .highlight_symbol(if is_focused { "> " } else { "  " }),
        area,
        state
    );
}

fn render_main_list(f: &mut Frame, app: &mut App, area: Rect) {
    let is_main_focused = app.active_panel == Panel::Main;
    let list_title = match app.active_selection {
        MenuSelection::Todos => " Todos ",
        MenuSelection::MergeRequests => " Merge Requests ",
        MenuSelection::Issues => " Issues ",
        MenuSelection::Pipelines => " Pipelines ",
        MenuSelection::Projects => " Projects ",
        MenuSelection::Groups => " Groups ",
    };
    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(list_title)
        .border_style(if is_main_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });
    
    let list_inner = list_block.inner(area);
    f.render_widget(list_block, area);

    let list_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(list_inner);

    let (titles, selected) = match app.active_selection {
        MenuSelection::Todos => (vec!["Pending", "Snoozed", "Done"], match app.todo_filter { TodoFilter::Pending => 0, TodoFilter::Snoozed => 1, TodoFilter::Done => 2 }),
        MenuSelection::MergeRequests => (vec!["Opened", "Merged", "Closed", "All"], match app.mr_filter { MergeRequestFilter::Opened => 0, MergeRequestFilter::Merged => 1, MergeRequestFilter::Closed => 2, MergeRequestFilter::All => 3 }),
        MenuSelection::Issues => (vec!["Opened", "Closed", "All"], match app.issue_filter { IssueFilter::Opened => 0, IssueFilter::Closed => 1, IssueFilter::All => 2 }),
        _ => (vec!["List"], 0),
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .divider(Span::raw(" "))
        .style(Style::default().fg(app.theme.dim))
        .highlight_style(Style::default().fg(app.theme.border).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED));
    
    f.render_widget(tabs, list_chunks[0]);

    let highlight_style = Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD);
    let no_highlight = Style::default();

    match app.active_selection {
        MenuSelection::Todos => {
            let items: Vec<ListItem> = app.filtered_todos.iter().map(|item| {
                match item {
                    GroupedItem::Item(t) => ListItem::new(format!("{} - {}", t.action_name, t.body)),
                    GroupedItem::Header(h) => ListItem::new(Line::from(vec![Span::styled(format!("📁 {}", h), Style::default().fg(Color::Blue))])),
                }
            }).collect();
            f.render_stateful_widget(List::new(items).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), list_chunks[1], &mut app.todos_state);
        }
        MenuSelection::MergeRequests => {
            use crate::ui::widgets::hex_to_color;
            let items: Vec<ListItem> = app.filtered_mrs.iter().map(|item| {
                match item {
                    GroupedItem::Item(mr) => {
                        let mut spans = vec![Span::raw(format!("!{} {}", mr.iid, mr.title)), Span::raw(" ")];
                        for label in &mr.labels {
                            spans.push(Span::styled(format!(" {} ", label.name()), Style::default().bg(hex_to_color(label.color())).fg(hex_to_color(label.text_color()))));
                            spans.push(Span::raw(" "));
                        }
                        ListItem::new(Line::from(spans))
                    }
                    GroupedItem::Header(h) => ListItem::new(Line::from(vec![Span::styled(format!("📁 {}", h), Style::default().fg(Color::Blue))])),
                }
            }).collect();
            f.render_stateful_widget(List::new(items).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), list_chunks[1], &mut app.mrs_state);
        }
        MenuSelection::Issues => {
            use crate::ui::widgets::hex_to_color;
            let items: Vec<ListItem> = app.filtered_issues.iter().map(|item| {
                match item {
                    GroupedItem::Item(i) => {
                        let mut spans = vec![Span::raw(format!("#{} {}", i.iid, i.title)), Span::raw(" ")];
                        for label in &i.labels {
                            spans.push(Span::styled(format!(" {} ", label.name()), Style::default().bg(hex_to_color(label.color())).fg(hex_to_color(label.text_color()))));
                            spans.push(Span::raw(" "));
                        }
                        ListItem::new(Line::from(spans))
                    }
                    GroupedItem::Header(h) => ListItem::new(Line::from(vec![Span::styled(format!("📁 {}", h), Style::default().fg(Color::Blue))])),
                }
            }).collect();
            f.render_stateful_widget(List::new(items).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), list_chunks[1], &mut app.issues_state);
        }
        MenuSelection::Pipelines => {
            let items: Vec<ListItem> = app.filtered_pipelines.iter().map(|p| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", p.status), crate::ui::widgets::get_status_style(&p.status, &app.theme)),
                    Span::raw(format!(" {} - {}", p.ref_name, p.source)),
                ]))
            }).collect();
            f.render_stateful_widget(List::new(items).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), list_chunks[1], &mut app.pipelines_state);
        }
        _ => {}
    }
}

fn render_discussions_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let is_disc_focused = app.active_panel == Panel::MainDetail && app.detail_section == DetailSection::Discussions;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Discussions ")
        .border_style(if is_disc_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });

    let comments: Vec<ListItem> = app.discussions.iter().enumerate().map(|(idx, d)| {
        let is_selected = is_disc_focused && app.discussion_state.selected() == Some(idx);
        let highlight = if is_selected { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() };
        if let Some(note) = d.notes.first() {
            let mut spans = vec![
                Span::styled(if is_selected { "> " } else { "  " }, highlight),
                Span::styled(format!("@{}", note.author.username), highlight.fg(app.theme.author)),
            ];
            if d.resolved.unwrap_or(false) { spans.push(Span::styled(" [R]", highlight.fg(app.theme.success))); }
            spans.push(Span::styled(": ", highlight));
            spans.push(Span::styled(note.body.chars().take(40).collect::<String>(), highlight));
            ListItem::new(Line::from(spans))
        } else { ListItem::new(" (empty)") }
    }).collect();

    f.render_stateful_widget(List::new(comments).block(block), area, &mut app.discussion_state);
}

fn render_metadata_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let is_meta_focused = app.active_panel == Panel::MainDetail && app.detail_section == DetailSection::Metadata;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Metadata ")
        .border_style(if is_meta_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });

    let mut lines = Vec::new();
    let highlight = Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD);
    let field_style = |f| if is_meta_focused && app.selected_metadata_field == f { highlight } else { Style::default() };

    use crate::models::MetadataField;
    
    if let Some(mr) = &app.selected_mr {
        let title = if is_meta_focused && app.selected_metadata_field == MetadataField::Title && app.input_mode { &app.reply_input } else { &mr.title };
        lines.push(Line::from(vec![Span::raw(" Title: "), Span::styled(title, field_style(MetadataField::Title))]));
        lines.push(Line::from(vec![Span::raw(" Labels: "), Span::styled(format!("{} labels", mr.labels.len()), field_style(MetadataField::Labels))]));
        lines.push(Line::from(vec![Span::raw(" Assign: "), Span::styled(if mr.assignees.is_empty() { "None".to_string() } else { format!("@{}", mr.assignees[0].username) }, field_style(MetadataField::Assignees))]));
        lines.push(Line::from(vec![Span::raw(" Miles: "), Span::styled(mr.milestone.as_ref().map(|m| m.title.as_str()).unwrap_or("None"), field_style(MetadataField::Milestone))]));
    } else if let Some(issue) = &app.selected_issue {
        let title = if is_meta_focused && app.selected_metadata_field == MetadataField::Title && app.input_mode { &app.reply_input } else { &issue.title };
        lines.push(Line::from(vec![Span::raw(" Title: "), Span::styled(title, field_style(MetadataField::Title))]));
        lines.push(Line::from(vec![Span::raw(" Labels: "), Span::styled(format!("{} labels", issue.labels.len()), field_style(MetadataField::Labels))]));
        lines.push(Line::from(vec![Span::raw(" Assign: "), Span::styled(if issue.assignees.is_empty() { "None".to_string() } else { format!("@{}", issue.assignees[0].username) }, field_style(MetadataField::Assignees))]));
        lines.push(Line::from(vec![Span::raw(" Weight: "), Span::styled(issue.weight.map(|w| w.to_string()).unwrap_or_else(|| "0".into()), field_style(MetadataField::Weight))]));
    } else if let Some(pipeline) = &app.selected_pipeline {
        lines.push(Line::from(vec![Span::raw(" Pipeline: "), Span::styled(format!("#{}", pipeline.id), field_style(MetadataField::Title))]));
        lines.push(Line::from(vec![Span::raw(" Status: "), Span::styled(&pipeline.status, crate::ui::widgets::get_status_style(&pipeline.status, &app.theme))]));
        lines.push(Line::from(vec![Span::raw(" Ref: "), Span::raw(&pipeline.ref_name)]));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}
