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
    pub system: bool,
    pub position: Option<Position>,
    #[serde(skip)]
    pub context: Option<Vec<(u64, String, LineType, bool)>>, // (line_number, content, type, is_target)
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
    MergeRequests,
    Issues,
    Pipelines,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    List,
    Detail,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Label {
    pub name: String,
    pub color: String,
    pub text_color: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Milestone {
    pub title: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TaskCompletionStatus {
    pub count: u64,
    pub completed_count: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Issue {
    pub iid: u64,
    pub project_id: u64,
    pub title: String,
    pub description: Option<String>,
    pub author: Author,
    pub assignees: Vec<Author>,
    pub state: String,
    pub web_url: String,
    pub issue_type: Option<String>,
    pub labels: Vec<Label>,
    pub milestone: Option<Milestone>,
    pub due_date: Option<String>,
    pub weight: Option<i64>,
    pub task_completion_status: Option<TaskCompletionStatus>,
    pub confidential: bool,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputTarget {
    Reply,
    NewComment,
    EditTitle,
    EditDescription,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DetailSection {
    Metadata,
    Discussions,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MetadataField {
    Title,
    Description,
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
    pub show_project_popup: bool,
    pub merge_requests: Vec<MergeRequest>,
    pub mrs_state: ListState,
    pub issues: Vec<Issue>,
    pub issues_state: ListState,
    pub focus_on_menu: bool,
    pub view_mode: ViewMode,
    pub detail_section: DetailSection,
    pub selected_metadata_field: MetadataField,
    pub selected_mr: Option<MergeRequest>,
    pub selected_issue: Option<Issue>,
    pub discussions: Vec<Discussion>,
    pub discussion_state: ListState,
    pub input_mode: bool,
    pub input_target: InputTarget,
    pub is_loading: bool,
    pub reply_input: String,
    pub cursor_index: usize,
    pub theme: Theme,
}
