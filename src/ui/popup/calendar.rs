use ratatui::{prelude::*, widgets::*};
use crate::models::{App, InputTarget};
use crate::ui::widgets::centered_rect;
use chrono::{Datelike, NaiveDate};

pub fn render_calendar_popup(f: &mut Frame, app: &mut App) {
    
    let area = centered_rect(40, 15, f.area());
    app.set_rect("popup_calendar", area);
    f.render_widget(Clear, area);

    let title = match app.input_target {
        InputTarget::EditIssueStartDate => format!(" Start Date: {} ", app.calendar_date.format("%B %Y")),
        InputTarget::EditIssueDueDate => format!(" Due Date: {} ", app.calendar_date.format("%B %Y")),
        _ => format!(" Select Date: {} ", app.calendar_date.format("%B %Y")),
    };
    f.render_widget(Block::default().bg(Color::Black).borders(Borders::ALL).title(title), area);

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
}

