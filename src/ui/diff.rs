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
            let diff_lines: Vec<ListItem> = mr_diff.display_lines.iter().map(|dl| {
                match dl {
                    DiffDisplayLine::Code(line, ..) => {
                        let style = if line.starts_with('+') {
                            Style::default().fg(Color::Green)
                        } else if line.starts_with('-') {
                            Style::default().fg(Color::Red)
                        } else if line.starts_with("@@") {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default()
                        };
                        ListItem::new(Line::from(Span::styled(line, style)))
                    }
                    DiffDisplayLine::Discussion(disc) => {
                        let is_collapsed = disc.collapsed;
                        let is_resolved = disc.resolved.unwrap_or(false);
                        let mut text = Vec::new();
                        
                        let prefix = if is_resolved { " [RESOLVED] " } else { "" };
                        let prefix_style = if is_resolved { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Yellow) };

                        if is_collapsed {
                            let first_note = disc.notes.first();
                            let author = first_note.map(|n| n.author.username.as_str()).unwrap_or("?");
                            let body = first_note.map(|n| n.body.chars().take(40).collect::<String>()).unwrap_or_else(|| "...".into());
                            text.push(Line::from(vec![
                                Span::styled(" [+] ", prefix_style.add_modifier(Modifier::BOLD)),
                                if is_resolved { Span::styled(prefix, prefix_style) } else { Span::raw("") },
                                Span::styled(format!("@{}", author), Style::default().fg(app.theme.author)),
                                Span::raw(": "),
                                Span::raw(body.replace('\n', " ")),
                                if first_note.map_or(false, |n| n.body.len() > 40) { Span::raw("...") } else { Span::raw("") },
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
                                    Span::raw(indent),
                                    Span::styled(format!("@{}", note.author.username), Style::default().fg(app.theme.author).add_modifier(Modifier::BOLD)),
                                    Span::raw(":"),
                                ]));
                                
                                // Render markdown body with indentation
                                let markdown_lines = render_markdown(&note.body, &app.theme);
                                for line in markdown_lines {
                                    let mut line_spans = vec![Span::raw(format!("{}  ", indent))];
                                    line_spans.extend(line.spans);
                                    text.push(Line::from(line_spans));
                                }
                                text.push(Line::from("")); // Spacer between notes
                            }
                        }
                        ListItem::new(text).style(Style::default().bg(Color::Rgb(30, 30, 30)))
                    }                }
            }).collect();

            let diff_content = List::new(diff_lines)
                .block(Block::default().title(format!(" {} ", mr_diff.new_path)))
                .highlight_style(if !is_file_list_focused { highlight_style } else { Style::default() })
                .highlight_symbol("> ");
            
            f.render_stateful_widget(diff_content, workspace_chunks[1], &mut app.diff_line_state);
        }
    } else {
        f.render_widget(Paragraph::new(" No file selected.").fg(app.theme.dim), workspace_chunks[1]);
    }

    render_shortcuts(f, chunks[2], vec![("j/k", "nav"), ("h/l", "file/line"), ("c", "comment"), ("r", "reply"), ("A", "approve"), ("esc", "back")], &app.theme);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::models::{MRDiff, MergeRequest, Author, References};
    use gitlab::GitlabBuilder;

    #[tokio::test]
    async fn test_render_diff_view() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::any()).respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        let client = GitlabBuilder::new(server.uri().replace("http://", ""), "t").insecure().build_async().await.unwrap();
        let mut app = App::new(client);

        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        // Empty diffs
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
            labels: vec![],
            milestone: None,
            created_at: Some("t".into()),
            upvotes: Some(0),
            downvotes: Some(0),
            reactions: vec![],
            draft: Some(false),
            diff_refs: None,
        });
        terminal.draw(|f| render_diff_view(f, &mut app, f.area())).unwrap();

        // With diffs
        app.mr_diffs = vec![
            MRDiff {
                old_path: "a.txt".into(),
                new_path: "b.txt".into(),
                diff: "@@ -1 +1 @@\n-old\n+new\n unchanged".into(),
                display_lines: vec![
                    DiffDisplayLine::Code("@@ -1 +1 @@".into(), None, None),
                    DiffDisplayLine::Code("-old".into(), Some(1), None),
                    DiffDisplayLine::Code("+new".into(), None, Some(1)),
                    DiffDisplayLine::Code(" unchanged".into(), Some(2), Some(2)),
                ],
            }
        ];
        app.mr_diff_state.select(Some(0));

        terminal.draw(|f| render_diff_view(f, &mut app, f.area())).unwrap();
        let buffer = terminal.backend().buffer();
        let mut content = String::new();
        for y in 0..buffer.area().height {
            for x in 0..buffer.area().width {
                content.push(buffer[(x, y)].symbol().chars().next().unwrap_or(' '));
            }
            content.push('\n');
        }
        assert!(content.contains(" b.txt"));
        assert!(content.contains("-old"));
        assert!(content.contains("+new"));
        assert!(content.contains(" unchanged"));
    }
}
