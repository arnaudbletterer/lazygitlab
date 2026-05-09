use ratatui::{prelude::*, widgets::*};
use crate::models::App;
use crate::ui::widgets::render_shortcuts;

pub fn render_diff_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title/File selection
            Constraint::Min(0),    // Diff content
            Constraint::Length(1), // Shortcuts
        ])
        .split(area);

    let mr_title = app.selected_mr.as_ref().map(|mr| format!("!{} - {}", mr.iid, mr.title)).unwrap_or_else(|| "Merge Request Diff".to_string());
    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", mr_title))
        .border_style(Style::default().fg(app.theme.border));
    
    if app.mr_diffs.is_empty() {
        f.render_widget(Paragraph::new(" No diffs available or still loading...").block(title_block), chunks[1]);
    } else {
        let diff_items: Vec<ListItem> = app.mr_diffs.iter().map(|d| {
            ListItem::new(format!(" {} -> {}", d.old_path, d.new_path))
        }).collect();

        let diff_list = List::new(diff_items)
            .block(title_block)
            .highlight_style(Style::default().bg(app.theme.highlight_bg))
            .highlight_symbol(">> ");
        f.render_stateful_widget(diff_list, chunks[0], &mut app.mr_diff_state);

        if let Some(selected_idx) = app.mr_diff_state.selected() {
            if let Some(mr_diff) = app.mr_diffs.get(selected_idx) {
                let diff_lines: Vec<ListItem> = mr_diff.diff.lines().map(|line| {
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
                }).collect();

                let diff_list = List::new(diff_lines)
                    .block(Block::default().borders(Borders::ALL).title(format!(" {} ", mr_diff.new_path)))
                    .highlight_style(Style::default().bg(app.theme.highlight_bg))
                    .highlight_symbol("> ");
                
                f.render_stateful_widget(diff_list, chunks[1], &mut app.diff_line_state);
            }
        }
    }

    render_shortcuts(f, chunks[2], vec![("j/k", "nav"), ("h/l", "file/line"), ("c", "comment"), ("esc", "back")], &app.theme);
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
        assert!(content.contains("a.txt -> b.txt"));
        assert!(content.contains("-old"));
        assert!(content.contains("+new"));
        assert!(content.contains("unchanged"));
    }
}
