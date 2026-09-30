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
            border: Color::Rgb(166, 227, 161),         // #a6e3a1 green
            highlight_bg: Color::Rgb(49, 50, 68),      // #313244 surface0
            author: Color::Rgb(245, 194, 231),         // #f5c2e7 pink
            dim: Color::Rgb(147, 153, 178),            // #9399b2 overlay2
            success: Color::Rgb(166, 227, 161),        // #a6e3a1 green
            failed: Color::Rgb(243, 139, 168),         // #f38ba8 red
            running: Color::Rgb(137, 180, 250),        // #89b4fa blue
            pending: Color::Rgb(249, 226, 175),        // #f9e2af yellow
            open_mr: Color::Rgb(137, 180, 250),        // #89b4fa blue
            merged_mr: Color::Rgb(203, 166, 247),      // #cba6f7 mauve
            closed_mr: Color::Rgb(243, 139, 168),      // #f38ba8 red
            header: Color::Rgb(137, 180, 250),         // #89b4fa blue
            header1: Color::Rgb(137, 220, 235),        // #89dceb sky
            header2: Color::Rgb(203, 166, 247),        // #cba6f7 mauve
            project: Color::Rgb(137, 220, 235),        // #89dceb sky
            issue: Color::Rgb(166, 227, 161),          // #a6e3a1 green
            mr: Color::Rgb(203, 166, 247),             // #cba6f7 mauve
            shortcut_key: Color::Rgb(137, 220, 235),   // #89dceb sky
            shortcut_desc: Color::Rgb(205, 214, 244),  // #cdd6f4 text
            keyword: Color::Rgb(243, 139, 168),        // #f38ba8 red
            code: Color::Rgb(166, 227, 161),           // #a6e3a1 green
            code_bg: Color::Rgb(24, 24, 37),           // #181825 mantle
            range_highlight: Color::Rgb(49, 50, 68),   // #313244 surface0
            active_border: Color::Rgb(166, 227, 161),  // #a6e3a1 green
            inactive_border: Color::Rgb(88, 91, 112),  // #585b70 surface2
            title_fg: Color::Rgb(205, 214, 244),       // #cdd6f4 text
            accent: Color::Rgb(137, 180, 250),         // #89b4fa blue
            status_success: Color::Rgb(166, 227, 161), // #a6e3a1 green
            status_failed: Color::Rgb(243, 139, 168),  // #f38ba8 red
            status_running: Color::Rgb(137, 180, 250), // #89b4fa blue
            status_pending: Color::Rgb(249, 226, 175), // #f9e2af yellow
            status_draft: Color::Rgb(147, 153, 178),   // #9399b2 overlay2
            status_merged: Color::Rgb(203, 166, 247),  // #cba6f7 mauve
            label_bg: Color::Rgb(49, 50, 68),          // #313244 surface0
            label_fg: Color::Rgb(205, 214, 244),       // #cdd6f4 text
            badge_bg: Color::Rgb(49, 50, 68),          // #313244 surface0
            badge_fg: Color::Rgb(137, 220, 235),       // #89dceb sky
            chip_bg: Color::Rgb(49, 50, 68),           // #313244 surface0
            chip_fg: Color::Rgb(166, 227, 161),        // #a6e3a1 green
            line_number_fg: Color::Rgb(147, 153, 178), // #9399b2 overlay2
            hunk_header_bg: Color::Rgb(49, 50, 68),    // #313244 surface0
            hunk_header_fg: Color::Rgb(137, 180, 250), // #89b4fa blue
            diff_add_bg: Color::Rgb(35, 59, 46),       // dark green tint
            diff_add_fg: Color::Rgb(166, 227, 161),    // #a6e3a1 green
            diff_del_bg: Color::Rgb(68, 36, 48),       // dark red tint
            diff_del_fg: Color::Rgb(243, 139, 168),    // #f38ba8 red
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
            highlight_bg: Color::Rgb(204, 208, 218),    // surface0 selection
            author: Color::Rgb(136, 57, 239),           // mauve
            dim: Color::Rgb(124, 127, 147),             // subtext0
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
            code_bg: Color::Rgb(230, 233, 239),         // mantle
            range_highlight: Color::Rgb(204, 208, 218), // surface0
            active_border: Color::Rgb(64, 160, 43),     // green
            inactive_border: Color::Rgb(156, 160, 176), // overlay0
            title_fg: Color::Rgb(76, 79, 105),          // text
            accent: Color::Rgb(30, 102, 245),           // blue
            status_success: Color::Rgb(64, 160, 43),    // green
            status_failed: Color::Rgb(210, 15, 57),     // red
            status_running: Color::Rgb(30, 102, 245),   // blue
            status_pending: Color::Rgb(223, 142, 29),   // yellow
            status_draft: Color::Rgb(124, 127, 147),    // subtext0
            status_merged: Color::Rgb(114, 135, 253),   // lavender
            label_bg: Color::Rgb(204, 208, 218),        // surface0
            label_fg: Color::Rgb(76, 79, 105),          // text
            badge_bg: Color::Rgb(204, 208, 218),        // surface0
            badge_fg: Color::Rgb(30, 102, 245),         // blue
            chip_bg: Color::Rgb(204, 208, 218),         // surface0
            chip_fg: Color::Rgb(64, 160, 43),           // green
            line_number_fg: Color::Rgb(124, 127, 147),  // subtext0
            hunk_header_bg: Color::Rgb(204, 208, 218),  // surface0
            hunk_header_fg: Color::Rgb(30, 102, 245),   // blue
            diff_add_bg: Color::Rgb(220, 240, 220),     // light green tint
            diff_add_fg: Color::Rgb(64, 160, 43),       // green
            diff_del_bg: Color::Rgb(245, 220, 225),     // light red tint
            diff_del_fg: Color::Rgb(210, 15, 57),       // red
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            border: Color::Rgb(115, 218, 202),         // teal
            highlight_bg: Color::Rgb(41, 46, 66),      // selection
            author: Color::Rgb(187, 154, 247),         // purple
            dim: Color::Rgb(115, 126, 172),            // subtext / comment
            success: Color::Rgb(158, 206, 106),        // green
            failed: Color::Rgb(247, 118, 142),         // red
            running: Color::Rgb(122, 162, 247),        // blue
            pending: Color::Rgb(224, 175, 104),        // yellow
            open_mr: Color::Rgb(122, 162, 247),        // blue
            merged_mr: Color::Rgb(187, 154, 247),      // purple
            closed_mr: Color::Rgb(247, 118, 142),      // red
            header: Color::Rgb(122, 162, 247),         // blue
            header1: Color::Rgb(125, 207, 255),        // cyan
            header2: Color::Rgb(187, 154, 247),        // purple
            project: Color::Rgb(125, 207, 255),        // cyan
            issue: Color::Rgb(158, 206, 106),          // green
            mr: Color::Rgb(187, 154, 247),             // purple
            shortcut_key: Color::Rgb(125, 207, 255),   // cyan
            shortcut_desc: Color::Rgb(192, 202, 245),  // fg
            keyword: Color::Rgb(247, 118, 142),        // red
            code: Color::Rgb(158, 206, 106),           // green
            code_bg: Color::Rgb(22, 22, 30),           // bg
            range_highlight: Color::Rgb(41, 46, 66),   // selection
            active_border: Color::Rgb(115, 218, 202),  // teal
            inactive_border: Color::Rgb(86, 95, 137),  // comment
            title_fg: Color::Rgb(192, 202, 245),       // fg
            accent: Color::Rgb(122, 162, 247),         // blue
            status_success: Color::Rgb(158, 206, 106), // green
            status_failed: Color::Rgb(247, 118, 142),  // red
            status_running: Color::Rgb(122, 162, 247), // blue
            status_pending: Color::Rgb(224, 175, 104), // yellow
            status_draft: Color::Rgb(115, 126, 172),   // comment
            status_merged: Color::Rgb(187, 154, 247),  // purple
            label_bg: Color::Rgb(41, 46, 66),          // selection
            label_fg: Color::Rgb(192, 202, 245),       // fg
            badge_bg: Color::Rgb(41, 46, 66),          // selection
            badge_fg: Color::Rgb(125, 207, 255),       // cyan
            chip_bg: Color::Rgb(41, 46, 66),           // selection
            chip_fg: Color::Rgb(158, 206, 106),        // green
            line_number_fg: Color::Rgb(115, 126, 172), // comment
            hunk_header_bg: Color::Rgb(41, 46, 66),    // selection
            hunk_header_fg: Color::Rgb(122, 162, 247), // blue
            diff_add_bg: Color::Rgb(30, 50, 40),       // dark green tint
            diff_add_fg: Color::Rgb(158, 206, 106),    // green
            diff_del_bg: Color::Rgb(55, 30, 40),       // dark red tint
            diff_del_fg: Color::Rgb(247, 118, 142),    // red
        }
    }

    pub fn nord() -> Self {
        Self {
            border: Color::Rgb(143, 188, 187),         // nord7 frost
            highlight_bg: Color::Rgb(67, 76, 94),      // nord1 selection
            author: Color::Rgb(180, 142, 173),         // nord15 purple
            dim: Color::Rgb(129, 142, 168),            // high contrast dim
            success: Color::Rgb(163, 190, 140),        // nord14 green
            failed: Color::Rgb(191, 97, 106),          // nord11 red
            running: Color::Rgb(129, 161, 193),        // nord9 blue
            pending: Color::Rgb(235, 203, 139),        // nord13 yellow
            open_mr: Color::Rgb(129, 161, 193),        // nord9
            merged_mr: Color::Rgb(180, 142, 173),      // nord15
            closed_mr: Color::Rgb(191, 97, 106),       // nord11
            header: Color::Rgb(136, 192, 208),         // nord8
            header1: Color::Rgb(143, 188, 187),        // nord7
            header2: Color::Rgb(180, 142, 173),        // nord15
            project: Color::Rgb(136, 192, 208),        // nord8
            issue: Color::Rgb(163, 190, 140),          // nord14
            mr: Color::Rgb(180, 142, 173),             // nord15
            shortcut_key: Color::Rgb(136, 192, 208),   // nord8
            shortcut_desc: Color::Rgb(229, 233, 240),  // nord4 snow
            keyword: Color::Rgb(191, 97, 106),         // nord11
            code: Color::Rgb(163, 190, 140),           // nord14
            code_bg: Color::Rgb(46, 52, 64),           // nord0
            range_highlight: Color::Rgb(67, 76, 94),   // nord1
            active_border: Color::Rgb(143, 188, 187),  // nord7 frost
            inactive_border: Color::Rgb(97, 110, 136), // nord3
            title_fg: Color::Rgb(229, 233, 240),       // nord4 snow
            accent: Color::Rgb(129, 161, 193),         // nord9 blue
            status_success: Color::Rgb(163, 190, 140), // nord14 green
            status_failed: Color::Rgb(191, 97, 106),   // nord11 red
            status_running: Color::Rgb(129, 161, 193), // nord9 blue
            status_pending: Color::Rgb(235, 203, 139), // nord13 yellow
            status_draft: Color::Rgb(129, 142, 168),   // dim
            status_merged: Color::Rgb(180, 142, 173),  // nord15
            label_bg: Color::Rgb(67, 76, 94),          // nord1
            label_fg: Color::Rgb(229, 233, 240),       // nord4
            badge_bg: Color::Rgb(67, 76, 94),          // nord1
            badge_fg: Color::Rgb(136, 192, 208),       // nord8
            chip_bg: Color::Rgb(67, 76, 94),           // nord1
            chip_fg: Color::Rgb(163, 190, 140),        // nord14
            line_number_fg: Color::Rgb(129, 142, 168), // dim
            hunk_header_bg: Color::Rgb(67, 76, 94),    // nord1
            hunk_header_fg: Color::Rgb(129, 161, 193), // nord9
            diff_add_bg: Color::Rgb(35, 55, 45),       // dark green tint
            diff_add_fg: Color::Rgb(163, 190, 140),    // nord14 green
            diff_del_bg: Color::Rgb(55, 35, 40),       // dark red tint
            diff_del_fg: Color::Rgb(191, 97, 106),     // nord11 red
        }
    }

    pub fn dracula() -> Self {
        Self {
            border: Color::Rgb(189, 147, 249),         // purple
            highlight_bg: Color::Rgb(68, 71, 90),      // selection
            author: Color::Rgb(255, 121, 198),         // pink
            dim: Color::Rgb(140, 155, 200),            // high contrast comment
            success: Color::Rgb(80, 250, 123),         // green
            failed: Color::Rgb(255, 85, 85),           // red
            running: Color::Rgb(139, 233, 253),        // cyan
            pending: Color::Rgb(241, 250, 140),        // yellow
            open_mr: Color::Rgb(139, 233, 253),        // cyan
            merged_mr: Color::Rgb(189, 147, 249),      // purple
            closed_mr: Color::Rgb(255, 85, 85),        // red
            header: Color::Rgb(189, 147, 249),         // purple
            header1: Color::Rgb(139, 233, 253),        // cyan
            header2: Color::Rgb(255, 121, 198),        // pink
            project: Color::Rgb(139, 233, 253),        // cyan
            issue: Color::Rgb(80, 250, 123),           // green
            mr: Color::Rgb(255, 121, 198),             // pink
            shortcut_key: Color::Rgb(139, 233, 253),   // cyan
            shortcut_desc: Color::Rgb(248, 248, 242),  // fg
            keyword: Color::Rgb(255, 85, 85),          // red
            code: Color::Rgb(80, 250, 123),            // green
            code_bg: Color::Rgb(40, 42, 54),           // current line
            range_highlight: Color::Rgb(68, 71, 90),   // selection
            active_border: Color::Rgb(189, 147, 249),  // purple
            inactive_border: Color::Rgb(98, 114, 164), // comment
            title_fg: Color::Rgb(248, 248, 242),       // fg
            accent: Color::Rgb(139, 233, 253),         // cyan
            status_success: Color::Rgb(80, 250, 123),  // green
            status_failed: Color::Rgb(255, 85, 85),    // red
            status_running: Color::Rgb(139, 233, 253), // cyan
            status_pending: Color::Rgb(241, 250, 140), // yellow
            status_draft: Color::Rgb(140, 155, 200),   // comment
            status_merged: Color::Rgb(189, 147, 249),  // purple
            label_bg: Color::Rgb(68, 71, 90),          // selection
            label_fg: Color::Rgb(248, 248, 242),       // fg
            badge_bg: Color::Rgb(68, 71, 90),          // selection
            badge_fg: Color::Rgb(139, 233, 253),       // cyan
            chip_bg: Color::Rgb(68, 71, 90),           // selection
            chip_fg: Color::Rgb(80, 250, 123),         // green
            line_number_fg: Color::Rgb(140, 155, 200), // comment
            hunk_header_bg: Color::Rgb(68, 71, 90),    // selection
            hunk_header_fg: Color::Rgb(189, 147, 249), // purple
            diff_add_bg: Color::Rgb(25, 55, 35),       // dark green tint
            diff_add_fg: Color::Rgb(80, 250, 123),     // green
            diff_del_bg: Color::Rgb(60, 30, 35),       // dark red tint
            diff_del_fg: Color::Rgb(255, 85, 85),      // red
        }
    }

    pub fn gruvbox_dark() -> Self {
        Self {
            border: Color::Rgb(184, 187, 38),           // green
            highlight_bg: Color::Rgb(80, 73, 69),       // dark2 selection
            author: Color::Rgb(211, 134, 155),          // purple
            dim: Color::Rgb(168, 153, 132),             // gray_246
            success: Color::Rgb(184, 187, 38),          // green
            failed: Color::Rgb(251, 73, 52),            // red
            running: Color::Rgb(131, 165, 152),         // blue
            pending: Color::Rgb(250, 189, 47),          // yellow
            open_mr: Color::Rgb(131, 165, 152),         // blue
            merged_mr: Color::Rgb(211, 134, 155),       // purple
            closed_mr: Color::Rgb(251, 73, 52),         // red
            header: Color::Rgb(254, 128, 25),           // orange
            header1: Color::Rgb(142, 192, 124),         // aqua
            header2: Color::Rgb(211, 134, 155),         // purple
            project: Color::Rgb(142, 192, 124),         // aqua
            issue: Color::Rgb(184, 187, 38),            // green
            mr: Color::Rgb(211, 134, 155),              // purple
            shortcut_key: Color::Rgb(254, 128, 25),     // orange
            shortcut_desc: Color::Rgb(235, 219, 178),   // light1
            keyword: Color::Rgb(251, 73, 52),           // red
            code: Color::Rgb(184, 187, 38),             // green
            code_bg: Color::Rgb(40, 40, 40),            // dark0
            range_highlight: Color::Rgb(80, 73, 69),    // dark2
            active_border: Color::Rgb(184, 187, 38),    // green
            inactive_border: Color::Rgb(146, 131, 116), // gray
            title_fg: Color::Rgb(235, 219, 178),        // light1
            accent: Color::Rgb(131, 165, 152),          // blue
            status_success: Color::Rgb(184, 187, 38),   // green
            status_failed: Color::Rgb(251, 73, 52),     // red
            status_running: Color::Rgb(131, 165, 152),  // blue
            status_pending: Color::Rgb(250, 189, 47),   // yellow
            status_draft: Color::Rgb(168, 153, 132),    // gray
            status_merged: Color::Rgb(211, 134, 155),   // purple
            label_bg: Color::Rgb(80, 73, 69),           // dark2
            label_fg: Color::Rgb(235, 219, 178),        // light1
            badge_bg: Color::Rgb(80, 73, 69),           // dark2
            badge_fg: Color::Rgb(142, 192, 124),        // aqua
            chip_bg: Color::Rgb(80, 73, 69),            // dark2
            chip_fg: Color::Rgb(184, 187, 38),          // green
            line_number_fg: Color::Rgb(168, 153, 132),  // gray
            hunk_header_bg: Color::Rgb(80, 73, 69),     // dark2
            hunk_header_fg: Color::Rgb(131, 165, 152),  // blue
            diff_add_bg: Color::Rgb(40, 55, 30),        // dark green tint
            diff_add_fg: Color::Rgb(184, 187, 38),      // green
            diff_del_bg: Color::Rgb(60, 30, 25),        // dark red tint
            diff_del_fg: Color::Rgb(251, 73, 52),       // red
        }
    }

    pub fn gruvbox_light() -> Self {
        Self {
            border: Color::Rgb(121, 116, 14),           // green
            highlight_bg: Color::Rgb(213, 196, 161),    // light2 selection
            author: Color::Rgb(143, 63, 113),           // purple
            dim: Color::Rgb(124, 111, 100),             // gray
            success: Color::Rgb(121, 116, 14),          // green
            failed: Color::Rgb(204, 36, 29),            // red
            running: Color::Rgb(7, 102, 120),           // blue
            pending: Color::Rgb(181, 118, 20),          // yellow/orange
            open_mr: Color::Rgb(7, 102, 120),           // blue
            merged_mr: Color::Rgb(143, 63, 113),        // purple
            closed_mr: Color::Rgb(204, 36, 29),         // red
            header: Color::Rgb(175, 58, 3),             // orange
            header1: Color::Rgb(66, 123, 88),           // aqua
            header2: Color::Rgb(143, 63, 113),          // purple
            project: Color::Rgb(7, 102, 120),           // blue
            issue: Color::Rgb(121, 116, 14),            // green
            mr: Color::Rgb(143, 63, 113),               // purple
            shortcut_key: Color::Rgb(175, 58, 3),       // orange
            shortcut_desc: Color::Rgb(40, 40, 40),      // dark0
            keyword: Color::Rgb(204, 36, 29),           // red
            code: Color::Rgb(121, 116, 14),             // green
            code_bg: Color::Rgb(235, 219, 178),         // light1
            range_highlight: Color::Rgb(213, 196, 161), // light2
            active_border: Color::Rgb(121, 116, 14),    // green
            inactive_border: Color::Rgb(146, 131, 116), // gray
            title_fg: Color::Rgb(40, 40, 40),           // dark0
            accent: Color::Rgb(7, 102, 120),            // blue
            status_success: Color::Rgb(121, 116, 14),   // green
            status_failed: Color::Rgb(204, 36, 29),     // red
            status_running: Color::Rgb(7, 102, 120),    // blue
            status_pending: Color::Rgb(181, 118, 20),   // yellow/orange
            status_draft: Color::Rgb(124, 111, 100),    // gray
            status_merged: Color::Rgb(143, 63, 113),    // purple
            label_bg: Color::Rgb(213, 196, 161),        // light2
            label_fg: Color::Rgb(40, 40, 40),           // dark0
            badge_bg: Color::Rgb(213, 196, 161),        // light2
            badge_fg: Color::Rgb(7, 102, 120),          // blue
            chip_bg: Color::Rgb(213, 196, 161),         // light2
            chip_fg: Color::Rgb(121, 116, 14),          // green
            line_number_fg: Color::Rgb(124, 111, 100),  // gray
            hunk_header_bg: Color::Rgb(213, 196, 161),  // light2
            hunk_header_fg: Color::Rgb(7, 102, 120),    // blue
            diff_add_bg: Color::Rgb(213, 196, 161),     // light green/light2 tint
            diff_add_fg: Color::Rgb(121, 116, 14),      // green
            diff_del_bg: Color::Rgb(235, 219, 178),     // light1 tint
            diff_del_fg: Color::Rgb(204, 36, 29),       // red
        }
    }

    pub fn github_dark() -> Self {
        Self {
            border: Color::Rgb(46, 160, 67),            // green
            highlight_bg: Color::Rgb(48, 54, 61),       // selection
            author: Color::Rgb(163, 113, 247),          // purple
            dim: Color::Rgb(139, 148, 158),             // muted
            success: Color::Rgb(63, 185, 80),           // green
            failed: Color::Rgb(248, 81, 73),            // red
            running: Color::Rgb(88, 166, 255),          // blue
            pending: Color::Rgb(210, 153, 34),          // yellow
            open_mr: Color::Rgb(88, 166, 255),          // blue
            merged_mr: Color::Rgb(163, 113, 247),       // purple
            closed_mr: Color::Rgb(248, 81, 73),         // red
            header: Color::Rgb(88, 166, 255),           // blue
            header1: Color::Rgb(57, 197, 187),          // cyan
            header2: Color::Rgb(163, 113, 247),         // purple
            project: Color::Rgb(88, 166, 255),          // blue
            issue: Color::Rgb(63, 185, 80),             // green
            mr: Color::Rgb(163, 113, 247),              // purple
            shortcut_key: Color::Rgb(88, 166, 255),     // blue
            shortcut_desc: Color::Rgb(240, 246, 252),   // fg
            keyword: Color::Rgb(248, 81, 73),           // red
            code: Color::Rgb(63, 185, 80),              // green
            code_bg: Color::Rgb(22, 27, 34),            // dark
            range_highlight: Color::Rgb(48, 54, 61),    // sel
            active_border: Color::Rgb(46, 160, 67),     // green
            inactive_border: Color::Rgb(110, 118, 129), // muted
            title_fg: Color::Rgb(240, 246, 252),        // fg
            accent: Color::Rgb(88, 166, 255),           // blue
            status_success: Color::Rgb(63, 185, 80),    // green
            status_failed: Color::Rgb(248, 81, 73),     // red
            status_running: Color::Rgb(88, 166, 255),   // blue
            status_pending: Color::Rgb(210, 153, 34),   // yellow
            status_draft: Color::Rgb(139, 148, 158),    // muted
            status_merged: Color::Rgb(163, 113, 247),   // purple
            label_bg: Color::Rgb(48, 54, 61),           // selection
            label_fg: Color::Rgb(240, 246, 252),        // fg
            badge_bg: Color::Rgb(48, 54, 61),           // selection
            badge_fg: Color::Rgb(88, 166, 255),         // blue
            chip_bg: Color::Rgb(48, 54, 61),            // selection
            chip_fg: Color::Rgb(63, 185, 80),           // green
            line_number_fg: Color::Rgb(139, 148, 158),  // muted
            hunk_header_bg: Color::Rgb(48, 54, 61),     // selection
            hunk_header_fg: Color::Rgb(88, 166, 255),   // blue
            diff_add_bg: Color::Rgb(20, 50, 30),        // dark green tint
            diff_add_fg: Color::Rgb(63, 185, 80),       // green
            diff_del_bg: Color::Rgb(55, 25, 30),        // dark red tint
            diff_del_fg: Color::Rgb(248, 81, 73),       // red
        }
    }

    pub fn github_light() -> Self {
        Self {
            border: Color::Rgb(26, 127, 55),            // green
            highlight_bg: Color::Rgb(221, 244, 255),    // light blue selection
            author: Color::Rgb(130, 80, 223),           // purple
            dim: Color::Rgb(101, 109, 118),             // muted
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
            active_border: Color::Rgb(26, 127, 55),     // green
            inactive_border: Color::Rgb(208, 215, 222), // muted
            title_fg: Color::Rgb(31, 35, 40),           // dark fg
            accent: Color::Rgb(9, 105, 218),            // blue
            status_success: Color::Rgb(26, 127, 55),    // green
            status_failed: Color::Rgb(207, 34, 46),     // red
            status_running: Color::Rgb(9, 105, 218),    // blue
            status_pending: Color::Rgb(154, 103, 0),    // yellow
            status_draft: Color::Rgb(101, 109, 118),    // muted
            status_merged: Color::Rgb(130, 80, 223),    // purple
            label_bg: Color::Rgb(234, 238, 242),        // light gray
            label_fg: Color::Rgb(31, 35, 40),           // dark fg
            badge_bg: Color::Rgb(221, 244, 255),        // light blue
            badge_fg: Color::Rgb(9, 105, 218),          // blue
            chip_bg: Color::Rgb(221, 244, 255),         // light blue
            chip_fg: Color::Rgb(26, 127, 55),           // green
            line_number_fg: Color::Rgb(101, 109, 118),  // muted
            hunk_header_bg: Color::Rgb(221, 244, 255),  // light blue
            hunk_header_fg: Color::Rgb(9, 105, 218),    // blue
            diff_add_bg: Color::Rgb(218, 251, 225),     // soft green tint
            diff_add_fg: Color::Rgb(26, 127, 55),       // green
            diff_del_bg: Color::Rgb(255, 235, 233),     // soft red tint
            diff_del_fg: Color::Rgb(207, 34, 46),       // red
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
            "gruvboxlight" => Some(Self::gruvbox_light()),
            "githubdark" => Some(Self::github_dark()),
            "githublight" => Some(Self::github_light()),
            _ => None,
        }
    }

    pub fn load() -> Self {
        // Search locations: local theme.toml, ~/.config/lazygitlab/theme.toml, or config.toml
        let mut candidates = Vec::new();
        candidates.push(std::path::PathBuf::from("theme.toml"));

        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            let config_dir = std::path::PathBuf::from(&home)
                .join(".config")
                .join("lazygitlab");
            candidates.push(config_dir.join("theme.toml"));
            candidates.push(config_dir.join("config.toml"));
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            let config_dir = std::path::PathBuf::from(appdata).join("lazygitlab");
            candidates.push(config_dir.join("theme.toml"));
            candidates.push(config_dir.join("config.toml"));
        }

        for path in candidates {
            if let Ok(content) = std::fs::read_to_string(&path)
                && let Some(t) = Self::parse_theme_content(&content)
            {
                return t;
            }
        }

        Self::default()
    }

    pub fn parse_theme_content(content: &str) -> Option<Self> {
        #[derive(Deserialize)]
        struct ThemeConfigFile {
            theme: Option<String>,
            #[serde(flatten)]
            custom: Option<PartialTheme>,
        }

        #[derive(Deserialize)]
        struct AppConfigWithTheme {
            theme: Option<String>,
        }

        if let Ok(cfg) = toml::from_str::<ThemeConfigFile>(content) {
            let mut base = cfg
                .theme
                .as_deref()
                .and_then(Self::from_name)
                .unwrap_or_default();
            if let Some(custom) = cfg.custom {
                custom.apply_to(&mut base);
            }
            return Some(base);
        }

        if let Ok(cfg) = toml::from_str::<AppConfigWithTheme>(content)
            && let Some(preset) = cfg.theme.as_deref().and_then(Self::from_name)
        {
            return Some(preset);
        }

        None
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PartialTheme {
    // 8 Core Base Colors
    pub accent: Option<Color>,
    pub border: Option<Color>,
    pub title_fg: Option<Color>,
    pub dim: Option<Color>,
    pub highlight_bg: Option<Color>,
    pub code_bg: Option<Color>,
    pub success: Option<Color>,
    pub failed: Option<Color>,

    // Secondary Base Colors
    pub running: Option<Color>,
    pub pending: Option<Color>,
    pub author: Option<Color>,
    pub project: Option<Color>,
    pub issue: Option<Color>,
    pub mr: Option<Color>,
    pub header: Option<Color>,
    pub header1: Option<Color>,
    pub header2: Option<Color>,
    pub shortcut_key: Option<Color>,
    pub shortcut_desc: Option<Color>,
    pub keyword: Option<Color>,
    pub code: Option<Color>,
    pub range_highlight: Option<Color>,
    pub active_border: Option<Color>,
    pub inactive_border: Option<Color>,
    pub status_success: Option<Color>,
    pub status_failed: Option<Color>,
    pub status_running: Option<Color>,
    pub status_pending: Option<Color>,
    pub status_draft: Option<Color>,
    pub status_merged: Option<Color>,
    pub open_mr: Option<Color>,
    pub merged_mr: Option<Color>,
    pub closed_mr: Option<Color>,
    pub label_bg: Option<Color>,
    pub label_fg: Option<Color>,
    pub badge_bg: Option<Color>,
    pub badge_fg: Option<Color>,
    pub chip_bg: Option<Color>,
    pub chip_fg: Option<Color>,
    pub line_number_fg: Option<Color>,
    pub hunk_header_bg: Option<Color>,
    pub hunk_header_fg: Option<Color>,
    pub diff_add_bg: Option<Color>,
    pub diff_add_fg: Option<Color>,
    pub diff_del_bg: Option<Color>,
    pub diff_del_fg: Option<Color>,
}

impl PartialTheme {
    pub fn apply_to(&self, base: &mut Theme) {
        if let Some(c) = self.border {
            base.border = c;
            base.active_border = c;
        }
        if let Some(c) = self.active_border {
            base.active_border = c;
        }
        if let Some(c) = self.inactive_border {
            base.inactive_border = c;
        }
        if let Some(c) = self.title_fg {
            base.title_fg = c;
            base.shortcut_desc = c;
            base.label_fg = c;
        }
        if let Some(c) = self.dim {
            base.dim = c;
            base.inactive_border = c;
            base.line_number_fg = c;
            base.status_draft = c;
        }
        if let Some(c) = self.accent {
            base.accent = c;
            base.header = c;
            base.running = c;
            base.status_running = c;
            base.open_mr = c;
            base.shortcut_key = c;
            base.badge_fg = c;
            base.hunk_header_fg = c;
        }
        if let Some(c) = self.highlight_bg {
            base.highlight_bg = c;
            base.range_highlight = c;
            base.label_bg = c;
            base.badge_bg = c;
            base.chip_bg = c;
            base.hunk_header_bg = c;
        }
        if let Some(c) = self.code_bg {
            base.code_bg = c;
        }
        if let Some(c) = self.success {
            base.success = c;
            base.status_success = c;
            base.issue = c;
            base.code = c;
            base.chip_fg = c;
            base.diff_add_fg = c;
        }
        if let Some(c) = self.failed {
            base.failed = c;
            base.status_failed = c;
            base.closed_mr = c;
            base.keyword = c;
            base.diff_del_fg = c;
        }
        if let Some(c) = self.running {
            base.running = c;
            base.status_running = c;
        }
        if let Some(c) = self.pending {
            base.pending = c;
            base.status_pending = c;
        }
        if let Some(c) = self.author {
            base.author = c;
        }
        if let Some(c) = self.project {
            base.project = c;
        }
        if let Some(c) = self.issue {
            base.issue = c;
        }
        if let Some(c) = self.mr {
            base.mr = c;
        }
        if let Some(c) = self.header {
            base.header = c;
        }
        if let Some(c) = self.header1 {
            base.header1 = c;
        }
        if let Some(c) = self.header2 {
            base.header2 = c;
        }
        if let Some(c) = self.shortcut_key {
            base.shortcut_key = c;
        }
        if let Some(c) = self.shortcut_desc {
            base.shortcut_desc = c;
        }
        if let Some(c) = self.keyword {
            base.keyword = c;
        }
        if let Some(c) = self.code {
            base.code = c;
        }
        if let Some(c) = self.range_highlight {
            base.range_highlight = c;
        }
        if let Some(c) = self.status_success {
            base.status_success = c;
        }
        if let Some(c) = self.status_failed {
            base.status_failed = c;
        }
        if let Some(c) = self.status_running {
            base.status_running = c;
        }
        if let Some(c) = self.status_pending {
            base.status_pending = c;
        }
        if let Some(c) = self.status_draft {
            base.status_draft = c;
        }
        if let Some(c) = self.status_merged {
            base.status_merged = c;
        }
        if let Some(c) = self.open_mr {
            base.open_mr = c;
        }
        if let Some(c) = self.merged_mr {
            base.merged_mr = c;
        }
        if let Some(c) = self.closed_mr {
            base.closed_mr = c;
        }
        if let Some(c) = self.label_bg {
            base.label_bg = c;
        }
        if let Some(c) = self.label_fg {
            base.label_fg = c;
        }
        if let Some(c) = self.badge_bg {
            base.badge_bg = c;
        }
        if let Some(c) = self.badge_fg {
            base.badge_fg = c;
        }
        if let Some(c) = self.chip_bg {
            base.chip_bg = c;
        }
        if let Some(c) = self.chip_fg {
            base.chip_fg = c;
        }
        if let Some(c) = self.line_number_fg {
            base.line_number_fg = c;
        }
        if let Some(c) = self.hunk_header_bg {
            base.hunk_header_bg = c;
        }
        if let Some(c) = self.hunk_header_fg {
            base.hunk_header_fg = c;
        }
        if let Some(c) = self.diff_add_bg {
            base.diff_add_bg = c;
        }
        if let Some(c) = self.diff_add_fg {
            base.diff_add_fg = c;
        }
        if let Some(c) = self.diff_del_bg {
            base.diff_del_bg = c;
        }
        if let Some(c) = self.diff_del_fg {
            base.diff_del_fg = c;
        }
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
    let (r, g, b) = if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0) * 17;
        let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0) * 17;
        let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0) * 17;
        (r, g, b)
    } else {
        return Color::Rgb(205, 214, 244); // #cdd6f4
    };
    let luminance = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;
    if luminance > 0.5 {
        Color::Rgb(17, 17, 27) // #11111b (dark text)
    } else {
        Color::Rgb(205, 214, 244) // #cdd6f4 (light text)
    }
}

pub fn get_contrast_color(color: Color) -> Color {
    let luminance = match color {
        Color::Rgb(r, g, b) => (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0,
        Color::Black => 0.0,
        Color::White => 1.0,
        Color::Green => 0.587,
        Color::Blue => 0.114,
        Color::Cyan => 0.701,
        Color::Magenta => 0.413,
        Color::Yellow => 0.886,
        Color::Red => 0.299,
        Color::Gray => 0.5,
        Color::DarkGray => 0.25,
        Color::LightRed => 0.5,
        Color::LightGreen => 0.75,
        Color::LightYellow => 0.95,
        Color::LightBlue => 0.4,
        Color::LightMagenta => 0.6,
        Color::LightCyan => 0.85,
        _ => 0.0,
    };
    if luminance > 0.5 {
        Color::Rgb(17, 17, 27) // #11111b
    } else {
        Color::Rgb(205, 214, 244) // #cdd6f4
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
        "success" | "passed" => (
            "✔".to_string(),
            Style::default().fg(theme.status_success).bold(),
        ),
        "failed" | "error" => (
            "✖".to_string(),
            Style::default().fg(theme.status_failed).bold(),
        ),
        "running" | "in_progress" => (
            "⟳".to_string(),
            Style::default().fg(theme.status_running).bold(),
        ),
        "pending" | "waiting" => (
            "⏸".to_string(),
            Style::default().fg(theme.status_pending).bold(),
        ),
        "draft" => (
            "⭘".to_string(),
            Style::default().fg(theme.status_draft).bold(),
        ),
        "merged" => (
            "✦".to_string(),
            Style::default().fg(theme.status_merged).bold(),
        ),
        _ => (status.to_string(), Style::default().fg(theme.dim)),
    }
}

pub fn render_pill<'a>(label: &str, fg: Color, bg: Color) -> Span<'a> {
    Span::styled(format!(" {} ", label), Style::default().fg(fg).bg(bg))
}

pub fn render_progress_bar<'a>(
    completed: u64,
    total: u64,
    width: usize,
    theme: &Theme,
) -> Line<'a> {
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
    Span::styled(
        format!(" {} ", key),
        Style::default()
            .fg(theme.badge_fg)
            .bg(theme.badge_bg)
            .bold(),
    )
}

pub fn render_milestone_chip<'a>(milestone: &str, theme: &Theme) -> Span<'a> {
    Span::styled(
        format!(" ⚑ {} ", milestone),
        Style::default().fg(theme.chip_fg).bg(theme.chip_bg),
    )
}

pub fn render_label_badge<'a>(label: &str, bg_hex: &str) -> Span<'a> {
    let bg_color = hex_to_color(bg_hex);
    let fg_color = get_contrasting_text_color(bg_hex);
    Span::styled(
        format!(" {} ", label),
        Style::default().fg(fg_color).bg(bg_color),
    )
}

pub fn render_breadcrumbs<'a>(crumbs: &[&str], theme: &Theme) -> Line<'a> {
    let mut spans = Vec::new();
    for (i, crumb) in crumbs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" ❯ ", Style::default().fg(theme.dim)));
        }
        spans.push(Span::styled(
            crumb.to_string(),
            Style::default().fg(theme.title_fg).bold(),
        ));
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
    fn test_get_contrasting_text_color() {
        // Bright background -> Dark text (#11111b)
        assert_eq!(
            get_contrasting_text_color("#ffffff"),
            Color::Rgb(17, 17, 27)
        );
        assert_eq!(
            get_contrasting_text_color("#a6e3a1"),
            Color::Rgb(17, 17, 27)
        );
        assert_eq!(
            get_contrasting_text_color("#ffff00"),
            Color::Rgb(17, 17, 27)
        );
        assert_eq!(get_contrasting_text_color("#fff"), Color::Rgb(17, 17, 27));

        // Dark background -> Light text (#cdd6f4)
        assert_eq!(
            get_contrasting_text_color("#000000"),
            Color::Rgb(205, 214, 244)
        );
        assert_eq!(
            get_contrasting_text_color("#1e1e2e"),
            Color::Rgb(205, 214, 244)
        );
        assert_eq!(
            get_contrasting_text_color("#1e66f5"),
            Color::Rgb(205, 214, 244)
        );
        assert_eq!(
            get_contrasting_text_color("#000"),
            Color::Rgb(205, 214, 244)
        );
        assert_eq!(
            get_contrasting_text_color("invalid"),
            Color::Rgb(205, 214, 244)
        );
    }

    #[test]
    fn test_get_contrast_color() {
        assert_eq!(get_contrast_color(Color::White), Color::Rgb(17, 17, 27));
        assert_eq!(get_contrast_color(Color::Yellow), Color::Rgb(17, 17, 27));
        assert_eq!(get_contrast_color(Color::Black), Color::Rgb(205, 214, 244));
        assert_eq!(get_contrast_color(Color::Blue), Color::Rgb(205, 214, 244));
        assert_eq!(
            get_contrast_color(Color::Rgb(255, 255, 255)),
            Color::Rgb(17, 17, 27)
        );
        assert_eq!(
            get_contrast_color(Color::Rgb(20, 20, 20)),
            Color::Rgb(205, 214, 244)
        );
    }

    #[test]
    fn test_all_theme_presets_loaded() {
        let presets = [
            "catppuccin-mocha",
            "catppuccin-latte",
            "tokyo-night",
            "nord",
            "dracula",
            "gruvbox-dark",
            "gruvbox-light",
            "github-dark",
            "github-light",
            "default",
        ];
        for preset in presets {
            assert!(
                Theme::from_name(preset).is_some(),
                "Preset '{}' should be recognized",
                preset
            );
        }
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
