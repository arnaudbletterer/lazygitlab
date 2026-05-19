pub mod issues;
pub mod mrs;
pub mod pipelines;
pub mod global;
pub mod boards;
pub mod search;

use gitlab::{AsyncGitlab, api::AsyncQuery};
use crate::models::*;
use crate::api::*;
use ratatui::widgets::ListState;
use crate::ui::Theme;
use chrono::{Local, Timelike};
use regex::Regex;

impl App {
    pub fn new(gitlab: AsyncGitlab, tx: tokio::sync::mpsc::UnboundedSender<DataUpdate>) -> Self {
        let mut dashboard_menu_state = ListState::default(); dashboard_menu_state.select(Some(0));
        let mut explore_menu_state = ListState::default(); explore_menu_state.select(Some(0));
        let mut project_menu_state = ListState::default(); project_menu_state.select(Some(0));

        Self {
            should_quit: false,
            active_context: Context::Dashboard,
            dashboard_menu_state,
            explore_menu_state,
            project_menu_state,
            active_selection: MenuSelection::Todos,
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
            job_log: String::new(),
            job_log_lines: Vec::new(),
            collapsed_sections: std::collections::HashSet::new(),
            job_log_scroll: 0,
            job_log_cursor: 0,
            job_log_selection_start: None,
            job_log_viewport_height: 0,
            follow_log: true,
            discussions: Vec::new(),
            discussion_state: ListState::default(),
            input_mode: false,
            input_target: InputTarget::Reply,
            is_loading: false,
            textarea: ratatui_textarea::TextArea::default(),
            theme: Theme::load(),
            last_refresh: std::time::Instant::now(),
            tx,
            project_labels: Vec::new(),
            project_cache: std::collections::HashMap::new(),
            project_label_cache: std::collections::HashMap::new(),
            project_milestone_cache: std::collections::HashMap::new(),
            project_member_cache: std::collections::HashMap::new(),
            project_variable_cache: std::collections::HashMap::new(),
            project_milestones: Vec::new(),
            project_members: Vec::new(),
            project_variables: Vec::new(),
            collapsed_groups: std::collections::HashSet::new(),
            show_label_popup: false,
            label_state: ListState::default(),
            selected_label_ids: Vec::new(),
            show_assignee_popup: false,
            assignee_state: ListState::default(),
            selected_assignee_ids: Vec::new(),
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
            mrs_page: 1, mrs_has_more: true,
            issues_page: 1, issues_has_more: true,
            pipelines_page: 1, pipelines_has_more: true,
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
            projects_page: 1, projects_has_more: true,
            filtered_projects: Vec::new(),
            groups: Vec::new(),
            groups_state: ListState::default(),
            filtered_groups: Vec::new(),
            groups_page: 1, groups_has_more: true,
            filter_input: String::new(),
            show_filter_input: false,
            todos: Vec::new(),
            todos_state: ListState::default(),
            filtered_todos: Vec::new(),
            todos_page: 1, todos_has_more: true,
            filtered_mrs: Vec::new(),
            filtered_issues: Vec::new(),
            filtered_pipelines: Vec::new(),
            todo_filter: TodoFilter::Pending,
            issue_filter: IssueFilter::Opened,
            mr_filter: MergeRequestFilter::Opened,
            last_error: None,
            status_message: None,
            loading_tasks: 0,
            last_fetch_todos: None,
            last_fetch_mrs: None,
            last_fetch_issues: None,
            last_fetch_projects: None,
            last_fetch_groups: None,
            last_fetch_pipelines: None,
            show_snooze_popup: false,
            snooze_state: ListState::default(),
            current_user: None,
            repository_tree: Vec::new(),
            repository_tree_state: tui_tree_widget::TreeState::default(),
            selected_file_content: None,
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
            show_search: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_state: ListState::default(),
            search_id: 0,
            member_project_ids: std::collections::HashSet::new(),
            is_syncing_membership: false,
            last_input: std::time::Instant::now(),
            syntax_set: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            theme_set: syntect::highlighting::ThemeSet::load_defaults(),
            pending_g_prefix: false,
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
            }
            DataUpdate::SearchResults { data, search_id } => {
                if search_id == self.search_id {
                    self.search_results = data;
                    self.search_state.select(if self.search_results.is_empty() { None } else { Some(0) });
                }
            }
            DataUpdate::AppendSearchResults { data, search_id } => {
                if search_id == self.search_id {
                    let mut existing_keys: std::collections::HashSet<String> = self.search_results.iter().map(|r| r.key()).collect();
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
            DataUpdate::Projects { data, page, has_more } => { 
                for p in &data { 
                    self.project_cache.insert(p.id, p.clone());
                    self.trigger_project_metadata_fetch(p.id); 
                }
                self.projects = data; self.projects_page = page; self.projects_has_more = has_more; self.filter_lists(); 
            }
            DataUpdate::AppendProjects(mut data) => { 
                for p in &data { 
                    self.project_cache.insert(p.id, p.clone());
                    self.trigger_project_metadata_fetch(p.id); 
                }
                self.projects_has_more = data.len() >= 50; self.projects.append(&mut data); self.projects_page += 1; self.filter_lists(); 
            }

            DataUpdate::Todos { data, page, has_more } => { 
                for t in &data { self.project_cache.insert(t.project.id, t.project.clone()); }
                self.todos = data; self.todos_page = page; self.todos_has_more = has_more; self.filter_lists(); 
            }
            DataUpdate::AppendTodos(mut data) => { 
                for t in &data { self.project_cache.insert(t.project.id, t.project.clone()); }
                self.todos_has_more = data.len() >= 50; self.todos.append(&mut data); self.todos_page += 1; self.filter_lists(); 
            }

            DataUpdate::Groups { data, page, has_more } => { self.groups = data; self.groups_page = page; self.groups_has_more = has_more; self.filter_lists(); }
            DataUpdate::AppendGroups(mut data) => { self.groups_has_more = data.len() >= 50; self.groups.append(&mut data); self.groups_page += 1; self.filter_lists(); }

            DataUpdate::GlobalMRs { data, page, has_more } => { 
                let mut data = data;
                for mr in &data { self.trigger_project_metadata_fetch(mr.project_id); }
                for mr in &mut data { self.fix_mr_labels(mr); }
                if self.active_context == Context::Dashboard { 
                    self.merge_requests = data; self.mrs_page = page; self.mrs_has_more = has_more; self.filter_lists(); 
                } 
            }
            DataUpdate::ProjectMRs { project_id, mut data, page, has_more } => { 
                self.trigger_project_metadata_fetch(project_id);
                for mr in &mut data { self.fix_mr_labels(mr); }
                if self.active_context == Context::Project && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { 
                    self.merge_requests = data; self.mrs_page = page; self.mrs_has_more = has_more; self.filter_lists(); 
                } 
            }
            DataUpdate::AppendGlobalMRs(mut data) => { 
                for mr in &data { self.trigger_project_metadata_fetch(mr.project_id); }
                for mr in &mut data { self.fix_mr_labels(mr); }
                if self.active_context == Context::Dashboard { 
                    let existing_keys: std::collections::HashSet<String> = self.merge_requests.iter().map(|m| format!("{}:{}", m.project_id, m.iid)).collect();
                    for item in data {
                        let key = format!("{}:{}", item.project_id, item.iid);
                        if !existing_keys.contains(&key) { self.merge_requests.push(item); }
                    }
                    self.merge_requests.sort_by(|a, b| b.iid.cmp(&a.iid));
                    self.filter_lists(); 
                } 
            }
            DataUpdate::AppendProjectMRs { project_id, mut data } => { 
                self.trigger_project_metadata_fetch(project_id);
                for mr in &mut data { self.fix_mr_labels(mr); }
                if self.active_context == Context::Project && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { 
                    self.mrs_has_more = data.len() >= 50; self.merge_requests.append(&mut data); self.mrs_page += 1; self.filter_lists(); 
                } 
            }
            DataUpdate::ClearGlobalMRs => { if self.active_context == Context::Dashboard { self.merge_requests.clear(); self.filter_lists(); } }

            DataUpdate::GlobalIssues { data, page, has_more } => { 
                let mut data = data;
                for issue in &data { self.trigger_project_metadata_fetch(issue.project_id); }
                for issue in &mut data { self.fix_issue_labels(issue); }
                if self.active_context == Context::Dashboard { 
                    self.issues = data; self.issues_page = page; self.issues_has_more = has_more; self.filter_lists(); 
                } 
            }
            DataUpdate::ProjectIssues { project_id, mut data, page, has_more } => { 
                self.trigger_project_metadata_fetch(project_id);
                for issue in &mut data { self.fix_issue_labels(issue); }
                if self.active_context == Context::Project && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { 
                    self.issues = data; self.issues_page = page; self.issues_has_more = has_more; self.filter_lists(); 
                } 
            }
            DataUpdate::AppendGlobalIssues(mut data) => { 
                for issue in &data { self.trigger_project_metadata_fetch(issue.project_id); }
                for issue in &mut data { self.fix_issue_labels(issue); }
                if self.active_context == Context::Dashboard { 
                    let existing_keys: std::collections::HashSet<String> = self.issues.iter().map(|i| format!("{}:{}", i.project_id, i.iid)).collect();
                    for item in data {
                        let key = format!("{}:{}", item.project_id, item.iid);
                        if !existing_keys.contains(&key) { self.issues.push(item); }
                    }
                    self.issues.sort_by(|a, b| b.iid.cmp(&a.iid));
                    self.issues_has_more = true;
                    self.filter_lists(); 
                } 
            }
            DataUpdate::AppendProjectIssues { project_id, mut data } => { 
                self.trigger_project_metadata_fetch(project_id);
                for issue in &mut data { self.fix_issue_labels(issue); }
                if self.active_context == Context::Project && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { 
                    self.issues_has_more = data.len() >= 50; self.issues.append(&mut data); self.issues_page += 1; self.filter_lists(); 
                } 
            }
            DataUpdate::ClearGlobalIssues => { if self.active_context == Context::Dashboard { self.issues.clear(); self.filter_lists(); } }

            DataUpdate::ProjectPipelines { project_id, data, page, has_more } => { if self.active_context == Context::Project && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.pipelines = data; self.pipelines_page = page; self.pipelines_has_more = has_more; self.filter_lists(); } }
            DataUpdate::AppendProjectPipelines { project_id, mut data } => { if self.active_context == Context::Project && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.pipelines_has_more = data.len() >= 50; self.pipelines.append(&mut data); self.pipelines_page += 1; self.filter_lists(); } }

            DataUpdate::Discussions { project_id, iid, mut data } => { 
                if let Some(selected) = &self.selected_mr { 
                    if selected.iid == iid && selected.project_id == project_id { 
                        // Preserve collapsed state OR default to collapsed for new ones
                        for new_disc in &mut data {
                            if let Some(old_disc) = self.discussions.iter().find(|d| d.id == new_disc.id) {
                                new_disc.collapsed = old_disc.collapsed;
                            } else {
                                new_disc.collapsed = true;
                            }
                        }
                        self.discussions = data; 
                        self.enrich_diffs_with_discussions();
                    } 
                } 
            }
            DataUpdate::IssueDiscussions { project_id, iid, mut data } => { 
                if let Some(selected) = &self.selected_issue { 
                    if selected.iid == iid && selected.project_id == project_id { 
                        // Preserve collapsed state OR default to collapsed for new ones
                        for new_disc in &mut data {
                            if let Some(old_disc) = self.discussions.iter().find(|d| d.id == new_disc.id) {
                                new_disc.collapsed = old_disc.collapsed;
                            } else {
                                new_disc.collapsed = true;
                            }
                        }
                        self.discussions = data; 
                    } 
                } 
            }
            DataUpdate::Jobs { project_id, pipeline_id, data } => { if let Some(selected) = &self.selected_pipeline { if selected.id == pipeline_id && selected.project_id == project_id { self.jobs = data; } } }
            DataUpdate::ProjectVariables { project_id, data } => {
                self.project_variable_cache.insert(project_id, data.clone());
                if self.selected_project.as_ref().map_or(false, |p| p.id == project_id) {
                    self.project_variables = data;
                }
            }
            DataUpdate::Log { project_id, job_id, data } => { if let Some(selected) = &self.selected_job { if selected.id == job_id && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.job_log = data; self.parse_job_log(); } } }
            DataUpdate::SingleDiffFetched { project_id, iid, old_path, new_path, diff } => {
                if let Some(selected) = &self.selected_mr {
                    if selected.iid == iid && selected.project_id == project_id {
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
            }
            DataUpdate::Diffs { project_id, iid, data } => {
                if let Some(selected) = &self.selected_mr {
                    if selected.iid == iid && selected.project_id == project_id {
                        self.mr_diffs = data;
                        self.enrich_diffs_with_discussions();
                    }
                }
            }            DataUpdate::Metadata { project_id, labels, milestones, members } => { 
                self.project_label_cache.insert(project_id, labels.clone());
                self.project_milestone_cache.insert(project_id, milestones.clone());
                self.project_member_cache.insert(project_id, members.clone());

                if self.selected_project.as_ref().map_or(false, |p| p.id == project_id) {
                    self.project_labels = labels.clone(); 
                    self.project_milestones = milestones.clone(); 
                    self.project_members = members.clone(); 
                }
                
                let labels_ref = &labels;
                for mr in &mut self.merge_requests { if mr.project_id == project_id { Self::fix_label_list(labels_ref, &mut mr.labels); } }
                for issue in &mut self.issues { if issue.project_id == project_id { Self::fix_label_list(labels_ref, &mut issue.labels); } }
                if let Some(selected) = &mut self.selected_mr { if selected.project_id == project_id { Self::fix_label_list(labels_ref, &mut selected.labels); } }
                if let Some(selected) = &mut self.selected_issue { if selected.project_id == project_id { Self::fix_label_list(labels_ref, &mut selected.labels); } }
                self.filter_lists();
            }
            DataUpdate::MRDetail(mut mr) => { 
                self.fix_mr_labels(&mut mr);
                if let Some(selected) = &mut self.selected_mr { 
                    if selected.iid == mr.iid && selected.project_id == mr.project_id { 
                        for label in &mut mr.labels {
                            if label.id == 0 || label.color == "#808080" {
                                if let Some(old) = selected.labels.iter().find(|l| l.name == label.name) {
                                    if old.color != "#808080" {
                                        label.id = old.id; label.color = old.color.clone(); label.text_color = old.text_color.clone();
                                    }
                                }
                            }
                        }
                        if !self.input_mode { *selected = mr; } 
                    } 
                } else { self.selected_mr = Some(mr); }
            }
            DataUpdate::IssueDetail(mut issue) => { 
                self.fix_issue_labels(&mut issue);
                if let Some(selected) = &mut self.selected_issue { 
                    if selected.iid == issue.iid && selected.project_id == issue.project_id { 
                        for label in &mut issue.labels {
                            if label.id == 0 || label.color == "#808080" {
                                if let Some(old) = selected.labels.iter().find(|l| l.name == label.name) {
                                    if old.color != "#808080" {
                                        label.id = old.id; label.color = old.color.clone(); label.text_color = old.text_color.clone();
                                    }
                                }
                            }
                        }
                        if !self.input_mode { *selected = issue; } 
                    } 
                } else { self.selected_issue = Some(issue); }
            }
            DataUpdate::Loading(b) => { 
                if b { self.loading_tasks += 1; } 
                else { self.loading_tasks = self.loading_tasks.saturating_sub(1); } 
                self.is_loading = self.loading_tasks > 0;
                if !self.is_loading {
                    if let Some(msg) = &self.status_message {
                        if msg.starts_with("Searching") {
                            if self.search_results.is_empty() && self.show_search {
                                self.status_message = Some("No results found.".into());
                            } else {
                                self.status_message = None;
                            }
                        } else if msg.starts_with("Fetching") || msg.starts_with("Loading") || msg.starts_with("Updating") {
                            self.status_message = None;
                        }
                    }
                }
            }
            DataUpdate::Error(e) => { 
                self.last_error = Some(e); 
                self.loading_tasks = self.loading_tasks.saturating_sub(1); 
                self.is_loading = self.loading_tasks > 0;
                if !self.is_loading {
                    if let Some(msg) = &self.status_message {
                        if msg.starts_with("Fetching") || msg.starts_with("Loading") || msg.starts_with("Updating") || msg.starts_with("Searching") {
                            self.status_message = None;
                        }
                    }
                }
            }
            DataUpdate::TriggerRefresh => { self.refresh().await; }
            DataUpdate::CheckPagination => { let _ = self.check_pagination().await; }
            DataUpdate::CurrentUser(user) => { self.current_user = Some(user); }
            DataUpdate::RepositoryTree { project_id, data } => {
                if self.selected_project.as_ref().map_or(false, |p| p.id == project_id) {
                    self.repository_tree = data;
                    self.update_tree_state();
                }
            }
            DataUpdate::RepositoryFile { project_id, file_path, data } => {
                if self.selected_project.as_ref().map_or(false, |p| p.id == project_id) {
                    if let Some(selected_path) = self.repository_tree_state.selected().last() {
                        if selected_path == &file_path { self.selected_file_content = Some(data); }
                    }
                }
            }
            DataUpdate::Branches { project_id, data, .. } => { if self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.project_branches = data; self.filter_lists(); } }
            DataUpdate::Commits { project_id, data, page, has_more } => { if self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.commits = data; self.commits_page = page; self.commits_has_more = has_more; self.filter_lists(); } }
            DataUpdate::AppendCommits { project_id, mut data } => { if self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.commits_has_more = data.len() >= 50; self.commits.append(&mut data); self.commits_page += 1; self.filter_lists(); } }
            DataUpdate::Boards { project_id, data } => { if self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.project_boards = data; self.project_boards_state.select(if self.project_boards.is_empty() { None } else { Some(0) }); } }
            DataUpdate::BoardLists { project_id, board_id, data } => { if self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { if self.selected_board.as_ref().map_or(false, |b| b.id == board_id) { self.board_lists = data.clone(); self.board_lists_state.select(if self.board_lists.is_empty() { None } else { Some(0) }); let lists_to_fetch = self.board_lists.clone(); for list in lists_to_fetch { let _ = self.fetch_board_list_issues(board_id, list.id); } } } }
            DataUpdate::BoardIssues { project_id, board_id: _, list_id, data } => { if self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.board_issues.insert(list_id, data); } }
            DataUpdate::Epics { group_id: _, data } => { self.epics = data; self.epics_state.select(if self.epics.is_empty() { None } else { Some(0) }); }
            DataUpdate::TerminalEscape(_) => {} // Handled in main loop
        }
    }

    pub async fn process_updates(&mut self, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
        while let Ok(update) = rx.try_recv() {
            self.process_update_single(update).await;
        }
        self.update_preview().await;
    }

    fn fix_label(cache: &mut std::collections::HashMap<u64, Vec<Label>>, project_id: u64, label: &mut Label) {
        // Learn from the label if it has color info
        if label.id != 0 && label.color != "#808080" {
            let entry = cache.entry(project_id).or_default();
            if !entry.iter().any(|l| l.name == label.name) {
                entry.push(label.clone());
            }
        }
        // Fix the label if it is missing info
        if label.id == 0 || label.color == "#808080" {
            if let Some(cached_labels) = cache.get(&project_id) {
                if let Some(matching) = cached_labels.iter().find(|l| l.name == label.name) {
                    label.id = matching.id;
                    label.color = matching.color.clone();
                    label.text_color = matching.text_color.clone();
                }
            }
        }
    }

    fn fix_label_list(labels: &[Label], item_labels: &mut [Label]) {
        for label in item_labels {
            if label.id == 0 || label.color == "#808080" {
                if let Some(matching) = labels.iter().find(|l| l.name == label.name) {
                    label.id = matching.id;
                    label.color = matching.color.clone();
                    label.text_color = matching.text_color.clone();
                }
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
        if parts.len() < 3 { return None; }
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
                s = s.trim_start_matches("./").trim_start_matches("a/").trim_start_matches("b/").trim_start_matches("/").to_string();
                if s.len() == start { break; }
            }
            s
        };

        for diff in &mut self.mr_diffs {
            if diff.diff.is_empty() {
                diff.collapsed = true;
            }

            if diff.collapsed && diff.diff.is_empty() {
                if diff.display_lines.is_empty() {
                    diff.display_lines.push(DiffDisplayLine::Code("@@ [Diff collapsed by GitLab API. File too large or too many files.] @@".to_string(), None, None));
                    diff.display_lines.push(DiffDisplayLine::Code("@@ Press 'E' or 'e' to force fetch this file's diff from the repository. @@".to_string(), None, None));
                }
                continue;
            }

            let mut display_lines = Vec::new();
            let mut current_old_line = 0;
            let mut current_new_line = 0;
            
            let diff_new_norm = normalize(&diff.new_path);
            let diff_old_norm = normalize(&diff.old_path);
            let diff_new_filename = diff.new_path.split('/').last().unwrap_or("").to_lowercase();

            for line in diff.diff.lines() {
                if line.starts_with("@@") {
                    if let Some((old_start, new_start)) = Self::parse_diff_hunk_header(line) {
                        current_old_line = old_start;
                        current_new_line = new_start;
                    }
                    display_lines.push(DiffDisplayLine::Code(line.to_string(), None, None));
                } else {
                    let old_val = if line.starts_with('+') { None } else { Some(current_old_line) };
                    let new_val = if line.starts_with('-') { None } else { Some(current_new_line) };

                    if line.starts_with('+') { current_new_line += 1; }
                    else if line.starts_with('-') { current_old_line += 1; }
                    else { current_old_line += 1; current_new_line += 1; }

                    display_lines.push(DiffDisplayLine::Code(line.to_string(), old_val, new_val));

                    // Inject discussions
                    for disc in &self.discussions {
                        if let Some(note) = disc.notes.first() {
                            if let Some(pos) = &note.position {
                                let pos_new_norm = pos.new_path.as_deref().map(normalize);
                                let pos_old_norm = pos.old_path.as_deref().map(normalize);
                                let pos_filename = pos.new_path.as_deref().and_then(|p| p.split('/').last()).map(|s| s.to_lowercase());
                                
                                // Aggressive matching: exact normalized path OR filename match if within line range
                                let path_matches = pos_new_norm == Some(diff_new_norm.clone()) 
                                    || pos_old_norm == Some(diff_old_norm.clone())
                                    || (pos_filename == Some(diff_new_filename.clone()) && (pos.new_line == new_val || pos.old_line == old_val));
                                
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
            }
            diff.display_lines = display_lines;
        }
    }

    pub async fn initial_setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_loading = true; self.loading_tasks = 4;
        let c = self.gitlab.clone(); let tx = self.tx.clone();

        let t0 = tx.clone(); let c0 = c.clone();
        tokio::spawn(async move {
            let ep = CurrentUser;
            let res: Result<Author, _> = ep.query_async(&c0).await;
            if let Ok(user) = res { let _ = t0.send(DataUpdate::CurrentUser(user)); }
            let _ = t0.send(DataUpdate::Loading(false));
        });

        let t1 = tx.clone(); let c1 = c.clone(); 
        tokio::spawn(async move { 
            let ep = Todos { page: Some(1), per_page: Some(50), state: Some("pending".to_string()) }; 
            let res: Result<Result<Vec<Todo>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c1)).await;
            match res {
                Ok(Ok(data)) => { let has_more = data.len() == 50; let _ = t1.send(DataUpdate::Todos { data, page: 1, has_more }); }
                Ok(Err(e)) => { let _ = t1.send(DataUpdate::Error(format!("Todos fetch failed: {}", e))); }
                Err(_) => { let _ = t1.send(DataUpdate::Error("Todos fetch timed out".into())); }
            }
            let _ = t1.send(DataUpdate::Loading(false)); 
        });
        let t2 = tx.clone(); let c2 = c.clone(); 
        tokio::spawn(async move { 
            let ep = PagedProjects { membership: true, search: None, page: 1, per_page: 100, simple: Some(true), archived: Some(false), order_by: Some("last_activity_at".into()) }; 
            let res: Result<Result<Vec<Project>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c2)).await;
            match res {
                Ok(Ok(data)) => { 
                    let has_more = data.len() == 100; 
                    let _ = t2.send(DataUpdate::Projects { data, page: 1, has_more }); 
                }
                Ok(Err(e)) => { let _ = t2.send(DataUpdate::Error(format!("Projects fetch failed: {}", e))); }
                Err(_) => { let _ = t2.send(DataUpdate::Error("Projects fetch timed out".into())); }
            }
            let _ = t2.send(DataUpdate::Loading(false)); 
        });
        let t3 = tx.clone(); let c3 = c.clone(); 
        tokio::spawn(async move { 
            let ep = Groups { page: Some(1), per_page: Some(50), min_access_level: Some(10), top_level_only: None }; 
            let res: Result<Result<Vec<Group>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c3)).await;
            match res {
                Ok(Ok(data)) => { let has_more = data.len() == 50; let _ = t3.send(DataUpdate::Groups { data, page: 1, has_more }); }
                Ok(Err(e)) => { let _ = t3.send(DataUpdate::Error(format!("Groups fetch failed: {}", e))); }
                Err(_) => { let _ = t3.send(DataUpdate::Error("Groups fetch timed out".into())); }
            }
            let _ = t3.send(DataUpdate::Loading(false)); 
        });
        Ok(())
    }

    pub fn trigger_project_metadata_fetch(&self, project_id: u64) {
        if self.project_label_cache.contains_key(&project_id) { return; }
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let lp_ep = ProjectLabels { project: project_id };
            let mp_ep = ProjectMilestones { project: project_id };
            let memp_ep = ProjectMembers { project: project_id };

            let lp = gitlab::api::paged(lp_ep, gitlab::api::Pagination::All);
            let mp = gitlab::api::paged(mp_ep, gitlab::api::Pagination::All);
            let memp = gitlab::api::paged(memp_ep, gitlab::api::Pagination::All);

            let (l_res, m_res, mem_res) = tokio::join!(
                tokio::time::timeout(std::time::Duration::from_secs(20), lp.query_async(&client)),
                tokio::time::timeout(std::time::Duration::from_secs(20), mp.query_async(&client)),
                tokio::time::timeout(std::time::Duration::from_secs(20), memp.query_async(&client))
            );

            let labels: Vec<Label> = match l_res { Ok(Ok(data)) => data, _ => Vec::new() };
            let milestones: Vec<Milestone> = match m_res { Ok(Ok(data)) => data, _ => Vec::new() };
            let members: Vec<Author> = match mem_res { Ok(Ok(data)) => data, _ => Vec::new() };

            if !labels.is_empty() || !milestones.is_empty() || !members.is_empty() {
                let _ = tx.send(DataUpdate::Metadata { project_id, labels, milestones, members });
            }
        });
    }

    pub async fn refresh(&mut self) {
        self.last_refresh = std::time::Instant::now();

        if self.current_user.is_none() {
            let c = self.gitlab.clone(); let tx = self.tx.clone();
            tokio::spawn(async move {
                let ep = CurrentUser;
                let res: Result<Author, _> = ep.query_async(&c).await;
                if let Ok(user) = res { let _ = tx.send(DataUpdate::CurrentUser(user)); }
            });
        }

        if self.member_project_ids.is_empty() && !self.is_syncing_membership {
            let _ = self.fetch_membership_project_ids().await;
        }

        match self.active_context {
            Context::Dashboard | Context::Explore => {
                match self.active_selection {
                    MenuSelection::Todos => { let _ = self.fetch_todos().await; }
                    MenuSelection::MergeRequests => { let _ = self.fetch_global_merge_requests().await; }
                    MenuSelection::Issues => { let _ = self.fetch_global_issues().await; }
                    MenuSelection::Projects => { let _ = self.fetch_projects().await; }
                    MenuSelection::Groups => { let _ = self.fetch_groups().await; }
                    _ => {}
                }
            }
            Context::Project => {
                if let Some(_project) = &self.selected_project {
                    match self.active_selection {
                        MenuSelection::MergeRequests => { let _ = self.fetch_merge_requests().await; }
                        MenuSelection::Issues => { let _ = self.fetch_issues().await; }
                        MenuSelection::Pipelines => { let _ = self.fetch_pipelines().await; }
                        MenuSelection::Repository => { let _ = self.fetch_repository_tree().await; }
                        MenuSelection::Branches => { if let Some(p) = &self.selected_project { let _ = self.fetch_branches(p.id).await; } }
                        MenuSelection::Commits => { let _ = self.fetch_commits().await; }
                        MenuSelection::Boards => { 
                            if let Some(board) = self.selected_board.clone() {
                                let _ = self.fetch_board_lists(board.id).await;
                            } else {
                                let _ = self.fetch_boards().await; 
                            }
                        }                        MenuSelection::Epics => { let _ = self.fetch_epics().await; }
                        _ => {}
                    }
                }
            }
        }

        if let Some(mr) = self.selected_mr.clone() { 
            let _ = self.fetch_discussions(mr.project_id, mr.iid).await; 
            self.trigger_project_metadata_fetch(mr.project_id);
        }
        else if let Some(issue) = self.selected_issue.clone() { 
            let _ = self.fetch_issue_discussions(issue.project_id, issue.iid).await; 
            self.trigger_project_metadata_fetch(issue.project_id);
        }
        else if let Some(pipeline) = self.selected_pipeline.clone() { 
            let _ = self.fetch_jobs(pipeline.project_id, pipeline.id).await; 
            self.trigger_project_metadata_fetch(pipeline.project_id);
        }
        else if let Some(project) = &self.selected_project {
            self.trigger_project_metadata_fetch(project.id);
        }
    }

    pub fn toggle_help(&mut self) { self.show_help_popup = !self.show_help_popup; }

    pub async fn update_preview(&mut self) {
        let prev_selection = self.active_selection;
        let prev_context = self.active_context;

        match self.active_panel {
            Panel::Dashboard => {
                self.active_context = Context::Dashboard;
                let idx = self.dashboard_menu_state.selected().unwrap_or(0);
                self.active_selection = match idx { 0 => MenuSelection::Todos, 1 => MenuSelection::MergeRequests, _ => MenuSelection::Issues };
            }
            Panel::Explore => {
                self.active_context = Context::Explore;
                let idx = self.explore_menu_state.selected().unwrap_or(0);
                self.active_selection = match idx { 0 => MenuSelection::Projects, _ => MenuSelection::Groups };
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
                    _ => MenuSelection::Epics 
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
            self.filter_lists();
            self.refresh().await;
        }

        if self.active_panel == Panel::Main || self.active_panel == Panel::MainDetail {
            match self.active_selection {
                MenuSelection::Todos => {
                    if let Some(i) = self.todos_state.selected() {
                        if let Some(GroupedItem::Item(todo)) = self.filtered_todos.get(i) {
                            let re = Regex::new(r"/(?:issues|merge_requests)/(\d+)").unwrap();
                            let iid = re.captures(&todo.target_url).and_then(|cap| cap.get(1)).and_then(|m| m.as_str().parse().ok());

                            if let Some(iid) = iid {
                                if todo.target_type == "MergeRequest" {
                                    if self.selected_mr.as_ref().map_or(true, |m| m.iid != iid || m.project_id != todo.project.id) {
                                        self.selected_issue = None; self.discussions.clear();
                                        self.selected_mr = Some(MergeRequest { iid, project_id: todo.project.id, title: todo.body.clone(), author: Author::new(0, "".into()), web_url: todo.target_url.clone(), state: "opened".into(), reactions: Vec::new(), description: None, references: None, upvotes: None, downvotes: None, labels: Vec::new(), milestone: None, assignees: Vec::new(), created_at: Some("".into()), draft: Some(false), diff_refs: None });
                                        let _ = self.fetch_discussions(todo.project.id, iid).await;
                                    }
                                } else if todo.target_type == "Issue" {
                                    if self.selected_issue.as_ref().map_or(true, |iss| iss.iid != iid || iss.project_id != todo.project.id) {
                                        self.selected_mr = None; self.discussions.clear();
                                        self.selected_issue = Some(Issue { iid, project_id: todo.project.id, title: todo.body.clone(), description: None, author: Author::new(0, "".into()), assignees: Vec::new(), state: "opened".into(), web_url: todo.target_url.clone(), references: None, issue_type: None, labels: Vec::new(), milestone: None, due_date: None, weight: None, task_completion_status: None, confidential: false, created_at: Some("".into()), upvotes: None, downvotes: None, reactions: Vec::new() });
                                        let _ = self.fetch_issue_discussions(todo.project.id, iid).await;
                                    }
                                }
                            }
                        } else { self.selected_mr = None; self.selected_issue = None; self.discussions.clear(); }
                    } else { self.selected_mr = None; self.selected_issue = None; self.discussions.clear(); }
                }
                MenuSelection::MergeRequests => {
                    if let Some(i) = self.mrs_state.selected() {
                        if let Some(GroupedItem::Item(mr)) = self.filtered_mrs.get(i) {
                            if self.selected_mr.as_ref().map_or(true, |prev| prev.iid != mr.iid || prev.project_id != mr.project_id) {
                                self.selected_mr = Some(mr.clone()); self.selected_issue = None; self.discussions.clear(); self.mr_diffs.clear();
                                let (pid, iid) = (mr.project_id, mr.iid);
                                let _ = self.fetch_discussions(pid, iid).await;
                                let _ = self.fetch_mr_diffs(pid, iid).await;
                            } else if let Some(selected) = &mut self.selected_mr {
                                selected.title = mr.title.clone();
                                selected.state = mr.state.clone();
                                selected.labels = mr.labels.clone();
                                selected.milestone = mr.milestone.clone();
                                selected.assignees = mr.assignees.clone();
                            }
                        }
                    }
                }
                MenuSelection::Issues => {
                    if let Some(i) = self.issues_state.selected() {
                        if let Some(GroupedItem::Item(issue)) = self.filtered_issues.get(i) {
                            if self.selected_issue.as_ref().map_or(true, |prev| prev.iid != issue.iid || prev.project_id != issue.project_id) {
                                self.selected_issue = Some(issue.clone()); self.selected_mr = None; self.discussions.clear();
                                let _ = self.fetch_issue_discussions(issue.project_id, issue.iid).await;
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
                }
                MenuSelection::Pipelines => {
                    if let Some(i) = self.pipelines_state.selected() {
                        if let Some(p) = self.pipelines.get(i) {
                            if self.selected_pipeline.as_ref().map_or(true, |prev| prev.id != p.id) {
                                self.selected_pipeline = Some(p.clone()); self.jobs.clear();
                                let _ = self.fetch_jobs(p.project_id, p.id).await;
                            }
                        }
                    }
                }
                MenuSelection::Boards => {
                    if let Some(i) = self.project_boards_state.selected() {
                        if let Some(board) = self.project_boards.get(i).cloned() {
                            if self.selected_board.as_ref().map_or(true, |b| b.id != board.id) {
                                self.selected_board = Some(board.clone());
                                self.board_lists.clear();
                                self.board_issues.clear();
                                let _ = self.fetch_board_lists(board.id).await;
                            }
                        }
                    }
                }
                _ => {}
            }
        } else {
            self.selected_mr = None; self.selected_issue = None; self.selected_pipeline = None;
            self.discussions.clear(); self.jobs.clear();
        }

        // Stability check: If we are in MainDetail but no item is actually selected (e.g. list empty), move back to Main
        if self.active_panel == Panel::MainDetail {
            let is_item_selected = self.selected_mr.is_some() || self.selected_issue.is_some() || self.selected_pipeline.is_some() || self.selected_board.is_some();
            if !is_item_selected {
                self.active_panel = Panel::Main;
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.show_project_popup {
            if !self.projects.is_empty() {
                let i = (self.projects_state.selected().unwrap_or(0) + 1) % self.projects.len();
                self.projects_state.select(Some(i));
            }
            return;
        }
        if self.show_pipeline_popup {
            if self.show_branches_dropdown && !self.filtered_branches.is_empty() {
                let i = (self.branches_state.selected().unwrap_or(0) + 1) % self.filtered_branches.len();
                self.branches_state.select(Some(i));
            }
            return;
        }
        match self.active_panel {
            Panel::Dashboard => {
                let current = self.dashboard_menu_state.selected().unwrap_or(0);
                if current >= 2 { self.active_panel = Panel::Explore; self.explore_menu_state.select(Some(0)); }
                else { self.dashboard_menu_state.select(Some(current + 1)); }
            }
            Panel::Explore => {
                let current = self.explore_menu_state.selected().unwrap_or(0);
                if current >= 1 { 
                    if self.selected_project.is_some() {
                        self.active_panel = Panel::Project; self.project_menu_state.select(Some(0)); 
                    } else {
                        self.active_panel = Panel::Dashboard; self.dashboard_menu_state.select(Some(0));
                    }
                }
                else { self.explore_menu_state.select(Some(current + 1)); }
            }
            Panel::Project => {
                let current = self.project_menu_state.selected().unwrap_or(0);
                let max_idx = if let Some(p) = &self.selected_project {
                    if p.namespace.as_ref().map_or(false, |n| n.kind == "group") { 7 } else { 6 }
                } else { 0 };
                if current >= max_idx { self.active_panel = Panel::Dashboard; self.dashboard_menu_state.select(Some(0)); }
                else { self.project_menu_state.select(Some(current + 1)); }
            }
            Panel::Main => {
                match self.active_selection {
                    MenuSelection::Todos => if !self.filtered_todos.is_empty() { let i = (self.todos_state.selected().unwrap_or(0) + 1) % self.filtered_todos.len(); self.todos_state.select(Some(i)); }
                    MenuSelection::MergeRequests => if !self.filtered_mrs.is_empty() { let i = (self.mrs_state.selected().unwrap_or(0) + 1) % self.filtered_mrs.len(); self.mrs_state.select(Some(i)); }
                    MenuSelection::Issues => if !self.filtered_issues.is_empty() { let i = (self.issues_state.selected().unwrap_or(0) + 1) % self.filtered_issues.len(); self.issues_state.select(Some(i)); }
                    MenuSelection::Pipelines => if !self.pipelines.is_empty() { let i = (self.pipelines_state.selected().unwrap_or(0) + 1) % self.pipelines.len(); self.pipelines_state.select(Some(i)); }
                    MenuSelection::Projects => if !self.projects.is_empty() { let i = (self.projects_state.selected().unwrap_or(0) + 1) % self.projects.len(); self.projects_state.select(Some(i)); }
                    MenuSelection::Groups => if !self.groups.is_empty() { let i = (self.groups_state.selected().unwrap_or(0) + 1) % self.groups.len(); self.groups_state.select(Some(i)); }
                    MenuSelection::Repository => { self.repository_tree_state.key_down(); }
                    MenuSelection::Branches => if !self.project_branches.is_empty() { let i = (self.branches_state.selected().unwrap_or(0) + 1) % self.project_branches.len(); self.branches_state.select(Some(i)); }
                    MenuSelection::Commits => if !self.commits.is_empty() { let i = (self.commits_state.selected().unwrap_or(0) + 1) % self.commits.len(); self.commits_state.select(Some(i)); }
                    MenuSelection::Boards => if !self.project_boards.is_empty() { let i = (self.project_boards_state.selected().unwrap_or(0) + 1) % self.project_boards.len(); self.project_boards_state.select(Some(i)); }
                    MenuSelection::Epics => if !self.epics.is_empty() { let i = (self.epics_state.selected().unwrap_or(0) + 1) % self.epics.len(); self.epics_state.select(Some(i)); }
                }
                let _ = self.tx.send(DataUpdate::CheckPagination);
            }
            Panel::MainDetail => {
                if self.active_selection == MenuSelection::Pipelines {
                    if !self.jobs.is_empty() {
                        let i = (self.jobs_state.selected().unwrap_or(0) + 1) % self.jobs.len();
                        self.jobs_state.select(Some(i));
                    }
                    return;
                }
                match self.detail_section {
                    DetailSection::Discussions => {
                        if !self.discussions.is_empty() {
                            let current = self.discussion_state.selected().unwrap_or(0);
                            if current >= self.discussions.len().saturating_sub(1) {
                                self.detail_section = DetailSection::Metadata;
                                self.selected_metadata_field = MetadataField::Title;
                                self.discussion_state.select(None);
                            } else {
                                self.discussion_state.select(Some(current + 1));
                            }
                        } else {
                            self.detail_section = DetailSection::Metadata;
                            self.selected_metadata_field = MetadataField::Title;
                        }
                    }
                    DetailSection::Metadata => {
                        match self.selected_metadata_field {
                            MetadataField::Title => self.selected_metadata_field = MetadataField::Labels,
                            MetadataField::Labels => self.selected_metadata_field = MetadataField::Assignees,
                            MetadataField::Assignees => {
                                if self.selected_mr.is_some() { self.selected_metadata_field = MetadataField::Milestone; }
                                else if self.selected_issue.is_some() { self.selected_metadata_field = MetadataField::Weight; }
                                else { self.detail_section = DetailSection::Description; }
                            }
                            MetadataField::Milestone | MetadataField::Weight => {
                                self.detail_section = DetailSection::Description;
                            }
                            _ => self.detail_section = DetailSection::Description,
                        }
                    }
                    DetailSection::Description => {
                        if self.selected_mr.is_some() {
                            self.detail_section = DetailSection::Changes;
                        } else {
                            self.detail_section = DetailSection::Discussions;
                            if !self.discussions.is_empty() { self.discussion_state.select(Some(0)); }
                        }
                    }
                    DetailSection::Changes => {
                        self.detail_section = DetailSection::Discussions;
                        if !self.discussions.is_empty() { self.discussion_state.select(Some(0)); }
                    }
                }
            }
        }
    }

    pub fn move_up(&mut self) {
        match self.active_panel {
            Panel::Dashboard => {
                let current = self.dashboard_menu_state.selected().unwrap_or(0);
                if current == 0 { 
                    if self.selected_project.is_some() {
                        self.active_panel = Panel::Project; 
                        let max_idx = if let Some(p) = &self.selected_project {
                            if p.namespace.as_ref().map_or(false, |n| n.kind == "group") { 7 } else { 6 }
                        } else { 0 };
                        self.project_menu_state.select(Some(max_idx)); 
                    } else {
                        self.active_panel = Panel::Explore;
                        self.explore_menu_state.select(Some(1));
                    }
                }
                else { self.dashboard_menu_state.select(Some(current - 1)); }
            }
            Panel::Explore => {
                let current = self.explore_menu_state.selected().unwrap_or(0);
                if current == 0 { self.active_panel = Panel::Dashboard; self.dashboard_menu_state.select(Some(2)); }
                else { self.explore_menu_state.select(Some(current - 1)); }
            }
            Panel::Project => {
                let current = self.project_menu_state.selected().unwrap_or(0);
                if current == 0 { self.active_panel = Panel::Explore; self.explore_menu_state.select(Some(1)); }
                else { self.project_menu_state.select(Some(current - 1)); }
            }
            Panel::Main => {
                match self.active_selection {
                    MenuSelection::Todos => if !self.filtered_todos.is_empty() { let i = if self.todos_state.selected().unwrap_or(0) == 0 { self.filtered_todos.len() - 1 } else { self.todos_state.selected().unwrap() - 1 }; self.todos_state.select(Some(i)); }
                    MenuSelection::MergeRequests => if !self.filtered_mrs.is_empty() { let i = if self.mrs_state.selected().unwrap_or(0) == 0 { self.filtered_mrs.len() - 1 } else { self.mrs_state.selected().unwrap() - 1 }; self.mrs_state.select(Some(i)); }
                    MenuSelection::Issues => if !self.filtered_issues.is_empty() { let i = if self.issues_state.selected().unwrap_or(0) == 0 { self.filtered_issues.len() - 1 } else { self.issues_state.selected().unwrap() - 1 }; self.issues_state.select(Some(i)); }
                    MenuSelection::Pipelines => if !self.pipelines.is_empty() { let i = if self.pipelines_state.selected().unwrap_or(0) == 0 { self.pipelines.len() - 1 } else { self.pipelines_state.selected().unwrap() - 1 }; self.pipelines_state.select(Some(i)); }
                    MenuSelection::Projects => if !self.projects.is_empty() { let i = if self.projects_state.selected().unwrap_or(0) == 0 { self.projects.len() - 1 } else { self.projects_state.selected().unwrap() - 1 }; self.projects_state.select(Some(i)); }
                    MenuSelection::Groups => if !self.groups.is_empty() { let i = if self.groups_state.selected().unwrap_or(0) == 0 { self.groups.len() - 1 } else { self.groups_state.selected().unwrap() - 1 }; self.groups_state.select(Some(i)); }
                    MenuSelection::Repository => { self.repository_tree_state.key_up(); }
                    MenuSelection::Branches => if !self.project_branches.is_empty() { let i = if self.branches_state.selected().unwrap_or(0) == 0 { self.project_branches.len() - 1 } else { self.branches_state.selected().unwrap() - 1 }; self.branches_state.select(Some(i)); }
                    MenuSelection::Commits => if !self.commits.is_empty() { let i = if self.commits_state.selected().unwrap_or(0) == 0 { self.commits.len() - 1 } else { self.commits_state.selected().unwrap() - 1 }; self.commits_state.select(Some(i)); }
                    MenuSelection::Boards => if !self.project_boards.is_empty() { let i = if self.project_boards_state.selected().unwrap_or(0) == 0 { self.project_boards.len() - 1 } else { self.project_boards_state.selected().unwrap() - 1 }; self.project_boards_state.select(Some(i)); }
                    MenuSelection::Epics => if !self.epics.is_empty() { let i = if self.epics_state.selected().unwrap_or(0) == 0 { self.epics.len() - 1 } else { self.epics_state.selected().unwrap() - 1 }; self.epics_state.select(Some(i)); }
                }
            }
            Panel::MainDetail => {
                if self.active_selection == MenuSelection::Pipelines {
                    if !self.jobs.is_empty() {
                        let i = if self.jobs_state.selected().unwrap_or(0) == 0 { self.jobs.len() - 1 } else { self.jobs_state.selected().unwrap() - 1 };
                        self.jobs_state.select(Some(i));
                    }
                    return;
                }
                match self.detail_section {
                    DetailSection::Discussions => {
                        let current = self.discussion_state.selected().unwrap_or(0);
                        if current == 0 {
                            if self.selected_mr.is_some() {
                                self.detail_section = DetailSection::Changes;
                            } else {
                                self.detail_section = DetailSection::Description;
                            }
                            self.discussion_state.select(None);
                        } else {
                            self.discussion_state.select(Some(current - 1));
                        }
                    }
                    DetailSection::Changes => {
                        self.detail_section = DetailSection::Description;
                    }
                    DetailSection::Metadata => {
                        match self.selected_metadata_field {
                            MetadataField::Title => {
                                self.detail_section = DetailSection::Discussions;
                                if !self.discussions.is_empty() { self.discussion_state.select(Some(self.discussions.len().saturating_sub(1))); }
                            }
                            MetadataField::Labels => self.selected_metadata_field = MetadataField::Title,
                            MetadataField::Assignees => self.selected_metadata_field = MetadataField::Labels,
                            MetadataField::Milestone | MetadataField::Weight => self.selected_metadata_field = MetadataField::Assignees,
                            _ => self.selected_metadata_field = MetadataField::Title,
                        }
                    }
                    DetailSection::Description => {
                        self.detail_section = DetailSection::Metadata;
                        if self.selected_mr.is_some() { self.selected_metadata_field = MetadataField::Milestone; }
                        else if self.selected_issue.is_some() { self.selected_metadata_field = MetadataField::Weight; }
                        else { self.selected_metadata_field = MetadataField::Assignees; }
                    }
                }
            }
        }
    }

    pub fn toggle_group_collapse(&mut self) -> bool {
        let (scope, idx) = match self.active_selection {
            MenuSelection::Todos => (Some("todos"), self.todos_state.selected()),
            MenuSelection::MergeRequests => (Some("mrs"), self.mrs_state.selected()),
            MenuSelection::Issues => (Some("issues"), self.issues_state.selected()),
            _ => (None, None),
        };
        
        if let (Some(s), Some(i)) = (scope, idx) {
            let header = match self.active_selection {
                MenuSelection::Todos => self.filtered_todos.get(i).and_then(|item| if let GroupedItem::Header(h) = item { Some(h.clone()) } else { None }),
                MenuSelection::MergeRequests => self.filtered_mrs.get(i).and_then(|item| if let GroupedItem::Header(h) = item { Some(h.clone()) } else { None }),
                MenuSelection::Issues => self.filtered_issues.get(i).and_then(|item| if let GroupedItem::Header(h) = item { Some(h.clone()) } else { None }),
                _ => None,
            };

            if let Some(h) = header {
                let key = format!("{}:{}", s, h);
                if self.collapsed_groups.contains(&key) { self.collapsed_groups.remove(&key); }
                else { self.collapsed_groups.insert(key); }
                self.filter_lists();
                return true;
            }
        }
        false
    }

    pub async fn go_forward(&mut self) {
        match self.active_panel {
            Panel::Dashboard | Panel::Explore | Panel::Project => {
                self.active_panel = Panel::Main;
            }
            Panel::Main => {
                if self.toggle_group_collapse() { return; }
                match self.active_selection {
                    MenuSelection::Pipelines => {
                        if let Some(i) = self.pipelines_state.selected() {
                            if let Some(p) = self.filtered_pipelines.get(i) {
                                self.selected_pipeline = Some(p.clone());
                                self.active_panel = Panel::MainDetail;
                                self.detail_section = DetailSection::Metadata;
                                let _ = self.fetch_jobs(p.project_id, p.id).await;
                            }
                        }
                    }
                    MenuSelection::Projects => {
                        if let Some(i) = self.projects_state.selected() {
                            self.selected_project = Some(self.filtered_projects[i].clone());
                            self.active_panel = Panel::Main;
                            self.active_context = Context::Project;
                            self.project_menu_state.select(Some(0));
                            self.active_selection = MenuSelection::MergeRequests;
                            let _ = self.update_preview().await;
                        }
                    }
                    MenuSelection::Repository => {
                        if let Some(selected_path) = self.repository_tree_state.selected().last() {
                            if let Some(item) = self.repository_tree.iter().find(|i| &i.path == selected_path) {
                                if item.r#type == "blob" {
                                    self.view_mode = ViewMode::Tree;
                                    let _ = self.fetch_repository_file(selected_path.clone()).await;
                                } else {
                                    self.repository_tree_state.toggle_selected();
                                }
                            }
                        }
                    }
                    MenuSelection::Boards => {
                        if let Some(i) = self.project_boards_state.selected() {
                            if let Some(board) = self.project_boards.get(i).cloned() {
                                self.selected_board = Some(board.clone());
                                self.view_mode = ViewMode::Board;
                                self.board_lists.clear();
                                self.board_issues.clear();
                                let _ = self.fetch_board_lists(board.id).await;
                            }
                        }
                    }
                    MenuSelection::MergeRequests if self.selected_mr.is_some() => {
                        self.view_mode = ViewMode::Diff;
                        self.mr_diff_state.select(Some(0));
                        let (pid, iid) = { let m = self.selected_mr.as_ref().unwrap(); (m.project_id, m.iid) };
                        let _ = self.fetch_mr_diffs(pid, iid).await;
                    }
                    _ => {
                        self.active_panel = Panel::MainDetail;
                        self.detail_section = DetailSection::Metadata;
                        self.selected_metadata_field = MetadataField::Title;
                    }
                }
            }
            Panel::MainDetail => {
                // Enter specialized views from Detail
                if self.detail_section == DetailSection::Changes && self.selected_mr.is_some() {
                    self.view_mode = ViewMode::Diff;
                    self.mr_diff_state.select(Some(0));
                    let (pid, iid) = { let m = self.selected_mr.as_ref().unwrap(); (m.project_id, m.iid) };
                    let _ = self.fetch_mr_diffs(pid, iid).await;
                } else if self.active_selection == MenuSelection::Pipelines {
                    if let Some(i) = self.jobs_state.selected() {
                        if let Some(j) = self.jobs.get(i).cloned() {
                            self.selected_job = Some(j.clone());
                            self.view_mode = ViewMode::JobLog;
                            self.job_log_scroll = 0;
                            let _ = self.fetch_job_log(self.selected_project.as_ref().unwrap().id, j.id).await;
                        }
                    }
                } else {
                    self.enter_detail_action().await;
                }
            }
        }
        let _ = self.update_preview().await;
    }

    pub async fn go_back(&mut self) {
        match self.active_panel {
            Panel::MainDetail => {
                self.active_panel = Panel::Main;
            }
            Panel::Main => {
                self.active_panel = match self.active_context {
                    Context::Dashboard => Panel::Dashboard,
                    Context::Explore => Panel::Explore,
                    Context::Project => Panel::Project,
                };
            }
            _ => {}
        }
        let _ = self.update_preview().await;
    }

    pub fn get_available_actions(&self) -> Vec<HelpItem> {
        let mut items = Vec::new();
        items.push(HelpItem { shortcut: "q".into(), description: "Quit application".into(), action: HelpAction::Quit, category: "Global".into() });
        items.push(HelpItem { shortcut: "?".into(), description: "Show/hide help".into(), action: HelpAction::ToggleHelp, category: "Global".into() });
        items.push(HelpItem { shortcut: "r".into(), description: "Refresh current view".into(), action: HelpAction::Refresh, category: "Global".into() });
        items.push(HelpItem { shortcut: "o".into(), description: "Open in browser".into(), action: HelpAction::OpenBrowser, category: "Global".into() });
        items.push(HelpItem { shortcut: "y".into(), description: "Copy URL/Content to clipboard".into(), action: HelpAction::CopyUrl, category: "Global".into() });
        if self.view_mode == ViewMode::Diff {
            items.push(HelpItem { shortcut: "e / E".into(), description: "Expand collapsed diff file".into(), action: HelpAction::ExpandCollapsedDiff, category: "Diff".into() });
            items.push(HelpItem { shortcut: "Shift + E".into(), description: "Fetch all missing diffs".into(), action: HelpAction::FetchAllMissingDiffs, category: "Diff".into() });
        }
        items.push(HelpItem { shortcut: "/".into(), description: "Global Search".into(), action: HelpAction::Search, category: "Global".into() });
        items.push(HelpItem { shortcut: "h/j/k/l".into(), description: "Navigate panels and items".into(), action: HelpAction::None, category: "Navigation".into() });
        items.push(HelpItem { shortcut: "Enter".into(), description: "Select / Open Detail".into(), action: HelpAction::Select, category: "Navigation".into() });
        items.push(HelpItem { shortcut: "Esc".into(), description: "Go back / Close panel".into(), action: HelpAction::Back, category: "Navigation".into() });
        items.push(HelpItem { shortcut: "Tab".into(), description: "Next Panel".into(), action: HelpAction::ToggleFocus, category: "Navigation".into() });
        items.push(HelpItem { shortcut: "g t".into(), description: "Go to Todos".into(), action: HelpAction::GotoTodos, category: "Jump".into() });
        items.push(HelpItem { shortcut: "g m".into(), description: "Go to Merge Requests".into(), action: HelpAction::GotoMRs, category: "Jump".into() });
        items.push(HelpItem { shortcut: "g i".into(), description: "Go to Issues".into(), action: HelpAction::GotoIssues, category: "Jump".into() });
        items.push(HelpItem { shortcut: "g p".into(), description: "Go to Pipelines".into(), action: HelpAction::GotoPipelines, category: "Jump".into() });
        if self.view_mode == ViewMode::List {
            items.push(HelpItem { shortcut: "f".into(), description: "Filter current list".into(), action: HelpAction::Filter, category: "List".into() });
        }
        if self.view_mode == ViewMode::Board {
            items.push(HelpItem { shortcut: "Shift + H/L".into(), description: "Move issue between columns".into(), action: HelpAction::None, category: "Kanban".into() });
        }
        items
    }

    pub async fn execute_help_action(&mut self, action: HelpAction) {
        match action {
            HelpAction::Quit => self.should_quit = true,
            HelpAction::ToggleHelp => {
                self.show_help_popup = !self.show_help_popup;
                if self.show_help_popup {
                    self.help_filter_input.clear();
                    self.show_help_filter = false;
                    self.filtered_help = self.get_available_actions();
                    self.help_state.select(Some(0));
                }
                let _ = self.update_preview().await;
                return; // Early return so we don't hide it
            }
            HelpAction::Back => { let _ = self.go_back().await; }
            HelpAction::Select => { let _ = self.go_forward().await; }
            HelpAction::ToggleFocus => {
                self.active_panel = match self.active_panel {
                    Panel::Dashboard | Panel::Explore | Panel::Project => Panel::Main,
                    Panel::Main => Panel::MainDetail,
                    Panel::MainDetail => match self.active_context { Context::Dashboard => Panel::Dashboard, Context::Explore => Panel::Explore, Context::Project => Panel::Project },
                };
            }
            HelpAction::Filter => { self.input_mode = true; self.input_target = InputTarget::Filter; self.show_filter_input = true; }
            HelpAction::Search => { self.show_search = true; self.input_mode = true; self.input_target = InputTarget::Search; self.search_query.clear(); }
            HelpAction::Refresh => { let _ = self.refresh().await; }
            HelpAction::OpenBrowser => self.open_in_browser(),
            HelpAction::CopyUrl => self.copy_url(),
            HelpAction::GotoTodos => { self.active_context = Context::Dashboard; self.active_selection = MenuSelection::Todos; self.active_panel = Panel::Main; let _ = self.refresh().await; }
            HelpAction::GotoMRs => { self.active_context = Context::Project; self.active_selection = MenuSelection::MergeRequests; self.active_panel = Panel::Main; let _ = self.refresh().await; }
            HelpAction::GotoIssues => { self.active_context = Context::Project; self.active_selection = MenuSelection::Issues; self.active_panel = Panel::Main; let _ = self.refresh().await; }
            _ => {}
        }
        self.show_help_popup = false;
        let _ = self.update_preview().await;
    }

    pub fn filter_help(&mut self) {
        let all = self.get_available_actions();
        if self.help_filter_input.is_empty() { self.filtered_help = all; }
        else {
            let filter = self.help_filter_input.to_lowercase();
            self.filtered_help = all.into_iter().filter(|item| item.description.to_lowercase().contains(&filter) || item.shortcut.to_lowercase().contains(&filter) || item.category.to_lowercase().contains(&filter)).collect();
        }
        if self.filtered_help.is_empty() {
            self.help_state.select(None);
        } else if self.help_state.selected().is_none() || self.help_state.selected().unwrap() >= self.filtered_help.len() {
            self.help_state.select(Some(0));
        }
    }

    pub async fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        self.last_input = std::time::Instant::now();

        // --- 0. PRE-PROCESSING ---
        let mut processed_key = key;
        if let KeyCode::Char(_) = processed_key.code {
            if processed_key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::ALT) {
                processed_key.modifiers.remove(KeyModifiers::CONTROL | KeyModifiers::ALT);
            }
        }

        // --- 1. POPUP FOCUS (Highest Priority, return early) ---
        
        // 1.1 Help Popup
        if self.show_help_popup {
            if self.show_help_filter {
                match processed_key.code {
                    KeyCode::Esc | KeyCode::Char('/') => { self.show_help_filter = false; }
                    KeyCode::Enter => { if let Some(i) = self.help_state.selected() { if let Some(item) = self.filtered_help.get(i).cloned() { self.execute_help_action(item.action).await; } } }
                    KeyCode::Down => { if !self.filtered_help.is_empty() { let i = (self.help_state.selected().unwrap_or(0) + 1) % self.filtered_help.len(); self.help_state.select(Some(i)); } }
                    KeyCode::Up => { if !self.filtered_help.is_empty() { let i = if self.help_state.selected().unwrap_or(0) == 0 { self.filtered_help.len() - 1 } else { self.help_state.selected().unwrap() - 1 }; self.help_state.select(Some(i)); } }
                    KeyCode::Backspace => { self.help_filter_input.pop(); self.filter_help(); }
                    KeyCode::Char(c) if c != '/' => { self.help_filter_input.push(c); self.filter_help(); }
                    _ => {}
                }
                return;
            }
            if processed_key.code == KeyCode::Esc || processed_key.code == KeyCode::Char('?') { self.show_help_popup = false; return; }
            match processed_key.code {
                KeyCode::Char('/') => { self.show_help_filter = true; }
                KeyCode::Down | KeyCode::Char('j') => { if !self.filtered_help.is_empty() { let i = (self.help_state.selected().unwrap_or(0) + 1) % self.filtered_help.len(); self.help_state.select(Some(i)); } }
                KeyCode::Up | KeyCode::Char('k') => { if !self.filtered_help.is_empty() { let i = if self.help_state.selected().unwrap_or(0) == 0 { self.filtered_help.len() - 1 } else { self.help_state.selected().unwrap() - 1 }; self.help_state.select(Some(i)); } }
                KeyCode::Enter => { if let Some(i) = self.help_state.selected() { if let Some(item) = self.filtered_help.get(i).cloned() { self.execute_help_action(item.action).await; } } }
                _ => {}
            }
            return;
        }

        // 1.2 Calendar Popup (Snooze Date Selection)
        if self.show_calendar_popup {
            use chrono::Duration;
            match processed_key.code {
                KeyCode::Down | KeyCode::Char('j') => { self.calendar_date += Duration::weeks(1); }
                KeyCode::Up | KeyCode::Char('k') => { self.calendar_date -= Duration::weeks(1); }
                KeyCode::Left | KeyCode::Char('h') => { self.calendar_date -= Duration::days(1); }
                KeyCode::Right | KeyCode::Char('l') => { self.calendar_date += Duration::days(1); }
                KeyCode::Enter => {
                    let snooze_until = format!("{}T09:00:00Z", self.calendar_date.format("%Y-%m-%d"));
                    let _ = self.snooze_todo(snooze_until).await;
                    self.show_calendar_popup = false;
                }
                KeyCode::Esc => { self.show_calendar_popup = false; }
                _ => {}
            }
            let _ = self.update_preview().await;
            return;
        }

        // 1.3 Snooze Presets Popup
        if self.show_snooze_popup {
            match processed_key.code {
                KeyCode::Down | KeyCode::Char('j') => { let i = (self.snooze_state.selected().unwrap_or(0) + 1) % 5; self.snooze_state.select(Some(i)); }
                KeyCode::Up | KeyCode::Char('k') => { let i = if self.snooze_state.selected().unwrap_or(0) == 0 { 4 } else { self.snooze_state.selected().unwrap() - 1 }; self.snooze_state.select(Some(i)); }
                KeyCode::Enter => {
                    let selected = self.snooze_state.selected().unwrap_or(0);
                    let snooze_until = match selected {
                        0 => (chrono::Local::now() + chrono::Duration::hours(1)).to_rfc3339(),
                        1 => (chrono::Local::now() + chrono::Duration::days(1)).with_hour(9).unwrap().with_minute(0).unwrap().with_second(0).unwrap().to_rfc3339(),
                        2 => (chrono::Local::now() + chrono::Duration::days(3)).with_hour(9).unwrap().with_minute(0).unwrap().with_second(0).unwrap().to_rfc3339(),
                        3 => (chrono::Local::now() + chrono::Duration::weeks(1)).with_hour(9).unwrap().with_minute(0).unwrap().with_second(0).unwrap().to_rfc3339(),
                        4 => { self.show_snooze_popup = false; self.show_calendar_popup = true; self.input_target = InputTarget::SnoozeUntil; return; }
                        _ => (chrono::Local::now() + chrono::Duration::hours(1)).to_rfc3339(),
                    };
                    let _ = self.snooze_todo(snooze_until).await;
                    self.show_snooze_popup = false;
                }
                KeyCode::Esc => { self.show_snooze_popup = false; }
                _ => {}
            }
            let _ = self.update_preview().await;
            return;
        }

        // 1.4 Global Search Popup
        if self.show_search {
            if self.input_target == InputTarget::Search && self.input_mode {
                match processed_key.code {
                    KeyCode::Esc => { self.input_mode = false; self.show_search = false; self.search_query.clear(); self.search_results.clear(); }
                    KeyCode::Enter => { self.input_mode = false; }
                    KeyCode::Char(c) => { self.search_query.push(c); let _ = self.trigger_global_search().await; }
                    KeyCode::Backspace => { self.search_query.pop(); let _ = self.trigger_global_search().await; }
                    _ => {}
                }
                return;
            }
            match processed_key.code {
                KeyCode::Esc => { self.show_search = false; self.search_results.clear(); }
                KeyCode::Down | KeyCode::Char('j') => { if !self.search_results.is_empty() { let i = (self.search_state.selected().unwrap_or(0) + 1) % self.search_results.len(); self.search_state.select(Some(i)); } }
                KeyCode::Up | KeyCode::Char('k') => { if !self.search_results.is_empty() { let i = if self.search_state.selected().unwrap_or(0) == 0 { self.search_results.len() - 1 } else { self.search_state.selected().unwrap() - 1 }; self.search_state.select(Some(i)); } }
                KeyCode::Enter => {
                    if let Some(idx) = self.search_state.selected() {
                        if let Some(res) = self.search_results.get(idx).cloned() {
                            match res {
                                SearchResult::Project(p) => { self.selected_project = Some(p); self.active_context = Context::Project; self.active_panel = Panel::Project; self.show_search = false; let _ = self.refresh().await; }
                                SearchResult::Issue(i) => { self.selected_issue = Some(i); self.view_mode = ViewMode::Detail; self.show_search = false; }
                                SearchResult::MergeRequest(m) => { self.selected_mr = Some(m); self.view_mode = ViewMode::Detail; self.show_search = false; }
                            }
                        }
                    }
                }
                KeyCode::Char('/') => { self.input_mode = true; self.input_target = InputTarget::Search; }
                _ => {}
            }
            let _ = self.update_preview().await;
            return;
        }

        // 1.5 Selection Popups (Simple List/Selection Popups)
        if self.show_project_popup || self.show_label_popup || self.show_assignee_popup || self.show_milestone_popup || self.show_weight_popup {
            match processed_key.code {
                KeyCode::Esc => { self.show_project_popup = false; self.show_label_popup = false; self.show_assignee_popup = false; self.show_milestone_popup = false; self.show_weight_popup = false; }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.show_project_popup { self.move_down(); }
                    else if self.show_label_popup { let pid = self.selected_mr.as_ref().map(|m| m.project_id).or_else(|| self.selected_issue.as_ref().map(|i| i.project_id)).unwrap_or(0); let labels = self.project_label_cache.get(&pid).unwrap_or(&self.project_labels); if !labels.is_empty() { let i = (self.label_state.selected().unwrap_or(0) + 1) % labels.len(); self.label_state.select(Some(i)); } }
                    else if self.show_assignee_popup { let pid = self.selected_mr.as_ref().map(|m| m.project_id).or_else(|| self.selected_issue.as_ref().map(|i| i.project_id)).unwrap_or(0); let members = self.project_member_cache.get(&pid).unwrap_or(&self.project_members); if !members.is_empty() { let i = (self.assignee_state.selected().unwrap_or(0) + 1) % members.len(); self.assignee_state.select(Some(i)); } }
                    else if self.show_milestone_popup { let pid = self.selected_mr.as_ref().map(|m| m.project_id).or_else(|| self.selected_issue.as_ref().map(|i| i.project_id)).unwrap_or(0); let milestones = self.project_milestone_cache.get(&pid).unwrap_or(&self.project_milestones); let i = (self.milestone_state.selected().unwrap_or(0) + 1) % (milestones.len() + 1); self.milestone_state.select(Some(i)); }
                    else if self.show_weight_popup { let i = (self.weight_state.selected().unwrap_or(0) + 1) % 21; self.weight_state.select(Some(i)); }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.show_project_popup { self.move_up(); }
                    else if self.show_label_popup { let pid = self.selected_mr.as_ref().map(|m| m.project_id).or_else(|| self.selected_issue.as_ref().map(|i| i.project_id)).unwrap_or(0); let labels = self.project_label_cache.get(&pid).unwrap_or(&self.project_labels); if !labels.is_empty() { let i = if self.label_state.selected().unwrap_or(0) == 0 { labels.len() - 1 } else { self.label_state.selected().unwrap() - 1 }; self.label_state.select(Some(i)); } }
                    else if self.show_assignee_popup { let pid = self.selected_mr.as_ref().map(|m| m.project_id).or_else(|| self.selected_issue.as_ref().map(|i| i.project_id)).unwrap_or(0); let members = self.project_member_cache.get(&pid).unwrap_or(&self.project_members); if !members.is_empty() { let i = if self.assignee_state.selected().unwrap_or(0) == 0 { members.len() - 1 } else { self.assignee_state.selected().unwrap() - 1 }; self.assignee_state.select(Some(i)); } }
                    else if self.show_milestone_popup { let pid = self.selected_mr.as_ref().map(|m| m.project_id).or_else(|| self.selected_issue.as_ref().map(|i| i.project_id)).unwrap_or(0); let milestones = self.project_milestone_cache.get(&pid).unwrap_or(&self.project_milestones); let i = if self.milestone_state.selected().unwrap_or(0) == 0 { milestones.len() } else { self.milestone_state.selected().unwrap() - 1 }; self.milestone_state.select(Some(i)); }
                    else if self.show_weight_popup { let i = if self.weight_state.selected().unwrap_or(0) == 0 { 20 } else { self.weight_state.selected().unwrap() - 1 }; self.weight_state.select(Some(i)); }
                }
                KeyCode::Char(' ') => {
                    if self.show_label_popup { let pid = self.selected_mr.as_ref().map(|m| m.project_id).or_else(|| self.selected_issue.as_ref().map(|i| i.project_id)).unwrap_or(0); let labels = self.project_label_cache.get(&pid).unwrap_or(&self.project_labels); if let Some(i) = self.label_state.selected() { if let Some(label) = labels.get(i) { let id = label.id(); if self.selected_label_ids.contains(&id) { self.selected_label_ids.retain(|&x| x != id); } else { self.selected_label_ids.push(id); } } } }
                    else if self.show_assignee_popup { let pid = self.selected_mr.as_ref().map(|m| m.project_id).or_else(|| self.selected_issue.as_ref().map(|i| i.project_id)).unwrap_or(0); let members = self.project_member_cache.get(&pid).unwrap_or(&self.project_members); if let Some(i) = self.assignee_state.selected() { if let Some(member) = members.get(i) { let id = member.id; if self.selected_assignee_ids.contains(&id) { self.selected_assignee_ids.retain(|&x| x != id); } else { self.selected_assignee_ids.push(id); } } } }
                }
                KeyCode::Enter => {
                    if self.show_project_popup { let _ = self.select_project().await; }
                    else if self.show_label_popup { if self.selected_issue.is_some() { let _ = self.update_issue_labels().await; } else if self.selected_mr.is_some() { let _ = self.update_mr_labels().await; } }
                    else if self.show_assignee_popup { if self.selected_issue.is_some() { let _ = self.update_issue_assignees().await; } else if self.selected_mr.is_some() { let _ = self.update_mr_assignees().await; } }
                    else if self.show_milestone_popup { let pid = self.selected_mr.as_ref().map(|m| m.project_id).or_else(|| self.selected_issue.as_ref().map(|i| i.project_id)).unwrap_or(0); let milestones = self.project_milestone_cache.get(&pid).unwrap_or(&self.project_milestones); let selected = self.milestone_state.selected().unwrap_or(0); self.selected_milestone_id = if selected == 0 { None } else { milestones.get(selected - 1).map(|m| m.id) }; if self.selected_issue.is_some() { let _ = self.update_issue_milestone().await; } else if self.selected_mr.is_some() { let _ = self.update_mr_milestone().await; } }
                    else if self.show_weight_popup { let _ = self.update_issue_weight().await; }
                }
                _ => {}
            }
            let _ = self.update_preview().await;
            return;
        }

        // 1.7 Creation Forms (Create Issue/MR)
        if self.show_create_issue_popup || self.show_create_mr_popup {
            match processed_key.code {
                KeyCode::Esc => { self.show_create_issue_popup = false; self.show_create_mr_popup = false; }
                KeyCode::Tab => {
                    if self.show_create_issue_popup {
                        if self.input_target == InputTarget::CreateIssueTitle { self.create_issue_title = self.textarea.lines().join("\n"); self.input_target = InputTarget::CreateIssueDescription; self.textarea = ratatui_textarea::TextArea::from(self.create_issue_description.lines()); }
                        else { self.create_issue_description = self.textarea.lines().join("\n"); self.input_target = InputTarget::CreateIssueTitle; self.textarea = ratatui_textarea::TextArea::from(self.create_issue_title.lines()); }
                    } else {
                        match self.input_target {
                            InputTarget::CreateMRTitle => { self.create_mr_title = self.textarea.lines().join("\n"); self.input_target = InputTarget::CreateMRSourceBranch; self.textarea = ratatui_textarea::TextArea::from(self.create_mr_source_branch.lines()); }
                            InputTarget::CreateMRSourceBranch => { self.create_mr_source_branch = self.textarea.lines().join("\n"); self.input_target = InputTarget::CreateMRTargetBranch; self.textarea = ratatui_textarea::TextArea::from(self.create_mr_target_branch.lines()); }
                            InputTarget::CreateMRTargetBranch => { self.create_mr_target_branch = self.textarea.lines().join("\n"); self.input_target = InputTarget::CreateMRDescription; self.textarea = ratatui_textarea::TextArea::from(self.create_mr_description.lines()); }
                            InputTarget::CreateMRDescription => { self.create_mr_description = self.textarea.lines().join("\n"); self.input_target = InputTarget::CreateMRTitle; self.textarea = ratatui_textarea::TextArea::from(self.create_mr_title.lines()); }
                            _ => {}
                        }
                    }
                    self.textarea.move_cursor(ratatui_textarea::CursorMove::End);
                }
                KeyCode::Enter if processed_key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.show_create_issue_popup {
                        if self.input_target == InputTarget::CreateIssueTitle { self.create_issue_title = self.textarea.lines().join("\n"); } else { self.create_issue_description = self.textarea.lines().join("\n"); }
                        let _ = self.create_issue().await;
                    } else {
                        match self.input_target { InputTarget::CreateMRTitle => self.create_mr_title = self.textarea.lines().join("\n"), InputTarget::CreateMRSourceBranch => self.create_mr_source_branch = self.textarea.lines().join("\n"), InputTarget::CreateMRTargetBranch => self.create_mr_target_branch = self.textarea.lines().join("\n"), InputTarget::CreateMRDescription => self.create_mr_description = self.textarea.lines().join("\n"), _ => {} }
                        let _ = self.create_mr().await;
                    }
                }
                _ => { self.textarea.input(processed_key); }
            }
            let _ = self.update_preview().await;
            return;
        }

        // 1.8 Pipeline Popup
        if self.show_pipeline_popup {
            match processed_key.code {
                KeyCode::Tab => {
                    match self.input_target {
                        InputTarget::PipelineRef => self.input_target = InputTarget::PipelineVariableKey,
                        InputTarget::PipelineVariableKey => self.input_target = InputTarget::PipelineVariableValue,
                        InputTarget::PipelineVariableValue => self.input_target = InputTarget::PipelineVariableList,
                        InputTarget::PipelineVariableList => self.input_target = InputTarget::PipelineRef,
                        _ => self.input_target = InputTarget::PipelineRef,
                    }
                }
                KeyCode::Char('p') if !self.input_mode => { let _ = self.submit_pipeline_launch().await; }
                KeyCode::Enter => {
                    match self.input_target {
                        InputTarget::PipelineRef => {
                             if self.show_branches_dropdown {
                                if let Some(idx) = self.branches_state.selected() { if let Some(b) = self.filtered_branches.get(idx) { self.pipeline_ref_input = b.name.clone(); } }
                                self.show_branches_dropdown = false; self.input_target = InputTarget::PipelineVariableKey;
                             } else { self.input_target = InputTarget::PipelineVariableKey; }
                             self.label_state.select(Some(0));
                        }
                        InputTarget::PipelineVariableKey => {
                            let suggested_vars: Vec<&ProjectVariable> = self.project_variables.iter().filter(|v| v.key.to_lowercase().contains(&self.pipeline_variable_key_input.to_lowercase())).collect();
                            if !suggested_vars.is_empty() { if let Some(idx) = self.label_state.selected() { if let Some(v) = suggested_vars.get(idx) { self.pipeline_variable_key_input = v.key.clone(); } } }
                            self.input_target = InputTarget::PipelineVariableValue;
                        }
                        InputTarget::PipelineVariableValue => {
                            if !self.pipeline_variable_key_input.is_empty() {
                                self.pipeline_variables.push(PipelineVariable { key: self.pipeline_variable_key_input.clone(), value: self.pipeline_variable_value_input.clone() });
                                self.pipeline_variable_key_input.clear(); self.pipeline_variable_value_input.clear(); self.input_target = InputTarget::PipelineVariableKey;
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::Delete | KeyCode::Backspace if self.input_target == InputTarget::PipelineVariableList => { if let Some(idx) = self.pipeline_variables_state.selected() { if idx < self.pipeline_variables.len() { self.pipeline_variables.remove(idx); } } }
                KeyCode::Esc => { self.show_pipeline_popup = false; self.input_mode = false; }
                _ => {
                    match self.input_target {
                        InputTarget::PipelineRef => { match processed_key.code { KeyCode::Char(c) => { self.pipeline_ref_input.push(c); self.show_branches_dropdown = true; self.filter_branches(); } KeyCode::Backspace => { self.pipeline_ref_input.pop(); self.filter_branches(); } KeyCode::Down => { if !self.filtered_branches.is_empty() { let i = (self.branches_state.selected().unwrap_or(0) + 1) % self.filtered_branches.len(); self.branches_state.select(Some(i)); } } KeyCode::Up => { if !self.filtered_branches.is_empty() { let i = (self.branches_state.selected().unwrap_or(0) + self.filtered_branches.len().saturating_sub(1)) % self.filtered_branches.len(); self.branches_state.select(Some(i)); } } _ => {} } }
                        InputTarget::PipelineVariableKey => {
                            let suggested_vars: Vec<&ProjectVariable> = self.project_variables.iter().filter(|v| v.key.to_lowercase().contains(&self.pipeline_variable_key_input.to_lowercase())).collect();
                            match processed_key.code { KeyCode::Char(c) => { self.pipeline_variable_key_input.push(c); self.label_state.select(Some(0)); } KeyCode::Backspace => { self.pipeline_variable_key_input.pop(); self.label_state.select(Some(0)); } KeyCode::Down => { if !suggested_vars.is_empty() { let i = (self.label_state.selected().unwrap_or(0) + 1) % suggested_vars.len(); self.label_state.select(Some(i)); } } KeyCode::Up => { if !suggested_vars.is_empty() { let i = (self.label_state.selected().unwrap_or(0) + suggested_vars.len().saturating_sub(1)) % suggested_vars.len(); self.label_state.select(Some(i)); } } _ => {} }
                        }
                        InputTarget::PipelineVariableValue => { if let KeyCode::Char(c) = processed_key.code { self.pipeline_variable_value_input.push(c); } else if let KeyCode::Backspace = processed_key.code { self.pipeline_variable_value_input.pop(); } }
                        _ => {}
                    }
                }
            }
            let _ = self.update_preview().await;
            return;
        }

        // --- 2. ABSOLUTE HIGH-PRIORITY GLOBAL KEYS (only when not in input mode) ---
        if !self.input_mode {
            let is_question_mark = processed_key.code == KeyCode::Char('?') 
                || (processed_key.code == KeyCode::Char('/') && processed_key.modifiers.contains(KeyModifiers::SHIFT))
                || (processed_key.code == KeyCode::Char(',') && processed_key.modifiers.contains(KeyModifiers::SHIFT));

            if is_question_mark { self.execute_help_action(HelpAction::ToggleHelp).await; return; }
            if processed_key.code == KeyCode::Char('q') { self.should_quit = true; return; }
        }

        // --- 3. G-PREFIX NAVIGATION ---
        if self.pending_g_prefix {
            self.pending_g_prefix = false;
            match processed_key.code {
                KeyCode::Char('t') => { self.active_context = Context::Dashboard; self.active_selection = MenuSelection::Todos; self.active_panel = Panel::Main; self.dashboard_menu_state.select(Some(0)); let _ = self.refresh().await; }
                KeyCode::Char('m') => { if self.active_context == Context::Project { self.active_selection = MenuSelection::MergeRequests; self.project_menu_state.select(Some(0)); } else { self.active_context = Context::Dashboard; self.active_selection = MenuSelection::MergeRequests; self.dashboard_menu_state.select(Some(1)); } self.active_panel = Panel::Main; let _ = self.refresh().await; }
                KeyCode::Char('i') => { if self.active_context == Context::Project { self.active_selection = MenuSelection::Issues; self.project_menu_state.select(Some(1)); } else { self.active_context = Context::Dashboard; self.active_selection = MenuSelection::Issues; self.dashboard_menu_state.select(Some(2)); } self.active_panel = Panel::Main; let _ = self.refresh().await; }
                KeyCode::Char('p') => { if self.selected_project.is_some() { self.active_context = Context::Project; self.active_selection = MenuSelection::Pipelines; self.project_menu_state.select(Some(2)); self.active_panel = Panel::Main; let _ = self.refresh().await; } }
                KeyCode::Char('r') => { if self.selected_project.is_some() { self.active_context = Context::Project; self.active_selection = MenuSelection::Repository; self.project_menu_state.select(Some(3)); self.active_panel = Panel::Main; let _ = self.refresh().await; } }
                _ => {}
            }
            return;
        }

        // --- 4. SPECIALIZED VIEW MODES ---

        // JobLog Mode
        if self.view_mode == ViewMode::JobLog {
            match processed_key.code {
                KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => { self.toggle_job_section(); let _ = self.update_preview().await; return; }
                KeyCode::Esc => { if self.job_log_selection_start.is_some() { self.job_log_selection_start = None; } else { self.view_mode = ViewMode::List; } let _ = self.update_preview().await; return; }
                KeyCode::Left | KeyCode::Char('h') => { self.view_mode = ViewMode::List; let _ = self.update_preview().await; return; }
                KeyCode::Down | KeyCode::Char('j') => { if !self.job_log_lines.is_empty() { let current = self.job_log_cursor; self.job_log_cursor = (self.job_log_cursor + 1).min(self.job_log_lines.len().saturating_sub(1)); if processed_key.modifiers.contains(KeyModifiers::SHIFT) { if self.job_log_selection_start.is_none() { self.job_log_selection_start = Some(current); } } else { self.job_log_selection_start = None; } if self.job_log_cursor >= self.job_log_scroll as usize + self.job_log_viewport_height as usize { self.job_log_scroll = self.job_log_scroll.saturating_add(1); } } let _ = self.update_preview().await; return; }
                KeyCode::Up | KeyCode::Char('k') => { if !self.job_log_lines.is_empty() { let current = self.job_log_cursor; self.job_log_cursor = self.job_log_cursor.saturating_sub(1); if processed_key.modifiers.contains(KeyModifiers::SHIFT) { if self.job_log_selection_start.is_none() { self.job_log_selection_start = Some(current); } } else { self.job_log_selection_start = None; } if (self.job_log_cursor as u16) < self.job_log_scroll { self.job_log_scroll = self.job_log_cursor as u16; } } let _ = self.update_preview().await; return; }
                KeyCode::Char('y') => { let text = if let Some(start) = self.job_log_selection_start { let (s, e) = if start <= self.job_log_cursor { (start, self.job_log_cursor) } else { (self.job_log_cursor, start) }; self.job_log_lines[s..=e].iter().map(|l| l.content.clone()).collect::<Vec<String>>().join("\n") } else if !self.job_log_lines.is_empty() { self.job_log_lines[self.job_log_cursor].content.clone() } else { String::new() }; self.copy_to_clipboard(text); self.job_log_selection_start = None; return; }
                KeyCode::Char('r') | KeyCode::Char('R') => { if let Some(j) = &self.selected_job { if let Some(p) = &self.selected_project { let _ = self.retry_job(p.id, j.id).await; } } return; }
                KeyCode::Char('c') | KeyCode::Char('C') => { if let Some(j) = &self.selected_job { if let Some(p) = &self.selected_project { let _ = self.cancel_job(p.id, j.id).await; } } return; }
                KeyCode::Char('p') | KeyCode::Char('P') => { if let Some(j) = &self.selected_job { if let Some(p) = &self.selected_project { let _ = self.play_job(p.id, j.id).await; } } return; }
                _ => {}
            }
            return;
        }

        // Tree Mode
        if self.view_mode == ViewMode::Tree {
            match processed_key.code {
                KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => { self.view_mode = ViewMode::List; let _ = self.update_preview().await; return; }
                KeyCode::Down | KeyCode::Char('j') => { self.job_log_scroll = self.job_log_scroll.saturating_add(1); let _ = self.update_preview().await; return; }
                KeyCode::Up | KeyCode::Char('k') => { self.job_log_scroll = self.job_log_scroll.saturating_sub(1); let _ = self.update_preview().await; return; }
                KeyCode::Char('y') => { if let Some(file) = &self.selected_file_content { use base64::{Engine as _, engine::general_purpose}; let clean_content = file.content.replace('\n', "").replace('\r', ""); if let Ok(decoded) = general_purpose::STANDARD.decode(&clean_content) { if let Ok(text) = String::from_utf8(decoded) { self.copy_to_clipboard(text); } } } return; }
                _ => {}
            }
            return;
        }

        // Diff Mode
        if self.view_mode == ViewMode::Diff {
            match processed_key.code {
                KeyCode::Esc => { self.view_mode = ViewMode::List; self.diff_selection_start = None; let _ = self.update_preview().await; return; }
                KeyCode::Left | KeyCode::Char('h') => { if self.diff_line_state.selected().is_some() { self.diff_line_state.select(None); self.diff_selection_start = None; } else { self.view_mode = ViewMode::List; } let _ = self.update_preview().await; return; }
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter if self.diff_line_state.selected().is_none() => { self.diff_line_state.select(Some(0)); self.diff_selection_start = None; let _ = self.update_preview().await; return; }
                KeyCode::Down | KeyCode::Char('j') => { if self.diff_line_state.selected().is_some() { if let Some(selected_idx) = self.mr_diff_state.selected() { if let Some(mr_diff) = self.mr_diffs.get(selected_idx) { let line_count = mr_diff.display_lines.len(); if line_count > 0 { let current = self.diff_line_state.selected().unwrap(); let next = (current + 1) % line_count; if processed_key.modifiers.contains(KeyModifiers::SHIFT) { if self.diff_selection_start.is_none() { self.diff_selection_start = Some(current); } } else { self.diff_selection_start = None; } self.diff_line_state.select(Some(next)); } } } } else { let i = (self.mr_diff_state.selected().unwrap_or(0) + 1) % self.mr_diffs.len().max(1); self.mr_diff_state.select(Some(i)); self.diff_line_state.select(None); self.diff_selection_start = None; } let _ = self.update_preview().await; return; }
                KeyCode::Up | KeyCode::Char('k') => { if self.diff_line_state.selected().is_some() { if let Some(selected_idx) = self.mr_diff_state.selected() { if let Some(mr_diff) = self.mr_diffs.get(selected_idx) { let line_count = mr_diff.display_lines.len(); if line_count > 0 { let current = self.diff_line_state.selected().unwrap(); let prev = if current == 0 { line_count - 1 } else { current - 1 }; if processed_key.modifiers.contains(KeyModifiers::SHIFT) { if self.diff_selection_start.is_none() { self.diff_selection_start = Some(current); } } else { self.diff_selection_start = None; } self.diff_line_state.select(Some(prev)); } } } } else { let i = if self.mr_diff_state.selected().unwrap_or(0) == 0 { self.mr_diffs.len().saturating_sub(1) } else { self.mr_diff_state.selected().unwrap() - 1 }; self.mr_diff_state.select(Some(i)); self.diff_line_state.select(None); self.diff_selection_start = None; } let _ = self.update_preview().await; return; }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    if let Some(selected_idx) = self.mr_diff_state.selected() {
                        if let Some(mr_diff) = self.mr_diffs.get(selected_idx).cloned() {
                            if processed_key.modifiers.contains(KeyModifiers::SHIFT) {
                                let _ = self.tx.send(DataUpdate::TerminalEscape("Fetching all missing diffs...".into()));
                                let _ = self.fetch_all_missing_diffs().await;
                            } else if mr_diff.collapsed {
                                if let Some(mr) = &self.selected_mr {
                                    let _ = self.tx.send(DataUpdate::TerminalEscape(format!("Fetching diff for {}...", mr_diff.new_path)));
                                    let _ = self.fetch_single_file_diff(mr.project_id, mr.iid, mr_diff.old_path, mr_diff.new_path).await;
                                }
                            }
                        }
                    }
                    return;
                }
                KeyCode::Char('y') => { if let Some(selected_idx) = self.mr_diff_state.selected() { if let Some(mr_diff) = self.mr_diffs.get(selected_idx) { let text = if let Some(start) = self.diff_selection_start { let (s, e) = if start <= self.diff_line_state.selected().unwrap() { (start, self.diff_line_state.selected().unwrap()) } else { (self.diff_line_state.selected().unwrap(), start) }; mr_diff.display_lines[s..=e].iter().filter_map(|l| match l { DiffDisplayLine::Code(c, ..) => Some(c.clone()), _ => None }).collect::<Vec<_>>().join("\n") } else if let Some(idx) = self.diff_line_state.selected() { match &mr_diff.display_lines[idx] { DiffDisplayLine::Code(c, ..) => c.clone(), _ => String::new() } } else { String::new() }; self.copy_to_clipboard(text); self.diff_selection_start = None; } } return; }
                KeyCode::Char('c') => { if let Some(selected_idx) = self.mr_diff_state.selected() { if let Some(mr_diff) = self.mr_diffs.get(selected_idx) { if let Some(line_idx) = self.diff_line_state.selected() { if let Some(DiffDisplayLine::Code(..)) = mr_diff.display_lines.get(line_idx) { self.input_mode = true; self.input_target = InputTarget::NewComment; self.textarea = ratatui_textarea::TextArea::default(); return; } } } } }
                KeyCode::Char('r') => { if let Some(selected_idx) = self.mr_diff_state.selected() { if let Some(mr_diff) = self.mr_diffs.get(selected_idx) { if let Some(line_idx) = self.diff_line_state.selected() { if let Some(DiffDisplayLine::Discussion(disc)) = mr_diff.display_lines.get(line_idx) { if let Some(pos) = self.discussions.iter().position(|d| d.id == disc.id) { self.discussion_state.select(Some(pos)); self.input_mode = true; self.input_target = InputTarget::Reply; self.textarea = ratatui_textarea::TextArea::default(); return; } } } } } }
                KeyCode::Char(' ') => { if let Some(selected_idx) = self.mr_diff_state.selected() { if let Some(mr_diff) = self.mr_diffs.get_mut(selected_idx) { if let Some(line_idx) = self.diff_line_state.selected() { if let Some(DiffDisplayLine::Discussion(disc)) = mr_diff.display_lines.get_mut(line_idx) { disc.collapsed = !disc.collapsed; if let Some(main_disc) = self.discussions.iter_mut().find(|d| d.id == disc.id) { main_disc.collapsed = disc.collapsed; } } } } } }
                KeyCode::Char('s') => { if let Some(selected_idx) = self.mr_diff_state.selected() { if let Some(mr_diff) = self.mr_diffs.get_mut(selected_idx) { if let Some(line_idx) = self.diff_line_state.selected() { if let Some(DiffDisplayLine::Discussion(disc)) = mr_diff.display_lines.get_mut(line_idx) { disc.collapsed = true; if let Some(main_disc) = self.discussions.iter_mut().find(|d| d.id == disc.id) { main_disc.collapsed = true; if let Some(pos) = self.discussions.iter().position(|d| d.id == disc.id) { self.discussion_state.select(Some(pos)); let _ = self.resolve_discussion().await; } } } } } } }
                KeyCode::Char('n') => { if let Some(selected_idx) = self.mr_diff_state.selected() { if let Some(mr_diff) = self.mr_diffs.get(selected_idx) { let current = self.diff_line_state.selected().unwrap_or(0); for i in (current + 1)..mr_diff.display_lines.len() { if let DiffDisplayLine::Discussion(_) = &mr_diff.display_lines[i] { self.diff_line_state.select(Some(i)); let _ = self.update_preview().await; return; } } } } }
                KeyCode::Char('p') => { if let Some(selected_idx) = self.mr_diff_state.selected() { if let Some(mr_diff) = self.mr_diffs.get(selected_idx) { let current = self.diff_line_state.selected().unwrap_or(0); for i in (0..current).rev() { if let DiffDisplayLine::Discussion(_) = &mr_diff.display_lines[i] { self.diff_line_state.select(Some(i)); let _ = self.update_preview().await; return; } } } } }
                _ => {}
            }
            return;
        }

        // Board Mode
        if self.view_mode == ViewMode::Board {
            match processed_key.code {
                KeyCode::Esc => { self.view_mode = ViewMode::List; self.selected_board = None; return; }
                KeyCode::Left | KeyCode::Char('h') => { let current = self.board_lists_state.selected().unwrap_or(0); let next = if current == 0 { self.board_lists.len().saturating_sub(1) } else { current - 1 }; self.board_lists_state.select(Some(next)); return; }
                KeyCode::Right | KeyCode::Char('l') => { let current = self.board_lists_state.selected().unwrap_or(0); let next = (current + 1) % self.board_lists.len().max(1); self.board_lists_state.select(Some(next)); return; }
                KeyCode::Char('H') => { let _ = self.move_board_issue(-1).await; return; }
                KeyCode::Char('L') => { let _ = self.move_board_issue(1).await; return; }
                KeyCode::Down | KeyCode::Char('j') => { if let Some(list_idx) = self.board_lists_state.selected() { if let Some(list) = self.board_lists.get(list_idx) { let issues = self.board_issues.get(&list.id).map(|v| v.len()).unwrap_or(0); if issues > 0 { let state = self.board_issue_states.entry(list.id).or_default(); let current = state.selected().unwrap_or(0); state.select(Some((current + 1) % issues)); } } } return; }
                KeyCode::Up | KeyCode::Char('k') => { if let Some(list_idx) = self.board_lists_state.selected() { if let Some(list) = self.board_lists.get(list_idx) { let issues = self.board_issues.get(&list.id).map(|v| v.len()).unwrap_or(0); if issues > 0 { let state = self.board_issue_states.entry(list.id).or_default(); let current = state.selected().unwrap_or(0); state.select(Some(if current == 0 { issues - 1 } else { current - 1 })); } } } return; }
                KeyCode::Enter => { if let Some(list_idx) = self.board_lists_state.selected() { if let Some(list) = self.board_lists.get(list_idx) { let state = self.board_issue_states.entry(list.id).or_default(); if let Some(issue_idx) = state.selected() { if let Some(issue) = self.board_issues.get(&list.id).and_then(|v| v.get(issue_idx)).cloned() { self.selected_issue = Some(issue); self.view_mode = ViewMode::Detail; self.detail_section = DetailSection::Metadata; } } } } return; }
                _ => {}
            }
            return;
        }

        // --- 5. INPUT MODES ---

        if self.input_mode {
            if self.input_target == InputTarget::Filter {
                match processed_key.code {
                    KeyCode::Esc | KeyCode::Char('/') => { self.input_mode = false; self.show_filter_input = false; self.filter_input.clear(); self.filter_lists(); }
                    KeyCode::Down | KeyCode::Char('j') => { self.move_down(); }
                    KeyCode::Up | KeyCode::Char('k') => { self.move_up(); }
                    KeyCode::Enter => { self.input_mode = false; self.show_filter_input = false; let _ = self.update_preview().await; let _ = self.go_forward().await; self.filter_input.clear(); self.filter_lists(); }
                    KeyCode::Char(c) if c != '/' => { self.filter_input.push(c); self.filter_lists(); }
                    KeyCode::Backspace => { self.filter_input.pop(); self.filter_lists(); }
                    _ => {}
                }
            } else {
                // Default textarea input modes (EditTitle, EditDescription, Reply, NewComment, SnoozeUntil)
                match processed_key.code {
                    KeyCode::Enter if processed_key.modifiers.contains(KeyModifiers::CONTROL) || processed_key.modifiers.contains(KeyModifiers::SHIFT) => { self.enter().await; }
                    KeyCode::Char('\n') | KeyCode::Char('\r') if processed_key.modifiers.contains(KeyModifiers::CONTROL) => { self.enter().await; }
                    KeyCode::Enter if matches!(self.input_target, InputTarget::EditTitle | InputTarget::SnoozeUntil) => { self.enter().await; }
                    KeyCode::Enter => { self.textarea.input(processed_key); }
                    KeyCode::Esc => { self.input_mode = false; }
                    _ => { self.textarea.input(processed_key); }
                }
            }
            return;
        }

        // --- 6. DEFAULT NAVIGATION (Background) ---

        // Panel-specific keys (like Job Retry in Pipelines)
        if self.active_panel == Panel::MainDetail && self.active_selection == MenuSelection::Pipelines {
            match processed_key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => { if let Some(i) = self.jobs_state.selected() { if let Some(j) = self.jobs.get(i).cloned() { if let Some(p) = &self.selected_project { let _ = self.retry_job(p.id, j.id).await; } } } return; }
                KeyCode::Char('c') | KeyCode::Char('C') => { if let Some(i) = self.jobs_state.selected() { if let Some(j) = self.jobs.get(i).cloned() { if let Some(p) = &self.selected_project { let _ = self.cancel_job(p.id, j.id).await; } } } return; }
                KeyCode::Char('p') | KeyCode::Char('P') => { if let Some(i) = self.jobs_state.selected() { if let Some(j) = self.jobs.get(i).cloned() { if let Some(p) = &self.selected_project { let _ = self.play_job(p.id, j.id).await; } } } return; }
                _ => {}
            }
        }

        match processed_key.code {
            KeyCode::Tab => {
                self.active_panel = match self.active_panel {
                    Panel::Dashboard | Panel::Explore | Panel::Project => Panel::Main,
                    Panel::Main => Panel::MainDetail,
                    Panel::MainDetail => match self.active_context { Context::Dashboard => Panel::Dashboard, Context::Explore => Panel::Explore, Context::Project => Panel::Project },
                };
                let _ = self.update_preview().await;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.active_panel = match self.active_panel {
                    Panel::Dashboard | Panel::Explore | Panel::Project => Panel::Main,
                    Panel::Main => Panel::MainDetail,
                    Panel::MainDetail => Panel::MainDetail,
                };
                let _ = self.update_preview().await;
            }
            KeyCode::BackTab => {
                self.active_panel = match self.active_panel {
                    Panel::Main => match self.active_context { Context::Dashboard => Panel::Dashboard, Context::Explore => Panel::Explore, Context::Project => Panel::Project },
                    Panel::Dashboard | Panel::Explore | Panel::Project => Panel::MainDetail,
                    Panel::MainDetail => Panel::Main,
                };
                let _ = self.update_preview().await;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.active_panel = match self.active_panel {
                    Panel::MainDetail => Panel::Main,
                    Panel::Main => match self.active_context { Context::Dashboard => Panel::Dashboard, Context::Explore => Panel::Explore, Context::Project => Panel::Project },
                    _ => self.active_panel,
                };
                let _ = self.update_preview().await;
            }
            KeyCode::Down | KeyCode::Char('j') => { self.move_down(); let _ = self.update_preview().await; }
            KeyCode::Up | KeyCode::Char('k') => { self.move_up(); let _ = self.update_preview().await; }
            KeyCode::Enter => { let _ = self.update_preview().await; self.go_forward().await; }
            KeyCode::Esc => { self.go_back().await; }            KeyCode::Char('v') => {
                if self.selected_mr.is_some() {
                    self.view_mode = ViewMode::Diff;
                    self.mr_diff_state.select(Some(0));
                    let (pid, iid) = { let m = self.selected_mr.as_ref().unwrap(); (m.project_id, m.iid) };
                    let _ = self.fetch_mr_diffs(pid, iid).await;
                    let _ = self.update_preview().await;
                }
            }
            KeyCode::Char('1') => { 
                if !matches!(self.active_panel, Panel::Dashboard | Panel::Explore | Panel::Project) {
                    self.active_panel = match self.active_context {
                        Context::Dashboard => Panel::Dashboard,
                        Context::Explore => Panel::Explore,
                        Context::Project => Panel::Project,
                    };
                }
                let _ = self.update_preview().await; 
            }
            KeyCode::Char('2') => { self.active_panel = Panel::Main; let _ = self.update_preview().await; }
            KeyCode::Char('3') => { self.active_panel = Panel::MainDetail; let _ = self.update_preview().await; }
            KeyCode::Char('p') => self.switch_project(),
            KeyCode::Char('r') => { let _ = self.refresh().await; }
            KeyCode::Char('f') => {
                if self.active_panel == Panel::Main {
                    self.input_mode = true;
                    self.input_target = InputTarget::Filter;
                    self.show_filter_input = true;
                }
            }
            KeyCode::Char('g') => {
                self.pending_g_prefix = true;
                return;
            }
            KeyCode::Char('/') => {
                self.show_search = true;
                self.input_mode = true;
                self.input_target = InputTarget::Search;
                self.search_query.clear();
                self.search_results.clear();
            }
            KeyCode::Char('[') => {
                match self.active_selection {
                    MenuSelection::Todos => self.todo_filter = match self.todo_filter { TodoFilter::Pending => TodoFilter::Done, TodoFilter::Snoozed => TodoFilter::Pending, TodoFilter::Done => TodoFilter::Snoozed },
                    MenuSelection::MergeRequests => self.mr_filter = match self.mr_filter { MergeRequestFilter::Opened => MergeRequestFilter::All, MergeRequestFilter::Merged => MergeRequestFilter::Opened, MergeRequestFilter::Closed => MergeRequestFilter::Merged, MergeRequestFilter::All => MergeRequestFilter::Closed },
                    MenuSelection::Issues => self.issue_filter = match self.issue_filter { IssueFilter::Opened => IssueFilter::All, IssueFilter::Closed => IssueFilter::Opened, IssueFilter::All => IssueFilter::Closed },
                    _ => {}
                }
                self.merge_requests.clear(); self.issues.clear(); self.todos.clear(); let _ = self.refresh().await;
            }
            KeyCode::Char(']') => {
                match self.active_selection {
                    MenuSelection::Todos => self.todo_filter = match self.todo_filter { TodoFilter::Pending => TodoFilter::Snoozed, TodoFilter::Snoozed => TodoFilter::Done, TodoFilter::Done => TodoFilter::Pending },
                    MenuSelection::MergeRequests => self.mr_filter = match self.mr_filter { MergeRequestFilter::Opened => MergeRequestFilter::Merged, MergeRequestFilter::Merged => MergeRequestFilter::Closed, MergeRequestFilter::Closed => MergeRequestFilter::All, MergeRequestFilter::All => MergeRequestFilter::Opened },
                    MenuSelection::Issues => self.issue_filter = match self.issue_filter { IssueFilter::Opened => IssueFilter::Closed, IssueFilter::Closed => IssueFilter::All, IssueFilter::All => IssueFilter::Opened },
                    _ => {}
                }
                self.merge_requests.clear(); self.issues.clear(); self.todos.clear(); let _ = self.refresh().await;
            }
            KeyCode::Char('c') => { if self.selected_issue.is_some() { let _ = self.toggle_issue_state().await; } else if self.selected_mr.is_some() { let _ = self.toggle_mr_state().await; } }
            KeyCode::Char('A') => { if self.selected_mr.is_some() { let _ = self.approve_mr().await; } }
            KeyCode::Char('C') => {
                if self.active_selection == MenuSelection::Pipelines {
                    if let Some(i) = self.pipelines_state.selected() {
                        if let Some(p) = self.filtered_pipelines.get(i) {
                            let _ = self.cancel_pipeline(p.project_id, p.id).await;
                        }
                    }
                }
            }
            KeyCode::Char('R') => { 
                if self.active_selection == MenuSelection::Pipelines {
                    if let Some(i) = self.pipelines_state.selected() {
                        if let Some(p) = self.filtered_pipelines.get(i) {
                            let _ = self.retry_pipeline(p.project_id, p.id).await;
                        }
                    }
                } else if self.selected_mr.is_some() { 
                    let _ = self.unapprove_mr().await; 
                } 
            }
            KeyCode::Char('D') => { if self.selected_mr.is_some() { let _ = self.toggle_mr_draft().await; } }
            KeyCode::Char('s') => {
                if self.active_selection == MenuSelection::Todos {
                    self.show_snooze_popup = true; self.snooze_state.select(Some(0));
                } else {
                    if self.selected_mr.is_some() || self.selected_issue.is_some() { 
                        if let Some(idx) = self.discussion_state.selected() {
                            if let Some(disc) = self.discussions.get_mut(idx) {
                                disc.collapsed = true;
                            }
                        }
                        let _ = self.resolve_discussion().await; 
                    }
                }
            }
            KeyCode::Char('d') if self.active_selection == MenuSelection::Todos => { let _ = self.mark_todo_as_done().await; }
            KeyCode::Char('n') => {
                if self.show_create_issue_popup || self.show_create_mr_popup || self.show_pipeline_popup || self.show_project_popup || self.show_help_popup || self.show_search { return; }
                match self.active_selection {
                    MenuSelection::Issues => {
                        self.show_create_issue_popup = true;
                        self.input_target = InputTarget::CreateIssueTitle;
                        self.create_issue_title.clear();
                        self.create_issue_description.clear();
                        self.textarea = ratatui_textarea::TextArea::default();
                    }
                    MenuSelection::MergeRequests => {
                        self.show_create_mr_popup = true;
                        self.input_target = InputTarget::CreateMRTitle;
                        self.create_mr_title.clear();
                        self.create_mr_description.clear();
                        self.textarea = ratatui_textarea::TextArea::default();
                        if let Some(p) = &self.selected_project {
                            self.create_mr_target_branch = p.default_branch.clone().unwrap_or_else(|| "main".to_string());
                        }
                    }
                    MenuSelection::Pipelines => {
                        self.show_pipeline_popup = true;
                        self.input_mode = true;
                        self.input_target = InputTarget::PipelineRef;
                        if let Some(p) = &self.selected_project {
                            self.pipeline_ref_input = self.selected_pipeline.as_ref().map(|pipe| pipe.ref_name.clone()).unwrap_or_else(|| p.default_branch.clone().unwrap_or_else(|| "main".to_string()));
                            let pid = p.id;
                            if self.project_variable_cache.contains_key(&pid) {
                                self.project_variables = self.project_variable_cache.get(&pid).cloned().unwrap();
                            } else {
                                let _ = self.fetch_project_variables(pid).await;
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        let _ = self.update_preview().await;
    }

    pub async fn enter_detail_action(&mut self) {
        if self.active_panel == Panel::MainDetail {
            let project_id = if let Some(mr) = &self.selected_mr { Some(mr.project_id) } else if let Some(issue) = &self.selected_issue { Some(issue.project_id) } else { None };
            if let Some(pid) = project_id {
                if let Some(labels) = self.project_label_cache.get(&pid) { self.project_labels = labels.clone(); }
                if let Some(milestones) = self.project_milestone_cache.get(&pid) { self.project_milestones = milestones.clone(); }
                if let Some(members) = self.project_member_cache.get(&pid) { self.project_members = members.clone(); }
            }

            match self.detail_section {
                DetailSection::Metadata => {
                    match self.selected_metadata_field {
                        MetadataField::Title => { self.input_mode = true; self.input_target = InputTarget::EditTitle; let text = if let Some(i) = &self.selected_issue { i.title.clone() } else if let Some(m) = &self.selected_mr { m.title.clone() } else { String::new() }; self.textarea = ratatui_textarea::TextArea::from(text.lines()); self.textarea.move_cursor(ratatui_textarea::CursorMove::End); }
                        MetadataField::Description => { self.input_mode = true; self.input_target = InputTarget::EditDescription; let text = if let Some(i) = &self.selected_issue { i.description.clone().unwrap_or_default() } else if let Some(m) = &self.selected_mr { m.description.clone().unwrap_or_default() } else { String::new() }; self.textarea = ratatui_textarea::TextArea::from(text.lines()); self.textarea.move_cursor(ratatui_textarea::CursorMove::End); }
                        MetadataField::Labels => { self.show_label_popup = true; self.selected_label_ids = if let Some(i) = &self.selected_issue { i.labels.iter().map(|l| l.id()).collect() } else if let Some(m) = &self.selected_mr { m.labels.iter().map(|l| l.id()).collect() } else { Vec::new() }; self.label_state.select(Some(0)); }
                        MetadataField::Assignees => { self.show_assignee_popup = true; self.selected_assignee_ids = if let Some(i) = &self.selected_issue { i.assignees.iter().map(|a| a.id).collect() } else if let Some(m) = &self.selected_mr { m.assignees.iter().map(|a| a.id).collect() } else { Vec::new() }; self.assignee_state.select(Some(0)); }
                        MetadataField::Milestone => { self.show_milestone_popup = true; let mid = if let Some(i) = &self.selected_issue { i.milestone.as_ref().map(|m| m.id) } else if let Some(m) = &self.selected_mr { m.milestone.as_ref().map(|mi| mi.id) } else { None }; let idx = mid.and_then(|id| self.project_milestones.iter().position(|m| m.id == id)).map(|p| p + 1).unwrap_or(0); self.milestone_state.select(Some(idx)); }
                        MetadataField::Weight => { if self.selected_issue.is_some() { self.show_weight_popup = true; let w = self.selected_issue.as_ref().and_then(|i| i.weight).unwrap_or(0); self.weight_state.select(Some(w as usize)); } }
                        _ => {}
                    }
                }
                DetailSection::Discussions => { self.input_mode = true; self.input_target = InputTarget::Reply; self.textarea = ratatui_textarea::TextArea::default(); }
                DetailSection::Description => { self.input_mode = true; self.input_target = InputTarget::EditDescription; let text = if let Some(i) = &self.selected_issue { i.description.clone().unwrap_or_default() } else if let Some(m) = &self.selected_mr { m.description.clone().unwrap_or_default() } else { String::new() }; self.textarea = ratatui_textarea::TextArea::from(text.lines()); self.textarea.move_cursor(ratatui_textarea::CursorMove::End); }
                DetailSection::Changes => {
                    if self.selected_mr.is_some() {
                        self.view_mode = ViewMode::Diff;
                        self.mr_diff_state.select(Some(0));
                        let (pid, iid) = { let m = self.selected_mr.as_ref().unwrap(); (m.project_id, m.iid) };
                        let _ = self.fetch_mr_diffs(pid, iid).await;
                    }
                }
            }
        }
    }


    pub fn filter_lists(&mut self) {
        let filter = self.filter_input.to_lowercase();
        
        // 1. Group Todos
        let mut grouped_todos = Vec::new();
        let mut todos_by_project: std::collections::BTreeMap<String, Vec<Todo>> = std::collections::BTreeMap::new();
        for t in self.todos.iter().filter(|t| t.body.to_lowercase().contains(&filter)) {
            let p_name = t.project.path_with_namespace.as_deref().unwrap_or(&t.project.name).to_string();
            todos_by_project.entry(p_name).or_default().push(t.clone());
        }
        for (p_name, items) in todos_by_project {
            let key = format!("todos:{}", p_name);
            let is_collapsed = self.collapsed_groups.contains(&key);
            grouped_todos.push(GroupedItem::Header(p_name));
            if !is_collapsed { for item in items { grouped_todos.push(GroupedItem::Item(item)); } }
        }
        self.filtered_todos = grouped_todos;

        // 2. Group Merge Requests
        let mut grouped_mrs = Vec::new();
        if self.active_context == Context::Dashboard {
            let mut mrs_by_project: std::collections::BTreeMap<String, Vec<MergeRequest>> = std::collections::BTreeMap::new();
            for mr in self.merge_requests.iter().filter(|m| m.title.to_lowercase().contains(&filter)) {
                let p_name = self.project_cache.get(&mr.project_id).and_then(|p| p.path_with_namespace.as_deref()).unwrap_or("Unknown Project").to_string();
                mrs_by_project.entry(p_name).or_default().push(mr.clone());
            }
            for (p_name, items) in mrs_by_project {
                let key = format!("mrs:{}", p_name);
                let is_collapsed = self.collapsed_groups.contains(&key);
                grouped_mrs.push(GroupedItem::Header(p_name));
                if !is_collapsed { for item in items { grouped_mrs.push(GroupedItem::Item(item)); } }
            }
        } else {
            grouped_mrs = self.merge_requests.iter().filter(|m| m.title.to_lowercase().contains(&filter)).cloned().map(GroupedItem::Item).collect();
        }
        self.filtered_mrs = grouped_mrs;

        // 3. Group Issues
        let mut grouped_issues = Vec::new();
        if self.active_context == Context::Dashboard {
            let mut issues_by_project: std::collections::BTreeMap<String, Vec<Issue>> = std::collections::BTreeMap::new();
            for i in self.issues.iter().filter(|i| i.title.to_lowercase().contains(&filter)) {
                let p_name = self.project_cache.get(&i.project_id).and_then(|p| p.path_with_namespace.as_deref()).unwrap_or("Unknown Project").to_string();
                issues_by_project.entry(p_name).or_default().push(i.clone());
            }
            for (p_name, items) in issues_by_project {
                let key = format!("issues:{}", p_name);
                let is_collapsed = self.collapsed_groups.contains(&key);
                grouped_issues.push(GroupedItem::Header(p_name));
                if !is_collapsed { for item in items { grouped_issues.push(GroupedItem::Item(item)); } }
            }
        } else {
            grouped_issues = self.issues.iter().filter(|i| i.title.to_lowercase().contains(&filter)).cloned().map(GroupedItem::Item).collect();
        }
        self.filtered_issues = grouped_issues;

        self.filtered_projects = self.projects.iter().filter(|p| p.name.to_lowercase().contains(&filter)).cloned().collect();
        self.filtered_pipelines = self.pipelines.iter().filter(|p| p.ref_name.to_lowercase().contains(&filter)).cloned().collect();
        self.filtered_branches = self.project_branches.iter().filter(|b| b.name.to_lowercase().contains(&filter)).cloned().collect();
        
        // Filter Board Issues
        self.filtered_board_issues.clear();
        for (list_id, issues) in &self.board_issues {
            let filtered: Vec<_> = issues.iter().filter(|i| i.title.to_lowercase().contains(&filter) || i.iid.to_string().contains(&filter)).cloned().collect();
            self.filtered_board_issues.insert(*list_id, filtered);
        }

        // Selection normalization
        if !filter.is_empty() {
            self.todos_state.select(Some(0));
            self.mrs_state.select(Some(0));
            self.issues_state.select(Some(0));
            self.projects_state.select(Some(0));
            self.pipelines_state.select(Some(0));
            self.branches_state.select(Some(0));
            for list in &self.board_lists { self.board_issue_states.entry(list.id).or_default().select(Some(0)); }
        }

        self.clamp_selections();
    }

    pub fn clamp_selections(&mut self) {
        let active_sel = self.active_selection;
        let is_main_focused = self.active_panel == Panel::Main;

        let clamp = |state: &mut ListState, len: usize, is_active: bool| {
            if len == 0 {
                // Proactive selection: if this is the active list, pre-select 0 even if empty
                // so it's ready when data arrives.
                if is_active && is_main_focused { state.select(Some(0)); }
                else { state.select(None); }
            }
            else if let Some(i) = state.selected() { 
                if i >= len { state.select(Some(len.saturating_sub(1))); }
            }
            else { 
                state.select(Some(0)); 
            }
        };

        clamp(&mut self.todos_state, self.filtered_todos.len(), active_sel == MenuSelection::Todos);
        clamp(&mut self.mrs_state, self.filtered_mrs.len(), active_sel == MenuSelection::MergeRequests);
        clamp(&mut self.issues_state, self.filtered_issues.len(), active_sel == MenuSelection::Issues);
        clamp(&mut self.projects_state, self.filtered_projects.len(), active_sel == MenuSelection::Projects);
        clamp(&mut self.pipelines_state, self.filtered_pipelines.len(), active_sel == MenuSelection::Pipelines);
        clamp(&mut self.groups_state, self.filtered_groups.len(), active_sel == MenuSelection::Groups);
        clamp(&mut self.branches_state, self.filtered_branches.len(), active_sel == MenuSelection::Branches);
        clamp(&mut self.commits_state, self.commits.len(), active_sel == MenuSelection::Commits);
        clamp(&mut self.jobs_state, self.jobs.len(), self.active_panel == Panel::MainDetail && active_sel == MenuSelection::Pipelines);
        clamp(&mut self.discussion_state, self.discussions.len(), self.active_panel == Panel::MainDetail && self.detail_section == DetailSection::Discussions);
        clamp(&mut self.epics_state, self.epics.len(), active_sel == MenuSelection::Epics);
        clamp(&mut self.project_boards_state, self.project_boards.len(), active_sel == MenuSelection::Boards);
        clamp(&mut self.search_state, self.search_results.len(), self.show_search);
        clamp(&mut self.mr_diff_state, self.mr_diffs.len(), self.active_panel == Panel::MainDetail && self.detail_section == DetailSection::Changes);
        clamp(&mut self.board_lists_state, self.board_lists.len(), self.view_mode == ViewMode::Board);

        // Clamp board issues
        for list in &self.board_lists {
            let len = self.filtered_board_issues.get(&list.id).map(|v| v.len()).unwrap_or(0);
            let state = self.board_issue_states.entry(list.id).or_default();
            if len == 0 {
                 if self.view_mode == ViewMode::Board && self.board_lists_state.selected() == self.board_lists.iter().position(|l| l.id == list.id) {
                     state.select(Some(0));
                 } else {
                     state.select(None);
                 }
            } else if let Some(i) = state.selected() {
                if i >= len { state.select(Some(len.saturating_sub(1))); }
            } else {
                state.select(Some(0));
            }
        }

        // Navigation state defaults
        if self.dashboard_menu_state.selected().is_none() { self.dashboard_menu_state.select(Some(0)); }
        if self.explore_menu_state.selected().is_none() { self.explore_menu_state.select(Some(0)); }
        if self.project_menu_state.selected().is_none() { self.project_menu_state.select(Some(0)); }
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
            MouseEventKind::ScrollDown => { self.move_down(); let _ = self.update_preview().await; }
            MouseEventKind::ScrollUp => { self.move_up(); let _ = self.update_preview().await; }
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                let x = mouse.column;
                let y = mouse.row;

                // 0. Handle Popups First (Highest Priority)
                if self.show_help_popup {
                    if let Some(r) = self.get_rect("popup_help") {
                        if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                             // Help results usually start after search box
                             let inner_y = y.saturating_sub(r.y).saturating_sub(if self.show_help_filter { 5 } else { 2 });
                             if !self.filtered_help.is_empty() {
                                 let idx = (self.help_state.offset() + inner_y as usize).min(self.filtered_help.len() - 1);
                                 self.help_state.select(Some(idx));
                             }
                        } else {
                            self.show_help_popup = false;
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                }

                if self.show_search {
                    if let Some(r) = self.get_rect("popup_search") {
                        if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                             let inner_y = y.saturating_sub(r.y).saturating_sub(5); // search box(3) + margin(2)
                             if !self.search_results.is_empty() {
                                 let idx = (self.search_state.offset() + inner_y as usize).min(self.search_results.len() - 1);
                                 self.search_state.select(Some(idx));
                             }
                        } else {
                            self.show_search = false;
                            self.input_mode = false;
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                }

                if self.show_project_popup {
                    if let Some(r) = self.get_rect("popup_project") {
                        if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                             let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                             if !self.projects.is_empty() {
                                 let idx = (self.projects_state.offset() + inner_y as usize).min(self.projects.len() - 1);
                                 self.projects_state.select(Some(idx));
                             }
                        } else {
                            self.show_project_popup = false;
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                }

                if self.show_label_popup {
                    if let Some(r) = self.get_rect("popup_label") {
                        if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                             let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                             if !self.project_labels.is_empty() {
                                 let idx = (self.label_state.offset() + inner_y as usize).min(self.project_labels.len() - 1);
                                 self.label_state.select(Some(idx));
                             }
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                }

                if self.show_assignee_popup {
                    if let Some(r) = self.get_rect("popup_assignee") {
                        if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                             let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                             if !self.project_members.is_empty() {
                                 let idx = (self.assignee_state.offset() + inner_y as usize).min(self.project_members.len() - 1);
                                 self.assignee_state.select(Some(idx));
                             }
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                }

                if self.show_milestone_popup {
                    if let Some(r) = self.get_rect("popup_milestone") {
                        if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                             let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                             let len = self.project_milestones.len() + 1;
                             let idx = (self.milestone_state.offset() + inner_y as usize).min(len - 1);
                             self.milestone_state.select(Some(idx));
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                }

                if self.show_snooze_popup {
                    if let Some(r) = self.get_rect("popup_snooze") {
                        if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                             let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                             let idx = (inner_y as usize).min(4);
                             self.snooze_state.select(Some(idx));
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                }

                // 1. Check Navigation Panels
                if let Some(r) = self.get_rect("nav_dashboard") {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
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
                }
                if let Some(r) = self.get_rect("nav_explore") {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
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
                }
                if let Some(r) = self.get_rect("nav_project") {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
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
                }

                // 2. Check Main List
                if let Some(r) = self.get_rect("main_list") {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        self.active_panel = Panel::Main;
                        
                        // Check if clicked on Tabs (y = r.y + 1 since border is at r.y)
                        if y == r.y + 1 {
                            let rel_x = x.saturating_sub(r.x).saturating_sub(1);
                            let titles = match self.active_selection {
                                MenuSelection::MergeRequests => vec!["Opened", "Merged", "Closed", "All"],
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
                                            self.mr_filter = match idx { 0 => MergeRequestFilter::Opened, 1 => MergeRequestFilter::Merged, 2 => MergeRequestFilter::Closed, _ => MergeRequestFilter::All };
                                        }
                                        MenuSelection::Issues => {
                                            self.issue_filter = match idx { 0 => IssueFilter::Opened, 1 => IssueFilter::Closed, _ => IssueFilter::All };
                                        }
                                        MenuSelection::Todos => {
                                            self.todo_filter = match idx { 0 => TodoFilter::Pending, 1 => TodoFilter::Snoozed, _ => TodoFilter::Done };
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
                                MenuSelection::Todos => { let idx = self.todos_state.offset() + inner_y; if idx < self.filtered_todos.len() { self.todos_state.select(Some(idx)); } }
                                MenuSelection::MergeRequests => { let idx = self.mrs_state.offset() + inner_y; if idx < self.filtered_mrs.len() { self.mrs_state.select(Some(idx)); } }
                                MenuSelection::Issues => { let idx = self.issues_state.offset() + inner_y; if idx < self.filtered_issues.len() { self.issues_state.select(Some(idx)); } }
                                MenuSelection::Pipelines => { let idx = self.pipelines_state.offset() + inner_y; if idx < self.pipelines.len() { self.pipelines_state.select(Some(idx)); } }
                                MenuSelection::Projects => { let idx = self.projects_state.offset() + inner_y; if idx < self.filtered_projects.len() { self.projects_state.select(Some(idx)); } }
                                MenuSelection::Groups => { let idx = self.groups_state.offset() + inner_y; if idx < self.filtered_groups.len() { self.groups_state.select(Some(idx)); } }
                                MenuSelection::Branches => { let idx = self.branches_state.offset() + inner_y; if idx < self.project_branches.len() { self.branches_state.select(Some(idx)); } }
                                MenuSelection::Commits => { let idx = self.commits_state.offset() + inner_y; if idx < self.commits.len() { self.commits_state.select(Some(idx)); } }
                                MenuSelection::Boards => { 
                                    if self.view_mode != ViewMode::Board {
                                        let idx = self.project_boards_state.offset() + inner_y;
                                        if idx < self.project_boards.len() { 
                                            self.project_boards_state.select(Some(idx)); 
                                        } 
                                    }
                                }
                                MenuSelection::Epics => { let idx = self.epics_state.offset() + inner_y; if idx < self.epics.len() { self.epics_state.select(Some(idx)); } }
                                _ => {}
                            }
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                }

                // 2.1 Check Kanban Columns specifically
                if self.view_mode == ViewMode::Board {
                    for i in 0..self.board_lists.len() {
                        if let Some(r) = self.get_rect(&format!("kanban_col_{}", i)) {
                            if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                                self.active_panel = Panel::Main;
                                self.board_lists_state.select(Some(i));
                                let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                                let list_id = self.board_lists[i].id;
                                let issues_len = self.filtered_board_issues.get(&list_id).map(|v| v.len()).unwrap_or(0);
                                if issues_len > 0 {
                                    let state = self.board_issue_states.entry(list_id).or_default();
                                    // Heuristic: items are ~4 lines high (Title + Meta + Labels + Spacer)
                                    let item_idx = (state.offset() + inner_y as usize / 4).min(issues_len - 1);
                                    state.select(Some(item_idx));
                                }
                                let _ = self.update_preview().await;
                                return;
                            }
                        }
                    }
                }

                // 3. Check Detail Panels
                if let Some(r) = self.get_rect("jobs_panel") {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        self.active_panel = Panel::MainDetail;
                        let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                        if (inner_y as usize) < self.jobs.len() {
                            self.jobs_state.select(Some(inner_y as usize));
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                }
                if let Some(r) = self.get_rect("metadata_panel") {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        self.active_panel = Panel::MainDetail;
                        self.detail_section = DetailSection::Metadata;
                        let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                        self.selected_metadata_field = match inner_y {
                            0 => MetadataField::Title,
                            1 => MetadataField::Labels,
                            2 => MetadataField::Assignees,
                            3 => MetadataField::Milestone,
                            _ => MetadataField::Description,
                        };
                        let _ = self.update_preview().await;
                        return;
                    }
                }
                if let Some(r) = self.get_rect("detail_view") {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        self.active_panel = Panel::MainDetail;
                        self.detail_section = DetailSection::Description;
                        let _ = self.update_preview().await;
                        return;
                    }
                }
                if let Some(r) = self.get_rect("discussions_panel") {
                    if x > r.x && x < r.x + r.width - 1 && y > r.y && y < r.y + r.height - 1 {
                        self.active_panel = Panel::MainDetail;
                        self.detail_section = DetailSection::Discussions;
                        let inner_y = y.saturating_sub(r.y).saturating_sub(1);
                        if (inner_y as usize) < self.discussions.len() {
                            self.discussion_state.select(Some(inner_y as usize));
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                }
            }
            _ => {}
        }
    }

    pub async fn move_board_issue(&mut self, direction: i32) -> Result<(), Box<dyn std::error::Error>> {
        if self.view_mode != ViewMode::Board { return Ok(()); }
        let current_list_idx = if let Some(i) = self.board_lists_state.selected() { i } else { return Ok(()); };
        let target_list_idx = if direction > 0 { current_list_idx + 1 } else { current_list_idx.saturating_sub(1) };
        
        if target_list_idx >= self.board_lists.len() || target_list_idx == current_list_idx { return Ok(()); }
        
        let from_list = self.board_lists[current_list_idx].clone();
        let to_list = self.board_lists[target_list_idx].clone();
        
        let issue_idx = if let Some(state) = self.board_issue_states.get(&from_list.id) {
            if let Some(i) = state.selected() { i } else { return Ok(()); }
        } else { return Ok(()); };
        
        let issue = if let Some(issues) = self.board_issues.get(&from_list.id) {
            if let Some(i) = issues.get(issue_idx) { i.clone() } else { return Ok(()); }
        } else { return Ok(()); };
        
        let project_id = issue.project_id;
        let issue_iid = issue.iid;
        
        let mut add_labels = Vec::new();
        let mut remove_labels = Vec::new();
        let mut state_event = None;
        
        if let Some(l) = &from_list.label { remove_labels.push(l.name.clone()); }
        if from_list.list_type == "closed" { state_event = Some("reopen".to_string()); }
        
        if let Some(l) = &to_list.label { add_labels.push(l.name.clone()); }
        if to_list.list_type == "closed" { state_event = Some("close".to_string()); }
        
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        
        let add_l_str = if add_labels.is_empty() { None } else { Some(add_labels.join(",")) };
        let rem_l_str = if remove_labels.is_empty() { None } else { Some(remove_labels.join(",")) };

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
                weight: None,
                confidential: None,
            };
            
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Move issue failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Move issue timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        
        self.board_lists_state.select(Some(target_list_idx));
        Ok(())
    }

    pub async fn back(&mut self) { self.view_mode = ViewMode::List; self.active_panel = Panel::Dashboard; }

    pub fn copy_to_clipboard(&mut self, text: String) {
        if text.is_empty() { return; }

        // Strip ANSI codes and GitLab noise for a clean shareable copy
        let clean_text = pipelines::strip_ansi(&text);

        // 1. Attempt local system clipboard (works on local machine)
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                let _ = clipboard.set_text(clean_text.clone());
            }
            Err(_) => {}
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
        if self.view_mode != ViewMode::Board { return None; }
        if let Some(list_idx) = self.board_lists_state.selected() {
            if let Some(list) = self.board_lists.get(list_idx) {
                if let Some(state) = self.board_issue_states.get(&list.id) {
                    if let Some(issue_idx) = state.selected() {
                        return self.board_issues.get(&list.id).and_then(|v| v.get(issue_idx)).map(|i| i.web_url.clone());
                    }
                }
            }
        }
        None
    }

    pub fn parse_job_log(&mut self) { self.job_log_lines = pipelines::parse_job_log(&self.job_log); }
    pub fn update_tree_state(&mut self) {
        // Build items for tui_tree_widget
        // This is a flat list right now, but tui_tree_widget requires TreeItem
        // We will deal with mapping it in the UI, but here we can just ensure state is valid.
    }
    pub async fn check_pagination(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.active_panel != Panel::Main { return Ok(()); }
        match self.active_selection {
            MenuSelection::Todos => { if let Some(i) = self.todos_state.selected() { if i + 10 >= self.filtered_todos.len() { self.fetch_next_todos().await?; } } }
            MenuSelection::MergeRequests => { if let Some(i) = self.mrs_state.selected() { if i + 10 >= self.filtered_mrs.len() { if self.active_context == Context::Project { self.fetch_next_merge_requests().await?; } else { self.fetch_next_global_merge_requests().await?; } } } }
            MenuSelection::Issues => { if let Some(i) = self.issues_state.selected() { if i + 10 >= self.filtered_issues.len() { if self.active_context == Context::Project { self.fetch_next_issues().await?; } else { self.fetch_next_global_issues().await?; } } } }
            MenuSelection::Pipelines => { if let Some(i) = self.pipelines_state.selected() { if i + 10 >= self.pipelines.len() { self.fetch_next_pipelines().await?; } } }
            MenuSelection::Projects => { if let Some(i) = self.projects_state.selected() { if i + 10 >= self.filtered_projects.len() { self.fetch_next_projects().await?; } } }
            MenuSelection::Groups => { if let Some(i) = self.groups_state.selected() { if i + 10 >= self.filtered_groups.len() { self.fetch_next_groups().await?; } } }
            MenuSelection::Repository | MenuSelection::Branches | MenuSelection::Boards | MenuSelection::Epics => {}
            MenuSelection::Commits => { if let Some(i) = self.commits_state.selected() { if i + 10 >= self.commits.len() { self.fetch_next_commits().await?; } } }
        }
        Ok(())
    }
    pub async fn enter(&mut self) { 
        match self.input_target {
            InputTarget::EditTitle | InputTarget::EditDescription => { if self.selected_issue.is_some() { let _ = self.update_issue().await; } else if self.selected_mr.is_some() { let _ = self.update_mr().await; } }
            InputTarget::Reply => { let _ = self.post_reply().await; }
            InputTarget::NewComment => { let _ = self.post_new_comment().await; }
            InputTarget::SnoozeUntil => { let snooze_until = format!("{}T09:00:00Z", self.calendar_date.format("%Y-%m-%d")); let _ = self.snooze_todo(snooze_until).await; self.show_calendar_popup = false; }
            _ => {}
        }
    }
    pub async fn post_reply(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let body = self.textarea.lines().join("\n");
        if body.is_empty() { return Ok(()); }
        if let Some(mr) = &self.selected_mr {
            if let Some(disc_idx) = self.discussion_state.selected() {
                if let Some(disc) = self.discussions.get(disc_idx) {
                    let pid = mr.project_id; let iid = mr.iid; let disc_id = disc.id.clone();
                    let client = self.gitlab.clone(); let tx = self.tx.clone(); let _ = tx.send(DataUpdate::Loading(true));
                    tokio::spawn(async move {
                        let endpoint = AddMergeRequestDiscussionNote {
                            project: pid, merge_request: iid, discussion: disc_id, body,
                        };
                        let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
                        match res {
                            Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                            Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Post reply failed: {}", e))); }
                            Err(_) => { let _ = tx.send(DataUpdate::Error("Post reply timed out".into())); }
                        }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
                    self.textarea = ratatui_textarea::TextArea::default();
                    self.input_mode = false;
                }
            }
        } else if let Some(issue) = &self.selected_issue {
            if let Some(disc_idx) = self.discussion_state.selected() {
                if let Some(disc) = self.discussions.get(disc_idx) {
                    let pid = issue.project_id; let iid = issue.iid; let disc_id = disc.id.clone();
                    let client = self.gitlab.clone(); let tx = self.tx.clone(); let _ = tx.send(DataUpdate::Loading(true));
                    tokio::spawn(async move {
                        let endpoint = AddIssueDiscussionNote {
                            project: pid, issue: iid, discussion: disc_id, body,
                        };
                        let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
                        match res {
                            Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                            Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Post reply failed: {}", e))); }
                            Err(_) => { let _ = tx.send(DataUpdate::Error("Post reply timed out".into())); }
                        }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
                    self.textarea = ratatui_textarea::TextArea::default();
                    self.input_mode = false;
                }
            }
        }
        Ok(())
    }
    pub fn toggle_focus(&mut self) { self.active_panel = match self.active_panel { Panel::Main | Panel::MainDetail => match self.active_context { Context::Dashboard => Panel::Dashboard, Context::Explore => Panel::Explore, Context::Project => Panel::Project }, _ => Panel::Main }; }
    pub fn switch_project(&mut self) { self.show_project_popup = true; let _ = self.fetch_projects(); }
    pub async fn select_project(&mut self) { 
        if let Some(i) = self.projects_state.selected() { 
            if let Some(project) = self.projects.get(i).cloned() { 
                self.selected_project = Some(project); 
                self.show_project_popup = false; 
                self.active_panel = Panel::Main; 
                self.active_context = Context::Project; 
                self.project_menu_state.select(Some(0)); 
                self.active_selection = MenuSelection::MergeRequests; 
                self.selected_mr = None; self.selected_issue = None; self.selected_pipeline = None; 
                self.discussions.clear(); self.jobs.clear(); self.merge_requests.clear(); self.issues.clear(); self.pipelines.clear(); 
                self.filter_lists(); 
                let _ = self.refresh().await; 
            } 
        } 
    }
}

pub fn parse_reactions(reaction_values: Vec<serde_json::Value>) -> Vec<AwardEmoji> {
    let mut reactions = Vec::new();
    for val in reaction_values {
        let name = val.get("name").and_then(|v| v.as_str()).or_else(|| val.get("emoji").and_then(|v| v.as_str())).or_else(|| val.get("emoji_name").and_then(|v| v.as_str()));
        if let Some(n) = name { let display_name = if let Some(emoji) = emojis::get_by_shortcode(n) { emoji.as_str().to_string() } else { n.to_string() }; reactions.push(AwardEmoji { name: display_name }); }
    }
    reactions
}

