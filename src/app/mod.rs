pub mod issues;
pub mod mrs;
pub mod pipelines;
pub mod global;

use gitlab::{AsyncGitlab, api::AsyncQuery};
use crate::models::*;
use crate::api::*;
use ratatui::widgets::ListState;
use crate::ui::Theme;
use chrono::Local;
use regex::Regex;

impl App {
    pub fn new(gitlab: AsyncGitlab) -> Self {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
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
            detail_section: DetailSection::Discussions,
            selected_metadata_field: MetadataField::Title,
            selected_mr: None,
            selected_issue: None,
            selected_pipeline: None,
            selected_job: None,
            mr_diffs: Vec::new(),
            mr_diff_state: ListState::default(),
            job_log: String::new(),
            job_log_lines: Vec::new(),
            collapsed_sections: std::collections::HashSet::new(),
            job_log_scroll: 0,
            job_log_viewport_height: 0,
            follow_log: true,
            discussions: Vec::new(),
            discussion_state: ListState::default(),
            input_mode: false,
            input_target: InputTarget::Reply,
            is_loading: false,
            reply_input: String::new(),
            cursor_index: 0,
            theme: Theme::load(),
            last_refresh: std::time::Instant::now(),
            tx,
            project_labels: Vec::new(),
            project_milestones: Vec::new(),
            project_members: Vec::new(),
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
            last_error: None,
            status_message: None,
            loading_tasks: 0,
            last_fetch_todos: None,
            last_fetch_mrs: None,
            last_fetch_issues: None,
            last_fetch_projects: None,
            last_fetch_groups: None,
            last_fetch_pipelines: None,
        }
    }

    pub async fn process_updates(&mut self, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
        while let Ok(update) = rx.try_recv() {
            match update {
                DataUpdate::Todos { data, page, has_more } => { self.todos = data; self.todos_page = page; self.todos_has_more = has_more; self.filter_lists(); }
                DataUpdate::AppendTodos(mut data) => { self.todos_has_more = data.len() >= 50; self.todos.append(&mut data); self.todos_page += 1; self.filter_lists(); }
                
                DataUpdate::Projects { data, page, has_more } => { self.projects = data; self.projects_page = page; self.projects_has_more = has_more; self.filter_lists(); }
                DataUpdate::AppendProjects(mut data) => { self.projects_has_more = data.len() >= 50; self.projects.append(&mut data); self.projects_page += 1; self.filter_lists(); }
                
                DataUpdate::Groups { data, page, has_more } => { self.groups = data; self.groups_page = page; self.groups_has_more = has_more; self.filter_lists(); }
                DataUpdate::AppendGroups(mut data) => { self.groups_has_more = data.len() >= 50; self.groups.append(&mut data); self.groups_page += 1; self.filter_lists(); }
                
                DataUpdate::GlobalMRs { data, page, has_more } => { if self.active_context == Context::Dashboard { self.merge_requests = data; self.mrs_page = page; self.mrs_has_more = has_more; self.filter_lists(); } }
                DataUpdate::ProjectMRs { project_id, data, page, has_more } => { if self.active_context == Context::Project && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.merge_requests = data; self.mrs_page = page; self.mrs_has_more = has_more; self.filter_lists(); } }
                DataUpdate::AppendGlobalMRs(mut data) => { if self.active_context == Context::Dashboard { self.mrs_has_more = data.len() >= 50; self.merge_requests.append(&mut data); self.mrs_page += 1; self.filter_lists(); } }
                DataUpdate::AppendProjectMRs { project_id, mut data } => { if self.active_context == Context::Project && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.mrs_has_more = data.len() >= 50; self.merge_requests.append(&mut data); self.mrs_page += 1; self.filter_lists(); } }
                DataUpdate::ClearGlobalMRs => { if self.active_context == Context::Dashboard { self.merge_requests.clear(); self.filter_lists(); } }
                
                DataUpdate::GlobalIssues { data, page, has_more } => { if self.active_context == Context::Dashboard { self.issues = data; self.issues_page = page; self.issues_has_more = has_more; self.filter_lists(); } }
                DataUpdate::ProjectIssues { project_id, data, page, has_more } => { if self.active_context == Context::Project && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.issues = data; self.issues_page = page; self.issues_has_more = has_more; self.filter_lists(); } }
                DataUpdate::AppendGlobalIssues(mut data) => { if self.active_context == Context::Dashboard { self.issues_has_more = data.len() >= 50; self.issues.append(&mut data); self.issues_page += 1; self.filter_lists(); } }
                DataUpdate::AppendProjectIssues { project_id, mut data } => { if self.active_context == Context::Project && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.issues_has_more = data.len() >= 50; self.issues.append(&mut data); self.issues_page += 1; self.filter_lists(); } }
                DataUpdate::ClearGlobalIssues => { if self.active_context == Context::Dashboard { self.issues.clear(); self.filter_lists(); } }
                
                DataUpdate::ProjectPipelines { project_id, data, page, has_more } => { if self.active_context == Context::Project && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.pipelines = data; self.pipelines_page = page; self.pipelines_has_more = has_more; self.filter_lists(); } }
                DataUpdate::AppendProjectPipelines { project_id, mut data } => { if self.active_context == Context::Project && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.pipelines_has_more = data.len() >= 50; self.pipelines.append(&mut data); self.pipelines_page += 1; self.filter_lists(); } }
                
                DataUpdate::Discussions { project_id, iid, data } => { if let Some(selected) = &self.selected_mr { if selected.iid == iid && selected.project_id == project_id { self.discussions = data; } } }
                DataUpdate::IssueDiscussions { project_id, iid, data } => { if let Some(selected) = &self.selected_issue { if selected.iid == iid && selected.project_id == project_id { self.discussions = data; } } }
                DataUpdate::Jobs { project_id, pipeline_id, data } => { if let Some(selected) = &self.selected_pipeline { if selected.id == pipeline_id && selected.project_id == project_id { self.jobs = data; } } }
                DataUpdate::Log { project_id, job_id, data } => { if let Some(selected) = &self.selected_job { if selected.id == job_id && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.job_log = data; self.parse_job_log(); } } }
                DataUpdate::Diffs { project_id, iid, data } => { if let Some(selected) = &self.selected_mr { if selected.iid == iid && selected.project_id == project_id { self.mr_diffs = data; } } }
                DataUpdate::Metadata { labels, milestones, members } => { self.project_labels = labels; self.project_milestones = milestones; self.project_members = members; }
                DataUpdate::MRDetail(mr) => { if let Some(selected) = &mut self.selected_mr { if selected.iid == mr.iid && selected.project_id == mr.project_id { if !self.input_mode { *selected = mr; } } } }
                DataUpdate::IssueDetail(issue) => { if let Some(selected) = &mut self.selected_issue { if selected.iid == issue.iid && selected.project_id == issue.project_id { if !self.input_mode { *selected = issue; } } } }
                DataUpdate::Loading(b) => { if b { self.loading_tasks += 1; } else { self.loading_tasks = self.loading_tasks.saturating_sub(1); } self.is_loading = self.loading_tasks > 0; }
                DataUpdate::Error(e) => { self.last_error = Some(e); self.loading_tasks = self.loading_tasks.saturating_sub(1); self.is_loading = self.loading_tasks > 0; }
                DataUpdate::TriggerRefresh => { self.refresh().await; }
            }
        }
        // Automatically update preview after processing updates to ensure detail pane matches newly arrived data
        self.update_preview().await;
    }

    pub async fn initial_setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_loading = true; self.loading_tasks = 3;
        let c = self.gitlab.clone(); let tx = self.tx.clone();
        let t1 = tx.clone(); let c1 = c.clone(); 
        tokio::spawn(async move { 
            let ep = Todos { page: Some(1), per_page: Some(50) }; 
            let res: Result<Vec<Todo>, _> = ep.query_async(&c1).await;
            if let Ok(data) = res { let has_more = data.len() == 50; let _ = t1.send(DataUpdate::Todos { data, page: 1, has_more }); } 
            let _ = t1.send(DataUpdate::Loading(false)); 
        });
        let t2 = tx.clone(); let c2 = c.clone(); 
        tokio::spawn(async move { 
            let ep = PagedProjects { membership: true, page: 1, per_page: 50 }; 
            let res: Result<Vec<Project>, _> = ep.query_async(&c2).await;
            if let Ok(data) = res { let has_more = data.len() == 50; let _ = t2.send(DataUpdate::Projects { data, page: 1, has_more }); } 
            let _ = t2.send(DataUpdate::Loading(false)); 
        });
        let t3 = tx.clone(); let c3 = c.clone(); 
        tokio::spawn(async move { 
            let ep = Groups { page: Some(1), per_page: Some(50) }; 
            let res: Result<Vec<Group>, _> = ep.query_async(&c3).await;
            if let Ok(data) = res { let has_more = data.len() == 50; let _ = t3.send(DataUpdate::Groups { data, page: 1, has_more }); } 
            let _ = t3.send(DataUpdate::Loading(false)); 
        });
        Ok(())
    }

    pub async fn refresh(&mut self) {
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
                if let Some(project) = &self.selected_project {
                    let pid = project.id;
                    match self.view_mode {
                        ViewMode::List => match self.active_selection {
                            MenuSelection::MergeRequests => { let _ = self.fetch_merge_requests().await; }
                            MenuSelection::Issues => { let _ = self.fetch_issues().await; }
                            MenuSelection::Pipelines => { let _ = self.fetch_pipelines().await; }
                            _ => {}
                        },
                        ViewMode::Detail => {
                            if let Some(mr) = self.selected_mr.clone() { let _ = self.fetch_discussions(pid, mr.iid).await; }
                            else if let Some(issue) = self.selected_issue.clone() { let _ = self.fetch_issue_discussions(pid, issue.iid).await; }
                            else if let Some(pipeline) = self.selected_pipeline.clone() { let _ = self.fetch_jobs(pid, pipeline.id).await; }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn toggle_help(&mut self) { self.show_help_popup = !self.show_help_popup; }
    pub fn filter_help(&mut self) {}

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
                self.active_selection = match idx { 0 => MenuSelection::Pipelines, 1 => MenuSelection::MergeRequests, _ => MenuSelection::Issues };
            }
            _ => {}
        }

        // If category OR context changed, clear all details to prevent stale previews/jumps
        if prev_selection != self.active_selection || prev_context != self.active_context {
            self.selected_mr = None;
            self.selected_issue = None;
            self.selected_pipeline = None;
            self.discussions.clear();
            self.jobs.clear();
            
            // We NO LONGER clear lists here to avoid visual flicker and selection jumps.
            // Stale data is prevented from appearing in details by the clearing above.
            // Source verification in process_updates handles the rest.

            self.filter_lists();
            self.refresh().await;
        }

        // Fetch/Enrich active item
        match self.active_selection {
            MenuSelection::Todos => {
                if let Some(i) = self.todos_state.selected() {
                    if let Some(GroupedItem::Item(todo)) = self.filtered_todos.get(i) {
                        // Robust IID Extraction: Find the number following /issues/ or /merge_requests/
                        let re = Regex::new(r"/(?:issues|merge_requests)/(\d+)").unwrap();
                        let iid = re.captures(&todo.target_url).and_then(|cap| cap.get(1)).and_then(|m| m.as_str().parse().ok());

                        if let Some(iid) = iid {
                            if todo.target_type == "MergeRequest" {
                                if self.selected_mr.as_ref().map_or(true, |m| m.iid != iid || m.project_id != todo.project.id) {
                                    self.selected_issue = None; self.discussions.clear();
                                    self.selected_mr = Some(MergeRequest { iid, project_id: todo.project.id, title: todo.body.clone(), author: Author { id: 0, username: "".into() }, web_url: todo.target_url.clone(), state: "opened".into(), reactions: Vec::new(), description: None, references: None, upvotes: None, downvotes: None, labels: Vec::new(), milestone: None, assignees: Vec::new(), created_at: "".into() });
                                    let _ = self.fetch_discussions(todo.project.id, iid).await;
                                }
                            } else if todo.target_type == "Issue" {
                                if self.selected_issue.as_ref().map_or(true, |iss| iss.iid != iid || iss.project_id != todo.project.id) {
                                    self.selected_mr = None; self.discussions.clear();
                                    self.selected_issue = Some(Issue { iid, project_id: todo.project.id, title: todo.body.clone(), description: None, author: Author { id: 0, username: "".into() }, assignees: Vec::new(), state: "opened".into(), web_url: todo.target_url.clone(), references: None, issue_type: None, labels: Vec::new(), milestone: None, due_date: None, weight: None, task_completion_status: None, confidential: false, created_at: "".into(), upvotes: None, downvotes: None, reactions: Vec::new() });
                                    let _ = self.fetch_issue_discussions(todo.project.id, iid).await;
                                }
                            }
                        } else {
                            self.selected_mr = None; self.selected_issue = None; self.discussions.clear();
                        }
                    } else {
                        self.selected_mr = None; self.selected_issue = None; self.discussions.clear();
                    }
                } else {
                    self.selected_mr = None; self.selected_issue = None; self.discussions.clear();
                }
            }
            MenuSelection::MergeRequests => {
                if let Some(i) = self.mrs_state.selected() {
                    if let Some(GroupedItem::Item(mr)) = self.filtered_mrs.get(i) {
                        if self.selected_mr.as_ref().map_or(true, |prev| prev.iid != mr.iid || prev.project_id != mr.project_id) {
                            self.selected_mr = Some(mr.clone()); self.selected_issue = None; self.discussions.clear();
                            let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                        }
                    } else {
                        self.selected_mr = None; self.discussions.clear();
                    }
                } else {
                    self.selected_mr = None; self.discussions.clear();
                }
            }
            MenuSelection::Issues => {
                if let Some(i) = self.issues_state.selected() {
                    if let Some(GroupedItem::Item(issue)) = self.filtered_issues.get(i) {
                        if self.selected_issue.as_ref().map_or(true, |prev| prev.iid != issue.iid || prev.project_id != issue.project_id) {
                            self.selected_issue = Some(issue.clone()); self.selected_mr = None; self.discussions.clear();
                            let _ = self.fetch_issue_discussions(issue.project_id, issue.iid).await;
                        }
                    } else {
                        self.selected_issue = None; self.discussions.clear();
                    }
                } else {
                    self.selected_issue = None; self.discussions.clear();
                }
            }
            MenuSelection::Pipelines => {
                if let Some(i) = self.pipelines_state.selected() {
                    if let Some(p) = self.pipelines.get(i) {
                        if self.selected_pipeline.as_ref().map_or(true, |prev| prev.id != p.id) {
                            self.selected_pipeline = Some(p.clone()); self.jobs.clear();
                            let _ = self.fetch_jobs(p.project_id, p.id).await;
                        }
                    } else {
                        self.selected_pipeline = None; self.jobs.clear();
                    }
                } else {
                    self.selected_pipeline = None; self.jobs.clear();
                }
            }
            _ => {}
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
                let next = (current + 1) % 3;
                self.dashboard_menu_state.select(Some(next));
                
                if next == 0 {
                    self.status_message = Some("Viewing Todos...".into());
                }
            }
            Panel::Explore => { let i = (self.explore_menu_state.selected().unwrap_or(0) + 1) % 2; self.explore_menu_state.select(Some(i)); }
            Panel::Project => { let i = (self.project_menu_state.selected().unwrap_or(0) + 1) % 3; self.project_menu_state.select(Some(i)); }
            Panel::Main => {
                match self.active_selection {
                    MenuSelection::Todos => if !self.filtered_todos.is_empty() { let i = (self.todos_state.selected().unwrap_or(0) + 1) % self.filtered_todos.len(); self.todos_state.select(Some(i)); }
                    MenuSelection::MergeRequests => if !self.filtered_mrs.is_empty() { let i = (self.mrs_state.selected().unwrap_or(0) + 1) % self.filtered_mrs.len(); self.mrs_state.select(Some(i)); }
                    MenuSelection::Issues => if !self.filtered_issues.is_empty() { let i = (self.issues_state.selected().unwrap_or(0) + 1) % self.filtered_issues.len(); self.issues_state.select(Some(i)); }
                    MenuSelection::Pipelines => if !self.pipelines.is_empty() { let i = (self.pipelines_state.selected().unwrap_or(0) + 1) % self.pipelines.len(); self.pipelines_state.select(Some(i)); }
                    MenuSelection::Projects => if !self.projects.is_empty() { let i = (self.projects_state.selected().unwrap_or(0) + 1) % self.projects.len(); self.projects_state.select(Some(i)); }
                    MenuSelection::Groups => if !self.groups.is_empty() { let i = (self.groups_state.selected().unwrap_or(0) + 1) % self.groups.len(); self.groups_state.select(Some(i)); }
                }
            }
            Panel::MainDetail => {
                match self.detail_section {
                    DetailSection::Metadata => {
                        self.selected_metadata_field = match self.selected_metadata_field {
                            MetadataField::Title => MetadataField::Labels,
                            MetadataField::Labels => MetadataField::Assignees,
                            MetadataField::Assignees => MetadataField::Milestone,
                            MetadataField::Milestone => MetadataField::Description,
                            _ => { self.detail_section = DetailSection::Discussions; self.discussion_state.select(Some(0)); MetadataField::Title }
                        };
                    }
                    DetailSection::Discussions => {
                        if !self.discussions.is_empty() {
                            let current = self.discussion_state.selected().unwrap_or(0);
                            if current >= self.discussions.len().saturating_sub(1) {
                                self.detail_section = DetailSection::Metadata; self.selected_metadata_field = MetadataField::Title; self.discussion_state.select(None);
                            } else { self.discussion_state.select(Some(current + 1)); }
                        } else { self.detail_section = DetailSection::Metadata; self.selected_metadata_field = MetadataField::Title; }
                    }
                }
            }
        }
    }

    pub fn move_up(&mut self) {
        match self.active_panel {
            Panel::Dashboard => { let i = if self.dashboard_menu_state.selected().unwrap_or(0) == 0 { 2 } else { self.dashboard_menu_state.selected().unwrap() - 1 }; self.dashboard_menu_state.select(Some(i)); }
            Panel::Explore => { let i = if self.explore_menu_state.selected().unwrap_or(0) == 0 { 1 } else { self.explore_menu_state.selected().unwrap() - 1 }; self.explore_menu_state.select(Some(i)); }
            Panel::Project => { let i = if self.project_menu_state.selected().unwrap_or(0) == 0 { 2 } else { self.project_menu_state.selected().unwrap() - 1 }; self.project_menu_state.select(Some(i)); }
            Panel::Main => {
                match self.active_selection {
                    MenuSelection::Todos => if !self.filtered_todos.is_empty() { let i = if self.todos_state.selected().unwrap_or(0) == 0 { self.filtered_todos.len() - 1 } else { self.todos_state.selected().unwrap() - 1 }; self.todos_state.select(Some(i)); }
                    MenuSelection::MergeRequests => if !self.filtered_mrs.is_empty() { let i = if self.mrs_state.selected().unwrap_or(0) == 0 { self.filtered_mrs.len() - 1 } else { self.mrs_state.selected().unwrap() - 1 }; self.mrs_state.select(Some(i)); }
                    MenuSelection::Issues => if !self.filtered_issues.is_empty() { let i = if self.issues_state.selected().unwrap_or(0) == 0 { self.filtered_issues.len() - 1 } else { self.issues_state.selected().unwrap() - 1 }; self.issues_state.select(Some(i)); }
                    MenuSelection::Pipelines => if !self.pipelines.is_empty() { let i = if self.pipelines_state.selected().unwrap_or(0) == 0 { self.pipelines.len() - 1 } else { self.pipelines_state.selected().unwrap() - 1 }; self.pipelines_state.select(Some(i)); }
                    MenuSelection::Projects => if !self.projects.is_empty() { let i = if self.projects_state.selected().unwrap_or(0) == 0 { self.projects.len() - 1 } else { self.projects_state.selected().unwrap() - 1 }; self.projects_state.select(Some(i)); }
                    MenuSelection::Groups => if !self.groups.is_empty() { let i = if self.groups_state.selected().unwrap_or(0) == 0 { self.groups.len() - 1 } else { self.groups_state.selected().unwrap() - 1 }; self.groups_state.select(Some(i)); }
                }
            }
            Panel::MainDetail => {
                match self.detail_section {
                    DetailSection::Metadata => {
                        self.selected_metadata_field = match self.selected_metadata_field {
                            MetadataField::Title => { self.detail_section = DetailSection::Discussions; if !self.discussions.is_empty() { self.discussion_state.select(Some(self.discussions.len().saturating_sub(1))); } MetadataField::Title }
                            MetadataField::Labels => MetadataField::Title,
                            MetadataField::Assignees => MetadataField::Labels,
                            MetadataField::Milestone => MetadataField::Assignees,
                            MetadataField::Description => MetadataField::Milestone,
                            _ => MetadataField::Description,
                        };
                    }
                    DetailSection::Discussions => {
                        if !self.discussions.is_empty() {
                            let current = self.discussion_state.selected().unwrap_or(0);
                            if current == 0 { self.detail_section = DetailSection::Metadata; self.selected_metadata_field = MetadataField::Description; self.discussion_state.select(None); }
                            else { self.discussion_state.select(Some(current - 1)); }
                        } else { self.detail_section = DetailSection::Metadata; self.selected_metadata_field = MetadataField::Description; }
                    }
                }
            }
        }
    }

    pub async fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        
        if self.show_project_popup {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => self.move_down(),
                KeyCode::Up | KeyCode::Char('k') => self.move_up(),
                KeyCode::Enter => self.select_project().await,
                KeyCode::Esc => self.show_project_popup = false,
                _ => {}
            }
            return;
        }

        if self.input_mode {
            match key.code {
                KeyCode::Enter => self.enter().await,
                KeyCode::Esc => { self.input_mode = false; }
                KeyCode::Left => { self.cursor_index = self.cursor_index.saturating_sub(1); }
                KeyCode::Right => { self.cursor_index = (self.cursor_index + 1).min(self.get_input_buffer().chars().count()); }
                KeyCode::Char(c) => {
                    let cursor_idx = self.cursor_index;
                    let input = self.get_input_buffer_mut();
                    let mut chars: Vec<char> = input.chars().collect();
                    chars.insert(cursor_idx.min(chars.len()), c);
                    *input = chars.into_iter().collect();
                    self.cursor_index += 1;
                    if self.input_target == InputTarget::Filter { self.filter_lists(); }
                }
                KeyCode::Backspace if self.cursor_index > 0 => {
                    let cursor_idx = self.cursor_index;
                    let input = self.get_input_buffer_mut();
                    let mut chars: Vec<char> = input.chars().collect();
                    chars.remove(cursor_idx - 1);
                    *input = chars.into_iter().collect();
                    self.cursor_index -= 1;
                    if self.input_target == InputTarget::Filter { self.filter_lists(); }
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Tab => {
                self.active_panel = match self.active_panel {
                    Panel::Dashboard => Panel::Explore,
                    Panel::Explore => Panel::Project,
                    Panel::Project => Panel::Dashboard,
                    _ => self.active_panel,
                };
                let _ = self.update_preview().await;
            }
            KeyCode::BackTab => {
                self.active_panel = match self.active_panel {
                    Panel::Dashboard => Panel::Project,
                    Panel::Explore => Panel::Dashboard,
                    Panel::Project => Panel::Explore,
                    _ => self.active_panel,
                };
                let _ = self.update_preview().await;
            }
            KeyCode::Down | KeyCode::Char('j') => { self.move_down(); let _ = self.update_preview().await; }
            KeyCode::Up | KeyCode::Char('k') => { self.move_up(); let _ = self.update_preview().await; }
            KeyCode::Left | KeyCode::Char('h') => {
                self.active_panel = match self.active_panel {
                    Panel::Main => match self.active_context {
                        Context::Dashboard => Panel::Dashboard,
                        Context::Explore => Panel::Explore,
                        Context::Project => Panel::Project,
                    },
                    Panel::MainDetail => Panel::Main,
                    _ => self.active_panel,
                };
                let _ = self.update_preview().await;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.active_panel = match self.active_panel {
                    Panel::Dashboard | Panel::Explore | Panel::Project => Panel::Main,
                    Panel::Main => Panel::MainDetail,
                    _ => self.active_panel,
                };
                let _ = self.update_preview().await;
            }
            KeyCode::Enter => {
                match self.active_panel {
                    Panel::Dashboard | Panel::Explore | Panel::Project => {
                        let idx = match self.active_panel {
                            Panel::Dashboard => self.dashboard_menu_state.selected().unwrap_or(0),
                            Panel::Explore => self.explore_menu_state.selected().unwrap_or(0),
                            Panel::Project => self.project_menu_state.selected().unwrap_or(0),
                            _ => 0,
                        };
                        self.active_selection = match (self.active_panel, idx) {
                            (Panel::Dashboard, 0) => MenuSelection::Todos,
                            (Panel::Dashboard, 1) => MenuSelection::MergeRequests,
                            (Panel::Dashboard, 2) => MenuSelection::Issues,
                            (Panel::Explore, 0) => MenuSelection::Projects,
                            (Panel::Explore, 1) => MenuSelection::Groups,
                            (Panel::Project, 0) => MenuSelection::Pipelines,
                            (Panel::Project, 1) => MenuSelection::MergeRequests,
                            (Panel::Project, 2) => MenuSelection::Issues,
                            _ => self.active_selection,
                        };
                        self.active_panel = Panel::Main; 
                    }
                    Panel::Main => { self.active_panel = Panel::MainDetail; self.detail_section = DetailSection::Metadata; self.selected_metadata_field = MetadataField::Title; }
                    Panel::MainDetail => { self.enter_detail_action().await; }
                }
                let _ = self.update_preview().await;
            }
            KeyCode::Esc => {
                match self.active_panel {
                    Panel::MainDetail => { self.active_panel = Panel::Main; }
                    Panel::Main => { self.active_panel = match self.active_context {
                        Context::Dashboard => Panel::Dashboard,
                        Context::Explore => Panel::Explore,
                        Context::Project => Panel::Project,
                    }; }
                    _ => {}
                }
                let _ = self.update_preview().await;
            }
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.toggle_help(),
            _ => {}
        }
    }

    pub async fn enter_detail_action(&mut self) {
        if self.active_panel == Panel::MainDetail {
            match self.detail_section {
                DetailSection::Metadata => {
                    match self.selected_metadata_field {
                        MetadataField::Title => { self.input_mode = true; self.input_target = InputTarget::EditTitle; if let Some(i) = &self.selected_issue { self.reply_input = i.title.clone(); } else if let Some(m) = &self.selected_mr { self.reply_input = m.title.clone(); } self.cursor_index = self.reply_input.chars().count(); }
                        MetadataField::Description => { self.input_mode = true; self.input_target = InputTarget::EditDescription; if let Some(i) = &self.selected_issue { self.reply_input = i.description.clone().unwrap_or_default(); } else if let Some(m) = &self.selected_mr { self.reply_input = m.description.clone().unwrap_or_default(); } self.cursor_index = self.reply_input.chars().count(); }
                        _ => {}
                    }
                }
                DetailSection::Discussions => { self.input_mode = true; self.input_target = InputTarget::Reply; self.reply_input.clear(); self.cursor_index = 0; }
            }
        }
    }

    pub fn filter_lists(&mut self) {
        let filter = self.filter_input.to_lowercase();
        self.filtered_todos = self.todos.iter().filter(|t| t.body.to_lowercase().contains(&filter)).cloned().map(GroupedItem::Item).collect();
        self.filtered_mrs = self.merge_requests.iter().filter(|m| m.title.to_lowercase().contains(&filter)).cloned().map(GroupedItem::Item).collect();
        self.filtered_issues = self.issues.iter().filter(|i| i.title.to_lowercase().contains(&filter)).cloned().map(GroupedItem::Item).collect();
        self.filtered_projects = self.projects.iter().filter(|p| p.name.to_lowercase().contains(&filter)).cloned().collect();
        self.filtered_pipelines = self.pipelines.iter().filter(|p| p.ref_name.to_lowercase().contains(&filter)).cloned().collect();
        self.clamp_selections();
    }

    pub fn clamp_selections(&mut self) {
        if !self.filtered_todos.is_empty() && self.todos_state.selected().is_none() { self.todos_state.select(Some(0)); }
        if !self.filtered_mrs.is_empty() && self.mrs_state.selected().is_none() { self.mrs_state.select(Some(0)); }
        if !self.filtered_issues.is_empty() && self.issues_state.selected().is_none() { self.issues_state.select(Some(0)); }
        if !self.filtered_projects.is_empty() && self.projects_state.selected().is_none() { self.projects_state.select(Some(0)); }
        if !self.filtered_pipelines.is_empty() && self.pipelines_state.selected().is_none() { self.pipelines_state.select(Some(0)); }
        
        if self.dashboard_menu_state.selected().is_none() { self.dashboard_menu_state.select(Some(0)); }
        if self.explore_menu_state.selected().is_none() { self.explore_menu_state.select(Some(0)); }
        if self.project_menu_state.selected().is_none() { self.project_menu_state.select(Some(0)); }
        
        // Sticky Selection: Cap at the last item instead of jumping to the top (Some(0))
        // This prevents "random jumps" when background fetches return shorter lists.
        if let Some(i) = self.todos_state.selected() { if i >= self.filtered_todos.len() { if self.filtered_todos.is_empty() { self.todos_state.select(None); } else { self.todos_state.select(Some(self.filtered_todos.len().saturating_sub(1))); } } }
        if let Some(i) = self.mrs_state.selected() { if i >= self.filtered_mrs.len() { if self.filtered_mrs.is_empty() { self.mrs_state.select(None); } else { self.mrs_state.select(Some(self.filtered_mrs.len().saturating_sub(1))); } } }
        if let Some(i) = self.issues_state.selected() { if i >= self.filtered_issues.len() { if self.filtered_issues.is_empty() { self.issues_state.select(None); } else { self.issues_state.select(Some(self.filtered_issues.len().saturating_sub(1))); } } }
        if let Some(i) = self.projects_state.selected() { if i >= self.filtered_projects.len() { if self.filtered_projects.is_empty() { self.projects_state.select(None); } else { self.projects_state.select(Some(self.filtered_projects.len().saturating_sub(1))); } } }
        if let Some(i) = self.pipelines_state.selected() { if i >= self.filtered_pipelines.len() { if self.filtered_pipelines.is_empty() { self.pipelines_state.select(None); } else { self.pipelines_state.select(Some(self.filtered_pipelines.len().saturating_sub(1))); } } }
    }

    pub async fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::MouseEventKind;
        match mouse.kind {
            MouseEventKind::ScrollDown => { self.move_down(); let _ = self.update_preview().await; }
            MouseEventKind::ScrollUp => { self.move_up(); let _ = self.update_preview().await; }
            _ => {}
        }
    }

    pub async fn execute_help_action(&mut self, action: HelpAction) {
        match action {
            HelpAction::Quit => self.should_quit = true,
            HelpAction::SwitchProject => self.switch_project(),
            HelpAction::Back => { self.view_mode = ViewMode::List; self.active_panel = Panel::Dashboard; }
            HelpAction::Select => { self.enter().await; }
            HelpAction::ToggleHelp => self.toggle_help(),
            HelpAction::ToggleFocus => self.toggle_focus(),
            HelpAction::Filter => {
                self.input_mode = true;
                self.input_target = InputTarget::Filter;
                self.show_filter_input = true;
                self.cursor_index = self.filter_input.chars().count();
            }
            HelpAction::OpenBrowser => self.open_in_browser(),
            _ => {}
        }
        self.show_help_popup = false;
    }
    pub async fn back(&mut self) { self.view_mode = ViewMode::List; self.active_panel = Panel::Dashboard; }
    pub fn open_in_browser(&self) {}
    pub fn parse_job_log(&mut self) {}
    pub fn update_menu_items(&mut self) {}
    pub async fn check_pagination(&mut self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    pub fn get_input_buffer(&self) -> &String { match self.input_target { InputTarget::Filter => &self.filter_input, _ => &self.reply_input } }
    pub fn get_input_buffer_mut(&mut self) -> &mut String { match self.input_target { InputTarget::Filter => &mut self.filter_input, _ => &mut self.reply_input } }
    pub async fn enter(&mut self) { 
        match self.input_target {
            InputTarget::EditTitle | InputTarget::EditDescription => { if self.selected_issue.is_some() { let _ = self.update_issue().await; } else if self.selected_mr.is_some() { let _ = self.update_mr().await; } }
            InputTarget::Reply => { let _ = self.post_reply().await; }
            _ => {}
        }
    }
    pub async fn post_reply(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.reply_input.is_empty() { return Ok(()); }
        if let Some(mr) = &self.selected_mr {
            if let Some(disc_idx) = self.discussion_state.selected() {
                if let Some(disc) = self.discussions.get(disc_idx) {
                    let endpoint = AddDiscussionNote { project: mr.project_id, merge_request: mr.iid, discussion: disc.id.clone(), body: self.reply_input.clone() };
                    let _: serde_json::Value = endpoint.query_async(&self.gitlab).await?;
                    self.reply_input.clear(); self.input_mode = false; let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                }
            }
        } else if let Some(issue) = &self.selected_issue {
            if let Some(disc_idx) = self.discussion_state.selected() {
                if let Some(disc) = self.discussions.get(disc_idx) {
                    let endpoint = AddIssueDiscussionNote { project: issue.project_id, issue: issue.iid, discussion: disc.id.clone(), body: self.reply_input.clone() };
                    let _: serde_json::Value = endpoint.query_async(&self.gitlab).await?;
                    self.reply_input.clear(); self.input_mode = false; let _ = self.fetch_issue_discussions(issue.project_id, issue.iid).await;
                }
            }
        }
        Ok(())
    }
    pub fn toggle_focus(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Main | Panel::MainDetail => match self.active_context {
                Context::Dashboard => Panel::Dashboard,
                Context::Explore => Panel::Explore,
                Context::Project => Panel::Project,
            },
            _ => Panel::Main,
        };
    }
    pub fn switch_project(&mut self) {
        self.show_project_popup = true;
        let _ = self.fetch_projects();
    }

    pub async fn select_project(&mut self) {
        if let Some(i) = self.projects_state.selected() {
            if let Some(project) = self.projects.get(i).cloned() {
                self.selected_project = Some(project);
                self.show_project_popup = false;
                self.active_panel = Panel::Project;
                self.active_context = Context::Project;
                self.project_menu_state.select(Some(0));
                self.active_selection = MenuSelection::Pipelines;
                
                self.selected_mr = None;
                self.selected_issue = None;
                self.selected_pipeline = None;
                self.discussions.clear();
                self.jobs.clear();
                
                self.merge_requests.clear();
                self.issues.clear();
                self.pipelines.clear();
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
        if let Some(n) = name {
            let display_name = if let Some(emoji) = emojis::get_by_shortcode(n) { emoji.as_str().to_string() } else { n.to_string() };
            reactions.push(AwardEmoji { name: display_name });
        }
    }
    reactions
}
