use ratatui::{prelude::*, widgets::*};
use crate::models::{App, MenuSelection};
use crate::ui::widgets::{render_shortcuts, get_status_style, hex_to_color};

pub fn render_list_view(f: &mut Frame, app: &mut App) {
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

    let proj_name = app.selected_project.as_ref().map(|p| p.name.as_str()).unwrap_or("Dashboard");
    let status_info = Paragraph::new(format!("Context: {}", proj_name))
        .block(Block::default().borders(Borders::ALL).title(" Context ").border_style(Style::default().fg(app.theme.border)));
    f.render_widget(status_info, left_chunks[0]);

    let items: Vec<ListItem> = app.menu_items.iter().map(|(name, _)| ListItem::new(name.as_str())).collect();
    let sidebar_style = if app.focus_on_menu { Style::default().fg(app.theme.border) } else { Style::default() };
    let sidebar_highlight_style = if app.focus_on_menu { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Navigation ").border_style(sidebar_style))
        .highlight_style(sidebar_highlight_style)
        .highlight_symbol(if app.focus_on_menu { ">> " } else { "   " });
    f.render_stateful_widget(list, left_chunks[1], &mut app.menu_state);

    let content_style = if !app.focus_on_menu { Style::default().fg(app.theme.border) } else { Style::default() };
    let content_highlight_style = if !app.focus_on_menu { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() };

    let content_chunks = if app.show_filter_input {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Filter input
                Constraint::Min(0),    // List
            ])
            .split(main_chunks[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(main_chunks[1])
    };

    if app.show_filter_input {
        let filter_block = Block::default()
            .borders(Borders::ALL)
            .title(" Filter (Esc to clear) ")
            .border_style(Style::default().fg(app.theme.highlight_bg));
        let filter_paragraph = Paragraph::new(app.filter_input.as_str())
            .block(filter_block);
        f.render_widget(filter_paragraph, content_chunks[0]);
    }

    let list_area = if app.show_filter_input { content_chunks[1] } else { content_chunks[0] };

    let is_global = app.selected_project.is_none();

    match app.active_selection {
        MenuSelection::Projects => {
            let projects: Vec<ListItem> = app.filtered_projects.iter().map(|p| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:<10} ", p.id), Style::default().fg(Color::Cyan)),
                    Span::raw(&p.path_with_namespace),
                ]))
            }).collect();
            f.render_stateful_widget(List::new(projects).block(Block::default().borders(Borders::ALL).title(" Explore Projects (Enter to Select) ").border_style(content_style)).highlight_style(content_highlight_style).highlight_symbol(if !app.focus_on_menu { "> " } else { "  " }), list_area, &mut app.projects_state);
        }
        MenuSelection::Groups => {
            let groups: Vec<ListItem> = app.filtered_groups.iter().map(|g| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:<10} ", g.id), Style::default().fg(Color::Cyan)),
                    Span::raw(&g.full_path),
                ]))
            }).collect();
            f.render_stateful_widget(List::new(groups).block(Block::default().borders(Borders::ALL).title(" Explore Groups ").border_style(content_style)).highlight_style(content_highlight_style).highlight_symbol(if !app.focus_on_menu { "> " } else { "  " }), list_area, &mut app.groups_state);
        }
        MenuSelection::Todos => {
            let todos: Vec<ListItem> = app.filtered_todos.iter().map(|todo| {
                let status_style = if todo.state == "pending" { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Green) };
                let line = Line::from(vec![
                    Span::styled(format!("[{}]", todo.state), status_style),
                    Span::raw(format!(" {}: {} - ", todo.project.name, todo.action_name)),
                    Span::raw(&todo.body),
                ]);
                ListItem::new(line)
            }).collect();
            f.render_stateful_widget(List::new(todos).block(Block::default().borders(Borders::ALL).title(" Global Dashboard: Todos ").border_style(content_style)).highlight_style(content_highlight_style).highlight_symbol(if !app.focus_on_menu { "> " } else { "  " }), list_area, &mut app.todos_state);
        }
        MenuSelection::MergeRequests => {
            let mrs: Vec<ListItem> = app.filtered_mrs.iter().map(|mr| {
                let state_style = get_status_style(&mr.state, &app.theme);
                let mut line_spans = vec![
                    Span::styled(format!("[{}]", mr.state), state_style),
                    Span::raw(format!(" {} - {} by ", mr.references.full, mr.title)),
                    Span::styled(format!("@{}", mr.author.username), Style::default().fg(app.theme.author)),
                ];
                if mr.upvotes > 0 || mr.downvotes > 0 {
                    line_spans.push(Span::raw(" | "));
                    if mr.upvotes > 0 {
                        line_spans.push(Span::styled(format!("👍 {}", mr.upvotes), Style::default().fg(Color::Green)));
                        line_spans.push(Span::raw(" "));
                    }
                    if mr.downvotes > 0 {
                        line_spans.push(Span::styled(format!("👎 {}", mr.downvotes), Style::default().fg(Color::Red)));
                    }
                }
                ListItem::from(Line::from(line_spans))
            }).collect();
            let title = if is_global { " Global Dashboard: Assigned MRs ".to_string() } else { format!(" MRs for {} ", proj_name) };
            f.render_stateful_widget(List::new(mrs).block(Block::default().borders(Borders::ALL).title(title).border_style(content_style)).highlight_style(content_highlight_style).highlight_symbol(if !app.focus_on_menu { "> " } else { "  " }), list_area, &mut app.mrs_state);
        }
        MenuSelection::Issues => {
            let issues: Vec<ListItem> = app.filtered_issues.iter().map(|issue| {
                let state_style = get_status_style(&issue.state, &app.theme);
                let mut spans = vec![
                    Span::styled(format!("[{}]", issue.state), state_style),
                    Span::styled(format!(" [{}]", issue.issue_type.as_deref().unwrap_or("issue")), Style::default().fg(app.theme.dim)),
                    Span::raw(format!(" #{} - {} by ", issue.iid, issue.title)),
                    Span::styled(format!("@{}", issue.author.username), Style::default().fg(app.theme.author)),
                ];
                for label in &issue.labels {
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(format!(" {} ", label.name), Style::default().bg(hex_to_color(&label.color)).fg(hex_to_color(&label.text_color)).add_modifier(Modifier::BOLD)));
                }
                if issue.upvotes > 0 || issue.downvotes > 0 {
                    spans.push(Span::raw(" | "));
                    if issue.upvotes > 0 {
                        spans.push(Span::styled(format!("👍 {}", issue.upvotes), Style::default().fg(Color::Green)));
                        spans.push(Span::raw(" "));
                    }
                    if issue.downvotes > 0 {
                        spans.push(Span::styled(format!("👎 {}", issue.downvotes), Style::default().fg(Color::Red)));
                    }
                }
                ListItem::from(Line::from(spans))
            }).collect();
            let title = if is_global { " Global Dashboard: Assigned Issues ".to_string() } else { format!(" Issues for {} ", proj_name) };
            f.render_stateful_widget(List::new(issues).block(Block::default().borders(Borders::ALL).title(title).border_style(content_style)).highlight_style(content_highlight_style).highlight_symbol(if !app.focus_on_menu { "> " } else { "  " }), list_area, &mut app.issues_state);
        }
        MenuSelection::Pipelines => {
            let pipelines_title = if app.is_loading { format!(" Pipelines for {} (Refreshing...) ", proj_name) } else { format!(" Pipelines for {} [LIVE] ", proj_name) };
            let pipelines_style = if app.is_loading { Style::default().fg(Color::Yellow) } else if !app.focus_on_menu { Style::default().fg(Color::Green) } else { content_style };

            if is_global {
                 f.render_widget(Paragraph::new("\n\n  Select a project from 'Explore Projects' to view pipelines.").block(Block::default().borders(Borders::ALL).title(" Pipelines ").border_style(pipelines_style)), list_area);
            } else if app.filtered_pipelines.is_empty() {
                f.render_widget(Paragraph::new("No pipelines found").block(Block::default().borders(Borders::ALL).title(pipelines_title).border_style(pipelines_style)), list_area);
            } else {
                let pipelines: Vec<ListItem> = app.filtered_pipelines.iter().map(|pipeline| {
                    let state_style = get_status_style(&pipeline.status, &app.theme);
                    let mut spans = vec![
                        Span::styled(format!("[{}]", pipeline.status), state_style),
                        Span::raw(format!(" #{} - {} from ", pipeline.id, pipeline.source)),
                        Span::styled(pipeline.ref_name.clone(), Style::default().fg(app.theme.border).add_modifier(Modifier::BOLD)),
                    ];

                    if pipeline.status == "running" {
                        if let Some(started) = &pipeline.started_at {
                            spans.push(Span::raw(" | "));
                            spans.push(Span::styled(crate::ui::widgets::get_elapsed_time(started), Style::default().fg(Color::Yellow)));
                        }
                    } else if let Some(duration) = pipeline.duration {
                        spans.push(Span::raw(" | "));
                        spans.push(Span::styled(crate::ui::widgets::format_duration(duration), Style::default().fg(app.theme.dim)));
                    }

                    ListItem::from(Line::from(spans))
                }).collect();
                f.render_stateful_widget(List::new(pipelines).block(Block::default().borders(Borders::ALL).title(pipelines_title).border_style(pipelines_style)).highlight_style(content_highlight_style).highlight_symbol(if !app.focus_on_menu { "> " } else { "  " }), list_area, &mut app.pipelines_state);
            }
        }
    }

    let shortcuts = if app.show_filter_input {
        vec![("j/k", "nav"), ("esc", "clear filter"), ("tab", "switch"), ("ctrl+r", "switch project"), ("enter", "select"), ("o", "open"), ("q", "quit")]
    } else {
        vec![("j/k", "nav"), ("/", "filter"), ("tab", "switch"), ("ctrl+r", "switch project"), ("enter", "select"), ("o", "open"), ("q", "quit")]
    };
    render_shortcuts(f, chunks[1], shortcuts, &app.theme);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::models::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::path;

    async fn setup_app() -> (MockServer, App) {
        use gitlab::GitlabBuilder;
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        let mut app = App::new(client);
        app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: "n/p".into(), web_url: "u".into() });
        (server, app)
    }

    #[test]
    fn test_render_list_view_all_selections() {
        let (_server, mut app) = tokio_test::block_on(setup_app());
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        for sel in [MenuSelection::MergeRequests, MenuSelection::Issues, MenuSelection::Pipelines] {
            app.active_selection = sel;
            // Empty
            terminal.draw(|f| render_list_view(f, &mut app)).unwrap();
        }

        // With data
        let mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 1, "downvotes": 1, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z"}"##;
        app.merge_requests = vec![serde_json::from_str(mr_json).unwrap()];
        
        let issue_json = r##"{"iid": 1, "project_id": 1, "title": "I", "description": "d", "author": {"id": 1, "username": "u"}, "assignees": [], "state": "opened", "web_url": "u", "labels": [{"id":1,"name":"l","color":"#ff0000","text_color":"#ffffff"}], "milestone": null, "due_date": null, "weight": 5, "confidential": false, "created_at": "2026-04-25T12:00:00Z", "upvotes": 1, "downvotes": 1, "issue_type": "issue"}"##;
        app.issues = vec![serde_json::from_str(issue_json).unwrap()];

        let pipeline_json = r##"{"id": 100, "project_id": 1, "status": "running", "source": "push", "ref": "master", "web_url": "url", "created_at": "2026-04-25T12:00:00Z", "started_at": "2026-04-25T12:00:00Z", "duration": 100.0}"##;
        app.pipelines = vec![serde_json::from_str(pipeline_json).unwrap()];

        for sel in [MenuSelection::MergeRequests, MenuSelection::Issues, MenuSelection::Pipelines] {
            app.active_selection = sel;
            for focus in [true, false] {
                app.focus_on_menu = focus;
                terminal.draw(|f| render_list_view(f, &mut app)).unwrap();
            }
        }
    }

    #[test]
    fn test_render_list_view_no_project() {
        let (_server, mut app) = tokio_test::block_on(setup_app());
        app.selected_project = None;
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_list_view(f, &mut app)).unwrap();
    }
}
