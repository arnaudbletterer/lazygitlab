use serde::Deserialize;
use ratatui::widgets::ListState;
use crate::ui::Theme;
use gitlab::AsyncGitlab;

#[derive(Debug, Deserialize, Clone)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MergeRequest {
    pub iid: u64,
    pub project_id: u64,
    pub title: String,
    pub description: Option<String>,
    pub author: Author,
    pub web_url: String,
    pub state: String,
    pub references: References,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Discussion {
    pub id: String,
    pub notes: Vec<Note>,
    #[serde(skip)]
    pub collapsed: bool,
    pub resolved: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MRDiff {
    pub old_path: String,
    pub new_path: String,
    pub diff: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineType {
    Addition,
    Removal,
    Unchanged,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Note {
    pub body: String,
    pub author: Author,
    #[serde(rename = "type")]
    pub note_type: Option<String>,
    pub position: Option<Position>,
    #[serde(skip)]
    pub context: Option<Vec<(u64, String, LineType, bool)>>, // (line_number, content, type, is_target)
    pub resolvable: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Position {
    pub new_path: Option<String>,
    pub old_path: Option<String>,
    pub new_line: Option<u64>,
    pub old_line: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct References {
    pub full: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Author {
    pub username: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MenuSelection {
    Projects,
    MergeRequests,
    Issues,
    Pipelines,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    ProjectSelection,
    List,
    Detail,
}

pub struct App {
    pub should_quit: bool,
    pub menu_state: ListState,
    pub menu_items: Vec<(String, MenuSelection)>,
    pub active_selection: MenuSelection,
    pub gitlab: AsyncGitlab,
    pub projects: Vec<Project>,
    pub projects_state: ListState,
    pub selected_project: Option<Project>,
    pub merge_requests: Vec<MergeRequest>,
    pub mrs_state: ListState,
    pub focus_on_menu: bool,
    pub view_mode: ViewMode,
    pub selected_mr: Option<MergeRequest>,
    pub discussions: Vec<Discussion>,
    pub discussion_state: ListState,
    pub input_mode: bool,
    pub reply_input: String,
    pub theme: Theme,
}
