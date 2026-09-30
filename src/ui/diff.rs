use crate::models::{App, DiffDisplayLine};
use ratatui::{prelude::*, widgets::*};

pub fn render_diff_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header with MR info
            Constraint::Min(0),    // Main workspace
            Constraint::Length(1), // Shortcuts
        ])
        .split(area);

    // 1. Render Header
    if let Some(mr) = &app.selected_mr {
        let mut header_lines = vec![
            Line::from(vec![
                Span::styled(
                    format!(" !{} ", mr.iid),
                    Style::default()
                        .bg(app.theme.border)
                        .fg(crate::ui::widgets::get_contrast_color(app.theme.border))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(&mr.title, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw(" Author: "),
                Span::styled(
                    format!("@{}", mr.author.username),
                    Style::default().fg(app.theme.author),
                ),
                Span::raw("  State: "),
                Span::styled(
                    &mr.state,
                    crate::ui::widgets::get_status_style(&mr.state, &app.theme),
                ),
                Span::raw("  Reactions: "),
                Span::styled(
                    format!("+{}", mr.upvotes.unwrap_or(0)),
                    Style::default().fg(app.theme.success),
                ),
                Span::raw("  Comments: "),
                Span::styled(
                    format!("{}", app.discussions.len()),
                    Style::default().fg(app.theme.pending),
                ),
                Span::raw("  Approvals: "),
                match &app.approvals {
                    Some(info) => {
                        let approvers: Vec<String> = info
                            .approved_by
                            .iter()
                            .filter_map(|a| a.user.as_ref().map(|u| format!("@{}", u.username)))
                            .collect();
                        Span::styled(
                            format!(
                                "{}{}",
                                match (info.approvals_left, info.approvals_required) {
                                    (Some(left), Some(req)) =>
                                        format!("{}/{} ", req.saturating_sub(left), req),
                                    _ => String::new(),
                                },
                                if approvers.is_empty() {
                                    "none".to_string()
                                } else {
                                    approvers.join(", ")
                                }
                            ),
                            Style::default().fg(if info.approved_by.is_empty() {
                                app.theme.pending
                            } else {
                                app.theme.success
                            }),
                        )
                    }
                    None => Span::styled("n/a", Style::default().fg(app.theme.dim)),
                },
            ]),
        ];
        if let Some(refs) = &mr.diff_refs {
            header_lines.push(Line::from(vec![
                Span::raw(" Range: "),
                Span::styled(
                    format!("{}...{}", &refs.start_sha[..8], &refs.head_sha[..8]),
                    Style::default().fg(app.theme.dim),
                ),
            ]));
        }
        f.render_widget(
            Paragraph::new(header_lines).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" Approval & Details ")),
            chunks[0],
        );
    }

    // 2. Render Main Workspace (2-Column)
    let workspace_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // File list
            Constraint::Percentage(75), // Diff content
        ])
        .split(chunks[1]);

    let is_file_list_focused = app.diff_line_state.selected().is_none();
    let highlight_style = Style::default()
        .bg(app.theme.highlight_bg)
        .add_modifier(Modifier::BOLD);

    // File List (Left)
    let diff_items: Vec<ListItem> = app
        .mr_diffs
        .iter()
        .map(|d| {
            let label = if d.new_path == "/dev/null" || d.new_path.is_empty() {
                format!(" [-] {}", d.old_path)
            } else if d.old_path == "/dev/null" || d.old_path.is_empty() {
                format!(" [+] {}", d.new_path)
            } else if d.old_path != d.new_path {
                format!(" [R] {} -> {}", d.old_path, d.new_path)
            } else {
                format!(" {}", d.new_path)
            };
            ListItem::new(label)
        })
        .collect();

    let diff_list = List::new(diff_items)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" Files "))
        .highlight_style(if is_file_list_focused {
            highlight_style
        } else {
            Style::default().fg(app.theme.dim)
        })
        .highlight_symbol(">> ");
    f.render_stateful_widget(diff_list, workspace_chunks[0], &mut app.mr_diff_state);

    // Diff Content (Right)
    if let Some(selected_idx) = app.mr_diff_state.selected() {
        if let Some(mr_diff) = app.mr_diffs.get(selected_idx) {
            let focused_line_idx = app.diff_line_state.selected();

            // 1. Determine if we are hovering a discussion and what its range is
            let hovered_range = if let Some(idx) = focused_line_idx {
                if let Some(DiffDisplayLine::Discussion(disc)) = mr_diff.display_lines.get(idx) {
                    if let Some(note) = disc.notes.first() {
                        if let Some(pos) = &note.position {
                            if let Some(range) = &pos.line_range {
                                Some((
                                    range.start.old_line,
                                    range.start.new_line,
                                    range.end.old_line,
                                    range.end.new_line,
                                ))
                            } else {
                                // Single line "range"
                                Some((pos.old_line, pos.new_line, pos.old_line, pos.new_line))
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let diff_lines: Vec<ListItem> = mr_diff
                .display_lines
                .iter()
                .enumerate()
                .map(|(idx, dl)| {
                    let is_cursor = focused_line_idx == Some(idx);
                    let is_in_manual_selection = if let Some(start) = app.diff_selection_start {
                        let current = focused_line_idx.unwrap_or(0);
                        let s = start.min(current);
                        let e = start.max(current);
                        idx >= s && idx <= e
                    } else {
                        false
                    };

                    match dl {
                        DiffDisplayLine::Code(line, old_line, new_line) => {
                            let is_in_hovered_range = if let Some((os, ns, oe, ne)) = hovered_range
                            {
                                let match_old = old_line.is_some()
                                    && os.is_some()
                                    && oe.is_some()
                                    && *old_line >= os
                                    && *old_line <= oe;
                                let match_new = new_line.is_some()
                                    && ns.is_some()
                                    && ne.is_some()
                                    && *new_line >= ns
                                    && *new_line <= ne;
                                match_old || match_new
                            } else {
                                false
                            };

                            let style = if line.starts_with('+') {
                                Style::default().fg(app.theme.success)
                            } else if line.starts_with('-') {
                                Style::default().fg(app.theme.failed)
                            } else if line.starts_with("@@") {
                                Style::default().fg(app.theme.shortcut_key)
                            } else {
                                Style::default()
                            };

                            let mut item = ListItem::new(Line::from(Span::styled(line, style)));
                            if !is_file_list_focused {
                                if is_cursor {
                                    item = item.style(
                                        Style::default()
                                            .bg(app.theme.highlight_bg)
                                            .add_modifier(Modifier::BOLD),
                                    );
                                } else if is_in_manual_selection || is_in_hovered_range {
                                    // Associated range highlight (using darker blue)
                                    item =
                                        item.style(Style::default().bg(app.theme.range_highlight));
                                }
                            }
                            item
                        }
                        DiffDisplayLine::Discussion(disc) => {
                            let is_collapsed = disc.collapsed;
                            let first_note = disc.notes.first();
                            let is_activity = first_note.map(|n| n.system).unwrap_or(false);
                            let is_resolved = !is_activity && disc.resolved.unwrap_or(false);
                            let mut text = Vec::new();

                            let (prefix, prefix_style, author_style, content_style) = if is_activity
                            {
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

                            if is_collapsed {
                                let author = first_note
                                    .map(|n| n.author.username.as_str())
                                    .unwrap_or("?");
                                let body = first_note
                                    .map(|n| n.body.chars().take(40).collect::<String>())
                                    .unwrap_or_else(|| "...".into());
                                text.push(Line::from(vec![
                                    Span::styled(
                                        " [+] ",
                                        prefix_style.add_modifier(Modifier::BOLD),
                                    ),
                                    Span::styled(prefix, prefix_style),
                                    Span::styled(format!("@{}", author), author_style),
                                    Span::styled(": ", content_style),
                                    Span::styled(body.replace('\n', " "), content_style),
                                    if first_note.is_some_and(|n| n.body.len() > 40) {
                                        Span::styled("...", content_style)
                                    } else {
                                        Span::raw("")
                                    },
                                    Span::styled(
                                        format!(" ({} more)", disc.notes.len().saturating_sub(1)),
                                        Style::default().fg(app.theme.dim),
                                    ),
                                ]));
                            } else {
                                text.push(Line::from(vec![
                                    Span::styled(
                                        " [-] ",
                                        prefix_style.add_modifier(Modifier::BOLD),
                                    ),
                                    Span::styled(prefix, prefix_style),
                                    Span::styled(
                                        "Discussion Thread",
                                        content_style.add_modifier(Modifier::BOLD),
                                    ),
                                ]));
                                for (i, note) in disc.notes.iter().enumerate() {
                                    let indent = if i > 0 { "    " } else { "  " };
                                    let note_is_activity = note.system;
                                    let note_is_resolved = !note_is_activity && is_resolved;

                                    let (
                                        note_prefix,
                                        note_prefix_style,
                                        note_author_style,
                                        note_content_style,
                                    ) = if note_is_activity {
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

                                    text.push(Line::from(vec![
                                        Span::styled(indent, note_content_style),
                                        Span::styled(note_prefix, note_prefix_style),
                                        Span::styled(
                                            format!("@{}", note.author.username),
                                            note_author_style,
                                        ),
                                        Span::styled(":", note_content_style),
                                    ]));

                                    // Render markdown body with indentation
                                    let markdown_lines = app.markdown_cached(&note.body);
                                    for line in markdown_lines.iter() {
                                        let mut line_spans = vec![Span::styled(
                                            format!("{}  ", indent),
                                            note_content_style,
                                        )];
                                        for span in &line.spans {
                                            let mut s = span.clone();
                                            if note_is_resolved || note_is_activity {
                                                s.style = s
                                                    .style
                                                    .patch(Style::default().fg(app.theme.dim));
                                            }
                                            line_spans.push(s);
                                        }
                                        text.push(Line::from(line_spans));
                                    }
                                    text.push(Line::from("")); // Spacer between notes
                                }
                            }
                            let mut item =
                                ListItem::new(text).style(Style::default().bg(app.theme.code_bg));
                            if is_cursor && !is_file_list_focused {
                                item = item.style(
                                    Style::default()
                                        .bg(app.theme.highlight_bg)
                                        .add_modifier(Modifier::BOLD),
                                );
                            } else if is_in_manual_selection && !is_file_list_focused {
                                item = item.style(Style::default().bg(app.theme.range_highlight));
                            }
                            item
                        }
                    }
                })
                .collect();

            if mr_diff.display_lines.is_empty() {
                f.render_widget(
                    Paragraph::new("  (No diff changes to display for this file.)")
                        .fg(app.theme.dim)
                        .block(Block::default().title(format!(" {} ", mr_diff.new_path))),
                    workspace_chunks[1],
                );
            } else if app.diff_side_by_side {
                let sbs_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(workspace_chunks[1]);

                let mut left_lines = Vec::new();
                let mut right_lines = Vec::new();

                for (idx, dl) in mr_diff.display_lines.iter().enumerate() {
                    let is_cursor = focused_line_idx == Some(idx);
                    let is_in_manual_selection = if let Some(start) = app.diff_selection_start {
                        let current = focused_line_idx.unwrap_or(0);
                        let s = start.min(current);
                        let e = start.max(current);
                        idx >= s && idx <= e
                    } else {
                        false
                    };

                    let (left_item, right_item) = match dl {
                        DiffDisplayLine::Code(line, old_line, new_line) => {
                            let (l_text, l_style, r_text, r_style) = if line.starts_with('+') {
                                (
                                    "".to_string(),
                                    Style::default(),
                                    format!("{:4} │ {}", new_line.unwrap_or(0), line),
                                    Style::default().fg(app.theme.success),
                                )
                            } else if line.starts_with('-') {
                                (
                                    format!("{:4} │ {}", old_line.unwrap_or(0), line),
                                    Style::default().fg(app.theme.failed),
                                    "".to_string(),
                                    Style::default(),
                                )
                            } else if line.starts_with("@@") {
                                (
                                    line.clone(),
                                    Style::default().fg(app.theme.shortcut_key),
                                    line.clone(),
                                    Style::default().fg(app.theme.shortcut_key),
                                )
                            } else {
                                (
                                    format!("{:4} │ {}", old_line.unwrap_or(0), line),
                                    Style::default(),
                                    format!("{:4} │ {}", new_line.unwrap_or(0), line),
                                    Style::default(),
                                )
                            };

                            let mut li = ListItem::new(Line::from(Span::styled(l_text, l_style)));
                            let mut ri = ListItem::new(Line::from(Span::styled(r_text, r_style)));
                            if !is_file_list_focused {
                                if is_cursor {
                                    li = li.style(
                                        Style::default()
                                            .bg(app.theme.highlight_bg)
                                            .add_modifier(Modifier::BOLD),
                                    );
                                    ri = ri.style(
                                        Style::default()
                                            .bg(app.theme.highlight_bg)
                                            .add_modifier(Modifier::BOLD),
                                    );
                                } else if is_in_manual_selection {
                                    li = li.style(Style::default().bg(app.theme.range_highlight));
                                    ri = ri.style(Style::default().bg(app.theme.range_highlight));
                                }
                            }
                            (li, ri)
                        }
                        DiffDisplayLine::Discussion(_) => {
                            let item = diff_lines
                                .get(idx)
                                .cloned()
                                .unwrap_or_else(|| ListItem::new(""));
                            (item.clone(), item)
                        }
                    };
                    left_lines.push(left_item);
                    right_lines.push(right_item);
                }

                let left_list = List::new(left_lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .title(format!(" Base: {} ", mr_diff.old_path)),
                    )
                    .highlight_style(Style::default())
                    .highlight_symbol("> ");
                let right_list = List::new(right_lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .title(format!(" Head: {} ", mr_diff.new_path)),
                    )
                    .highlight_style(Style::default())
                    .highlight_symbol("> ");

                let mut state_clone = app.diff_line_state;
                f.render_stateful_widget(left_list, sbs_chunks[0], &mut app.diff_line_state);
                f.render_stateful_widget(right_list, sbs_chunks[1], &mut state_clone);
            } else {
                let diff_content = List::new(diff_lines)
                    .block(Block::default().title(format!(" {} ", mr_diff.new_path)))
                    .highlight_style(Style::default()) // Manual highlight logic used above
                    .highlight_symbol("> ");

                f.render_stateful_widget(
                    diff_content,
                    workspace_chunks[1],
                    &mut app.diff_line_state,
                );
            }
        }
    } else {
        f.render_widget(
            Paragraph::new(" No file selected.").fg(app.theme.dim),
            workspace_chunks[1],
        );
    }

    // 3. Render Shortcuts Bar (Bottom)
    let shortcuts_spans = vec![
        Span::styled("[h/l]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" Focus  ", Style::default().fg(app.theme.dim)),
        Span::styled("[j/k]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" Move  ", Style::default().fg(app.theme.dim)),
        Span::styled("[w]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(if app.diff_side_by_side { " Unified  " } else { " Side-by-Side  " }, Style::default().fg(app.theme.dim)),
        Span::styled("[[/]]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" File/Hunk  ", Style::default().fg(app.theme.dim)),
        Span::styled("[c]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" Comment  ", Style::default().fg(app.theme.dim)),
        Span::styled("[C]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" File Note  ", Style::default().fg(app.theme.dim)),
        Span::styled("[r]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" Reply  ", Style::default().fg(app.theme.dim)),
        Span::styled("[Space]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" Fold  ", Style::default().fg(app.theme.dim)),
        Span::styled("[s]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" Resolve  ", Style::default().fg(app.theme.dim)),
        Span::styled("[n/N]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" Disc  ", Style::default().fg(app.theme.dim)),
        Span::styled("[A]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" Approve  ", Style::default().fg(app.theme.dim)),
        Span::styled("[M]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" Merge  ", Style::default().fg(app.theme.dim)),
        Span::styled("[D]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" Draft  ", Style::default().fg(app.theme.dim)),
        Span::styled("[y]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" Copy  ", Style::default().fg(app.theme.dim)),
        Span::styled("[Esc]", Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
        Span::styled(" Back", Style::default().fg(app.theme.dim)),
    ];
    f.render_widget(Paragraph::new(Line::from(shortcuts_spans)), chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Author, MRDiff, MergeRequest, References};
    use gitlab::GitlabBuilder;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use wiremock::matchers::path;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_render_diff_view() {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id":1,"username":"u"})),
            )
            .mount(&server)
            .await;

        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();
        let client = GitlabBuilder::new(server.uri().replace("http://", ""), "t")
            .insecure()
            .build_async()
            .await
            .unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(client, tx, None);

        app.selected_mr = Some(MergeRequest {
            iid: 1,
            project_id: 1,
            title: "T".into(),
            description: Some("D".into()),
            author: Author::new(1, "u".into()),
            assignees: vec![],
            reviewers: vec![],
            state: "opened".into(),
            web_url: "u".into(),
            references: Some(References { full: "r".into() }),
            upvotes: Some(0),
            downvotes: Some(0),
            labels: vec![],
            milestone: None,
            created_at: Some("t".into()),
            draft: Some(false),
            squash: Some(false),
            diff_refs: None,
            reactions: vec![],
            merge_status: None,
            detailed_merge_status: None,
            has_conflicts: None,
            head_pipeline: None,
            source_branch: "".into(),
            target_branch: "".into(),
            participants: vec![],
        });

        app.mr_diffs = vec![MRDiff {
            old_path: "a.txt".into(),
            new_path: "b.txt".into(),
            diff: "-old\n+new\n unchanged".into(),
            collapsed: false,
            display_lines: vec![
                DiffDisplayLine::Code("@@ -1 +1 @@".into(), None, None),
                DiffDisplayLine::Code("-old".into(), Some(1), None),
                DiffDisplayLine::Code("+new".into(), None, Some(1)),
                DiffDisplayLine::Code(" unchanged".into(), Some(2), Some(2)),
            ],
        }];
        app.mr_diff_state.select(Some(0));

        terminal
            .draw(|f| {
                render_diff_view(f, &mut app, f.area());
            })
            .unwrap();

        let mut content = String::new();
        let buffer = terminal.backend().buffer();
        for y in 0..buffer.area().height {
            for x in 0..buffer.area().width {
                content.push(buffer[(x, y)].symbol().chars().next().unwrap());
            }
            content.push('\n');
        }
        assert!(content.contains(" b.txt"));
        assert!(content.contains("-old"));
        assert!(content.contains("+new"));
        assert!(content.contains(" unchanged"));
    }
}
