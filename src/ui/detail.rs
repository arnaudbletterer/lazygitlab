use crate::models::{App, DetailSection, InputTarget, LineType};
use crate::ui::list::{render_discussions_panel, render_metadata_panel};
use ratatui::{prelude::*, widgets::*};

pub fn render_detail_view(f: &mut Frame, app: &mut App, area: Rect) {
    let is_detail_focused = app.active_panel == crate::models::Panel::MainDetail;

    // Outer block containing the detail area
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " 🔍 Details & Inspector ",
            if is_detail_focused {
                Style::default().fg(app.theme.accent).bold()
            } else {
                Style::default().fg(app.theme.title_fg)
            },
        ))
        .border_style(if is_detail_focused {
            Style::default().fg(app.theme.active_border)
        } else {
            Style::default().fg(app.theme.inactive_border)
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split inner area: tab headers (Length(1)), divider (Length(1)), and content (Min(0))
    let detail_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tabs
            Constraint::Length(1), // Horizontal Divider
            Constraint::Min(0),    // Content Area
        ])
        .split(inner);

    // Generate tab titles
    let has_changes = app.selected_mr.is_some();
    let mut titles = vec![
        " 📄 Overview ".to_string(),
        format!(" 💬 Discussions ({}) ", app.discussions.len()),
    ];
    if has_changes {
        titles.push(format!(" 📝 Changes ({}) ", app.mr_diffs.len()));
    }

    // Determine active tab index based on detail_section
    let active_tab_idx = match app.detail_section {
        DetailSection::Description | DetailSection::Metadata => 0,
        DetailSection::Discussions => 1,
        DetailSection::Changes => 2,
    };
    let max_tabs = if has_changes { 3 } else { 2 };
    let active_tab_idx = active_tab_idx.min(max_tabs - 1);

    // Render Tabs widget
    let tabs = Tabs::new(titles)
        .select(active_tab_idx)
        .divider(Span::styled(" │ ", Style::default().fg(app.theme.dim)))
        .style(Style::default().fg(app.theme.dim))
        .highlight_style(
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, detail_chunks[0]);
    app.set_rect("detail_tabs", detail_chunks[0]);

    // Render Divider Line
    let divider = Paragraph::new(Span::styled(
        "─".repeat(detail_chunks[1].width as usize),
        Style::default().fg(app.theme.dim),
    ));
    f.render_widget(divider, detail_chunks[1]);

    let content_area = detail_chunks[2];

    // Render tab content based on selected tab index
    match active_tab_idx {
        0 => {
            // TAB 0: Description & Metadata
            let is_editing_desc =
                app.input_mode && app.input_target == InputTarget::EditDescription;
            let is_desc_focused =
                is_detail_focused && app.detail_section == DetailSection::Description;

            // Split content vertically: Metadata (rows + borders) and Description Preview (Min(0))
            let meta_rows: u16 = if app.selected_mr.is_some() {
                12
            } else if app.selected_issue.is_some() {
                11
            } else {
                3
            };
            let desc_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(meta_rows + 2), // Metadata block
                    Constraint::Min(0),                // Description text
                ])
                .split(content_area);

            // Render Metadata Panel
            render_metadata_panel(f, app, desc_chunks[0]);
            app.set_rect("metadata_panel", desc_chunks[0]);

            // Render Description Panel
            let desc_title = if is_editing_desc {
                " 📄 Description (Editing: Ctrl+Enter to save, Esc to cancel) "
            } else if is_desc_focused {
                " 📄 Description (Focused) [j/k: scroll | e/Enter: edit | m: metadata] "
            } else {
                " 📄 Description [press 'd' to focus] "
            };
            let desc_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    desc_title,
                    if is_editing_desc || is_desc_focused {
                        Style::default().fg(app.theme.accent).bold()
                    } else {
                        Style::default().fg(app.theme.title_fg)
                    },
                ))
                .border_style(if is_editing_desc || is_desc_focused {
                    Style::default().fg(app.theme.active_border)
                } else {
                    Style::default().fg(app.theme.inactive_border)
                });
            let desc_inner = desc_block.inner(desc_chunks[1]);
            f.render_widget(desc_block, desc_chunks[1]);
            app.set_rect("detail_view", desc_chunks[1]);

            if is_editing_desc {
                f.render_widget(&app.textarea, desc_inner);
            } else {
                let mut content = Vec::new();
                let description_body: String = if let Some(mr) = &app.selected_mr {
                    mr.description
                        .as_deref()
                        .unwrap_or("No description provided.")
                        .to_string()
                } else if let Some(issue) = &app.selected_issue {
                    issue
                        .description
                        .as_deref()
                        .unwrap_or("No description provided.")
                        .to_string()
                } else {
                    " No item selected.".to_string()
                };
                for line in app.markdown_cached(&description_body).iter() {
                    content.push(line.clone());
                }
                f.render_widget(
                    Paragraph::new(content)
                        .wrap(Wrap { trim: false })
                        .scroll((app.description_scroll, 0)),
                    desc_inner,
                );
            }
        }
        1 => {
            // TAB 1: Discussions (Thread list at top, selected comments at bottom)
            let is_disc_focused =
                is_detail_focused && app.detail_section == DetailSection::Discussions;

            let disc_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(35), // Threads list summary
                    Constraint::Min(0),         // Thread comments
                ])
                .split(content_area);

            // Render threads list summary panel
            render_discussions_panel(f, app, disc_chunks[0]);
            app.set_rect("discussions_panel", disc_chunks[0]);

            // Render selected thread comments panel
            let comments_block = Block::default()
                .borders(Borders::ALL)
                .title(" Discussion Thread ")
                .border_style(if is_disc_focused {
                    Style::default().fg(app.theme.border)
                } else {
                    Style::default().fg(app.theme.dim)
                });
            let comments_inner = comments_block.inner(disc_chunks[1]);
            f.render_widget(comments_block, disc_chunks[1]);

            let mut content = Vec::new();
            let mut excerpt_data = None;

            if let Some(idx) = app.discussion_state.selected() {
                if let Some(disc) = app.discussions.get(idx) {
                    // Check if first note has context
                    if let Some(first_note) = disc.notes.first()
                        && let Some(context) = &first_note.context
                        && !context.is_empty()
                        && disc_chunks[1].height >= 6
                    {
                        excerpt_data = Some((context.clone(), first_note.position.clone()));
                    }

                    for note in &disc.notes {
                        let is_activity = note.system;
                        let is_resolved = !is_activity && disc.resolved.unwrap_or(false);

                        let (prefix, prefix_style, author_style, _content_style) = if is_activity {
                            (
                                "[ACTIVITY] ",
                                Style::default().fg(app.theme.dim),
                                Style::default().fg(app.theme.dim),
                                Style::default().fg(app.theme.dim),
                            )
                        } else if let Some(resolved) = disc.resolved {
                            if resolved {
                                (
                                    "[RESOLVED] ",
                                    Style::default().fg(app.theme.dim),
                                    Style::default().fg(app.theme.dim),
                                    Style::default().fg(app.theme.dim),
                                )
                            } else {
                                (
                                    "[UNRESOLVED] ",
                                    Style::default().fg(app.theme.pending),
                                    Style::default()
                                        .fg(app.theme.author)
                                        .add_modifier(Modifier::BOLD),
                                    Style::default(),
                                )
                            }
                        } else {
                            (
                                "",
                                Style::default(),
                                Style::default()
                                    .fg(app.theme.author)
                                    .add_modifier(Modifier::BOLD),
                                Style::default(),
                            )
                        };

                        let header = vec![
                            Span::styled(prefix, prefix_style),
                            Span::styled(format!("@{}", note.author.username), author_style),
                        ];
                        content.push(Line::from(header));

                        let markdown_lines = app.markdown_cached(&note.body);
                        for line in markdown_lines.iter() {
                            let mut line_spans = Vec::new();
                            for span in &line.spans {
                                let mut s = span.clone();
                                if is_resolved || is_activity {
                                    s.style = s.style.patch(Style::default().fg(app.theme.dim));
                                }
                                line_spans.push(s);
                            }
                            content.push(Line::from(line_spans));
                        }
                        content.push(Line::from(""));
                    }
                    if disc.notes.is_empty() {
                        content.push(Line::from(" No notes in this discussion."));
                    }
                }
            } else {
                content.push(Line::from(
                    " Select a discussion thread to view full conversation.",
                ));
            }

            let final_notes_inner = if let Some((context, position_opt)) = excerpt_data {
                let needed_height = (context.len() as u16 + 2)
                    .min(disc_chunks[1].height.saturating_sub(4))
                    .max(3);
                let split = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(needed_height), Constraint::Min(0)])
                    .split(comments_inner);

                let file_name = position_opt
                    .as_ref()
                    .and_then(|pos| pos.new_path.as_deref().or(pos.old_path.as_deref()))
                    .unwrap_or("Unknown File");

                let excerpt_block = Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" File Excerpt: {} ", file_name))
                    .border_style(Style::default().fg(app.theme.dim));
                let excerpt_inner = excerpt_block.inner(split[0]);
                f.render_widget(excerpt_block, split[0]);

                let mut excerpt_lines = Vec::new();
                for (ln, line_text, ltype, is_target) in &context {
                    let mut line_style = match ltype {
                        LineType::Addition => Style::default().fg(app.theme.success),
                        LineType::Removal => Style::default().fg(app.theme.failed),
                        LineType::Unchanged => Style::default(),
                    };
                    if *is_target {
                        line_style = line_style
                            .bg(app.theme.highlight_bg)
                            .add_modifier(Modifier::BOLD);
                    }
                    excerpt_lines.push(Line::from(vec![
                        Span::styled(
                            format!("{:4} │ ", ln),
                            if *is_target {
                                line_style
                            } else {
                                Style::default().fg(app.theme.dim)
                            },
                        ),
                        Span::styled(line_text.clone(), line_style),
                    ]));
                }
                f.render_widget(Paragraph::new(excerpt_lines), excerpt_inner);
                split[1]
            } else {
                comments_inner
            };

            f.render_widget(
                Paragraph::new(content).wrap(Wrap { trim: false }),
                final_notes_inner,
            );
        }
        _ => {
            // TAB 2: Changes / Files
            let is_changes_focused =
                is_detail_focused && app.detail_section == DetailSection::Changes;

            let changes_block = Block::default()
                .borders(Borders::ALL)
                .title(" Changes / Files ")
                .border_style(if is_changes_focused {
                    Style::default().fg(app.theme.border)
                } else {
                    Style::default().fg(app.theme.dim)
                });
            let changes_inner = changes_block.inner(content_area);
            f.render_widget(changes_block, content_area);
            app.set_rect("changes_panel", changes_inner);

            if app.mr_diffs.is_empty() {
                f.render_widget(
                    Paragraph::new(" No changes found or still loading...").fg(app.theme.dim),
                    changes_inner,
                );
            } else {
                let files: Vec<ListItem> = app
                    .mr_diffs
                    .iter()
                    .map(|d| {
                        ListItem::new(Line::from(vec![
                            Span::styled(" ● ", Style::default().fg(app.theme.border)),
                            Span::raw(format!(" {} -> {}", d.old_path, d.new_path)),
                        ]))
                    })
                    .collect();
                f.render_stateful_widget(
                    List::new(files)
                        .highlight_style(if is_changes_focused {
                            Style::default()
                                .bg(app.theme.highlight_bg)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        })
                        .highlight_symbol(if is_changes_focused { "> " } else { "  " }),
                    changes_inner,
                    &mut app.mr_diff_state,
                );

                // Helpful hint at the bottom
                let hint_area = Rect::new(
                    changes_inner.x,
                    changes_inner.y + changes_inner.height.saturating_sub(1),
                    changes_inner.width,
                    1,
                );
                f.render_widget(
                    Paragraph::new(" Press [Enter] or [v] to view full diff ")
                        .fg(app.theme.dim)
                        .alignment(Alignment::Right),
                    hint_area,
                );
            }
        }
    }
}
