use chrono::{DateTime, Utc};
use chrono_humanize::{Accuracy, HumanTime, Tense};
use ratatui::{prelude::*, widgets::*};

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
    if started_at.is_empty() {
        return "00:00".to_string();
    }

    let now = Utc::now();
    let dt_parsed = DateTime::parse_from_rfc3339(started_at)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            DateTime::parse_from_str(started_at, "%Y-%m-%dT%H:%M:%S%.3fZ")
                .map(|dt| dt.with_timezone(&Utc))
        });

    match dt_parsed {
        Ok(dt_utc) => {
            let duration = now.signed_duration_since(dt_utc);
            let seconds = duration.num_seconds() as f64;
            if seconds < 0.0 {
                return "00:00".to_string();
            }
            format_duration(seconds)
        }
        Err(_) => "??:??".to_string(),
    }
}

pub fn format_relative_time(datetime_str: &str) -> String {
    if datetime_str.is_empty() {
        return "None".to_string();
    }

    match DateTime::parse_from_rfc3339(datetime_str) {
        Ok(dt) => {
            let dt_utc: DateTime<Utc> = dt.with_timezone(&Utc);
            let ht = HumanTime::from(dt_utc);
            ht.to_text_en(Accuracy::Rough, Tense::Past)
        }
        Err(_) => match DateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.3fZ") {
            Ok(dt) => {
                let ht = HumanTime::from(dt);
                ht.to_text_en(Accuracy::Rough, Tense::Past)
            }
            Err(_) => datetime_str.to_string(),
        },
    }
}

pub fn format_time_only(datetime_str: &str) -> String {
    if datetime_str.len() >= 19 {
        datetime_str[11..19].to_string()
    } else {
        datetime_str.to_string()
    }
}

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Theme {
    pub border: Color,
    pub highlight_bg: Color,
    pub author: Color,
    pub dim: Color,
    pub success: Color,
    pub failed: Color,
    pub running: Color,
    pub pending: Color,
    pub open_mr: Color,
    pub merged_mr: Color,
    pub closed_mr: Color,
    pub header: Color,
    pub header1: Color,
    pub header2: Color,
    pub project: Color,
    pub issue: Color,
    pub mr: Color,
    pub shortcut_key: Color,
    pub shortcut_desc: Color,
    pub keyword: Color,
    pub code: Color,
    pub code_bg: Color,
    pub range_highlight: Color,
    pub active_border: Color,
    pub inactive_border: Color,
    pub title_fg: Color,
    pub accent: Color,
    pub status_success: Color,
    pub status_failed: Color,
    pub status_running: Color,
    pub status_pending: Color,
    pub status_draft: Color,
    pub status_merged: Color,
    pub label_bg: Color,
    pub label_fg: Color,
    pub badge_bg: Color,
    pub badge_fg: Color,
    pub chip_bg: Color,
    pub chip_fg: Color,
    pub line_number_fg: Color,
    pub hunk_header_bg: Color,
    pub hunk_header_fg: Color,
    pub diff_add_bg: Color,
    pub diff_add_fg: Color,
    pub diff_del_bg: Color,
    pub diff_del_fg: Color,
}
impl Default for Theme {
    fn default() -> Self {

        Self {
            border: Color::Green,
            highlight_bg: Color::Rgb(100, 100, 200),
            author: Color::Magenta,
            dim: Color::Rgb(100, 100, 100),
            success: Color::Rgb(26, 170, 85),
            failed: Color::Rgb(219, 59, 33),
            running: Color::Rgb(31, 117, 203),
            pending: Color::Rgb(252, 148, 3),
            open_mr: Color::Rgb(31, 117, 203),
            merged_mr: Color::Rgb(102, 102, 196),
            closed_mr: Color::Rgb(219, 59, 33),
            header: Color::Rgb(100, 150, 255),
            header1: Color::Cyan,
            header2: Color::Blue,
            project: Color::Cyan,
            issue: Color::Green,
            mr: Color::Magenta,
            shortcut_key: Color::Cyan,
            shortcut_desc: Color::White,
            keyword: Color::Rgb(255, 120, 120),
            code: Color::Rgb(150, 255, 150),
            code_bg: Color::Rgb(30, 30, 30),
            range_highlight: Color::Rgb(60, 60, 100),
        

            active_border: Color::Green,
            inactive_border: Color::Rgb(100, 100, 100),
            title_fg: Color::Rgb(150, 255, 150),
            accent: Color::Rgb(31, 117, 203),
            status_success: Color::Rgb(26, 170, 85),
            status_failed: Color::Rgb(219, 59, 33),
            status_running: Color::Rgb(31, 117, 203),
            status_pending: Color::Rgb(252, 148, 3),
            status_draft: Color::Rgb(100, 100, 100),
            status_merged: Color::Rgb(102, 102, 196),
            label_bg: Color::Rgb(100, 100, 100),
            label_fg: Color::Rgb(30, 30, 30),
            badge_bg: Color::Rgb(100, 100, 200),
            badge_fg: Color::Rgb(150, 255, 150),
            chip_bg: Color::Green,
            chip_fg: Color::Rgb(30, 30, 30),
            line_number_fg: Color::Rgb(100, 100, 100),
            hunk_header_bg: Color::Rgb(100, 100, 200),
            hunk_header_fg: Color::Rgb(150, 255, 150),
            diff_add_bg: Color::Rgb(26, 170, 85),
            diff_add_fg: Color::Rgb(30, 30, 30),
            diff_del_bg: Color::Rgb(219, 59, 33),
            diff_del_fg: Color::Rgb(30, 30, 30),
        }
    }
}

impl Theme {
    pub fn catppuccin_mocha() -> Self {

        Self::default()
    }

    pub fn catppuccin_latte() -> Self {
        Self {
            border: Color::Rgb(64, 160, 43),            // green
            highlight_bg: Color::Rgb(30, 102, 245),     // blue
            author: Color::Rgb(136, 57, 239),           // mauve
            dim: Color::Rgb(156, 160, 176),             // overlay0
            success: Color::Rgb(64, 160, 43),           // green
            failed: Color::Rgb(210, 15, 57),            // red
            running: Color::Rgb(30, 102, 245),          // blue
            pending: Color::Rgb(223, 142, 29),          // yellow
            open_mr: Color::Rgb(30, 102, 245),          // blue
            merged_mr: Color::Rgb(114, 135, 253),       // lavender
            closed_mr: Color::Rgb(210, 15, 57),         // red
            header: Color::Rgb(30, 102, 245),           // blue
            header1: Color::Rgb(32, 159, 181),          // sapphire
            header2: Color::Rgb(136, 57, 239),          // mauve
            project: Color::Rgb(32, 159, 181),          // sapphire
            issue: Color::Rgb(64, 160, 43),             // green
            mr: Color::Rgb(136, 57, 239),               // mauve
            shortcut_key: Color::Rgb(30, 102, 245),     // blue
            shortcut_desc: Color::Rgb(76, 79, 105),     // text
            keyword: Color::Rgb(210, 15, 57),           // red
            code: Color::Rgb(64, 160, 43),              // green
            code_bg: Color::Rgb(230, 233, 239),         // surface0
            range_highlight: Color::Rgb(204, 208, 218), // surface2
        

            active_border: Color::Rgb(64, 160, 43),            // green,
            inactive_border: Color::Rgb(156, 160, 176),             // overlay0,
            title_fg: Color::Rgb(64, 160, 43),              // green,
            accent: Color::Rgb(30, 102, 245),          // blue,
            status_success: Color::Rgb(64, 160, 43),           // green,
            status_failed: Color::Rgb(210, 15, 57),            // red,
            status_running: Color::Rgb(30, 102, 245),          // blue,
            status_pending: Color::Rgb(223, 142, 29),          // yellow,
            status_draft: Color::Rgb(156, 160, 176),             // overlay0,
            status_merged: Color::Rgb(114, 135, 253),       // lavender,
            label_bg: Color::Rgb(156, 160, 176),             // overlay0,
            label_fg: Color::Rgb(230, 233, 239),         // surface0,
            badge_bg: Color::Rgb(30, 102, 245),     // blue,
            badge_fg: Color::Rgb(64, 160, 43),              // green,
            chip_bg: Color::Rgb(64, 160, 43),            // green,
            chip_fg: Color::Rgb(230, 233, 239),         // surface0,
            line_number_fg: Color::Rgb(156, 160, 176),             // overlay0,
            hunk_header_bg: Color::Rgb(30, 102, 245),     // blue,
            hunk_header_fg: Color::Rgb(64, 160, 43),              // green,
            diff_add_bg: Color::Rgb(64, 160, 43),           // green,
            diff_add_fg: Color::Rgb(230, 233, 239),         // surface0,
            diff_del_bg: Color::Rgb(210, 15, 57),            // red,
            diff_del_fg: Color::Rgb(230, 233, 239),         // surface0,
        }
    }

    pub fn tokyo_night() -> Self {

        Self {
            border: Color::Rgb(115, 218, 202),        // teal
            highlight_bg: Color::Rgb(61, 89, 161),    // blue highlight
            author: Color::Rgb(187, 154, 247),        // purple
            dim: Color::Rgb(86, 95, 137),             // comment
            success: Color::Rgb(158, 206, 106),       // green
            failed: Color::Rgb(247, 118, 142),        // red
            running: Color::Rgb(122, 162, 247),       // blue
            pending: Color::Rgb(224, 175, 104),       // yellow
            open_mr: Color::Rgb(122, 162, 247),       // blue
            merged_mr: Color::Rgb(187, 154, 247),     // purple
            closed_mr: Color::Rgb(247, 118, 142),     // red
            header: Color::Rgb(122, 162, 247),        // blue
            header1: Color::Rgb(125, 207, 255),       // cyan
            header2: Color::Rgb(187, 154, 247),       // purple
            project: Color::Rgb(125, 207, 255),       // cyan
            issue: Color::Rgb(158, 206, 106),         // green
            mr: Color::Rgb(187, 154, 247),            // purple
            shortcut_key: Color::Rgb(125, 207, 255),  // cyan
            shortcut_desc: Color::Rgb(192, 202, 245), // fg
            keyword: Color::Rgb(247, 118, 142),       // red
            code: Color::Rgb(158, 206, 106),          // green
            code_bg: Color::Rgb(26, 27, 38),          // bg
            range_highlight: Color::Rgb(41, 46, 66),  // selection
        

            active_border: Color::Rgb(115, 218, 202),        // teal,
            inactive_border: Color::Rgb(86, 95, 137),             // comment,
            title_fg: Color::Rgb(158, 206, 106),          // green,
            accent: Color::Rgb(122, 162, 247),       // blue,
            status_success: Color::Rgb(158, 206, 106),       // green,
            status_failed: Color::Rgb(247, 118, 142),        // red,
            status_running: Color::Rgb(122, 162, 247),       // blue,
            status_pending: Color::Rgb(224, 175, 104),       // yellow,
            status_draft: Color::Rgb(86, 95, 137),             // comment,
            status_merged: Color::Rgb(187, 154, 247),     // purple,
            label_bg: Color::Rgb(86, 95, 137),             // comment,
            label_fg: Color::Rgb(26, 27, 38),          // bg,
            badge_bg: Color::Rgb(61, 89, 161),    // blue highlight,
            badge_fg: Color::Rgb(158, 206, 106),          // green,
            chip_bg: Color::Rgb(115, 218, 202),        // teal,
            chip_fg: Color::Rgb(26, 27, 38),          // bg,
            line_number_fg: Color::Rgb(86, 95, 137),             // comment,
            hunk_header_bg: Color::Rgb(61, 89, 161),    // blue highlight,
            hunk_header_fg: Color::Rgb(158, 206, 106),          // green,
            diff_add_bg: Color::Rgb(158, 206, 106),       // green,
            diff_add_fg: Color::Rgb(26, 27, 38),          // bg,
            diff_del_bg: Color::Rgb(247, 118, 142),        // red,
            diff_del_fg: Color::Rgb(26, 27, 38),          // bg,
        }
    }

    pub fn nord() -> Self {

        Self {
            border: Color::Rgb(143, 188, 187),        // nord7 frost
            highlight_bg: Color::Rgb(94, 129, 172),   // nord10 blue
            author: Color::Rgb(180, 142, 173),        // nord15 purple
            dim: Color::Rgb(97, 110, 136),            // nord3
            success: Color::Rgb(163, 190, 140),       // nord14 green
            failed: Color::Rgb(191, 97, 106),         // nord11 red
            running: Color::Rgb(129, 161, 193),       // nord9 blue
            pending: Color::Rgb(235, 203, 139),       // nord13 yellow
            open_mr: Color::Rgb(129, 161, 193),       // nord9
            merged_mr: Color::Rgb(180, 142, 173),     // nord15
            closed_mr: Color::Rgb(191, 97, 106),      // nord11
            header: Color::Rgb(136, 192, 208),        // nord8
            header1: Color::Rgb(143, 188, 187),       // nord7
            header2: Color::Rgb(180, 142, 173),       // nord15
            project: Color::Rgb(136, 192, 208),       // nord8
            issue: Color::Rgb(163, 190, 140),         // nord14
            mr: Color::Rgb(180, 142, 173),            // nord15
            shortcut_key: Color::Rgb(136, 192, 208),  // nord8
            shortcut_desc: Color::Rgb(229, 233, 240), // nord4 snow
            keyword: Color::Rgb(191, 97, 106),        // nord11
            code: Color::Rgb(163, 190, 140),          // nord14
            code_bg: Color::Rgb(46, 52, 64),          // nord0
            range_highlight: Color::Rgb(67, 76, 94),  // nord1
        

            active_border: Color::Rgb(143, 188, 187),        // nord7 frost,
            inactive_border: Color::Rgb(97, 110, 136),            // nord3,
            title_fg: Color::Rgb(163, 190, 140),          // nord14,
            accent: Color::Rgb(129, 161, 193),       // nord9 blue,
            status_success: Color::Rgb(163, 190, 140),       // nord14 green,
            status_failed: Color::Rgb(191, 97, 106),         // nord11 red,
            status_running: Color::Rgb(129, 161, 193),       // nord9 blue,
            status_pending: Color::Rgb(235, 203, 139),       // nord13 yellow,
            status_draft: Color::Rgb(97, 110, 136),            // nord3,
            status_merged: Color::Rgb(180, 142, 173),     // nord15,
            label_bg: Color::Rgb(97, 110, 136),            // nord3,
            label_fg: Color::Rgb(46, 52, 64),          // nord0,
            badge_bg: Color::Rgb(94, 129, 172),   // nord10 blue,
            badge_fg: Color::Rgb(163, 190, 140),          // nord14,
            chip_bg: Color::Rgb(143, 188, 187),        // nord7 frost,
            chip_fg: Color::Rgb(46, 52, 64),          // nord0,
            line_number_fg: Color::Rgb(97, 110, 136),            // nord3,
            hunk_header_bg: Color::Rgb(94, 129, 172),   // nord10 blue,
            hunk_header_fg: Color::Rgb(163, 190, 140),          // nord14,
            diff_add_bg: Color::Rgb(163, 190, 140),       // nord14 green,
            diff_add_fg: Color::Rgb(46, 52, 64),          // nord0,
            diff_del_bg: Color::Rgb(191, 97, 106),         // nord11 red,
            diff_del_fg: Color::Rgb(46, 52, 64),          // nord0,
        }
    }

    pub fn dracula() -> Self {

        Self {
            border: Color::Rgb(189, 147, 249),        // purple
            highlight_bg: Color::Rgb(98, 114, 164),   // comment / sel
            author: Color::Rgb(255, 121, 198),        // pink
            dim: Color::Rgb(98, 114, 164),            // comment
            success: Color::Rgb(80, 250, 123),        // green
            failed: Color::Rgb(255, 85, 85),          // red
            running: Color::Rgb(139, 233, 253),       // cyan
            pending: Color::Rgb(241, 250, 140),       // yellow
            open_mr: Color::Rgb(139, 233, 253),       // cyan
            merged_mr: Color::Rgb(189, 147, 249),     // purple
            closed_mr: Color::Rgb(255, 85, 85),       // red
            header: Color::Rgb(189, 147, 249),        // purple
            header1: Color::Rgb(139, 233, 253),       // cyan
            header2: Color::Rgb(255, 121, 198),       // pink
            project: Color::Rgb(139, 233, 253),       // cyan
            issue: Color::Rgb(80, 250, 123),          // green
            mr: Color::Rgb(255, 121, 198),            // pink
            shortcut_key: Color::Rgb(139, 233, 253),  // cyan
            shortcut_desc: Color::Rgb(248, 248, 242), // fg
            keyword: Color::Rgb(255, 85, 85),         // red
            code: Color::Rgb(80, 250, 123),           // green
            code_bg: Color::Rgb(40, 42, 54),          // current line
            range_highlight: Color::Rgb(68, 71, 90),  // selection
        

            active_border: Color::Rgb(189, 147, 249),        // purple,
            inactive_border: Color::Rgb(98, 114, 164),            // comment,
            title_fg: Color::Rgb(80, 250, 123),           // green,
            accent: Color::Rgb(139, 233, 253),       // cyan,
            status_success: Color::Rgb(80, 250, 123),        // green,
            status_failed: Color::Rgb(255, 85, 85),          // red,
            status_running: Color::Rgb(139, 233, 253),       // cyan,
            status_pending: Color::Rgb(241, 250, 140),       // yellow,
            status_draft: Color::Rgb(98, 114, 164),            // comment,
            status_merged: Color::Rgb(189, 147, 249),     // purple,
            label_bg: Color::Rgb(98, 114, 164),            // comment,
            label_fg: Color::Rgb(40, 42, 54),          // current line,
            badge_bg: Color::Rgb(98, 114, 164),   // comment / sel,
            badge_fg: Color::Rgb(80, 250, 123),           // green,
            chip_bg: Color::Rgb(189, 147, 249),        // purple,
            chip_fg: Color::Rgb(40, 42, 54),          // current line,
            line_number_fg: Color::Rgb(98, 114, 164),            // comment,
            hunk_header_bg: Color::Rgb(98, 114, 164),   // comment / sel,
            hunk_header_fg: Color::Rgb(80, 250, 123),           // green,
            diff_add_bg: Color::Rgb(80, 250, 123),        // green,
            diff_add_fg: Color::Rgb(40, 42, 54),          // current line,
            diff_del_bg: Color::Rgb(255, 85, 85),          // red,
            diff_del_fg: Color::Rgb(40, 42, 54),          // current line,
        }
    }

    pub fn gruvbox_dark() -> Self {

        Self {
            border: Color::Rgb(184, 187, 38),         // green
            highlight_bg: Color::Rgb(69, 133, 136),   // blue/aqua
            author: Color::Rgb(211, 134, 155),        // purple
            dim: Color::Rgb(146, 131, 116),           // gray
            success: Color::Rgb(184, 187, 38),        // green
            failed: Color::Rgb(251, 73, 52),          // red
            running: Color::Rgb(131, 165, 152),       // blue
            pending: Color::Rgb(250, 189, 47),        // yellow
            open_mr: Color::Rgb(131, 165, 152),       // blue
            merged_mr: Color::Rgb(211, 134, 155),     // purple
            closed_mr: Color::Rgb(251, 73, 52),       // red
            header: Color::Rgb(254, 128, 25),         // orange
            header1: Color::Rgb(142, 192, 124),       // aqua
            header2: Color::Rgb(211, 134, 155),       // purple
            project: Color::Rgb(142, 192, 124),       // aqua
            issue: Color::Rgb(184, 187, 38),          // green
            mr: Color::Rgb(211, 134, 155),            // purple
            shortcut_key: Color::Rgb(254, 128, 25),   // orange
            shortcut_desc: Color::Rgb(235, 219, 178), // light1
            keyword: Color::Rgb(251, 73, 52),         // red
            code: Color::Rgb(184, 187, 38),           // green
            code_bg: Color::Rgb(40, 40, 40),          // dark0
            range_highlight: Color::Rgb(80, 73, 69),  // dark2
        

            active_border: Color::Rgb(184, 187, 38),         // green,
            inactive_border: Color::Rgb(146, 131, 116),           // gray,
            title_fg: Color::Rgb(184, 187, 38),           // green,
            accent: Color::Rgb(131, 165, 152),       // blue,
            status_success: Color::Rgb(184, 187, 38),        // green,
            status_failed: Color::Rgb(251, 73, 52),          // red,
            status_running: Color::Rgb(131, 165, 152),       // blue,
            status_pending: Color::Rgb(250, 189, 47),        // yellow,
            status_draft: Color::Rgb(146, 131, 116),           // gray,
            status_merged: Color::Rgb(211, 134, 155),     // purple,
            label_bg: Color::Rgb(146, 131, 116),           // gray,
            label_fg: Color::Rgb(40, 40, 40),          // dark0,
            badge_bg: Color::Rgb(69, 133, 136),   // blue/aqua,
            badge_fg: Color::Rgb(184, 187, 38),           // green,
            chip_bg: Color::Rgb(184, 187, 38),         // green,
            chip_fg: Color::Rgb(40, 40, 40),          // dark0,
            line_number_fg: Color::Rgb(146, 131, 116),           // gray,
            hunk_header_bg: Color::Rgb(69, 133, 136),   // blue/aqua,
            hunk_header_fg: Color::Rgb(184, 187, 38),           // green,
            diff_add_bg: Color::Rgb(184, 187, 38),        // green,
            diff_add_fg: Color::Rgb(40, 40, 40),          // dark0,
            diff_del_bg: Color::Rgb(251, 73, 52),          // red,
            diff_del_fg: Color::Rgb(40, 40, 40),          // dark0,
        }
    }

    pub fn github_dark() -> Self {

        Self {
            border: Color::Rgb(46, 160, 67),          // green
            highlight_bg: Color::Rgb(31, 111, 235),   // blue
            author: Color::Rgb(163, 113, 247),        // purple
            dim: Color::Rgb(110, 118, 129),           // muted
            success: Color::Rgb(63, 185, 80),         // green
            failed: Color::Rgb(248, 81, 73),          // red
            running: Color::Rgb(88, 166, 255),        // blue
            pending: Color::Rgb(210, 153, 34),        // yellow
            open_mr: Color::Rgb(88, 166, 255),        // blue
            merged_mr: Color::Rgb(163, 113, 247),     // purple
            closed_mr: Color::Rgb(248, 81, 73),       // red
            header: Color::Rgb(88, 166, 255),         // blue
            header1: Color::Rgb(57, 197, 187),        // cyan
            header2: Color::Rgb(163, 113, 247),       // purple
            project: Color::Rgb(88, 166, 255),        // blue
            issue: Color::Rgb(63, 185, 80),           // green
            mr: Color::Rgb(163, 113, 247),            // purple
            shortcut_key: Color::Rgb(88, 166, 255),   // blue
            shortcut_desc: Color::Rgb(240, 246, 252), // fg
            keyword: Color::Rgb(248, 81, 73),         // red
            code: Color::Rgb(63, 185, 80),            // green
            code_bg: Color::Rgb(22, 27, 34),          // dark
            range_highlight: Color::Rgb(48, 54, 61),  // sel
        

            active_border: Color::Rgb(46, 160, 67),          // green,
            inactive_border: Color::Rgb(110, 118, 129),           // muted,
            title_fg: Color::Rgb(63, 185, 80),            // green,
            accent: Color::Rgb(88, 166, 255),        // blue,
            status_success: Color::Rgb(63, 185, 80),         // green,
            status_failed: Color::Rgb(248, 81, 73),          // red,
            status_running: Color::Rgb(88, 166, 255),        // blue,
            status_pending: Color::Rgb(210, 153, 34),        // yellow,
            status_draft: Color::Rgb(110, 118, 129),           // muted,
            status_merged: Color::Rgb(163, 113, 247),     // purple,
            label_bg: Color::Rgb(110, 118, 129),           // muted,
            label_fg: Color::Rgb(22, 27, 34),          // dark,
            badge_bg: Color::Rgb(31, 111, 235),   // blue,
            badge_fg: Color::Rgb(63, 185, 80),            // green,
            chip_bg: Color::Rgb(46, 160, 67),          // green,
            chip_fg: Color::Rgb(22, 27, 34),          // dark,
            line_number_fg: Color::Rgb(110, 118, 129),           // muted,
            hunk_header_bg: Color::Rgb(31, 111, 235),   // blue,
            hunk_header_fg: Color::Rgb(63, 185, 80),            // green,
            diff_add_bg: Color::Rgb(63, 185, 80),         // green,
            diff_add_fg: Color::Rgb(22, 27, 34),          // dark,
            diff_del_bg: Color::Rgb(248, 81, 73),          // red,
            diff_del_fg: Color::Rgb(22, 27, 34),          // dark,
        }
    }

    pub fn github_light() -> Self {

        Self {
            border: Color::Rgb(26, 127, 55),            // green
            highlight_bg: Color::Rgb(9, 105, 218),      // blue
            author: Color::Rgb(130, 80, 223),           // purple
            dim: Color::Rgb(110, 119, 129),             // muted
            success: Color::Rgb(26, 127, 55),           // green
            failed: Color::Rgb(207, 34, 46),            // red
            running: Color::Rgb(9, 105, 218),           // blue
            pending: Color::Rgb(154, 103, 0),           // yellow
            open_mr: Color::Rgb(9, 105, 218),           // blue
            merged_mr: Color::Rgb(130, 80, 223),        // purple
            closed_mr: Color::Rgb(207, 34, 46),         // red
            header: Color::Rgb(9, 105, 218),            // blue
            header1: Color::Rgb(5, 80, 174),            // cyan
            header2: Color::Rgb(130, 80, 223),          // purple
            project: Color::Rgb(9, 105, 218),           // blue
            issue: Color::Rgb(26, 127, 55),             // green
            mr: Color::Rgb(130, 80, 223),               // purple
            shortcut_key: Color::Rgb(9, 105, 218),      // blue
            shortcut_desc: Color::Rgb(31, 35, 40),      // dark fg
            keyword: Color::Rgb(207, 34, 46),           // red
            code: Color::Rgb(26, 127, 55),              // green
            code_bg: Color::Rgb(246, 248, 250),         // light
            range_highlight: Color::Rgb(221, 244, 255), // light blue
        

            active_border: Color::Rgb(26, 127, 55),            // green,
            inactive_border: Color::Rgb(110, 119, 129),             // muted,
            title_fg: Color::Rgb(26, 127, 55),              // green,
            accent: Color::Rgb(9, 105, 218),           // blue,
            status_success: Color::Rgb(26, 127, 55),           // green,
            status_failed: Color::Rgb(207, 34, 46),            // red,
            status_running: Color::Rgb(9, 105, 218),           // blue,
            status_pending: Color::Rgb(154, 103, 0),           // yellow,
            status_draft: Color::Rgb(110, 119, 129),             // muted,
            status_merged: Color::Rgb(130, 80, 223),        // purple,
            label_bg: Color::Rgb(110, 119, 129),             // muted,
            label_fg: Color::Rgb(246, 248, 250),         // light,
            badge_bg: Color::Rgb(9, 105, 218),      // blue,
            badge_fg: Color::Rgb(26, 127, 55),              // green,
            chip_bg: Color::Rgb(26, 127, 55),            // green,
            chip_fg: Color::Rgb(246, 248, 250),         // light,
            line_number_fg: Color::Rgb(110, 119, 129),             // muted,
            hunk_header_bg: Color::Rgb(9, 105, 218),      // blue,
            hunk_header_fg: Color::Rgb(26, 127, 55),              // green,
            diff_add_bg: Color::Rgb(26, 127, 55),           // green,
            diff_add_fg: Color::Rgb(246, 248, 250),         // light,
            diff_del_bg: Color::Rgb(207, 34, 46),            // red,
            diff_del_fg: Color::Rgb(246, 248, 250),         // light,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().replace(['-', '_', ' '], "").as_str() {
            "catppuccinmocha" | "mocha" | "default" => Some(Self::catppuccin_mocha()),
            "catppuccinlatte" | "latte" => Some(Self::catppuccin_latte()),
            "tokyonight" | "tokyo" => Some(Self::tokyo_night()),
            "nord" => Some(Self::nord()),
            "dracula" => Some(Self::dracula()),
            "gruvboxdark" | "gruvbox" => Some(Self::gruvbox_dark()),
            "githubdark" => Some(Self::github_dark()),
            "githublight" => Some(Self::github_light()),
            _ => None,
        }
    }

    pub fn load() -> Self {
        let config_path = "theme.toml";
        if let Ok(content) = std::fs::read_to_string(config_path) {
            // First check if there is a preset theme = "name"
            #[derive(Deserialize)]
            struct ThemeConfig {
                theme: Option<String>,
                #[serde(flatten)]
                custom: Option<Theme>,
            }
            if let Ok(cfg) = toml::from_str::<ThemeConfig>(&content) {
                if let Some(preset) = cfg.theme.as_deref().and_then(Self::from_name) {
                    return preset;
                }
                if let Some(custom) = cfg.custom {
                    return custom;
                }
            }
            if let Ok(t) = toml::from_str::<Theme>(&content) {
                return t;
            }
        }
        Self::default()
    }
}

pub fn get_status_style(status: &str, theme: &Theme) -> Style {
    match status {
        "opened" | "open" => Style::default().fg(theme.open_mr),
        "merged" | "resolved" => Style::default().fg(theme.merged_mr),
        "closed" => Style::default().fg(theme.closed_mr),
        "success" => Style::default().fg(theme.success),
        "failed" => Style::default().fg(theme.failed),
        "running" => Style::default().fg(theme.running),
        "pending" => Style::default().fg(theme.pending),
        "canceled" | "skipped" | "manual" | "created" | "scheduled" => {
            Style::default().fg(theme.dim)
        }
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
        spans.push(Span::styled(
            *desc,
            Style::default().fg(theme.shortcut_desc),
        ));
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

pub fn get_contrasting_text_color(bg_hex: &str) -> Color {
    let hex = bg_hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32;
        let luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
        if luminance > 0.5 {
            Color::Black
        } else {
            Color::White
        }
    } else {
        Color::White
    }
}

pub fn truncate_str(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", cut)
}

pub fn render_status_badge(status: &str, theme: &Theme) -> (String, Style) {
    match status.to_lowercase().as_str() {
        "success" | "passed" => ("✔".to_string(), Style::default().fg(theme.status_success).bold()),
        "failed" | "error" => ("✖".to_string(), Style::default().fg(theme.status_failed).bold()),
        "running" | "in_progress" => ("⟳".to_string(), Style::default().fg(theme.status_running).bold()),
        "pending" | "waiting" => ("⏸".to_string(), Style::default().fg(theme.status_pending).bold()),
        "draft" => ("⭘".to_string(), Style::default().fg(theme.status_draft).bold()),
        "merged" => ("✦".to_string(), Style::default().fg(theme.status_merged).bold()),
        _ => (status.to_string(), Style::default().fg(theme.dim)),
    }
}

pub fn render_pill<'a>(label: &str, fg: Color, bg: Color) -> Span<'a> {
    Span::styled(format!(" {} ", label), Style::default().fg(fg).bg(bg))
}

pub fn render_progress_bar<'a>(completed: u64, total: u64, width: usize, theme: &Theme) -> Line<'a> {
    if total == 0 {
        return Line::from(String::new());
    }
    let ratio = (completed as f64 / total as f64).clamp(0.0, 1.0);
    let filled_width = (ratio * width as f64).round() as usize;
    let empty_width = width.saturating_sub(filled_width);
    
    let filled = "█".repeat(filled_width);
    let empty = "░".repeat(empty_width);
    
    let style = if ratio >= 1.0 {
        Style::default().fg(theme.status_success)
    } else {
        Style::default().fg(theme.status_running)
    };
    
    Line::from(vec![
        Span::styled(filled, style),
        Span::styled(empty, Style::default().fg(theme.dim)),
    ])
}

pub fn format_time_ago(datetime_str: &str) -> String {
    format_relative_time(datetime_str)
}

pub fn get_modern_block<'a>(title: &'a str, is_focused: bool, theme: &Theme) -> Block<'a> {
    let border_style = if is_focused {
        Style::default().fg(theme.active_border)
    } else {
        Style::default().fg(theme.inactive_border)
    };

    let title_style = if is_focused {
        Style::default().fg(theme.accent).bold()
    } else {
        Style::default().fg(theme.title_fg)
    };

    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Span::styled(format!(" {} ", title), title_style))
}

pub fn render_keybinding_badge<'a>(key: &str, theme: &Theme) -> Span<'a> {
    Span::styled(format!(" {} ", key), Style::default().fg(theme.badge_fg).bg(theme.badge_bg).bold())
}

pub fn render_milestone_chip<'a>(milestone: &str, theme: &Theme) -> Span<'a> {
    Span::styled(format!(" ⚑ {} ", milestone), Style::default().fg(theme.chip_fg).bg(theme.chip_bg))
}

pub fn render_label_badge<'a>(label: &str, bg_hex: &str) -> Span<'a> {
    let bg_color = hex_to_color(bg_hex);
    let fg_color = get_contrasting_text_color(bg_hex);
    Span::styled(format!(" {} ", label), Style::default().fg(fg_color).bg(bg_color))
}

pub fn render_breadcrumbs<'a>(crumbs: &[&str], theme: &Theme) -> Line<'a> {
    let mut spans = Vec::new();
    for (i, crumb) in crumbs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" ❯ ", Style::default().fg(theme.dim)));
        }
        spans.push(Span::styled(crumb.to_string(), Style::default().fg(theme.title_fg).bold()));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.0), "00:00");
        assert_eq!(format_duration(59.0), "00:59");
        assert_eq!(format_duration(60.0), "01:00");
        assert_eq!(format_duration(3599.0), "59:59");
        assert_eq!(format_duration(3600.0), "01:00:00");
        assert_eq!(format_duration(3661.0), "01:01:01");
    }

    #[test]
    fn test_get_elapsed_time() {
        assert_eq!(get_elapsed_time(""), "00:00");
        assert_eq!(get_elapsed_time("invalid"), "??:??");
        assert_eq!(get_elapsed_time("2100-01-01T00:00:00Z"), "00:00");
    }

    #[test]
    fn test_format_relative_time() {
        assert_eq!(format_relative_time(""), "None");
        assert_eq!(format_relative_time("invalid"), "invalid");
        let s = format_relative_time("2020-01-01T00:00:00Z");
        assert!(s.contains("years") || s.contains("months") || s.contains("days"));
    }

    #[test]
    fn test_format_time_only() {
        assert_eq!(format_time_only("2026-04-14T19:37:21.116729Z"), "19:37:21");
        assert_eq!(format_time_only("too short"), "too short");
    }

    #[test]
    fn test_hex_to_color() {
        assert_eq!(hex_to_color("#ffffff"), Color::Rgb(255, 255, 255));
        assert_eq!(hex_to_color("000000"), Color::Rgb(0, 0, 0));
        assert_eq!(hex_to_color("invalid"), Color::Gray);
        assert_eq!(hex_to_color("#123"), Color::Gray);
    }

    #[test]
    fn test_get_status_style() {
        let theme = Theme::default();
        assert_eq!(get_status_style("success", &theme).fg, Some(theme.success));
        assert_eq!(get_status_style("failed", &theme).fg, Some(theme.failed));
        assert_eq!(get_status_style("opened", &theme).fg, Some(theme.open_mr));
        assert_eq!(get_status_style("running", &theme).fg, Some(theme.running));
        assert_eq!(get_status_style("pending", &theme).fg, Some(theme.pending));
    }

    #[test]
    fn test_centered_rect() {
        let r = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 50, r);
        assert_eq!(centered.width, 50);
        assert_eq!(centered.height, 50);
        assert_eq!(centered.x, 25);
        assert_eq!(centered.y, 25);
    }
}
