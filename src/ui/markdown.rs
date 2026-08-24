use crate::ui::Theme;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::prelude::*;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

fn ensure_prefix(spans: &mut Vec<Span<'static>>, depth: usize) {
    if spans.is_empty() && depth > 0 {
        for _ in 0..depth {
            spans.push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));
        }
    }
}

fn render_table(rows: &[Vec<Vec<Span<'static>>>], theme: &Theme, lines: &mut Vec<Line<'static>>) {
    if rows.is_empty() {
        return;
    }
    let has_header = !rows.is_empty() && !rows[0].is_empty();
    let header_rows: &[Vec<Vec<Span<'static>>>] = if has_header { &rows[..1] } else { &[] };
    let body_rows: &[Vec<Vec<Span<'static>>>] = if has_header { &rows[1..] } else { rows };

    let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if col_count == 0 {
        return;
    }
    let mut widths = vec![0usize; col_count];
    for row in rows.iter() {
        for (i, cell) in row.iter().enumerate() {
            let w: usize = cell.iter().map(|s| s.content.chars().count()).sum();
            if w + 2 > widths[i] {
                widths[i] = w + 2;
            }
        }
    }

    let header_style = Style::default()
        .fg(theme.header1)
        .add_modifier(Modifier::BOLD);
    let border_style = Style::default().fg(theme.dim);
    let body_style = Style::default();

    for row in header_rows.iter() {
        let mut line_spans: Vec<Span<'static>> = vec![Span::styled("│".to_string(), border_style)];
        for (j, cell) in row.iter().enumerate() {
            line_spans.push(Span::styled(" ".to_string(), body_style));
            for span in cell {
                line_spans.push(Span::styled(span.content.clone(), header_style));
            }
            let cell_w: usize = cell.iter().map(|s| s.content.chars().count()).sum();
            let pad = widths[j].saturating_sub(cell_w + 2);
            if pad > 0 {
                line_spans.push(Span::styled(" ".repeat(pad), body_style));
            }
            line_spans.push(Span::styled("│".to_string(), border_style));
        }
        lines.push(Line::from(line_spans));
    }

    let mut sep_spans: Vec<Span<'static>> = Vec::new();
    for w in &widths {
        sep_spans.push(Span::styled("├".to_string(), border_style));
        sep_spans.push(Span::styled("─".repeat(*w), border_style));
    }
    sep_spans.push(Span::styled("┤".to_string(), border_style));
    lines.push(Line::from(sep_spans));

    for row in body_rows.iter() {
        let mut line_spans: Vec<Span<'static>> = vec![Span::styled("│".to_string(), border_style)];
        for (j, cell) in row.iter().enumerate() {
            line_spans.push(Span::styled(" ".to_string(), body_style));
            for span in cell {
                line_spans.push(Span::styled(span.content.clone(), body_style));
            }
            let cell_w: usize = cell.iter().map(|s| s.content.chars().count()).sum();
            let pad = widths[j].saturating_sub(cell_w + 2);
            if pad > 0 {
                line_spans.push(Span::styled(" ".repeat(pad), body_style));
            }
            line_spans.push(Span::styled("│".to_string(), border_style));
        }
        lines.push(Line::from(line_spans));
    }

    lines.push(Line::from(""));
}

pub fn render_markdown(
    text: &str,
    theme: &Theme,
    syntax_set: &SyntaxSet,
    theme_set: &ThemeSet,
) -> Vec<Line<'static>> {
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

    let mut table_rows: Vec<Vec<Vec<Span<'static>>>> = Vec::new();
    let mut table_current_row: Vec<Vec<Span<'static>>> = Vec::new();
    let mut table_current_cell: Vec<Span<'static>> = Vec::new();
    let mut in_table = false;

    let syn_theme = &theme_set.themes["base16-ocean.dark"];

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => in_heading = Some(level),
                Tag::List(_) => list_level += 1,
                Tag::Item => {
                    ensure_prefix(&mut current_line_spans, blockquote_depth);
                    let indent = "  ".repeat(list_level.saturating_sub(1));
                    current_line_spans.push(Span::raw(format!("{}• ", indent)));
                }
                Tag::CodeBlock(kind) => match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        in_code_block = Some(lang.to_string())
                    }
                    _ => in_code_block = Some("".to_string()),
                },
                Tag::BlockQuote(_) => {
                    blockquote_depth += 1;
                }
                Tag::Strong => in_strong = true,
                Tag::Emphasis => in_emphasis = true,
                Tag::Strikethrough => in_strikethrough = true,
                Tag::Paragraph => {}
                Tag::Link { .. } => {
                    ensure_prefix(&mut current_line_spans, blockquote_depth);
                    current_line_spans.push(Span::styled("[", Style::default().fg(theme.dim)));
                }
                Tag::Table(_) => {
                    in_table = true;
                }
                Tag::TableRow => {
                    let cell = std::mem::take(&mut table_current_cell);
                    if !table_current_row.is_empty() || !cell.is_empty() {
                        table_current_row.push(cell);
                    }
                    if !table_current_row.is_empty() {
                        table_rows.push(std::mem::take(&mut table_current_row));
                    }
                }
                Tag::TableCell => {
                    let cell = std::mem::take(&mut table_current_cell);
                    if !table_current_row.is_empty() || !cell.is_empty() {
                        table_current_row.push(cell);
                    }
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(_) => {
                    let level = in_heading.unwrap_or(pulldown_cmark::HeadingLevel::H1);
                    let style = match level {
                        pulldown_cmark::HeadingLevel::H1 => Style::default()
                            .fg(theme.header1)
                            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                        pulldown_cmark::HeadingLevel::H2 => Style::default()
                            .fg(theme.header2)
                            .add_modifier(Modifier::BOLD),
                        _ => Style::default()
                            .fg(theme.shortcut_key)
                            .add_modifier(Modifier::BOLD),
                    };
                    for span in &mut current_line_spans {
                        span.style = span.style.patch(style);
                    }
                    lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                    lines.push(Line::from(""));
                    in_heading = None;
                }
                TagEnd::List(_) => {
                    list_level = list_level.saturating_sub(1);
                    if list_level == 0 {
                        lines.push(Line::from(""));
                    }
                }
                TagEnd::Item | TagEnd::Paragraph => {
                    if !current_line_spans.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                    }
                }
                TagEnd::BlockQuote(_) => {
                    if !current_line_spans.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line_spans)));
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
                    if let Some(last) = table_current_row.last()
                        && !last.is_empty()
                    {
                        table_current_row.push(last.clone());
                    }
                    if !table_current_row.is_empty() {
                        table_rows.push(std::mem::take(&mut table_current_row));
                    }
                    render_table(&table_rows, theme, &mut lines);
                    table_rows.clear();
                    in_table = false;
                }
                TagEnd::TableRow => {
                    let cell = std::mem::take(&mut table_current_cell);
                    if !table_current_row.is_empty() || !cell.is_empty() {
                        table_current_row.push(cell);
                    }
                    if !table_current_row.is_empty() {
                        table_rows.push(std::mem::take(&mut table_current_row));
                    }
                }
                TagEnd::TableCell => {
                    let cell = std::mem::take(&mut table_current_cell);
                    if !table_current_row.is_empty() || !cell.is_empty() {
                        table_current_row.push(cell);
                    }
                }
                _ => {}
            },
            Event::Text(t) => {
                if in_table {
                    let mut style = Style::default();
                    if in_strong {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if in_emphasis {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    if in_strikethrough {
                        style = style.add_modifier(Modifier::CROSSED_OUT);
                    }
                    let text_str = t.to_string();
                    for (i, s) in text_str.split('\n').enumerate() {
                        if i > 0 {
                            table_current_row.push(std::mem::take(&mut table_current_cell));
                        }
                        table_current_cell.push(Span::styled(s.to_string(), style));
                    }
                } else if let Some(lang) = &in_code_block {
                    let syntax = syntax_set
                        .find_syntax_by_token(lang)
                        .or_else(|| syntax_set.find_syntax_by_extension(lang))
                        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

                    let mut h = HighlightLines::new(syntax, syn_theme);
                    for line in t.lines() {
                        let ranges = h.highlight_line(line, syntax_set).unwrap();
                        let mut spans = Vec::new();
                        ensure_prefix(&mut spans, blockquote_depth);
                        for (style, content) in ranges {
                            let fg = Color::Rgb(
                                style.foreground.r,
                                style.foreground.g,
                                style.foreground.b,
                            );
                            spans.push(Span::styled(
                                content.to_string(),
                                Style::default().fg(fg).bg(theme.code_bg),
                            ));
                        }
                        lines.push(Line::from(spans));
                    }
                } else {
                    let mut style = Style::default();
                    if in_strong {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if in_emphasis {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    if in_strikethrough {
                        style = style.add_modifier(Modifier::CROSSED_OUT);
                    }

                    let text_str = t.to_string();
                    let lines_str: Vec<&str> = text_str.split('\n').collect();
                    for (i, s) in lines_str.iter().enumerate() {
                        if i > 0 {
                            lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                        }
                        ensure_prefix(&mut current_line_spans, blockquote_depth);
                        current_line_spans.push(Span::styled(s.to_string(), style));
                    }
                }
            }
            Event::Code(t) => {
                if in_table {
                    table_current_cell.push(Span::styled(t.to_string(), Style::default()));
                } else {
                    ensure_prefix(&mut current_line_spans, blockquote_depth);
                    current_line_spans.push(Span::styled(
                        t.to_string(),
                        Style::default().fg(theme.code).bg(theme.code_bg),
                    ));
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_table {
                    table_current_row.push(std::mem::take(&mut table_current_cell));
                } else {
                    lines.push(Line::from(std::mem::take(&mut current_line_spans)));
                }
            }
            Event::TaskListMarker(checked) => {
                if let Some(last) = current_line_spans
                    .last_mut()
                    .filter(|l| l.content.ends_with("• "))
                {
                    let indent = "  ".repeat(list_level.saturating_sub(1));
                    let marker = if checked { "[x] " } else { "[ ] " };
                    *last = Span::raw(format!("{}{}", indent, marker));
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
        (
            SyntaxSet::load_defaults_newlines(),
            ThemeSet::load_defaults(),
        )
    }

    #[test]
    fn test_render_markdown_plain() {
        let theme = Theme::default();
        let (ss, ts) = setup();
        let lines = render_markdown("Hello world", &theme, &ss, &ts);
        assert!(!lines.is_empty());
        assert!(
            lines
                .iter()
                .any(|l| l.spans.iter().any(|s| s.content.contains("Hello world")))
        );
    }

    #[test]
    fn test_render_markdown_exhaustive() {
        let theme = Theme::default();
        let (ss, ts) = setup();
        let text = "# H1\n## H2\n### H3\n> quote\n- item 1\n* item 2\n- [x] done\n\n```rust\nfn main() {}\n```\nplain **bold** _italic_ ~~strike~~ `code` [link](url)";
        let lines = render_markdown(text, &theme, &ss, &ts);

        let all_content: String = lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

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

    #[test]
    fn test_render_markdown_table_with_aligned_columns() {
        let theme = Theme::default();
        let (ss, ts) = setup();
        let text =
            "| Name | Age | City |\n|------|-----|------|\n| Alice | 30 | NYC |\n| Bob | 25 | LA |";
        let lines = render_markdown(text, &theme, &ss, &ts);

        let all_content: String = lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(all_content.contains("Name"));
        assert!(all_content.contains("Age"));
        assert!(all_content.contains("City"));
        assert!(all_content.contains("Alice"));
        assert!(all_content.contains("Bob"));
        assert!(all_content.contains("NYC"));
        assert!(all_content.contains("LA"));
        // column separator should use box-drawing characters
        assert!(all_content.contains("│"));
        assert!(all_content.contains("├"));
        assert!(all_content.contains("┤"));
        assert!(all_content.contains("─"));
    }

    #[test]
    fn test_render_markdown_table_with_inline_formatting() {
        let theme = Theme::default();
        let (ss, ts) = setup();
        let text = "| Feature | Status |\n|---------|--------|\n| **bold** | `code` |";
        let lines = render_markdown(text, &theme, &ss, &ts);
        let all_content: String = lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(all_content.contains("Feature"));
        assert!(all_content.contains("bold"));
        assert!(all_content.contains("code"));
    }
}
