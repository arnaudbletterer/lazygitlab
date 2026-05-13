use ratatui::prelude::*;
use crate::ui::Theme;
use pulldown_cmark::{Parser, Event, Tag, TagEnd, Options};
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::easy::HighlightLines;

pub fn render_markdown(text: &str, theme: &Theme, syntax_set: &SyntaxSet, theme_set: &ThemeSet) -> Vec<Line<'static>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    
    let parser = Parser::new_ext(text, options);
    let mut lines = Vec::new();
    let mut current_line_spans = Vec::new();
    
    let mut list_level: usize = 0;
    let mut in_code_block: Option<String> = None;
    let mut in_heading = None;
    let mut in_strong = false;
    let mut in_emphasis = false;
    let mut in_strikethrough = false;
    let mut blockquote_depth: usize = 0;

    let syn_theme = &theme_set.themes["base16-ocean.dark"];

    fn ensure_prefix(spans: &mut Vec<Span<'static>>, depth: usize) {
        if spans.is_empty() && depth > 0 {
            for _ in 0..depth {
                spans.push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));
            }
        }
    }

    for event in parser {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Heading { level, .. } => in_heading = Some(level),
                    Tag::List(_) => list_level += 1,
                    Tag::Item => {
                        ensure_prefix(&mut current_line_spans, blockquote_depth);
                        let indent = "  ".repeat(list_level.saturating_sub(1));
                        current_line_spans.push(Span::raw(format!("{}• ", indent)));
                    }
                    Tag::CodeBlock(kind) => {
                        match kind {
                            pulldown_cmark::CodeBlockKind::Fenced(lang) => in_code_block = Some(lang.to_string()),
                            _ => in_code_block = Some("".to_string()),
                        }
                    }
                    Tag::BlockQuote(_) => {
                        blockquote_depth += 1;
                    }
                    Tag::Strong => in_strong = true,
                    Tag::Emphasis => in_emphasis = true,
                    Tag::Strikethrough => in_strikethrough = true,
                    Tag::Paragraph => {},
                    Tag::Link { .. } => {
                         ensure_prefix(&mut current_line_spans, blockquote_depth);
                         current_line_spans.push(Span::styled("[", Style::default().fg(theme.dim)));
                    }
                    Tag::Table(_) => {
                        lines.push(Line::from(vec![Span::styled("--- Table ---", Style::default().fg(theme.dim))]));
                    }
                    _ => {}
                }
            }
            Event::End(tag) => {
                match tag {
                    TagEnd::Heading(_) => {
                        let level = in_heading.unwrap_or(pulldown_cmark::HeadingLevel::H1);
                        let style = match level {
                            pulldown_cmark::HeadingLevel::H1 => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                            pulldown_cmark::HeadingLevel::H2 => Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                            _ => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                        };
                        for span in &mut current_line_spans {
                            span.style = span.style.patch(style);
                        }
                        lines.push(Line::from(current_line_spans.drain(..).collect::<Vec<_>>()));
                        lines.push(Line::from(""));
                        in_heading = None;
                    }
                    TagEnd::List(_) => {
                        list_level = list_level.saturating_sub(1);
                        if list_level == 0 { lines.push(Line::from("")); }
                    }
                    TagEnd::Item | TagEnd::Paragraph => {
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.drain(..).collect::<Vec<_>>()));
                        }
                    }
                    TagEnd::BlockQuote(_) => {
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.drain(..).collect::<Vec<_>>()));
                        }
                        blockquote_depth = blockquote_depth.saturating_sub(1);
                    }
                    TagEnd::CodeBlock => {
                        in_code_block = None;
                        lines.push(Line::from(""));
                    }
                    TagEnd::Strong => in_strong = false,
                    TagEnd::Emphasis => in_emphasis = false,
                    TagEnd::Strikethrough => in_strikethrough = false,
                    TagEnd::Link => {
                         current_line_spans.push(Span::styled("]", Style::default().fg(theme.dim)));
                    }
                    TagEnd::Table => {
                        lines.push(Line::from(vec![Span::styled("-------------", Style::default().fg(theme.dim))]));
                    }
                    TagEnd::TableRow => {
                        lines.push(Line::from(current_line_spans.drain(..).collect::<Vec<_>>()));
                    }
                    TagEnd::TableCell => {
                        current_line_spans.push(Span::raw(" | "));
                    }
                    _ => {}
                }
            }
            Event::Text(t) => {
                if let Some(lang) = &in_code_block {
                    let syntax = syntax_set.find_syntax_by_token(lang)
                        .or_else(|| syntax_set.find_syntax_by_extension(lang))
                        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
                    
                    let mut h = HighlightLines::new(syntax, syn_theme);
                    for line in t.lines() {
                        let ranges = h.highlight_line(line, syntax_set).unwrap();
                        let mut spans = Vec::new();
                        ensure_prefix(&mut spans, blockquote_depth);
                        for (style, content) in ranges {
                            let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                            spans.push(Span::styled(content.to_string(), Style::default().fg(fg).bg(theme.code_bg)));
                        }
                        lines.push(Line::from(spans));
                    }
                } else {
                    let mut style = Style::default();
                    if in_strong { style = style.add_modifier(Modifier::BOLD); }
                    if in_emphasis { style = style.add_modifier(Modifier::ITALIC); }
                    if in_strikethrough { style = style.add_modifier(Modifier::CROSSED_OUT); }

                    let text_str = t.to_string();
                    let lines_str: Vec<&str> = text_str.split('\n').collect();
                    for (i, s) in lines_str.iter().enumerate() {
                        if i > 0 {
                            lines.push(Line::from(current_line_spans.drain(..).collect::<Vec<_>>()));
                        }
                        ensure_prefix(&mut current_line_spans, blockquote_depth);
                        current_line_spans.push(Span::styled(s.to_string(), style));
                    }
                }
            }
            Event::Code(t) => {
                ensure_prefix(&mut current_line_spans, blockquote_depth);
                current_line_spans.push(Span::styled(t.to_string(), Style::default().fg(theme.code).bg(theme.code_bg)));
            }
            Event::SoftBreak | Event::HardBreak => {
                lines.push(Line::from(current_line_spans.drain(..).collect::<Vec<_>>()));
            }
            Event::TaskListMarker(checked) => {
                if let Some(last) = current_line_spans.last_mut() {
                    if last.content.ends_with("• ") {
                        let indent = "  ".repeat(list_level.saturating_sub(1));
                        let marker = if checked { "[x] " } else { "[ ] " };
                        *last = Span::raw(format!("{}{}", indent, marker));
                    }
                }
            }
            _ => {}
        }
    }
    
    if !current_line_spans.is_empty() {
        lines.push(Line::from(current_line_spans));
    }
    
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::Theme;

    fn setup() -> (SyntaxSet, ThemeSet) {
        (SyntaxSet::load_defaults_newlines(), ThemeSet::load_defaults())
    }

    #[test]
    fn test_render_markdown_plain() {
        let theme = Theme::default();
        let (ss, ts) = setup();
        let lines = render_markdown("Hello world", &theme, &ss, &ts);
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.spans.iter().any(|s| s.content.contains("Hello world"))));
    }

    #[test]
    fn test_render_markdown_exhaustive() {
        let theme = Theme::default();
        let (ss, ts) = setup();
        let text = "# H1\n## H2\n### H3\n> quote\n- item 1\n* item 2\n- [x] done\n\n```rust\nfn main() {}\n```\nplain **bold** _italic_ ~~strike~~ `code` [link](url)";
        let lines = render_markdown(text, &theme, &ss, &ts);
        
        for (i, line) in lines.iter().enumerate() {
            println!("Line {}: {:?}", i, line);
        }

        let all_content: String = lines.iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect::<String>()).collect::<Vec<_>>().join("\n");
        
        assert!(all_content.contains("H1"));
        assert!(all_content.contains("H2"));
        assert!(all_content.contains("H3"));
        assert!(all_content.contains("│ quote"));
        assert!(all_content.contains("• item 1"));
        assert!(all_content.contains("[x] done"));
        assert!(all_content.contains("fn main() {}"));
        assert!(all_content.contains("bold"));
        assert!(all_content.contains("italic"));
        assert!(all_content.contains("strike"));
        assert!(all_content.contains("[link]"));
    }
}
