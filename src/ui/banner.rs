use crate::ui::Theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct Banner;

impl Banner {
    /// Returns the ideal height needed to render the banner given the available width and height.
    pub fn height_for(width: u16, max_height: u16) -> u16 {
        if max_height >= 8 && width >= 66 {
            7
        } else if max_height >= 5 && width >= 42 {
            4
        } else if max_height >= 2 && width >= 25 {
            1
        } else {
            0
        }
    }

    /// Renders the styled ASCII banner centered in the given area.
    pub fn render(f: &mut Frame, area: Rect, theme: &Theme) {
        if area.width < 25 || area.height == 0 {
            return;
        }

        let lines = Self::get_lines(theme, area.width, area.height);
        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    /// Renders a branded empty-state header and actionable instructions.
    pub fn render_empty_state(
        f: &mut Frame,
        area: Rect,
        theme: &Theme,
        title: &str,
        hints: &[(&str, &str)],
    ) {
        let banner_h = Self::height_for(area.width, area.height);
        let show_banner = banner_h > 0 && area.height >= 14;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if show_banner {
                vec![
                    Constraint::Length(banner_h),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ]
            } else {
                vec![Constraint::Length(0), Constraint::Length(0), Constraint::Min(0)]
            })
            .split(area);

        if show_banner {
            Self::render(f, chunks[0], theme);
        }

        let mut lines = vec![
            Line::from(Span::styled(
                title.to_string(),
                Style::default().fg(theme.title_fg).bold(),
            )),
            Line::from(""),
        ];

        for (key, desc) in hints {
            lines.push(Line::from(vec![
                Span::styled("  • Press ", Style::default().fg(theme.dim)),
                Span::styled(key.to_string(), Style::default().fg(theme.accent).bold()),
                Span::styled(format!(" to {}", desc), Style::default().fg(theme.dim)),
            ]));
        }

        f.render_widget(Paragraph::new(lines), chunks[2]);
    }

    /// Generates the responsive styled lines of the ASCII banner.
    pub fn get_lines(theme: &Theme, width: u16, height: u16) -> Vec<Line<'static>> {
        if height >= 7 && width >= 66 {
            Self::full_banner_lines(theme)
        } else if height >= 4 && width >= 42 {
            Self::compact_banner_lines(theme)
        } else {
            Self::micro_banner_lines(theme)
        }
    }

    fn full_banner_lines(theme: &Theme) -> Vec<Line<'static>> {
        let accent = Style::default().fg(theme.accent).bold();
        let accent_normal = Style::default().fg(theme.accent);
        let title_bold = Style::default().fg(theme.title_fg).bold();
        let dim = Style::default().fg(theme.dim);

        vec![
            // Line 0
            Line::from(vec![
                Span::styled("    /\\       /\\      ", accent),
                Span::styled(" _                       ", title_bold),
                Span::styled("_ _   _       _     ", accent),
            ]),
            // Line 1
            Line::from(vec![
                Span::styled("   /  \\_____/  \\    ", accent),
                Span::styled("| | __ _ _____   _  ", title_bold),
                Span::styled("__ _(_) |_| | __ _| |__  ", accent),
            ]),
            // Line 2 (with terminal prompt in fox visor)
            Line::from(vec![
                Span::styled("  /   / ", accent),
                Span::styled(">", title_bold),
                Span::styled(" _ ", accent),
                Span::styled("\\   \\   ", accent),
                Span::styled("| |/ _` |_  / | | ", title_bold),
                Span::styled("|/ _` | | __| |/ _` | '_ \\ ", accent),
            ]),
            // Line 3
            Line::from(vec![
                Span::styled("  \\   \\__", accent),
                Span::styled("v", dim),
                Span::styled("__/   /   ", accent),
                Span::styled("| | (_| |/ /| |_| ", title_bold),
                Span::styled("| (_| | | |_| | (_| | |_) |", accent),
            ]),
            // Line 4
            Line::from(vec![
                Span::styled("   \\___________/    ", accent),
                Span::styled("|_|\\__,_/___|\\__, ", title_bold),
                Span::styled("|\\__, |_|\\__|_|\\__,_|_.__/ ", accent),
            ]),
            // Line 5
            Line::from(vec![
                Span::raw("                                     "),
                Span::styled("|___/ |___/                     ", accent_normal),
            ]),
            // Line 6 (Tagline)
            Line::from(vec![
                Span::styled("  › ", accent),
                Span::styled("The fast, keyboard-driven Terminal UI for GitLab", dim),
            ]),
        ]
    }

    fn compact_banner_lines(theme: &Theme) -> Vec<Line<'static>> {
        let accent = Style::default().fg(theme.accent).bold();
        let title_bold = Style::default().fg(theme.title_fg).bold();
        let dim = Style::default().fg(theme.dim);
        let key_style = Style::default().fg(theme.shortcut_key).bold();

        vec![
            // Line 0
            Line::from(vec![
                Span::styled("   /\\_/\\    ", accent),
                Span::styled("lazy", title_bold),
                Span::styled("gitlab", accent),
            ]),
            // Line 1
            Line::from(vec![
                Span::styled("  / ", accent),
                Span::styled(">", title_bold),
                Span::styled(" _ ", accent),
                Span::styled("\\   ", accent),
                Span::styled("The fast Terminal UI for GitLab", dim),
            ]),
            // Line 2
            Line::from(vec![
                Span::styled("  \\__", accent),
                Span::styled("v", dim),
                Span::styled("__/   ", accent),
                Span::styled("[", dim),
                Span::styled("?", key_style),
                Span::styled("] Help  ", dim),
                Span::styled("[", dim),
                Span::styled("/", key_style),
                Span::styled("] Search  ", dim),
                Span::styled("[", dim),
                Span::styled("q", key_style),
                Span::styled("] Quit", dim),
            ]),
        ]
    }

    fn micro_banner_lines(theme: &Theme) -> Vec<Line<'static>> {
        let accent = Style::default().fg(theme.accent).bold();
        let title_bold = Style::default().fg(theme.title_fg).bold();
        let dim = Style::default().fg(theme.dim);

        vec![Line::from(vec![
            Span::styled("🦊 ", accent),
            Span::styled(">_ ", title_bold),
            Span::styled("lazy", title_bold),
            Span::styled("gitlab", accent),
            Span::styled(" — Terminal UI for GitLab", dim),
        ])]
    }
}
