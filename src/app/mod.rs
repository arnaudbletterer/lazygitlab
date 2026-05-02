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

impl App {
    pub fn new(gitlab: AsyncGitlab) -> Self {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        
        let mut menu_state = ListState::default();
        menu_state.select(Some(1));
        
        let mut projects_state = ListState::default();
        projects_state.select(Some(0));

        let mut mrs_state = ListState::default();
        mrs_state.select(Some(0));

        let mut issues_state = ListState::default();
        issues_state.select(Some(0));

        let mut pipelines_state = ListState::default();
        pipelines_state.select(Some(0));

        let mut jobs_state = ListState::default();
        jobs_state.select(Some(0));

        let mut discussion_state = ListState::default();
        discussion_state.select(Some(0));

        let mut mr_diff_state = ListState::default();
        mr_diff_state.select(Some(0));

        let mut todos_state = ListState::default();
        todos_state.select(Some(0));

        let mut groups_state = ListState::default();
        groups_state.select(Some(0));

        Self {
            should_quit: false,
            menu_state,
            menu_items: vec![
                (" [DASHBOARD] ".to_string(), MenuSelection::Todos),
                (" Todos".to_string(), MenuSelection::Todos),
                (" Merge Requests".to_string(), MenuSelection::MergeRequests),
                (" Issues".to_string(), MenuSelection::Issues),
                (" [EXPLORE] ".to_string(), MenuSelection::Projects),
                (" Projects".to_string(), MenuSelection::Projects),
                (" Groups".to_string(), MenuSelection::Groups),
                (" [PROJECT] ".to_string(), MenuSelection::Pipelines),
                (" Pipelines".to_string(), MenuSelection::Pipelines),
            ],
            active_selection: MenuSelection::Todos,
            gitlab,
            projects: Vec::new(),
            projects_state,
            selected_project: None,
            show_project_popup: false,
            merge_requests: Vec::new(),
            mrs_state,
            issues: Vec::new(),
            issues_state,
            pipelines: Vec::new(),
            pipelines_state,
            jobs: Vec::new(),
            jobs_state,
            active_panel: Panel::Sidebar,
            view_mode: ViewMode::List,
            detail_section: DetailSection::Discussions,
            selected_metadata_field: MetadataField::Title,
            selected_mr: None,
            selected_issue: None,
            selected_pipeline: None,
            selected_job: None,
            mr_diffs: Vec::new(),
            mr_diff_state,
            job_log: String::new(),
            job_log_lines: Vec::new(),
            collapsed_sections: std::collections::HashSet::new(),
            job_log_scroll: 0,
            job_log_viewport_height: 0,
            follow_log: true,
            discussions: Vec::new(),
            discussion_state,
            input_mode: false,
            input_target: InputTarget::Reply,
            is_loading: false,
            reply_input: String::new(),
            cursor_index: 0,
            theme: Theme::load(),
            last_refresh: std::time::Instant::now(),
            tx,

            // Project Metadata
            project_labels: Vec::new(),
            project_milestones: Vec::new(),
            project_members: Vec::new(),

            // Popups
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

            // Pagination
            mrs_page: 1,
            mrs_has_more: true,
            issues_page: 1,
            issues_has_more: true,
            pipelines_page: 1,
            pipelines_has_more: true,

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
            groups_state,
            filtered_groups: Vec::new(),
            groups_page: 1,
            groups_has_more: true,

            filter_input: String::new(),
            show_filter_input: false,
            todos: Vec::new(),
            todos_state,
            filtered_todos: Vec::new(),
            todos_page: 1,
            todos_has_more: true,
            filtered_mrs: Vec::new(),
            filtered_issues: Vec::new(),
            filtered_pipelines: Vec::new(),
            last_error: None,
            status_message: None,

            last_fetch_todos: None,
            last_fetch_mrs: None,
            last_fetch_issues: None,
            last_fetch_projects: None,
            last_fetch_groups: None,
            last_fetch_pipelines: None,
        }
    }

    pub fn process_updates(&mut self, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
        while let Ok(update) = rx.try_recv() {
            match update {
                DataUpdate::Todos { data, page, has_more } => { 
                    self.todos = data; self.todos_page = page; self.todos_has_more = has_more;
                    self.filter_lists(); self.last_fetch_todos = Some(std::time::Instant::now()); 
                    self.status_message = None;
                }
                DataUpdate::AppendTodos(mut data) => {
                    self.todos.append(&mut data); self.todos_page += 1; self.todos_has_more = data.len() >= 20;
                    self.filter_lists(); self.last_fetch_todos = Some(std::time::Instant::now());
                    self.status_message = None;
                }
                DataUpdate::Projects { data, page, has_more } => { 
                    self.projects = data; self.projects_page = page; self.projects_has_more = has_more;
                    self.filter_lists(); self.last_fetch_projects = Some(std::time::Instant::now()); 
                    self.status_message = None;
                }
                DataUpdate::AppendProjects(mut data) => {
                    self.projects.append(&mut data); self.projects_page += 1; self.projects_has_more = data.len() >= 20;
                    self.filter_lists(); self.last_fetch_projects = Some(std::time::Instant::now());
                    self.status_message = None;
                }
                DataUpdate::Groups { data, page, has_more } => { 
                    self.groups = data; self.groups_page = page; self.groups_has_more = has_more;
                    self.filter_lists(); self.last_fetch_groups = Some(std::time::Instant::now()); 
                    self.status_message = None;
                }
                DataUpdate::AppendGroups(mut data) => {
                    self.groups.append(&mut data); self.groups_page += 1; self.groups_has_more = data.len() >= 20;
                    self.filter_lists(); self.last_fetch_groups = Some(std::time::Instant::now());
                    self.status_message = None;
                }
                DataUpdate::GlobalMRs { data, page, has_more } | DataUpdate::ProjectMRs { data, page, has_more } => { 
                    self.merge_requests = data; self.mrs_page = page; self.mrs_has_more = has_more;
                    self.filter_lists(); self.last_fetch_mrs = Some(std::time::Instant::now()); 
                    self.status_message = None;
                }
                DataUpdate::ClearGlobalMRs => {
                    self.merge_requests.clear(); self.filter_lists();
                }
                DataUpdate::AppendGlobalMRs(mut data) => {
                    self.merge_requests.append(&mut data); self.filter_lists(); self.last_fetch_mrs = Some(std::time::Instant::now());
                    self.status_message = None;
                }
                DataUpdate::GlobalIssues { data, page, has_more } | DataUpdate::ProjectIssues { data, page, has_more } => { 
                    self.issues = data; self.issues_page = page; self.issues_has_more = has_more;
                    self.filter_lists(); self.last_fetch_issues = Some(std::time::Instant::now()); 
                    self.status_message = None;
                }
                DataUpdate::ClearGlobalIssues => {
                    self.issues.clear(); self.filter_lists();
                }
                DataUpdate::AppendGlobalIssues(mut data) => {
                    self.issues.append(&mut data); self.filter_lists(); self.last_fetch_issues = Some(std::time::Instant::now());
                    self.status_message = None;
                }
                DataUpdate::ProjectPipelines { data, page, has_more } => { 
                    self.pipelines = data; self.pipelines_page = page; self.pipelines_has_more = has_more;
                    self.filter_lists(); self.last_fetch_pipelines = Some(std::time::Instant::now()); 
                    self.status_message = None;
                }
                DataUpdate::Discussions(data) | DataUpdate::IssueDiscussions(data) => { self.discussions = data; self.status_message = None; }
                DataUpdate::Jobs(data) => { self.jobs = data; self.status_message = None; }
                DataUpdate::Log(data) => { self.job_log = data; self.parse_job_log(); self.status_message = None; }
                DataUpdate::Diffs(data) => { self.mr_diffs = data; self.status_message = None; }
                DataUpdate::Metadata { labels, milestones, members } => {
                    self.project_labels = labels;
                    self.project_milestones = milestones;
                    self.project_members = members;
                    self.status_message = None;
                }
                DataUpdate::Loading(b) => { self.is_loading = b; if !b { self.status_message = None; } }
                DataUpdate::Error(e) => { self.last_error = Some(e); self.is_loading = false; self.status_message = None; }
            }
        }
    }

    pub async fn initial_setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_loading { return Ok(()); }
        self.is_loading = true;
        self.last_error = None;
        self.status_message = Some("Initializing Dashboard...".into());
        
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        
        tokio::spawn(async move {
            let ep_todos = Todos { page: Some(1), per_page: Some(50) };
            let ep_projects = PagedProjects { membership: true, page: 1, per_page: 50 };
            let ep_groups = Groups { page: Some(1), per_page: Some(50) };

            let h1 = {
                let c = client.clone(); let t = tx.clone();
                tokio::spawn(async move {
                    let res: Result<Vec<Todo>, _> = ep_todos.query_async(&c).await;
                    match res {
                        Ok(data) => { let has_more = data.len() == 50; let _ = t.send(DataUpdate::Todos { data, page: 1, has_more }); }
                        Err(e) => { let _ = t.send(DataUpdate::Error(format!("Todos Error: {}", e))); }
                    }
                })
            };

            let h2 = {
                let c = client.clone(); let t = tx.clone();
                tokio::spawn(async move {
                    let res: Result<Vec<Project>, _> = ep_projects.query_async(&c).await;
                    match res {
                        Ok(data) => { let has_more = data.len() == 50; let _ = t.send(DataUpdate::Projects { data, page: 1, has_more }); }
                        Err(e) => { let _ = t.send(DataUpdate::Error(format!("Projects Error: {}", e))); }
                    }
                })
            };

            let h3 = {
                let c = client.clone(); let t = tx.clone();
                tokio::spawn(async move {
                    let res: Result<Vec<Group>, _> = ep_groups.query_async(&c).await;
                    match res {
                        Ok(data) => { let has_more = data.len() == 50; let _ = t.send(DataUpdate::Groups { data, page: 1, has_more }); }
                        Err(e) => { let _ = t.send(DataUpdate::Error(format!("Groups Error: {}", e))); }
                    }
                })
            };

            let _ = tokio::join!(h1, h2, h3);
            let _ = tx.send(DataUpdate::Loading(false));
        });

        Ok(())
    }

    pub async fn refresh(&mut self) {
        if self.is_loading { return; }
        self.last_error = None;

        let now = std::time::Instant::now();
        let cache_duration = std::time::Duration::from_secs(30);

        if self.selected_project.is_none() {
            let needs_fetch = match self.active_selection {
                MenuSelection::Todos => self.last_fetch_todos.map_or(true, |t| now.duration_since(t) > cache_duration),
                MenuSelection::MergeRequests => self.last_fetch_mrs.map_or(true, |t| now.duration_since(t) > cache_duration),
                MenuSelection::Issues => self.last_fetch_issues.map_or(true, |t| now.duration_since(t) > cache_duration),
                MenuSelection::Projects => self.last_fetch_projects.map_or(true, |t| now.duration_since(t) > cache_duration),
                MenuSelection::Groups => self.last_fetch_groups.map_or(true, |t| now.duration_since(t) > cache_duration),
                _ => false,
            };

            if !needs_fetch { return; }

            self.is_loading = true;
            self.last_refresh = now;
            match self.active_selection {
                MenuSelection::Todos => { self.status_message = Some("Fetching Todos...".into()); let _ = self.fetch_todos().await; }
                MenuSelection::MergeRequests => { self.status_message = Some("Fetching Merge Requests...".into()); let _ = self.fetch_global_merge_requests().await; }
                MenuSelection::Issues => { self.status_message = Some("Fetching Issues...".into()); let _ = self.fetch_global_issues().await; }
                MenuSelection::Projects => { self.status_message = Some("Fetching Projects...".into()); let _ = self.fetch_projects().await; }
                MenuSelection::Groups => { self.status_message = Some("Fetching Groups...".into()); let _ = self.fetch_groups().await; }
                _ => { self.is_loading = false; }
            }
            return;
        }

        let pid = self.selected_project.as_ref().unwrap().id;
        
        let needs_fetch = match self.view_mode {
            ViewMode::List => match self.active_selection {
                MenuSelection::MergeRequests => self.last_fetch_mrs.map_or(true, |t| now.duration_since(t) > cache_duration),
                MenuSelection::Issues => self.last_fetch_issues.map_or(true, |t| now.duration_since(t) > cache_duration),
                MenuSelection::Pipelines => self.last_fetch_pipelines.map_or(true, |t| now.duration_since(t) > cache_duration),
                _ => false,
            },
            _ => true,
        };

        if !needs_fetch { return; }

        self.is_loading = true;
        self.last_refresh = std::time::Instant::now();
        
        let client = self.gitlab.clone();
        let tx = self.tx.clone();

        if self.show_project_popup {
            tokio::spawn(async move {
                let ep = PagedProjects { membership: true, page: 1, per_page: 100 };
                let res: Result<Vec<Project>, _> = ep.query_async(&client).await;
                match res {
                    Ok(data) => { let _ = tx.send(DataUpdate::Projects { data, page: 1, has_more: true }); }
                    Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
            return;
        }

        match self.view_mode {
            ViewMode::List => match self.active_selection {
                MenuSelection::MergeRequests => {
                    tokio::spawn(async move {
                        let ep = PagedMergeRequests { project: pid, page: 1, per_page: 50 };
                        let res: Result<Vec<MergeRequest>, _> = ep.query_async(&client).await;
                        match res {
                            Ok(data) => { let _ = tx.send(DataUpdate::ProjectMRs { data, page: 1, has_more: true }); }
                            Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                        }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
                }
                MenuSelection::Issues => {
                    tokio::spawn(async move {
                        let ep = PagedIssues { project: pid, page: 1, per_page: 50 };
                        let res: Result<Vec<Issue>, _> = ep.query_async(&client).await;
                        match res {
                            Ok(data) => { let _ = tx.send(DataUpdate::ProjectIssues { data, page: 1, has_more: true }); }
                            Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                        }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
                }
                MenuSelection::Pipelines => {
                    tokio::spawn(async move {
                        let ep = ProjectPipelines { project: pid, page: Some(1), per_page: Some(50) };
                        let res: Result<Vec<Pipeline>, _> = ep.query_async(&client).await;
                        match res {
                            Ok(data) => { let _ = tx.send(DataUpdate::ProjectPipelines { data, page: 1, has_more: true }); }
                            Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                        }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
                }
                _ => { self.is_loading = false; }
            },
            ViewMode::Detail => {
                if let Some(mr) = self.selected_mr.clone() {
                    tokio::spawn(async move {
                        let ep = MergeRequestDiscussions { project: pid, merge_request: mr.iid };
                        let res: Result<Vec<Discussion>, _> = ep.query_async(&client).await;
                        match res {
                            Ok(data) => { let _ = tx.send(DataUpdate::Discussions(data)); }
                            Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                        }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
                } else if let Some(issue) = self.selected_issue.clone() {
                    tokio::spawn(async move {
                        let ep = IssueDiscussions { project: pid, issue: issue.iid };
                        let res: Result<Vec<Discussion>, _> = ep.query_async(&client).await;
                        match res {
                            Ok(data) => { let _ = tx.send(DataUpdate::IssueDiscussions(data)); }
                            Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                        }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
                } else if let Some(pipeline) = self.selected_pipeline.clone() {
                    tokio::spawn(async move {
                        let ep = ProjectPipelineJobs { project: pid, pipeline: pipeline.id };
                        let res: Result<Vec<Job>, _> = ep.query_async(&client).await;
                        match res {
                            Ok(data) => { let _ = tx.send(DataUpdate::Jobs(data)); }
                            Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                        }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
                } else { self.is_loading = false; }
            }
            ViewMode::JobLog => {
                if let Some(job) = self.selected_job.clone() {
                    tokio::spawn(async move {
                        let ep = JobTrace { project: pid, job: job.id };
                        let res: Result<Vec<u8>, _> = gitlab::api::raw(ep).query_async(&client).await;
                        match res {
                            Ok(data) => { let _ = tx.send(DataUpdate::Log(String::from_utf8_lossy(&data).to_string())); }
                            Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                        }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
                } else { self.is_loading = false; }
            }
            ViewMode::Diff => {
                if let Some(mr) = self.selected_mr.clone() {
                    tokio::spawn(async move {
                        let ep = MergeRequestDiffs { project: pid, merge_request: mr.iid };
                        let res: Result<Vec<MRDiff>, _> = ep.query_async(&client).await;
                        match res {
                            Ok(data) => { let _ = tx.send(DataUpdate::Diffs(data)); }
                            Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                        }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
                } else { self.is_loading = false; }
            }
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help_popup = !self.show_help_popup;
        if self.show_help_popup {
            self.help_filter_input.clear();
            self.show_help_filter = false;
            self.filter_help();
            self.help_state.select(Some(0));
        }
    }

    pub fn get_contextual_help(&self) -> Vec<HelpItem> {
        let mut items = vec![
            HelpItem { shortcut: "j/k or Down/Up".into(), description: "Move Selection".into(), action: HelpAction::None },
            HelpItem { shortcut: "Enter".into(), description: "Select/Open".into(), action: HelpAction::Select },
            HelpItem { shortcut: "Esc/Backspace".into(), description: "Back/Close".into(), action: HelpAction::Back },
            HelpItem { shortcut: "Ctrl-R".into(), description: "Switch Project".into(), action: HelpAction::SwitchProject },
            HelpItem { shortcut: "q".into(), description: "Quit".into(), action: HelpAction::Quit },
            HelpItem { shortcut: "?".into(), description: "Toggle Help".into(), action: HelpAction::ToggleHelp },
        ];

        if self.view_mode == ViewMode::List {
            items.push(HelpItem { shortcut: "h/l or Left/Right/Tab".into(), description: "Toggle Menu/List Focus".into(), action: HelpAction::ToggleFocus });
            items.push(HelpItem { shortcut: "/".into(), description: "Filter List".into(), action: HelpAction::Filter });
            items.push(HelpItem { shortcut: "v or o".into(), description: "Open in Browser".into(), action: HelpAction::OpenBrowser });
            if self.active_selection == MenuSelection::MergeRequests {
                items.push(HelpItem { shortcut: "d".into(), description: "View Diff".into(), action: HelpAction::None });
            }
            if self.active_selection == MenuSelection::Pipelines {
                items.push(HelpItem { shortcut: "p".into(), description: "Launch New Pipeline".into(), action: HelpAction::LaunchPipeline });
                items.push(HelpItem { shortcut: "c".into(), description: "Cancel Selected Pipeline".into(), action: HelpAction::CancelPipeline });
            }
        } else if self.view_mode == ViewMode::Detail {
            items.push(HelpItem { shortcut: "v or o".into(), description: "Open in Browser".into(), action: HelpAction::OpenBrowser });
            if self.selected_mr.is_some() || self.selected_issue.is_some() {
                items.push(HelpItem { shortcut: "n".into(), description: "New Comment/Note".into(), action: HelpAction::NewComment });
                items.push(HelpItem { shortcut: "Space".into(), description: "Toggle Discussion Collapse".into(), action: HelpAction::ToggleCollapse });
            }
            if let Some(mr) = &self.selected_mr {
                items.push(HelpItem { shortcut: "d".into(), description: "View Diff".into(), action: HelpAction::None });
                if mr.state == "opened" {
                    items.push(HelpItem { shortcut: "a".into(), description: "Approve MR".into(), action: HelpAction::None });
                    items.push(HelpItem { shortcut: "m".into(), description: "Merge MR".into(), action: HelpAction::None });
                }
                items.push(HelpItem { shortcut: "c".into(), description: "Toggle MR State".into(), action: HelpAction::None });
                items.push(HelpItem { shortcut: "s".into(), description: "Resolve/Unresolve Discussion".into(), action: HelpAction::ResolveDiscussion });
            }
            if self.selected_issue.is_some() {
                items.push(HelpItem { shortcut: "c".into(), description: "Toggle Issue State".into(), action: HelpAction::ToggleIssueState });
            }
            if self.selected_pipeline.is_some() {
                items.push(HelpItem { shortcut: "p".into(), description: "Play Manual Job".into(), action: HelpAction::PlayJob });
                items.push(HelpItem { shortcut: "r".into(), description: "Retry Job/Pipeline".into(), action: HelpAction::RetryJob });
                items.push(HelpItem { shortcut: "c".into(), description: "Cancel Job/Pipeline".into(), action: HelpAction::CancelJob });
            }
        } else if self.view_mode == ViewMode::JobLog {
            items.push(HelpItem { shortcut: "Tab".into(), description: "Toggle Section Collapse".into(), action: HelpAction::ToggleCollapse });
        }

        items
    }

    pub fn filter_help(&mut self) {
        let filter = self.help_filter_input.to_lowercase();
        let all_help = self.get_contextual_help();
        self.filtered_help = all_help.into_iter()
            .filter(|h| h.shortcut.to_lowercase().contains(&filter) || h.description.to_lowercase().contains(&filter))
            .collect();
        
        if let Some(i) = self.help_state.selected() {
            if i >= self.filtered_help.len() && !self.filtered_help.is_empty() { self.help_state.select(Some(0)); }
            else if self.filtered_help.is_empty() { self.help_state.select(None); }
        }
    }

    pub async fn execute_help_action(&mut self, action: HelpAction) {
        match action {
            HelpAction::Quit => self.should_quit = true,
            HelpAction::SwitchProject => self.switch_project(),
            HelpAction::Back => { self.back().await; }
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
            HelpAction::LaunchPipeline => {
                if self.view_mode == ViewMode::List && self.active_selection == MenuSelection::Pipelines {
                    self.show_pipeline_launch_popup().await;
                }
            }
            HelpAction::CancelPipeline => {
                if let Some(proj) = &self.selected_project {
                    if self.view_mode == ViewMode::List && self.active_selection == MenuSelection::Pipelines {
                        if let Some(i) = self.pipelines_state.selected() {
                            if let Some(p) = self.pipelines.get(i) { let _ = self.cancel_pipeline(proj.id, p.id).await; }
                        }
                    }
                }
            }
            HelpAction::PlayJob => {
                if let Some(proj) = &self.selected_project {
                    if let Some(i) = self.jobs_state.selected() {
                        if let Some(job) = self.jobs.get(i) {
                            if job.status == "manual" { let _ = self.play_job(proj.id, job.id).await; }
                        }
                    }
                }
            }
            HelpAction::RetryJob => {
                if let Some(proj) = &self.selected_project {
                    if let Some(i) = self.jobs_state.selected() {
                        if let Some(job) = self.jobs.get(i) { let _ = self.retry_job(proj.id, job.id).await; }
                    } else if let Some(pipeline) = &self.selected_pipeline { let _ = self.retry_pipeline(proj.id, pipeline.id).await; }
                }
            }
            HelpAction::CancelJob => {
                if let Some(proj) = &self.selected_project {
                    if let Some(i) = self.jobs_state.selected() {
                        if let Some(job) = self.jobs.get(i) { let _ = self.cancel_job(proj.id, job.id).await; }
                    } else if let Some(pipeline) = &self.selected_pipeline { let _ = self.cancel_pipeline(proj.id, pipeline.id).await; }
                }
            }
            HelpAction::NewComment => {
                self.input_mode = true;
                self.input_target = InputTarget::NewComment;
                self.reply_input.clear();
                self.cursor_index = 0;
            }
            HelpAction::ResolveDiscussion => { let _ = self.resolve_discussion().await; }
            HelpAction::ToggleIssueState => { let _ = self.toggle_issue_state().await; }
            HelpAction::ToggleCollapse => {
                if self.view_mode == ViewMode::JobLog { self.toggle_job_section(); }
                else { self.toggle_discussion(); }
            }
            _ => {}
        }
        self.show_help_popup = false;
    }

    pub fn move_down(&mut self) {
        if self.is_loading { return; }
        if self.show_pipeline_popup {
            if self.show_branches_dropdown {
                if !self.filtered_branches.is_empty() {
                    let i = match self.branches_state.selected() {
                        Some(i) => if i >= self.filtered_branches.len().saturating_sub(1) { 0 } else { i + 1 },
                        None => 0,
                    };
                    self.branches_state.select(Some(i));
                }
            } else if self.input_target == InputTarget::PipelineVariableList {
                if !self.pipeline_variables.is_empty() {
                    let i = match self.pipeline_variables_state.selected() {
                        Some(i) => if i >= self.pipeline_variables.len().saturating_sub(1) { 0 } else { i + 1 },
                        None => 0,
                    };
                    self.pipeline_variables_state.select(Some(i));
                }
            }
            return;
        }
        if self.show_label_popup {
            let i = match self.label_state.selected() {
                Some(i) => if i >= self.project_labels.len().saturating_sub(1) { 0 } else { i + 1 },
                None => 0,
            };
            self.label_state.select(Some(i));
            return;
        }
        if self.show_assignee_popup {
            let i = match self.assignee_state.selected() {
                Some(i) => if i >= self.project_members.len().saturating_sub(1) { 0 } else { i + 1 },
                None => 0,
            };
            self.assignee_state.select(Some(i));
            return;
        }
        if self.show_milestone_popup {
            let i = match self.milestone_state.selected() {
                Some(i) => if i >= self.project_milestones.len() { 0 } else { i + 1 },
                None => 0,
            };
            self.milestone_state.select(Some(i));
            return;
        }
        if self.show_weight_popup {
            let i = match self.weight_state.selected() {
                Some(i) => if i >= 21 { 0 } else { i + 1 }, // Weight 0 to 20
                None => 0,
            };
            self.weight_state.select(Some(i));
            return;
        }
        if self.input_mode { return; }
        
        if self.show_project_popup {
            if !self.projects.is_empty() {
                let i = match self.projects_state.selected() {
                    Some(i) => if i >= self.projects.len().saturating_sub(1) { 0 } else { i + 1 },
                    None => 0,
                };
                self.projects_state.select(Some(i));
            }
            return;
        }

        match self.view_mode {
            ViewMode::List => {
                if self.active_panel == Panel::Sidebar {
                    let current = self.menu_state.selected().unwrap_or(0);
                    let mut next = if current >= self.menu_items.len() - 1 { 0 } else { current + 1 };
                    while self.menu_items[next].0.starts_with(" [") && next != current {
                        next = if next >= self.menu_items.len() - 1 { 0 } else { next + 1 };
                    }
                    self.menu_state.select(Some(next));
                    self.active_selection = self.menu_items[next].1;
                } else {
                    match self.active_selection {
                        MenuSelection::Todos => {
                            if !self.filtered_todos.is_empty() {
                                let current = self.todos_state.selected().unwrap_or(0);
                                let mut next = if current >= self.filtered_todos.len().saturating_sub(1) { 0 } else { current + 1 };
                                while matches!(self.filtered_todos[next], GroupedItem::Header(_)) && next != current {
                                    next = if next >= self.filtered_todos.len().saturating_sub(1) { 0 } else { next + 1 };
                                }
                                self.todos_state.select(Some(next));
                            }
                        }
                        MenuSelection::MergeRequests => {
                            if !self.filtered_mrs.is_empty() {
                                let current = self.mrs_state.selected().unwrap_or(0);
                                let mut next = if current >= self.filtered_mrs.len() - 1 { 0 } else { current + 1 };
                                while matches!(self.filtered_mrs[next], GroupedItem::Header(_)) && next != current {
                                    next = if next >= self.filtered_mrs.len() - 1 { 0 } else { next + 1 };
                                }
                                self.mrs_state.select(Some(next));
                            }
                        }
                        MenuSelection::Issues => {
                            if !self.filtered_issues.is_empty() {
                                let current = self.issues_state.selected().unwrap_or(0);
                                let mut next = if current >= self.filtered_issues.len().saturating_sub(1) { 0 } else { current + 1 };
                                while matches!(self.filtered_issues[next], GroupedItem::Header(_)) && next != current {
                                    next = if next >= self.filtered_issues.len().saturating_sub(1) { 0 } else { next + 1 };
                                }
                                self.issues_state.select(Some(next));
                            }
                        }
                        MenuSelection::Pipelines => {
                            if !self.pipelines.is_empty() {
                                let i = match self.pipelines_state.selected() {
                                    Some(i) => if i >= self.pipelines.len().saturating_sub(1) { 0 } else { i + 1 },
                                    None => 0,
                                };
                                self.pipelines_state.select(Some(i));
                            }
                        }
                        MenuSelection::Projects => {
                            if !self.projects.is_empty() {
                                let i = match self.projects_state.selected() {
                                    Some(i) => if i >= self.projects.len().saturating_sub(1) { 0 } else { i + 1 },
                                    None => 0,
                                };
                                self.projects_state.select(Some(i));
                            }
                        }
                        MenuSelection::Groups => {
                            if !self.groups.is_empty() {
                                let i = match self.groups_state.selected() {
                                    Some(i) => if i >= self.groups.len().saturating_sub(1) { 0 } else { i + 1 },
                                    None => 0,
                                };
                                self.groups_state.select(Some(i));
                            }
                        }
                    }
                }
            }
            ViewMode::JobLog => {
                let lines = self.get_visible_job_log_lines_count();
                let inner_height = self.job_log_viewport_height;
                if self.job_log_scroll < lines.saturating_sub(inner_height) {
                    self.job_log_scroll += 1;
                }
                if self.job_log_scroll >= lines.saturating_sub(inner_height) {
                    self.follow_log = true;
                }
            }
            ViewMode::Detail => {
                if self.selected_pipeline.is_some() {
                    if !self.jobs.is_empty() {
                        let i = match self.jobs_state.selected() {
                            Some(i) => if i >= self.jobs.len().saturating_sub(1) { 0 } else { i + 1 },
                            None => 0,
                        };
                        self.jobs_state.select(Some(i));
                    }
                    return;
                }
                match self.detail_section {
                    DetailSection::Metadata => {
                        self.selected_metadata_field = match self.selected_metadata_field {
                            MetadataField::Title => MetadataField::Labels,
                            MetadataField::Labels => MetadataField::Assignees,
                            MetadataField::Assignees => MetadataField::Milestone,
                            MetadataField::Milestone => {
                                if self.selected_issue.is_some() { MetadataField::DueDate }
                                else { MetadataField::Description }
                            }
                            MetadataField::DueDate => MetadataField::Weight,
                            MetadataField::Weight => MetadataField::Confidential,
                            MetadataField::Confidential => MetadataField::Description,
                            MetadataField::Description => {
                                self.detail_section = DetailSection::Discussions;
                                self.discussion_state.select(Some(0));
                                MetadataField::Title
                            }
                        };
                    }
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
                }
            }
            ViewMode::Diff => {
                if !self.mr_diffs.is_empty() {
                    let i = match self.mr_diff_state.selected() {
                        Some(i) => if i >= self.mr_diffs.len().saturating_sub(1) { 0 } else { i + 1 },
                        None => 0,
                    };
                    self.mr_diff_state.select(Some(i));
                }
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.is_loading { return; }
        if self.show_pipeline_popup {
            if self.input_target == InputTarget::PipelineRefSearch {
                if !self.filtered_branches.is_empty() {
                    let i = match self.branches_state.selected() {
                        Some(i) => if i == 0 { self.filtered_branches.len().saturating_sub(1) } else { i - 1 },
                        None => 0,
                    };
                    self.branches_state.select(Some(i));
                }
            } else if !self.pipeline_variables.is_empty() {
                let i = match self.pipeline_variables_state.selected() {
                    Some(i) => if i == 0 { self.pipeline_variables.len().saturating_sub(1) } else { i - 1 },
                    None => 0,
                };
                self.pipeline_variables_state.select(Some(i));
            }
            return;
        }
        if self.show_label_popup {
            let i = match self.label_state.selected() {
                Some(i) => if i == 0 { self.project_labels.len().saturating_sub(1) } else { i - 1 },
                None => 0,
            };
            self.label_state.select(Some(i));
            return;
        }
        if self.show_assignee_popup {
            let i = match self.assignee_state.selected() {
                Some(i) => if i == 0 { self.project_members.len().saturating_sub(1) } else { i - 1 },
                None => 0,
            };
            self.assignee_state.select(Some(i));
            return;
        }
        if self.show_milestone_popup {
            let i = match self.milestone_state.selected() {
                Some(i) => if i == 0 { self.project_milestones.len() } else { i - 1 },
                None => 0,
            };
            self.milestone_state.select(Some(i));
            return;
        }
        if self.show_weight_popup {
            let i = match self.weight_state.selected() {
                Some(i) => if i == 0 { 21 } else { i - 1 },
                None => 0,
            };
            self.weight_state.select(Some(i));
            return;
        }
        if self.input_mode { return; }
        
        if self.show_project_popup {
            if !self.projects.is_empty() {
                let i = match self.projects_state.selected() {
                    Some(i) => if i == 0 { self.projects.len().saturating_sub(1) } else { i - 1 },
                    None => 0,
                };
                self.projects_state.select(Some(i));
            }
            return;
        }

        match self.view_mode {
            ViewMode::List => {
                if self.active_panel == Panel::Sidebar {
                    let current = self.menu_state.selected().unwrap_or(0);
                    let mut prev = if current == 0 { self.menu_items.len() - 1 } else { current - 1 };
                    while self.menu_items[prev].0.starts_with(" [") && prev != current {
                        prev = if prev == 0 { self.menu_items.len() - 1 } else { prev - 1 };
                    }
                    self.menu_state.select(Some(prev));
                    self.active_selection = self.menu_items[prev].1;
                } else {
                    match self.active_selection {
                        MenuSelection::Todos => {
                            if !self.filtered_todos.is_empty() {
                                let current = self.todos_state.selected().unwrap_or(0);
                                let mut prev = if current == 0 { self.filtered_todos.len().saturating_sub(1) } else { current - 1 };
                                while matches!(self.filtered_todos[prev], GroupedItem::Header(_)) && prev != current {
                                    prev = if prev == 0 { self.filtered_todos.len().saturating_sub(1) } else { prev - 1 };
                                }
                                self.todos_state.select(Some(prev));
                            }
                        }
                        MenuSelection::MergeRequests => {
                            if !self.filtered_mrs.is_empty() {
                                let current = self.mrs_state.selected().unwrap_or(0);
                                let mut prev = if current == 0 { self.filtered_mrs.len().saturating_sub(1) } else { current - 1 };
                                while matches!(self.filtered_mrs[prev], GroupedItem::Header(_)) && prev != current {
                                    prev = if prev == 0 { self.filtered_mrs.len().saturating_sub(1) } else { prev - 1 };
                                }
                                self.mrs_state.select(Some(prev));
                            }
                        }
                        MenuSelection::Issues => {
                            if !self.filtered_issues.is_empty() {
                                let current = self.issues_state.selected().unwrap_or(0);
                                let mut prev = if current == 0 { self.filtered_issues.len().saturating_sub(1) } else { current - 1 };
                                while matches!(self.filtered_issues[prev], GroupedItem::Header(_)) && prev != current {
                                    prev = if prev == 0 { self.filtered_issues.len().saturating_sub(1) } else { prev - 1 };
                                }
                                self.issues_state.select(Some(prev));
                            }
                        }
                        MenuSelection::Pipelines => {
                            if !self.pipelines.is_empty() {
                                let i = match self.pipelines_state.selected() {
                                    Some(i) => if i == 0 { self.pipelines.len().saturating_sub(1) } else { i - 1 },
                                    None => 0,
                                };
                                self.pipelines_state.select(Some(i));
                            }
                        }
                        MenuSelection::Projects => {
                            if !self.projects.is_empty() {
                                let i = match self.projects_state.selected() {
                                    Some(i) => if i == 0 { self.projects.len().saturating_sub(1) } else { i - 1 },
                                    None => 0,
                                };
                                self.projects_state.select(Some(i));
                            }
                        }
                        MenuSelection::Groups => {
                            if !self.groups.is_empty() {
                                let i = match self.groups_state.selected() {
                                    Some(i) => if i == 0 { self.groups.len().saturating_sub(1) } else { i - 1 },
                                    None => 0,
                                };
                                self.groups_state.select(Some(i));
                            }
                        }
                    }
                }
            }
            ViewMode::JobLog => {
                if self.job_log_scroll > 0 {
                    self.job_log_scroll -= 1;
                    self.follow_log = false;
                }
            }
            ViewMode::Detail => {
                if self.selected_pipeline.is_some() {
                    if !self.jobs.is_empty() {
                        let i = match self.jobs_state.selected() {
                            Some(i) => if i == 0 { self.jobs.len().saturating_sub(1) } else { i - 1 },
                            None => 0,
                        };
                        self.jobs_state.select(Some(i));
                    }
                    return;
                }
                match self.detail_section {
                    DetailSection::Metadata => {
                        self.selected_metadata_field = match self.selected_metadata_field {
                            MetadataField::Title => {
                                self.detail_section = DetailSection::Discussions;
                                if !self.discussions.is_empty() {
                                    self.discussion_state.select(Some(self.discussions.len().saturating_sub(1)));
                                }
                                MetadataField::Title
                            }
                            MetadataField::Labels => MetadataField::Title,
                            MetadataField::Assignees => MetadataField::Labels,
                            MetadataField::Milestone => MetadataField::Assignees,
                            MetadataField::DueDate => MetadataField::Milestone,
                            MetadataField::Weight => MetadataField::DueDate,
                            MetadataField::Confidential => MetadataField::Weight,
                            MetadataField::Description => {
                                if self.selected_issue.is_some() { MetadataField::Confidential }
                                else { MetadataField::Milestone }
                            }
                        };
                    }
                    DetailSection::Discussions => {
                        if !self.discussions.is_empty() {
                            let current = self.discussion_state.selected().unwrap_or(0);
                            if current == 0 {
                                self.detail_section = DetailSection::Metadata;
                                self.selected_metadata_field = MetadataField::Description;
                                self.discussion_state.select(None);
                            } else {
                                self.discussion_state.select(Some(current - 1));
                            }
                        } else {
                            self.detail_section = DetailSection::Metadata;
                            self.selected_metadata_field = MetadataField::Description;
                        }
                    }
                }
            }
            ViewMode::Diff => {
                if !self.mr_diffs.is_empty() {
                    let i = match self.mr_diff_state.selected() {
                        Some(i) => if i == 0 { self.mr_diffs.len().saturating_sub(1) } else { i - 1 },
                        None => 0,
                    };
                    self.mr_diff_state.select(Some(i));
                }
            }
        }
    }

    pub fn toggle_focus(&mut self) {
        if self.show_project_popup { return; }
        match self.view_mode {
            ViewMode::List => {
                self.active_panel = match self.active_panel {
                    Panel::Sidebar => Panel::Content,
                    Panel::Content => Panel::Sidebar,
                };
            }
            ViewMode::Detail => {
                self.detail_section = match self.detail_section {
                    DetailSection::Metadata => DetailSection::Discussions,
                    DetailSection::Discussions => DetailSection::Metadata,
                };
            }
            _ => {}
        }
    }

    pub fn page_down(&mut self) {
        if self.view_mode == ViewMode::JobLog {
            let lines = self.get_visible_job_log_lines_count();
            let inner_height = self.job_log_viewport_height;
            self.job_log_scroll = (self.job_log_scroll + inner_height).min(lines.saturating_sub(inner_height));
            if self.job_log_scroll >= lines.saturating_sub(inner_height) {
                self.follow_log = true;
            }
        }
    }

    pub fn page_up(&mut self) {
        if self.view_mode == ViewMode::JobLog {
            let inner_height = self.job_log_viewport_height;
            self.job_log_scroll = self.job_log_scroll.saturating_sub(inner_height);
            self.follow_log = false;
        }
    }

    pub async fn fetch_project_metadata(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            
            self.is_loading = true;
            tokio::spawn(async move {
                let ep_labels = ProjectLabels { project: pid };
                let ep_milestones = ProjectMilestones { project: pid };
                let ep_members = ProjectMembers { project: pid };
                
                let labels_res = ep_labels.query_async(&client);
                let milestones_res = ep_milestones.query_async(&client);
                let members_res = ep_members.query_async(&client);
                
                let (l, m, mem) = tokio::join!(labels_res, milestones_res, members_res);
                let labels = l.unwrap_or_default();
                let milestones = m.unwrap_or_default();
                let members = mem.unwrap_or_default();
                
                let _ = tx.send(DataUpdate::Metadata { labels, milestones, members });
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub fn toggle_discussion(&mut self) {
        if let Some(i) = self.discussion_state.selected() {
            if let Some(d) = self.discussions.get_mut(i) {
                d.collapsed = !d.collapsed;
            }
        }
    }

    pub async fn post_reply(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.reply_input.is_empty() { return Ok(()); }
        
        if let Some(mr) = &self.selected_mr {
            if let Some(disc_idx) = self.discussion_state.selected() {
                if let Some(disc) = self.discussions.get(disc_idx) {
                    let endpoint = AddDiscussionNote {
                        project: mr.project_id,
                        merge_request: mr.iid,
                        discussion: disc.id.clone(),
                        body: self.reply_input.clone(),
                    };
                    let _: serde_json::Value = endpoint.query_async(&self.gitlab).await?;
                    self.reply_input.clear();
                    self.input_mode = false;
                    let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                }
            }
        } else if let Some(issue) = &self.selected_issue {
            if let Some(disc_idx) = self.discussion_state.selected() {
                if let Some(disc) = self.discussions.get(disc_idx) {
                    let endpoint = AddIssueDiscussionNote {
                        project: issue.project_id,
                        issue: issue.iid,
                        discussion: disc.id.clone(),
                        body: self.reply_input.clone(),
                    };
                    let _: serde_json::Value = endpoint.query_async(&self.gitlab).await?;
                    self.reply_input.clear();
                    self.input_mode = false;
                    let _ = self.fetch_issue_discussions(issue.project_id, issue.iid).await;
                }
            }
        }
        Ok(())
    }

    pub async fn resolve_discussion(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            if let Some(disc_idx) = self.discussion_state.selected() {
                if let Some(disc) = self.discussions.get(disc_idx) {
                    let is_resolved = disc.resolved.unwrap_or(false);
                    let endpoint = ResolveDiscussion {
                        project: mr.project_id,
                        merge_request: mr.iid,
                        discussion: disc.id.clone(),
                        resolved: !is_resolved,
                    };
                    let _: serde_json::Value = endpoint.query_async(&self.gitlab).await?;
                    let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                }
            }
        } else if let Some(issue) = &self.selected_issue {
            if let Some(disc_idx) = self.discussion_state.selected() {
                if let Some(disc) = self.discussions.get(disc_idx) {
                    let is_resolved = disc.resolved.unwrap_or(false);
                    let endpoint = ResolveIssueDiscussion {
                        project: issue.project_id,
                        issue: issue.iid,
                        discussion: disc.id.clone(),
                        resolved: !is_resolved,
                    };
                    let _: serde_json::Value = endpoint.query_async(&self.gitlab).await?;
                    let _ = self.fetch_issue_discussions(issue.project_id, issue.iid).await;
                }
            }
        }
        Ok(())
    }

    pub async fn post_comment(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.reply_input.is_empty() { return Ok(()); }

        if let Some(issue) = &self.selected_issue {
            let iid = issue.iid;
            let project_id = issue.project_id;
            self.is_loading = true;
            let endpoint = AddIssueNote {
                project: project_id,
                issue: iid,
                body: self.reply_input.clone(),
            };
            let res: Result<serde_json::Value, _> = endpoint.query_async(&self.gitlab).await;
            if res.is_ok() {
                self.reply_input.clear();
                self.input_mode = false;
                let _ = self.fetch_issue_discussions(project_id, iid).await;
            }
            self.is_loading = false;
        } else if let Some(mr) = &self.selected_mr {
             let iid = mr.iid;
             let project_id = mr.project_id;
             self.is_loading = true;
             let endpoint = AddNote {
                 project: project_id,
                 merge_request: iid,
                 body: self.reply_input.clone(),
             };
             let res: Result<serde_json::Value, _> = endpoint.query_async(&self.gitlab).await;
             if res.is_ok() {
                 self.reply_input.clear();
                 self.input_mode = false;
                 let _ = self.fetch_discussions(project_id, iid).await;
             }
             self.is_loading = false;
        }
        Ok(())
    }

    pub async fn select_project(&mut self) {
        if let Some(i) = self.projects_state.selected() {
            if let Some(project) = self.projects.get(i) {
                self.selected_project = Some(project.clone());
                self.show_project_popup = false;
                self.active_selection = MenuSelection::MergeRequests;
                self.menu_state.select(Some(2)); // MRs is 2nd in list with headers
                self.active_panel = Panel::Sidebar;
                let _ = self.refresh().await;
                let _ = self.fetch_project_metadata().await;
            }
        }
    }

    pub fn switch_project(&mut self) {
        self.show_project_popup = true;
        self.active_panel = Panel::Content;
    }

    pub async fn enter(&mut self) {
        if self.input_mode {
            match self.input_target {
                InputTarget::Reply => { let _ = self.post_reply().await; }
                InputTarget::NewComment => { let _ = self.post_comment().await; }
                InputTarget::EditTitle | InputTarget::EditDescription => {
                    if self.selected_issue.is_some() { let _ = self.update_issue().await; }
                    else if self.selected_mr.is_some() { let _ = self.update_mr().await; }
                }
                InputTarget::PipelineRef => {
                    if self.show_branches_dropdown {
                        if let Some(i) = self.branches_state.selected() {
                            if let Some(branch) = self.filtered_branches.get(i) {
                                self.pipeline_ref_input = branch.clone();
                            }
                        }
                        self.show_branches_dropdown = false;
                        self.input_target = InputTarget::PipelineVariableKey;
                        self.cursor_index = self.pipeline_variable_key_input.chars().count();
                    } else {
                        self.show_branches_dropdown = true;
                    }
                }
                InputTarget::PipelineRefSearch => {
                    if let Some(i) = self.branches_state.selected() {
                        if let Some(branch) = self.filtered_branches.get(i) {
                             self.pipeline_ref_input = branch.clone();
                        }
                    }
                    self.show_branches_dropdown = false;
                    self.input_target = InputTarget::PipelineVariableKey;
                    self.cursor_index = self.pipeline_variable_key_input.chars().count();
                }
                InputTarget::PipelineVariableKey => {
                    self.input_target = InputTarget::PipelineVariableValue;
                    self.cursor_index = self.pipeline_variable_value_input.chars().count();
                }
                InputTarget::PipelineVariableValue => {
                    self.add_pipeline_variable();
                    self.input_target = InputTarget::PipelineVariableKey;
                    self.cursor_index = 0;
                }
                InputTarget::PipelineVariableList => {
                    self.input_target = InputTarget::PipelineRef;
                    self.cursor_index = self.pipeline_ref_input.chars().count();
                }
                InputTarget::Filter => {
                    self.input_mode = false;
                    self.show_filter_input = !self.filter_input.is_empty();
                }
                InputTarget::CreateIssueTitle => {
                    self.input_target = InputTarget::CreateIssueDescription;
                    self.cursor_index = self.create_issue_description.chars().count();
                }
                InputTarget::CreateIssueDescription => {
                    let _ = self.create_issue().await;
                }
                InputTarget::CreateMRTitle => {
                    self.input_target = InputTarget::CreateMRDescription;
                    self.cursor_index = self.create_mr_description.chars().count();
                }
                InputTarget::CreateMRDescription => {
                    self.input_target = InputTarget::CreateMRSourceBranch;
                    self.cursor_index = self.create_mr_source_branch.chars().count();
                }
                InputTarget::CreateMRSourceBranch => {
                    self.input_target = InputTarget::CreateMRTargetBranch;
                    self.cursor_index = self.create_mr_target_branch.chars().count();
                }
                InputTarget::CreateMRTargetBranch => {
                    let _ = self.create_mr().await;
                }
            }
            return;
        }

        if self.show_project_popup {
            self.select_project().await;
            return;
        }

        if self.show_pipeline_popup {
            if !self.input_mode {
                self.input_mode = true;
                self.input_target = InputTarget::PipelineRef;
                self.cursor_index = self.pipeline_ref_input.chars().count();
            }
            return;
        }

        if self.show_label_popup {
            if self.selected_issue.is_some() { let _ = self.update_issue_labels().await; }
            else if self.selected_mr.is_some() { let _ = self.update_mr_labels().await; }
            return;
        }
        if self.show_assignee_popup {
            if self.selected_issue.is_some() { let _ = self.update_issue_assignees().await; }
            else if self.selected_mr.is_some() { let _ = self.update_mr_assignees().await; }
            return;
        }
        if self.show_milestone_popup {
            if self.selected_issue.is_some() { let _ = self.update_issue_milestone().await; }
            else if self.selected_mr.is_some() { let _ = self.update_mr_milestone().await; }
            return;
        }
        if self.show_calendar_popup {
            let _ = self.update_issue_due_date().await;
            return;
        }
        if self.show_weight_popup {
            let _ = self.update_issue_weight().await;
            return;
        }

        match self.view_mode {
            ViewMode::List => {
                if self.active_panel == Panel::Sidebar {
                    self.active_panel = Panel::Content;
                } else {
                    match self.active_selection {
                        MenuSelection::MergeRequests => {
                            if let Some(i) = self.mrs_state.selected() {
                                if let Some(GroupedItem::Item(mr)) = self.filtered_mrs.get(i) {
                                    self.selected_mr = Some(mr.clone());
                                    let _ = self.refresh().await;
                                    self.discussion_state.select(Some(0));
                                    self.detail_section = DetailSection::Discussions;
                                    self.view_mode = ViewMode::Detail;
                                }
                            }
                        }
                        MenuSelection::Issues => {
                            if let Some(i) = self.issues_state.selected() {
                                if let Some(GroupedItem::Item(issue)) = self.filtered_issues.get(i) {
                                    self.selected_issue = Some(issue.clone());
                                    let _ = self.refresh().await;
                                    self.discussion_state.select(Some(0));
                                    self.detail_section = DetailSection::Discussions;
                                    self.view_mode = ViewMode::Detail;
                                }
                            }
                        }
                        MenuSelection::Pipelines => {
                            if let Some(i) = self.pipelines_state.selected() {
                                if let Some(pipeline) = self.pipelines.get(i) {
                                    self.selected_pipeline = Some(pipeline.clone());
                                    let _ = self.refresh().await;
                                    self.jobs_state.select(Some(0));
                                    self.view_mode = ViewMode::Detail;
                                }
                            }
                        }
                        MenuSelection::Todos => {
                            if let Some(i) = self.todos_state.selected() {
                                if let Some(GroupedItem::Item(_)) = self.filtered_todos.get(i) {
                                    self.open_in_browser();
                                }
                            }
                        }
                        MenuSelection::Projects => {
                            if let Some(i) = self.projects_state.selected() {
                                if let Some(project) = self.projects.get(i) {
                                    self.selected_project = Some(project.clone());
                                    self.active_selection = MenuSelection::MergeRequests;
                                    self.menu_state.select(Some(2));
                                    let _ = self.refresh().await;
                                    let _ = self.fetch_project_metadata().await;
                                    self.active_panel = Panel::Sidebar;
                                }
                            }
                        }
                        MenuSelection::Groups => {
                            if let Some(i) = self.groups_state.selected() {
                                if self.groups.get(i).is_some() {
                                    self.open_in_browser();
                                }
                            }
                        }
                    }
                }
            }
            ViewMode::JobLog => {}
            ViewMode::Diff => {}
            ViewMode::Detail => {
                if self.selected_pipeline.is_some() {
                    if let Some(i) = self.jobs_state.selected() {
                        if let Some(job) = self.jobs.get(i) {
                            self.selected_job = Some(job.clone());
                            self.follow_log = true;
                            let _ = self.refresh().await;
                            self.view_mode = ViewMode::JobLog;
                        }
                    }
                    return;
                }
                match self.detail_section {
                    DetailSection::Metadata => {
                        match self.selected_metadata_field {
                            MetadataField::Title => {
                                if let Some(issue) = &self.selected_issue {
                                    self.input_mode = true;
                                    self.input_target = InputTarget::EditTitle;
                                    self.reply_input = issue.title.clone();
                                    self.cursor_index = self.reply_input.chars().count();
                                } else if let Some(mr) = &self.selected_mr {
                                    self.input_mode = true;
                                    self.input_target = InputTarget::EditTitle;
                                    self.reply_input = mr.title.clone();
                                    self.cursor_index = self.reply_input.chars().count();
                                }
                            }
                            MetadataField::Description => {
                                if let Some(issue) = &self.selected_issue {
                                    self.input_mode = true;
                                    self.input_target = InputTarget::EditDescription;
                                    self.reply_input = issue.description.clone().unwrap_or_default();
                                    self.cursor_index = self.reply_input.chars().count();
                                } else if let Some(mr) = &self.selected_mr {
                                    self.input_mode = true;
                                    self.input_target = InputTarget::EditDescription;
                                    self.reply_input = mr.description.clone().unwrap_or_default();
                                    self.cursor_index = self.reply_input.chars().count();
                                }
                            }
                            MetadataField::Labels => {
                                if let Some(issue) = &self.selected_issue {
                                    self.selected_label_ids = issue.labels.iter().map(|l| l.id).collect();
                                    self.show_label_popup = true;
                                    self.label_state.select(Some(0));
                                } else if let Some(mr) = &self.selected_mr {
                                    self.selected_label_ids = mr.labels.iter().map(|l| l.id).collect();
                                    self.show_label_popup = true;
                                    self.label_state.select(Some(0));
                                }
                            }
                            MetadataField::Assignees => {
                                if let Some(issue) = &self.selected_issue {
                                    self.selected_assignee_ids = issue.assignees.iter().map(|a| a.id).collect();
                                    self.show_assignee_popup = true;
                                    self.assignee_state.select(Some(0));
                                } else if let Some(mr) = &self.selected_mr {
                                    self.selected_assignee_ids = mr.assignees.iter().map(|a| a.id).collect();
                                    self.show_assignee_popup = true;
                                    self.assignee_state.select(Some(0));
                                }
                            }
                            MetadataField::Milestone => {
                                if let Some(issue) = &self.selected_issue {
                                    self.selected_milestone_id = issue.milestone.as_ref().map(|m| {
                                        self.project_milestones.iter().find(|pm| pm.title == m.title).map(|pm| pm.id).unwrap_or(0)
                                    });
                                    self.show_milestone_popup = true;
                                    self.milestone_state.select(Some(0));
                                } else if let Some(mr) = &self.selected_mr {
                                    self.selected_milestone_id = mr.milestone.as_ref().map(|m| {
                                        self.project_milestones.iter().find(|pm| pm.title == m.title).map(|pm| pm.id).unwrap_or(0)
                                    });
                                    self.show_milestone_popup = true;
                                    self.milestone_state.select(Some(0));
                                }
                            }
                            MetadataField::DueDate => {
                                if let Some(issue) = &self.selected_issue {
                                    self.calendar_date = issue.due_date.as_ref()
                                        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                                        .unwrap_or_else(|| Local::now().date_naive());
                                    self.show_calendar_popup = true;
                                }
                            }
                            MetadataField::Weight => {
                                if let Some(issue) = &self.selected_issue {
                                    let weight = issue.weight.unwrap_or(0);
                                    self.weight_state.select(Some(weight as usize));
                                    self.show_weight_popup = true;
                                }
                            }
                            MetadataField::Confidential => {
                                let _ = self.toggle_confidential().await;
                            }
                        }
                    }
                    DetailSection::Discussions => {
                        self.input_mode = true;
                        self.input_target = InputTarget::Reply;
                        self.cursor_index = 0;
                    }
                }
            }
        }
    }

    pub async fn back(&mut self) {
        if self.show_label_popup { self.show_label_popup = false; return; }
        if self.show_assignee_popup { self.show_assignee_popup = false; return; }
        if self.show_milestone_popup { self.show_milestone_popup = false; return; }
        if self.show_calendar_popup { self.show_calendar_popup = false; return; }
        if self.show_weight_popup { self.show_weight_popup = false; return; }
        if self.show_pipeline_popup { self.show_pipeline_popup = false; self.input_mode = false; return; }

        if self.input_mode {
            self.input_mode = false;
            self.reply_input.clear();
            if self.show_filter_input {
                self.show_filter_input = false;
                self.filter_input.clear();
            }
        } else if self.show_project_popup {
            if self.selected_project.is_some() {
                self.show_project_popup = false;
                self.active_panel = Panel::Sidebar;
                self.refresh().await;
            }
        } else if self.view_mode == ViewMode::JobLog {
            self.view_mode = ViewMode::Detail;
            self.selected_job = None;
            self.job_log.clear();
            self.refresh().await;
        } else if self.view_mode == ViewMode::Diff {
            self.view_mode = ViewMode::Detail;
            self.mr_diffs.clear();
            self.refresh().await;
        } else if self.view_mode == ViewMode::Detail {
            self.view_mode = ViewMode::List;
            self.selected_mr = None;
            self.selected_issue = None;
            self.selected_pipeline = None;
            self.discussions.clear();
            self.jobs.clear();
            self.refresh().await;
        } else if self.active_panel != Panel::Sidebar {
            self.active_panel = Panel::Sidebar;
        }
    }

    pub fn open_in_browser(&self) {
        let url: Option<String> = if let Some(mr) = &self.selected_mr { Some(mr.web_url.clone()) }
        else if let Some(issue) = &self.selected_issue { Some(issue.web_url.clone()) }
        else if let Some(pipeline) = &self.selected_pipeline {
            self.jobs_state.selected().and_then(|i| self.jobs.get(i)).map(|job| job.web_url.clone()).or(Some(pipeline.web_url.clone()))
        } else if self.view_mode == ViewMode::List {
            match self.active_selection {
                MenuSelection::MergeRequests => self.mrs_state.selected().and_then(|i| {
                    match self.filtered_mrs.get(i) {
                        Some(GroupedItem::Item(mr)) => Some(mr.web_url.clone()),
                        _ => None,
                    }
                }),
                MenuSelection::Issues => self.issues_state.selected().and_then(|i| {
                    match self.filtered_issues.get(i) {
                        Some(GroupedItem::Item(issue)) => Some(issue.web_url.clone()),
                        _ => None,
                    }
                }),
                MenuSelection::Pipelines => self.pipelines_state.selected().and_then(|i| self.pipelines.get(i)).map(|p| p.web_url.clone()),
                MenuSelection::Todos => self.todos_state.selected().and_then(|i| {
                    match self.filtered_todos.get(i) {
                        Some(GroupedItem::Item(todo)) => Some(todo.target_url.clone()),
                        _ => None,
                    }
                }),
                MenuSelection::Projects => self.projects_state.selected().and_then(|i| self.projects.get(i)).and_then(|p| p.web_url.clone()),
                MenuSelection::Groups => self.groups_state.selected().and_then(|i| self.groups.get(i)).map(|g| format!("https://gitlab.com/groups/{}", g.full_path)),
            }
        } else { None };

        if let Some(url) = url {
            if url.starts_with("http") {
                let _ = std::process::Command::new("powershell")
                    .arg("-NoProfile")
                    .arg("-Command")
                    .arg(format!("Start-Process '{}'", url))
                    .spawn();
            }
        }
    }

    pub async fn check_pagination(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.view_mode != ViewMode::List || self.active_panel == Panel::Sidebar { return Ok(()); }
        
        match self.active_selection {
            MenuSelection::MergeRequests => {
                if let Some(i) = self.mrs_state.selected() {
                    if i >= self.filtered_mrs.len().saturating_sub(5) && self.mrs_has_more {
                        let _ = self.fetch_next_merge_requests().await;
                    }
                }
            }
            MenuSelection::Issues => {
                if let Some(i) = self.issues_state.selected() {
                    if i >= self.filtered_issues.len().saturating_sub(5) && self.issues_has_more {
                        let _ = self.fetch_next_issues().await;
                    }
                }
            }
            MenuSelection::Pipelines => {
                if let Some(i) = self.pipelines_state.selected() {
                    if i >= self.pipelines.len().saturating_sub(5) && self.pipelines_has_more {
                        let _ = self.fetch_next_pipelines().await;
                    }
                }
            }
            MenuSelection::Todos => {
                if let Some(i) = self.todos_state.selected() {
                    if i >= self.filtered_todos.len().saturating_sub(5) && self.todos_has_more {
                        let _ = self.fetch_next_todos().await;
                    }
                }
            }
            MenuSelection::Projects => {
                if let Some(i) = self.projects_state.selected() {
                    if i >= self.projects.len().saturating_sub(5) && self.projects_has_more {
                        let _ = self.fetch_next_projects().await;
                    }
                }
            }
            MenuSelection::Groups => {
                if let Some(i) = self.groups_state.selected() {
                    if i >= self.groups.len().saturating_sub(5) && self.groups_has_more {
                        let _ = self.fetch_next_groups().await;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::MouseEventKind;
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                if self.view_mode == ViewMode::JobLog { self.page_down(); }
                else { self.move_down(); let _ = self.check_pagination().await; }
            }
            MouseEventKind::ScrollUp => {
                if self.view_mode == ViewMode::JobLog { self.page_up(); }
                else { self.move_up(); }
            }
            _ => {}
        }
    }

    pub fn filter_lists(&mut self) {
        let filter = self.filter_input.to_lowercase();
        let selected_pid = self.selected_project.as_ref().map(|p| p.id);
        let is_global = selected_pid.is_none();
        
        // Todos
        let mut raw_todos: Vec<Todo> = self.todos.iter()
            .filter(|t| {
                let matches_filter = t.body.to_lowercase().contains(&filter) || t.project.path_with_namespace.as_deref().unwrap_or("").to_lowercase().contains(&filter) || t.action_name.to_lowercase().contains(&filter);
                let matches_project = is_global || Some(t.project.id) == selected_pid;
                matches_filter && matches_project
            })
            .cloned().collect();
        raw_todos.sort_by(|a, b| a.project.path_with_namespace.as_deref().unwrap_or("").cmp(b.project.path_with_namespace.as_deref().unwrap_or("")));
        
        self.filtered_todos = Vec::new();
        let mut last_todo_proj = String::new();
        for t in raw_todos {
            let path = t.project.path_with_namespace.as_deref().unwrap_or("");
            if is_global && path != last_todo_proj {
                last_todo_proj = path.to_string();
                self.filtered_todos.push(GroupedItem::Header(last_todo_proj.clone()));
            }
            self.filtered_todos.push(GroupedItem::Item(t));
        }

        self.filtered_projects = self.projects.iter()
            .filter(|p| p.name.to_lowercase().contains(&filter) || p.path_with_namespace.as_deref().unwrap_or("").to_lowercase().contains(&filter))
            .cloned().collect();

        self.filtered_groups = self.groups.iter()
            .filter(|g| g.name.to_lowercase().contains(&filter) || g.full_path.to_lowercase().contains(&filter))
            .cloned().collect();

        // MRs
        let mut raw_mrs: Vec<MergeRequest> = self.merge_requests.iter()
            .filter(|mr| {
                let matches_filter = mr.title.to_lowercase().contains(&filter) || mr.author.username.to_lowercase().contains(&filter) || mr.references.full.to_lowercase().contains(&filter);
                let matches_project = is_global || Some(mr.project_id) == selected_pid;
                matches_filter && matches_project
            })
            .cloned().collect();
        raw_mrs.sort_by(|a, b| a.references.full.cmp(&b.references.full));
        
        self.filtered_mrs = Vec::new();
        let mut last_mr_proj = String::new();
        for mr in raw_mrs {
            let proj = mr.references.full.split('!').next().unwrap_or("").to_string();
            if is_global && proj != last_mr_proj {
                last_mr_proj = proj;
                self.filtered_mrs.push(GroupedItem::Header(last_mr_proj.clone()));
            }
            self.filtered_mrs.push(GroupedItem::Item(mr));
        }
            
        // Issues
        let mut raw_issues: Vec<Issue> = self.issues.iter()
            .filter(|i| {
                let matches_filter = i.title.to_lowercase().contains(&filter) || i.author.username.to_lowercase().contains(&filter) || i.references.full.to_lowercase().contains(&filter);
                let matches_project = is_global || Some(i.project_id) == selected_pid;
                matches_filter && matches_project
            })
            .cloned().collect();
        raw_issues.sort_by(|a, b| a.references.full.cmp(&b.references.full));
        
        self.filtered_issues = Vec::new();
        let mut last_issue_proj = String::new();
        for i in raw_issues {
            let proj = i.references.full.split('#').next().unwrap_or("").to_string();
            if is_global && proj != last_issue_proj {
                last_issue_proj = proj;
                self.filtered_issues.push(GroupedItem::Header(last_issue_proj.clone()));
            }
            self.filtered_issues.push(GroupedItem::Item(i));
        }
            
        self.filtered_pipelines = self.pipelines.iter()
            .filter(|p| {
                let matches_filter = p.ref_name.to_lowercase().contains(&filter) || p.status.to_lowercase().contains(&filter) || format!("#{}", p.id).contains(&filter);
                let matches_project = is_global || Some(p.project_id) == selected_pid;
                matches_filter && matches_project
            })
            .cloned().collect();

        // Reset selections
        if self.filtered_todos.is_empty() { self.todos_state.select(None); }
        else {
            let mut i = self.todos_state.selected().unwrap_or(0);
            if i >= self.filtered_todos.len() { i = 0; }
            while i < self.filtered_todos.len() && matches!(self.filtered_todos[i], GroupedItem::Header(_)) { i += 1; }
            if i >= self.filtered_todos.len() { self.todos_state.select(None); } else { self.todos_state.select(Some(i)); }
        }

        if self.filtered_mrs.is_empty() { self.mrs_state.select(None); }
        else {
            let mut i = self.mrs_state.selected().unwrap_or(0);
            if i >= self.filtered_mrs.len() { i = 0; }
            while i < self.filtered_mrs.len() && matches!(self.filtered_mrs[i], GroupedItem::Header(_)) { i += 1; }
            if i >= self.filtered_mrs.len() { self.mrs_state.select(None); } else { self.mrs_state.select(Some(i)); }
        }

        if self.filtered_issues.is_empty() { self.issues_state.select(None); }
        else {
            let mut i = self.issues_state.selected().unwrap_or(0);
            if i >= self.filtered_issues.len() { i = 0; }
            while i < self.filtered_issues.len() && matches!(self.filtered_issues[i], GroupedItem::Header(_)) { i += 1; }
            if i >= self.filtered_issues.len() { self.issues_state.select(None); } else { self.issues_state.select(Some(i)); }
        }

        if self.filtered_pipelines.is_empty() { self.pipelines_state.select(None); }
        else if self.pipelines_state.selected().map_or(true, |i| i >= self.filtered_pipelines.len()) { self.pipelines_state.select(Some(0)); }
    }

    pub async fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        if self.show_help_popup {
            if self.show_help_filter {
                match key.code {
                    KeyCode::Enter => {
                        self.show_help_filter = false;
                        if let Some(i) = self.help_state.selected() { if let Some(item) = self.filtered_help.get(i) { let action = item.action; self.execute_help_action(action).await; } }
                    }
                    KeyCode::Esc => { self.show_help_filter = false; self.help_filter_input.clear(); self.filter_help(); }
                    KeyCode::Char(c) => { self.help_filter_input.push(c); self.filter_help(); }
                    KeyCode::Backspace => { self.help_filter_input.pop(); self.filter_help(); }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('?') | KeyCode::Esc | KeyCode::Backspace => self.show_help_popup = false,
                    KeyCode::Char('/') => self.show_help_filter = true,
                    KeyCode::Down | KeyCode::Char('j') => { let i = match self.help_state.selected() { Some(i) => if i >= self.filtered_help.len().saturating_sub(1) { 0 } else { i + 1 }, None => 0 }; self.help_state.select(Some(i)); }
                    KeyCode::Up | KeyCode::Char('k') => { let i = match self.help_state.selected() { Some(i) => if i == 0 { self.filtered_help.len().saturating_sub(1) } else { i - 1 }, None => 0 }; self.help_state.select(Some(i)); }
                    KeyCode::Enter => { if let Some(i) = self.help_state.selected() { if let Some(item) = self.filtered_help.get(i) { let action = item.action; self.execute_help_action(action).await; } } }
                    _ => {}
                }
            }
            return;
        }

        if self.show_project_popup {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => self.move_down(),
                KeyCode::Up | KeyCode::Char('k') => self.move_up(),
                KeyCode::Enter => self.select_project().await,
                KeyCode::Esc => self.show_project_popup = false,
                _ => {}
            }
        } else if self.show_calendar_popup {
            use chrono::Days;
            match key.code {
                KeyCode::Left => { self.calendar_date = self.calendar_date.checked_sub_days(Days::new(1)).unwrap(); }
                KeyCode::Right => { self.calendar_date = self.calendar_date.checked_add_days(Days::new(1)).unwrap(); }
                KeyCode::Up => { self.calendar_date = self.calendar_date.checked_sub_days(Days::new(7)).unwrap(); }
                KeyCode::Down => { self.calendar_date = self.calendar_date.checked_add_days(Days::new(7)).unwrap(); }
                KeyCode::Enter => self.enter().await,
                KeyCode::Delete | KeyCode::Backspace => { let _ = self.clear_due_date().await; }
                KeyCode::Esc => self.back().await,
                _ => {}
            }
        } else if self.input_mode {
            match key.code {
                KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    if self.input_target == InputTarget::EditDescription || self.input_target == InputTarget::Reply || self.input_target == InputTarget::NewComment {
                        let mut chars: Vec<char> = self.reply_input.chars().collect();
                        chars.insert(self.cursor_index, '\n');
                        self.reply_input = chars.into_iter().collect();
                        self.cursor_index += 1;
                    }
                }
                KeyCode::Enter => self.enter().await,
                KeyCode::Esc => self.back().await,
                KeyCode::Left => { self.cursor_index = self.cursor_index.saturating_sub(1); }
                KeyCode::Right => { self.cursor_index = (self.cursor_index + 1).min(self.get_input_buffer().chars().count()); }
                KeyCode::Home => { self.cursor_index = 0; }
                KeyCode::End => { self.cursor_index = self.get_input_buffer().chars().count(); }
                KeyCode::Char(c) => {
                    let cursor_idx = self.cursor_index;
                    let input = self.get_input_buffer_mut();
                    let mut chars: Vec<char> = input.chars().collect();
                    let cursor = cursor_idx.min(chars.len());
                    chars.insert(cursor, c);
                    *input = chars.into_iter().collect();
                    self.cursor_index = cursor + 1;
                    if self.input_target == InputTarget::PipelineRef { self.filter_branches(); self.show_branches_dropdown = true; }
                    if self.input_target == InputTarget::Filter { self.filter_lists(); }
                }
                KeyCode::Backspace if self.cursor_index > 0 => {
                    let cursor_idx = self.cursor_index;
                    let input = self.get_input_buffer_mut();
                    let mut chars: Vec<char> = input.chars().collect();
                    let cursor = cursor_idx.min(chars.len());
                    if cursor > 0 {
                        chars.remove(cursor - 1);
                        *input = chars.into_iter().collect();
                        self.cursor_index = cursor - 1;
                    }
                    if self.input_target == InputTarget::PipelineRef { self.filter_branches(); self.show_branches_dropdown = true; }
                    if self.input_target == InputTarget::Filter { self.filter_lists(); }
                }
                KeyCode::Delete => {
                    let cursor_idx = self.cursor_index;
                    let input = self.get_input_buffer_mut();
                    let mut chars: Vec<char> = input.chars().collect();
                    let cursor = cursor_idx.min(chars.len());
                    if cursor < chars.len() { chars.remove(cursor); *input = chars.into_iter().collect(); }
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char('?') => self.toggle_help(),
                KeyCode::Char('d') if self.view_mode == ViewMode::List || self.view_mode == ViewMode::Detail => {
                    if self.selected_mr.is_some() || (self.active_selection == MenuSelection::MergeRequests && self.mrs_state.selected().is_some()) {
                        self.view_mode = ViewMode::Diff;
                        let _ = self.refresh().await;
                    }
                }
                KeyCode::Char('a') if self.view_mode == ViewMode::Detail && self.selected_mr.is_some() => { let _ = self.approve_mr().await; }
                KeyCode::Char('m') if self.view_mode == ViewMode::Detail && self.selected_mr.is_some() => { let _ = self.merge_mr().await; }
                KeyCode::Char('/') if self.view_mode == ViewMode::List => {
                    self.input_mode = true;
                    self.input_target = InputTarget::Filter;
                    self.show_filter_input = true;
                    self.cursor_index = self.filter_input.chars().count();
                }
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => self.switch_project(),
                KeyCode::Down | KeyCode::Char('j') => { self.move_down(); let _ = self.check_pagination().await; }
                KeyCode::Up | KeyCode::Char('k') => self.move_up(),
                KeyCode::PageDown => self.page_down(),
                KeyCode::PageUp => self.page_up(),
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab if self.view_mode != ViewMode::JobLog => self.toggle_focus(),
                KeyCode::Tab if self.view_mode == ViewMode::JobLog => self.toggle_job_section(),
                KeyCode::Char('s') if self.view_mode == ViewMode::Detail && self.selected_mr.is_some() => { let _ = self.resolve_discussion().await; }
                KeyCode::Char('c') if self.view_mode == ViewMode::Detail && self.selected_mr.is_some() => { let _ = self.toggle_mr_state().await; }
                KeyCode::Char('c') if self.view_mode == ViewMode::Detail && self.selected_issue.is_some() => { let _ = self.toggle_issue_state().await; }
                KeyCode::Char('n') if self.view_mode == ViewMode::Detail && (self.selected_issue.is_some() || self.selected_mr.is_some()) => {
                    self.input_mode = true;
                    self.input_target = InputTarget::NewComment;
                    self.reply_input.clear();
                    self.cursor_index = 0;
                }
                KeyCode::Char('C') if self.view_mode == ViewMode::List => {
                    match self.active_selection {
                        MenuSelection::Issues => { self.show_create_issue_popup = true; self.input_mode = true; self.input_target = InputTarget::CreateIssueTitle; self.create_issue_title.clear(); self.create_issue_description.clear(); self.cursor_index = 0; }
                        MenuSelection::MergeRequests => { self.show_create_mr_popup = true; self.input_mode = true; self.input_target = InputTarget::CreateMRTitle; self.create_mr_title.clear(); self.create_mr_description.clear(); self.create_mr_source_branch.clear(); self.create_mr_target_branch.clear(); self.cursor_index = 0; }
                        _ => {}
                    }
                }
                KeyCode::Char('p') if self.view_mode == ViewMode::List && self.active_selection == MenuSelection::Pipelines => { self.show_pipeline_launch_popup().await; }
                KeyCode::Char('p') if self.view_mode == ViewMode::Detail && self.selected_pipeline.is_some() => {
                    if let Some(proj) = &self.selected_project { if let Some(i) = self.jobs_state.selected() { if let Some(job) = self.jobs.get(i) { if job.status == "manual" { let _ = self.play_job(proj.id, job.id).await; } } } }
                }
                KeyCode::Char('r') if self.view_mode == ViewMode::Detail && self.selected_pipeline.is_some() => {
                    if let Some(proj) = &self.selected_project { if let Some(i) = self.jobs_state.selected() { if let Some(job) = self.jobs.get(i) { let _ = self.retry_job(proj.id, job.id).await; } } else if let Some(pipeline) = &self.selected_pipeline { let _ = self.retry_pipeline(proj.id, pipeline.id).await; } }
                }
                KeyCode::Char('c') if self.view_mode == ViewMode::List && self.active_selection == MenuSelection::Pipelines => {
                    if let Some(proj) = &self.selected_project { if let Some(i) = self.pipelines_state.selected() { if let Some(pipeline) = self.pipelines.get(i) { let _ = self.cancel_pipeline(proj.id, pipeline.id).await; } } }
                }
                KeyCode::Char('c') if self.view_mode == ViewMode::Detail && self.selected_pipeline.is_some() => {
                    if let Some(proj) = &self.selected_project { if let Some(i) = self.jobs_state.selected() { if let Some(job) = self.jobs.get(i) { let _ = self.cancel_job(proj.id, job.id).await; } } else if let Some(pipeline) = &self.selected_pipeline { let _ = self.cancel_pipeline(proj.id, pipeline.id).await; } }
                }
                KeyCode::Char(' ') => self.toggle_discussion(),
                KeyCode::Enter => self.enter().await,
                KeyCode::Backspace | KeyCode::Esc => self.back().await,
                KeyCode::Char('v') | KeyCode::Char('o') => self.open_in_browser(),
                _ => {}
            }
        }
    }

    pub async fn create_issue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let endpoint = CreateIssue {
                project: project.id,
                title: self.create_issue_title.clone(),
                description: Some(self.create_issue_description.clone()),
                assignee_ids: Vec::new(),
                label_names: Vec::new(),
            };
            let _: serde_json::Value = endpoint.query_async(&self.gitlab).await?;
            self.show_create_issue_popup = false;
            self.input_mode = false;
            let _ = self.refresh().await;
        }
        Ok(())
    }

    pub async fn create_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let endpoint = CreateMergeRequest {
                project: project.id,
                source_branch: self.create_mr_source_branch.clone(),
                target_branch: self.create_mr_target_branch.clone(),
                title: self.create_mr_title.clone(),
                description: Some(self.create_mr_description.clone()),
            };
            let _: serde_json::Value = endpoint.query_async(&self.gitlab).await?;
            self.show_create_mr_popup = false;
            self.input_mode = false;
            let _ = self.refresh().await;
        }
        Ok(())
    }

    pub fn get_input_buffer(&self) -> &String {
        match self.input_target {
            InputTarget::PipelineRef | InputTarget::PipelineRefSearch => &self.pipeline_ref_input,
            InputTarget::PipelineVariableKey => &self.pipeline_variable_key_input,
            InputTarget::PipelineVariableValue => &self.pipeline_variable_value_input,
            InputTarget::Filter => &self.filter_input,
            InputTarget::CreateIssueTitle => &self.create_issue_title,
            InputTarget::CreateIssueDescription => &self.create_issue_description,
            InputTarget::CreateMRTitle => &self.create_mr_title,
            InputTarget::CreateMRDescription => &self.create_mr_description,
            InputTarget::CreateMRSourceBranch => &self.create_mr_source_branch,
            InputTarget::CreateMRTargetBranch => &self.create_mr_target_branch,
            _ => &self.reply_input,
        }
    }

    pub fn get_input_buffer_mut(&mut self) -> &mut String {
        match self.input_target {
            InputTarget::PipelineRef | InputTarget::PipelineRefSearch => &mut self.pipeline_ref_input,
            InputTarget::PipelineVariableKey => &mut self.pipeline_variable_key_input,
            InputTarget::PipelineVariableValue => &mut self.pipeline_variable_value_input,
            InputTarget::Filter => &mut self.filter_input,
            InputTarget::CreateIssueTitle => &mut self.create_issue_title,
            InputTarget::CreateIssueDescription => &mut self.create_issue_description,
            InputTarget::CreateMRTitle => &mut self.create_mr_title,
            InputTarget::CreateMRDescription => &mut self.create_mr_description,
            InputTarget::CreateMRSourceBranch => &mut self.create_mr_source_branch,
            InputTarget::CreateMRTargetBranch => &mut self.create_mr_target_branch,
            _ => &mut self.reply_input,
        }
    }

    pub fn parse_job_log(&mut self) {
        let mut lines = Vec::new();
        for line in self.job_log.lines() {
            lines.push(LogLine {
                timestamp: "".into(),
                content: line.to_string(),
                section: None,
                is_section_start: false,
                is_section_end: false,
            });
        }
        self.job_log_lines = lines;
    }
}

pub fn parse_reactions(reaction_values: Vec<serde_json::Value>) -> Vec<AwardEmoji> {
    let mut reactions = Vec::new();
    for val in reaction_values {
        let name = val.get("name").and_then(|v| v.as_str())
            .or_else(|| val.get("emoji").and_then(|v| v.as_str()))
            .or_else(|| val.get("emoji_name").and_then(|v| v.as_str()));
        if let Some(n) = name { reactions.push(AwardEmoji { name: n.to_string() }); }
    }
    reactions
}
