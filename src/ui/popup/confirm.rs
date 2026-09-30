use crate::models::App;
use crate::ui::widgets::{centered_rect, get_contrast_color, get_status_style};
use ratatui::{prelude::*, widgets::*};

pub fn render_confirm_merge_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(65, 55, f.area());
    app.set_rect("popup_confirm_merge", area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .bg(app.theme.code_bg)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " Confirm Merge Request ",
            Style::default().fg(app.theme.title_fg).bold(),
        ))
        .border_style(Style::default().fg(app.theme.shortcut_key));

    f.render_widget(block, area);

    let inner = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Are you sure you want to merge this Merge Request?",
        Style::default().fg(app.theme.title_fg),
    )));
    lines.push(Line::from(""));

    if let Some(mr) = &app.selected_mr {
        lines.push(Line::from(vec![
            Span::styled(
                "Title: ",
                Style::default()
                    .fg(app.theme.title_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&mr.title, Style::default().fg(app.theme.title_fg)),
        ]));

        lines.push(Line::from(vec![
            Span::styled(
                "Branches: ",
                Style::default()
                    .fg(app.theme.title_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&mr.source_branch, Style::default().fg(app.theme.author)),
            Span::styled(" -> ", Style::default().fg(app.theme.dim)),
            Span::styled(&mr.target_branch, Style::default().fg(app.theme.project)),
        ]));

        lines.push(Line::from(vec![
            Span::styled(
                "[s] Squash commits: ",
                Style::default()
                    .fg(app.theme.shortcut_key)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if app.merge_squash {
                    "[X] Yes"
                } else {
                    "[ ] No"
                },
                Style::default()
                    .fg(if app.merge_squash {
                        app.theme.success
                    } else {
                        app.theme.dim
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled(
                "[d] Delete source branch: ",
                Style::default()
                    .fg(app.theme.shortcut_key)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if app.merge_remove_source_branch {
                    "[X] Yes"
                } else {
                    "[ ] No"
                },
                Style::default()
                    .fg(if app.merge_remove_source_branch {
                        app.theme.failed
                    } else {
                        app.theme.dim
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // Pipeline info if available
        if let Some(pipeline) = &mr.head_pipeline {
            lines.push(Line::from(vec![
                Span::styled(
                    "Pipeline: ",
                    Style::default()
                        .fg(app.theme.title_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &pipeline.status,
                    get_status_style(&pipeline.status, &app.theme),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    "Pipeline: ",
                    Style::default()
                        .fg(app.theme.title_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("None", Style::default().fg(app.theme.dim)),
            ]));
        }

        // Merge status info if available
        let detailed_status = mr.detailed_merge_status.as_deref().unwrap_or("unknown");
        let status_color = match detailed_status {
            "mergeable" => app.theme.success,
            "conflict" => app.theme.failed,
            _ => app.theme.pending,
        };
        lines.push(Line::from(vec![
            Span::styled(
                "Status: ",
                Style::default()
                    .fg(app.theme.title_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(detailed_status, Style::default().fg(status_color)),
        ]));

        if let Some(info) = &app.approvals {
            let approvers: Vec<String> = info
                .approved_by
                .iter()
                .filter_map(|a| a.user.as_ref().map(|u| format!("@{}", u.username)))
                .collect();
            lines.push(Line::from(vec![
                Span::styled(
                    "Approvals: ",
                    Style::default()
                        .fg(app.theme.title_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    match (info.approvals_left, info.approvals_required) {
                        (Some(left), Some(req)) => format!("{}/{} ", req.saturating_sub(left), req),
                        _ => String::new(),
                    },
                    Style::default().fg(app.theme.title_fg),
                ),
                Span::styled(
                    if approvers.is_empty() {
                        "none".to_string()
                    } else {
                        approvers.join(", ")
                    },
                    Style::default().fg(app.theme.success),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "[s] Toggle Squash",
            Style::default().fg(app.theme.shortcut_key),
        ),
        Span::raw("   "),
        Span::styled(
            "[d] Toggle Delete Branch",
            Style::default().fg(app.theme.shortcut_key),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "  [ Enter: Confirm ]  ",
            Style::default()
                .bg(app.theme.success)
                .fg(get_contrast_color(app.theme.success))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            "  [ Esc: Cancel ]  ",
            Style::default()
                .bg(app.theme.dim)
                .fg(get_contrast_color(app.theme.dim)),
        ),
    ]));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(paragraph, inner);
}

pub fn render_confirm_rebase_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 35, f.area());
    app.set_rect("popup_confirm_rebase", area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .bg(app.theme.code_bg)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " Confirm Rebase ",
            Style::default().fg(app.theme.title_fg).bold(),
        ))
        .border_style(Style::default().fg(app.theme.shortcut_key));

    f.render_widget(block, area);

    let inner = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Are you sure you want to rebase this Merge Request?",
        Style::default().fg(app.theme.title_fg),
    )));
    lines.push(Line::from(""));

    if let Some(mr) = &app.selected_mr {
        lines.push(Line::from(vec![
            Span::styled(
                "Title: ",
                Style::default()
                    .fg(app.theme.title_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&mr.title, Style::default().fg(app.theme.title_fg)),
        ]));

        lines.push(Line::from(vec![
            Span::styled(
                "Branches: ",
                Style::default()
                    .fg(app.theme.title_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&mr.source_branch, Style::default().fg(app.theme.author)),
            Span::styled(" -> ", Style::default().fg(app.theme.dim)),
            Span::styled(&mr.target_branch, Style::default().fg(app.theme.project)),
        ]));

        // Pipeline info if available
        if let Some(pipeline) = &mr.head_pipeline {
            lines.push(Line::from(vec![
                Span::styled(
                    "Pipeline: ",
                    Style::default()
                        .fg(app.theme.title_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &pipeline.status,
                    get_status_style(&pipeline.status, &app.theme),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    "Pipeline: ",
                    Style::default()
                        .fg(app.theme.title_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("None", Style::default().fg(app.theme.dim)),
            ]));
        }

        // Merge status info if available
        let detailed_status = mr.detailed_merge_status.as_deref().unwrap_or("unknown");
        let status_color = match detailed_status {
            "mergeable" => app.theme.success,
            "conflict" => app.theme.failed,
            _ => app.theme.pending,
        };
        lines.push(Line::from(vec![
            Span::styled(
                "Status: ",
                Style::default()
                    .fg(app.theme.title_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(detailed_status, Style::default().fg(status_color)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "  [ Enter: Confirm ]  ",
            Style::default()
                .bg(app.theme.success)
                .fg(get_contrast_color(app.theme.success))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            "  [ Esc: Cancel ]  ",
            Style::default()
                .bg(app.theme.dim)
                .fg(get_contrast_color(app.theme.dim)),
        ),
    ]));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(paragraph, inner);
}

pub fn render_confirm_delete_branch_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 25, f.area());
    app.set_rect("popup_confirm_delete_branch", area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .bg(app.theme.code_bg)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " Confirm Delete Branch ",
            Style::default().fg(app.theme.title_fg).bold(),
        ))
        .border_style(Style::default().fg(app.theme.failed));

    f.render_widget(block, area);

    let inner = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Are you sure you want to DELETE this branch?",
        Style::default().fg(app.theme.title_fg),
    )));
    lines.push(Line::from(""));
    if let Some((_, name)) = &app.pending_delete_branch {
        lines.push(Line::from(vec![
            Span::styled("Branch: ", Style::default().fg(app.theme.title_fg)),
            Span::styled(
                name.clone(),
                Style::default()
                    .fg(app.theme.failed)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "  [ Enter: Confirm ]  ",
            Style::default()
                .bg(app.theme.failed)
                .fg(get_contrast_color(app.theme.failed))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            "  [ Esc: Cancel ]  ",
            Style::default()
                .bg(app.theme.dim)
                .fg(get_contrast_color(app.theme.dim)),
        ),
    ]));

    let para = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(para, inner);
}
