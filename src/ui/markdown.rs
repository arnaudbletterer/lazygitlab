use ratatui::prelude::*;
use crate::ui::Theme;

pub fn render_markdown(text: &str, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let clean_text = text.replace("\r\n", "\n");
    let mut in_code_block = false;

    // Simple keyword highlighter
    let keywords = ["fn", "let", "pub", "struct", "enum", "impl", "match", "if", "else", "return", "use", "mod", "crate", "type", "trait", "mut", "async", "await", "true", "false", "Some", "None", "Ok", "Err", "Result", "Option"];

    for line in clean_text.split('\n') {
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            let mut spans = Vec::new();
            let words: Vec<&str> = line.split_inclusive(|c: char| !c.is_alphanumeric() && c != '_').collect();
            for word in words {
                let trimmed = word.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
                let separator = &word[trimmed.len()..];
                
                if keywords.contains(&trimmed) {
                    spans.push(Span::styled(trimmed.to_string(), Style::default().fg(theme.keyword).bg(theme.code_bg).add_modifier(Modifier::BOLD)));
                } else if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_numeric()) {
                    spans.push(Span::styled(trimmed.to_string(), Style::default().fg(Color::Yellow).bg(theme.code_bg)));
                } else {
                    spans.push(Span::styled(trimmed.to_string(), Style::default().fg(theme.code).bg(theme.code_bg)));
                }
                spans.push(Span::styled(separator.to_string(), Style::default().fg(theme.code).bg(theme.code_bg)));
            }
            lines.push(Line::from(spans));
            continue;
        }

        if line.starts_with("# ") {
            lines.push(Line::from(vec![
                Span::styled(line[2..].to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
            ]));
            continue;
        } else if line.starts_with("## ") {
            lines.push(Line::from(vec![
                Span::styled(line[3..].to_string(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
            ]));
            continue;
        } else if line.starts_with("### ") {
            lines.push(Line::from(vec![
                Span::styled(line[4..].to_string(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
            ]));
            continue;
        } else if line.starts_with("> ") {
            lines.push(Line::from(vec![
                Span::styled(format!("│ {}", &line[2..]), Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
            ]));
            continue;
        } else if line.starts_with("- ") || line.starts_with("* ") {
            let mut spans = vec![Span::styled("  • ", Style::default().fg(theme.border))];
            spans.extend(parse_inline_markdown(&line[2..], theme));
            lines.push(Line::from(spans));
            continue;
        }

        if line.trim().is_empty() {
            lines.push(Line::from(""));
            continue;
        }

        lines.push(Line::from(parse_inline_markdown(line, theme)));
    }
    lines
}

fn parse_inline_markdown(text: &str, theme: &Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = text.chars().collect();

    let keywords = ["fn", "let", "pub", "struct", "enum", "impl", "match", "if", "else", "return", "use", "mod", "crate", "type", "trait", "mut", "async", "await", "true", "false", "Some", "None", "Ok", "Err", "Result", "Option"];

    while current_pos < chars.len() {
        if chars[current_pos..].starts_with(&['*', '*']) || chars[current_pos..].starts_with(&['_', '_']) {
            let marker = if chars[current_pos] == '*' { &['*', '*'] } else { &['_', '_'] };
            let start = current_pos + 2;
            let mut end = start;
            while end < chars.len() && !chars[end..].starts_with(marker) {
                end += 1;
            }
            
            if end < chars.len() {
                let content: String = chars[start..end].iter().collect();
                spans.push(Span::styled(content, Style::default().add_modifier(Modifier::BOLD)));
                current_pos = end + 2;
            } else {
                spans.push(Span::raw(chars[current_pos].to_string()));
                current_pos += 1;
            }
        } else if chars[current_pos] == '`' {
            let start = current_pos + 1;
            let mut end = start;
            while end < chars.len() && chars[end] != '`' {
                end += 1;
            }
            
            if end < chars.len() {
                let content: String = chars[start..end].iter().collect();
                let words: Vec<&str> = content.split_inclusive(|c: char| !c.is_alphanumeric() && c != '_').collect();
                for word in words {
                    let trimmed = word.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
                    let separator = &word[trimmed.len()..];
                    
                    if keywords.contains(&trimmed) {
                        spans.push(Span::styled(trimmed.to_string(), Style::default().fg(theme.keyword).bg(theme.code_bg).add_modifier(Modifier::BOLD)));
                    } else if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_numeric()) {
                        spans.push(Span::styled(trimmed.to_string(), Style::default().fg(Color::Yellow).bg(theme.code_bg)));
                    } else {
                        spans.push(Span::styled(trimmed.to_string(), Style::default().fg(theme.code).bg(theme.code_bg)));
                    }
                    spans.push(Span::styled(separator.to_string(), Style::default().fg(theme.code).bg(theme.code_bg)));
                }
                current_pos = end + 1;
            } else {
                spans.push(Span::raw("`".to_string()));
                current_pos += 1;
            }
        } else {
            let start = current_pos;
            let mut end = start;
            while end < chars.len() && chars[end] != '*' && chars[end] != '_' && chars[end] != '`' {
                end += 1;
            }
            
            if start == end {
                spans.push(Span::raw(chars[current_pos].to_string()));
                current_pos += 1;
            } else {
                let content: String = chars[start..end].iter().collect();
                spans.push(Span::raw(content));
                current_pos = end;
            }
        }
    }
    spans
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
    fn test_render_markdown_exhaustive() {
        let theme = Theme::default();
        let text = "# H1\n## H2\n### H3\n> quote\n- item 1\n* item 2\n\n```\ncode\n```\nplain **bold** `code`";
        let lines = render_markdown(text, &theme);
        assert!(lines.len() >= 9);
        
        // H1
        assert_eq!(lines[0].spans[0].content, "H1");
        // H2
        assert_eq!(lines[1].spans[0].content, "H2");
        // H3
        assert_eq!(lines[2].spans[0].content, "H3");
        // Quote
        assert!(lines[3].spans[0].content.contains("│ quote"));
        // Lists
        assert!(lines[4].spans[0].content.contains("•"));
        assert!(lines[5].spans[0].content.contains("•"));
        // Empty line
        assert_eq!(lines[6].spans.len(), 0);
        // Code block
        assert!(lines[7].spans.iter().any(|s| s.content == "code"));
        // Inline
        assert!(lines[8].spans.iter().any(|s| s.content == "bold"));
        assert!(lines[8].spans.iter().any(|s| s.content == "code"));
    }
}
