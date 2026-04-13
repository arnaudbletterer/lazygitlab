use ratatui::prelude::*;
use crate::ui::Theme;

pub fn render_markdown<'a>(text: &'a str, theme: &Theme) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let clean_text = text.replace("\r\n", "\n");
    for line in clean_text.split('\n') {
        let mut spans = Vec::new();
        let mut current_pos = 0;
        let line_chars: Vec<char> = line.chars().collect();

        if line_chars.is_empty() {
            lines.push(Line::from(""));
            continue;
        }

        while current_pos < line_chars.len() {
            if line_chars[current_pos..].starts_with(&['*', '*']) || line_chars[current_pos..].starts_with(&['_', '_']) {
                let marker = if line_chars[current_pos] == '*' { &['*', '*'] } else { &['_', '_'] };
                let start = current_pos + 2;
                let mut end = start;
                while end < line_chars.len() && !line_chars[end..].starts_with(marker) {
                    end += 1;
                }
                
                if end < line_chars.len() {
                    let content: String = line_chars[start..end].iter().collect();
                    spans.push(Span::styled(content, Style::default().add_modifier(Modifier::BOLD)));
                    current_pos = end + 2;
                } else {
                    spans.push(Span::raw(line_chars[current_pos].to_string()));
                    current_pos += 1;
                }
            } else if line_chars[current_pos] == '`' {
                let start = current_pos + 1;
                let mut end = start;
                while end < line_chars.len() && line_chars[end] != '`' {
                    end += 1;
                }
                
                if end < line_chars.len() {
                    let content: String = line_chars[start..end].iter().collect();
                    spans.push(Span::styled(content, Style::default().fg(theme.shortcut_key).bg(Color::Rgb(30, 30, 30))));
                    current_pos = end + 1;
                } else {
                    spans.push(Span::raw("`"));
                    current_pos += 1;
                }
            } else {
                let start = current_pos;
                let mut end = start;
                while end < line_chars.len() && line_chars[end] != '*' && line_chars[end] != '_' && line_chars[end] != '`' {
                    end += 1;
                }
                
                if start == end {
                    spans.push(Span::raw(line_chars[current_pos].to_string()));
                    current_pos += 1;
                } else {
                    let content: String = line_chars[start..end].iter().collect();
                    spans.push(Span::raw(content));
                    current_pos = end;
                }
            }
        }
        lines.push(Line::from(spans));
    }
    lines
}
