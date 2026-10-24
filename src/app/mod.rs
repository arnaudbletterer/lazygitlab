pub mod boards;
pub mod global;
pub mod issues;
pub mod keymap;
pub mod menus;
pub mod mrs;
pub mod pipelines;
pub mod popups;
pub mod search;

use crate::api::*;
use crate::models::*;
use crate::ui::Theme;
use chrono::Local;
use gitlab::{AsyncGitlab, api::AsyncQuery};
use ratatui::widgets::ListState;
use regex::Regex;

static TODO_TARGET_RE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"/(?:issues|merge_requests)/(\d+)").unwrap());

impl App {
    pub fn new(
        gitlab: AsyncGitlab,
        tx: tokio::sync::mpsc::UnboundedSender<DataUpdate>,
        target_project: Option<String>,
    ) -> Self {
        let mut dashboard_menu_state = ListState::default();
        dashboard_menu_state.select(Some(0));
        let mut explore_menu_state = ListState::default();
        explore_menu_state.select(Some(0));
        let mut project_menu_state = ListState::default();
        project_menu_state.select(Some(0));

        Self {
            target_project,
            local_git_status: detect_local_git_status(),
            skip_tls_verify: false,
            should_quit: false,
            active_context: Context::Dashboard,
            dashboard_menu_state,
            explore_menu_state,
            project_menu_state,
            active_selection: MenuSelection::Home,
            home_card_index: 0,
            gitlab,
            projects: Vec::new(),
            projects_state: ListState::default(),
            selected_project: None,
            show_project_popup: false,
            merge_requests: Vec::new(),
            mrs_state: ListState::default(),
            issues: Vec::new(),
            issues_state: ListState::default(),
            pipelines: Vec::new(),
            cross_pipelines: Vec::new(),
            cross_pipelines_page: 1,
            cross_pipelines_has_more: false,
            pipelines_state: ListState::default(),
            jobs: Vec::new(),
            jobs_state: ListState::default(),
            active_panel: Panel::Dashboard,
            view_mode: ViewMode::List,
            detail_section: DetailSection::Metadata,
            selected_metadata_field: MetadataField::Title,
            selected_mr: None,
            selected_issue: None,
            selected_pipeline: None,
            selected_job: None,
            mr_diffs: Vec::new(),
            mr_diff_state: ListState::default(),
            diff_line_state: ListState::default(),
            diff_selection_start: None,
            diff_side_by_side: false,
            job_log: String::new(),
            job_log_lines: Vec::new(),
            collapsed_sections: std::collections::HashSet::new(),
            job_log_scroll: 0,
            job_log_cursor: 0,
            job_log_selection_start: None,
            job_log_viewport_height: 0,
            job_log_search: String::new(),
            follow_log: true,
            discussions: Vec::new(),
            discussion_state: ListState::default(),
            input_mode: false,
            input_target: InputTarget::Reply,
            is_loading: false,
            is_fetching_diffs: false,
            textarea: ratatui_textarea::TextArea::default(),
            theme: Theme::load(),
            last_refresh: std::time::Instant::now(),
            tx,
            project_labels: Vec::new(),
            project_cache: std::collections::HashMap::new(),
            project_label_cache: std::collections::HashMap::new(),
            project_milestone_cache: std::collections::HashMap::new(),
            project_member_cache: std::collections::HashMap::new(),
            project_metadata_fetches_in_progress: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            project_variable_cache: std::collections::HashMap::new(),
            project_milestones: Vec::new(),
            project_members: Vec::new(),
            project_variables: Vec::new(),
            show_schedules_popup: false,
            pipeline_schedules: Vec::new(),
            schedules_state: ListState::default(),
            show_ci_vars_popup: false,
            ci_vars_state: ListState::default(),
            ci_var_editing: None,
            approvals: None,
            collapsed_groups: std::collections::HashSet::new(),
            show_label_popup: false,
            label_state: ListState::default(),
            selected_label_ids: Vec::new(),
            show_assignee_popup: false,
            assignee_state: ListState::default(),
            selected_assignee_ids: Vec::new(),
            show_reviewer_popup: false,
            reviewer_state: ListState::default(),
            selected_reviewer_ids: Vec::new(),
            show_milestone_popup: false,
            milestone_state: ListState::default(),
            selected_milestone_id: None,
            show_weight_popup: false,
            weight_state: ListState::default(),
            show_calendar_popup: false,
            calendar_date: Local::now().date_naive(),
            show_pipeline_popup: false,
            pipeline_ref_input: String::new(),
            pipeline_variable_key_input: String::new(),
            pipeline_variable_value_input: String::new(),
            pipeline_variables: Vec::new(),
            pipeline_variables_state: ListState::default(),
            project_branches: Vec::new(),
            filtered_branches: Vec::new(),
            branches_state: ListState::default(),
            show_branches_dropdown: false,
            show_target_branch_popup: false,
            target_branch_state: ListState::default(),
            show_create_branch_popup: false,
            create_branch_name: String::new(),
            create_branch_ref_input: String::new(),
            create_branch_state: ListState::default(),
            show_create_branch_ref_dropdown: false,
            mrs_page: 1,
            mrs_has_more: true,
            issues_page: 1,
            issues_has_more: true,
            pipelines_page: 1,
            pipelines_has_more: true,
            show_confirm_merge_popup: false,
            merge_squash: false,
            merge_remove_source_branch: true,
            show_confirm_rebase_popup: false,
            show_confirm_delete_branch_popup: false,
            pending_delete_branch: None,
            show_help_popup: false,
            help_state: ListState::default(),
            help_filter_input: String::new(),
            show_help_filter: false,
            filtered_help: Vec::new(),
            show_create_issue_popup: false,
            create_issue_title: String::new(),
            create_issue_description: String::new(),
            show_create_mr_popup: false,
            create_mr_title: String::new(),
            create_mr_description: String::new(),
            create_mr_source_branch: String::new(),
            create_mr_target_branch: String::new(),
            create_mr_source_branch_state: ListState::default(),
            create_mr_target_branch_state: ListState::default(),
            projects_page: 1,
            projects_has_more: true,
            filtered_projects: Vec::new(),
            groups: Vec::new(),
            groups_state: ListState::default(),
            filtered_groups: Vec::new(),
            groups_page: 1,
            groups_has_more: true,
            filter_input: String::new(),
            previous_filter_input: String::new(),
            show_filter_input: false,
            todos: Vec::new(),
            todos_state: ListState::default(),
            filtered_todos: Vec::new(),
            todos_page: 1,
            todos_has_more: true,
            todos_etag: None,
            filtered_mrs: Vec::new(),
            filtered_issues: Vec::new(),
            filtered_pipelines: Vec::new(),
            todo_filter: TodoFilter::Pending,
            issue_filter: IssueFilter::Opened,
            issue_group_by_milestone: false,
            mr_filter: MergeRequestFilter::Opened,
            pipeline_filter: PipelineFilter::All,
            last_error: None,
            status_message: None,
            last_status_text: None,
            status_since: std::time::Instant::now(),
            last_error_since: None,
            loading_tasks: 0,
            diff_fetch_semaphore: std::sync::Arc::new(tokio::sync::Semaphore::new(10)),
            last_fetch_todos: None,
            last_fetch_mrs: None,
            last_fetch_issues: None,
            last_fetch_projects: None,
            last_fetch_groups: None,
            last_fetch_pipelines: None,
            show_snooze_popup: false,
            snooze_state: ListState::default(),
            current_user: None,
            show_involved_only: false,
            repository_tree: Vec::new(),
            repository_tree_state: tui_tree_widget::TreeState::default(),
            selected_file_content: None,
            repository_ref: None,
            show_tree_branch_popup: false,
            tree_branch_state: ListState::default(),
            related_mrs: Vec::new(),
            mr_closing_issues: Vec::new(),
            linked_issues: Vec::new(),
            issue_closing_mrs: Vec::new(),
            file_scroll: 0,
            description_scroll: 0,
            file_viewport_height: 0,
            file_highlight_cache: None,
            job_log_epoch: 0,
            job_log_display_cache: (0, 0),
            markdown_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
            commits: Vec::new(),
            commits_state: ListState::default(),
            commits_page: 1,
            commits_has_more: true,
            project_boards: Vec::new(),
            project_boards_state: ListState::default(),
            selected_board: None,
            board_lists: Vec::new(),
            board_lists_state: ListState::default(),
            board_issues: std::collections::HashMap::new(),
            filtered_board_issues: std::collections::HashMap::new(),
            board_issue_states: std::collections::HashMap::new(),
            epics: Vec::new(),
            epics_state: ListState::default(),
            selected_epic: None,
            epic_issues: Vec::new(),
            filtered_epic_issues: Vec::new(),
            epic_issues_state: ListState::default(),
            show_search: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_state: ListState::default(),
            search_id: 0,
            member_project_ids: std::collections::HashSet::new(),
            is_syncing_membership: false,
            membership_sync_attempted: false,
            last_input: std::time::Instant::now(),
            syntax_set: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            theme_set: syntect::highlighting::ThemeSet::load_defaults(),
            pending_g_prefix: false,
            last_detail_fetch: None,
            last_rendered_rects: std::collections::HashMap::new(),
        }
    }

    pub async fn process_update_single(&mut self, update: DataUpdate) {
        match update {
            DataUpdate::TriggerSearch(search_id) => {
                if search_id == self.search_id {
                    let _ = self.trigger_global_search().await;
                }
            }
            DataUpdate::MemberProjectIds(ids) => {
                self.member_project_ids = ids;
                self.is_syncing_membership = false;
                self.membership_sync_attempted = true;
                self.filter_lists();
                if self.active_context == Context::Dashboard {
                    let should_fetch =
                        self.view_mode == ViewMode::List && self.active_panel != Panel::MainDetail;
                    if should_fetch {
                        match self.active_selection {
                            MenuSelection::Issues => {
                                let _ = self.fetch_global_issues().await;
                            }
                            MenuSelection::MergeRequests => {
                                let _ = self.fetch_global_merge_requests().await;
                            }
                            _ => {}
                        }
                    }
                }
            }
            DataUpdate::SearchResults { data, search_id } => {
                if search_id == self.search_id {
                    self.search_results = data;
                    self.search_state.select(if self.search_results.is_empty() {
                        None
                    } else {
                        Some(0)
                    });
                }
            }
            DataUpdate::AppendSearchResults { data, search_id } => {
                if search_id == self.search_id {
                    let mut existing_keys: std::collections::HashSet<String> =
                        self.search_results.iter().map(|r| r.key()).collect();
                    for item in data {
                        let key = item.key();
                        if !existing_keys.contains(&key) {
                            existing_keys.insert(key);
                            self.search_results.push(item);
                        }
                    }
                    if self.search_state.selected().is_none() && !self.search_results.is_empty() {
                        self.search_state.select(Some(0));
                    }
                }
            }
            DataUpdate::Projects {
                data,
                page,
                has_more,
            } => {
                for p in &data {
                    self.project_cache.insert(p.id, p.clone());
                }
                if self.projects != data
                    || self.projects_page != page
                    || self.projects_has_more != has_more
                {
                    self.projects = data;
                    self.projects_page = page;
                    self.projects_has_more = has_more;
                    self.filter_lists();
                }
            }
            DataUpdate::AppendProjects(mut data) => {
                for p in &data {
                    self.project_cache.insert(p.id, p.clone());
                }
                if !data.is_empty() {
                    self.projects_has_more = data.len() >= 50;
                    self.projects.append(&mut data);
                    self.projects_page += 1;
                    self.filter_lists();
                }
            }

            DataUpdate::Todos {
                data,
                page,
                has_more,
            } => {
                for t in &data {
                    if let Some(ref p) = t.project {
                        self.project_cache.insert(p.id, p.clone());
                    }
                }
                if self.todos != data || self.todos_page != page || self.todos_has_more != has_more
                {
                    self.todos = data;
                    self.todos_page = page;
                    self.todos_has_more = has_more;
                    self.filter_lists();
                }
            }
            DataUpdate::AppendTodos(mut data) => {
                for t in &data {
                    if let Some(ref p) = t.project {
                        self.project_cache.insert(p.id, p.clone());
                    }
                }
                if !data.is_empty() {
                    self.todos_has_more = data.len() >= 50;
                    self.todos.append(&mut data);
                    self.todos_page += 1;
                    self.filter_lists();
                }
            }

            DataUpdate::Groups {
                data,
                page,
                has_more,
            } => {
                if self.groups != data
                    || self.groups_page != page
                    || self.groups_has_more != has_more
                {
                    self.groups = data;
                    self.groups_page = page;
                    self.groups_has_more = has_more;
                    self.filter_lists();
                }
            }
            DataUpdate::AppendGroups(mut data) => {
                if !data.is_empty() {
                    self.groups_has_more = data.len() >= 50;
                    self.groups.append(&mut data);
                    self.groups_page += 1;
                    self.filter_lists();
                }
            }

            DataUpdate::GlobalMRs {
                data,
                page,
                has_more,
            } => {
                let mut data = data;
                for mr in &mut data {
                    self.fix_mr_labels(mr);
                }
                if self.active_context == Context::Dashboard
                    && (self.merge_requests != data
                        || self.mrs_page != page
                        || self.mrs_has_more != has_more)
                {
                    self.merge_requests = data;
                    self.mrs_page = page;
                    self.mrs_has_more = has_more;
                    self.filter_lists();
                }
            }
            DataUpdate::ProjectMRs {
                project_id,
                mut data,
                page,
                has_more,
            } => {
                for mr in &mut data {
                    self.fix_mr_labels(mr);
                }
                if self.active_context == Context::Project
                    && self
                        .selected_project
                        .as_ref()
                        .is_some_and(|p| p.id == project_id)
                    && (self.merge_requests != data
                        || self.mrs_page != page
                        || self.mrs_has_more != has_more)
                {
                    self.merge_requests = data;
                    self.mrs_page = page;
                    self.mrs_has_more = has_more;
                    self.filter_lists();
                }
            }
            DataUpdate::AppendGlobalMRs { mut data, has_more } => {
                for mr in &mut data {
                    self.fix_mr_labels(mr);
                }
                if self.active_context == Context::Dashboard {
                    let existing_keys: std::collections::HashSet<String> = self
                        .merge_requests
                        .iter()
                        .map(|m| format!("{}:{}", m.project_id, m.iid))
                        .collect();
                    let mut added = false;
                    for item in data {
                        let key = format!("{}:{}", item.project_id, item.iid);
                        if !existing_keys.contains(&key) {
                            self.merge_requests.push(item);
                            added = true;
                        }
                    }
                    self.mrs_has_more = has_more;
                    if added {
                        self.merge_requests
                            .sort_by_key(|b| std::cmp::Reverse(b.iid));
                        self.filter_lists();
                    }
                }
            }
            DataUpdate::AppendProjectMRs {
                project_id,
                mut data,
            } => {
                for mr in &mut data {
                    self.fix_mr_labels(mr);
                }
                if self.active_context == Context::Project
                    && self
                        .selected_project
                        .as_ref()
                        .is_some_and(|p| p.id == project_id)
                    && !data.is_empty()
                {
                    self.mrs_has_more = data.len() >= 50;
                    self.merge_requests.append(&mut data);
                    self.mrs_page += 1;
                    self.filter_lists();
                }
            }
            DataUpdate::ClearGlobalMRs => {
                if self.active_context == Context::Dashboard && !self.merge_requests.is_empty() {
                    self.merge_requests.clear();
                    self.filter_lists();
                }
            }

            DataUpdate::GlobalIssues {
                data,
                page,
                has_more,
            } => {
                let mut data = data;
                for issue in &mut data {
                    self.fix_issue_labels(issue);
                }
                if self.active_context == Context::Dashboard
                    && (self.issues != data
                        || self.issues_page != page
                        || self.issues_has_more != has_more)
                {
                    self.issues = data;
                    self.issues_page = page;
                    self.issues_has_more = has_more;
                    self.filter_lists();
                }
            }
            DataUpdate::ProjectIssues {
                project_id,
                mut data,
                page,
                has_more,
            } => {
                for issue in &mut data {
                    self.fix_issue_labels(issue);
                }
                if self.active_context == Context::Project
                    && self
                        .selected_project
                        .as_ref()
                        .is_some_and(|p| p.id == project_id)
                    && (self.issues != data
                        || self.issues_page != page
                        || self.issues_has_more != has_more)
                {
                    self.issues = data;
                    self.issues_page = page;
                    self.issues_has_more = has_more;
                    self.filter_lists();
                }
            }
            DataUpdate::AppendGlobalIssues { mut data, has_more } => {
                for issue in &mut data {
                    self.fix_issue_labels(issue);
                }
                if self.active_context == Context::Dashboard {
                    let existing_keys: std::collections::HashSet<String> = self
                        .issues
                        .iter()
                        .map(|i| format!("{}:{}", i.project_id, i.iid))
                        .collect();
                    let mut added = false;
                    for item in data {
                        let key = format!("{}:{}", item.project_id, item.iid);
                        if !existing_keys.contains(&key) {
                            self.issues.push(item);
                            added = true;
                        }
                    }
                    self.issues_has_more = has_more;
                    if added {
                        self.issues.sort_by_key(|b| std::cmp::Reverse(b.iid));
                        self.filter_lists();
                    }
                }
            }
            DataUpdate::AppendProjectIssues {
                project_id,
                mut data,
            } => {
                for issue in &mut data {
                    self.fix_issue_labels(issue);
                }
                if self.active_context == Context::Project
                    && self
                        .selected_project
                        .as_ref()
                        .is_some_and(|p| p.id == project_id)
                    && !data.is_empty()
                {
                    self.issues_has_more = data.len() >= 50;
                    self.issues.append(&mut data);
                    self.issues_page += 1;
                    self.filter_lists();
                }
            }
            DataUpdate::ClearGlobalIssues => {
                if self.active_context == Context::Dashboard && !self.issues.is_empty() {
                    self.issues.clear();
                    self.filter_lists();
                }
            }

            DataUpdate::ProjectPipelines {
                project_id,
                data,
                page,
                has_more,
            } => {
                if self.active_context == Context::Project
                    && self
                        .selected_project
                        .as_ref()
                        .is_some_and(|p| p.id == project_id)
                    && (self.pipelines != data
                        || self.pipelines_page != page
                        || self.pipelines_has_more != has_more)
                {
                    self.pipelines = data;
                    self.pipelines_page = page;
                    self.pipelines_has_more = has_more;
                    self.filter_lists();
                }
            }
            DataUpdate::GlobalPipelines {
                data,
                page,
                has_more,
            } => {
                if self.active_context != Context::Project
                    && (self.cross_pipelines != data
                        || self.cross_pipelines_page != page
                        || self.cross_pipelines_has_more != has_more)
                {
                    self.cross_pipelines = data;
                    self.cross_pipelines_page = page;
                    self.cross_pipelines_has_more = has_more;
                    self.filter_lists();
                }
            }
            DataUpdate::PipelineSchedules { project_id, data } => {
                if self
                    .selected_project
                    .as_ref()
                    .is_some_and(|p| p.id == project_id)
                {
                    self.pipeline_schedules = data;
                    self.schedules_state
                        .select(if self.pipeline_schedules.is_empty() {
                            None
                        } else {
                            Some(0)
                        });
                }
            }
            DataUpdate::StatusMessage(msg) => {
                self.status_message = Some(msg);
            }
            DataUpdate::ApprovalInfo {
                project_id,
                iid,
                info,
            } => {
                if self
                    .selected_mr
                    .as_ref()
                    .is_some_and(|m| m.project_id == project_id && m.iid == iid)
                {
                    self.approvals = Some(info);
                }
            }
            DataUpdate::VariableDeleted { key } => {
                self.project_variables.retain(|v| v.key != key);
                if let Some(pid) = self.selected_project.as_ref().map(|p| p.id) {
                    self.project_variable_cache
                        .insert(pid, self.project_variables.clone());
                }
                self.status_message = Some(format!("Variable '{}' deleted", key));
            }
            DataUpdate::VariableSaved { key, value } => {
                if let Some(v) = self.project_variables.iter_mut().find(|v| v.key == key) {
                    v.value = value;
                } else {
                    self.project_variables.push(ProjectVariable {
                        key,
                        value,
                        variable_type: "env_var".into(),
                        protected: false,
                        masked: false,
                        environment_scope: "*".into(),
                    });
                }
                if let Some(pid) = self.selected_project.as_ref().map(|p| p.id) {
                    self.project_variable_cache
                        .insert(pid, self.project_variables.clone());
                }
                self.status_message = Some("Variable saved".into());
            }
            DataUpdate::AppendProjectPipelines {
                project_id,
                mut data,
            } => {
                if self.active_context == Context::Project
                    && self
                        .selected_project
                        .as_ref()
                        .is_some_and(|p| p.id == project_id)
                    && !data.is_empty()
                {
                    self.pipelines_has_more = data.len() >= 50;
                    self.pipelines.append(&mut data);
                    self.pipelines_page += 1;
                    self.filter_lists();
                }
            }

            DataUpdate::Discussions {
                project_id,
                iid,
                mut data,
            } => {
                if let Some(selected) = &self.selected_mr
                    && selected.iid == iid
                    && selected.project_id == project_id
                {
                    // Preserve collapsed state OR default to collapsed for new ones
                    for new_disc in &mut data {
                        if let Some(old_disc) =
                            self.discussions.iter().find(|d| d.id == new_disc.id)
                        {
                            new_disc.collapsed = old_disc.collapsed;
                        } else {
                            new_disc.collapsed = true;
                        }
                    }
                    self.discussions = data;
                    self.enrich_diffs_with_discussions();
                }
            }
            DataUpdate::IssueDiscussions {
                project_id,
                iid,
                mut data,
            } => {
                if let Some(selected) = &self.selected_issue
                    && selected.iid == iid
                    && selected.project_id == project_id
                {
                    // Preserve collapsed state OR default to collapsed for new ones
                    for new_disc in &mut data {
                        if let Some(old_disc) =
                            self.discussions.iter().find(|d| d.id == new_disc.id)
                        {
                            new_disc.collapsed = old_disc.collapsed;
                        } else {
                            new_disc.collapsed = true;
                        }
                    }
                    self.discussions = data;
                }
            }
            DataUpdate::Jobs {
                project_id,
                pipeline_id,
                data,
            } => {
                if let Some(selected) = &self.selected_pipeline
                    && selected.id == pipeline_id
                    && selected.project_id == project_id
                {
                    self.jobs = data.clone();
                    if let Some(curr_job) = &self.selected_job
                        && let Some(updated_job) = data.iter().find(|j| j.id == curr_job.id)
                    {
                        self.selected_job = Some(updated_job.clone());
                    }
                }
            }
            DataUpdate::ProjectVariables { project_id, data } => {
                self.project_variable_cache.insert(project_id, data.clone());
                if self
                    .selected_project
                    .as_ref()
                    .is_some_and(|p| p.id == project_id)
                {
                    self.project_variables = data;
                }
            }
            DataUpdate::Log {
                project_id,
                job_id,
                data,
            } => {
                if self.selected_job.as_ref().is_some_and(|j| j.id == job_id)
                    && self
                        .selected_project
                        .as_ref()
                        .is_some_and(|p| p.id == project_id)
                {
                    self.job_log = data;
                    self.parse_job_log();
                    self.bump_job_log_epoch();
                }
            }
            DataUpdate::SingleDiffFetched {
                project_id,
                iid,
                old_path,
                new_path,
                diff,
            } => {
                if let Some(selected) = &self.selected_mr
                    && selected.iid == iid
                    && selected.project_id == project_id
                {
                    for mr_diff in &mut self.mr_diffs {
                        if mr_diff.old_path == old_path && mr_diff.new_path == new_path {
                            mr_diff.diff = diff.clone();
                            mr_diff.collapsed = false;
                            break;
                        }
                    }
                    self.enrich_diffs_with_discussions();
                }
            }
            DataUpdate::Diffs {
                project_id,
                iid,
                data,
            } => {
                self.is_fetching_diffs = false;
                if let Some(selected) = &self.selected_mr
                    && selected.iid == iid
                    && selected.project_id == project_id
                {
                    self.mr_diffs = data;
                    self.enrich_diffs_with_discussions();
                    let _ = self.fetch_all_missing_diffs().await;
                }
            }
            DataUpdate::Metadata {
                project_id,
                labels,
                milestones,
                members,
            } => {
                self.project_label_cache.insert(project_id, labels.clone());
                self.project_milestone_cache
                    .insert(project_id, milestones.clone());
                self.project_member_cache
                    .insert(project_id, members.clone());

                let active_pid = self
                    .selected_mr
                    .as_ref()
                    .map(|m| m.project_id)
                    .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                    .or_else(|| self.selected_project.as_ref().map(|p| p.id));
                if active_pid.is_some_and(|pid| pid == project_id) {
                    self.project_labels = labels.clone();
                    self.project_milestones = milestones.clone();
                    self.project_members = members.clone();
                }

                let labels_ref = &labels;
                for mr in &mut self.merge_requests {
                    if mr.project_id == project_id {
                        Self::fix_label_list(labels_ref, &mut mr.labels);
                    }
                }
                for issue in &mut self.issues {
                    if issue.project_id == project_id {
                        Self::fix_label_list(labels_ref, &mut issue.labels);
                    }
                }
                if let Some(selected) = &mut self.selected_mr
                    && selected.project_id == project_id
                {
                    Self::fix_label_list(labels_ref, &mut selected.labels);
                }
                if let Some(selected) = &mut self.selected_issue
                    && selected.project_id == project_id
                {
                    Self::fix_label_list(labels_ref, &mut selected.labels);
                }
                self.filter_lists();
            }
            DataUpdate::MRDetail(mut mr) => {
                self.fix_mr_labels(&mut mr);
                if let Some(selected) = &mut self.selected_mr {
                    if selected.iid == mr.iid && selected.project_id == mr.project_id {
                        for label in &mut mr.labels {
                            if (label.id == 0 || label.color == "#808080")
                                && let Some(old) =
                                    selected.labels.iter().find(|l| l.name == label.name)
                                && old.color != "#808080"
                            {
                                label.id = old.id;
                                label.color = old.color.clone();
                                label.text_color = old.text_color.clone();
                            }
                        }
                        if !self.input_mode {
                            *selected = mr.clone();
                        }
                    }
                } else {
                    self.selected_mr = Some(mr.clone());
                }
                if let Some(existing) = self
                    .merge_requests
                    .iter_mut()
                    .find(|x| x.iid == mr.iid && x.project_id == mr.project_id)
                {
                    *existing = mr;
                    self.filter_lists();
                }
            }
            DataUpdate::IssueDetail(mut issue) => {
                self.fix_issue_labels(&mut issue);
                if let Some(selected) = &mut self.selected_issue {
                    if selected.iid == issue.iid && selected.project_id == issue.project_id {
                        for label in &mut issue.labels {
                            if (label.id == 0 || label.color == "#808080")
                                && let Some(old) =
                                    selected.labels.iter().find(|l| l.name == label.name)
                                && old.color != "#808080"
                            {
                                label.id = old.id;
                                label.color = old.color.clone();
                                label.text_color = old.text_color.clone();
                            }
                        }
                        if !self.input_mode {
                            *selected = issue;
                        }
                    }
                } else {
                    self.selected_issue = Some(issue);
                }
            }
            DataUpdate::Loading(b) => {
                if b {
                    self.loading_tasks += 1;
                } else {
                    self.loading_tasks = self.loading_tasks.saturating_sub(1);
                }
                self.is_loading = self.loading_tasks > 0;
                if !self.is_loading
                    && let Some(msg) = &self.status_message
                {
                    if msg.starts_with("Searching") {
                        if self.search_results.is_empty() && self.show_search {
                            self.status_message = Some("No results found.".into());
                        } else {
                            self.status_message = None;
                        }
                    } else if msg.starts_with("Fetching")
                        || msg.starts_with("Loading")
                        || msg.starts_with("Updating")
                    {
                        self.status_message = None;
                    }
                }
            }
            DataUpdate::Error(e) => {
                let e = crate::errors::friendly_error(&crate::redaction::redact_secrets(&e));
                if e.contains("Diffs") {
                    self.is_fetching_diffs = false;
                }
                self.last_error = Some(e);
                self.last_error_since = Some(std::time::Instant::now());
                self.loading_tasks = self.loading_tasks.saturating_sub(1);
                self.is_loading = self.loading_tasks > 0;
                if !self.is_loading
                    && let Some(msg) = &self.status_message
                    && (msg.starts_with("Fetching")
                        || msg.starts_with("Loading")
                        || msg.starts_with("Updating")
                        || msg.starts_with("Searching"))
                {
                    self.status_message = None;
                }
            }
            DataUpdate::TriggerRefresh => {
                self.refresh().await;
            }
            DataUpdate::RemovePendingIssue { iid } => {
                self.issues.retain(|i| i.iid != iid);
                self.filter_lists();
            }
            DataUpdate::RemovePendingMR { iid } => {
                self.merge_requests.retain(|m| m.iid != iid);
                self.filter_lists();
            }
            DataUpdate::CheckPagination => {
                let _ = self.check_pagination().await;
            }
            DataUpdate::CurrentUser(user) => {
                self.current_user = Some(user);
            }
            DataUpdate::RepositoryTree { project_id, data } => {
                if self
                    .selected_project
                    .as_ref()
                    .is_some_and(|p| p.id == project_id)
                {
                    self.repository_tree = data;
                    self.update_tree_state();
                }
            }
            DataUpdate::RepositoryFile {
                project_id,
                file_path,
                data,
            } => {
                if self
                    .selected_project
                    .as_ref()
                    .is_some_and(|p| p.id == project_id)
                    && self
                        .repository_tree_state
                        .selected()
                        .last()
                        .is_some_and(|selected_path| selected_path == &file_path)
                {
                    self.file_highlight_cache = None;
                    self.file_scroll = 0;
                    self.selected_file_content = Some(data);
                }
            }
            DataUpdate::RelatedMRs {
                project_id,
                iid,
                data,
            } => {
                if self
                    .selected_mr
                    .as_ref()
                    .is_some_and(|m| m.project_id == project_id && m.iid == iid)
                    && self.related_mrs != data
                {
                    self.related_mrs = data;
                }
            }
            DataUpdate::MRClosesIssues {
                project_id,
                iid,
                data,
            } => {
                if self
                    .selected_mr
                    .as_ref()
                    .is_some_and(|m| m.project_id == project_id && m.iid == iid)
                    && self.mr_closing_issues != data
                {
                    self.mr_closing_issues = data;
                }
            }
            DataUpdate::IssueLinks {
                project_id,
                iid,
                data,
            } => {
                if self
                    .selected_issue
                    .as_ref()
                    .is_some_and(|i| i.project_id == project_id && i.iid == iid)
                    && self.linked_issues != data
                {
                    self.linked_issues = data;
                }
            }
            DataUpdate::IssueClosedBy {
                project_id,
                iid,
                data,
            } => {
                if self
                    .selected_issue
                    .as_ref()
                    .is_some_and(|i| i.project_id == project_id && i.iid == iid)
                    && self.issue_closing_mrs != data
                {
                    self.issue_closing_mrs = data;
                }
            }
            DataUpdate::Branches {
                project_id, data, ..
            } => {
                let is_selected_project = self
                    .selected_project
                    .as_ref()
                    .is_some_and(|p| p.id == project_id);
                let is_selected_mr_project = self
                    .selected_mr
                    .as_ref()
                    .is_some_and(|mr| mr.project_id == project_id);
                if (is_selected_project || is_selected_mr_project) && self.project_branches != data
                {
                    self.project_branches = data;
                    self.filter_lists();
                }
                if self.show_target_branch_popup
                    && is_selected_mr_project
                    && let Some(mr) = &self.selected_mr
                {
                    let target = mr.target_branch.clone();
                    let idx = self
                        .project_branches
                        .iter()
                        .position(|b| b.name == target)
                        .unwrap_or(0);
                    self.target_branch_state.select(Some(idx));
                }
            }
            DataUpdate::Commits {
                project_id,
                data,
                page,
                has_more,
            } => {
                if self
                    .selected_project
                    .as_ref()
                    .is_some_and(|p| p.id == project_id)
                    && (self.commits != data
                        || self.commits_page != page
                        || self.commits_has_more != has_more)
                {
                    self.commits = data;
                    self.commits_page = page;
                    self.commits_has_more = has_more;
                    self.filter_lists();
                }
            }
            DataUpdate::AppendCommits {
                project_id,
                mut data,
            } => {
                if self
                    .selected_project
                    .as_ref()
                    .is_some_and(|p| p.id == project_id)
                    && !data.is_empty()
                {
                    self.commits_has_more = data.len() >= 50;
                    self.commits.append(&mut data);
                    self.commits_page += 1;
                    self.filter_lists();
                }
            }
            DataUpdate::Boards { project_id, data } => {
                if self
                    .selected_project
                    .as_ref()
                    .is_some_and(|p| p.id == project_id)
                {
                    self.project_boards = data;
                    self.project_boards_state
                        .select(if self.project_boards.is_empty() {
                            None
                        } else {
                            Some(0)
                        });
                }
            }
            DataUpdate::BoardLists {
                project_id,
                board_id,
                data,
            } => {
                if self
                    .selected_project
                    .as_ref()
                    .is_some_and(|p| p.id == project_id)
                    && self
                        .selected_board
                        .as_ref()
                        .is_some_and(|b| b.id == board_id)
                {
                    let prev_sel = self.board_lists_state.selected();
                    self.board_lists = data.clone();
                    self.board_lists_state
                        .select(if self.board_lists.is_empty() {
                            None
                        } else {
                            Some(
                                prev_sel
                                    .map(|s| s.min(self.board_lists.len() - 1))
                                    .unwrap_or(0),
                            )
                        });
                    let lists_to_fetch = self.board_lists.clone();
                    for list in lists_to_fetch {
                        let _ = self.fetch_board_list_issues(board_id, list.id).await;
                    }
                }
            }
            DataUpdate::BoardIssues {
                project_id,
                board_id: _,
                list_id,
                data,
            } => {
                if self
                    .selected_project
                    .as_ref()
                    .is_some_and(|p| p.id == project_id)
                {
                    self.board_issues.insert(list_id, data);
                    self.filter_lists();
                }
            }
            DataUpdate::Epics { group_id: _, data } => {
                self.epics = data;
                self.epics_state
                    .select(if self.epics.is_empty() { None } else { Some(0) });
            }
            DataUpdate::EpicIssues {
                group_id,
                epic_iid,
                data,
            } => {
                if self
                    .selected_epic
                    .as_ref()
                    .is_some_and(|e| e.group_id == Some(group_id) && e.iid == epic_iid)
                {
                    self.epic_issues = data;
                    self.epic_issues_state
                        .select(if self.epic_issues.is_empty() {
                            None
                        } else {
                            Some(0)
                        });
                    self.filter_lists();
                }
            }
            DataUpdate::TerminalEscape(_) => {} // Handled in main loop
        }
    }

    pub async fn process_updates(
        &mut self,
        rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
    ) {
        while let Ok(update) = rx.try_recv() {
            self.process_update_single(update).await;
        }
        self.update_preview().await;
    }

    fn fix_label(
        cache: &mut std::collections::HashMap<u64, Vec<Label>>,
        project_id: u64,
        label: &mut Label,
    ) {
        // Learn from the label if it has color info
        if label.id != 0 && label.color != "#808080" {
            let entry = cache.entry(project_id).or_default();
            if !entry.iter().any(|l| l.name == label.name) {
                entry.push(label.clone());
            }
        }
        // Fix the label if it is missing info
        let matching = (label.id == 0 || label.color == "#808080")
            .then(|| {
                cache
                    .get(&project_id)
                    .and_then(|cached_labels| cached_labels.iter().find(|l| l.name == label.name))
            })
            .flatten();
        if let Some(matching) = matching {
            label.id = matching.id;
            label.color = matching.color.clone();
            label.text_color = matching.text_color.clone();
        }
    }

    fn fix_label_list(labels: &[Label], item_labels: &mut [Label]) {
        for label in item_labels {
            let matching = (label.id == 0 || label.color == "#808080")
                .then(|| labels.iter().find(|l| l.name == label.name))
                .flatten();
            if let Some(matching) = matching {
                label.id = matching.id;
                label.color = matching.color.clone();
                label.text_color = matching.text_color.clone();
            }
        }
    }

    fn fix_issue_labels(&mut self, issue: &mut Issue) {
        let project_id = issue.project_id;
        for label in &mut issue.labels {
            Self::fix_label(&mut self.project_label_cache, project_id, label);
        }
    }

    fn fix_mr_labels(&mut self, mr: &mut MergeRequest) {
        let project_id = mr.project_id;
        for label in &mut mr.labels {
            Self::fix_label(&mut self.project_label_cache, project_id, label);
        }
    }

    fn parse_diff_hunk_header(line: &str) -> Option<(u64, u64)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return None;
        }
        let old_info = parts[1].trim_start_matches('-');
        let new_info = parts[2].trim_start_matches('+');
        let old_start = old_info.split(',').next()?.parse().ok()?;
        let new_start = new_info.split(',').next()?.parse().ok()?;
        Some((old_start, new_start))
    }

    pub fn enrich_diffs_with_discussions(&mut self) {
        let normalize = |p: &str| {
            let mut s = p.to_lowercase().replace('\\', "/");
            loop {
                let start = s.len();
                s = s
                    .trim_start_matches("./")
                    .trim_start_matches("a/")
                    .trim_start_matches("b/")
                    .trim_start_matches("/")
                    .to_string();
                if s.len() == start {
                    break;
                }
            }
            s
        };

        // Populate context on discussions
        for disc in &mut self.discussions {
            if let Some(note) = disc.notes.first_mut()
                && let Some(pos) = &note.position
            {
                let pos_new_norm = pos.new_path.as_deref().map(normalize);
                let pos_old_norm = pos.old_path.as_deref().map(normalize);
                let diff_opt = self.mr_diffs.iter().find(|diff| {
                    let diff_new_norm = normalize(&diff.new_path);
                    let diff_old_norm = normalize(&diff.old_path);
                    pos_new_norm == Some(diff_new_norm) || pos_old_norm == Some(diff_old_norm)
                });
                if let Some(diff) = diff_opt {
                    note.context = mrs::parse_diff_context(&diff.diff, pos.new_line, pos.old_line);
                }
            }
        }

        for diff in &mut self.mr_diffs {
            if diff.diff.is_empty() && diff.collapsed {
                if diff.display_lines.is_empty() {
                    diff.display_lines.push(DiffDisplayLine::Code(
                        "@@ [Diff collapsed by GitLab API. File too large or too many files.] @@"
                            .to_string(),
                        None,
                        None,
                    ));
                    diff.display_lines.push(DiffDisplayLine::Code("@@ Press 'E' or 'e' to force fetch this file's diff from the repository. @@".to_string(), None, None));
                }
                continue;
            } else if diff.diff.is_empty() {
                diff.display_lines = vec![DiffDisplayLine::Code(
                    "@@ [Empty diff or binary file: no changes to display] @@".to_string(),
                    None,
                    None,
                )];
                continue;
            }

            let is_binary = diff.diff.contains('\0') || diff.diff.starts_with("Binary files ");
            if is_binary {
                diff.display_lines = vec![DiffDisplayLine::Code(
                    format!("@@ [Binary file: {}] @@", diff.new_path),
                    None,
                    None,
                )];
                continue;
            }

            let mut display_lines = Vec::new();
            let mut current_old_line = 0;
            let mut current_new_line = 0;

            let diff_new_norm = normalize(&diff.new_path);
            let diff_old_norm = normalize(&diff.old_path);
            let diff_new_filename = diff
                .new_path
                .split('/')
                .next_back()
                .unwrap_or("")
                .to_lowercase();

            for line in diff.diff.lines() {
                if line.starts_with("@@") {
                    if let Some((old_start, new_start)) = Self::parse_diff_hunk_header(line) {
                        current_old_line = old_start;
                        current_new_line = new_start;
                    }
                    display_lines.push(DiffDisplayLine::Code(line.to_string(), None, None));
                } else {
                    let old_val = if line.starts_with('+') {
                        None
                    } else {
                        Some(current_old_line)
                    };
                    let new_val = if line.starts_with('-') {
                        None
                    } else {
                        Some(current_new_line)
                    };

                    if line.starts_with('+') {
                        current_new_line += 1;
                    } else if line.starts_with('-') {
                        current_old_line += 1;
                    } else {
                        current_old_line += 1;
                        current_new_line += 1;
                    }

                    display_lines.push(DiffDisplayLine::Code(line.to_string(), old_val, new_val));

                    // Inject discussions
                    for disc in &self.discussions {
                        if let Some(pos) =
                            disc.notes.first().and_then(|note| note.position.as_ref())
                        {
                            let pos_new_norm = pos.new_path.as_deref().map(normalize);
                            let pos_old_norm = pos.old_path.as_deref().map(normalize);
                            let pos_filename = pos
                                .new_path
                                .as_deref()
                                .and_then(|p| p.split('/').next_back())
                                .map(|s| s.to_lowercase());

                            // Aggressive matching: exact normalized path OR filename match if within line range
                            let path_matches = pos_new_norm == Some(diff_new_norm.clone())
                                || pos_old_norm == Some(diff_old_norm.clone())
                                || (pos_filename == Some(diff_new_filename.clone())
                                    && (pos.new_line == new_val || pos.old_line == old_val));

                            if path_matches {
                                let line_matches = if line.starts_with('+') {
                                    pos.new_line == new_val
                                } else if line.starts_with('-') {
                                    pos.old_line == old_val
                                } else {
                                    pos.new_line == new_val || pos.old_line == old_val
                                };

                                if line_matches {
                                    display_lines.push(DiffDisplayLine::Discussion(disc.clone()));
                                }
                            }
                        }
                    }
                }
            }
            diff.display_lines = display_lines;
        }
    }

    pub async fn initial_setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_loading = true;
        self.loading_tasks = 4;
        let c = self.gitlab.clone();
        let tx = self.tx.clone();

        let t0 = tx.clone();
        let c0 = c.clone();
        tokio::spawn(async move {
            let ep = CurrentUser;
            let res: Result<Author, _> = ep.query_async(&c0).await;
            if let Ok(user) = res {
                let _ = t0.send(DataUpdate::CurrentUser(user));
            }
            let _ = t0.send(DataUpdate::Loading(false));
        });

        let t1 = tx.clone();
        let c1 = c.clone();
        tokio::spawn(async move {
            let ep = Todos {
                page: Some(1),
                per_page: Some(50),
                state: Some("pending".to_string()),
            };
            let res: Result<Result<Vec<Todo>, _>, _> =
                tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c1)).await;
            match res {
                Ok(Ok(data)) => {
                    let has_more = data.len() == 50;
                    let _ = t1.send(DataUpdate::Todos {
                        data,
                        page: 1,
                        has_more,
                    });
                }
                Ok(Err(e)) => {
                    let _ = t1.send(DataUpdate::Error(format!("Todos fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = t1.send(DataUpdate::Error("Todos fetch timed out".into()));
                }
            }
            let _ = t1.send(DataUpdate::Loading(false));
        });
        let t2 = tx.clone();
        let c2 = c.clone();
        tokio::spawn(async move {
            let ep = PagedProjects {
                membership: true,
                search: None,
                page: 1,
                per_page: 100,
                simple: Some(true),
                archived: Some(false),
                order_by: Some("last_activity_at".into()),
            };
            let res: Result<Result<Vec<Project>, _>, _> =
                tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c2)).await;
            match res {
                Ok(Ok(data)) => {
                    let has_more = data.len() == 100;
                    let _ = t2.send(DataUpdate::Projects {
                        data,
                        page: 1,
                        has_more,
                    });
                }
                Ok(Err(e)) => {
                    let _ = t2.send(DataUpdate::Error(format!("Projects fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = t2.send(DataUpdate::Error("Projects fetch timed out".into()));
                }
            }
            let _ = t2.send(DataUpdate::Loading(false));
        });
        let t3 = tx.clone();
        let c3 = c.clone();
        tokio::spawn(async move {
            let ep = Groups {
                page: Some(1),
                per_page: Some(50),
                min_access_level: Some(10),
                top_level_only: None,
            };
            let res: Result<Result<Vec<Group>, _>, _> =
                tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c3)).await;
            match res {
                Ok(Ok(data)) => {
                    let has_more = data.len() == 50;
                    let _ = t3.send(DataUpdate::Groups {
                        data,
                        page: 1,
                        has_more,
                    });
                }
                Ok(Err(e)) => {
                    let _ = t3.send(DataUpdate::Error(format!("Groups fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = t3.send(DataUpdate::Error("Groups fetch timed out".into()));
                }
            }
            let _ = t3.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub fn trigger_project_metadata_fetch(&self, project_id: u64) {
        self.trigger_project_metadata_fetch_impl(project_id, false);
    }

    pub fn trigger_project_metadata_fetch_force(&self, project_id: u64) {
        self.trigger_project_metadata_fetch_impl(project_id, true);
    }

    fn trigger_project_metadata_fetch_impl(&self, project_id: u64, force: bool) {
        if !force && self.project_label_cache.contains_key(&project_id) {
            let labels = self
                .project_label_cache
                .get(&project_id)
                .cloned()
                .unwrap_or_default();
            let milestones = self
                .project_milestone_cache
                .get(&project_id)
                .cloned()
                .unwrap_or_default();
            let members = self
                .project_member_cache
                .get(&project_id)
                .cloned()
                .unwrap_or_default();
            if !members.is_empty() {
                let _ = self.tx.send(DataUpdate::Metadata {
                    project_id,
                    labels,
                    milestones,
                    members,
                });
                return;
            }
        }
        {
            let mut fetches = self.project_metadata_fetches_in_progress.lock().unwrap();
            if !fetches.insert(project_id) {
                return;
            }
        }
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let fetches_in_progress = self.project_metadata_fetches_in_progress.clone();
        tokio::spawn(async move {
            let lp_ep = ProjectLabels {
                project: project_id,
            };
            let mp_ep = ProjectMilestones {
                project: project_id,
            };
            let memp_all_ep = ProjectMembersAll {
                project: project_id,
            };

            let lp = gitlab::api::paged(lp_ep, gitlab::api::Pagination::All);
            let mp = gitlab::api::paged(mp_ep, gitlab::api::Pagination::All);
            let memp_all = gitlab::api::paged(memp_all_ep, gitlab::api::Pagination::All);

            let (l_res, m_res, mem_all_res) = tokio::join!(
                tokio::time::timeout(std::time::Duration::from_secs(20), lp.query_async(&client)),
                tokio::time::timeout(std::time::Duration::from_secs(20), mp.query_async(&client)),
                tokio::time::timeout(
                    std::time::Duration::from_secs(20),
                    memp_all.query_async(&client)
                )
            );

            let labels: Vec<Label> = match l_res {
                Ok(Ok(data)) => data,
                Ok(Err(err)) => {
                    tracing::error!(
                        "Failed to fetch project labels for project {}: {:?}",
                        project_id,
                        err
                    );
                    Vec::new()
                }
                Err(_) => {
                    tracing::error!("Timeout fetching project labels for project {}", project_id);
                    Vec::new()
                }
            };
            let milestones: Vec<Milestone> = match m_res {
                Ok(Ok(data)) => data,
                Ok(Err(err)) => {
                    tracing::error!(
                        "Failed to fetch project milestones for project {}: {:?}",
                        project_id,
                        err
                    );
                    Vec::new()
                }
                Err(_) => {
                    tracing::error!(
                        "Timeout fetching project milestones for project {}",
                        project_id
                    );
                    Vec::new()
                }
            };
            let mut members: Vec<Author> = match mem_all_res {
                Ok(Ok(data)) => data,
                Ok(Err(err)) => {
                    tracing::warn!(
                        "Failed to fetch all project members for project {}: {:?}",
                        project_id,
                        err
                    );
                    Vec::new()
                }
                Err(_) => {
                    tracing::warn!(
                        "Timeout fetching all project members for project {}",
                        project_id
                    );
                    Vec::new()
                }
            };

            if members.is_empty() {
                // Fallback 1: GET /projects/:id/members
                let memp_ep = ProjectMembers {
                    project: project_id,
                };
                let memp = gitlab::api::paged(memp_ep, gitlab::api::Pagination::All);
                match tokio::time::timeout(
                    std::time::Duration::from_secs(20),
                    memp.query_async(&client),
                )
                .await
                {
                    Ok(Ok(data)) => {
                        members = data;
                    }
                    Ok(Err(err)) => {
                        tracing::warn!(
                            "Failed fallback 1 (direct project members) for project {}: {:?}",
                            project_id,
                            err
                        );
                    }
                    Err(_) => {
                        tracing::warn!(
                            "Timeout fallback 1 (direct project members) for project {}",
                            project_id
                        );
                    }
                }
            }

            if members.is_empty() {
                // Fallback 2: GET /projects/:id/users
                let users_ep = ProjectUsers {
                    project: project_id,
                };
                let users = gitlab::api::paged(users_ep, gitlab::api::Pagination::All);
                match tokio::time::timeout(
                    std::time::Duration::from_secs(20),
                    users.query_async(&client),
                )
                .await
                {
                    Ok(Ok(data)) => {
                        members = data;
                    }
                    Ok(Err(err)) => {
                        tracing::error!(
                            "Failed fallback 2 (project users) for project {}: {:?}",
                            project_id,
                            err
                        );
                    }
                    Err(_) => {
                        tracing::error!(
                            "Timeout fallback 2 (project users) for project {}",
                            project_id
                        );
                    }
                }
            }

            if !labels.is_empty() || !milestones.is_empty() || !members.is_empty() {
                tracing::info!(
                    "Sending metadata for project {}: {} labels, {} milestones, {} members",
                    project_id,
                    labels.len(),
                    milestones.len(),
                    members.len()
                );
                let _ = tx.send(DataUpdate::Metadata {
                    project_id,
                    labels,
                    milestones,
                    members,
                });
            }
            let _ = fetches_in_progress.lock().unwrap().remove(&project_id);
        });
    }

    pub async fn refresh(&mut self) {
        self.refresh_impl(true).await;
    }

    pub async fn refresh_background(&mut self) {
        self.refresh_impl(false).await;
    }

    pub async fn refresh_impl(&mut self, force_detail: bool) {
        self.last_refresh = std::time::Instant::now();

        if force_detail {
            self.membership_sync_attempted = false;
        }

        if self.current_user.is_none() {
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let ep = CurrentUser;
                let res: Result<Author, _> = ep.query_async(&c).await;
                if let Ok(user) = res {
                    let _ = tx.send(DataUpdate::CurrentUser(user));
                }
            });
        }

        if self.member_project_ids.is_empty()
            && !self.is_syncing_membership
            && !self.membership_sync_attempted
        {
            let _ = self.fetch_membership_project_ids().await;
        }

        let should_fetch_lists =
            self.view_mode == ViewMode::List && self.active_panel != Panel::MainDetail;

        match self.active_context {
            Context::Dashboard | Context::Explore => match self.active_selection {
                MenuSelection::Home => {
                    // Pre-fetch todos & MRs in background so badges populate on Home
                    let _ = self.fetch_todos().await;
                    let _ = self.fetch_global_merge_requests().await;
                }
                MenuSelection::Todos => {
                    if should_fetch_lists {
                        if self.todos_etag.is_some() || !self.todos.is_empty() {
                            if let Err(e) = self.fetch_todos_conditional().await {
                                tracing::debug!(
                                    "conditional todos poll failed, falling back: {}",
                                    e
                                );
                                let _ = self.fetch_todos().await;
                            }
                        } else {
                            let _ = self.fetch_todos().await;
                        }
                    }
                }
                MenuSelection::MergeRequests => {
                    if should_fetch_lists {
                        let _ = self.fetch_global_merge_requests().await;
                    }
                }
                MenuSelection::Issues => {
                    if should_fetch_lists {
                        let _ = self.fetch_global_issues().await;
                    }
                }
                MenuSelection::Pipelines => {
                    if should_fetch_lists {
                        let _ = self.fetch_cross_pipelines().await;
                    }
                }
                MenuSelection::Projects => {
                    if should_fetch_lists {
                        let _ = self.fetch_projects().await;
                    }
                }
                MenuSelection::Groups if should_fetch_lists => {
                    let _ = self.fetch_groups().await;
                }
                _ => {}
            },
            Context::Project => {
                if let Some(_project) = &self.selected_project {
                    match self.active_selection {
                        MenuSelection::MergeRequests => {
                            if should_fetch_lists {
                                let _ = self.fetch_merge_requests().await;
                            }
                        }
                        MenuSelection::Issues => {
                            if should_fetch_lists {
                                let _ = self.fetch_issues().await;
                            }
                        }
                        MenuSelection::Pipelines => {
                            if should_fetch_lists {
                                let _ = self.fetch_pipelines().await;
                            }
                        }
                        MenuSelection::Repository => {
                            if should_fetch_lists {
                                let _ = self.fetch_repository_tree().await;
                            }
                        }
                        MenuSelection::Branches => {
                            if let Some(p) = &self.selected_project
                                && should_fetch_lists
                            {
                                let _ = self.fetch_branches(p.id).await;
                            }
                        }
                        MenuSelection::Commits => {
                            if should_fetch_lists {
                                let _ = self.fetch_commits().await;
                            }
                        }
                        MenuSelection::Boards => {
                            if self.view_mode == ViewMode::Board
                                && self.active_panel != Panel::MainDetail
                            {
                                if let Some(board) = self.selected_board.clone() {
                                    let _ = self.fetch_board_lists(board.id).await;
                                }
                            } else if self.view_mode == ViewMode::List
                                && self.active_panel != Panel::MainDetail
                            {
                                let _ = self.fetch_boards().await;
                            }
                        }
                        MenuSelection::Epics if should_fetch_lists => {
                            let _ = self.fetch_epics().await;
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Some(mr) = self.selected_mr.clone() {
            if force_detail || self.discussions.is_empty() {
                let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
            }
            if self.active_panel == Panel::MainDetail
                && self.related_mrs.is_empty()
                && self.mr_closing_issues.is_empty()
            {
                let _ = self.fetch_related_items().await;
            }
            let has_metadata = self.project_label_cache.contains_key(&mr.project_id);
            if self.active_panel == Panel::MainDetail && !has_metadata {
                self.trigger_project_metadata_fetch(mr.project_id);
            }
        } else if let Some(issue) = self.selected_issue.clone() {
            if force_detail || self.discussions.is_empty() {
                let _ = self
                    .fetch_issue_discussions(issue.project_id, issue.iid)
                    .await;
            }
            if self.active_panel == Panel::MainDetail
                && self.linked_issues.is_empty()
                && self.issue_closing_mrs.is_empty()
            {
                let _ = self.fetch_related_items().await;
            }
            let has_metadata = self.project_label_cache.contains_key(&issue.project_id);
            if self.active_panel == Panel::MainDetail && !has_metadata {
                self.trigger_project_metadata_fetch(issue.project_id);
            }
        } else if let Some(pipeline) = self.selected_pipeline.clone() {
            if self.view_mode == ViewMode::JobLog {
                let _ = self.fetch_jobs(pipeline.project_id, pipeline.id).await;
                if let Some(job) = &self.selected_job
                    && (force_detail || !job.is_finished())
                {
                    let _ = self.fetch_job_log(pipeline.project_id, job.id).await;
                }
            } else {
                let is_finished = pipeline.is_finished();
                if force_detail || self.jobs.is_empty() || !is_finished {
                    let _ = self.fetch_jobs(pipeline.project_id, pipeline.id).await;
                }
            }
            let has_metadata = self.project_label_cache.contains_key(&pipeline.project_id);
            if self.active_panel == Panel::MainDetail && !has_metadata {
                self.trigger_project_metadata_fetch(pipeline.project_id);
            }
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help_popup = !self.show_help_popup;
    }

    pub fn clear_related_items(&mut self) {
        self.related_mrs.clear();
        self.mr_closing_issues.clear();
        self.linked_issues.clear();
        self.issue_closing_mrs.clear();
        self.approvals = None;
    }

    pub fn ensure_job_log_cursor_visible(&mut self) {
        let viewport = (self.job_log_viewport_height as usize).max(1);
        if self.job_log_cursor < self.job_log_scroll as usize {
            self.job_log_scroll = self.job_log_cursor as u16;
        } else if self.job_log_cursor >= self.job_log_scroll as usize + viewport {
            self.job_log_scroll = (self.job_log_cursor + 1).saturating_sub(viewport) as u16;
        }
    }

    pub fn bump_job_log_epoch(&mut self) {
        self.job_log_epoch += 1;
    }

    fn should_fetch_detail(&mut self, pid: u64, iid: u64) -> bool {
        let now = std::time::Instant::now();
        if let Some((p, i, at)) = self.last_detail_fetch
            && p == pid
            && i == iid
            && now.duration_since(at) < std::time::Duration::from_millis(250)
        {
            return false;
        }
        self.last_detail_fetch = Some((pid, iid, now));
        true
    }

    pub fn markdown_cached(&self, body: &str) -> std::sync::Arc<Vec<ratatui::text::Line<'static>>> {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        body.hash(&mut hasher);
        let key = hasher.finish();
        if let Some(cached) = self.markdown_cache.borrow().get(&key) {
            return cached.clone();
        }
        if self.markdown_cache.borrow().len() > 256 {
            self.markdown_cache.borrow_mut().clear();
        }
        let rendered = std::sync::Arc::new(crate::ui::markdown::render_markdown(
            body,
            &self.theme,
            &self.syntax_set,
            &self.theme_set,
        ));
        self.markdown_cache
            .borrow_mut()
            .insert(key, rendered.clone());
        rendered
    }

    pub fn job_log_search_next(&mut self) {
        if self.job_log_lines.is_empty() || self.job_log_search.is_empty() {
            return;
        }
        let needle = self.job_log_search.to_lowercase();
        let len = self.job_log_lines.len();
        for offset in 1..=len {
            let idx = (self.job_log_cursor + offset) % len;
            if self.job_log_lines[idx]
                .content
                .to_lowercase()
                .contains(&needle)
            {
                self.job_log_cursor = idx;
                self.ensure_job_log_cursor_visible();
                return;
            }
        }
        self.status_message = Some(format!("No match for '{}'", self.job_log_search));
    }

    pub fn job_log_search_prev(&mut self) {
        if self.job_log_lines.is_empty() || self.job_log_search.is_empty() {
            return;
        }
        let needle = self.job_log_search.to_lowercase();
        let len = self.job_log_lines.len();
        for offset in 1..=len {
            let idx = (self.job_log_cursor + len - (offset % len)) % len;
            if self.job_log_lines[idx]
                .content
                .to_lowercase()
                .contains(&needle)
            {
                self.job_log_cursor = idx;
                self.ensure_job_log_cursor_visible();
                return;
            }
        }
        self.status_message = Some(format!("No match for '{}'", self.job_log_search));
    }

    const STATUS_TTL: std::time::Duration = std::time::Duration::from_secs(6);
    const ERROR_TTL: std::time::Duration = std::time::Duration::from_secs(15);

    pub fn tick_status(&mut self) {
        if self.last_status_text.as_deref() != self.status_message.as_deref() {
            self.last_status_text = self.status_message.clone();
            self.status_since = std::time::Instant::now();
        } else if let Some(msg) = &self.status_message {
            let is_progress = msg.starts_with("Fetching")
                || msg.starts_with("Loading")
                || msg.starts_with("Updating")
                || msg.starts_with("Searching");
            if !is_progress && self.status_since.elapsed() >= Self::STATUS_TTL {
                self.status_message = None;
                self.last_status_text = None;
                self.status_since = std::time::Instant::now();
            }
        }
        if self.last_error.is_some()
            && self
                .last_error_since
                .map(|t| t.elapsed())
                .unwrap_or_default()
                >= Self::ERROR_TTL
        {
            self.last_error = None;
            self.last_error_since = None;
        }
    }

    pub async fn update_preview(&mut self) {
        let prev_selection = self.active_selection;
        let prev_context = self.active_context;

        match self.active_panel {
            Panel::Dashboard => {
                self.active_context = Context::Dashboard;
                let idx = self.dashboard_menu_state.selected().unwrap_or(0);
                self.active_selection = match idx {
                    0 => MenuSelection::Home,
                    1 => MenuSelection::Todos,
                    2 => MenuSelection::MergeRequests,
                    3 => MenuSelection::Issues,
                    _ => MenuSelection::Pipelines,
                };
            }
            Panel::Explore => {
                self.active_context = Context::Explore;
                let idx = self.explore_menu_state.selected().unwrap_or(0);
                self.active_selection = match idx {
                    0 => MenuSelection::Projects,
                    _ => MenuSelection::Groups,
                };
            }
            Panel::Project => {
                self.active_context = Context::Project;
                let idx = self.project_menu_state.selected().unwrap_or(0);
                self.active_selection = match idx {
                    0 => MenuSelection::MergeRequests,
                    1 => MenuSelection::Issues,
                    2 => MenuSelection::Pipelines,
                    3 => MenuSelection::Repository,
                    4 => MenuSelection::Branches,
                    5 => MenuSelection::Commits,
                    6 => MenuSelection::Boards,
                    _ => MenuSelection::Epics,
                };
            }
            _ => {}
        }

        if prev_selection != self.active_selection || prev_context != self.active_context {
            self.selected_mr = None;
            self.selected_issue = None;
            self.selected_pipeline = None;
            self.discussions.clear();
            self.jobs.clear();
            self.merge_requests.clear();
            self.issues.clear();
            self.pipelines.clear();
            self.repository_ref = None;
            self.clear_related_items();
            self.filter_lists();
            self.refresh().await;
        }

        if self.active_panel == Panel::Main || self.active_panel == Panel::MainDetail {
            match self.active_selection {
                MenuSelection::Todos => {
                    if let Some(GroupedItem::Item(todo)) = self
                        .todos_state
                        .selected()
                        .and_then(|i| self.filtered_todos.get(i))
                    {
                        let re = &*TODO_TARGET_RE;
                        let iid = re
                            .captures(&todo.target_url)
                            .and_then(|cap| cap.get(1))
                            .and_then(|m| m.as_str().parse().ok());

                        if let (Some(iid), Some(p)) = (iid, todo.project.as_ref()) {
                            let project_id = p.id;
                            let target_type = todo.target_type.clone();
                            let title = todo.body.clone();
                            let target_url = todo.target_url.clone();
                            if target_type == "MergeRequest"
                                && self
                                    .selected_mr
                                    .as_ref()
                                    .is_none_or(|m| m.iid != iid || m.project_id != project_id)
                            {
                                self.selected_issue = None;
                                self.discussions.clear();
                                self.mr_diffs.clear();
                                self.is_fetching_diffs = false;
                                self.selected_mr = Some(MergeRequest {
                                    iid,
                                    project_id,
                                    title,
                                    author: Author::new(0, "".into()),
                                    web_url: target_url,
                                    state: "opened".into(),
                                    reactions: Vec::new(),
                                    description: None,
                                    references: None,
                                    upvotes: None,
                                    downvotes: None,
                                    labels: Vec::new(),
                                    milestone: None,
                                    assignees: Vec::new(),
                                    reviewers: Vec::new(),
                                    squash: Some(false),
                                    created_at: Some("".into()),
                                    draft: Some(false),
                                    diff_refs: None,
                                    merge_status: None,
                                    detailed_merge_status: None,
                                    has_conflicts: None,
                                    head_pipeline: None,
                                    source_branch: "".into(),
                                    target_branch: "".into(),
                                    participants: Vec::new(),
                                });
                                if self.should_fetch_detail(project_id, iid) {
                                    let _ = self.fetch_discussions(project_id, iid).await;
                                }
                                if self.active_panel == Panel::MainDetail {
                                    self.trigger_project_metadata_fetch(project_id);
                                }
                            } else if target_type == "Issue"
                                && self.selected_issue.as_ref().is_none_or(|iss| {
                                    iss.iid != iid || iss.project_id != project_id
                                })
                            {
                                self.selected_mr = None;
                                self.discussions.clear();
                                self.selected_issue = Some(Issue {
                                    iid,
                                    project_id,
                                    title,
                                    description: None,
                                    author: Author::new(0, "".into()),
                                    assignees: Vec::new(),
                                    state: "opened".into(),
                                    web_url: target_url,
                                    references: None,
                                    issue_type: None,
                                    labels: Vec::new(),
                                    milestone: None,
                                    due_date: None,
                                    start_date: None,
                                    weight: None,
                                    task_completion_status: None,
                                    confidential: false,
                                    created_at: Some("".into()),
                                    upvotes: None,
                                    downvotes: None,
                                    reactions: Vec::new(),
                                    participants: Vec::new(),
                                });
                                if self.should_fetch_detail(project_id, iid) {
                                    let _ = self.fetch_issue_discussions(project_id, iid).await;
                                }
                                if self.active_panel == Panel::MainDetail {
                                    self.trigger_project_metadata_fetch(project_id);
                                }
                            }
                        } else {
                            self.selected_mr = None;
                            self.selected_issue = None;
                            self.discussions.clear();
                        }
                    }
                }
                MenuSelection::MergeRequests => {
                    if let Some(GroupedItem::Item(mr)) = self
                        .mrs_state
                        .selected()
                        .and_then(|i| self.filtered_mrs.get(i))
                    {
                        if self.selected_mr.as_ref().is_none_or(|prev| {
                            prev.iid != mr.iid || prev.project_id != mr.project_id
                        }) {
                            let mr_clone = mr.clone();
                            let pid = mr.project_id;
                            let iid = mr.iid;
                            self.selected_mr = Some(mr_clone);
                            self.selected_issue = None;
                            self.discussions.clear();
                            self.mr_diffs.clear();
                            self.is_fetching_diffs = false;
                            self.clear_related_items();
                            if self.should_fetch_detail(pid, iid) {
                                let _ = self.fetch_discussions(pid, iid).await;
                            }
                            if self.active_panel == Panel::MainDetail {
                                self.trigger_project_metadata_fetch(pid);
                            }
                        } else if let Some(selected) = &mut self.selected_mr {
                            selected.title = mr.title.clone();
                            selected.state = mr.state.clone();
                            selected.labels = mr.labels.clone();
                            selected.milestone = mr.milestone.clone();
                            selected.assignees = mr.assignees.clone();
                        }
                    }
                }
                MenuSelection::Issues => {
                    if let Some(GroupedItem::Item(issue)) = self
                        .issues_state
                        .selected()
                        .and_then(|i| self.filtered_issues.get(i))
                    {
                        if self.selected_issue.as_ref().is_none_or(|prev| {
                            prev.iid != issue.iid || prev.project_id != issue.project_id
                        }) {
                            let issue_clone = issue.clone();
                            let pid = issue.project_id;
                            let iid = issue.iid;
                            self.selected_issue = Some(issue_clone);
                            self.selected_mr = None;
                            self.discussions.clear();
                            self.clear_related_items();
                            if self.should_fetch_detail(pid, iid) {
                                let _ = self.fetch_issue_discussions(pid, iid).await;
                            }
                            if self.active_panel == Panel::MainDetail {
                                self.trigger_project_metadata_fetch(pid);
                            }
                        } else if let Some(selected) = &mut self.selected_issue {
                            selected.title = issue.title.clone();
                            selected.state = issue.state.clone();
                            selected.labels = issue.labels.clone();
                            selected.milestone = issue.milestone.clone();
                            selected.assignees = issue.assignees.clone();
                            selected.weight = issue.weight;
                            selected.due_date = issue.due_date.clone();
                        }
                    }
                }
                MenuSelection::Pipelines => {
                    if let Some(p) = self
                        .pipelines_state
                        .selected()
                        .and_then(|i| self.pipelines.get(i))
                        .filter(|p| {
                            self.selected_pipeline
                                .as_ref()
                                .is_none_or(|prev| prev.id != p.id)
                        })
                    {
                        let p_clone = p.clone();
                        let project_id = p.project_id;
                        let pipeline_id = p.id;
                        self.selected_pipeline = Some(p_clone);
                        self.jobs.clear();
                        let _ = self.fetch_jobs(project_id, pipeline_id).await;
                        if self.active_panel == Panel::MainDetail {
                            self.trigger_project_metadata_fetch(project_id);
                        }
                    }
                }
                MenuSelection::Boards => {
                    if let Some(board) = self
                        .project_boards_state
                        .selected()
                        .and_then(|i| self.project_boards.get(i))
                        .cloned()
                        .filter(|board| {
                            self.selected_board
                                .as_ref()
                                .is_none_or(|b| b.id != board.id)
                        })
                    {
                        self.selected_board = Some(board.clone());
                        self.board_lists.clear();
                        self.board_issues.clear();
                        let _ = self.fetch_board_lists(board.id).await;
                    }
                }
                _ => {}
            }
        } else {
            self.selected_mr = None;
            self.selected_issue = None;
            self.selected_pipeline = None;
            self.discussions.clear();
            self.jobs.clear();
        }

        // Stability check: If we are in MainDetail but no item is actually selected (e.g. list empty), move back to Main
        if self.active_panel == Panel::MainDetail {
            let is_item_selected = self.selected_mr.is_some()
                || self.selected_issue.is_some()
                || self.selected_pipeline.is_some()
                || self.selected_board.is_some();
            if !is_item_selected {
                self.active_panel = Panel::Main;
            }
        }

        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let needs_diffs = self.view_mode == ViewMode::Diff
                || (self.active_panel == Panel::MainDetail
                    && self.detail_section == DetailSection::Changes);
            if needs_diffs && self.mr_diffs.is_empty() && !self.is_fetching_diffs {
                let _ = self.fetch_mr_diffs(pid, iid).await;
            }
        }
    }

    pub async fn enter_detail_action(&mut self) {
        if self.active_panel == Panel::MainDetail {
            let project_id = self
                .selected_mr
                .as_ref()
                .map(|mr| mr.project_id)
                .or_else(|| self.selected_issue.as_ref().map(|issue| issue.project_id));
            if let Some(pid) = project_id {
                if let Some(labels) = self.project_label_cache.get(&pid) {
                    self.project_labels = labels.clone();
                }
                if let Some(milestones) = self.project_milestone_cache.get(&pid) {
                    self.project_milestones = milestones.clone();
                }
                if let Some(members) = self.project_member_cache.get(&pid) {
                    self.project_members = members.clone();
                }
            }

            match self.detail_section {
                DetailSection::Metadata => match self.selected_metadata_field {
                    MetadataField::Title => {
                        self.input_mode = true;
                        self.input_target = InputTarget::EditTitle;
                        let text = if let Some(i) = &self.selected_issue {
                            i.title.clone()
                        } else if let Some(m) = &self.selected_mr {
                            m.title.clone()
                        } else {
                            String::new()
                        };
                        self.textarea = ratatui_textarea::TextArea::from(text.lines());
                        self.textarea.move_cursor(ratatui_textarea::CursorMove::End);
                    }
                    MetadataField::Description => {
                        self.input_mode = true;
                        self.input_target = InputTarget::EditDescription;
                        let text = if let Some(i) = &self.selected_issue {
                            i.description.clone().unwrap_or_default()
                        } else if let Some(m) = &self.selected_mr {
                            m.description.clone().unwrap_or_default()
                        } else {
                            String::new()
                        };
                        self.textarea = ratatui_textarea::TextArea::from(text.lines());
                        self.textarea.move_cursor(ratatui_textarea::CursorMove::End);
                    }
                    MetadataField::Labels => {
                        self.show_label_popup = true;
                        self.selected_label_ids = if let Some(i) = &self.selected_issue {
                            i.labels.iter().map(|l| l.id()).collect()
                        } else if let Some(m) = &self.selected_mr {
                            m.labels.iter().map(|l| l.id()).collect()
                        } else {
                            Vec::new()
                        };
                        self.label_state.select(Some(0));
                    }
                    MetadataField::Assignees => {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        if self
                            .project_member_cache
                            .get(&pid)
                            .map(|m| m.is_empty())
                            .unwrap_or(true)
                        {
                            self.trigger_project_metadata_fetch_force(pid);
                        }
                        self.show_assignee_popup = true;
                        self.selected_assignee_ids = if let Some(i) = &self.selected_issue {
                            i.assignees.iter().map(|a| a.id).collect()
                        } else if let Some(m) = &self.selected_mr {
                            m.assignees.iter().map(|a| a.id).collect()
                        } else {
                            Vec::new()
                        };
                        self.assignee_state.select(Some(0));
                    }
                    MetadataField::Reviewers => {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        if self
                            .project_member_cache
                            .get(&pid)
                            .map(|m| m.is_empty())
                            .unwrap_or(true)
                        {
                            self.trigger_project_metadata_fetch_force(pid);
                        }
                        self.show_reviewer_popup = true;
                        self.selected_reviewer_ids = if let Some(m) = &self.selected_mr {
                            m.reviewers.iter().map(|a| a.id).collect()
                        } else {
                            Vec::new()
                        };
                        self.reviewer_state.select(Some(0));
                    }
                    MetadataField::Milestone => {
                        self.show_milestone_popup = true;
                        let mid = if let Some(i) = &self.selected_issue {
                            i.milestone.as_ref().map(|m| m.id)
                        } else if let Some(m) = &self.selected_mr {
                            m.milestone.as_ref().map(|mi| mi.id)
                        } else {
                            None
                        };
                        let idx = mid
                            .and_then(|id| self.project_milestones.iter().position(|m| m.id == id))
                            .map(|p| p + 1)
                            .unwrap_or(0);
                        self.milestone_state.select(Some(idx));
                    }
                    MetadataField::StartDate => {
                        if let Some(i) = &self.selected_issue {
                            self.show_calendar_popup = true;
                            self.input_target = InputTarget::EditIssueStartDate;
                            self.calendar_date = i
                                .start_date
                                .as_ref()
                                .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                                .unwrap_or_else(|| chrono::Local::now().date_naive());
                        }
                    }
                    MetadataField::DueDate => {
                        if let Some(i) = &self.selected_issue {
                            self.show_calendar_popup = true;
                            self.input_target = InputTarget::EditIssueDueDate;
                            self.calendar_date = i
                                .due_date
                                .as_ref()
                                .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                                .unwrap_or_else(|| chrono::Local::now().date_naive());
                        }
                    }
                    MetadataField::Confidential => {
                        if self.selected_issue.is_some() {
                            let _ = self.toggle_issue_confidential().await;
                        }
                    }
                    MetadataField::TargetBranch => {
                        if let Some(mr) = &self.selected_mr {
                            let pid = mr.project_id;
                            self.show_target_branch_popup = true;
                            let target = mr.target_branch.clone();
                            let idx = self
                                .project_branches
                                .iter()
                                .position(|b| b.name == target)
                                .unwrap_or(0);
                            self.target_branch_state.select(Some(idx));
                            let _ = self.fetch_branches(pid).await;
                        }
                    }
                },
                DetailSection::Discussions => {
                    self.input_mode = true;
                    self.input_target = InputTarget::Reply;
                    self.textarea = ratatui_textarea::TextArea::default();
                }
                DetailSection::Description => {
                    self.input_mode = true;
                    self.input_target = InputTarget::EditDescription;
                    let text = if let Some(i) = &self.selected_issue {
                        i.description.clone().unwrap_or_default()
                    } else if let Some(m) = &self.selected_mr {
                        m.description.clone().unwrap_or_default()
                    } else {
                        String::new()
                    };
                    self.textarea = ratatui_textarea::TextArea::from(text.lines());
                    self.textarea.move_cursor(ratatui_textarea::CursorMove::End);
                }
                DetailSection::Changes => {
                    if self.selected_mr.is_some() {
                        self.view_mode = ViewMode::Diff;
                        self.mr_diff_state.select(Some(0));
                        let (pid, iid) = {
                            let m = self.selected_mr.as_ref().unwrap();
                            (m.project_id, m.iid)
                        };
                        let _ = self.fetch_mr_diffs(pid, iid).await;
                    }
                }
            }
        }
    }

    pub fn filter_lists(&mut self) {
        let filter = self.filter_input.to_lowercase();
        let me_id = self.current_user.as_ref().map(|u| u.id);
        let show_involved = self.show_involved_only;
        let member_ids = &self.member_project_ids;

        let is_mr_member = |mr: &MergeRequest| {
            if self.active_context != Context::Dashboard {
                return true;
            }
            if !self.membership_sync_attempted && member_ids.is_empty() {
                true
            } else {
                member_ids.contains(&mr.project_id)
            }
        };

        let is_issue_member = |issue: &Issue| {
            if self.active_context != Context::Dashboard {
                return true;
            }
            if !self.membership_sync_attempted && member_ids.is_empty() {
                true
            } else {
                member_ids.contains(&issue.project_id)
            }
        };

        let is_mr_involved = |mr: &MergeRequest| {
            if !show_involved {
                return true;
            }
            if let Some(uid) = me_id {
                mr.author.id == uid
                    || mr.assignees.iter().any(|a| a.id == uid)
                    || mr.reviewers.iter().any(|r| r.id == uid)
                    || mr.participants.iter().any(|p| p.id == uid)
            } else {
                false
            }
        };

        let is_issue_involved = |issue: &Issue| {
            if !show_involved {
                return true;
            }
            if let Some(uid) = me_id {
                issue.author.id == uid
                    || issue.assignees.iter().any(|a| a.id == uid)
                    || issue.participants.iter().any(|p| p.id == uid)
            } else {
                false
            }
        };

        // 1. Group Todos
        let mut grouped_todos = Vec::new();
        let mut todos_by_project: std::collections::BTreeMap<String, Vec<Todo>> =
            std::collections::BTreeMap::new();
        for t in self
            .todos
            .iter()
            .filter(|t| t.body.to_lowercase().contains(&filter))
        {
            let p_name = if let Some(ref p) = t.project {
                p.path_with_namespace
                    .as_deref()
                    .unwrap_or(&p.name)
                    .to_string()
            } else if let Some(ref g) = t.group {
                g.full_path.clone()
            } else {
                "Global".to_string()
            };
            todos_by_project.entry(p_name).or_default().push(t.clone());
        }
        for (p_name, items) in todos_by_project {
            let key = format!("todos:{}", p_name);
            let is_collapsed = self.collapsed_groups.contains(&key);
            grouped_todos.push(GroupedItem::Header(p_name));
            if !is_collapsed {
                for item in items {
                    grouped_todos.push(GroupedItem::Item(item));
                }
            }
        }
        self.filtered_todos = grouped_todos;

        // 2. Group Merge Requests
        let mut grouped_mrs = Vec::new();
        if self.active_context == Context::Dashboard {
            let mut mrs_by_project: std::collections::BTreeMap<String, Vec<MergeRequest>> =
                std::collections::BTreeMap::new();
            for mr in self.merge_requests.iter().filter(|m| {
                m.title.to_lowercase().contains(&filter) && is_mr_involved(m) && is_mr_member(m)
            }) {
                let p_name = self
                    .project_cache
                    .get(&mr.project_id)
                    .and_then(|p| p.path_with_namespace.as_deref())
                    .map(|s| s.to_string())
                    .or_else(|| {
                        mr.references.as_ref().map(|r| {
                            let parts: Vec<&str> = r.full.split('!').collect();
                            parts[0].to_string()
                        })
                    })
                    .unwrap_or_else(|| "Unknown Project".to_string());
                mrs_by_project.entry(p_name).or_default().push(mr.clone());
            }
            for (p_name, items) in mrs_by_project {
                let key = format!("mrs:{}", p_name);
                let is_collapsed = self.collapsed_groups.contains(&key);
                grouped_mrs.push(GroupedItem::Header(p_name));
                if !is_collapsed {
                    for item in items {
                        grouped_mrs.push(GroupedItem::Item(item));
                    }
                }
            }
        } else {
            grouped_mrs = self
                .merge_requests
                .iter()
                .filter(|m| {
                    m.title.to_lowercase().contains(&filter) && is_mr_involved(m) && is_mr_member(m)
                })
                .cloned()
                .map(GroupedItem::Item)
                .collect();
        }
        self.filtered_mrs = grouped_mrs;

        // 3. Group Issues
        let mut grouped_issues = Vec::new();
        if self.issue_group_by_milestone {
            let mut issues_by_milestone: std::collections::BTreeMap<String, Vec<Issue>> =
                std::collections::BTreeMap::new();
            for i in self.issues.iter().filter(|i| {
                i.title.to_lowercase().contains(&filter)
                    && is_issue_involved(i)
                    && is_issue_member(i)
            }) {
                let m_name = i
                    .milestone
                    .as_ref()
                    .map(|m| m.title.clone())
                    .unwrap_or_else(|| "No milestone".to_string());
                issues_by_milestone
                    .entry(m_name)
                    .or_default()
                    .push(i.clone());
            }
            for (m_name, items) in issues_by_milestone {
                let open = items.iter().filter(|i| i.state == "opened").count();
                let closed = items.iter().filter(|i| i.state == "closed").count();
                grouped_issues.push(GroupedItem::Header(format!(
                    "{} ({} open / {} closed)",
                    m_name, open, closed
                )));
                for item in items {
                    grouped_issues.push(GroupedItem::Item(item));
                }
            }
        } else if self.active_context == Context::Dashboard {
            let mut issues_by_project: std::collections::BTreeMap<String, Vec<Issue>> =
                std::collections::BTreeMap::new();
            for i in self.issues.iter().filter(|i| {
                i.title.to_lowercase().contains(&filter)
                    && is_issue_involved(i)
                    && is_issue_member(i)
            }) {
                let p_name = self
                    .project_cache
                    .get(&i.project_id)
                    .and_then(|p| p.path_with_namespace.as_deref())
                    .map(|s| s.to_string())
                    .or_else(|| {
                        i.references.as_ref().map(|r| {
                            let parts: Vec<&str> = r.full.split('#').collect();
                            parts[0].to_string()
                        })
                    })
                    .unwrap_or_else(|| "Unknown Project".to_string());
                issues_by_project.entry(p_name).or_default().push(i.clone());
            }
            for (p_name, items) in issues_by_project {
                let key = format!("issues:{}", p_name);
                let is_collapsed = self.collapsed_groups.contains(&key);
                grouped_issues.push(GroupedItem::Header(p_name));
                if !is_collapsed {
                    for item in items {
                        grouped_issues.push(GroupedItem::Item(item));
                    }
                }
            }
        } else {
            grouped_issues = self
                .issues
                .iter()
                .filter(|i| {
                    i.title.to_lowercase().contains(&filter)
                        && is_issue_involved(i)
                        && is_issue_member(i)
                })
                .cloned()
                .map(GroupedItem::Item)
                .collect();
        }
        self.filtered_issues = grouped_issues;

        self.filtered_projects = self
            .projects
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&filter))
            .cloned()
            .collect();
        self.filtered_pipelines = self
            .pipelines
            .iter()
            .filter(|p| p.ref_name.to_lowercase().contains(&filter))
            .filter(|p| match self.pipeline_filter {
                PipelineFilter::All => true,
                PipelineFilter::Running => matches!(
                    p.status.as_str(),
                    "running" | "pending" | "waiting_for_resource" | "preparing"
                ),
                PipelineFilter::Failed => p.status == "failed",
                PipelineFilter::Passed => p.status == "success",
            })
            .cloned()
            .collect();
        if self.active_context == Context::Dashboard {
            self.filtered_pipelines = self
                .cross_pipelines
                .iter()
                .filter(|p| p.ref_name.to_lowercase().contains(&filter))
                .filter(|p| match self.pipeline_filter {
                    PipelineFilter::All => true,
                    PipelineFilter::Running => matches!(
                        p.status.as_str(),
                        "running" | "pending" | "waiting_for_resource" | "preparing"
                    ),
                    PipelineFilter::Failed => p.status == "failed",
                    PipelineFilter::Passed => p.status == "success",
                })
                .cloned()
                .collect();
        }
        self.filtered_branches = self
            .project_branches
            .iter()
            .filter(|b| b.name.to_lowercase().contains(&filter))
            .cloned()
            .collect();
        self.filtered_epic_issues = self
            .epic_issues
            .iter()
            .filter(|i| i.title.to_lowercase().contains(&filter))
            .cloned()
            .collect();

        // Filter Board Issues
        self.filtered_board_issues.clear();
        for (list_id, issues) in &self.board_issues {
            let filtered: Vec<_> = issues
                .iter()
                .filter(|i| {
                    i.title.to_lowercase().contains(&filter) || i.iid.to_string().contains(&filter)
                })
                .cloned()
                .collect();
            self.filtered_board_issues.insert(*list_id, filtered);
        }

        // Selection normalization
        if !filter.is_empty() && self.filter_input != self.previous_filter_input {
            self.todos_state.select(Some(0));
            self.mrs_state.select(Some(0));
            self.issues_state.select(Some(0));
            self.projects_state.select(Some(0));
            self.pipelines_state.select(Some(0));
            self.branches_state.select(Some(0));
            for list in &self.board_lists {
                self.board_issue_states
                    .entry(list.id)
                    .or_default()
                    .select(Some(0));
            }
        }
        self.previous_filter_input = self.filter_input.clone();

        self.clamp_selections();
    }

    pub fn clamp_selections(&mut self) {
        let active_sel = self.active_selection;
        let is_main_focused = self.active_panel == Panel::Main;

        let clamp = |state: &mut ListState, len: usize, is_active: bool| {
            if len == 0 {
                // Proactive selection: if this is the active list, pre-select 0 even if empty
                // so it's ready when data arrives.
                if is_active && is_main_focused {
                    state.select(Some(0));
                } else {
                    state.select(None);
                }
            } else if let Some(i) = state.selected() {
                if i >= len {
                    state.select(Some(len.saturating_sub(1)));
                }
            } else {
                state.select(Some(0));
            }
        };

        clamp(
            &mut self.todos_state,
            self.filtered_todos.len(),
            active_sel == MenuSelection::Todos,
        );
        clamp(
            &mut self.mrs_state,
            self.filtered_mrs.len(),
            active_sel == MenuSelection::MergeRequests,
        );
        clamp(
            &mut self.issues_state,
            self.filtered_issues.len(),
            active_sel == MenuSelection::Issues,
        );
        clamp(
            &mut self.projects_state,
            self.filtered_projects.len(),
            active_sel == MenuSelection::Projects,
        );
        clamp(
            &mut self.pipelines_state,
            self.filtered_pipelines.len(),
            active_sel == MenuSelection::Pipelines,
        );
        clamp(
            &mut self.groups_state,
            self.filtered_groups.len(),
            active_sel == MenuSelection::Groups,
        );
        clamp(
            &mut self.branches_state,
            self.filtered_branches.len(),
            active_sel == MenuSelection::Branches,
        );
        clamp(
            &mut self.commits_state,
            self.commits.len(),
            active_sel == MenuSelection::Commits,
        );
        clamp(
            &mut self.jobs_state,
            self.jobs.len(),
            self.active_panel == Panel::MainDetail && active_sel == MenuSelection::Pipelines,
        );
        clamp(
            &mut self.discussion_state,
            self.discussions.len(),
            self.active_panel == Panel::MainDetail
                && self.detail_section == DetailSection::Discussions,
        );
        clamp(
            &mut self.epics_state,
            self.epics.len(),
            active_sel == MenuSelection::Epics,
        );
        clamp(
            &mut self.epic_issues_state,
            self.filtered_epic_issues.len(),
            active_sel == MenuSelection::Epics && self.selected_epic.is_some(),
        );
        clamp(
            &mut self.project_boards_state,
            self.project_boards.len(),
            active_sel == MenuSelection::Boards,
        );
        clamp(
            &mut self.search_state,
            self.search_results.len(),
            self.show_search,
        );
        clamp(
            &mut self.mr_diff_state,
            self.mr_diffs.len(),
            self.active_panel == Panel::MainDetail && self.detail_section == DetailSection::Changes,
        );
        clamp(
            &mut self.board_lists_state,
            self.board_lists.len(),
            self.view_mode == ViewMode::Board,
        );

        // Clamp board issues
        for list in &self.board_lists {
            let len = self
                .filtered_board_issues
                .get(&list.id)
                .map(|v| v.len())
                .unwrap_or(0);
            let state = self.board_issue_states.entry(list.id).or_default();
            if len == 0 {
                if self.view_mode == ViewMode::Board
                    && self.board_lists_state.selected()
                        == self.board_lists.iter().position(|l| l.id == list.id)
                {
                    state.select(Some(0));
                } else {
                    state.select(None);
                }
            } else if let Some(i) = state.selected() {
                if i >= len {
                    state.select(Some(len.saturating_sub(1)));
                }
            } else {
                state.select(Some(0));
            }
        }

        // Navigation state defaults
        if self.dashboard_menu_state.selected().is_none() {
            self.dashboard_menu_state.select(Some(0));
        }
        if self.explore_menu_state.selected().is_none() {
            self.explore_menu_state.select(Some(0));
        }
        if self.project_menu_state.selected().is_none() {
            self.project_menu_state.select(Some(0));
        }
    }

    pub fn set_rect(&mut self, key: &str, rect: ratatui::layout::Rect) {
        self.last_rendered_rects.insert(key.to_string(), rect);
    }

    pub fn get_rect(&self, key: &str) -> Option<ratatui::layout::Rect> {
        self.last_rendered_rects.get(key).cloned()
    }

    pub async fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::MouseEventKind;
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                self.move_down();
                let _ = self.update_preview().await;
            }
            MouseEventKind::ScrollUp => {
                self.move_up();
                let _ = self.update_preview().await;
            }
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                let x = mouse.column;
                let y = mouse.row;

                // 0. Handle Popups First (Highest Priority)
                if let Some(r) = self
                    .show_help_popup
                    .then(|| self.get_rect("popup_help"))
                    .flatten()
                {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        // Help results usually start after search box
                        let inner_y = y
                            .saturating_sub(r.y)
                            .saturating_sub(if self.show_help_filter { 5 } else { 2 });
                        if !self.filtered_help.is_empty() {
                            let idx = (self.help_state.offset() + inner_y as usize)
                                .min(self.filtered_help.len() - 1);
                            self.help_state.select(Some(idx));
                        }
                    } else {
                        self.show_help_popup = false;
                    }
                    let _ = self.update_preview().await;
                    return;
                }

                if let Some(r) = self
                    .show_search
                    .then(|| self.get_rect("popup_search"))
                    .flatten()
                {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        let inner_y = y.saturating_sub(r.y).saturating_sub(5); // search box(3) + margin(2)
                        if !self.search_results.is_empty() {
                            let idx = (self.search_state.offset() + inner_y as usize)
                                .min(self.search_results.len() - 1);
                            self.search_state.select(Some(idx));
                        }
                    } else {
                        self.show_search = false;
                        self.input_mode = false;
                    }
                    let _ = self.update_preview().await;
                    return;
                }

                if let Some(r) = self
                    .show_project_popup
                    .then(|| self.get_rect("popup_project"))
                    .flatten()
                {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                        if !self.projects.is_empty() {
                            let idx = (self.projects_state.offset() + inner_y as usize)
                                .min(self.projects.len() - 1);
                            self.projects_state.select(Some(idx));
                        }
                    } else {
                        self.show_project_popup = false;
                    }
                    let _ = self.update_preview().await;
                    return;
                }

                if let Some(r) = self
                    .show_label_popup
                    .then(|| self.get_rect("popup_label"))
                    .flatten()
                {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                        if !self.project_labels.is_empty() {
                            let idx = (self.label_state.offset() + inner_y as usize)
                                .min(self.project_labels.len() - 1);
                            self.label_state.select(Some(idx));
                        }
                    }
                    let _ = self.update_preview().await;
                    return;
                }

                if let Some(r) = self
                    .show_assignee_popup
                    .then(|| self.get_rect("popup_assignee"))
                    .flatten()
                {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                        if !self.project_members.is_empty() {
                            let idx = (self.assignee_state.offset() + inner_y as usize)
                                .min(self.project_members.len() - 1);
                            self.assignee_state.select(Some(idx));
                        }
                    }
                    let _ = self.update_preview().await;
                    return;
                }

                if let Some(r) = self
                    .show_reviewer_popup
                    .then(|| self.get_rect("popup_reviewer"))
                    .flatten()
                {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                        if !self.project_members.is_empty() {
                            let idx = (self.reviewer_state.offset() + inner_y as usize)
                                .min(self.project_members.len() - 1);
                            self.reviewer_state.select(Some(idx));
                        }
                    }
                    let _ = self.update_preview().await;
                    return;
                }

                if let Some(r) = self
                    .show_milestone_popup
                    .then(|| self.get_rect("popup_milestone"))
                    .flatten()
                {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                        let len = self.project_milestones.len() + 1;
                        let idx = (self.milestone_state.offset() + inner_y as usize).min(len - 1);
                        self.milestone_state.select(Some(idx));
                    }
                    let _ = self.update_preview().await;
                    return;
                }

                if let Some(r) = self
                    .show_target_branch_popup
                    .then(|| self.get_rect("popup_target_branch"))
                    .flatten()
                {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                        if !self.project_branches.is_empty() {
                            let idx = (self.target_branch_state.offset() + inner_y as usize)
                                .min(self.project_branches.len() - 1);
                            self.target_branch_state.select(Some(idx));
                        }
                    }
                    let _ = self.update_preview().await;
                    return;
                }

                if let Some(r) = self
                    .show_snooze_popup
                    .then(|| self.get_rect("popup_snooze"))
                    .flatten()
                {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                        let idx = (inner_y as usize).min(4);
                        self.snooze_state.select(Some(idx));
                    }
                    let _ = self.update_preview().await;
                    return;
                }

                // 1. Check Navigation Panels
                if let Some(r) = self.get_rect("nav_dashboard").filter(|r| {
                    x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1
                }) {
                    self.active_panel = Panel::Dashboard;
                    self.active_context = Context::Dashboard;
                    let inner_y = y.saturating_sub(r.y).saturating_sub(1); // subtract border
                    if inner_y < 3 {
                        self.dashboard_menu_state.select(Some(inner_y as usize));
                        let _ = self.update_preview().await;
                        let _ = self.refresh().await;
                    }
                    return;
                }
                if let Some(r) = self.get_rect("nav_explore").filter(|r| {
                    x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1
                }) {
                    self.active_panel = Panel::Explore;
                    self.active_context = Context::Explore;
                    let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                    if inner_y < 2 {
                        self.explore_menu_state.select(Some(inner_y as usize));
                        let _ = self.update_preview().await;
                        let _ = self.refresh().await;
                    }
                    return;
                }
                if let Some(r) = self.get_rect("nav_project").filter(|r| {
                    x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1
                }) {
                    self.active_panel = Panel::Project;
                    self.active_context = Context::Project;
                    let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                    // Project menu can have up to 8 items
                    if inner_y < 8 {
                        self.project_menu_state.select(Some(inner_y as usize));
                        let _ = self.update_preview().await;
                        let _ = self.refresh().await;
                    }
                    return;
                }

                // 2. Check Main List
                if let Some(r) = self.get_rect("main_list").filter(|r| {
                    x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1
                }) {
                    self.active_panel = Panel::Main;

                    // Check if clicked on Tabs (y = r.y + 1 since border is at r.y)
                    if y == r.y + 1 {
                        let rel_x = x.saturating_sub(r.x).saturating_sub(1);
                        let titles = match self.active_selection {
                            MenuSelection::MergeRequests => {
                                vec!["Opened", "Review", "Assigned", "Merged", "Closed", "All"]
                            }
                            MenuSelection::Issues => vec!["Opened", "Closed", "All"],
                            MenuSelection::Todos => vec!["Pending", "Snoozed", "Done"],
                            _ => vec!["List"],
                        };

                        let mut current_x = 0;
                        for (idx, title) in titles.iter().enumerate() {
                            let next_x = current_x + title.len() + 3; // padding(2) + divider(1)
                            if (rel_x as usize) >= current_x && (rel_x as usize) < next_x {
                                match self.active_selection {
                                    MenuSelection::MergeRequests => {
                                        self.mr_filter = match idx {
                                            0 => MergeRequestFilter::Opened,
                                            1 => MergeRequestFilter::Review,
                                            2 => MergeRequestFilter::Assigned,
                                            3 => MergeRequestFilter::Merged,
                                            4 => MergeRequestFilter::Closed,
                                            _ => MergeRequestFilter::All,
                                        };
                                    }
                                    MenuSelection::Issues => {
                                        self.issue_filter = match idx {
                                            0 => IssueFilter::Opened,
                                            1 => IssueFilter::Closed,
                                            _ => IssueFilter::All,
                                        };
                                    }
                                    MenuSelection::Todos => {
                                        self.todo_filter = match idx {
                                            0 => TodoFilter::Pending,
                                            1 => TodoFilter::Snoozed,
                                            _ => TodoFilter::Done,
                                        };
                                    }
                                    _ => {}
                                }
                                let _ = self.refresh().await;
                                break;
                            }
                            current_x = next_x;
                        }
                    } else if y > r.y + 1 {
                        // Select item in list
                        let inner_y = y.saturating_sub(r.y).saturating_sub(2) as usize;

                        match self.active_selection {
                            MenuSelection::Todos => {
                                let idx = self.todos_state.offset() + inner_y;
                                if idx < self.filtered_todos.len() {
                                    self.todos_state.select(Some(idx));
                                }
                            }
                            MenuSelection::MergeRequests => {
                                let idx = self.mrs_state.offset() + inner_y;
                                if idx < self.filtered_mrs.len() {
                                    self.mrs_state.select(Some(idx));
                                }
                            }
                            MenuSelection::Issues => {
                                let idx = self.issues_state.offset() + inner_y;
                                if idx < self.filtered_issues.len() {
                                    self.issues_state.select(Some(idx));
                                }
                            }
                            MenuSelection::Pipelines => {
                                let idx = self.pipelines_state.offset() + inner_y;
                                if idx < self.pipelines.len() {
                                    self.pipelines_state.select(Some(idx));
                                }
                            }
                            MenuSelection::Projects => {
                                let idx = self.projects_state.offset() + inner_y;
                                if idx < self.filtered_projects.len() {
                                    self.projects_state.select(Some(idx));
                                }
                            }
                            MenuSelection::Groups => {
                                let idx = self.groups_state.offset() + inner_y;
                                if idx < self.filtered_groups.len() {
                                    self.groups_state.select(Some(idx));
                                }
                            }
                            MenuSelection::Branches => {
                                let idx = self.branches_state.offset() + inner_y;
                                if idx < self.project_branches.len() {
                                    self.branches_state.select(Some(idx));
                                }
                            }
                            MenuSelection::Commits => {
                                let idx = self.commits_state.offset() + inner_y;
                                if idx < self.commits.len() {
                                    self.commits_state.select(Some(idx));
                                }
                            }
                            MenuSelection::Boards => {
                                if self.view_mode != ViewMode::Board {
                                    let idx = self.project_boards_state.offset() + inner_y;
                                    if idx < self.project_boards.len() {
                                        self.project_boards_state.select(Some(idx));
                                    }
                                }
                            }
                            MenuSelection::Epics => {
                                let idx = self.epics_state.offset() + inner_y;
                                if idx < self.epics.len() {
                                    self.epics_state.select(Some(idx));
                                }
                            }
                            _ => {}
                        }
                    }
                    let _ = self.update_preview().await;
                    return;
                }

                // 2.1 Check Kanban Columns specifically
                if self.view_mode == ViewMode::Board {
                    for i in 0..self.board_lists.len() {
                        if let Some(r) = self.get_rect(&format!("kanban_col_{}", i)).filter(|r| {
                            x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1
                        }) {
                            self.active_panel = Panel::Main;
                            self.board_lists_state.select(Some(i));
                            let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                            let list_id = self.board_lists[i].id;
                            let issues_len = self
                                .filtered_board_issues
                                .get(&list_id)
                                .map(|v| v.len())
                                .unwrap_or(0);
                            if issues_len > 0 {
                                let state = self.board_issue_states.entry(list_id).or_default();
                                // Heuristic: items are ~4 lines high (Title + Meta + Labels + Spacer)
                                let item_idx =
                                    (state.offset() + inner_y as usize / 4).min(issues_len - 1);
                                state.select(Some(item_idx));
                            }
                            let _ = self.update_preview().await;
                            return;
                        }
                    }
                }

                // 3. Check Detail Panels
                if let Some(r) = self.get_rect("jobs_panel").filter(|r| {
                    x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1
                }) {
                    self.active_panel = Panel::MainDetail;
                    let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                    if (inner_y as usize) < self.jobs.len() {
                        self.jobs_state.select(Some(inner_y as usize));
                    }
                    let _ = self.update_preview().await;
                    return;
                }
                if let Some(r) = self.get_rect("metadata_panel").filter(|r| {
                    x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1
                }) {
                    self.active_panel = Panel::MainDetail;
                    self.detail_section = DetailSection::Metadata;
                    let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                    self.selected_metadata_field = if self.selected_mr.is_some() {
                        match inner_y {
                            0 => MetadataField::Title,
                            1 => MetadataField::TargetBranch,
                            2 => MetadataField::Labels,
                            3 => MetadataField::Assignees,
                            4 => MetadataField::Reviewers,
                            5 => MetadataField::Milestone,
                            _ => MetadataField::Title,
                        }
                    } else {
                        match inner_y {
                            0 => MetadataField::Title,
                            1 => MetadataField::Title, // status line is at row 0, title is row 1
                            2 => MetadataField::Labels,
                            3 => MetadataField::Assignees,
                            4 => MetadataField::Milestone,
                            5 => MetadataField::StartDate,
                            6 => MetadataField::DueDate,
                            7 => MetadataField::Confidential,
                            _ => MetadataField::Title,
                        }
                    };
                    let _ = self.update_preview().await;
                    return;
                }
                if self
                    .get_rect("detail_view")
                    .filter(|r| {
                        x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1
                    })
                    .is_some()
                {
                    self.active_panel = Panel::MainDetail;
                    self.detail_section = DetailSection::Description;
                    let _ = self.update_preview().await;
                    return;
                }
                if let Some(r) = self.get_rect("discussions_panel").filter(|r| {
                    x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1
                }) {
                    self.active_panel = Panel::MainDetail;
                    self.detail_section = DetailSection::Discussions;
                    let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                    if (inner_y as usize) < self.discussions.len() {
                        self.discussion_state.select(Some(inner_y as usize));
                    }
                    let _ = self.update_preview().await;
                }
                if let Some(r) = self
                    .get_rect("detail_tabs")
                    .filter(|r| x >= r.x && x < r.x + r.width && y == r.y)
                {
                    self.active_panel = Panel::MainDetail;
                    let has_changes = self.selected_mr.is_some();
                    let mut titles = vec![
                        " Description ".to_string(),
                        format!(" Discussions ({}) ", self.discussions.len()),
                    ];
                    if has_changes {
                        titles.push(format!(" Changes ({}) ", self.mr_diffs.len()));
                    }

                    let rel_x = x.saturating_sub(r.x) as usize;
                    let mut current_x = 0;
                    for (idx, title) in titles.iter().enumerate() {
                        let tab_width = title.len();
                        if rel_x >= current_x && rel_x < current_x + tab_width {
                            self.detail_section = match idx {
                                0 => DetailSection::Description,
                                1 => DetailSection::Discussions,
                                2 => DetailSection::Changes,
                                _ => DetailSection::Description,
                            };
                            let _ = self.update_preview().await;
                            break;
                        }
                        current_x += tab_width + 3;
                    }
                    return;
                }
                if let Some(r) = self.get_rect("changes_panel").filter(|r| {
                    x > r.x && x < r.x + r.width - 1 && y >= r.y && y < r.y + r.height - 1
                }) {
                    self.active_panel = Panel::MainDetail;
                    self.detail_section = DetailSection::Changes;
                    let inner_y = y.saturating_sub(r.y) as usize;
                    let idx = self.mr_diff_state.offset() + inner_y;
                    if idx < self.mr_diffs.len() {
                        self.mr_diff_state.select(Some(idx));
                    }
                    let _ = self.update_preview().await;
                }
            }
            _ => {}
        }
    }

    pub async fn move_board_issue(
        &mut self,
        direction: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.view_mode != ViewMode::Board {
            return Ok(());
        }
        let current_list_idx = if let Some(i) = self.board_lists_state.selected() {
            i
        } else {
            return Ok(());
        };
        let target_list_idx = if direction > 0 {
            current_list_idx + 1
        } else {
            current_list_idx.saturating_sub(1)
        };

        if target_list_idx >= self.board_lists.len() || target_list_idx == current_list_idx {
            return Ok(());
        }

        let from_list = self.board_lists[current_list_idx].clone();
        let to_list = self.board_lists[target_list_idx].clone();

        let issue_idx = if let Some(state) = self.board_issue_states.get(&from_list.id) {
            if let Some(i) = state.selected() {
                i
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        let issue = if let Some(issues) = self.filtered_board_issues.get(&from_list.id) {
            if let Some(i) = issues.get(issue_idx) {
                i.clone()
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        let project_id = issue.project_id;
        let issue_iid = issue.iid;

        let mut add_labels = Vec::new();
        let mut remove_labels = Vec::new();
        let mut state_event = None;

        if let Some(l) = &from_list.label {
            remove_labels.push(l.name.clone());
        }
        if from_list.list_type == "closed" {
            state_event = Some("reopen".to_string());
        }

        if let Some(l) = &to_list.label {
            add_labels.push(l.name.clone());
        }
        if to_list.list_type == "closed" {
            state_event = Some("close".to_string());
        }

        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));

        let add_l_str = if add_labels.is_empty() {
            None
        } else {
            Some(add_labels.join(","))
        };
        let rem_l_str = if remove_labels.is_empty() {
            None
        } else {
            Some(remove_labels.join(","))
        };

        tokio::spawn(async move {
            let endpoint = crate::api::EditIssue {
                project: project_id,
                issue: issue_iid,
                title: None,
                description: None,
                state_event,
                labels: None,
                add_labels: add_l_str,
                remove_labels: rem_l_str,
                assignee_ids: None,
                milestone_id: None,
                due_date: None,
                start_date: None,
                weight: None,
                confidential: None,
            };

            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                endpoint.query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(_)) => {
                    let _ = tx.send(DataUpdate::TriggerRefresh);
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Move issue failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Move issue timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });

        // Optimistic local update
        if let Some(from_issues) = self.board_issues.get_mut(&from_list.id)
            && let Some(pos) = from_issues.iter().position(|i| i.iid == issue_iid)
        {
            let mut moved = from_issues.remove(pos);
            if let Some(l) = &from_list.label {
                moved.labels.retain(|label| label.name != l.name);
            }
            if let Some(l) = &to_list.label
                && !moved.labels.iter().any(|label| label.name == l.name)
            {
                moved.labels.push(l.clone());
            }
            if to_list.list_type == "closed" {
                moved.state = "closed".to_string();
            } else if from_list.list_type == "closed" {
                moved.state = "opened".to_string();
            }
            let to_issues = self.board_issues.entry(to_list.id).or_default();
            to_issues.push(moved);
        }
        self.filter_lists();
        if let Some(state) = self.board_issue_states.get_mut(&from_list.id) {
            let remaining = self
                .filtered_board_issues
                .get(&from_list.id)
                .map(|v| v.len())
                .unwrap_or(0);
            if remaining == 0 {
                state.select(None);
            } else if let Some(cur) = state.selected() {
                state.select(Some(cur.min(remaining - 1)));
            }
        }
        let to_count = self
            .filtered_board_issues
            .get(&to_list.id)
            .map(|v| v.len())
            .unwrap_or(0);
        if to_count > 0 {
            self.board_issue_states
                .entry(to_list.id)
                .or_default()
                .select(Some(to_count - 1));
        }
        self.board_lists_state.select(Some(target_list_idx));
        Ok(())
    }

    pub fn open_confirm_merge_popup(&mut self) {
        if let Some(mr) = &self.selected_mr {
            self.merge_squash = mr.squash.unwrap_or(false);
            self.merge_remove_source_branch = true;
            self.show_confirm_merge_popup = true;
        }
    }

    pub async fn back(&mut self) {
        self.view_mode = ViewMode::List;
        self.active_panel = Panel::Dashboard;
    }

    pub fn copy_to_clipboard(&mut self, text: String) {
        if text.is_empty() {
            return;
        }

        // Strip ANSI codes and GitLab noise for a clean shareable copy
        let clean_text = pipelines::strip_ansi(&text);

        // 1. Attempt local system clipboard (works on local machine)
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(clean_text.clone());
        }

        // 2. Send OSC 52 sequence via DataUpdate to the main loop
        use base64::{Engine as _, engine::general_purpose};
        let b64 = general_purpose::STANDARD.encode(&clean_text);

        // OSC 52 sequence: ESC ] 52 ; c ; [base64] BEL
        let mut osc52 = format!("\x1b]52;c;{}\x07", b64);

        // If inside tmux, we need a special escape wrapper to pass through to the outer terminal
        if std::env::var_os("TMUX").is_some() {
            osc52 = format!("\x1bPtmux;\x1b\x1b]52;c;{}\x07\x1b\\", b64);
        }

        let _ = self.tx.send(DataUpdate::TerminalEscape(osc52));

        let line_count = clean_text.lines().count();
        if line_count > 1 {
            self.status_message = Some(format!("Copied {} lines to clipboard!", line_count));
        } else {
            self.status_message = Some("Copied to clipboard!".to_string());
        }
    }

    pub fn get_current_board_item_url(&self) -> Option<String> {
        if self.view_mode != ViewMode::Board {
            return None;
        }
        let list = self
            .board_lists_state
            .selected()
            .and_then(|idx| self.board_lists.get(idx))?;
        let state = self.board_issue_states.get(&list.id)?;
        let issue_idx = state.selected()?;
        self.filtered_board_issues
            .get(&list.id)
            .and_then(|v| v.get(issue_idx))
            .map(|i| i.web_url.clone())
    }

    pub fn parse_job_log(&mut self) {
        self.job_log_lines = pipelines::parse_job_log(&self.job_log);
    }
    pub fn update_tree_state(&mut self) {
        // Build items for tui_tree_widget
        // This is a flat list right now, but tui_tree_widget requires TreeItem
        // We will deal with mapping it in the UI, but here we can just ensure state is valid.
    }
    pub async fn check_pagination(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.active_panel != Panel::Main {
            return Ok(());
        }
        match self.active_selection {
            MenuSelection::Todos => {
                if self
                    .todos_state
                    .selected()
                    .is_some_and(|i| i + 10 >= self.filtered_todos.len())
                {
                    self.fetch_next_todos().await?;
                }
            }
            MenuSelection::MergeRequests => {
                if self
                    .mrs_state
                    .selected()
                    .is_some_and(|i| i + 10 >= self.filtered_mrs.len())
                {
                    if self.active_context == Context::Project {
                        self.fetch_next_merge_requests().await?;
                    } else {
                        self.fetch_next_global_merge_requests().await?;
                    }
                }
            }
            MenuSelection::Issues => {
                if self
                    .issues_state
                    .selected()
                    .is_some_and(|i| i + 10 >= self.filtered_issues.len())
                {
                    if self.active_context == Context::Project {
                        self.fetch_next_issues().await?;
                    } else {
                        self.fetch_next_global_issues().await?;
                    }
                }
            }
            MenuSelection::Pipelines => {
                if self
                    .pipelines_state
                    .selected()
                    .is_some_and(|i| i + 10 >= self.pipelines.len())
                {
                    self.fetch_next_pipelines().await?;
                }
            }
            MenuSelection::Projects => {
                if self
                    .projects_state
                    .selected()
                    .is_some_and(|i| i + 10 >= self.filtered_projects.len())
                {
                    self.fetch_next_projects().await?;
                }
            }
            MenuSelection::Groups => {
                if self
                    .groups_state
                    .selected()
                    .is_some_and(|i| i + 10 >= self.filtered_groups.len())
                {
                    self.fetch_next_groups().await?;
                }
            }
            MenuSelection::Home
            | MenuSelection::Repository
            | MenuSelection::Branches
            | MenuSelection::Boards
            | MenuSelection::Epics => {}
            MenuSelection::Commits => {
                if self
                    .commits_state
                    .selected()
                    .is_some_and(|i| i + 10 >= self.commits.len())
                {
                    self.fetch_next_commits().await?;
                }
            }
        }
        Ok(())
    }
    pub async fn enter(&mut self) {
        match self.input_target {
            InputTarget::EditTitle | InputTarget::EditDescription => {
                if self.selected_issue.is_some() {
                    let _ = self.update_issue().await;
                } else if self.selected_mr.is_some() {
                    let _ = self.update_mr().await;
                }
            }
            InputTarget::Reply => {
                let _ = self.post_reply().await;
            }
            InputTarget::NewComment | InputTarget::NewFileComment => {
                let _ = self.post_new_comment().await;
            }
            InputTarget::SnoozeUntil => {
                let snooze_until = format!("{}T09:00:00Z", self.calendar_date.format("%Y-%m-%d"));
                let _ = self.snooze_todo(snooze_until).await;
                self.show_calendar_popup = false;
            }
            _ => {}
        }
    }
    pub async fn post_reply(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let body = self.textarea.lines().join("\n");
        if body.is_empty() {
            return Ok(());
        }

        let discussion = self
            .discussion_state
            .selected()
            .and_then(|idx| self.discussions.get(idx));

        if let Some(disc) = discussion {
            let disc_id = disc.id.clone();
            let client = self.gitlab.clone();
            let tx = self.tx.clone();

            if let Some(mr) = &self.selected_mr {
                let pid = mr.project_id;
                let iid = mr.iid;
                let _ = tx.send(DataUpdate::Loading(true));
                tokio::spawn(async move {
                    let endpoint = AddMergeRequestDiscussionNote {
                        project: pid,
                        merge_request: iid,
                        discussion: disc_id,
                        body,
                    };
                    let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                        std::time::Duration::from_secs(30),
                        endpoint.query_async(&client),
                    )
                    .await;
                    match res {
                        Ok(Ok(_)) => {
                            let _ = tx.send(DataUpdate::TriggerRefresh);
                        }
                        Ok(Err(e)) => {
                            let _ = tx.send(DataUpdate::Error(format!("Post reply failed: {}", e)));
                        }
                        Err(_) => {
                            let _ = tx.send(DataUpdate::Error("Post reply timed out".into()));
                        }
                    }
                    let _ = tx.send(DataUpdate::Loading(false));
                });
                self.textarea = ratatui_textarea::TextArea::default();
                self.input_mode = false;
            } else if let Some(issue) = &self.selected_issue {
                let pid = issue.project_id;
                let iid = issue.iid;
                let _ = tx.send(DataUpdate::Loading(true));
                tokio::spawn(async move {
                    let endpoint = AddIssueDiscussionNote {
                        project: pid,
                        issue: iid,
                        discussion: disc_id,
                        body,
                    };
                    let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                        std::time::Duration::from_secs(30),
                        endpoint.query_async(&client),
                    )
                    .await;
                    match res {
                        Ok(Ok(_)) => {
                            let _ = tx.send(DataUpdate::TriggerRefresh);
                        }
                        Ok(Err(e)) => {
                            let _ = tx.send(DataUpdate::Error(format!("Post reply failed: {}", e)));
                        }
                        Err(_) => {
                            let _ = tx.send(DataUpdate::Error("Post reply timed out".into()));
                        }
                    }
                    let _ = tx.send(DataUpdate::Loading(false));
                });
                self.textarea = ratatui_textarea::TextArea::default();
                self.input_mode = false;
            }
        }
        Ok(())
    }
    pub async fn switch_project(&mut self) {
        self.show_project_popup = true;
        let _ = self.fetch_projects().await;
    }
    pub async fn preselect_project(&mut self, spec: &str) -> Result<(), String> {
        use gitlab::api::AsyncQuery;
        let endpoint = gitlab::api::projects::Project::builder()
            .project(spec)
            .build()
            .map_err(|e| format!("invalid project '{spec}': {e}"))?;
        let project: Project = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            endpoint.query_async(&self.gitlab),
        )
        .await
        .map_err(|_| format!("timed out loading project '{spec}'"))?
        .map_err(|e| format!("cannot load project '{spec}': {e}"))?;
        self.selected_project = Some(project);
        self.active_panel = Panel::Project;
        self.active_context = Context::Project;
        self.project_menu_state.select(Some(0));
        self.active_selection = MenuSelection::MergeRequests;
        Ok(())
    }

    pub async fn select_project(&mut self) {
        if let Some(project) = self
            .projects_state
            .selected()
            .and_then(|i| self.projects.get(i))
            .cloned()
        {
            self.selected_project = Some(project);
            self.show_project_popup = false;
            self.active_panel = Panel::Project;
            self.active_context = Context::Project;
            self.project_menu_state.select(Some(0));
            self.active_selection = MenuSelection::MergeRequests;
            self.selected_mr = None;
            self.selected_issue = None;
            self.selected_pipeline = None;
            self.discussions.clear();
            self.jobs.clear();
            self.merge_requests.clear();
            self.issues.clear();
            self.pipelines.clear();
            self.repository_ref = None;
            self.repository_tree.clear();
            self.project_branches.clear();
            self.selected_file_content = None;
            self.file_scroll = 0;
            self.file_highlight_cache = None;
            self.filter_lists();
            let _ = self.refresh().await;
        }
    }
}

pub fn detect_local_git_status() -> Option<LocalGitStatus> {
    if std::env::current_exe()
        .map(|p| {
            p.parent()
                .and_then(|d| d.file_name())
                .is_some_and(|n| n == "deps")
        })
        .unwrap_or(false)
    {
        return None;
    }
    let branch_output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if !branch_output.status.success() {
        return None;
    }
    let branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();
    if branch.is_empty() {
        return None;
    }
    let status_output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    let is_dirty = !String::from_utf8_lossy(&status_output.stdout)
        .trim()
        .is_empty();
    Some(LocalGitStatus { branch, is_dirty })
}

pub fn parse_reactions(reaction_values: Vec<serde_json::Value>) -> Vec<AwardEmoji> {
    let mut reactions = Vec::new();
    for val in reaction_values {
        let name = val
            .get("name")
            .and_then(|v| v.as_str())
            .or_else(|| val.get("emoji").and_then(|v| v.as_str()))
            .or_else(|| val.get("emoji_name").and_then(|v| v.as_str()));
        if let Some(n) = name {
            let display_name = if let Some(emoji) = emojis::get_by_shortcode(n) {
                emoji.as_str().to_string()
            } else {
                n.to_string()
            };
            reactions.push(AwardEmoji { name: display_name });
        }
    }
    reactions
}

pub(crate) fn is_selectable_diff_line(dl: &DiffDisplayLine) -> bool {
    match dl {
        DiffDisplayLine::Discussion(_) => true,
        DiffDisplayLine::Code(_, old, new) => old.unwrap_or(0) > 0 || new.unwrap_or(0) > 0,
    }
}
