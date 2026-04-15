use ratatui::{prelude::*, widgets::*};
use chrono::{DateTime, Utc};
use chrono_humanize::{HumanTime, Accuracy, Tense};

pub fn format_duration(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u64;
    let minutes = ((seconds % 3600.0) / 60.0).floor() as u64;
    let seconds = (seconds % 60.0).floor() as u64;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

pub fn get_elapsed_time(started_at: &str) -> String {
    if started_at.is_empty() { return "00:00".to_string(); }
    
    let now = Utc::now();
    let dt_parsed = DateTime::parse_from_rfc3339(started_at)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| DateTime::parse_from_str(started_at, "%Y-%m-%dT%H:%M:%S%.3fZ").map(|dt| dt.with_timezone(&Utc)));

    match dt_parsed {
        Ok(dt_utc) => {
            let duration = now.signed_duration_since(dt_utc);
            let seconds = duration.num_seconds() as f64;
            if seconds < 0.0 { return "00:00".to_string(); }
            format_duration(seconds)
        }
        Err(_) => "??:??".to_string(),
    }
}

pub fn format_relative_time(datetime_str: &str) -> String {
    if datetime_str.is_empty() { return "None".to_string(); }
    
    // GitLab ISO8601 can have variants, let's try to parse
    match DateTime::parse_from_rfc3339(datetime_str) {
        Ok(dt) => {
            let dt_utc: DateTime<Utc> = dt.with_timezone(&Utc);
            let ht = HumanTime::from(dt_utc);
            ht.to_text_en(Accuracy::Rough, Tense::Past)
        }
        Err(_) => {
            // Try another format if RFC3339 fails (GitLab sometimes uses this)
            match DateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.3fZ") {
                Ok(dt) => {
                    let ht = HumanTime::from(dt);
                    ht.to_text_en(Accuracy::Rough, Tense::Past)
                }
                Err(_) => datetime_str.to_string(),
            }
        }
    }
}

pub fn format_time_only(datetime_str: &str) -> String {
    // Expected: 2026-04-14T19:37:21.116729Z
    if datetime_str.len() >= 19 {
        // Just extract HH:MM:SS
        datetime_str[11..19].to_string()
    } else {
        datetime_str.to_string()
    }
}

pub struct Theme {
    pub border: Color,
    pub highlight_bg: Color,
    pub author: Color,
    pub dim: Color,
    pub success: Color,    // GitLab Green
    pub failed: Color,     // GitLab Red
    pub running: Color,    // GitLab Blue
    pub pending: Color,    // GitLab Yellow/Orange
    pub open_mr: Color,    // GitLab Blue (Open MRs)
    pub merged_mr: Color,  // GitLab Purple (Merged MRs)
    pub closed_mr: Color,  // GitLab Red (Closed MRs)
    pub shortcut_key: Color,
    pub shortcut_desc: Color,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            border: Color::Green,
            highlight_bg: Color::Blue,
            author: Color::Magenta,
            dim: Color::Rgb(100, 100, 100),
            success: Color::Rgb(26, 170, 85), // #1aaa55
            failed: Color::Rgb(219, 59, 33),  // #db3b21
            running: Color::Rgb(31, 117, 203), // #1f75cb
            pending: Color::Rgb(252, 148, 3), // #fc9403
            open_mr: Color::Rgb(31, 117, 203), // #1f75cb (Same as running/blue)
            merged_mr: Color::Rgb(102, 102, 196), // #6666c4
            closed_mr: Color::Rgb(219, 59, 33), // #db3b21 (Same as failed/red)
            shortcut_key: Color::Cyan,
            shortcut_desc: Color::White,
        }
    }
}

pub fn get_status_style(status: &str, theme: &Theme) -> Style {
    match status {
        // MR/Issue statuses
        "opened" | "open" => Style::default().fg(theme.open_mr),
        "merged" | "resolved" => Style::default().fg(theme.merged_mr),
        "closed" => Style::default().fg(theme.closed_mr),
        
        // Pipeline/Job statuses
        "success" => Style::default().fg(theme.success),
        "failed" => Style::default().fg(theme.failed),
        "running" => Style::default().fg(theme.running),
        "pending" => Style::default().fg(theme.pending),
        "canceled" | "skipped" | "manual" | "created" | "scheduled" => Style::default().fg(theme.dim),
        
        _ => Style::default(),
    }
}

pub fn render_shortcuts(f: &mut Frame, area: Rect, shortcuts: Vec<(&str, &str)>, theme: &Theme) {
    let mut spans = Vec::new();
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(", ", Style::default().fg(theme.shortcut_desc)));
        }
        spans.push(Span::styled(*key, Style::default().fg(theme.shortcut_key)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*desc, Style::default().fg(theme.shortcut_desc)));
    }
    let p = Paragraph::new(Line::from(spans));
    f.render_widget(p, area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Color::Rgb(r, g, b)
    } else {
        Color::Gray
    }
}
