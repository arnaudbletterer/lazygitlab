use ratatui::{prelude::*, widgets::*};
use crate::models::{App, InputTarget, SearchResult};
use crate::ui::widgets::{centered_rect, render_shortcuts};

pub fn render_project_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);

    let projects: Vec<ListItem> = app.projects
        .iter()
        .map(|p| ListItem::new(format!("{} ({})", p.name, p.path_with_namespace.as_deref().unwrap_or(""))))
        .collect();

    let list = List::new(projects)
        .block(Block::default().borders(Borders::ALL).title(" Select Project ").border_style(Style::default().fg(app.theme.border)))
        .highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.projects_state);
}

pub fn render_label_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 60, f.area());
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);

    use crate::ui::widgets::hex_to_color;

    let project_id = app.selected_mr.as_ref().map(|m| m.project_id)
        .or_else(|| app.selected_issue.as_ref().map(|i| i.project_id))
        .unwrap_or(0);

    let labels_source = app.project_label_cache.get(&project_id).unwrap_or(&app.project_labels);

    let labels: Vec<ListItem> = labels_source
        .iter()
        .map(|l| {
            let prefix = if app.selected_label_ids.contains(&l.id()) { "[x] " } else { "[ ] " };
            let color = hex_to_color(l.color());
            let text_color = hex_to_color(l.text_color());

            ListItem::new(Line::from(vec![
                Span::raw(prefix),
                Span::styled(format!(" {} ", l.name()), Style::default().bg(color).fg(text_color)),
            ]))
        })
        .collect();

    let list = List::new(labels)
        .block(Block::default().borders(Borders::ALL).title(" Edit Labels (Space to toggle, Enter to save) "))
        .highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, area, &mut app.label_state);
}

pub fn render_assignee_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 60, f.area());
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);

    let project_id = app.selected_mr.as_ref().map(|m| m.project_id)
        .or_else(|| app.selected_issue.as_ref().map(|i| i.project_id))
        .unwrap_or(0);

    let members_source = app.project_member_cache.get(&project_id).unwrap_or(&app.project_members);

    let members: Vec<ListItem> = members_source
        .iter()
        .map(|m| {
            let prefix = if app.selected_assignee_ids.contains(&m.id) { "[x] " } else { "[ ] " };
            ListItem::new(format!("{}{}", prefix, m.username))
        })
        .collect();

    let list = List::new(members)
        .block(Block::default().borders(Borders::ALL).title(" Edit Assignees (Space to toggle, Enter to save) "))
        .highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, area, &mut app.assignee_state);
}

pub fn render_milestone_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 40, f.area());
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);

    let project_id = app.selected_mr.as_ref().map(|m| m.project_id)
        .or_else(|| app.selected_issue.as_ref().map(|i| i.project_id))
        .unwrap_or(0);

    let milestones_source = app.project_milestone_cache.get(&project_id).unwrap_or(&app.project_milestones);

    let mut milestones: Vec<ListItem> = milestones_source
        .iter()
        .map(|m| ListItem::new(m.title.clone()))
        .collect();

    milestones.insert(0, ListItem::new("None (Clear Milestone)"));

    let list = List::new(milestones)
        .block(Block::default().borders(Borders::ALL).title(" Select Milestone (Enter to save) "))
        .highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, area, &mut app.milestone_state);
}
pub fn render_weight_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(30, 40, f.area());
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);

    let weights: Vec<ListItem> = (0..=20)
        .map(|w| ListItem::new(w.to_string()))
        .collect();

    let list = List::new(weights)
        .block(Block::default().borders(Borders::ALL).title(" Select Weight "))
        .highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, area, &mut app.weight_state);
}

pub fn render_calendar_popup(f: &mut Frame, app: &mut App) {
    use chrono::{Datelike, NaiveDate};
    
    let area = centered_rect(40, 15, f.area());
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black).borders(Borders::ALL).title(format!(" Due Date: {} ", app.calendar_date.format("%B %Y"))), area);

    let inner = area.inner(Margin { horizontal: 1, vertical: 1 });
    let cols = [Constraint::Length(4); 7];
    let header_area = Rect::new(inner.x, inner.y, inner.width, 1);
    let days_area = Rect::new(inner.x, inner.y + 1, inner.width, inner.height - 1);

    // Days of week
    let dow = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let header = Row::new(dow.iter().map(|d| Cell::from(*d).style(Style::default().fg(app.theme.shortcut_key))));
    f.render_widget(Table::new(vec![header], cols), header_area);

    // Actual days
    let first_day = app.calendar_date.with_day(1).unwrap();
    let weekday_offset = first_day.weekday().number_from_monday() - 1;
    let days_in_month = if app.calendar_date.month() == 12 {
        NaiveDate::from_ymd_opt(app.calendar_date.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(app.calendar_date.year(), app.calendar_date.month() + 1, 1).unwrap()
    }.signed_duration_since(first_day).num_days();

    let mut rows = Vec::new();
    let mut current_row = vec![Cell::from(""); weekday_offset as usize];
    
    for day in 1..=days_in_month {
        let mut cell = Cell::from(day.to_string());
        if day == app.calendar_date.day() as i64 {
            cell = cell.style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD));
        }
        current_row.push(cell);
        if current_row.len() == 7 {
            rows.push(Row::new(current_row));
            current_row = Vec::new();
        }
    }
    if !current_row.is_empty() {
        while current_row.len() < 7 { current_row.push(Cell::from("")); }
        rows.push(Row::new(current_row));
    }

    f.render_widget(Table::new(rows, cols), days_area);
    
    let footer_area = Rect::new(area.x, area.y + area.height, area.width, 1);
    render_shortcuts(f, footer_area, vec![("arrows", "nav"), ("enter", "save"), ("del", "clear"), ("esc", "back")], &app.theme);
}

pub fn render_pipeline_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black).borders(Borders::ALL).title(" Launch Pipeline ").border_style(Style::default().fg(app.theme.border)), area);

    let inner = area.inner(Margin { horizontal: 2, vertical: 2 });
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Ref Input
            Constraint::Length(3), // Variable Key
            Constraint::Length(3), // Variable Value
            Constraint::Min(5),    // Variable List
            Constraint::Length(1), // Footer
        ])
        .split(inner);

    use crate::models::InputTarget;

    // Ref Input
    let ref_style = if let InputTarget::PipelineRef = app.input_target { Style::default().fg(Color::Yellow) } else { Style::default() };
    let ref_input = Paragraph::new(app.pipeline_ref_input.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Ref (Branch/Tag) ").border_style(ref_style));
    f.render_widget(ref_input, chunks[0]);

    // Variable Key Input
    let key_style = if let InputTarget::PipelineVariableKey = app.input_target { Style::default().fg(Color::Yellow) } else { Style::default() };
    let key_input = Paragraph::new(app.pipeline_variable_key_input.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Variable Key ").border_style(key_style));
    f.render_widget(key_input, chunks[1]);

    // Variable Value Input
    let val_style = if let InputTarget::PipelineVariableValue = app.input_target { Style::default().fg(Color::Yellow) } else { Style::default() };
    let val_input = Paragraph::new(app.pipeline_variable_value_input.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Variable Value ").border_style(val_style));
    f.render_widget(val_input, chunks[2]);

    // Variables List
    let items: Vec<ListItem> = app.pipeline_variables.iter()
        .map(|v| ListItem::new(format!("{}: {}", v.key, v.value)))
        .collect();
    let is_var_list_focused = app.input_target == InputTarget::PipelineVariableList;
    let var_list_style = if is_var_list_focused { Style::default().fg(Color::Yellow) } else { Style::default() };
    let highlight_style = if is_var_list_focused {
        Style::default().bg(app.theme.highlight_bg)
    } else {
        Style::default()
    };
    let highlight_symbol = if is_var_list_focused { "> " } else { "  " };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Variables ").border_style(var_list_style))
        .highlight_style(highlight_style)
        .highlight_symbol(highlight_symbol);
    f.render_stateful_widget(list, chunks[3], &mut app.pipeline_variables_state);

    // Shortcuts
    render_shortcuts(f, chunks[4], vec![("enter", "next"), ("p", "launch"), ("del", "remove var"), ("esc", "back")], &app.theme);

    // Dropdown for branches
    if app.show_branches_dropdown && !app.filtered_branches.is_empty() {
        let dropdown_area = Rect::new(
            chunks[0].x + 1,
            chunks[0].y + 3,
            chunks[0].width - 2,
            (app.filtered_branches.len() as u16).min(10).max(3),
        );
        f.render_widget(Clear, dropdown_area);
        let branches: Vec<ListItem> = app.filtered_branches.iter()
            .map(|b| ListItem::new(b.name.as_str()))
            .collect();
        let list = List::new(branches)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)))
            .highlight_style(Style::default().bg(app.theme.highlight_bg))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, dropdown_area, &mut app.branches_state);
    }
}

pub fn render_help_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);
    
    let block = Block::default()
        .bg(Color::Black)
        .borders(Borders::ALL)
        .title(" Help & Shortcuts (Esc to close, / to search) ")
        .border_style(Style::default().fg(app.theme.border));
    f.render_widget(block, area);

    let inner = area.inner(Margin { horizontal: 2, vertical: 2 });
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if app.show_help_filter { 3 } else { 0 }),
            Constraint::Min(0),
        ])
        .split(inner);

    if app.show_help_filter {
        let filter_block = Block::default()
            .borders(Borders::ALL)
            .title(" Search Command ")
            .border_style(Style::default().fg(app.theme.highlight_bg));
        let filter_paragraph = Paragraph::new(app.help_filter_input.as_str())
            .block(filter_block);
        f.render_widget(filter_paragraph, chunks[0]);

        f.set_cursor_position(Position::new(
            chunks[0].x + app.help_filter_input.chars().count() as u16 + 1,
            chunks[0].y + 1,
        ));
    }

    let items: Vec<ListItem> = app.filtered_help.iter()
        .map(|h| {
            let line = Line::from(vec![
                Span::styled(format!("{:<20}", h.shortcut), Style::default().fg(app.theme.shortcut_key).add_modifier(Modifier::BOLD)),
                Span::styled(&h.description, Style::default().fg(app.theme.shortcut_desc)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().bg(app.theme.highlight_bg))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[1], &mut app.help_state);
}

pub fn render_create_issue_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).title(" Create New Issue ").border_style(Style::default().fg(app.theme.border));
    f.render_widget(block, area);

    let inner = area.inner(Margin { horizontal: 2, vertical: 2 });
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title input
            Constraint::Min(0),    // Description input
            Constraint::Length(1), // Help
        ])
        .split(inner);

    // Title Field
    let title_style = if app.input_target == InputTarget::CreateIssueTitle { Style::default().fg(app.theme.highlight_bg) } else { Style::default() };
    let title_block = Block::default().borders(Borders::ALL).title(" Title (Tab to switch) ").border_style(title_style);
    if app.input_target == InputTarget::CreateIssueTitle {
        app.textarea.set_block(title_block);
        f.render_widget(&app.textarea, chunks[0]);
    } else {
        let title_para = Paragraph::new(app.create_issue_title.as_str()).block(title_block);
        f.render_widget(title_para, chunks[0]);
    }

    // Description Field
    let desc_style = if app.input_target == InputTarget::CreateIssueDescription { Style::default().fg(app.theme.highlight_bg) } else { Style::default() };
    let desc_block = Block::default().borders(Borders::ALL).title(" Description (Tab to switch, Ctrl+Enter to save) ").border_style(desc_style);
    if app.input_target == InputTarget::CreateIssueDescription {
        app.textarea.set_block(desc_block);
        f.render_widget(&app.textarea, chunks[1]);
    } else {
        let desc_para = Paragraph::new(app.create_issue_description.as_str()).block(desc_block).wrap(Wrap { trim: false });
        f.render_widget(desc_para, chunks[1]);
    }

    render_shortcuts(f, chunks[2], vec![("tab", "next field"), ("ctrl+enter", "save"), ("esc", "cancel")], &app.theme);
}

pub fn render_create_mr_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 60, f.area());
    f.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).title(" Create New Merge Request ").border_style(Style::default().fg(app.theme.border));
    f.render_widget(block, area);

    let inner = area.inner(Margin { horizontal: 2, vertical: 2 });
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Source Branch
            Constraint::Length(3), // Target Branch
            Constraint::Min(0),    // Description
            Constraint::Length(1), // Help
        ])
        .split(inner);

    let field_chunks = [chunks[0], chunks[1], chunks[2], chunks[3]];
    let targets = [InputTarget::CreateMRTitle, InputTarget::CreateMRSourceBranch, InputTarget::CreateMRTargetBranch, InputTarget::CreateMRDescription];
    let titles = [" Title ", " Source Branch ", " Target Branch ", " Description "];
    let contents = [&app.create_mr_title, &app.create_mr_source_branch, &app.create_mr_target_branch, &app.create_mr_description];

    for i in 0..4 {
        let is_focused = app.input_target == targets[i];
        let style = if is_focused { Style::default().fg(app.theme.highlight_bg) } else { Style::default() };
        let block = Block::default().borders(Borders::ALL).title(format!("{} (Tab to switch) ", titles[i])).border_style(style);
        
        if is_focused {
            app.textarea.set_block(block);
            f.render_widget(&app.textarea, field_chunks[i]);
        } else {
            let para = Paragraph::new(contents[i].as_str()).block(block);
            f.render_widget(para, field_chunks[i]);
        }
    }

    render_shortcuts(f, chunks[4], vec![("tab", "next field"), ("ctrl+enter", "save"), ("esc", "cancel")], &app.theme);
}

pub fn render_snooze_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(40, 30, f.area());
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black).borders(Borders::ALL).title(" Snooze Todo ").border_style(Style::default().fg(app.theme.border)), area);

    let suggestions = vec![
        "In 1 hour",
        "Tomorrow",
        "In 3 days",
        "Next week",
        "Custom date/time...",
    ];

    let items: Vec<ListItem> = suggestions.iter().map(|s| ListItem::new(*s)).collect();

    let list = List::new(items)
        .highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area.inner(Margin { horizontal: 2, vertical: 2 }), &mut app.snooze_state);
}

pub fn render_search_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black).borders(Borders::ALL).title(" Global Search ").border_style(Style::default().fg(app.theme.border)), area);

    let inner = area.inner(Margin { horizontal: 2, vertical: 2 });
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search Input
            Constraint::Min(0),    // Results
            Constraint::Length(1), // Footer
        ])
        .split(inner);

    // Search Input
    let search_style = if app.input_target == InputTarget::Search { Style::default().fg(app.theme.highlight_bg) } else { Style::default() };
    let search_block = Block::default().borders(Borders::ALL).title(" Search Query (Enter to search, Esc to close) ").border_style(search_style);
    let search_para = Paragraph::new(app.search_query.as_str()).block(search_block);
    f.render_widget(search_para, chunks[0]);

    if app.input_target == InputTarget::Search {
        f.set_cursor_position(Position::new(
            chunks[0].x + app.search_query.chars().count() as u16 + 1,
            chunks[0].y + 1,
        ));
    }

    // Results
    let items: Vec<ListItem> = app.search_results.iter().map(|res| {
        match res {
            SearchResult::Project(p) => ListItem::new(Line::from(vec![
                Span::styled(" [Project] ", Style::default().fg(Color::Cyan)),
                Span::raw(format!(" {}", p.path_with_namespace.as_deref().unwrap_or(&p.name))),
            ])),
            SearchResult::Issue(i) => ListItem::new(Line::from(vec![
                Span::styled(" [Issue]   ", Style::default().fg(Color::Green)),
                Span::raw(format!(" #{} {}", i.iid, i.title)),
            ])),
            SearchResult::MergeRequest(m) => ListItem::new(Line::from(vec![
                Span::styled(" [MR]      ", Style::default().fg(Color::Magenta)),
                Span::raw(format!(" !{} {}", m.iid, m.title)),
            ])),
        }
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(" Results ({}) ", app.search_results.len())))
        .highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, chunks[1], &mut app.search_state);

    // Footer
    render_shortcuts(f, chunks[2], vec![("arrows", "nav"), ("enter", "select"), ("esc", "close")], &app.theme);
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
        (server, App::new(client))
    }

    #[test]
    fn test_render_all_popups() {
        let (_server, mut app) = tokio_test::block_on(setup_app());
        let backend = TestBackend::new(120, 60);
        let mut terminal = Terminal::new(backend).unwrap();

        // 1. Project Popup
        app.projects = vec![Project { id: 1, name: "P1".into(), path_with_namespace: Some("ns/p1".into()), web_url: Some("u".into()), default_branch: None, namespace: None }];
        terminal.draw(|f| render_project_popup(f, &mut app)).unwrap();

        // 2. Label Popup
        app.project_labels = vec![Label { id: 1, name: "L1".into(), color: "#f00".into(), text_color: Some("#fff".into()) }];
        terminal.draw(|f| render_label_popup(f, &mut app)).unwrap();
        app.selected_label_ids.push(1);
        terminal.draw(|f| render_label_popup(f, &mut app)).unwrap();

        // 3. Assignee Popup
        app.project_members = vec![Author { id: 1, username: "u1".into() }];
        terminal.draw(|f| render_assignee_popup(f, &mut app)).unwrap();
        app.selected_assignee_ids.push(1);
        terminal.draw(|f| render_assignee_popup(f, &mut app)).unwrap();

        // 4. Milestone Popup
        app.project_milestones = vec![Milestone { id: 1, title: "M1".into() }];
        terminal.draw(|f| render_milestone_popup(f, &mut app)).unwrap();
        app.selected_milestone_id = Some(1);
        terminal.draw(|f| render_milestone_popup(f, &mut app)).unwrap();

        // 5. Weight Popup
        terminal.draw(|f| render_weight_popup(f, &mut app)).unwrap();

        // 6. Calendar Popup
        terminal.draw(|f| render_calendar_popup(f, &mut app)).unwrap();
        
        // Test December branch
        app.calendar_date = chrono::NaiveDate::from_ymd_opt(2023, 12, 1).unwrap();
        terminal.draw(|f| render_calendar_popup(f, &mut app)).unwrap();

        // Test month with exactly 28 days starting on Monday (no leftover row)
        app.calendar_date = chrono::NaiveDate::from_ymd_opt(2021, 2, 1).unwrap();
        terminal.draw(|f| render_calendar_popup(f, &mut app)).unwrap();

        // 7. Pipeline Popup
        app.input_target = InputTarget::PipelineRef;
        terminal.draw(|f| render_pipeline_popup(f, &mut app)).unwrap();

        app.input_target = InputTarget::PipelineVariableKey;
        terminal.draw(|f| render_pipeline_popup(f, &mut app)).unwrap();

        app.input_target = InputTarget::PipelineVariableValue;
        terminal.draw(|f| render_pipeline_popup(f, &mut app)).unwrap();

        app.pipeline_variables.push(PipelineVariable { key: "K".into(), value: "V".into() });
        app.input_target = InputTarget::PipelineVariableList;
        terminal.draw(|f| render_pipeline_popup(f, &mut app)).unwrap();

        app.show_branches_dropdown = true;
        app.filtered_branches = vec![
            Branch { name: "b1".into(), merged: None, protected: None, default: None, web_url: None },
            Branch { name: "b2".into(), merged: None, protected: None, default: None, web_url: None },
        ];
        terminal.draw(|f| render_pipeline_popup(f, &mut app)).unwrap();

        // 8. Help Popup
        app.show_help_popup = true;
        app.filter_help();
        terminal.draw(|f| render_help_popup(f, &mut app)).unwrap();
        app.show_help_filter = true;
        terminal.draw(|f| render_help_popup(f, &mut app)).unwrap();

        // 9. Create Issue Popup
        app.show_create_issue_popup = true;
        app.input_target = InputTarget::CreateIssueTitle;
        terminal.draw(|f| render_create_issue_popup(f, &mut app)).unwrap();
        app.input_target = InputTarget::CreateIssueDescription;
        terminal.draw(|f| render_create_issue_popup(f, &mut app)).unwrap();

        // 10. Create MR Popup
        app.show_create_mr_popup = true;
        for target in [InputTarget::CreateMRTitle, InputTarget::CreateMRSourceBranch, InputTarget::CreateMRTargetBranch, InputTarget::CreateMRDescription] {
            app.input_target = target;
            terminal.draw(|f| render_create_mr_popup(f, &mut app)).unwrap();
        }

        // 11. Snooze Popup
        app.show_snooze_popup = true;
        terminal.draw(|f| render_snooze_popup(f, &mut app)).unwrap();
    }
}
