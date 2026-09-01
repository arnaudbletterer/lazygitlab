use lazygitlab::ui::banner::Banner;
use lazygitlab::ui::widgets::Theme;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

#[test]
fn test_banner_height_responsiveness() {
    // Large area
    assert_eq!(Banner::height_for(80, 30), 7);
    assert_eq!(Banner::height_for(66, 8), 7);

    // Compact area
    assert_eq!(Banner::height_for(50, 20), 4);
    assert_eq!(Banner::height_for(42, 5), 4);

    // Micro area
    assert_eq!(Banner::height_for(30, 3), 1);
    assert_eq!(Banner::height_for(25, 2), 1);

    // Too small
    assert_eq!(Banner::height_for(20, 3), 0);
    assert_eq!(Banner::height_for(80, 1), 0);
}

#[test]
fn test_banner_lines_generation() {
    let theme = Theme::default();

    // Full banner
    let full_lines = Banner::get_lines(&theme, 80, 20);
    assert_eq!(full_lines.len(), 7);
    let full_text = full_lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(full_text.contains("GitLab"));
    assert!(full_text.contains(">"));

    // Compact banner
    let compact_lines = Banner::get_lines(&theme, 50, 5);
    assert_eq!(compact_lines.len(), 3);
    let compact_text = compact_lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(compact_text.contains("lazygitlab"));
    assert!(compact_text.contains("Help"));

    // Micro banner
    let micro_lines = Banner::get_lines(&theme, 30, 2);
    assert_eq!(micro_lines.len(), 1);
    let micro_text: String = micro_lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(micro_text.contains("lazygitlab"));
}

#[test]
fn test_banner_rendering_all_terminal_sizes() {
    let theme = Theme::default();
    let test_sizes = [(120, 40), (80, 24), (60, 20), (45, 15), (30, 8), (15, 4)];

    for (w, h) in test_sizes {
        let backend = TestBackend::new(w, h);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                Banner::render(f, Rect::new(0, 0, w, h), &theme);
            })
            .unwrap();
    }
}
