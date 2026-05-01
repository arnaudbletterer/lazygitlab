use serde::{Deserialize, Serialize};
use ratatui::widgets::ListState;
use crate::ui::Theme;
use gitlab::AsyncGitlab;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
    pub web_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AwardEmoji {
    #[serde(alias = "emoji", alias = "emoji_name")]
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MergeRequest {
    pub iid: u64,
    pub project_id: u64,
    pub title: String,
    pub description: Option<String>,
    pub author: Author,
    pub web_url: String,
    pub state: String,
    pub references: References,
    pub upvotes: u64,
    pub downvotes: u64,
    pub labels: Vec<Label>,
    pub milestone: Option<Milestone>,
    pub assignees: Vec<Author>,
    pub created_at: String,
    #[serde(skip)]
    pub reactions: Vec<AwardEmoji>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Discussion {
    pub id: String,
    pub notes: Vec<Note>,
    #[serde(skip)]
    pub collapsed: bool,
    pub resolved: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MRDiff {
    pub old_path: String,
    pub new_path: String,
    pub diff: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum LineType {
    Addition,
    Removal,
    Unchanged,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Note {
    pub body: String,
    pub author: Author,
    pub system: bool,
    pub position: Option<Position>,
    #[serde(skip)]
    pub context: Option<Vec<(u64, String, LineType, bool)>>, // (line_number, content, type, is_target)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Position {
    pub new_path: Option<String>,
    pub old_path: Option<String>,
    pub new_line: Option<u64>,
    pub old_line: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct References {
    pub full: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Author {
    pub id: u64,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Group {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub full_path: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum MenuSelection {
    Todos,
    MergeRequests,
    Issues,
    Pipelines,
    Projects,
    Groups,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Todo {
    pub id: u64,
    pub project: Project,
    pub action_name: String,
    pub target_type: String,
    pub target_url: String,
    pub body: String,
    pub state: String,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    List,
    Detail,
    JobLog,
    Diff,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Label {
    pub id: u64,
    pub name: String,
    pub color: String,
    pub text_color: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Milestone {
    pub id: u64,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TaskCompletionStatus {
    pub count: u64,
    pub completed_count: u64,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PipelineVariable {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pipeline {
    pub id: u64,
    pub project_id: u64,
    pub status: String,
    pub source: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub web_url: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Runner {
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Job {
    pub id: u64,
    pub name: String,
    pub stage: String,
    pub status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration: Option<f64>,
    pub user: Option<Author>,
    pub runner: Option<Runner>,
    pub allow_failure: bool,
    pub web_url: String,
    pub tag_list: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Issue {
    pub iid: u64,
    pub project_id: u64,
    pub title: String,
    pub description: Option<String>,
    pub author: Author,
    pub assignees: Vec<Author>,
    pub state: String,
    pub web_url: String,
    pub references: References,
    pub issue_type: Option<String>,
    pub labels: Vec<Label>,
    pub milestone: Option<Milestone>,
    pub due_date: Option<String>,
    pub weight: Option<i64>,
    pub task_completion_status: Option<TaskCompletionStatus>,
    pub confidential: bool,
    pub created_at: String,
    pub upvotes: u64,
    pub downvotes: u64,
    #[serde(skip)]
    pub reactions: Vec<AwardEmoji>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Branch {
    pub name: String,
}


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputTarget {
    Reply,
    NewComment,
    EditTitle,
    EditDescription,
    PipelineRef,
    PipelineRefSearch,
    PipelineVariableKey,
    PipelineVariableValue,
    PipelineVariableList,
    Filter,
    CreateIssueTitle,
    CreateIssueDescription,
    CreateMRTitle,
    CreateMRDescription,
    CreateMRSourceBranch,
    CreateMRTargetBranch,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DetailSection {
    Metadata,
    Discussions,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MetadataField {
    Title,
    Assignees,
    Labels,
    Milestone,
    DueDate,
    Weight,
    Confidential,
    Description,
}

#[derive(Debug, Clone)]
pub struct LogLine {
    pub timestamp: String,
    pub content: String,
    pub section: Option<String>,
    pub is_section_start: bool,
    pub is_section_end: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpAction {
    Quit,
    SwitchProject,
    Back,
    Select,
    ToggleHelp,
    ToggleFocus,
    Filter,
    OpenBrowser,
    LaunchPipeline,
    CancelPipeline,
    PlayJob,
    RetryJob,
    CancelJob,
    NewComment,
    ResolveDiscussion,
    ToggleIssueState,
    ToggleCollapse,
    EditField,
    None,
}

#[derive(Debug, Clone)]
pub struct HelpItem {
    pub shortcut: String,
    pub description: String,
    pub action: HelpAction,
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
    pub pipelines: Vec<Pipeline>,
    pub pipelines_state: ListState,
    pub jobs: Vec<Job>,
    pub jobs_state: ListState,
    pub focus_on_menu: bool,
    pub view_mode: ViewMode,
    pub detail_section: DetailSection,
    pub selected_metadata_field: MetadataField,
    pub selected_mr: Option<MergeRequest>,
    pub selected_issue: Option<Issue>,
    pub selected_pipeline: Option<Pipeline>,
    pub selected_job: Option<Job>,
    pub mr_diffs: Vec<MRDiff>,
    pub mr_diff_state: ListState,
    pub job_log: String,
    pub job_log_lines: Vec<LogLine>,
    pub collapsed_sections: std::collections::HashSet<String>,
    pub job_log_scroll: u16,
    pub job_log_viewport_height: u16,
    pub follow_log: bool,
    pub discussions: Vec<Discussion>,
    pub discussion_state: ListState,
    pub input_mode: bool,
    pub input_target: InputTarget,
    pub is_loading: bool,
    pub reply_input: String,
    pub cursor_index: usize,
    pub theme: Theme,
    pub last_refresh: std::time::Instant,

    // Project Metadata
    pub project_labels: Vec<Label>,
    pub project_milestones: Vec<Milestone>,
    pub project_members: Vec<Author>,

    // Popups
    pub show_label_popup: bool,
    pub label_state: ListState,
    pub selected_label_ids: Vec<u64>,

    pub show_assignee_popup: bool,
    pub assignee_state: ListState,
    pub selected_assignee_ids: Vec<u64>,

    pub show_milestone_popup: bool,
    pub milestone_state: ListState,
    pub selected_milestone_id: Option<u64>,

    pub show_weight_popup: bool,
    pub weight_state: ListState,

    pub show_calendar_popup: bool,
    pub calendar_date: chrono::NaiveDate,

    pub show_pipeline_popup: bool,
    pub pipeline_ref_input: String,
    pub pipeline_variable_key_input: String,
    pub pipeline_variable_value_input: String,
    pub pipeline_variables: Vec<PipelineVariable>,
    pub pipeline_variables_state: ListState,

    pub project_branches: Vec<Branch>,
    pub filtered_branches: Vec<String>,
    pub branches_state: ListState,
    pub show_branches_dropdown: bool,

    // Pagination
    pub mrs_page: u64,
    pub mrs_has_more: bool,
    pub issues_page: u64,
    pub issues_has_more: bool,
    pub pipelines_page: u64,
    pub pipelines_has_more: bool,

    pub show_help_popup: bool,
    pub help_state: ListState,
    pub help_filter_input: String,
    pub show_help_filter: bool,
    pub filtered_help: Vec<HelpItem>,

    pub show_create_issue_popup: bool,
    pub create_issue_title: String,
    pub create_issue_description: String,

    pub show_create_mr_popup: bool,
    pub create_mr_title: String,
    pub create_mr_description: String,
    pub create_mr_source_branch: String,
    pub create_mr_target_branch: String,
    pub create_mr_source_branch_state: ListState,
    pub create_mr_target_branch_state: ListState,

    pub projects_page: u64,
    pub projects_has_more: bool,
    pub filtered_projects: Vec<Project>,

    pub groups: Vec<Group>,
    pub groups_state: ListState,
    pub filtered_groups: Vec<Group>,
    pub groups_page: u64,
    pub groups_has_more: bool,

    pub filter_input: String,
    pub show_filter_input: bool,
    pub todos: Vec<Todo>,
    pub todos_state: ListState,
    pub filtered_todos: Vec<Todo>,
    pub todos_page: u64,
    pub todos_has_more: bool,
    pub filtered_mrs: Vec<MergeRequest>,
    pub filtered_issues: Vec<Issue>,
    pub filtered_pipelines: Vec<Pipeline>,
}
