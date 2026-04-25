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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::Theme;

    #[test]
    fn test_render_markdown_plain() {
        let theme = Theme::default();
        let lines = render_markdown("Hello world", &theme);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].spans.len(), 1);
        assert_eq!(lines[0].spans[0].content, "Hello world");
    }

    #[test]
    fn test_render_markdown_bold() {
        let theme = Theme::default();
        let lines = render_markdown("Hello **bold** text", &theme);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].spans.len(), 3);
        assert_eq!(lines[0].spans[0].content, "Hello ");
        assert_eq!(lines[0].spans[1].content, "bold");
        assert_eq!(lines[0].spans[1].style.add_modifier, Modifier::BOLD);
        assert_eq!(lines[0].spans[2].content, " text");
    }

    #[test]
    fn test_render_markdown_code() {
        let theme = Theme::default();
        let lines = render_markdown("Code `println!()` block", &theme);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].spans.len(), 3);
        assert_eq!(lines[0].spans[0].content, "Code ");
        assert_eq!(lines[0].spans[1].content, "println!()");
        assert_eq!(lines[0].spans[1].style.fg, Some(theme.shortcut_key));
        assert_eq!(lines[0].spans[2].content, " block");
    }

    #[test]
    fn test_render_markdown_multiline_and_mixed() {
        let theme = Theme::default();
        let lines = render_markdown("Line 1\nLine **2** with `code`\r\nLine 3", &theme);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].spans[0].content, "Line 1");
        assert_eq!(lines[1].spans[0].content, "Line ");
        assert_eq!(lines[1].spans[1].content, "2");
        assert_eq!(lines[1].spans[1].style.add_modifier, Modifier::BOLD);
        assert_eq!(lines[1].spans[2].content, " with ");
        assert_eq!(lines[1].spans[3].content, "code");
        assert_eq!(lines[1].spans[3].style.fg, Some(theme.shortcut_key));
        assert_eq!(lines[2].spans[0].content, "Line 3");
    }

    #[test]
    fn test_render_markdown_unclosed_tags() {
        let theme = Theme::default();
        let lines = render_markdown("Unclosed **bold and `code", &theme);
        assert_eq!(lines.len(), 1);
        // The current simple parser might just render the rest as plain text or keep it styled.
        // We just ensure it doesn't panic.
        assert!(!lines[0].spans.is_empty());
    }
}
