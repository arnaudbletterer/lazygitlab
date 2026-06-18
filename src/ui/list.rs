use ratatui::{prelude::*, widgets::*};
use crate::models::{App, Panel, GroupedItem, MenuSelection, TodoFilter, IssueFilter, MergeRequestFilter, DetailSection};
use crate::ui::detail::render_detail_view;
use crate::ui::Theme;

pub fn render_list_view(f: &mut Frame, app: &mut App, area: Rect) {
    let is_item_selected = app.selected_mr.is_some() || app.selected_issue.is_some() || app.selected_pipeline.is_some();

    let chunks = if is_item_selected {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // Column 1: Navigation
                Constraint::Percentage(35), // Column 2: Main List
                Constraint::Percentage(45), // Column 3: Detail Panel
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // Column 1: Navigation
                Constraint::Min(0),         // Column 2: Main List
            ])
            .split(area)
    };

    // --- LEFT COLUMN: STATUS & NAVIGATION (Filling the space) ---
    let side_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Status (Top)
            Constraint::Length(5),      // Dashboard (3 items + title/border)
            Constraint::Length(4),      // Explore (2 items + title/border)
            if app.selected_project.is_some() { Constraint::Min(10) } else { Constraint::Length(0) },
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
        " No Project Selected\n Context: Dashboard".to_string()
    };
    f.render_widget(Paragraph::new(status_text).block(status_block), side_chunks[0]);

    // 2. Navigation Panels (Filling space)
    app.set_rect("nav_dashboard", side_chunks[1]);
    render_nav_panel(f, app.active_panel == Panel::Dashboard, &app.theme, side_chunks[1], " Dashboard ", &mut app.dashboard_menu_state, vec!["  Todos", "  Merge Requests", "  Issues"]);
    
    app.set_rect("nav_explore", side_chunks[2]);
    render_nav_panel(f, app.active_panel == Panel::Explore, &app.theme, side_chunks[2], " Explore ", &mut app.explore_menu_state, vec!["  Projects", "  Groups"]);
    
    if app.selected_project.is_some() {
        let has_epics = app.selected_project.as_ref()
            .and_then(|p| p.namespace.as_ref())
            .is_some_and(|ns| ns.kind == "group");
        app.set_rect("nav_project", side_chunks[3]);
        let mut project_items = vec!["  Merge Requests", "  Issues", "  Pipelines", "  Repository", "  Branches", "  Commits", "  Boards"];
        if has_epics {
            project_items.push("  Epics");
        }
        render_nav_panel(f, app.active_panel == Panel::Project, &app.theme, side_chunks[3], " Project ", &mut app.project_menu_state, project_items);
    }

    // --- RIGHT COLUMN: MAIN REGION (LIST & DETAIL COLUMN) ---
    if is_item_selected {
        if app.active_selection == MenuSelection::Pipelines {
            app.set_rect("main_list", chunks[1]);
            render_main_list(f, app, chunks[1]);
            app.set_rect("jobs_panel", chunks[2]);
            render_jobs_panel(f, app, chunks[2]);
        } else {
            app.set_rect("main_list", chunks[1]);
            render_main_list(f, app, chunks[1]);
            app.set_rect("detail_view", chunks[2]);
            render_detail_view(f, app, chunks[2]);
        }
    } else {
        app.set_rect("main_list", chunks[1]);
        render_main_list(f, app, chunks[1]);
    }
}

fn render_nav_panel(f: &mut Frame, is_focused: bool, theme: &Theme, area: Rect, title: &str, state: &mut ListState, items: Vec<&str>) {
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
        MenuSelection::Todos => " To-Do List ".to_string(),
        MenuSelection::MergeRequests => {
            if app.show_involved_only {
                " Merge Requests (Involved Only) ".to_string()
            } else {
                " Merge Requests ".to_string()
            }
        }
        MenuSelection::Issues => {
            if app.show_involved_only {
                " Issues (Involved Only) ".to_string()
            } else {
                " Issues ".to_string()
            }
        }
        MenuSelection::Pipelines => " Pipelines ".to_string(),
        MenuSelection::Projects => " Projects ".to_string(),
        MenuSelection::Groups => " Groups ".to_string(),
        MenuSelection::Repository => " Repository ".to_string(),
        MenuSelection::Branches => " Branches ".to_string(),
        MenuSelection::Commits => " Commits ".to_string(),
        MenuSelection::Boards => " Boards ".to_string(),
        MenuSelection::Epics => " Epics ".to_string(),
    };
    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(list_title.as_str())
        .border_style(if is_main_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });
    
    let list_inner = list_block.inner(area);
    f.render_widget(list_block, area);

    let (titles, selected) = match app.active_selection {
        MenuSelection::Todos => (vec!["Pending", "Snoozed", "Done"], Some(match app.todo_filter { TodoFilter::Pending => 0, TodoFilter::Snoozed => 1, TodoFilter::Done => 2 })),
        MenuSelection::MergeRequests => (vec!["Opened", "Merged", "Closed", "All"], Some(match app.mr_filter { MergeRequestFilter::Opened => 0, MergeRequestFilter::Merged => 1, MergeRequestFilter::Closed => 2, MergeRequestFilter::All => 3 })),
        MenuSelection::Issues => (vec!["Opened", "Closed", "All"], Some(match app.issue_filter { IssueFilter::Opened => 0, IssueFilter::Closed => 1, IssueFilter::All => 2 })),
        _ => (vec![], None),
    };

    let list_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if !titles.is_empty() {
            [Constraint::Length(1), Constraint::Min(0)]
        } else {
            [Constraint::Length(0), Constraint::Min(0)]
        })
        .split(list_inner);

    if !titles.is_empty() {
        let tabs = Tabs::new(titles)
            .select(selected.unwrap_or(0))
            .divider(Span::styled(" │ ", Style::default().fg(app.theme.dim)))
            .style(Style::default().fg(app.theme.dim))
            .highlight_style(Style::default().fg(app.theme.border).add_modifier(Modifier::BOLD));
        
        f.render_widget(tabs, list_chunks[0]);
    }

    let highlight_style = Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD);
    let no_highlight = Style::default();

    match app.active_selection {
        MenuSelection::Todos => {
            let items: Vec<ListItem> = app.filtered_todos.iter().map(|item| {
                match item {
                    GroupedItem::Item(t) => ListItem::new(format!("{} - {}", t.action_name, t.body)),
                    GroupedItem::Header(h) => {
                        let key = format!("todos:{}", h);
                        let icon = if app.collapsed_groups.contains(&key) { "▶" } else { "▼" };
                        ListItem::new(Line::from(vec![Span::styled(format!("{} {}", icon, h), Style::default().fg(app.theme.header).add_modifier(Modifier::BOLD))]))
                    },
                }
            }).collect();
            f.render_stateful_widget(List::new(items).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), list_chunks[1], &mut app.todos_state);
        }
        MenuSelection::MergeRequests => {
            let items: Vec<ListItem> = app.filtered_mrs.iter().map(|item| {
                match item {
                    GroupedItem::Item(mr) => {
                        use crate::ui::widgets::{hex_to_color, get_contrasting_text_color};
                        let mut involved = false;
                        if let Some(ref me) = app.current_user
                            && (mr.author.id == me.id
                                || mr.assignees.iter().any(|a| a.id == me.id)
                                || mr.reviewers.iter().any(|r| r.id == me.id)
                                || mr.participants.iter().any(|p| p.id == me.id))
                        {
                            involved = true;
                        }
                        let mut spans = Vec::new();
                        if involved {
                            spans.push(Span::styled("● ", Style::default().fg(Color::Yellow)));
                        }
                        spans.push(Span::raw(format!("!{} {}", mr.iid, mr.title)));
                        spans.push(Span::raw(" "));
                        for label in &mr.labels {
                            let bg = hex_to_color(label.color());
                            let fg = get_contrasting_text_color(label.color());
                            spans.push(Span::styled(format!(" {} ", label.name()), Style::default().bg(bg).fg(fg)));
                            spans.push(Span::raw(" "));
                        }
                        ListItem::new(Line::from(spans))
                    }
                    GroupedItem::Header(h) => {
                        let scope = match app.active_selection { MenuSelection::MergeRequests => "mrs", MenuSelection::Issues => "issues", _ => "unknown" };
                        let key = format!("{}:{}", scope, h);
                        let icon = if app.collapsed_groups.contains(&key) { "▶" } else { "▼" };
                        ListItem::new(Line::from(vec![Span::styled(format!("{} {}", icon, h), Style::default().fg(app.theme.header).add_modifier(Modifier::BOLD))]))
                    },
                }
            }).collect();
            f.render_stateful_widget(List::new(items).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), list_chunks[1], &mut app.mrs_state);
        }
        MenuSelection::Issues => {
            let items: Vec<ListItem> = app.filtered_issues.iter().map(|item| {
                match item {
                    GroupedItem::Item(i) => {
                        use crate::ui::widgets::{hex_to_color, get_contrasting_text_color};
                        let mut involved = false;
                        if let Some(ref me) = app.current_user
                            && (i.author.id == me.id
                                || i.assignees.iter().any(|a| a.id == me.id)
                                || i.participants.iter().any(|p| p.id == me.id))
                        {
                            involved = true;
                        }
                        let mut spans = Vec::new();
                        if involved {
                            spans.push(Span::styled("● ", Style::default().fg(Color::Yellow)));
                        }
                        spans.push(Span::raw(format!("#{} {}", i.iid, i.title)));
                        spans.push(Span::raw(" "));
                        for label in &i.labels {
                            let bg = hex_to_color(label.color());
                            let fg = get_contrasting_text_color(label.color());
                            spans.push(Span::styled(format!(" {} ", label.name()), Style::default().bg(bg).fg(fg)));
                            spans.push(Span::raw(" "));
                        }
                        ListItem::new(Line::from(spans))
                    }
                    GroupedItem::Header(h) => {
                        let scope = match app.active_selection { MenuSelection::MergeRequests => "mrs", MenuSelection::Issues => "issues", _ => "unknown" };
                        let key = format!("{}:{}", scope, h);
                        let icon = if app.collapsed_groups.contains(&key) { "▶" } else { "▼" };
                        ListItem::new(Line::from(vec![Span::styled(format!("{} {}", icon, h), Style::default().fg(app.theme.header).add_modifier(Modifier::BOLD))]))
                    },
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
        MenuSelection::Projects => {
            let items: Vec<ListItem> = app.filtered_projects.iter().map(|p| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", p.name), Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" ({})", p.path_with_namespace.as_deref().unwrap_or("")), Style::default().fg(app.theme.dim)),
                ]))
            }).collect();
            f.render_stateful_widget(List::new(items).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), list_chunks[1], &mut app.projects_state);
        }
        MenuSelection::Groups => {
            let items: Vec<ListItem> = app.filtered_groups.iter().map(|g| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", g.name), Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" ({})", g.full_path), Style::default().fg(app.theme.dim)),
                ]))
            }).collect();
            f.render_stateful_widget(List::new(items).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), list_chunks[1], &mut app.groups_state);
        }
        MenuSelection::Repository => {
            let mut tree_items: Vec<tui_tree_widget::TreeItem<'static, String>> = Vec::new();
            for item in &app.repository_tree {
                let text = if item.r#type == "tree" { format!("📁 {}", item.name) } else { format!("📄 {}", item.name) };
                tree_items.push(tui_tree_widget::TreeItem::new_leaf(item.path.clone(), text));
            }
            let tree = tui_tree_widget::Tree::new(&tree_items)
                .unwrap_or_else(|_| tui_tree_widget::Tree::new(&[]).unwrap())
                .block(Block::default().title("Tree").borders(Borders::ALL).border_style(if is_main_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) }))
                .highlight_style(if is_main_focused { highlight_style } else { no_highlight })
                .highlight_symbol(if is_main_focused { "> " } else { "  " });
            f.render_stateful_widget(tree, list_chunks[1], &mut app.repository_tree_state);
        }
        MenuSelection::Branches => {
            let items: Vec<ListItem> = app.filtered_branches.iter().map(|b| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", b.name), Style::default().add_modifier(Modifier::BOLD)),
                ]))
            }).collect();
            f.render_stateful_widget(List::new(items).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), list_chunks[1], &mut app.branches_state);
        }
        MenuSelection::Commits => {
            let items: Vec<ListItem> = app.commits.iter().map(|c| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", c.short_id), Style::default().fg(app.theme.dim)),
                    Span::styled(c.title.clone(), Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" - {}", c.author_name), Style::default().fg(app.theme.dim)),
                ]))
            }).collect();
            f.render_stateful_widget(List::new(items).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), list_chunks[1], &mut app.commits_state);
        }
        MenuSelection::Boards => {
            let items: Vec<ListItem> = app.project_boards.iter().map(|b| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", b.name), Style::default().add_modifier(Modifier::BOLD)),
                ]))
            }).collect();
            f.render_stateful_widget(List::new(items).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), list_chunks[1], &mut app.project_boards_state);
        }
        MenuSelection::Epics => {
            let items: Vec<ListItem> = app.epics.iter().map(|e| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("&{} ", e.iid), Style::default().fg(app.theme.dim)),
                    Span::styled(e.title.clone(), Style::default().add_modifier(Modifier::BOLD)),
                ]))
            }).collect();
            f.render_stateful_widget(List::new(items).highlight_style(if is_main_focused { highlight_style } else { no_highlight }).highlight_symbol(if is_main_focused { "> " } else { "  " }), list_chunks[1], &mut app.epics_state);
        }
    }
}

pub fn render_discussions_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let is_disc_focused = app.active_panel == Panel::MainDetail && app.detail_section == DetailSection::Discussions;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Discussions (Threads) ")
        .border_style(if is_disc_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });

    let inner = block.inner(area);
    f.render_widget(block, area);

    let is_replying = is_disc_focused && app.input_mode && app.input_target == crate::models::InputTarget::Reply;
    
    let chunks = if is_replying {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(5)])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(inner)
    };

    let comments: Vec<ListItem> = app.discussions.iter().enumerate().map(|(idx, d)| {
        let is_selected = is_disc_focused && app.discussion_state.selected() == Some(idx);
        let highlight = if is_selected { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() };
        if let Some(note) = d.notes.first() {
            let is_activity = note.system;
            let is_resolved = !is_activity && d.resolved.unwrap_or(false);

            let mut spans = vec![
                Span::styled(if is_selected { "● " } else { "  " }, highlight.fg(app.theme.border)),
            ];

            let text_style = if is_resolved || is_activity {
                highlight.fg(app.theme.dim)
            } else {
                highlight
            };

            if is_activity {
                spans.push(Span::styled("[ACTIVITY] ", highlight.fg(app.theme.dim)));
                spans.push(Span::styled(format!("@{} ", note.author.username), highlight.fg(app.theme.dim).add_modifier(Modifier::BOLD)));
            } else if let Some(resolved) = d.resolved {
                if resolved {
                    spans.push(Span::styled("[RESOLVED] ", highlight.fg(app.theme.dim)));
                    spans.push(Span::styled(format!("@{} ", note.author.username), highlight.fg(app.theme.dim).add_modifier(Modifier::BOLD)));
                } else {
                    spans.push(Span::styled("[UNRESOLVED] ", highlight.fg(app.theme.pending)));
                    spans.push(Span::styled(format!("@{} ", note.author.username), highlight.fg(app.theme.author).add_modifier(Modifier::BOLD)));
                }
            } else {
                spans.push(Span::styled(format!("@{} ", note.author.username), highlight.fg(app.theme.author).add_modifier(Modifier::BOLD)));
            }

            spans.push(Span::styled(": ", text_style));
            
            // Show more content (up to 100 chars) for a better overview
            let body = note.body.replace('\n', " ");
            spans.push(Span::styled(body.chars().take(100).collect::<String>(), text_style));
            if body.len() > 100 { spans.push(Span::styled("...", text_style)); }
            ListItem::new(Line::from(spans))
        } else { ListItem::new(" (empty thread)") }
    }).collect();

    if comments.is_empty() {
        f.render_widget(Paragraph::new(" No discussions found.").fg(app.theme.dim), chunks[0]);
    } else {
        f.render_stateful_widget(List::new(comments), chunks[0], &mut app.discussion_state);
    }

    if is_replying {
        let reply_block = Block::default().borders(Borders::ALL).title(" Reply (Ctrl+Enter to send, Esc to cancel) ").border_style(Style::default().fg(app.theme.border));
        f.render_widget(Clear, chunks[1]);
        app.textarea.set_block(reply_block);
        f.render_widget(&app.textarea, chunks[1]);
    }
}

pub fn render_jobs_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let is_jobs_focused = app.active_panel == Panel::MainDetail;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Pipeline Jobs ")
        .border_style(if is_jobs_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });

    let items: Vec<ListItem> = app.jobs.iter().map(|j| {
        ListItem::new(Line::from(vec![
            Span::styled(format!(" {} ", j.status), crate::ui::widgets::get_status_style(&j.status, &app.theme)),
            Span::raw(format!(" {} ", j.name)),
            Span::styled(format!("({})", j.stage), Style::default().fg(app.theme.dim)),
        ]))
    }).collect();

    if items.is_empty() {
        f.render_widget(Paragraph::new(" No jobs found for this pipeline.").fg(app.theme.dim).block(block), area);
    } else {
        f.render_stateful_widget(
            List::new(items)
                .block(block)
                .highlight_style(if is_jobs_focused { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() })
                .highlight_symbol(if is_jobs_focused { "> " } else { "  " }), 
            area, 
            &mut app.jobs_state
        );
    }
}

pub fn render_metadata_panel(f: &mut Frame, app: &mut App, area: Rect) {
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
        let title = if is_meta_focused && app.selected_metadata_field == MetadataField::Title && app.input_mode { app.textarea.lines().first().cloned().unwrap_or_default() } else { mr.title.clone() };
        lines.push(Line::from(vec![Span::raw(" Title: "), Span::styled(title, field_style(MetadataField::Title))]));
        
        let target_branch_style = if is_meta_focused && app.selected_metadata_field == MetadataField::TargetBranch {
            highlight
        } else {
            Style::default().fg(app.theme.project)
        };
        lines.push(Line::from(vec![
            Span::raw(" Branches: "),
            Span::styled(&mr.source_branch, Style::default().fg(app.theme.author)),
            Span::raw(" -> "),
            Span::styled(&mr.target_branch, target_branch_style),
        ]));

        let mut label_spans = vec![Span::raw(" Labels: ")];
        if mr.labels.is_empty() { label_spans.push(Span::styled("None", field_style(MetadataField::Labels))); }
        else {
            use crate::ui::widgets::{hex_to_color, get_contrasting_text_color};
            for (i, label) in mr.labels.iter().enumerate() {
                if i > 0 { label_spans.push(Span::raw(", ")); }
                let bg = hex_to_color(label.color());
                let fg = get_contrasting_text_color(label.color());
                label_spans.push(Span::styled(format!(" {} ", label.name()), field_style(MetadataField::Labels).bg(bg).fg(fg)));
            }
        }
        lines.push(Line::from(label_spans));

        let mut assign_spans = vec![Span::raw(" Assign: ")];
        if mr.assignees.is_empty() { assign_spans.push(Span::styled("None", field_style(MetadataField::Assignees))); }
        else {
            for (i, a) in mr.assignees.iter().enumerate() {
                if i > 0 { assign_spans.push(Span::raw(", ")); }
                assign_spans.push(Span::styled(format!("@{}", a.username), field_style(MetadataField::Assignees).fg(app.theme.author)));
            }
        }
        lines.push(Line::from(assign_spans));

        let mut reviewer_spans = vec![Span::raw(" Review: ")];
        if mr.reviewers.is_empty() { reviewer_spans.push(Span::styled("None", field_style(MetadataField::Reviewers))); }
        else {
            for (i, a) in mr.reviewers.iter().enumerate() {
                if i > 0 { reviewer_spans.push(Span::raw(", ")); }
                reviewer_spans.push(Span::styled(format!("@{}", a.username), field_style(MetadataField::Reviewers).fg(app.theme.author)));
            }
        }
        lines.push(Line::from(reviewer_spans));

        lines.push(Line::from(vec![Span::raw(" Miles: "), Span::styled(mr.milestone.as_ref().map(|m| m.title.as_str()).unwrap_or("None"), field_style(MetadataField::Milestone))]));
        lines.push(Line::from(vec![Span::raw(" Squash: "), Span::raw(if mr.squash.unwrap_or(false) { "Yes" } else { "No" })]));

        use crate::ui::widgets::get_status_style;
        let pipeline_status = if let Some(pipeline) = &mr.head_pipeline {
            Span::styled(&pipeline.status, get_status_style(&pipeline.status, &app.theme))
        } else {
            Span::raw("None")
        };
        lines.push(Line::from(vec![Span::raw(" Pipeline: "), pipeline_status]));

        let detailed_status = mr.detailed_merge_status.as_deref().unwrap_or("unknown");
        let status_color = match detailed_status {
            "mergeable" => app.theme.success,
            "conflict" => app.theme.failed,
            _ => app.theme.pending,
        };
        lines.push(Line::from(vec![
            Span::raw(" Status: "),
            Span::styled(detailed_status, Style::default().fg(status_color)),
        ]));

        let mut participant_spans = vec![Span::raw(" Participants: ")];
        if mr.participants.is_empty() {
            participant_spans.push(Span::styled("None", Style::default().fg(app.theme.dim)));
        } else {
            for (i, p) in mr.participants.iter().enumerate() {
                if i > 0 { participant_spans.push(Span::raw(", ")); }
                participant_spans.push(Span::styled(format!("@{}", p.username), Style::default().fg(app.theme.author)));
            }
        }
        lines.push(Line::from(participant_spans));
    } else if let Some(issue) = &app.selected_issue {
        use crate::ui::widgets::get_status_style;
        lines.push(Line::from(vec![
            Span::raw(" Status: "),
            Span::styled(&issue.state, get_status_style(&issue.state, &app.theme)),
        ]));

        let title = if is_meta_focused && app.selected_metadata_field == MetadataField::Title && app.input_mode { app.textarea.lines().first().cloned().unwrap_or_default() } else { issue.title.clone() };
        lines.push(Line::from(vec![Span::raw(" Title: "), Span::styled(title, field_style(MetadataField::Title))]));

        let mut label_spans = vec![Span::raw(" Labels: ")];
        if issue.labels.is_empty() { label_spans.push(Span::styled("None", field_style(MetadataField::Labels))); }
        else {
            use crate::ui::widgets::{hex_to_color, get_contrasting_text_color};
            for (i, label) in issue.labels.iter().enumerate() {
                if i > 0 { label_spans.push(Span::raw(", ")); }
                let bg = hex_to_color(label.color());
                let fg = get_contrasting_text_color(label.color());
                label_spans.push(Span::styled(format!(" {} ", label.name()), field_style(MetadataField::Labels).bg(bg).fg(fg)));
            }
        }
        lines.push(Line::from(label_spans));

        let mut assign_spans = vec![Span::raw(" Assign: ")];
        if issue.assignees.is_empty() { assign_spans.push(Span::styled("None", field_style(MetadataField::Assignees))); }
        else {
            for (i, a) in issue.assignees.iter().enumerate() {
                if i > 0 { assign_spans.push(Span::raw(", ")); }
                assign_spans.push(Span::styled(format!("@{}", a.username), field_style(MetadataField::Assignees).fg(app.theme.author)));
            }
        }
        lines.push(Line::from(assign_spans));

        lines.push(Line::from(vec![Span::raw(" Miles: "), Span::styled(issue.milestone.as_ref().map(|m| m.title.as_str()).unwrap_or("None"), field_style(MetadataField::Milestone))]));
        
        let start_date_str = issue.start_date.as_deref().unwrap_or("None");
        lines.push(Line::from(vec![Span::raw(" Start Date: "), Span::styled(start_date_str, field_style(MetadataField::StartDate))]));

        let due_date_str = issue.due_date.as_deref().unwrap_or("None");
        lines.push(Line::from(vec![Span::raw(" Due Date: "), Span::styled(due_date_str, field_style(MetadataField::DueDate))]));

        lines.push(Line::from(vec![Span::raw(" Confidential: "), Span::styled(if issue.confidential { "Yes" } else { "No" }, field_style(MetadataField::Confidential))]));

        let mut participant_spans = vec![Span::raw(" Participants: ")];
        if issue.participants.is_empty() {
            participant_spans.push(Span::styled("None", Style::default().fg(app.theme.dim)));
        } else {
            for (i, p) in issue.participants.iter().enumerate() {
                if i > 0 { participant_spans.push(Span::raw(", ")); }
                participant_spans.push(Span::styled(format!("@{}", p.username), Style::default().fg(app.theme.author)));
            }
        }
        lines.push(Line::from(participant_spans));
    } else if let Some(pipeline) = &app.selected_pipeline {
        lines.push(Line::from(vec![Span::raw(" Pipeline: "), Span::styled(format!("#{}", pipeline.id), field_style(MetadataField::Title))]));
        lines.push(Line::from(vec![Span::raw(" Status: "), Span::styled(&pipeline.status, crate::ui::widgets::get_status_style(&pipeline.status, &app.theme))]));
        lines.push(Line::from(vec![Span::raw(" Ref: "), Span::raw(&pipeline.ref_name)]));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}
