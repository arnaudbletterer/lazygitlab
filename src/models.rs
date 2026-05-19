use serde::{Deserialize, Serialize};
use ratatui::widgets::ListState;
use crate::ui::Theme;
use gitlab::AsyncGitlab;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum TodoFilter {
    Pending,
    Snoozed,
    Done,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum IssueFilter {
    Opened,
    Closed,
    All,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum MergeRequestFilter {
    Opened,
    Merged,
    Closed,
    All,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Namespace {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub full_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: Option<String>,
    pub web_url: Option<String>,
    pub default_branch: Option<String>,
    pub namespace: Option<Namespace>,
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
    #[serde(default)]
    pub description: Option<String>,
    pub author: Author,
    #[serde(default)]
    pub web_url: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub draft: Option<bool>,
    #[serde(default)]
    pub references: Option<References>,
    #[serde(default)]
    pub upvotes: Option<u64>,
    #[serde(default)]
    pub downvotes: Option<u64>,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub milestone: Option<Milestone>,
    #[serde(default)]
    pub assignees: Vec<Author>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub diff_refs: Option<DiffRefs>,
    #[serde(skip)]
    pub reactions: Vec<AwardEmoji>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiffRefs {
    pub base_sha: String,
    pub head_sha: String,
    pub start_sha: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Discussion {
    pub id: String,
    pub notes: Vec<Note>,
    #[serde(default)]
    pub individual_note: bool,
    #[serde(skip)]
    pub collapsed: bool,
    pub resolved: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MRDiff {
    pub old_path: String,
    pub new_path: String,
    pub diff: String,
    #[serde(default)]
    pub collapsed: bool,
    #[serde(skip)]
    pub display_lines: Vec<DiffDisplayLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffDisplayLine {
    Code(String, Option<u64>, Option<u64>),
    Discussion(Discussion),
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
    #[serde(default)]
    pub system: bool,
    pub position: Option<Position>,
    #[serde(skip)]
    pub context: Option<Vec<(u64, String, LineType, bool)>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Position {
    pub new_path: Option<String>,
    pub old_path: Option<String>,
    pub new_line: Option<u64>,
    pub old_line: Option<u64>,
    pub line_range: Option<LineRange>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LineRange {
    pub start: LinePosition,
    pub end: LinePosition,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LinePosition {
    pub line_code: Option<String>,
    pub r#type: Option<String>,
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
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub web_url: Option<String>,
}

impl Author {
    pub fn new(id: u64, username: String) -> Self {
        Self {
            id,
            username,
            name: None,
            state: None,
            avatar_url: None,
            web_url: None,
        }
    }
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Group {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub full_path: String,
    pub web_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Branch {
    pub name: String,
    pub merged: Option<bool>,
    pub protected: Option<bool>,
    pub default: Option<bool>,
    pub web_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Commit {
    pub id: String,
    pub short_id: String,
    pub title: String,
    pub author_name: String,
    pub created_at: String,
    pub message: String,
    pub web_url: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum MenuSelection {
    Todos,
    MergeRequests,
    Issues,
    Pipelines,
    Projects,
    Groups,
    Repository,
    Branches,
    Commits,
    Boards,
    Epics,
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
    Tree,
    Board,
    Epics,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepositoryTreeItem {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub path: String,
    pub mode: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepositoryFile {
    pub file_name: String,
    pub file_path: String,
    pub size: u64,
    pub encoding: String,
    pub content_sha256: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub blob_id: String,
    pub commit_id: String,
    pub last_commit_id: String,
    pub content: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Label {
    pub id: u64,
    pub name: String,
    pub color: String,
    pub text_color: Option<String>,
}

impl<'de> serde::Deserialize<'de> for Label {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LabelVisitor;
        impl<'de> serde::de::Visitor<'de> for LabelVisitor {
            type Value = Label;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or a label object")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error {
                Ok(Label { id: 0, name: v.to_string(), color: "#808080".to_string(), text_color: None })
            }
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error> where M: serde::de::MapAccess<'de> {
                let mut id = 0;
                let mut name = String::new();
                let mut color = "#808080".to_string();
                let mut text_color = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => id = map.next_value()?,
                        "name" => name = map.next_value()?,
                        "color" => color = map.next_value()?,
                        "text_color" => text_color = Some(map.next_value()?),
                        _ => { let _: serde::de::IgnoredAny = map.next_value()?; }
                    }
                }
                Ok(Label { id, name, color, text_color })
            }
        }
        deserializer.deserialize_any(LabelVisitor)
    }
}

impl Label {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn color(&self) -> &str {
        &self.color
    }

    pub fn text_color(&self) -> &str {
        self.text_color.as_deref().unwrap_or("#ffffff")
    }
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
    #[serde(default)]
    pub description: Option<String>,
    pub author: Author,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub assignees: Vec<Author>,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub web_url: String,
    #[serde(default)]
    pub references: Option<References>,
    #[serde(default)]
    pub issue_type: Option<String>,
    #[serde(default)]
    pub milestone: Option<Milestone>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub weight: Option<i64>,
    #[serde(default)]
    pub task_completion_status: Option<TaskCompletionStatus>,
    #[serde(default)]
    pub confidential: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub upvotes: Option<u64>,
    #[serde(default)]
    pub downvotes: Option<u64>,
    #[serde(default)]
    pub reactions: Vec<AwardEmoji>,
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
    SnoozeUntil,
    Search,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Context {
    Dashboard,
    Explore,
    Project,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DetailSection {
    Metadata,
    Discussions,
    Description,
    Changes,
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
    Search,
    Refresh,
    OpenBrowser,
    CopyUrl,
    CopyContent,
    LaunchPipeline,
    CancelPipeline,
    RetryPipeline,
    PlayJob,
    RetryJob,
    CancelJob,
    ToggleJobSection,
    NewComment,
    Reply,
    ResolveDiscussion,
    ToggleIssueState,
    ToggleCollapse,
    ToggleDraft,
    Approve,
    Unapprove,
    MarkDone,
    Snooze,
    CreateIssue,
    CreateMR,
    MoveBoardLeft,
    MoveBoardRight,
    NextDiscussion,
    PrevDiscussion,
    EditField,
    GotoTodos,
    GotoMRs,
    GotoIssues,
    GotoPipelines,
    GotoRepository,
    Panel1,
    Panel2,
    Panel3,
    ExpandCollapsedDiff,
    FetchAllMissingDiffs,
    None,
}

#[derive(Debug, Clone)]
pub struct HelpItem {
    pub shortcut: String,
    pub description: String,
    pub action: HelpAction,
    pub category: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Panel {
    Dashboard,
    Explore,
    Project,
    Main,
    MainDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupedItem<T> {
    Header(String),
    Item(T),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Board {
    pub id: u64,
    pub name: String,
    pub lists: Option<Vec<BoardList>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BoardList {
    pub id: u64,
    pub label: Option<Label>,
    pub position: Option<u32>,
    #[serde(default)]
    pub list_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Epic {
    pub id: u64,
    pub iid: u64,
    pub group_id: Option<u64>,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub web_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchResult {
    Project(Project),
    Issue(Issue),
    MergeRequest(MergeRequest),
}

impl SearchResult {
    pub fn key(&self) -> String {
        match self {
            SearchResult::Project(p) => format!("p:{}", p.id),
            SearchResult::Issue(i) => format!("i:{}:{}", i.project_id, i.iid),
            SearchResult::MergeRequest(m) => format!("m:{}:{}", m.project_id, m.iid),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectVariable {
    pub key: String,
    pub value: String,
    pub variable_type: String,
    pub protected: bool,
    pub masked: bool,
    pub environment_scope: String,
}

#[derive(Debug)]
pub enum DataUpdate {
    Todos { data: Vec<Todo>, page: u64, has_more: bool },
    AppendTodos(Vec<Todo>),
    Projects { data: Vec<Project>, page: u64, has_more: bool },
    AppendProjects(Vec<Project>),
    Groups { data: Vec<Group>, page: u64, has_more: bool },
    AppendGroups(Vec<Group>),
    GlobalMRs { data: Vec<MergeRequest>, page: u64, has_more: bool },
    ClearGlobalMRs,
    AppendGlobalMRs(Vec<MergeRequest>),
    GlobalIssues { data: Vec<Issue>, page: u64, has_more: bool },
    ClearGlobalIssues,
    AppendGlobalIssues(Vec<Issue>),
    ProjectMRs { project_id: u64, data: Vec<MergeRequest>, page: u64, has_more: bool },
    AppendProjectMRs { project_id: u64, data: Vec<MergeRequest> },
    ProjectIssues { project_id: u64, data: Vec<Issue>, page: u64, has_more: bool },
    AppendProjectIssues { project_id: u64, data: Vec<Issue> },
    ProjectPipelines { project_id: u64, data: Vec<Pipeline>, page: u64, has_more: bool },
    AppendProjectPipelines { project_id: u64, data: Vec<Pipeline> },
    Discussions { project_id: u64, iid: u64, data: Vec<Discussion> },
    IssueDiscussions { project_id: u64, iid: u64, data: Vec<Discussion> },
    Jobs { project_id: u64, pipeline_id: u64, data: Vec<Job> },
    ProjectVariables { project_id: u64, data: Vec<ProjectVariable> },
    Log { project_id: u64, job_id: u64, data: String },
    TerminalEscape(String),
    Diffs { project_id: u64, iid: u64, data: Vec<MRDiff> },
    SingleDiffFetched { project_id: u64, iid: u64, old_path: String, new_path: String, diff: String },
    Metadata { project_id: u64, labels: Vec<Label>, milestones: Vec<Milestone>, members: Vec<Author> },
    MemberProjectIds(std::collections::HashSet<u64>),
    MRDetail(MergeRequest),
    IssueDetail(Issue),
    CurrentUser(Author),
    RepositoryTree { project_id: u64, data: Vec<RepositoryTreeItem> },
    RepositoryFile { project_id: u64, file_path: String, data: RepositoryFile },
    Branches { project_id: u64, data: Vec<Branch>, page: u64, has_more: bool },
    Commits { project_id: u64, data: Vec<Commit>, page: u64, has_more: bool },
    AppendCommits { project_id: u64, data: Vec<Commit> },
    Boards { project_id: u64, data: Vec<Board> },
    BoardLists { project_id: u64, board_id: u64, data: Vec<BoardList> },
    BoardIssues { project_id: u64, board_id: u64, list_id: u64, data: Vec<Issue> },
    Epics { group_id: u64, data: Vec<Epic> },
    SearchResults { data: Vec<SearchResult>, search_id: u64 },
    AppendSearchResults { data: Vec<SearchResult>, search_id: u64 },
    TriggerSearch(u64),
    Loading(bool),
    Error(String),
    TriggerRefresh,
    CheckPagination,
}

pub struct App {
    pub should_quit: bool,
    pub active_context: Context,
    pub dashboard_menu_state: ListState,
    pub explore_menu_state: ListState,
    pub project_menu_state: ListState,
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
    pub active_panel: Panel,
    pub view_mode: ViewMode,
    pub detail_section: DetailSection,
    pub selected_metadata_field: MetadataField,
    pub selected_mr: Option<MergeRequest>,
    pub selected_issue: Option<Issue>,
    pub selected_pipeline: Option<Pipeline>,
    pub selected_job: Option<Job>,
    pub mr_diffs: Vec<MRDiff>,
    pub mr_diff_state: ListState,
    pub diff_line_state: ListState,
    pub diff_selection_start: Option<usize>,
    pub job_log: String,
    pub job_log_lines: Vec<LogLine>,
    pub collapsed_sections: std::collections::HashSet<String>,
    pub job_log_scroll: u16,
    pub job_log_cursor: usize,
    pub job_log_selection_start: Option<usize>,
    pub job_log_viewport_height: u16,
    pub follow_log: bool,
    pub discussions: Vec<Discussion>,
    pub discussion_state: ListState,
    pub input_mode: bool,
    pub input_target: InputTarget,
    pub is_loading: bool,
    pub textarea: ratatui_textarea::TextArea<'static>,
    pub theme: Theme,
    pub last_refresh: std::time::Instant,
    pub tx: tokio::sync::mpsc::UnboundedSender<DataUpdate>,

    // Project Metadata
    pub project_labels: Vec<Label>,
    pub project_cache: std::collections::HashMap<u64, Project>,
    pub project_label_cache: std::collections::HashMap<u64, Vec<Label>>,
    pub project_milestone_cache: std::collections::HashMap<u64, Vec<Milestone>>,
    pub project_member_cache: std::collections::HashMap<u64, Vec<Author>>,
    pub project_variable_cache: std::collections::HashMap<u64, Vec<ProjectVariable>>,
    pub project_milestones: Vec<Milestone>,
    pub project_members: Vec<Author>,
    pub project_variables: Vec<ProjectVariable>,
    pub collapsed_groups: std::collections::HashSet<String>,

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
    pub filtered_branches: Vec<Branch>,
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
    pub filtered_todos: Vec<GroupedItem<Todo>>,
    pub todos_page: u64,
    pub todos_has_more: bool,
    pub filtered_mrs: Vec<GroupedItem<MergeRequest>>,
    pub filtered_issues: Vec<GroupedItem<Issue>>,
    pub filtered_pipelines: Vec<Pipeline>,
    
    pub todo_filter: TodoFilter,
    pub issue_filter: IssueFilter,
    pub mr_filter: MergeRequestFilter,

    pub last_error: Option<String>,
    pub status_message: Option<String>,
    pub loading_tasks: usize,

    // Cache timestamps
    pub last_fetch_todos: Option<std::time::Instant>,
    pub last_fetch_mrs: Option<std::time::Instant>,
    pub last_fetch_issues: Option<std::time::Instant>,
    pub last_fetch_projects: Option<std::time::Instant>,
    pub last_fetch_groups: Option<std::time::Instant>,
    pub last_fetch_pipelines: Option<std::time::Instant>,
    pub show_snooze_popup: bool,
    pub snooze_state: ListState,
    pub current_user: Option<Author>,

    // Repository
    pub repository_tree: Vec<RepositoryTreeItem>,
    pub repository_tree_state: tui_tree_widget::TreeState<String>,
    pub selected_file_content: Option<RepositoryFile>,

    // History
    pub commits: Vec<Commit>,
    pub commits_state: ListState,
    pub commits_page: u64,
    pub commits_has_more: bool,

    // Kanban Boards & Epics
    pub project_boards: Vec<Board>,
    pub project_boards_state: ListState,
    pub selected_board: Option<Board>,
    pub board_lists: Vec<BoardList>,
    pub board_lists_state: ListState,
    pub board_issues: std::collections::HashMap<u64, Vec<Issue>>,
    pub filtered_board_issues: std::collections::HashMap<u64, Vec<Issue>>,
    pub board_issue_states: std::collections::HashMap<u64, ListState>,
    pub epics: Vec<Epic>,
    pub epics_state: ListState,

    // Search
    pub show_search: bool,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub search_state: ListState,
    pub search_id: u64,
    pub member_project_ids: std::collections::HashSet<u64>,
    pub is_syncing_membership: bool,
    pub last_input: std::time::Instant,

    pub syntax_set: syntect::parsing::SyntaxSet,
    pub theme_set: syntect::highlighting::ThemeSet,
    pub pending_g_prefix: bool,

    pub last_rendered_rects: std::collections::HashMap<String, ratatui::layout::Rect>,
}
