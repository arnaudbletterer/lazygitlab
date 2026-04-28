use ratatui::{prelude::*, widgets::*};
use crate::models::App;
use crate::ui::widgets::render_shortcuts;

pub fn render_diff_view(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title/File selection
            Constraint::Min(0),    // Diff content
            Constraint::Length(1), // Shortcuts
        ])
        .split(f.area());

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
                let mut lines = Vec::new();
                for line in mr_diff.diff.lines() {
                    let style = if line.starts_with('+') {
                        Style::default().fg(Color::Green)
                    } else if line.starts_with('-') {
                        Style::default().fg(Color::Red)
                    } else if line.starts_with("@@") {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    };
                    lines.push(Line::from(Span::styled(line, style)));
                }

                let diff_paragraph = Paragraph::new(lines)
                    .block(Block::default().borders(Borders::ALL).title(format!(" {} ", mr_diff.new_path)))
                    .wrap(Wrap { trim: false })
                    .scroll((0, 0)); // Could add scrolling here too
                f.render_widget(diff_paragraph, chunks[1]);
            }
        }
    }

    render_shortcuts(f, chunks[2], vec![("j/k", "nav"), ("esc", "back"), ("q", "quit")], &app.theme);
}
