use ratatui::{prelude::*, widgets::*};
use crate::models::{App, InputTarget};
use crate::ui::widgets::{centered_rect, render_shortcuts};

pub fn render_create_issue_popup(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 40, f.area());
    app.set_rect("popup_create_issue", area);
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
    app.set_rect("popup_create_mr", area);
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

