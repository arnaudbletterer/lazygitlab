use ratatui::{prelude::*, widgets::*};
use crate::models::App;
use crate::ui::widgets::{centered_rect, hex_to_color};

pub fn render_label_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 60, f.area());
    app.set_rect("popup_label", area);
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);


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
    app.set_rect("popup_assignee", area);
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

pub fn render_reviewer_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 60, f.area());
    app.set_rect("popup_reviewer", area);
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);

    let project_id = app.selected_mr.as_ref().map(|m| m.project_id)
        .or_else(|| app.selected_issue.as_ref().map(|i| i.project_id))
        .unwrap_or(0);

    let members_source = app.project_member_cache.get(&project_id).unwrap_or(&app.project_members);

    let members: Vec<ListItem> = members_source
        .iter()
        .map(|m| {
            let prefix = if app.selected_reviewer_ids.contains(&m.id) { "[x] " } else { "[ ] " };
            ListItem::new(format!("{}{}", prefix, m.username))
        })
        .collect();

    let list = List::new(members)
        .block(Block::default().borders(Borders::ALL).title(" Edit Reviewers (Space to toggle, Enter to save) "))
        .highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, area, &mut app.reviewer_state);
}

pub fn render_milestone_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 40, f.area());
    app.set_rect("popup_milestone", area);
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
    app.set_rect("popup_weight", area);
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

pub fn render_target_branch_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 40, f.area());
    app.set_rect("popup_target_branch", area);
    f.render_widget(Clear, area);
    f.render_widget(Block::default().bg(Color::Black), area);

    let branches: Vec<ListItem> = app.project_branches
        .iter()
        .map(|b| ListItem::new(b.name.clone()))
        .collect();

    let list = List::new(branches)
        .block(Block::default().borders(Borders::ALL).title(" Select Target Branch (Enter to save) "))
        .highlight_style(Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, area, &mut app.target_branch_state);
}

