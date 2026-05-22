use ratatui::{prelude::*, widgets::*};
use crate::models::{App, InputTarget, DetailSection};
use crate::ui::markdown::render_markdown;
use crate::ui::list::{render_metadata_panel, render_discussions_panel};

pub fn render_detail_view(f: &mut Frame, app: &mut App, area: Rect) {
    let is_detail_focused = app.active_panel == crate::models::Panel::MainDetail;
    
    // Outer block containing the detail area
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Details ")
        .border_style(if is_detail_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split inner area: tab headers (Length(1)), divider (Length(1)), and content (Min(0))
    let detail_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tabs
            Constraint::Length(1), // Horizontal Divider
            Constraint::Min(0),    // Content Area
        ])
        .split(inner);

    // Generate tab titles
    let has_changes = app.selected_mr.is_some();
    let mut titles = vec![
        " Description ".to_string(),
        format!(" Discussions ({}) ", app.discussions.len()),
    ];
    if has_changes {
        titles.push(format!(" Changes ({}) ", app.mr_diffs.len()));
    }

    // Determine active tab index based on detail_section
    let active_tab_idx = match app.detail_section {
        DetailSection::Description | DetailSection::Metadata => 0,
        DetailSection::Discussions => 1,
        DetailSection::Changes => 2,
    };
    let max_tabs = if has_changes { 3 } else { 2 };
    let active_tab_idx = active_tab_idx.min(max_tabs - 1);

    // Render Tabs widget
    let tabs = Tabs::new(titles)
        .select(active_tab_idx)
        .divider(Span::styled(" │ ", Style::default().fg(app.theme.dim)))
        .style(Style::default().fg(app.theme.dim))
        .highlight_style(Style::default().fg(app.theme.border).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, detail_chunks[0]);

    // Render Divider Line
    let divider = Paragraph::new(Span::styled(
        "─".repeat(detail_chunks[1].width as usize),
        Style::default().fg(app.theme.dim),
    ));
    f.render_widget(divider, detail_chunks[1]);

    let content_area = detail_chunks[2];

    // Render tab content based on selected tab index
    match active_tab_idx {
        0 => {
            // TAB 0: Description & Metadata
            let is_editing_desc = app.input_mode && app.input_target == InputTarget::EditDescription;
            let is_desc_focused = is_detail_focused && app.detail_section == DetailSection::Description;
            
            // Split content vertically: Metadata (Length(11)) and Description Preview (Min(0))
            let desc_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(11), // Metadata block
                    Constraint::Min(0),     // Description text
                ])
                .split(content_area);

            // Render Metadata Panel
            render_metadata_panel(f, app, desc_chunks[0]);

            // Render Description Panel
            let desc_block = Block::default()
                .borders(Borders::ALL)
                .title(" Description / Preview ")
                .border_style(if is_editing_desc || is_desc_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });
            let desc_inner = desc_block.inner(desc_chunks[1]);
            f.render_widget(desc_block, desc_chunks[1]);

            if is_editing_desc {
                f.render_widget(&app.textarea, desc_inner);
            } else {
                let mut content = Vec::new();
                if let Some(mr) = &app.selected_mr {
                    content.extend(render_markdown(mr.description.as_deref().unwrap_or("No description provided."), &app.theme, &app.syntax_set, &app.theme_set));
                } else if let Some(issue) = &app.selected_issue {
                    content.extend(render_markdown(issue.description.as_deref().unwrap_or("No description provided."), &app.theme, &app.syntax_set, &app.theme_set));
                } else {
                    content.push(Line::from(" No item selected."));
                }
                f.render_widget(Paragraph::new(content).wrap(Wrap { trim: false }), desc_inner);
            }
        }
        1 => {
            // TAB 1: Discussions (Thread list at top, selected comments at bottom)
            let is_disc_focused = is_detail_focused && app.detail_section == DetailSection::Discussions;
            
            let disc_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(35), // Threads list summary
                    Constraint::Min(0),         // Thread comments
                ])
                .split(content_area);

            // Render threads list summary panel
            render_discussions_panel(f, app, disc_chunks[0]);

            // Render selected thread comments panel
            let comments_block = Block::default()
                .borders(Borders::ALL)
                .title(" Discussion Thread ")
                .border_style(if is_disc_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });
            let comments_inner = comments_block.inner(disc_chunks[1]);
            f.render_widget(comments_block, disc_chunks[1]);

            let mut content = Vec::new();
            if let Some(idx) = app.discussion_state.selected() {
                if let Some(disc) = app.discussions.get(idx) {
                    for note in &disc.notes {
                        let mut header = vec![
                            Span::styled(format!("@{}", note.author.username), Style::default().fg(app.theme.author).add_modifier(Modifier::BOLD)),
                            Span::raw(" "),
                        ];
                        if note.system { header.push(Span::styled(" [SYSTEM]", Style::default().fg(app.theme.dim))); }
                        content.push(Line::from(header));
                        content.extend(render_markdown(&note.body, &app.theme, &app.syntax_set, &app.theme_set));
                        content.push(Line::from(""));
                    }
                    if disc.notes.is_empty() { content.push(Line::from(" No notes in this discussion.")); }
                }
            } else {
                content.push(Line::from(" Select a discussion thread to view full conversation."));
            }
            f.render_widget(Paragraph::new(content).wrap(Wrap { trim: false }), comments_inner);
        }
        _ => {
            // TAB 2: Changes / Files
            let is_changes_focused = is_detail_focused && app.detail_section == DetailSection::Changes;
            
            let changes_block = Block::default()
                .borders(Borders::ALL)
                .title(" Changes / Files ")
                .border_style(if is_changes_focused { Style::default().fg(app.theme.border) } else { Style::default().fg(app.theme.dim) });
            let changes_inner = changes_block.inner(content_area);
            f.render_widget(changes_block, content_area);

            if app.mr_diffs.is_empty() {
                f.render_widget(Paragraph::new(" No changes found or still loading...").fg(app.theme.dim), changes_inner);
            } else {
                let files: Vec<ListItem> = app.mr_diffs.iter().map(|d| {
                    ListItem::new(Line::from(vec![
                        Span::styled(" ● ", Style::default().fg(app.theme.border)),
                        Span::raw(format!(" {} -> {}", d.old_path, d.new_path)),
                    ]))
                }).collect();
                f.render_stateful_widget(
                    List::new(files)
                        .highlight_style(if is_changes_focused { Style::default().bg(app.theme.highlight_bg).add_modifier(Modifier::BOLD) } else { Style::default() })
                        .highlight_symbol(if is_changes_focused { "> " } else { "  " }), 
                    changes_inner,
                    &mut app.mr_diff_state
                );
                
                // Helpful hint at the bottom
                let hint_area = Rect::new(changes_inner.x, changes_inner.y + changes_inner.height.saturating_sub(1), changes_inner.width, 1);
                f.render_widget(Paragraph::new(" Press [Enter] or [v] to view full diff ").fg(app.theme.dim).alignment(Alignment::Right), hint_area);
            }
        }
    }
}
