use crate::models::{App, InputTarget, ProjectVariable};
use crate::ui::widgets::{centered_rect, render_shortcuts};
use ratatui::{prelude::*, widgets::*};

pub fn render_pipeline_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 60, f.area());
    app.set_rect("popup_pipeline", area);
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default()
            .bg(Color::Black)
            .borders(Borders::ALL)
            .title(" Launch Pipeline ")
            .border_style(Style::default().fg(app.theme.border)),
        area,
    );

    let inner = area.inner(Margin {
        horizontal: 2,
        vertical: 2,
    });
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

    // Ref Input
    let ref_style = if let InputTarget::PipelineRef = app.input_target {
        Style::default().fg(app.theme.highlight_bg)
    } else {
        Style::default()
    };
    let ref_input = Paragraph::new(app.pipeline_ref_input.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Ref (Branch/Tag) ")
            .border_style(ref_style),
    );
    f.render_widget(ref_input, chunks[0]);

    // Variable Key Input
    let key_style = if let InputTarget::PipelineVariableKey = app.input_target {
        Style::default().fg(app.theme.highlight_bg)
    } else {
        Style::default()
    };
    let key_input = Paragraph::new(app.pipeline_variable_key_input.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Variable Key ")
            .border_style(key_style),
    );
    f.render_widget(key_input, chunks[1]);

    // Variable Value Input
    let val_style = if let InputTarget::PipelineVariableValue = app.input_target {
        Style::default().fg(app.theme.highlight_bg)
    } else {
        Style::default()
    };
    let val_input = Paragraph::new(app.pipeline_variable_value_input.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Variable Value ")
            .border_style(val_style),
    );
    f.render_widget(val_input, chunks[2]);

    // Variables List
    let items: Vec<ListItem> = app
        .pipeline_variables
        .iter()
        .map(|v| ListItem::new(format!("{}: {}", v.key, v.value)))
        .collect();
    let is_var_list_focused = app.input_target == InputTarget::PipelineVariableList;
    let var_list_style = if is_var_list_focused {
        Style::default().fg(app.theme.highlight_bg)
    } else {
        Style::default()
    };
    let highlight_style = if is_var_list_focused {
        Style::default().bg(app.theme.highlight_bg)
    } else {
        Style::default()
    };
    let highlight_symbol = if is_var_list_focused { "> " } else { "  " };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Variables ")
                .border_style(var_list_style),
        )
        .highlight_style(highlight_style)
        .highlight_symbol(highlight_symbol);
    f.render_stateful_widget(list, chunks[3], &mut app.pipeline_variables_state);

    // Shortcuts
    render_shortcuts(
        f,
        chunks[4],
        vec![
            ("enter", "next"),
            ("p", "launch"),
            ("del", "remove var"),
            ("esc", "back"),
        ],
        &app.theme,
    );

    // Dropdown for branches
    if app.show_branches_dropdown && !app.filtered_branches.is_empty() {
        let dropdown_area = Rect::new(
            chunks[0].x + 1,
            chunks[0].y + 3,
            chunks[0].width - 2,
            (app.filtered_branches.len() as u16).clamp(3, 10),
        );
        f.render_widget(Clear, dropdown_area);
        let branches: Vec<ListItem> = app
            .filtered_branches
            .iter()
            .map(|b| ListItem::new(b.name.as_str()))
            .collect();
        let list = List::new(branches)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.highlight_bg)),
            )
            .highlight_style(Style::default().bg(app.theme.highlight_bg))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, dropdown_area, &mut app.branches_state);
    }

    // Dropdown for variable keys
    if let InputTarget::PipelineVariableKey = app.input_target {
        let suggested_vars: Vec<&ProjectVariable> = app
            .project_variables
            .iter()
            .filter(|v| {
                v.key
                    .to_lowercase()
                    .contains(&app.pipeline_variable_key_input.to_lowercase())
            })
            .collect();

        if !suggested_vars.is_empty() {
            let dropdown_area = Rect::new(
                chunks[1].x + 1,
                chunks[1].y + 3,
                chunks[1].width - 2,
                (suggested_vars.len() as u16).clamp(3, 10),
            );
            f.render_widget(Clear, dropdown_area);
            let items: Vec<ListItem> = suggested_vars
                .iter()
                .map(|v| ListItem::new(format!("{} ({})", v.key, v.environment_scope)))
                .collect();
            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Suggested Keys ")
                        .border_style(Style::default().fg(app.theme.highlight_bg)),
                )
                .highlight_style(Style::default().bg(app.theme.highlight_bg))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, dropdown_area, &mut app.label_state);
        }
    }
}

pub fn render_schedules_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 50, f.area());
    app.set_rect("popup_schedules", area);
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default()
            .bg(Color::Black)
            .borders(Borders::ALL)
            .title(" Pipeline Schedules (Enter = run now) ")
            .border_style(Style::default().fg(app.theme.border)),
        area,
    );

    let items: Vec<ListItem> = app
        .pipeline_schedules
        .iter()
        .map(|s| {
            let status_style = if s.active {
                Style::default().fg(app.theme.success)
            } else {
                Style::default().fg(app.theme.dim)
            };
            ListItem::new(Line::from(vec![
                Span::styled(if s.active { "[on]  " } else { "[off] " }, status_style),
                Span::styled(
                    crate::ui::widgets::truncate_str(&s.description, 30),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("  {} @ {}", s.ref_name, s.cron)),
            ]))
        })
        .collect();

    if items.is_empty() {
        let inner = area.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });
        f.render_widget(
            Paragraph::new("No pipeline schedules configured.")
                .style(Style::default().fg(app.theme.dim)),
            inner,
        );
    } else {
        let list = List::new(items).highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .add_modifier(Modifier::BOLD),
        );
        f.render_stateful_widget(list, area, &mut app.schedules_state);
    }
}

pub fn render_ci_vars_popup(f: &mut Frame, app: &mut App) {
    let editing = app.ci_var_editing.is_some();
    let height = if editing { 60 } else { 45 };
    let area = centered_rect(55, height, f.area());
    app.set_rect("popup_ci_vars", area);
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default()
            .bg(Color::Black)
            .borders(Borders::ALL)
            .title(" CI Variables ")
            .border_style(Style::default().fg(app.theme.border)),
        area,
    );

    let inner = area.inner(Margin {
        horizontal: 2,
        vertical: 2,
    });
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if editing {
            vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(1),
            ]
        } else {
            vec![
                Constraint::Min(5),
                Constraint::Length(0),
                Constraint::Length(1),
            ]
        })
        .split(inner);

    if editing {
        let key_style = if app.input_target == InputTarget::PipelineVariableKey {
            Style::default().fg(app.theme.highlight_bg)
        } else {
            Style::default()
        };
        let val_style = if app.input_target == InputTarget::PipelineVariableValue {
            Style::default().fg(app.theme.highlight_bg)
        } else {
            Style::default()
        };
        let key_input = Paragraph::new(app.pipeline_variable_key_input.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " Key {} ",
                    if app.ci_var_editing.as_deref() == Some("") {
                        "(new)"
                    } else {
                        ""
                    }
                ))
                .border_style(key_style),
        );
        f.render_widget(key_input, chunks[0]);
        let val_input = Paragraph::new(app.pipeline_variable_value_input.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Value (Tab to switch, Ctrl+Enter to save) ")
                .border_style(val_style),
        );
        f.render_widget(val_input, chunks[1]);
        render_shortcuts(
            f,
            chunks[2],
            vec![("tab", "next"), ("ctrl+enter", "save"), ("esc", "cancel")],
            &app.theme,
        );
    } else {
        let items: Vec<ListItem> = app
            .project_variables
            .iter()
            .map(|v| {
                let mut spans = vec![
                    Span::styled(
                        crate::ui::widgets::truncate_str(&v.key, 24),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": "),
                    Span::raw(if v.masked {
                        "*****".to_string()
                    } else {
                        crate::ui::widgets::truncate_str(&v.value, 30)
                    }),
                ];
                if v.protected {
                    spans.push(Span::styled(
                        " [protected]",
                        Style::default().fg(app.theme.pending),
                    ));
                }
                ListItem::new(Line::from(spans))
            })
            .collect();
        if items.is_empty() {
            f.render_widget(
                Paragraph::new("No CI variables. Press n to create one.")
                    .style(Style::default().fg(app.theme.dim)),
                chunks[0],
            );
        } else {
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        .bg(app.theme.highlight_bg)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_stateful_widget(list, chunks[0], &mut app.ci_vars_state);
        }
        render_shortcuts(
            f,
            chunks[2],
            vec![
                ("enter", "edit value"),
                ("n", "new"),
                ("d", "delete"),
                ("esc", "close"),
            ],
            &app.theme,
        );
    }
}
