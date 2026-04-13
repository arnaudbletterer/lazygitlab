pub mod markdown;
pub mod widgets;
pub mod list;
pub mod detail;
pub mod popup;

pub use widgets::Theme;
use ratatui::prelude::*;
use crate::models::{App, ViewMode};
use list::render_list_view;
use detail::render_detail_view;
use popup::*;

pub fn ui(f: &mut Frame, app: &mut App) {
    match app.view_mode {
        ViewMode::List => {
            render_list_view(f, app);
        }
        ViewMode::Detail => {
            render_detail_view(f, app);
        }
    }

    if app.show_project_popup {
        render_project_popup(f, app);
    }
    if app.show_label_popup {
        render_label_popup(f, app);
    }
    if app.show_assignee_popup {
        render_assignee_popup(f, app);
    }
    if app.show_milestone_popup {
        render_milestone_popup(f, app);
    }
    if app.show_calendar_popup {
        render_calendar_popup(f, app);
    }
    if app.show_weight_popup {
        render_weight_popup(f, app);
    }
}
