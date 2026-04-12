use ratatui::{prelude::*, widgets::*};
use crate::models::{App, ViewMode, MenuSelection, LineType};

pub struct Theme {
    pub border: Color,
    pub highlight_bg: Color,
    pub author: Color,
    pub dim: Color,
    pub addition: Color,
    pub removal: Color,
    pub opened: Color,
    pub merged: Color,
    pub closed: Color,
    pub shortcut_key: Color,
    pub shortcut_desc: Color,
    pub unchanged: Color,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            border: Color::Green,
            highlight_bg: Color::Blue,
            author: Color::Magenta,
            dim: Color::Rgb(100, 100, 100),
            addition: Color::Green,
            removal: Color::Red,
            opened: Color::Red,
            merged: Color::Blue,
            closed: Color::Rgb(150, 150, 150),
            shortcut_key: Color::Cyan,
            shortcut_desc: Color::White,
            unchanged: Color::Gray,
        }
    }
}

pub fn get_status_style(status: &str, theme: &Theme) -> Style {
    match status {
        "opened" | "open" => Style::default().fg(theme.opened),
        "merged" | "resolved" => Style::default().fg(theme.merged),
        "closed" => Style::default().fg(theme.closed),
        _ => Style::default(),
    }
}

fn render_shortcuts(f: &mut Frame, area: Rect, shortcuts: Vec<(&str, &str)>, theme: &Theme) {
    let mut spans = Vec::new();
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(", ", Style::default().fg(theme.shortcut_desc)));
        }
        spans.push(Span::styled(*key, Style::default().fg(theme.shortcut_key)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*desc, Style::default().fg(theme.shortcut_desc)));
    }
    let p = Paragraph::new(Line::from(spans));
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Color::Rgb(r, g, b)
    } else {
        Color::Gray
    }
}

pub fn ui(f: &mut Frame, app: &mut App) {
    match app.view_mode {
        ViewMode::List => {
            render_list_view(f, app);
        }
        ViewMode::Detail => {
            render_detail_view(f, app);
        }
    }

    if app.show_project_popup {
        render_project_popup(f, app);
    }
}

fn render_project_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 40, f.area());
    
    // Clear the background
    let clear = Block::default().bg(Color::Black);
    f.render_widget(Clear, area);
    f.render_widget(clear, area);

    let projects: Vec<ListItem> = app.projects
        .iter()
        .map(|p| ListItem::new(format!("{} ({})", p.name, p.path_with_namespace)))
        .collect();

    let list = List::new(projects)
        .block(Block::default().borders(Borders::ALL).title(" Select Project ").border_style(Style::default().fg(app.theme.border)))
        .highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.projects_state);
}

fn render_list_view(f: &mut Frame, app: &mut App) {
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

    // Status Panel (Top Left)
    let proj_name = app.selected_project.as_ref().map(|p| p.name.as_str()).unwrap_or("None");
    let status_info = Paragraph::new(format!("Project: {}", proj_name))
        .block(Block::default().borders(Borders::ALL).title(" Status ").border_style(Style::default().fg(app.theme.border)));
    f.render_widget(status_info, left_chunks[0]);

    // Sidebar (Menu)
    let items: Vec<ListItem> = app.menu_items
        .iter()
        .map(|(name, _)| ListItem::new(name.as_str()))
        .collect();

    let sidebar_style = if app.focus_on_menu {
        Style::default().fg(app.theme.border)
    } else {
        Style::default()
    };

    let sidebar_highlight_style = if app.focus_on_menu {
        Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Menu ").border_style(sidebar_style))
        .highlight_style(sidebar_highlight_style)
        .highlight_symbol(if app.focus_on_menu { ">> " } else { "   " });

    f.render_stateful_widget(list, left_chunks[1], &mut app.menu_state);

    // Main Content
    let content_style = if !app.focus_on_menu {
        Style::default().fg(app.theme.border)
    } else {
        Style::default()
    };

    let content_highlight_style = if !app.focus_on_menu {
        Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    if app.selected_project.is_none() {
        let placeholder = Paragraph::new("\n\n  No project selected.\n  Press Ctrl+R to select a project.")
            .block(Block::default().borders(Borders::ALL).title(" Content ").border_style(content_style));
        f.render_widget(placeholder, main_chunks[1]);
    } else {
        match app.active_selection {
            MenuSelection::MergeRequests => {
                let mrs: Vec<ListItem> = app.merge_requests
                    .iter()
                    .map(|mr| {
                        let disp_state = &mr.state;
                        let state_style = get_status_style(disp_state, &app.theme);
                        
                        Line::from(vec![
                            Span::styled(format!("[{}]", disp_state), state_style),
                            Span::raw(format!(" {} - {} by ", mr.references.full, mr.title)),
                            Span::styled(format!("@{}", mr.author.username), Style::default().fg(app.theme.author)),
                        ])
                    })
                    .map(ListItem::from)
                    .collect();
                
                let mrs_list = List::new(mrs)
                    .block(Block::default().borders(Borders::ALL).title(format!(" MRs for {} ", proj_name)).border_style(content_style))
                    .highlight_style(content_highlight_style)
                    .highlight_symbol(if !app.focus_on_menu { "> " } else { "  " });
                
                f.render_stateful_widget(mrs_list, main_chunks[1], &mut app.mrs_state);
            }
            MenuSelection::Issues => {
                let issues: Vec<ListItem> = app.issues
                    .iter()
                    .map(|issue| {
                        let state_style = get_status_style(&issue.state, &app.theme);
                        let itype = issue.issue_type.as_deref().unwrap_or("issue");
                        
                        let mut spans = vec![
                            Span::styled(format!("[{}]", issue.state), state_style),
                            Span::styled(format!(" [{}]", itype), Style::default().fg(app.theme.dim)),
                            Span::raw(format!(" #{} - {} by ", issue.iid, issue.title)),
                            Span::styled(format!("@{}", issue.author.username), Style::default().fg(app.theme.author)),
                        ];

                        if !issue.labels.is_empty() {
                            spans.push(Span::raw(" "));
                            for label in &issue.labels {
                                let bg = hex_to_color(&label.color);
                                let fg = hex_to_color(&label.text_color);
                                spans.push(Span::styled(
                                    format!(" {} ", label.name), 
                                    Style::default().bg(bg).fg(fg).add_modifier(Modifier::BOLD)
                                ));
                                spans.push(Span::raw(" "));
                            }
                        }

                        Line::from(spans)
                    })
                    .map(ListItem::from)
                    .collect();
                
                let issues_list = List::new(issues)
                    .block(Block::default().borders(Borders::ALL).title(format!(" Issues for {} ", proj_name)).border_style(content_style))
                    .highlight_style(content_highlight_style)
                    .highlight_symbol(if !app.focus_on_menu { "> " } else { "  " });
                
                f.render_stateful_widget(issues_list, main_chunks[1], &mut app.issues_state);
            }
            _ => {
                let placeholder = Paragraph::new(format!("Content for {:?}", app.active_selection))
                    .block(Block::default().borders(Borders::ALL).title(" Content ").border_style(content_style));
                f.render_widget(placeholder, main_chunks[1]);
            }
        }
    }

    let shortcuts = vec![
        ("j/k", "nav"),
        ("tab", "switch"),
        ("ctrl+r", "switch project"),
        ("enter", "select"),
        ("r", "refresh"),
        ("q", "quit"),
    ];
    render_shortcuts(f, chunks[1], shortcuts, &app.theme);
}

pub fn render_detail_view(f: &mut Frame, app: &mut App) {
    if let Some(mr) = &app.selected_mr {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Min(0),
                Constraint::Length(if app.input_mode { 3 } else { 0 }),
                Constraint::Length(1),
            ])
            .split(f.area());

        let state_style = get_status_style(&mr.state, &app.theme);
        let description = mr.description.as_deref().unwrap_or("No description provided.");
        let mr_info_spans = vec![
            Line::from(vec![Span::raw("Title: "), Span::raw(&mr.title)]),
            Line::from(vec![Span::raw("Author: "), Span::styled(format!("@{}", mr.author.username), Style::default().fg(app.theme.author))]),
            Line::from(vec![Span::raw("State: "), Span::styled(&mr.state, state_style)]),
            Line::from(vec![Span::raw("URL: "), Span::raw(&mr.web_url)]),
            Line::from(""),
            Line::from(vec![Span::raw("Description: "), Span::raw(description)]),
        ];
        let mr_info_widget = Paragraph::new(mr_info_spans)
            .block(Block::default().borders(Borders::ALL).title(format!(" MR: {} ", mr.references.full)))
            .wrap(Wrap { trim: true });
        f.render_widget(mr_info_widget, chunks[0]);

        let selected_idx = app.discussion_state.selected();

        let comments_list: Vec<ListItem> = app.discussions
            .iter()
            .enumerate()
            .map(|(d_idx, d)| {
                let mut lines = Vec::new();
                let is_item_selected = selected_idx == Some(d_idx);
                
                let is_resolved = d.resolved.unwrap_or(false);
                let (prefix_style, author_style, body_style) = if is_resolved {
                    (Style::default().fg(app.theme.dim), Style::default().fg(app.theme.dim), Style::default().fg(app.theme.dim).add_modifier(Modifier::ITALIC))
                } else {
                    (Style::default().fg(app.theme.border), Style::default().fg(app.theme.author), Style::default().fg(app.theme.unchanged).add_modifier(Modifier::ITALIC))
                };

                let highlight_style = if is_item_selected && !app.input_mode {
                    Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                if d.collapsed {
                    if let Some(note) = d.notes.first() {
                        let path_info = if let Some(pos) = &note.position {
                            format!(" [{}]", pos.new_path.as_deref().or(pos.old_path.as_deref()).unwrap_or("?"))
                        } else {
                            "".to_string()
                        };
                        let replies = if d.notes.len() > 1 { format!(" (+{} replies)", d.notes.len() - 1) } else { "".to_string() };
                        let body_preview = note.body.chars().take(50).collect::<String>();
                        let preview_dots = if note.body.len() > 50 { "..." } else { "" };

                        let resolution = if is_resolved {
                            Span::styled(" [resolved]", Style::default().fg(app.theme.dim))
                        } else if d.notes.iter().any(|n| n.resolvable.unwrap_or(false)) {
                            Span::styled(" [open]", Style::default().fg(app.theme.opened))
                        } else {
                            Span::raw("")
                        };

                        let first_line_prefix = if is_item_selected && !app.input_mode { "> " } else { "[+] " };

                        lines.push(Line::from(vec![
                            Span::styled(first_line_prefix, if is_item_selected { highlight_style.fg(app.theme.border) } else { prefix_style }),
                            Span::styled(format!("@{}", note.author.username), if is_item_selected { highlight_style.fg(app.theme.author) } else { author_style }),
                            Span::styled(path_info, if is_item_selected { highlight_style } else { Style::default() }),
                            resolution.patch_style(highlight_style),
                            Span::styled(": ", if is_item_selected { highlight_style } else { Style::default() }),
                            Span::styled(format!("{}{}", body_preview, preview_dots), if is_item_selected { highlight_style } else { body_style }),
                            Span::styled(replies, if is_item_selected { highlight_style.fg(app.theme.merged) } else if is_resolved { Style::default().fg(app.theme.dim) } else { Style::default().fg(app.theme.merged) }),
                        ]));
                    }
                } else {
                    let resolution = if is_resolved {
                        Span::styled(" [resolved]", Style::default().fg(app.theme.dim))
                    } else if d.notes.iter().any(|n| n.resolvable.unwrap_or(false)) {
                        Span::styled(" [open]", Style::default().fg(app.theme.opened))
                    } else {
                        Span::raw("")
                    };

                    for (i, note) in d.notes.iter().enumerate() {
                        if i == 0 {
                            // Header for the first note in discussion
                            let first_line_prefix = if is_item_selected && !app.input_mode { "> " } else { "[-] " };
                            lines.push(Line::from(vec![
                                Span::styled(first_line_prefix, if is_item_selected { highlight_style.fg(app.theme.border) } else { prefix_style }),
                                Span::styled(format!("@{}", note.author.username), if is_item_selected { highlight_style.fg(app.theme.author) } else { author_style.add_modifier(Modifier::BOLD) }),
                                resolution.clone().patch_style(highlight_style),
                                Span::styled(" says:", if is_item_selected { highlight_style } else { Style::default() }),
                            ]));
    
                            // If it's a code comment, show the context
                            if let Some(pos) = &note.position {
                                let path = pos.new_path.as_deref().or(pos.old_path.as_deref()).unwrap_or("unknown");
                                lines.push(Line::from(vec![
                                    Span::styled(format!("  File: {}", path), Style::default().fg(if is_resolved { app.theme.dim } else { app.theme.border }).add_modifier(Modifier::BOLD)),
                                ]));
    
                                if let Some(context) = &note.context {
                                    for (ln, content, ltype, is_target) in context {
                                        let (style, prefix) = match ltype {
                                            LineType::Addition => (Style::default().fg(if is_resolved { app.theme.dim } else { app.theme.addition }), "+"),
                                            LineType::Removal => (Style::default().fg(if is_resolved { app.theme.dim } else { app.theme.removal }), "-"),
                                            LineType::Unchanged => (Style::default().fg(app.theme.dim), " "),
                                        };
    
                                        let final_style = if *is_target {
                                            style.add_modifier(Modifier::BOLD).bg(if is_resolved { Color::Rgb(30, 30, 30) } else { Color::Rgb(60, 60, 60) })
                                        } else {
                                            style
                                        };
    
                                        lines.push(Line::from(vec![
                                            Span::styled(format!("  {}{:>4} | ", prefix, ln), final_style),
                                            Span::styled(content, final_style),
                                        ]));
                                    }
                                } else if let Some(line) = pos.new_line.or(pos.old_line) {
                                    lines.push(Line::from(vec![
                                        Span::styled(format!("  (Line: {})", line), Style::default().fg(if is_resolved { app.theme.dim } else { app.theme.border })),
                                    ]));
                                }
                            } else if let Some(note_type) = &note.note_type {
                                // General comment or other types
                                lines.push(Line::from(vec![
                                    Span::styled(format!("[{}]", note_type), Style::default().fg(app.theme.dim)),
                                ]));
                            }
                        } else {
                            // Indented reply
                            lines.push(Line::from(vec![
                                Span::raw("  "),
                                Span::styled(format!("@{}", note.author.username), if is_resolved { Style::default().fg(app.theme.dim) } else { Style::default().fg(app.theme.merged) }),
                                Span::raw(" replied:"),
                            ]));
                        }
                        let body_style = if is_resolved { Style::default().fg(app.theme.dim) } else { Style::default() };
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(&note.body, body_style),
                        ]));
                        lines.push(Line::from(""));
                    }
                }
                ListItem::new(lines)
            })
            .collect();
        let comments_style = if !app.input_mode {
            Style::default().fg(app.theme.border)
        } else {
            Style::default()
        };

        let comments = List::new(comments_list)
            .block(Block::default().borders(Borders::ALL).title(" Discussions ").border_style(comments_style));

        f.render_stateful_widget(comments, chunks[1], &mut app.discussion_state);
        if app.input_mode {
            let input = Paragraph::new(app.reply_input.as_str())
                .style(Style::default().fg(app.theme.border))
                .block(Block::default().borders(Borders::ALL).title(" Reply "));
            f.render_widget(input, chunks[2]);
        }

        let shortcuts = vec![
            ("j/k", "nav"),
            ("space", "expand"),
            ("s", "resolve"),
            ("enter", "reply"),
            ("esc", "back"),
            ("v", "vscode"),
            ("r", "refresh"),
            ("q", "quit"),
        ];
        render_shortcuts(f, chunks[3], shortcuts, &app.theme);
    }
}
