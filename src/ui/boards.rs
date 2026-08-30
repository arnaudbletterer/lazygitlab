use crate::models::App;
use ratatui::{prelude::*, widgets::*};

pub fn render_boards_view(f: &mut Frame, app: &mut App, area: Rect) {
    if app.selected_board.is_none() {
        render_board_list(f, app, area);
    } else {
        render_kanban_board(f, app, area);
    }
}

fn render_board_list(f: &mut Frame, app: &mut App, area: Rect) {
    app.set_rect("main_list", area);
    let is_main_focused = app.active_panel == crate::models::Panel::Main;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Select a Board ")
        .border_style(if is_main_focused {
            Style::default().fg(app.theme.border)
        } else {
            Style::default().fg(app.theme.dim)
        });

    let items: Vec<ListItem> = app
        .project_boards
        .iter()
        .map(|b| ListItem::new(format!("  {}", b.name)))
        .collect();

    if items.is_empty() {
        f.render_widget(
            Paragraph::new(" No boards found for this project.")
                .fg(app.theme.dim)
                .block(block),
            area,
        );
    } else {
        f.render_stateful_widget(
            List::new(items)
                .block(block)
                .highlight_style(if is_main_focused {
                    Style::default()
                        .bg(app.theme.highlight_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                })
                .highlight_symbol(if is_main_focused { "> " } else { "  " }),
            area,
            &mut app.project_boards_state,
        );
    }
}

fn render_kanban_board(f: &mut Frame, app: &mut App, area: Rect) {
    let lists = app.board_lists.clone();

    if lists.is_empty() {
        if app.is_loading {
            f.render_widget(
                Paragraph::new("\n  Loading board columns...").fg(app.theme.pending),
                area,
            );
        } else {
            f.render_widget(Paragraph::new("\n  No columns found for this board.\n  (Note: Backlog and Closed columns are currently not shown if they are not explicitly added as label lists)").fg(app.theme.dim), area);
        }
        return;
    }

    // Determine visible columns
    let column_width = 38; // Increased width for better readability
    let max_visible = (area.width / column_width).max(1);
    let selected_col = app.board_lists_state.selected().unwrap_or(0);

    // Windowing logic for horizontal scroll
    let start_col = if selected_col < (max_visible as usize / 2) {
        0
    } else {
        (selected_col + 1).saturating_sub(max_visible as usize)
    };
    let end_col = (start_col + max_visible as usize).min(lists.len());
    let visible_lists = &lists[start_col..end_col];

    let constraints: Vec<Constraint> = (0..visible_lists.len())
        .map(|_| Constraint::Length(column_width))
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, list) in visible_lists.iter().enumerate() {
        let title = match &list.label {
            Some(l) => l.name.clone(),
            None => list.list_type.clone(),
        };

        let list_idx = start_col + i;
        app.set_rect(&format!("kanban_col_{}", list_idx), chunks[i]);
        let is_column_focused = app.active_panel == crate::models::Panel::Main
            && app.board_lists_state.selected() == Some(list_idx);

        let issues = app
            .filtered_board_issues
            .get(&list.id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let header_style = if is_column_focused {
            Style::default()
                .fg(Color::Black)
                .bg(app.theme.border)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(app.theme.border)
                .add_modifier(Modifier::BOLD)
        };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            format!(" 📋 {} ({}) ", title.to_uppercase(), issues.len()),
            header_style,
        ))
        .border_style(Style::default().fg(if is_column_focused {
            app.theme.active_border
        } else {
            app.theme.inactive_border
        }));

        if issues.is_empty() && app.is_loading {
            f.render_widget(
                Paragraph::new("\n  Loading issues...")
                    .fg(app.theme.pending)
                    .block(block),
                chunks[i],
            );
            continue;
        }

        if issues.is_empty() {
            f.render_widget(
                Paragraph::new("\n  Empty column")
                    .fg(app.theme.dim)
                    .block(block),
                chunks[i],
            );
            continue;
        }

        let list_items: Vec<ListItem> = issues
            .iter()
            .map(|issue| {
                let mut lines = Vec::new();

                // 1. Title Line
                let title_line = vec![
                    Span::styled(
                        format!("#{} ", issue.iid),
                        Style::default().fg(app.theme.dim),
                    ),
                    Span::styled(&issue.title, Style::default().add_modifier(Modifier::BOLD)),
                ];
                lines.push(Line::from(title_line));

                // 2. Metadata Line (Author, Milestone, Votes)
                let mut meta_spans = vec![Span::styled(
                    format!(" @{} ", issue.author.username),
                    Style::default().fg(app.theme.author),
                )];

                if let Some(ms) = &issue.milestone {
                    meta_spans.push(Span::raw(" | "));
                    meta_spans.push(Span::styled(
                        format!(" 󰔛 {} ", ms.title),
                        Style::default().fg(app.theme.pending),
                    ));
                }

                let upvotes = issue.upvotes.unwrap_or(0);
                if upvotes > 0 {
                    meta_spans.push(Span::raw(" | "));
                    meta_spans.push(Span::styled(
                        format!(" 👍 {} ", upvotes),
                        Style::default().fg(app.theme.success),
                    ));
                }

                lines.push(Line::from(meta_spans));

                // 3. Labels Line
                if !issue.labels.is_empty() {
                    let mut label_spans = vec![Span::raw(" ")];
                    for label in issue.labels.iter().take(3) {
                        label_spans.push(Span::styled(
                            format!(" {} ", label.name),
                            Style::default()
                                .bg(crate::ui::widgets::hex_to_color(&label.color))
                                .fg(crate::ui::widgets::get_contrasting_text_color(&label.color)),
                        ));
                        label_spans.push(Span::raw(" "));
                    }
                    if issue.labels.len() > 3 {
                        label_spans.push(Span::styled(
                            format!("+{}", issue.labels.len() - 3),
                            Style::default().fg(app.theme.dim),
                        ));
                    }
                    lines.push(Line::from(label_spans));
                }

                lines.push(Line::from("")); // Spacer

                ListItem::new(lines)
            })
            .collect();

        let mut local_state = ListState::default();
        let state_ref = if is_column_focused {
            app.board_issue_states.entry(list.id).or_default()
        } else {
            &mut local_state
        };

        f.render_stateful_widget(
            List::new(list_items)
                .block(block)
                .highlight_style(if is_column_focused {
                    Style::default().bg(app.theme.highlight_bg)
                } else {
                    Style::default()
                })
                .highlight_symbol(if is_column_focused { "> " } else { "  " }),
            chunks[i],
            state_ref,
        );
    }
}

pub fn render_epics_view(f: &mut Frame, app: &mut App, area: Rect) {
    app.set_rect("main_list", area);
    let is_main_focused = app.active_panel == crate::models::Panel::Main;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Epics ")
        .border_style(if is_main_focused {
            Style::default().fg(app.theme.border)
        } else {
            Style::default().fg(app.theme.dim)
        });

    let items: Vec<ListItem> = app
        .epics
        .iter()
        .map(|e| {
            let mut spans = vec![
                Span::styled(format!("&{} ", e.iid), Style::default().fg(app.theme.dim)),
                Span::styled(
                    e.title.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ];
            if e.state == "closed" {
                spans.push(Span::styled(
                    " [CLOSED]",
                    Style::default().fg(app.theme.failed),
                ));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    if items.is_empty() {
        f.render_widget(
            Paragraph::new(" No epics found for this group.")
                .fg(app.theme.dim)
                .block(block),
            area,
        );
    } else {
        f.render_stateful_widget(
            List::new(items)
                .block(block)
                .highlight_style(if is_main_focused {
                    Style::default()
                        .bg(app.theme.highlight_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                })
                .highlight_symbol(if is_main_focused { "> " } else { "  " }),
            area,
            &mut app.epics_state,
        );
    }
}
