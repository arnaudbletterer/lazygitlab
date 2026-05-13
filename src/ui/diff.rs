use ratatui::{prelude::*, widgets::*};
use crate::models::{App, DiffDisplayLine};
use crate::ui::widgets::render_shortcuts;
use crate::ui::markdown::render_markdown;

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
                Span::styled(format!(" !{} ", mr.iid), Style::default().bg(app.theme.border).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(&mr.title, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw(" Author: "), Span::styled(format!("@{}", mr.author.username), Style::default().fg(app.theme.author)),
                Span::raw("  State: "), Span::styled(&mr.state, Style::default().fg(if mr.state == "opened" { Color::Green } else { Color::Red })),
                Span::raw("  Approvals: "), Span::styled(format!("+{}", mr.upvotes.unwrap_or(0)), Style::default().fg(app.theme.success)),
                Span::raw("  Comments: "), Span::styled(format!("{}", app.discussions.len()), Style::default().fg(Color::Yellow)),
            ]),
        ];
        if let Some(refs) = &mr.diff_refs {
            header_lines.push(Line::from(vec![
                Span::raw(" Range: "), Span::styled(format!("{}...{}", &refs.start_sha[..8], &refs.head_sha[..8]), Style::default().fg(app.theme.dim)),
            ]));
        }
        f.render_widget(Paragraph::new(header_lines).block(Block::default().borders(Borders::BOTTOM)), chunks[0]);
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
    let highlight_style = Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD);

    // File List (Left)
    let diff_items: Vec<ListItem> = app.mr_diffs.iter().map(|d| {
        ListItem::new(format!(" {}", d.new_path))
    }).collect();

    let diff_list = List::new(diff_items)
        .block(Block::default().borders(Borders::RIGHT).title(" Files "))
        .highlight_style(if is_file_list_focused { highlight_style } else { Style::default().fg(app.theme.dim) })
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
                                Some((range.start.old_line, range.start.new_line, range.end.old_line, range.end.new_line))
                            } else {
                                // Single line "range"
                                Some((pos.old_line, pos.new_line, pos.old_line, pos.new_line))
                            }
                        } else { None }
                    } else { None }
                } else { None }
            } else { None };

            let diff_lines: Vec<ListItem> = mr_diff.display_lines.iter().enumerate().map(|(idx, dl)| {
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
                        let is_in_hovered_range = if let Some((os, ns, oe, ne)) = hovered_range {
                            let match_old = old_line.is_some() && os.is_some() && oe.is_some() && *old_line >= os && *old_line <= oe;
                            let match_new = new_line.is_some() && ns.is_some() && ne.is_some() && *new_line >= ns && *new_line <= ne;
                            match_old || match_new
                        } else { false };

                        let style = if line.starts_with('+') {
                            Style::default().fg(Color::Green)
                        } else if line.starts_with('-') {
                            Style::default().fg(Color::Red)
                        } else if line.starts_with("@@") {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default()
                        };
                        
                        let mut item = ListItem::new(Line::from(Span::styled(line, style)));
                        if !is_file_list_focused {
                            if is_cursor {
                                item = item.style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD));
                            } else if is_in_manual_selection || is_in_hovered_range {
                                // Associated range highlight (using darker blue)
                                item = item.style(Style::default().bg(app.theme.range_highlight));
                            }
                        }
                        item
                    }
                    DiffDisplayLine::Discussion(disc) => {
                        let is_collapsed = disc.collapsed;
                        let is_resolved = disc.resolved.unwrap_or(false);
                        let mut text = Vec::new();
                        
                        let prefix = if is_resolved { " [RESOLVED] " } else { "" };
                        // Resolved threads use Dim color; Unresolved use bright Yellow
                        let prefix_style = if is_resolved { Style::default().fg(app.theme.dim) } else { Style::default().fg(Color::Yellow) };
                        let content_style = if is_resolved { Style::default().fg(app.theme.dim) } else { Style::default() };
                        let author_style = if is_resolved { Style::default().fg(app.theme.dim) } else { Style::default().fg(app.theme.author) };

                        if is_collapsed {
                            let first_note = disc.notes.first();
                            let author = first_note.map(|n| n.author.username.as_str()).unwrap_or("?");
                            let body = first_note.map(|n| n.body.chars().take(40).collect::<String>()).unwrap_or_else(|| "...".into());
                            text.push(Line::from(vec![
                                Span::styled(" [+] ", prefix_style.add_modifier(Modifier::BOLD)),
                                if is_resolved { Span::styled(prefix, prefix_style) } else { Span::raw("") },
                                Span::styled(format!("@{}", author), author_style),
                                Span::styled(": ", content_style),
                                Span::styled(body.replace('\n', " "), content_style),
                                if first_note.map_or(false, |n| n.body.len() > 40) { Span::styled("...", content_style) } else { Span::raw("") },
                                Span::styled(format!(" ({} more)", disc.notes.len().saturating_sub(1)), Style::default().fg(app.theme.dim)),
                            ]));
                        } else {
                            text.push(Line::from(vec![
                                Span::styled(" [-] Discussion Thread", prefix_style.add_modifier(Modifier::BOLD)),
                                if is_resolved { Span::styled(prefix, prefix_style) } else { Span::raw("") },
                            ]));
                            for (i, note) in disc.notes.iter().enumerate() {
                                let indent = if i > 0 { "    " } else { "  " };
                                text.push(Line::from(vec![
                                    Span::styled(indent, content_style),
                                    Span::styled(format!("@{}", note.author.username), if is_resolved { author_style } else { author_style.add_modifier(Modifier::BOLD) }),
                                    Span::styled(":", content_style),
                                ]));
                                
                                // Render markdown body with indentation
                                let markdown_lines = render_markdown(&note.body, &app.theme, &app.syntax_set, &app.theme_set);
                                for line in markdown_lines {
                                    let mut line_spans = vec![Span::styled(format!("{}  ", indent), content_style)];
                                    for span in line.spans {
                                        let mut s = span;
                                        if is_resolved {
                                            s.style = s.style.patch(Style::default().fg(app.theme.dim));
                                        }
                                        line_spans.push(s);
                                    }
                                    text.push(Line::from(line_spans));
                                }
                                text.push(Line::from("")); // Spacer between notes
                            }
                        }
                        let mut item = ListItem::new(text).style(Style::default().bg(Color::Rgb(30, 30, 30)));
                        if is_cursor && !is_file_list_focused {
                            item = item.style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD));
                        } else if is_in_manual_selection && !is_file_list_focused {
                            item = item.style(Style::default().bg(app.theme.range_highlight));
                        }
                        item
                    }
                }
            }).collect();

            let diff_content = List::new(diff_lines)
                .block(Block::default().title(format!(" {} ", mr_diff.new_path)))
                .highlight_style(Style::default()) // Manual highlight logic used above
                .highlight_symbol("> ");
            
            f.render_stateful_widget(diff_content, workspace_chunks[1], &mut app.diff_line_state);
        }
    } else {
        f.render_widget(Paragraph::new(" No file selected.").fg(app.theme.dim), workspace_chunks[1]);
    }

    render_shortcuts(f, chunks[2], vec![("j/k", "nav"), ("h/l", "file/line"), ("c", "comment"), ("r", "reply"), ("s", "resolve"), ("space", "exp/col"), ("ctrl+ent", "save"), ("esc", "back")], &app.theme);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::models::{MRDiff, MergeRequest, Author, References};
    use gitlab::GitlabBuilder;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::path;

    #[tokio::test]
    async fn test_render_diff_view() {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"})))
            .mount(&server)
            .await;

        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        let mut app = App::new(client);
        
        app.selected_mr = Some(MergeRequest {
            iid: 1,
            project_id: 1,
            title: "T".into(),
            description: Some("D".into()),
            author: Author { id: 1, username: "u".into() },
            assignees: vec![],
            state: "opened".into(),
            web_url: "u".into(),
            references: Some(References { full: "r".into() }),
            upvotes: Some(0),
            downvotes: Some(0),
            labels: vec![],
            milestone: None,
            created_at: Some("t".into()),
            draft: Some(false),
            diff_refs: None,
            reactions: vec![],
        });

        app.mr_diffs = vec![
            MRDiff {
                old_path: "a.txt".into(),
                new_path: "b.txt".into(),
                diff: "-old\n+new\n unchanged".into(),
                display_lines: vec![
                    DiffDisplayLine::Code("@@ -1 +1 @@".into(), None, None),
                    DiffDisplayLine::Code("-old".into(), Some(1), None),
                    DiffDisplayLine::Code("+new".into(), None, Some(1)),
                    DiffDisplayLine::Code(" unchanged".into(), Some(2), Some(2)),
                ],
            }
        ];
        app.mr_diff_state.select(Some(0));

        terminal.draw(|f| {
            render_diff_view(f, &mut app, f.area());
        }).unwrap();

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
