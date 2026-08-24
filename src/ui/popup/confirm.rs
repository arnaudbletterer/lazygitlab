use ratatui::{prelude::*, widgets::*};
use crate::models::App;
use crate::ui::widgets::{centered_rect, get_status_style};

pub fn render_confirm_merge_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 35, f.area());
    app.set_rect("popup_confirm_merge", area);
    f.render_widget(Clear, area);
    
    let block = Block::default()
        .bg(Color::Black)
        .borders(Borders::ALL)
        .title(" Confirm Merge Request ")
        .border_style(Style::default().fg(app.theme.border));
        
    f.render_widget(block.clone(), area);
    
    let inner = area.inner(Margin { horizontal: 2, vertical: 1 });
    
    let mut lines = Vec::new();
    lines.push(Line::from("Are you sure you want to merge this Merge Request?"));
    lines.push(Line::from(""));
    
    if let Some(mr) = &app.selected_mr {
        lines.push(Line::from(vec![
            Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&mr.title),
        ]));
        
        lines.push(Line::from(vec![
            Span::styled("Branches: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(&mr.source_branch, Style::default().fg(app.theme.author)),
            Span::raw(" -> "),
            Span::styled(&mr.target_branch, Style::default().fg(app.theme.project)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Squash: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(if mr.squash.unwrap_or(false) { "Yes" } else { "No" }),
        ]));

        // Pipeline info if available
        if let Some(pipeline) = &mr.head_pipeline {
            lines.push(Line::from(vec![
                Span::styled("Pipeline: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(&pipeline.status, get_status_style(&pipeline.status, &app.theme)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("Pipeline: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("None"),
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
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(detailed_status, Style::default().fg(status_color)),
        ]));
    }
    
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [y/Enter] Confirm  ", Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled("  [n/Esc] Cancel  ", Style::default().bg(app.theme.dim).fg(Color::Black)),
    ]));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(paragraph, inner);
}

pub fn render_confirm_rebase_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 35, f.area());
    app.set_rect("popup_confirm_rebase", area);
    f.render_widget(Clear, area);
    
    let block = Block::default()
        .bg(Color::Black)
        .borders(Borders::ALL)
        .title(" Confirm Rebase ")
        .border_style(Style::default().fg(app.theme.border));
        
    f.render_widget(block.clone(), area);
    
    let inner = area.inner(Margin { horizontal: 2, vertical: 1 });
    
    let mut lines = Vec::new();
    lines.push(Line::from("Are you sure you want to rebase this Merge Request?"));
    lines.push(Line::from(""));
    
    if let Some(mr) = &app.selected_mr {
        lines.push(Line::from(vec![
            Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&mr.title),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Branches: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(&mr.source_branch, Style::default().fg(app.theme.author)),
            Span::raw(" -> "),
            Span::styled(&mr.target_branch, Style::default().fg(app.theme.project)),
        ]));

        // Pipeline info if available
        if let Some(pipeline) = &mr.head_pipeline {
            lines.push(Line::from(vec![
                Span::styled("Pipeline: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(&pipeline.status, get_status_style(&pipeline.status, &app.theme)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("Pipeline: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("None"),
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
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(detailed_status, Style::default().fg(status_color)),
        ]));
    }
    
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [y/Enter] Confirm  ", Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled("  [n/Esc] Cancel  ", Style::default().bg(app.theme.dim).fg(Color::Black)),
    ]));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(paragraph, inner);
}


pub fn render_confirm_delete_branch_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 25, f.area());
    app.set_rect("popup_confirm_delete_branch", area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .bg(Color::Black)
        .borders(Borders::ALL)
        .title(" Confirm Delete Branch ")
        .border_style(Style::default().fg(app.theme.failed));

    f.render_widget(block.clone(), area);

    let inner = area.inner(Margin { horizontal: 2, vertical: 1 });

    let mut lines = Vec::new();
    lines.push(Line::from("Are you sure you want to DELETE this branch?"));
    lines.push(Line::from(""));
    if let Some((_, name)) = &app.pending_delete_branch {
        lines.push(Line::from(vec![
            Span::raw("Branch: "),
            Span::styled(name.clone(), Style::default().fg(app.theme.failed).add_modifier(Modifier::BOLD)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("y / Enter = delete   n / Esc = cancel", Style::default().fg(app.theme.dim))));

    let para = Paragraph::new(lines).block(Block::default());
    f.render_widget(para, inner);
}
