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
            detail_section: DetailSection::Metadata,
            selected_metadata_field: MetadataField::Title,
            selected_mr: None,
            selected_issue: None,
            selected_pipeline: None,
            selected_job: None,
            mr_diffs: Vec::new(),
            mr_diff_state: ListState::default(),
            diff_line_state: ListState::default(),
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
            textarea: ratatui_textarea::TextArea::default(),
            theme: Theme::load(),
            last_refresh: std::time::Instant::now(),
            tx,
            project_labels: Vec::new(),
            project_label_cache: std::collections::HashMap::new(),
            project_milestone_cache: std::collections::HashMap::new(),
            project_member_cache: std::collections::HashMap::new(),
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
        }
    }

    pub async fn process_updates(&mut self, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
        while let Ok(update) = rx.try_recv() {
            match update {
                DataUpdate::Todos { data, page, has_more } => { self.todos = data; self.todos_page = page; self.todos_has_more = has_more; self.filter_lists(); }
                DataUpdate::AppendTodos(mut data) => { self.todos_has_more = data.len() >= 50; self.todos.append(&mut data); self.todos_page += 1; self.filter_lists(); }

                DataUpdate::Projects { data, page, has_more } => { 
                    for p in &data { self.trigger_project_metadata_fetch(p.id); }
                    self.projects = data; self.projects_page = page; self.projects_has_more = has_more; self.filter_lists(); 
                }
                DataUpdate::AppendProjects(mut data) => { 
                    for p in &data { self.trigger_project_metadata_fetch(p.id); }
                    self.projects_has_more = data.len() >= 50; self.projects.append(&mut data); self.projects_page += 1; self.filter_lists(); 
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
                        self.mrs_has_more = data.len() >= 50; self.merge_requests.append(&mut data); self.mrs_page += 1; self.filter_lists(); 
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
                        self.issues_has_more = data.len() >= 50; self.issues.append(&mut data); self.issues_page += 1; self.filter_lists(); 
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
                DataUpdate::Log { project_id, job_id, data } => { if let Some(selected) = &self.selected_job { if selected.id == job_id && self.selected_project.as_ref().map_or(false, |p| p.id == project_id) { self.job_log = data; self.parse_job_log(); } } }
                DataUpdate::Diffs { project_id, iid, data } => { 
                    if let Some(selected) = &self.selected_mr { 
                        if selected.iid == iid && selected.project_id == project_id { 
                            self.mr_diffs = data; 
                            self.enrich_diffs_with_discussions();
                        } 
                    } 
                }
                DataUpdate::Metadata { project_id, labels, milestones, members } => { 
                    self.project_label_cache.insert(project_id, labels.clone());
                    self.project_milestone_cache.insert(project_id, milestones.clone());
                    self.project_member_cache.insert(project_id, members.clone());

                    if self.selected_project.as_ref().map_or(false, |p| p.id == project_id) {
                        self.project_labels = labels.clone(); 
                        self.project_milestones = milestones.clone(); 
                        self.project_members = members.clone(); 
                    }
                    
                    let labels_ref = &labels;
                    // Re-enrich existing lists using the NEW labels for THIS project
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
                            // Preserve colors from existing selected_mr if the new one is grey
                            for label in &mut mr.labels {
                                if label.id == 0 || label.color == "#808080" {
                                    if let Some(old) = selected.labels.iter().find(|l| l.name == label.name) {
                                        if old.color != "#808080" {
                                            label.id = old.id;
                                            label.color = old.color.clone();
                                            label.text_color = old.text_color.clone();
                                        }
                                    }
                                }
                            }
                            if !self.input_mode { *selected = mr; } 
                        } 
                    } else {
                        self.selected_mr = Some(mr);
                    }
                }
                DataUpdate::IssueDetail(mut issue) => { 
                    self.fix_issue_labels(&mut issue);
                    if let Some(selected) = &mut self.selected_issue { 
                        if selected.iid == issue.iid && selected.project_id == issue.project_id { 
                            // Preserve colors from existing selected_issue if the new one is grey
                            for label in &mut issue.labels {
                                if label.id == 0 || label.color == "#808080" {
                                    if let Some(old) = selected.labels.iter().find(|l| l.name == label.name) {
                                        if old.color != "#808080" {
                                            label.id = old.id;
                                            label.color = old.color.clone();
                                            label.text_color = old.text_color.clone();
                                        }
                                    }
                                }
                            }
                            if !self.input_mode { *selected = issue; } 
                        } 
                    } else {
                        self.selected_issue = Some(issue);
                    }
                }
                DataUpdate::Loading(b) => { if b { self.loading_tasks += 1; } else { self.loading_tasks = self.loading_tasks.saturating_sub(1); } self.is_loading = self.loading_tasks > 0; }
                DataUpdate::Error(e) => { self.last_error = Some(e); self.loading_tasks = self.loading_tasks.saturating_sub(1); self.is_loading = self.loading_tasks > 0; }
                DataUpdate::TriggerRefresh => { self.refresh().await; }
                DataUpdate::CheckPagination => { let _ = self.check_pagination().await; }
                DataUpdate::CurrentUser(user) => { self.current_user = Some(user); }
            }
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
            let ep = PagedProjects { membership: true, page: 1, per_page: 100, simple: Some(true), archived: Some(false), order_by: Some("last_activity_at".into()) }; 
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
                                        self.selected_mr = Some(MergeRequest { iid, project_id: todo.project.id, title: todo.body.clone(), author: Author { id: 0, username: "".into() }, web_url: todo.target_url.clone(), state: "opened".into(), reactions: Vec::new(), description: None, references: None, upvotes: None, downvotes: None, labels: Vec::new(), milestone: None, assignees: Vec::new(), created_at: Some("".into()), draft: Some(false), diff_refs: None });
                                        let _ = self.fetch_discussions(todo.project.id, iid).await;
                                    }
                                } else if todo.target_type == "Issue" {
                                    if self.selected_issue.as_ref().map_or(true, |iss| iss.iid != iid || iss.project_id != todo.project.id) {
                                        self.selected_mr = None; self.discussions.clear();
                                        self.selected_issue = Some(Issue { iid, project_id: todo.project.id, title: todo.body.clone(), description: None, author: Author { id: 0, username: "".into() }, assignees: Vec::new(), state: "opened".into(), web_url: todo.target_url.clone(), references: None, issue_type: None, labels: Vec::new(), milestone: None, due_date: None, weight: None, task_completion_status: None, confidential: false, created_at: Some("".into()), upvotes: None, downvotes: None, reactions: Vec::new() });
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
                _ => {}
            }
        } else {
            self.selected_mr = None; self.selected_issue = None; self.selected_pipeline = None;
            self.discussions.clear(); self.jobs.clear();
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
                if current >= 1 { self.active_panel = Panel::Project; self.project_menu_state.select(Some(0)); }
                else { self.explore_menu_state.select(Some(current + 1)); }
            }
            Panel::Project => {
                let current = self.project_menu_state.selected().unwrap_or(0);
                if current >= 2 { self.active_panel = Panel::Dashboard; self.dashboard_menu_state.select(Some(0)); }
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
                }
                let _ = self.tx.send(DataUpdate::CheckPagination);
            }
            Panel::MainDetail => {
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
                if current == 0 { self.active_panel = Panel::Project; self.project_menu_state.select(Some(2)); }
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
                }
            }
            Panel::MainDetail => {
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

    pub async fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        use chrono::Timelike;

        // On Windows, characters produced via AltGr are often reported with Control + Alt modifiers.
        // We strip these modifiers for character input to ensure they are correctly processed by text areas and filters.
        let mut processed_key = key;
        if let KeyCode::Char(_) = processed_key.code {
            if processed_key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::ALT) {
                processed_key.modifiers.remove(KeyModifiers::CONTROL | KeyModifiers::ALT);
            }
        }

        if self.show_snooze_popup {
            match processed_key.code {
                KeyCode::Down | KeyCode::Char('j') => { let i = (self.snooze_state.selected().unwrap_or(0) + 1) % 5; self.snooze_state.select(Some(i)); }
                KeyCode::Up | KeyCode::Char('k') => { let i = if self.snooze_state.selected().unwrap_or(0) == 0 { 4 } else { self.snooze_state.selected().unwrap() - 1 }; self.snooze_state.select(Some(i)); }
                KeyCode::Enter => {
                    let selected = self.snooze_state.selected().unwrap_or(0);
                    let snooze_until = match selected {
                        0 => (Local::now() + chrono::Duration::hours(1)).to_rfc3339(),
                        1 => (Local::now() + chrono::Duration::days(1)).with_hour(9).unwrap().with_minute(0).unwrap().with_second(0).unwrap().to_rfc3339(),
                        2 => (Local::now() + chrono::Duration::days(3)).with_hour(9).unwrap().with_minute(0).unwrap().with_second(0).unwrap().to_rfc3339(),
                        3 => (Local::now() + chrono::Duration::weeks(1)).with_hour(9).unwrap().with_minute(0).unwrap().with_second(0).unwrap().to_rfc3339(),
                        4 => { self.show_snooze_popup = false; self.show_calendar_popup = true; self.input_target = InputTarget::SnoozeUntil; return; }
                        _ => (Local::now() + chrono::Duration::hours(1)).to_rfc3339(),
                    };
                    let _ = self.snooze_todo(snooze_until).await;
                }
                KeyCode::Esc => self.show_snooze_popup = false,
                _ => {}
            }
            return;
        }
        if self.show_project_popup {
            match processed_key.code {
                KeyCode::Down | KeyCode::Char('j') => self.move_down(),
                KeyCode::Up | KeyCode::Char('k') => self.move_up(),
                KeyCode::Enter => self.select_project().await,
                KeyCode::Esc => self.show_project_popup = false,
                _ => {}
            }
            return;
        }
        if self.show_label_popup {
            let project_id = self.selected_mr.as_ref().map(|m| m.project_id).or_else(|| self.selected_issue.as_ref().map(|i| i.project_id)).unwrap_or(0);
            let labels = self.project_label_cache.get(&project_id).unwrap_or(&self.project_labels);
            if !labels.is_empty() {
                match processed_key.code {
                    KeyCode::Down | KeyCode::Char('j') => { let i = (self.label_state.selected().unwrap_or(0) + 1) % labels.len(); self.label_state.select(Some(i)); }
                    KeyCode::Up | KeyCode::Char('k') => { let i = if self.label_state.selected().unwrap_or(0) == 0 { labels.len() - 1 } else { self.label_state.selected().unwrap() - 1 }; self.label_state.select(Some(i)); }
                    KeyCode::Char(' ') => { if let Some(i) = self.label_state.selected() { if let Some(label) = labels.get(i) { let id = label.id(); if self.selected_label_ids.contains(&id) { self.selected_label_ids.retain(|&x| x != id); } else { self.selected_label_ids.push(id); } } } }
                    _ => {}
                }
            }
            match processed_key.code {
                KeyCode::Enter => { if self.selected_issue.is_some() { let _ = self.update_issue_labels().await; } else if self.selected_mr.is_some() { let _ = self.update_mr_labels().await; } }
                KeyCode::Esc => self.show_label_popup = false,
                _ => {}
            }
            return;
        }
        if self.show_assignee_popup {
            let project_id = self.selected_mr.as_ref().map(|m| m.project_id).or_else(|| self.selected_issue.as_ref().map(|i| i.project_id)).unwrap_or(0);
            let members = self.project_member_cache.get(&project_id).unwrap_or(&self.project_members);
            if !members.is_empty() {
                match processed_key.code {
                    KeyCode::Down | KeyCode::Char('j') => { let i = (self.assignee_state.selected().unwrap_or(0) + 1) % members.len(); self.assignee_state.select(Some(i)); }
                    KeyCode::Up | KeyCode::Char('k') => { let i = if self.assignee_state.selected().unwrap_or(0) == 0 { members.len() - 1 } else { self.assignee_state.selected().unwrap() - 1 }; self.assignee_state.select(Some(i)); }
                    KeyCode::Char(' ') => { if let Some(i) = self.assignee_state.selected() { if let Some(member) = members.get(i) { let id = member.id; if self.selected_assignee_ids.contains(&id) { self.selected_assignee_ids.retain(|&x| x != id); } else { self.selected_assignee_ids.push(id); } } } }
                    _ => {}
                }
            }
            match processed_key.code {
                KeyCode::Enter => { if self.selected_issue.is_some() { let _ = self.update_issue_assignees().await; } else if self.selected_mr.is_some() { let _ = self.update_mr_assignees().await; } }
                KeyCode::Esc => self.show_assignee_popup = false,
                _ => {}
            }
            return;
        }
        if self.show_milestone_popup {
            let project_id = self.selected_mr.as_ref().map(|m| m.project_id).or_else(|| self.selected_issue.as_ref().map(|i| i.project_id)).unwrap_or(0);
            let milestones = self.project_milestone_cache.get(&project_id).unwrap_or(&self.project_milestones);
            match processed_key.code {
                KeyCode::Down | KeyCode::Char('j') => { let i = (self.milestone_state.selected().unwrap_or(0) + 1) % (milestones.len() + 1); self.milestone_state.select(Some(i)); }
                KeyCode::Up | KeyCode::Char('k') => { let i = if self.milestone_state.selected().unwrap_or(0) == 0 { milestones.len() } else { self.milestone_state.selected().unwrap() - 1 }; self.milestone_state.select(Some(i)); }
                KeyCode::Enter => { let selected = self.milestone_state.selected().unwrap_or(0); self.selected_milestone_id = if selected == 0 { None } else { milestones.get(selected - 1).map(|m| m.id) }; if self.selected_issue.is_some() { let _ = self.update_issue_milestone().await; } else if self.selected_mr.is_some() { let _ = self.update_mr_milestone().await; } }
                KeyCode::Esc => self.show_milestone_popup = false,
                _ => {}
            }
            return;
        }
        if self.show_weight_popup {
            match processed_key.code {
                KeyCode::Down | KeyCode::Char('j') => { let i = (self.weight_state.selected().unwrap_or(0) + 1) % 21; self.weight_state.select(Some(i)); }
                KeyCode::Up | KeyCode::Char('k') => { let i = if self.weight_state.selected().unwrap_or(0) == 0 { 20 } else { self.weight_state.selected().unwrap() - 1 }; self.weight_state.select(Some(i)); }
                KeyCode::Enter => { let _ = self.update_issue_weight().await; }
                KeyCode::Esc => self.show_weight_popup = false,
                _ => {}
            }
            return;
        }
        if self.input_mode {
            if self.input_target == InputTarget::Filter {
                match processed_key.code {
                    KeyCode::Enter => self.input_mode = false,
                    KeyCode::Esc => { self.input_mode = false; self.show_filter_input = false; self.filter_input.clear(); self.filter_lists(); }
                    KeyCode::Char(c) => { self.filter_input.push(c); self.filter_lists(); }
                    KeyCode::Backspace => { self.filter_input.pop(); self.filter_lists(); }
                    _ => {}
                }
            } else {
                match processed_key.code {
                    KeyCode::Enter if !processed_key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) && !processed_key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) => {
                         if matches!(self.input_target, InputTarget::EditTitle | InputTarget::SnoozeUntil) {
                             self.enter().await;
                         } else {
                             self.textarea.input(processed_key);
                         }
                    }
                    KeyCode::Enter if processed_key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) || processed_key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) => {
                        self.enter().await;
                    }
                    KeyCode::Esc => { self.input_mode = false; }
                    _ => { self.textarea.input(processed_key); }
                }
            }
            return;
        }

        if self.view_mode == ViewMode::JobLog {
            match processed_key.code {
                KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => { self.toggle_job_section(); let _ = self.update_preview().await; return; }
                KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => { self.view_mode = ViewMode::List; let _ = self.update_preview().await; return; }
                KeyCode::Down | KeyCode::Char('j') => { self.job_log_scroll = self.job_log_scroll.saturating_add(1); let _ = self.update_preview().await; return; }
                KeyCode::Up | KeyCode::Char('k') => { self.job_log_scroll = self.job_log_scroll.saturating_sub(1); let _ = self.update_preview().await; return; }
                _ => {}
            }
        }

        if self.view_mode == ViewMode::Diff {
            match processed_key.code {
                KeyCode::Esc => { self.view_mode = ViewMode::List; let _ = self.update_preview().await; return; }
                KeyCode::Left | KeyCode::Char('h') => { self.diff_line_state.select(None); let _ = self.update_preview().await; return; }
                KeyCode::Right | KeyCode::Char('l') => { if self.diff_line_state.selected().is_none() { self.diff_line_state.select(Some(0)); } let _ = self.update_preview().await; return; }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.diff_line_state.selected().is_some() {
                        if let Some(selected_idx) = self.mr_diff_state.selected() {
                            if let Some(mr_diff) = self.mr_diffs.get(selected_idx) {
                                let line_count = mr_diff.display_lines.len();
                                if line_count > 0 {
                                    let i = (self.diff_line_state.selected().unwrap() + 1) % line_count;
                                    self.diff_line_state.select(Some(i));
                                }
                            }
                        }
                    } else {
                        let i = (self.mr_diff_state.selected().unwrap_or(0) + 1) % self.mr_diffs.len().max(1);
                        self.mr_diff_state.select(Some(i));
                        self.diff_line_state.select(None);
                    }
                    let _ = self.update_preview().await; return;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.diff_line_state.selected().is_some() {
                        if let Some(selected_idx) = self.mr_diff_state.selected() {
                            if let Some(mr_diff) = self.mr_diffs.get(selected_idx) {
                                let line_count = mr_diff.display_lines.len();
                                if line_count > 0 {
                                    let i = if self.diff_line_state.selected().unwrap() == 0 { line_count - 1 } else { self.diff_line_state.selected().unwrap() - 1 };
                                    self.diff_line_state.select(Some(i));
                                }
                            }
                        }
                    } else {
                        let i = if self.mr_diff_state.selected().unwrap_or(0) == 0 { self.mr_diffs.len().saturating_sub(1) } else { self.mr_diff_state.selected().unwrap() - 1 };
                        self.mr_diff_state.select(Some(i));
                        self.diff_line_state.select(None);
                    }
                    let _ = self.update_preview().await; return;
                }
                KeyCode::Char('c') => {
                    if let Some(selected_idx) = self.mr_diff_state.selected() {
                        if let Some(mr_diff) = self.mr_diffs.get(selected_idx) {
                            if let Some(line_idx) = self.diff_line_state.selected() {
                                if let Some(DiffDisplayLine::Code(..)) = mr_diff.display_lines.get(line_idx) {
                                    self.input_mode = true;
                                    self.input_target = InputTarget::NewComment;
                                    self.textarea = ratatui_textarea::TextArea::default();
                                    return;
                                }
                            }
                        }
                    }
                }
                KeyCode::Char('r') => {
                    if let Some(selected_idx) = self.mr_diff_state.selected() {
                        if let Some(mr_diff) = self.mr_diffs.get(selected_idx) {
                            if let Some(line_idx) = self.diff_line_state.selected() {
                                if let Some(DiffDisplayLine::Discussion(disc)) = mr_diff.display_lines.get(line_idx) {
                                    if let Some(pos) = self.discussions.iter().position(|d| d.id == disc.id) {
                                        self.discussion_state.select(Some(pos));
                                        self.input_mode = true;
                                        self.input_target = InputTarget::Reply;
                                        self.textarea = ratatui_textarea::TextArea::default();
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    if let Some(selected_idx) = self.mr_diff_state.selected() {
                        if let Some(mr_diff) = self.mr_diffs.get_mut(selected_idx) {
                            if let Some(line_idx) = self.diff_line_state.selected() {
                                if let Some(DiffDisplayLine::Discussion(disc)) = mr_diff.display_lines.get_mut(line_idx) {
                                    disc.collapsed = !disc.collapsed;
                                    if let Some(main_disc) = self.discussions.iter_mut().find(|d| d.id == disc.id) {
                                        main_disc.collapsed = disc.collapsed;
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        match processed_key.code {
            KeyCode::Tab => {
                self.active_panel = match self.active_panel {
                    Panel::Main => match self.active_context { Context::Dashboard => Panel::Dashboard, Context::Explore => Panel::Explore, Context::Project => Panel::Project },
                    Panel::Dashboard | Panel::Explore | Panel::Project => Panel::MainDetail,
                    Panel::MainDetail => Panel::Main,
                };
                let _ = self.update_preview().await;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.active_panel = match self.active_panel {
                    Panel::Dashboard | Panel::Explore | Panel::Project | Panel::Main => Panel::MainDetail,
                    Panel::MainDetail => Panel::MainDetail,
                };
                let _ = self.update_preview().await;
            }
            KeyCode::BackTab => {
                self.active_panel = match self.active_panel {
                    Panel::Main => Panel::MainDetail,
                    Panel::Dashboard | Panel::Explore | Panel::Project => Panel::Main,
                    Panel::MainDetail => match self.active_context { Context::Dashboard => Panel::Dashboard, Context::Explore => Panel::Explore, Context::Project => Panel::Project },
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
            KeyCode::Enter => {
                match self.active_panel {
                    Panel::Dashboard | Panel::Explore | Panel::Project => {
                        self.active_context = match self.active_panel { Panel::Dashboard => Context::Dashboard, Panel::Explore => Context::Explore, Panel::Project => Context::Project, _ => self.active_context };
                        let idx = match self.active_panel { Panel::Dashboard => self.dashboard_menu_state.selected().unwrap_or(0), Panel::Explore => self.explore_menu_state.selected().unwrap_or(0), Panel::Project => self.project_menu_state.selected().unwrap_or(0), _ => 0 };
                        self.active_selection = match (self.active_panel, idx) { (Panel::Dashboard, 0) => MenuSelection::Todos, (Panel::Dashboard, 1) => MenuSelection::MergeRequests, (Panel::Dashboard, 2) => MenuSelection::Issues, (Panel::Explore, 0) => MenuSelection::Projects, (Panel::Explore, 1) => MenuSelection::Groups, (Panel::Project, 0) => MenuSelection::Pipelines, (Panel::Project, 1) => MenuSelection::MergeRequests, (Panel::Project, 2) => MenuSelection::Issues, _ => self.active_selection };
                        self.active_panel = Panel::Main; 
                    }
                    Panel::Main => {
                        if self.active_selection == MenuSelection::MergeRequests && self.selected_mr.is_some() {
                            self.view_mode = ViewMode::Diff;
                            self.mr_diff_state.select(Some(0));
                            let (pid, iid) = { let m = self.selected_mr.as_ref().unwrap(); (m.project_id, m.iid) };
                            let _ = self.fetch_mr_diffs(pid, iid).await;
                        } else {
                            self.active_panel = Panel::MainDetail;
                            self.detail_section = DetailSection::Metadata;
                            self.selected_metadata_field = MetadataField::Title;
                        }
                    }
                    Panel::MainDetail => { 
                        if self.detail_section == DetailSection::Changes && self.selected_mr.is_some() {
                            self.view_mode = ViewMode::Diff;
                            self.mr_diff_state.select(Some(0));
                            let (pid, iid) = { let m = self.selected_mr.as_ref().unwrap(); (m.project_id, m.iid) };
                            let _ = self.fetch_mr_diffs(pid, iid).await;
                        } else {
                            self.enter_detail_action().await; 
                        }
                    }
                }
                let _ = self.update_preview().await;
            }
            KeyCode::Esc => {
                match self.active_panel { Panel::MainDetail => { self.active_panel = Panel::Main; } Panel::Main => { self.active_panel = match self.active_context { Context::Dashboard => Panel::Dashboard, Context::Explore => Panel::Explore, Context::Project => Panel::Project }; } _ => {} }
                let _ = self.update_preview().await;
            }
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('v') => {
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
            KeyCode::Char('o') => self.open_in_browser(),
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
            KeyCode::Char('c') => { if let Some(_) = &self.selected_issue { let _ = self.toggle_issue_state().await; } else if let Some(_) = &self.selected_mr { let _ = self.toggle_mr_state().await; } }
            KeyCode::Char('A') => { if self.selected_mr.is_some() { let _ = self.approve_mr().await; } }
            KeyCode::Char('R') => { if self.selected_mr.is_some() { let _ = self.unapprove_mr().await; } }
            KeyCode::Char('D') => { if self.selected_mr.is_some() { let _ = self.toggle_mr_draft().await; } }
            KeyCode::Char('s') => {
                if self.active_selection == MenuSelection::Todos {
                    self.show_snooze_popup = true; self.snooze_state.select(Some(0));
                } else {
                    if let Some(_) = &self.selected_mr { let _ = self.resolve_discussion().await; }
                    else if let Some(_) = &self.selected_issue { let _ = self.resolve_issue_discussion().await; }
                }
            }
            KeyCode::Char('d') if self.active_selection == MenuSelection::Todos => { let _ = self.mark_todo_as_done().await; }
            _ => {}
        }
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
            HelpAction::Filter => { self.input_mode = true; self.input_target = InputTarget::Filter; self.show_filter_input = true; }
            HelpAction::OpenBrowser => self.open_in_browser(),
            _ => {}
        }
        self.show_help_popup = false;
    }
    pub async fn back(&mut self) { self.view_mode = ViewMode::List; self.active_panel = Panel::Dashboard; }
    pub fn open_in_browser(&self) {
        let url = if let Some(mr) = &self.selected_mr { Some(mr.web_url.clone()) } else if let Some(issue) = &self.selected_issue { Some(issue.web_url.clone()) } else if let Some(pipeline) = &self.selected_pipeline { Some(pipeline.web_url.clone()) } else if let Some(job) = &self.selected_job { Some(job.web_url.clone()) } else if self.active_selection == MenuSelection::Projects { if let Some(idx) = self.projects_state.selected() { self.filtered_projects.get(idx).and_then(|p| p.web_url.clone()) } else { self.selected_project.as_ref().and_then(|p| p.web_url.clone()) } } else if self.active_selection == MenuSelection::Groups { if let Some(idx) = self.groups_state.selected() { self.filtered_groups.get(idx).and_then(|g| g.web_url.clone()) } else { None } } else if let Some(project) = &self.selected_project { project.web_url.clone() } else { None };
        if let Some(url) = url {
            #[cfg(target_os = "windows")] let _ = std::process::Command::new("cmd").arg("/c").arg("start").arg(&url).spawn();
            #[cfg(target_os = "macos")] let _ = std::process::Command::new("open").arg(&url).spawn();
            #[cfg(target_os = "linux")] let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
        }
    }
    pub fn parse_job_log(&mut self) { self.job_log_lines = pipelines::parse_job_log(&self.job_log); }
    pub async fn check_pagination(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.active_panel != Panel::Main { return Ok(()); }
        match self.active_selection {
            MenuSelection::Todos => { if let Some(i) = self.todos_state.selected() { if i + 10 >= self.filtered_todos.len() { self.fetch_next_todos().await?; } } }
            MenuSelection::MergeRequests => { if let Some(i) = self.mrs_state.selected() { if i + 10 >= self.filtered_mrs.len() { if self.active_context == Context::Project { self.fetch_next_merge_requests().await?; } else { self.fetch_next_global_merge_requests().await?; } } } }
            MenuSelection::Issues => { if let Some(i) = self.issues_state.selected() { if i + 10 >= self.filtered_issues.len() { if self.active_context == Context::Project { self.fetch_next_issues().await?; } else { self.fetch_next_global_issues().await?; } } } }
            MenuSelection::Pipelines => { if let Some(i) = self.pipelines_state.selected() { if i + 10 >= self.pipelines.len() { self.fetch_next_pipelines().await?; } } }
            MenuSelection::Projects => { if let Some(i) = self.projects_state.selected() { if i + 10 >= self.filtered_projects.len() { self.fetch_next_projects().await?; } } }
            MenuSelection::Groups => { if let Some(i) = self.groups_state.selected() { if i + 10 >= self.filtered_groups.len() { self.fetch_next_groups().await?; } } }
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
                    let endpoint = AddMergeRequestDiscussionNote {
                        project: mr.project_id,
                        merge_request: mr.iid,
                        discussion: disc.id.clone(),
                        body,
                    };
                    let _: serde_json::Value = endpoint.query_async(&self.gitlab).await?;
                    self.textarea = ratatui_textarea::TextArea::default();
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
                        body,
                    };
                    let _: serde_json::Value = endpoint.query_async(&self.gitlab).await?;
                    self.textarea = ratatui_textarea::TextArea::default();
                    self.input_mode = false;
                    let _ = self.fetch_issue_discussions(issue.project_id, issue.iid).await;
                }
            }
        }
        Ok(())
    }
    pub fn toggle_focus(&mut self) { self.active_panel = match self.active_panel { Panel::Main | Panel::MainDetail => match self.active_context { Context::Dashboard => Panel::Dashboard, Context::Explore => Panel::Explore, Context::Project => Panel::Project }, _ => Panel::Main }; }
    pub fn switch_project(&mut self) { self.show_project_popup = true; let _ = self.fetch_projects(); }
    pub async fn select_project(&mut self) { if let Some(i) = self.projects_state.selected() { if let Some(project) = self.projects.get(i).cloned() { self.selected_project = Some(project); self.show_project_popup = false; self.active_panel = Panel::Project; self.active_context = Context::Project; self.project_menu_state.select(Some(0)); self.active_selection = MenuSelection::Pipelines; self.selected_mr = None; self.selected_issue = None; self.selected_pipeline = None; self.discussions.clear(); self.jobs.clear(); self.merge_requests.clear(); self.issues.clear(); self.pipelines.clear(); self.filter_lists(); let _ = self.refresh().await; } } }
}

pub fn parse_reactions(reaction_values: Vec<serde_json::Value>) -> Vec<AwardEmoji> {
    let mut reactions = Vec::new();
    for val in reaction_values {
        let name = val.get("name").and_then(|v| v.as_str()).or_else(|| val.get("emoji").and_then(|v| v.as_str())).or_else(|| val.get("emoji_name").and_then(|v| v.as_str()));
        if let Some(n) = name { let display_name = if let Some(emoji) = emojis::get_by_shortcode(n) { emoji.as_str().to_string() } else { n.to_string() }; reactions.push(AwardEmoji { name: display_name }); }
    }
    reactions
}
