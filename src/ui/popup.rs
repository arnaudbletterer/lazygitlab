use ratatui::{prelude::*, widgets::*};
use crate::models::App;
use crate::ui::widgets::{centered_rect, render_shortcuts};

pub fn render_project_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);

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

pub fn render_label_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 60, f.area());
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);

    let labels: Vec<ListItem> = app.project_labels
        .iter()
        .map(|l| {
            let prefix = if app.selected_label_ids.contains(&l.id) { "[x] " } else { "[ ] " };
            ListItem::new(format!("{}{}", prefix, l.name))
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

    let members: Vec<ListItem> = app.project_members
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

    let mut milestones: Vec<ListItem> = app.project_milestones
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
            .map(|b| ListItem::new(b.as_str()))
            .collect();
        let list = List::new(branches)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)))
            .highlight_style(Style::default().bg(app.theme.highlight_bg))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, dropdown_area, &mut app.branches_state);
    }
}
