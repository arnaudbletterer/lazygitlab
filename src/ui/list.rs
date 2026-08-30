use crate::models::{
    App, Context, DetailSection, GroupedItem, IssueFilter, MenuSelection, MergeRequestFilter,
    Panel, PipelineFilter, TodoFilter,
};
use crate::ui::Theme;
use crate::ui::detail::render_detail_view;
use ratatui::{prelude::*, widgets::*};

pub fn render_list_view(f: &mut Frame, app: &mut App, area: Rect) {
    let is_item_selected = app.selected_mr.is_some()
        || app.selected_issue.is_some()
        || app.selected_pipeline.is_some();

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
            Constraint::Length(3), // Status (Top)
            Constraint::Length(5), // Dashboard (3 items + title/border)
            Constraint::Length(4), // Explore (2 items + title/border)
            if app.selected_project.is_some() {
                Constraint::Min(10)
            } else {
                Constraint::Length(0)
            },
        ])
        .split(chunks[0]);

    // 1. Modern Status Header Card
    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " 🦊 GitLab ",
            Style::default().fg(app.theme.accent).bold(),
        ))
        .border_style(Style::default().fg(app.theme.dim));

    let status_lines = if let Some(project) = &app.selected_project {
        vec![
            Line::from(vec![
                Span::styled(" 📂 ", Style::default().fg(app.theme.project)),
                Span::styled(
                    crate::ui::widgets::truncate_str(&project.name, 18),
                    Style::default().fg(app.theme.title_fg).bold(),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    format!(" 🌿 {}", project.default_branch.as_deref().unwrap_or("main")),
                    Style::default().fg(app.theme.dim),
                ),
                Span::styled(" ● Live", Style::default().fg(app.theme.status_success)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled(" 🏠 ", Style::default().fg(app.theme.accent)),
                Span::styled("Personal Dashboard", Style::default().fg(app.theme.title_fg).bold()),
            ]),
            Line::from(vec![
                Span::styled(" Global View", Style::default().fg(app.theme.dim)),
                Span::styled(" ● Ready", Style::default().fg(app.theme.status_success)),
            ]),
        ]
    };
    f.render_widget(Paragraph::new(status_lines).block(status_block), side_chunks[0]);

    // 2. Modern Navigation Panels
    let todo_badge = if !app.todos.is_empty() {
        format!(" ({})", app.todos.len())
    } else {
        String::new()
    };
    let mr_badge = if !app.merge_requests.is_empty() {
        format!(" ({})", app.merge_requests.len())
    } else {
        String::new()
    };
    let issue_badge = if !app.issues.is_empty() {
        format!(" ({})", app.issues.len())
    } else {
        String::new()
    };

    app.set_rect("nav_dashboard", side_chunks[1]);
    render_nav_panel(
        f,
        app.active_panel == Panel::Dashboard,
        &app.theme,
        side_chunks[1],
        "🏠 Dashboard",
        &mut app.dashboard_menu_state,
        vec![
            format!("  › To-Dos{}", todo_badge),
            format!("  › Merge Requests{}", mr_badge),
            format!("  › Issues{}", issue_badge),
            "  › Pipelines".to_string(),
        ],
    );

    app.set_rect("nav_explore", side_chunks[2]);
    render_nav_panel(
        f,
        app.active_panel == Panel::Explore,
        &app.theme,
        side_chunks[2],
        "🌐 Explore",
        &mut app.explore_menu_state,
        vec!["  › Projects".to_string(), "  › Groups".to_string()],
    );

    if app.selected_project.is_some() {
        let has_epics = app
            .selected_project
            .as_ref()
            .and_then(|p| p.namespace.as_ref())
            .is_some_and(|ns| ns.kind == "group");
        app.set_rect("nav_project", side_chunks[3]);
        let mut project_items = vec![
            "  › Merge Requests".to_string(),
            "  › Issues".to_string(),
            "  › Pipelines".to_string(),
            "  › Repository".to_string(),
            "  › Branches".to_string(),
            "  › Commits".to_string(),
            "  › Boards".to_string(),
        ];
        if has_epics {
            project_items.push("  › Epics".to_string());
        }
        render_nav_panel(
            f,
            app.active_panel == Panel::Project,
            &app.theme,
            side_chunks[3],
            "📂 Project",
            &mut app.project_menu_state,
            project_items,
        );
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

fn render_nav_panel(
    f: &mut Frame,
    is_focused: bool,
    theme: &Theme,
    area: Rect,
    title: &str,
    state: &mut ListState,
    items: Vec<String>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            format!(" {} ", title),
            if is_focused {
                Style::default().fg(theme.accent).bold()
            } else {
                Style::default().fg(theme.title_fg)
            },
        ))
        .border_style(if is_focused {
            Style::default().fg(theme.active_border)
        } else {
            Style::default().fg(theme.inactive_border)
        });

    let list_items: Vec<ListItem> = items.into_iter().map(ListItem::new).collect();
    f.render_stateful_widget(
        List::new(list_items)
            .block(block)
            .highlight_style(if is_focused {
                Style::default()
                    .bg(theme.highlight_bg)
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
            .highlight_symbol(if is_focused { "▎ " } else { "  " }),
        area,
        state,
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
        MenuSelection::Repository => match &app.repository_ref {
            Some(r) => format!(" Repository [{}] ", r),
            None => " Repository ".to_string(),
        },
        MenuSelection::Branches => " Branches ".to_string(),
        MenuSelection::Commits => " Commits ".to_string(),
        MenuSelection::Boards => " Boards ".to_string(),
        MenuSelection::Epics => match &app.selected_epic {
            Some(epic) => format!(
                " Epic: {} ",
                crate::ui::widgets::truncate_str(&epic.title, 40)
            ),
            None => " Epics ".to_string(),
        },
    };
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            list_title.as_str(),
            if is_main_focused {
                Style::default().fg(app.theme.accent).bold()
            } else {
                Style::default().fg(app.theme.title_fg)
            },
        ))
        .border_style(if is_main_focused {
            Style::default().fg(app.theme.active_border)
        } else {
            Style::default().fg(app.theme.inactive_border)
        });

    let list_inner = list_block.inner(area);
    f.render_widget(list_block, area);

    let (titles, selected) = match app.active_selection {
        MenuSelection::Todos => (
            vec!["Pending", "Snoozed", "Done"],
            Some(match app.todo_filter {
                TodoFilter::Pending => 0,
                TodoFilter::Snoozed => 1,
                TodoFilter::Done => 2,
            }),
        ),
        MenuSelection::MergeRequests => (
            vec!["Opened", "Review", "Assigned", "Merged", "Closed", "All"],
            Some(match app.mr_filter {
                MergeRequestFilter::Opened => 0,
                MergeRequestFilter::Review => 1,
                MergeRequestFilter::Assigned => 2,
                MergeRequestFilter::Merged => 3,
                MergeRequestFilter::Closed => 4,
                MergeRequestFilter::All => 5,
            }),
        ),
        MenuSelection::Issues => (
            vec!["Opened", "Closed", "All"],
            Some(match app.issue_filter {
                IssueFilter::Opened => 0,
                IssueFilter::Closed => 1,
                IssueFilter::All => 2,
            }),
        ),
        MenuSelection::Pipelines => (
            vec!["All", "Running", "Failed", "Passed"],
            Some(match app.pipeline_filter {
                PipelineFilter::All => 0,
                PipelineFilter::Running => 1,
                PipelineFilter::Failed => 2,
                PipelineFilter::Passed => 3,
            }),
        ),
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
            .highlight_style(
                Style::default()
                    .fg(app.theme.border)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(tabs, list_chunks[0]);
    }

    let highlight_style = Style::default()
        .bg(app.theme.highlight_bg)
        .add_modifier(Modifier::BOLD);
    let no_highlight = Style::default();

    match app.active_selection {
        MenuSelection::Todos => {
            let items: Vec<ListItem> = app
                .filtered_todos
                .iter()
                .map(|item| match item {
                    GroupedItem::Item(t) => {
                        let action_style = Style::default().fg(app.theme.accent).bold();
                        let target_icon = if t.target_type == "MergeRequest" { "🔀 " } else { "📋 " };
                        let time_str = crate::ui::widgets::format_relative_time(&t.created_at);
                        ListItem::new(Line::from(vec![
                            Span::styled(target_icon, Style::default().fg(app.theme.dim)),
                            Span::styled(format!("[{}] ", t.action_name.replace('_', " ")), action_style),
                            Span::styled(crate::ui::widgets::truncate_str(&t.body, 50), Style::default().fg(app.theme.title_fg)),
                            Span::styled(format!(" ({})", time_str), Style::default().fg(app.theme.dim)),
                        ]))
                    }
                    GroupedItem::Header(h) => {
                        let key = format!("todos:{}", h);
                        let icon = if app.collapsed_groups.contains(&key) {
                            "▶"
                        } else {
                            "▼"
                        };
                        ListItem::new(Line::from(vec![Span::styled(
                            format!("{} {}", icon, h),
                            Style::default()
                                .fg(app.theme.header)
                                .add_modifier(Modifier::BOLD),
                        )]))
                    }
                })
                .collect();
            if items.is_empty() {
                f.render_widget(
                    Paragraph::new("No todos match this view or filter.")
                        .style(Style::default().fg(app.theme.dim)),
                    list_chunks[1],
                );
            } else {
                f.render_stateful_widget(
                    List::new(items)
                        .highlight_style(if is_main_focused {
                            highlight_style
                        } else {
                            no_highlight
                        })
                        .highlight_symbol(if is_main_focused { "▎ " } else { "  " }),
                    list_chunks[1],
                    &mut app.todos_state,
                );
            }
        }
        MenuSelection::MergeRequests => {
            let items: Vec<ListItem> = app
                .filtered_mrs
                .iter()
                .map(|item| match item {
                    GroupedItem::Item(mr) => {
                        use crate::ui::widgets::{get_contrasting_text_color, hex_to_color};
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

                        // State Icon badge
                        let (icon, state_style) = if mr.draft.unwrap_or(false) {
                            ("✎ ", Style::default().fg(app.theme.dim))
                        } else if mr.state == "merged" {
                            ("✔ ", Style::default().fg(app.theme.merged_mr))
                        } else if mr.state == "closed" {
                            ("✖ ", Style::default().fg(app.theme.closed_mr))
                        } else {
                            ("✦ ", Style::default().fg(app.theme.open_mr))
                        };
                        spans.push(Span::styled(icon, state_style));

                        spans.push(Span::styled(
                            format!("!{} ", mr.iid),
                            Style::default().fg(app.theme.accent).bold(),
                        ));

                        let max_title = (area.width as usize).saturating_sub(28).max(16);
                        spans.push(Span::styled(
                            crate::ui::widgets::truncate_str(&mr.title, max_title),
                            Style::default().fg(app.theme.title_fg).bold(),
                        ));

                        if let Some(up) = mr.upvotes && up > 0 {
                            spans.push(Span::styled(format!(" 👍{}", up), Style::default().fg(app.theme.status_success)));
                        }

                        spans.push(Span::raw(" "));
                        for label in &mr.labels {
                            let bg = hex_to_color(label.color());
                            let fg = get_contrasting_text_color(label.color());
                            spans.push(Span::styled(
                                format!(" {} ", label.name()),
                                Style::default().bg(bg).fg(fg),
                            ));
                            spans.push(Span::raw(" "));
                        }
                        ListItem::new(Line::from(spans))
                    }
                    GroupedItem::Header(h) => {
                        let scope = match app.active_selection {
                            MenuSelection::MergeRequests => "mrs",
                            MenuSelection::Issues => "issues",
                            _ => "unknown",
                        };
                        let key = format!("{}:{}", scope, h);
                        let icon = if app.collapsed_groups.contains(&key) {
                            "▶"
                        } else {
                            "▼"
                        };
                        ListItem::new(Line::from(vec![Span::styled(
                            format!("{} {}", icon, h),
                            Style::default()
                                .fg(app.theme.header)
                                .add_modifier(Modifier::BOLD),
                        )]))
                    }
                })
                .collect();
            if items.is_empty() {
                f.render_widget(
                    Paragraph::new("No merge requests match this view or filter.")
                        .style(Style::default().fg(app.theme.dim)),
                    list_chunks[1],
                );
            } else {
                f.render_stateful_widget(
                    List::new(items)
                        .highlight_style(if is_main_focused {
                            highlight_style
                        } else {
                            no_highlight
                        })
                        .highlight_symbol(if is_main_focused { "▎ " } else { "  " }),
                    list_chunks[1],
                    &mut app.mrs_state,
                );
            }
        }
        MenuSelection::Issues => {
            let items: Vec<ListItem> = app
                .filtered_issues
                .iter()
                .map(|item| match item {
                    GroupedItem::Item(i) => {
                        use crate::ui::widgets::{get_contrasting_text_color, hex_to_color};
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

                        let (icon, state_style) = if i.state == "closed" {
                            ("✔ ", Style::default().fg(app.theme.merged_mr))
                        } else {
                            ("✦ ", Style::default().fg(app.theme.status_success))
                        };
                        spans.push(Span::styled(icon, state_style));

                        spans.push(Span::styled(
                            format!("#{} ", i.iid),
                            Style::default().fg(app.theme.issue).bold(),
                        ));

                        let max_title = (area.width as usize).saturating_sub(32).max(16);
                        spans.push(Span::styled(
                            crate::ui::widgets::truncate_str(&i.title, max_title),
                            Style::default().fg(app.theme.title_fg).bold(),
                        ));
                        spans.push(Span::raw(" "));

                        if let Some(m) = &i.milestone {
                            spans.push(Span::styled(
                                format!(" ⚑ {} ", m.title),
                                Style::default()
                                    .fg(app.theme.project)
                                    .bg(app.theme.chip_bg),
                            ));
                            spans.push(Span::raw(" "));
                        }
                        if let Some(due) = &i.due_date {
                            let overdue = chrono::NaiveDate::parse_from_str(due, "%Y-%m-%d")
                                .map(|d| d < chrono::Local::now().date_naive())
                                .unwrap_or(false);
                            let due_style = if overdue {
                                Style::default()
                                    .fg(app.theme.failed)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(app.theme.dim)
                            };
                            spans.push(Span::styled(format!(" 📅 due {} ", due), due_style));
                            spans.push(Span::raw(" "));
                        }
                        for label in &i.labels {
                            let bg = hex_to_color(label.color());
                            let fg = get_contrasting_text_color(label.color());
                            spans.push(Span::styled(
                                format!(" {} ", label.name()),
                                Style::default().bg(bg).fg(fg),
                            ));
                            spans.push(Span::raw(" "));
                        }
                        ListItem::new(Line::from(spans))
                    }
                    GroupedItem::Header(h) => {
                        let scope = match app.active_selection {
                            MenuSelection::MergeRequests => "mrs",
                            MenuSelection::Issues => "issues",
                            _ => "unknown",
                        };
                        let key = format!("{}:{}", scope, h);
                        let icon = if app.collapsed_groups.contains(&key) {
                            "▶"
                        } else {
                            "▼"
                        };
                        ListItem::new(Line::from(vec![Span::styled(
                            format!("{} {}", icon, h),
                            Style::default()
                                .fg(app.theme.header)
                                .add_modifier(Modifier::BOLD),
                        )]))
                    }
                })
                .collect();
            if items.is_empty() {
                f.render_widget(
                    Paragraph::new("No issues match this view or filter.")
                        .style(Style::default().fg(app.theme.dim)),
                    list_chunks[1],
                );
            } else {
                f.render_stateful_widget(
                    List::new(items)
                        .highlight_style(if is_main_focused {
                            highlight_style
                        } else {
                            no_highlight
                        })
                        .highlight_symbol(if is_main_focused { "▎ " } else { "  " }),
                    list_chunks[1],
                    &mut app.issues_state,
                );
            }
        }
        MenuSelection::Pipelines => {
            let items: Vec<ListItem> = app
                .filtered_pipelines
                .iter()
                .map(|p| {
                    let (icon, status_style) = match p.status.as_str() {
                        "success" | "passed" => ("✔", Style::default().fg(app.theme.status_success).bold()),
                        "failed" => ("✖", Style::default().fg(app.theme.failed).bold()),
                        "running" => ("⟳", Style::default().fg(app.theme.running).bold()),
                        "pending" => ("⏱", Style::default().fg(app.theme.pending).bold()),
                        _ => ("⭘", Style::default().fg(app.theme.dim)),
                    };
                    let mut spans = vec![
                        Span::styled(format!(" {} {} ", icon, p.status), status_style),
                        Span::styled(format!("#{} ", p.id), Style::default().fg(app.theme.title_fg).bold()),
                    ];
                    if app.active_context != Context::Project {
                        let project_label = app
                            .project_cache
                            .get(&p.project_id)
                            .and_then(|proj| proj.path_with_namespace.clone())
                            .unwrap_or_else(|| format!("#{}", p.project_id));
                        spans.push(Span::styled(
                            format!(" 📂 {} ", crate::ui::widgets::truncate_str(&project_label, 20)),
                            Style::default().fg(app.theme.project),
                        ));
                    }
                    spans.push(Span::styled(
                        format!(" 🌿 {} ", p.ref_name),
                        Style::default().fg(app.theme.author),
                    ));
                    spans.push(Span::styled(
                        format!(" [{}] ", p.source),
                        Style::default().fg(app.theme.dim),
                    ));
                    if let Some(dur) = p.duration {
                        spans.push(Span::styled(
                            format!(" ⏱ {} ", crate::ui::widgets::format_duration(dur)),
                            Style::default().fg(app.theme.dim),
                        ));
                    }
                    ListItem::new(Line::from(spans))
                })
                .collect();
            if items.is_empty() {
                f.render_widget(
                    Paragraph::new("No pipelines match this view or filter.")
                        .style(Style::default().fg(app.theme.dim)),
                    list_chunks[1],
                );
            } else {
                f.render_stateful_widget(
                    List::new(items)
                        .highlight_style(if is_main_focused {
                            highlight_style
                        } else {
                            no_highlight
                        })
                        .highlight_symbol(if is_main_focused { "▎ " } else { "  " }),
                    list_chunks[1],
                    &mut app.pipelines_state,
                );
            }
        }
        MenuSelection::Projects => {
            let items: Vec<ListItem> = app
                .filtered_projects
                .iter()
                .map(|p| {
                    ListItem::new(Line::from(vec![
                        Span::styled(" 📂 ", Style::default().fg(app.theme.project)),
                        Span::styled(
                            format!("{} ", p.name),
                            Style::default().fg(app.theme.title_fg).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("({})", p.path_with_namespace.as_deref().unwrap_or("")),
                            Style::default().fg(app.theme.dim),
                        ),
                    ]))
                })
                .collect();
            f.render_stateful_widget(
                List::new(items)
                    .highlight_style(if is_main_focused {
                        highlight_style
                    } else {
                        no_highlight
                    })
                    .highlight_symbol(if is_main_focused { "▎ " } else { "  " }),
                list_chunks[1],
                &mut app.projects_state,
            );
        }
        MenuSelection::Groups => {
            let items: Vec<ListItem> = app
                .filtered_groups
                .iter()
                .map(|g| {
                    ListItem::new(Line::from(vec![
                        Span::styled(" 👥 ", Style::default().fg(app.theme.accent)),
                        Span::styled(
                            format!("{} ", g.name),
                            Style::default().fg(app.theme.title_fg).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("({})", g.full_path),
                            Style::default().fg(app.theme.dim),
                        ),
                    ]))
                })
                .collect();
            f.render_stateful_widget(
                List::new(items)
                    .highlight_style(if is_main_focused {
                        highlight_style
                    } else {
                        no_highlight
                    })
                    .highlight_symbol(if is_main_focused { "▎ " } else { "  " }),
                list_chunks[1],
                &mut app.groups_state,
            );
        }
        MenuSelection::Repository => {
            let mut tree_items: Vec<tui_tree_widget::TreeItem<'static, String>> = Vec::new();
            for item in &app.repository_tree {
                let text = if item.r#type == "tree" {
                    format!("📁 {}", item.name)
                } else {
                    format!("📄 {}", item.name)
                };
                tree_items.push(tui_tree_widget::TreeItem::new_leaf(item.path.clone(), text));
            }
            let tree = tui_tree_widget::Tree::new(&tree_items)
                .unwrap_or_else(|_| tui_tree_widget::Tree::new(&[]).unwrap())
                .block(
                    Block::default()
                        .title(" Tree ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(if is_main_focused {
                            Style::default().fg(app.theme.active_border)
                        } else {
                            Style::default().fg(app.theme.inactive_border)
                        }),
                )
                .highlight_style(if is_main_focused {
                    highlight_style
                } else {
                    no_highlight
                })
                .highlight_symbol(if is_main_focused { "▎ " } else { "  " });
            f.render_stateful_widget(tree, list_chunks[1], &mut app.repository_tree_state);
        }
        MenuSelection::Branches => {
            let items: Vec<ListItem> = app
                .filtered_branches
                .iter()
                .map(|b| {
                    let mut spans = vec![
                        Span::styled(" 🌿 ", Style::default().fg(app.theme.author)),
                        Span::styled(
                            format!("{} ", b.name),
                            Style::default().fg(app.theme.title_fg).add_modifier(Modifier::BOLD),
                        ),
                    ];
                    if b.default.unwrap_or(false) {
                        spans.push(Span::styled(" [Default] ", Style::default().fg(app.theme.status_success).bold()));
                    }
                    if b.merged.unwrap_or(false) {
                        spans.push(Span::styled(" [Merged] ", Style::default().fg(app.theme.dim)));
                    }
                    ListItem::new(Line::from(spans))
                })
                .collect();
            f.render_stateful_widget(
                List::new(items)
                    .highlight_style(if is_main_focused {
                        highlight_style
                    } else {
                        no_highlight
                    })
                    .highlight_symbol(if is_main_focused { "▎ " } else { "  " }),
                list_chunks[1],
                &mut app.branches_state,
            );
        }
        MenuSelection::Commits => {
            let items: Vec<ListItem> = app
                .commits
                .iter()
                .map(|c| {
                    ListItem::new(Line::from(vec![
                        Span::styled(" 📜 ", Style::default().fg(app.theme.dim)),
                        Span::styled(
                            format!("[{}] ", c.short_id),
                            Style::default().fg(app.theme.accent).bold(),
                        ),
                        Span::styled(
                            crate::ui::widgets::truncate_str(&c.title, (area.width as usize).saturating_sub(35).max(16)),
                            Style::default().fg(app.theme.title_fg).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!(" - @{}", c.author_name),
                            Style::default().fg(app.theme.dim),
                        ),
                    ]))
                })
                .collect();
            f.render_stateful_widget(
                List::new(items)
                    .highlight_style(if is_main_focused {
                        highlight_style
                    } else {
                        no_highlight
                    })
                    .highlight_symbol(if is_main_focused { "▎ " } else { "  " }),
                list_chunks[1],
                &mut app.commits_state,
            );
        }
        MenuSelection::Boards => {
            let items: Vec<ListItem> = app
                .project_boards
                .iter()
                .map(|b| {
                    ListItem::new(Line::from(vec![
                        Span::styled(" 📋 ", Style::default().fg(app.theme.accent)),
                        Span::styled(
                            format!("{} ", b.name),
                            Style::default().fg(app.theme.title_fg).add_modifier(Modifier::BOLD),
                        ),
                    ]))
                })
                .collect();
            f.render_stateful_widget(
                List::new(items)
                    .highlight_style(if is_main_focused {
                        highlight_style
                    } else {
                        no_highlight
                    })
                    .highlight_symbol(if is_main_focused { "▎ " } else { "  " }),
                list_chunks[1],
                &mut app.project_boards_state,
            );
        }
        MenuSelection::Epics => {
            if app.selected_epic.is_some() {
                let items: Vec<ListItem> = app
                    .filtered_epic_issues
                    .iter()
                    .map(|i| {
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!(" {} ", i.state),
                                crate::ui::widgets::get_status_style(&i.state, &app.theme),
                            ),
                            Span::styled(format!("#{} ", i.iid), Style::default().fg(app.theme.issue).bold()),
                            Span::styled(
                                crate::ui::widgets::truncate_str(
                                    &i.title,
                                    (area.width as usize).saturating_sub(24).max(16),
                                ),
                                Style::default().fg(app.theme.title_fg).add_modifier(Modifier::BOLD),
                            ),
                        ]))
                    })
                    .collect();
                if items.is_empty() {
                    f.render_widget(
                        Paragraph::new("No issues linked to this epic.")
                            .style(Style::default().fg(app.theme.dim)),
                        list_chunks[1],
                    );
                } else {
                    f.render_stateful_widget(
                        List::new(items)
                            .highlight_style(if is_main_focused {
                                highlight_style
                            } else {
                                no_highlight
                            })
                            .highlight_symbol(if is_main_focused { "▎ " } else { "  " }),
                        list_chunks[1],
                        &mut app.epic_issues_state,
                    );
                }
            } else {
                let items: Vec<ListItem> = app
                    .epics
                    .iter()
                    .map(|e| {
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("&{} ", e.iid),
                                Style::default().fg(app.theme.dim),
                            ),
                            Span::styled(
                                e.title.clone(),
                                Style::default().add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!(" [{}]", e.state),
                                crate::ui::widgets::get_status_style(&e.state, &app.theme),
                            ),
                        ]))
                    })
                    .collect();
                f.render_stateful_widget(
                    List::new(items)
                        .highlight_style(if is_main_focused {
                            highlight_style
                        } else {
                            no_highlight
                        })
                        .highlight_symbol(if is_main_focused { "> " } else { "  " }),
                    list_chunks[1],
                    &mut app.epics_state,
                );
            }
        }
    }
}

pub fn render_discussions_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let is_disc_focused =
        app.active_panel == Panel::MainDetail && app.detail_section == DetailSection::Discussions;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " 💬 Discussions (Threads) ",
            if is_disc_focused {
                Style::default().fg(app.theme.accent).bold()
            } else {
                Style::default().fg(app.theme.title_fg)
            },
        ))
        .border_style(if is_disc_focused {
            Style::default().fg(app.theme.active_border)
        } else {
            Style::default().fg(app.theme.inactive_border)
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    let is_replying =
        is_disc_focused && app.input_mode && app.input_target == crate::models::InputTarget::Reply;

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

    let comments: Vec<ListItem> = app
        .discussions
        .iter()
        .enumerate()
        .map(|(idx, d)| {
            let is_selected = is_disc_focused && app.discussion_state.selected() == Some(idx);
            let highlight = if is_selected {
                Style::default()
                    .bg(app.theme.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            if let Some(note) = d.notes.first() {
                let is_activity = note.system;
                let is_resolved = !is_activity && d.resolved.unwrap_or(false);

                let mut spans = vec![Span::styled(
                    if is_selected { "● " } else { "  " },
                    highlight.fg(app.theme.border),
                )];

                let text_style = if is_resolved || is_activity {
                    highlight.fg(app.theme.dim)
                } else {
                    highlight
                };

                if is_activity {
                    spans.push(Span::styled("[ACTIVITY] ", highlight.fg(app.theme.dim)));
                    spans.push(Span::styled(
                        format!("@{} ", note.author.username),
                        highlight.fg(app.theme.dim).add_modifier(Modifier::BOLD),
                    ));
                } else if let Some(resolved) = d.resolved {
                    if resolved {
                        spans.push(Span::styled("[RESOLVED] ", highlight.fg(app.theme.dim)));
                        spans.push(Span::styled(
                            format!("@{} ", note.author.username),
                            highlight.fg(app.theme.dim).add_modifier(Modifier::BOLD),
                        ));
                    } else {
                        spans.push(Span::styled(
                            "[UNRESOLVED] ",
                            highlight.fg(app.theme.pending),
                        ));
                        spans.push(Span::styled(
                            format!("@{} ", note.author.username),
                            highlight.fg(app.theme.author).add_modifier(Modifier::BOLD),
                        ));
                    }
                } else {
                    spans.push(Span::styled(
                        format!("@{} ", note.author.username),
                        highlight.fg(app.theme.author).add_modifier(Modifier::BOLD),
                    ));
                }

                spans.push(Span::styled(": ", text_style));

                // Show more content (up to 100 chars) for a better overview
                let body = note.body.replace('\n', " ");
                spans.push(Span::styled(
                    body.chars().take(100).collect::<String>(),
                    text_style,
                ));
                if body.len() > 100 {
                    spans.push(Span::styled("...", text_style));
                }
                ListItem::new(Line::from(spans))
            } else {
                ListItem::new(" (empty thread)")
            }
        })
        .collect();

    if comments.is_empty() {
        f.render_widget(
            Paragraph::new(" No discussions found.").fg(app.theme.dim),
            chunks[0],
        );
    } else {
        f.render_stateful_widget(List::new(comments), chunks[0], &mut app.discussion_state);
    }

    if is_replying {
        let reply_block = Block::default()
            .borders(Borders::ALL)
            .title(" Reply (Ctrl+Enter to send, Esc to cancel) ")
            .border_style(Style::default().fg(app.theme.border));
        f.render_widget(Clear, chunks[1]);
        app.textarea.set_block(reply_block);
        f.render_widget(&app.textarea, chunks[1]);
    }
}

pub fn render_jobs_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let is_jobs_focused = app.active_panel == Panel::MainDetail;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " 🚀 Pipeline Jobs ",
            if is_jobs_focused {
                Style::default().fg(app.theme.accent).bold()
            } else {
                Style::default().fg(app.theme.title_fg)
            },
        ))
        .border_style(if is_jobs_focused {
            Style::default().fg(app.theme.active_border)
        } else {
            Style::default().fg(app.theme.inactive_border)
        });

    let items: Vec<ListItem> = app
        .jobs
        .iter()
        .map(|j| {
            let (icon, status_style) = match j.status.as_str() {
                "success" | "passed" => ("✔", Style::default().fg(app.theme.status_success).bold()),
                "failed" => ("✖", Style::default().fg(app.theme.failed).bold()),
                "running" => ("⟳", Style::default().fg(app.theme.running).bold()),
                _ => ("⏱", Style::default().fg(app.theme.pending)),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} {} ", icon, j.status), status_style),
                Span::styled(format!("{} ", j.name), Style::default().fg(app.theme.title_fg).bold()),
                Span::styled(format!("({})", j.stage), Style::default().fg(app.theme.dim)),
            ]))
        })
        .collect();

    if items.is_empty() {
        f.render_widget(
            Paragraph::new(" No jobs found for this pipeline.")
                .fg(app.theme.dim)
                .block(block),
            area,
        );
    } else {
        f.render_stateful_widget(
            List::new(items)
                .block(block)
                .highlight_style(if is_jobs_focused {
                    Style::default()
                        .bg(app.theme.highlight_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                })
                .highlight_symbol(if is_jobs_focused { "▎ " } else { "  " }),
            area,
            &mut app.jobs_state,
        );
    }
}

pub fn render_metadata_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let is_meta_focused =
        app.active_panel == Panel::MainDetail && app.detail_section == DetailSection::Metadata;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " 📋 Metadata ",
            if is_meta_focused {
                Style::default().fg(app.theme.accent).bold()
            } else {
                Style::default().fg(app.theme.title_fg)
            },
        ))
        .border_style(if is_meta_focused {
            Style::default().fg(app.theme.active_border)
        } else {
            Style::default().fg(app.theme.inactive_border)
        });

    let mut lines = Vec::new();
    let highlight = Style::default()
        .bg(app.theme.highlight_bg)
        .add_modifier(Modifier::BOLD);
    let field_style = |f| {
        if is_meta_focused && app.selected_metadata_field == f {
            highlight
        } else {
            Style::default()
        }
    };

    use crate::models::MetadataField;

    if let Some(mr) = &app.selected_mr {
        let title = if is_meta_focused
            && app.selected_metadata_field == MetadataField::Title
            && app.input_mode
        {
            app.textarea.lines().first().cloned().unwrap_or_default()
        } else {
            mr.title.clone()
        };
        lines.push(Line::from(vec![
            Span::raw(" Title: "),
            Span::styled(title, field_style(MetadataField::Title)),
        ]));

        let target_branch_style =
            if is_meta_focused && app.selected_metadata_field == MetadataField::TargetBranch {
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
        if mr.labels.is_empty() {
            label_spans.push(Span::styled("None", field_style(MetadataField::Labels)));
        } else {
            use crate::ui::widgets::{get_contrasting_text_color, hex_to_color};
            for (i, label) in mr.labels.iter().enumerate() {
                if i > 0 {
                    label_spans.push(Span::raw(", "));
                }
                let bg = hex_to_color(label.color());
                let fg = get_contrasting_text_color(label.color());
                label_spans.push(Span::styled(
                    format!(" {} ", label.name()),
                    field_style(MetadataField::Labels).bg(bg).fg(fg),
                ));
            }
        }
        lines.push(Line::from(label_spans));

        let mut assign_spans = vec![Span::raw(" Assign: ")];
        if mr.assignees.is_empty() {
            assign_spans.push(Span::styled("None", field_style(MetadataField::Assignees)));
        } else {
            for (i, a) in mr.assignees.iter().enumerate() {
                if i > 0 {
                    assign_spans.push(Span::raw(", "));
                }
                assign_spans.push(Span::styled(
                    format!("@{}", a.username),
                    field_style(MetadataField::Assignees).fg(app.theme.author),
                ));
            }
        }
        lines.push(Line::from(assign_spans));

        let mut reviewer_spans = vec![Span::raw(" Review: ")];
        if mr.reviewers.is_empty() {
            reviewer_spans.push(Span::styled("None", field_style(MetadataField::Reviewers)));
        } else {
            for (i, a) in mr.reviewers.iter().enumerate() {
                if i > 0 {
                    reviewer_spans.push(Span::raw(", "));
                }
                reviewer_spans.push(Span::styled(
                    format!("@{}", a.username),
                    field_style(MetadataField::Reviewers).fg(app.theme.author),
                ));
            }
        }
        lines.push(Line::from(reviewer_spans));

        lines.push(Line::from(vec![
            Span::raw(" Miles: "),
            Span::styled(
                mr.milestone
                    .as_ref()
                    .map(|m| m.title.as_str())
                    .unwrap_or("None"),
                field_style(MetadataField::Milestone),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw(" Squash: "),
            Span::raw(if mr.squash.unwrap_or(false) {
                "Yes"
            } else {
                "No"
            }),
        ]));

        use crate::ui::widgets::get_status_style;
        let pipeline_status = if let Some(pipeline) = &mr.head_pipeline {
            Span::styled(
                &pipeline.status,
                get_status_style(&pipeline.status, &app.theme),
            )
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
                if i > 0 {
                    participant_spans.push(Span::raw(", "));
                }
                participant_spans.push(Span::styled(
                    format!("@{}", p.username),
                    Style::default().fg(app.theme.author),
                ));
            }
        }
        lines.push(Line::from(participant_spans));

        let mut related_mr_spans = vec![Span::raw(" Rel. MRs: ")];
        if app.related_mrs.is_empty() {
            related_mr_spans.push(Span::styled("None", Style::default().fg(app.theme.dim)));
        } else {
            for (i, r) in app.related_mrs.iter().enumerate() {
                if i > 0 {
                    related_mr_spans.push(Span::raw(", "));
                }
                related_mr_spans.push(Span::styled(
                    format!("!{} {}", r.iid, r.title),
                    Style::default().fg(app.theme.author),
                ));
                related_mr_spans.push(Span::styled(
                    format!(" [{}]", r.state),
                    crate::ui::widgets::get_status_style(&r.state, &app.theme),
                ));
            }
        }
        lines.push(Line::from(related_mr_spans));

        let mut closes_spans = vec![Span::raw(" Closes: ")];
        if app.mr_closing_issues.is_empty() {
            closes_spans.push(Span::styled("None", Style::default().fg(app.theme.dim)));
        } else {
            for (i, iss) in app.mr_closing_issues.iter().enumerate() {
                if i > 0 {
                    closes_spans.push(Span::raw(", "));
                }
                closes_spans.push(Span::styled(
                    format!("#{} {}", iss.iid, iss.title),
                    Style::default().fg(app.theme.author),
                ));
                closes_spans.push(Span::styled(
                    format!(" [{}]", iss.state),
                    crate::ui::widgets::get_status_style(&iss.state, &app.theme),
                ));
            }
        }
        lines.push(Line::from(closes_spans));
    } else if let Some(issue) = &app.selected_issue {
        use crate::ui::widgets::get_status_style;
        lines.push(Line::from(vec![
            Span::raw(" Status: "),
            Span::styled(&issue.state, get_status_style(&issue.state, &app.theme)),
        ]));

        let title = if is_meta_focused
            && app.selected_metadata_field == MetadataField::Title
            && app.input_mode
        {
            app.textarea.lines().first().cloned().unwrap_or_default()
        } else {
            issue.title.clone()
        };
        lines.push(Line::from(vec![
            Span::raw(" Title: "),
            Span::styled(title, field_style(MetadataField::Title)),
        ]));

        let mut label_spans = vec![Span::raw(" Labels: ")];
        if issue.labels.is_empty() {
            label_spans.push(Span::styled("None", field_style(MetadataField::Labels)));
        } else {
            use crate::ui::widgets::{get_contrasting_text_color, hex_to_color};
            for (i, label) in issue.labels.iter().enumerate() {
                if i > 0 {
                    label_spans.push(Span::raw(", "));
                }
                let bg = hex_to_color(label.color());
                let fg = get_contrasting_text_color(label.color());
                label_spans.push(Span::styled(
                    format!(" {} ", label.name()),
                    field_style(MetadataField::Labels).bg(bg).fg(fg),
                ));
            }
        }
        lines.push(Line::from(label_spans));

        let mut assign_spans = vec![Span::raw(" Assign: ")];
        if issue.assignees.is_empty() {
            assign_spans.push(Span::styled("None", field_style(MetadataField::Assignees)));
        } else {
            for (i, a) in issue.assignees.iter().enumerate() {
                if i > 0 {
                    assign_spans.push(Span::raw(", "));
                }
                assign_spans.push(Span::styled(
                    format!("@{}", a.username),
                    field_style(MetadataField::Assignees).fg(app.theme.author),
                ));
            }
        }
        lines.push(Line::from(assign_spans));

        lines.push(Line::from(vec![
            Span::raw(" Miles: "),
            Span::styled(
                issue
                    .milestone
                    .as_ref()
                    .map(|m| m.title.as_str())
                    .unwrap_or("None"),
                field_style(MetadataField::Milestone),
            ),
        ]));

        let start_date_str = issue.start_date.as_deref().unwrap_or("None");
        lines.push(Line::from(vec![
            Span::raw(" Start Date: "),
            Span::styled(start_date_str, field_style(MetadataField::StartDate)),
        ]));

        let due_date_str = issue.due_date.as_deref().unwrap_or("None");
        let is_overdue = issue
            .due_date
            .as_deref()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .map(|d| d < chrono::Local::now().date_naive())
            .unwrap_or(false);
        let due_style = if is_overdue {
            Style::default()
                .fg(app.theme.failed)
                .add_modifier(Modifier::BOLD)
        } else {
            field_style(MetadataField::DueDate)
        };
        let due_label = if is_overdue { " (overdue)" } else { "" };
        lines.push(Line::from(vec![
            Span::raw(" Due Date: "),
            Span::styled(format!("{}{}", due_date_str, due_label), due_style),
        ]));

        lines.push(Line::from(vec![
            Span::raw(" Confidential: "),
            Span::styled(
                if issue.confidential { "Yes" } else { "No" },
                field_style(MetadataField::Confidential),
            ),
        ]));

        let mut participant_spans = vec![Span::raw(" Participants: ")];
        if issue.participants.is_empty() {
            participant_spans.push(Span::styled("None", Style::default().fg(app.theme.dim)));
        } else {
            for (i, p) in issue.participants.iter().enumerate() {
                if i > 0 {
                    participant_spans.push(Span::raw(", "));
                }
                participant_spans.push(Span::styled(
                    format!("@{}", p.username),
                    Style::default().fg(app.theme.author),
                ));
            }
        }
        lines.push(Line::from(participant_spans));

        let mut links_spans = vec![Span::raw(" Links: ")];
        if app.linked_issues.is_empty() {
            links_spans.push(Span::styled("None", Style::default().fg(app.theme.dim)));
        } else {
            for (i, l) in app.linked_issues.iter().enumerate() {
                if i > 0 {
                    links_spans.push(Span::raw(", "));
                }
                links_spans.push(Span::styled(
                    format!("#{} {}", l.iid, l.title),
                    Style::default().fg(app.theme.author),
                ));
                links_spans.push(Span::styled(
                    format!(" [{}]", l.state),
                    crate::ui::widgets::get_status_style(&l.state, &app.theme),
                ));
            }
        }
        lines.push(Line::from(links_spans));

        let mut closed_by_spans = vec![Span::raw(" Closed By: ")];
        if app.issue_closing_mrs.is_empty() {
            closed_by_spans.push(Span::styled("None", Style::default().fg(app.theme.dim)));
        } else {
            for (i, m) in app.issue_closing_mrs.iter().enumerate() {
                if i > 0 {
                    closed_by_spans.push(Span::raw(", "));
                }
                closed_by_spans.push(Span::styled(
                    format!("!{} {}", m.iid, m.title),
                    Style::default().fg(app.theme.author),
                ));
                closed_by_spans.push(Span::styled(
                    format!(" [{}]", m.state),
                    crate::ui::widgets::get_status_style(&m.state, &app.theme),
                ));
            }
        }
        lines.push(Line::from(closed_by_spans));
    } else if let Some(pipeline) = &app.selected_pipeline {
        lines.push(Line::from(vec![
            Span::raw(" Pipeline: "),
            Span::styled(
                format!("#{}", pipeline.id),
                field_style(MetadataField::Title),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw(" Status: "),
            Span::styled(
                &pipeline.status,
                crate::ui::widgets::get_status_style(&pipeline.status, &app.theme),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw(" Ref: "),
            Span::raw(&pipeline.ref_name),
        ]));
    }

    f.render_widget(Paragraph::new(lines).block(block), area);
}
