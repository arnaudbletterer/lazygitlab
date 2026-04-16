pub mod issues;
pub mod mrs;
pub mod pipelines;

use gitlab::{AsyncGitlab, api::{self, AsyncQuery}};
use crate::models::*;
use crate::api::*;
use ratatui::widgets::ListState;
use crate::ui::Theme;
use chrono::Local;

impl App {
    pub fn new(gitlab: AsyncGitlab) -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        
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

        Self {
            should_quit: false,
            menu_state,
            menu_items: vec![
                ("Merge Requests".to_string(), MenuSelection::MergeRequests),
                ("Issues".to_string(), MenuSelection::Issues),
                ("Pipelines".to_string(), MenuSelection::Pipelines),
            ],
            active_selection: MenuSelection::MergeRequests,
            gitlab,
            projects: Vec::new(),
            projects_state,
            selected_project: None,
            show_project_popup: true,
            merge_requests: Vec::new(),
            mrs_state,
            issues: Vec::new(),
            issues_state,
            pipelines: Vec::new(),
            pipelines_state,
            jobs: Vec::new(),
            jobs_state,
            focus_on_menu: false,
            view_mode: ViewMode::List,
            detail_section: DetailSection::Discussions,
            selected_metadata_field: MetadataField::Title,
            selected_mr: None,
            selected_issue: None,
            selected_pipeline: None,
            selected_job: None,
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
            theme: Theme::default(),
            last_refresh: std::time::Instant::now(),

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
        }
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
                    Some(i) => if i >= self.projects.len() - 1 { 0 } else { i + 1 },
                    None => 0,
                };
                self.projects_state.select(Some(i));
            }
            return;
        }

        match self.view_mode {
            ViewMode::List => {
                if self.focus_on_menu {
                    let i = match self.menu_state.selected() {
                        Some(i) => if i >= self.menu_items.len() - 1 { 0 } else { i + 1 },
                        None => 0,
                    };
                    self.menu_state.select(Some(i));
                    self.active_selection = self.menu_items[i].1;
                } else {
                    match self.active_selection {
                        MenuSelection::MergeRequests => {
                            if !self.merge_requests.is_empty() {
                                let i = match self.mrs_state.selected() {
                                    Some(i) => if i >= self.merge_requests.len() - 1 { 0 } else { i + 1 },
                                    None => 0,
                                };
                                self.mrs_state.select(Some(i));
                            }
                        }
                        MenuSelection::Issues => {
                            if !self.issues.is_empty() {
                                let i = match self.issues_state.selected() {
                                    Some(i) => if i >= self.issues.len() - 1 { 0 } else { i + 1 },
                                    None => 0,
                                };
                                self.issues_state.select(Some(i));
                            }
                        }
                        MenuSelection::Pipelines => {
                            if !self.pipelines.is_empty() {
                                let i = match self.pipelines_state.selected() {
                                    Some(i) => if i >= self.pipelines.len() - 1 { 0 } else { i + 1 },
                                    None => 0,
                                };
                                self.pipelines_state.select(Some(i));
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
                            Some(i) => if i >= self.jobs.len() - 1 { 0 } else { i + 1 },
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
                                if self.selected_issue.is_some() {
                                    MetadataField::DueDate
                                } else {
                                    MetadataField::Description
                                }
                            }
                            MetadataField::DueDate => {
                                if let Some(issue) = &self.selected_issue {
                                    if issue.weight.is_none() {
                                        MetadataField::Confidential
                                    } else {
                                        MetadataField::Weight
                                    }
                                } else {
                                    MetadataField::Description
                                }
                            }
                            MetadataField::Weight => MetadataField::Confidential,
                            MetadataField::Confidential => MetadataField::Description,
                            MetadataField::Description => {
                                self.detail_section = DetailSection::Discussions;
                                self.discussion_state.select(Some(0));
                                MetadataField::Title // Reset for next time
                            }
                        };
                    }
                    DetailSection::Discussions => {
                        if !self.discussions.is_empty() {
                            let current = self.discussion_state.selected().unwrap_or(0);
                            if current >= self.discussions.len() - 1 {
                                self.detail_section = DetailSection::Metadata;
                                self.selected_metadata_field = MetadataField::Title;
                                self.discussion_state.select(None);
                            } else {
                                self.discussion_state.select(Some(current + 1));
                            }
                        } else if self.selected_pipeline.is_some() && !self.jobs.is_empty() {
                            let i = match self.jobs_state.selected() {
                                Some(i) => if i >= self.jobs.len() - 1 { 0 } else { i + 1 },
                                None => 0,
                            };
                            self.jobs_state.select(Some(i));
                        } else {
                            self.detail_section = DetailSection::Metadata;
                            self.selected_metadata_field = MetadataField::Title;
                        }
                    }
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
                    Some(i) => if i == 0 { self.projects.len() - 1 } else { i - 1 },
                    None => 0,
                };
                self.projects_state.select(Some(i));
            }
            return;
        }

        match self.view_mode {
            ViewMode::List => {
                if self.focus_on_menu {
                    let i = match self.menu_state.selected() {
                        Some(i) => if i == 0 { self.menu_items.len() - 1 } else { i - 1 },
                        None => 0,
                    };
                    self.menu_state.select(Some(i));
                    self.active_selection = self.menu_items[i].1;
                } else {
                    match self.active_selection {
                        MenuSelection::MergeRequests => {
                            if !self.merge_requests.is_empty() {
                                let i = match self.mrs_state.selected() {
                                    Some(i) => if i == 0 { self.merge_requests.len() - 1 } else { i - 1 },
                                    None => 0,
                                };
                                self.mrs_state.select(Some(i));
                            }
                        }
                        MenuSelection::Issues => {
                            if !self.issues.is_empty() {
                                let i = match self.issues_state.selected() {
                                    Some(i) => if i == 0 { self.issues.len() - 1 } else { i - 1 },
                                    None => 0,
                                };
                                self.issues_state.select(Some(i));
                            }
                        }
                        MenuSelection::Pipelines => {
                            if !self.pipelines.is_empty() {
                                let i = match self.pipelines_state.selected() {
                                    Some(i) => if i == 0 { self.pipelines.len() - 1 } else { i - 1 },
                                    None => 0,
                                };
                                self.pipelines_state.select(Some(i));
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
                            Some(i) => if i == 0 { self.jobs.len() - 1 } else { i - 1 },
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
                                    self.discussion_state.select(Some(self.discussions.len() - 1));
                                }
                                MetadataField::Description // Dummy value to reset
                            }
                            MetadataField::Labels => MetadataField::Title,
                            MetadataField::Assignees => MetadataField::Labels,
                            MetadataField::Milestone => MetadataField::Assignees,
                            MetadataField::DueDate => MetadataField::Milestone,
                            MetadataField::Weight => MetadataField::DueDate,
                            MetadataField::Confidential => {
                                if let Some(issue) = &self.selected_issue {
                                    if issue.weight.is_none() {
                                        MetadataField::DueDate
                                    } else {
                                        MetadataField::Weight
                                    }
                                } else {
                                    MetadataField::Milestone // Back to milestone for MR
                                }
                            }
                            MetadataField::Description => {
                                if self.selected_issue.is_some() {
                                    MetadataField::Confidential
                                } else {
                                    MetadataField::Milestone
                                }
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
        }
    }

    pub fn toggle_focus(&mut self) {
        if !self.show_project_popup && self.view_mode == ViewMode::List {
            self.focus_on_menu = !self.focus_on_menu;
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
        }
        Ok(())
    }

    pub async fn fetch_projects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = api::projects::Projects::builder()
            .membership(true)
            .build()?;
        let projects: Vec<Project> = endpoint.query_async(&self.gitlab).await?;
        self.projects = projects;
        Ok(())
    }

    pub async fn fetch_project_metadata(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            
            let labels_endpoint = ProjectLabels { project: pid };
            self.project_labels = labels_endpoint.query_async(&self.gitlab).await?;

            let milestones_endpoint = ProjectMilestones { project: pid };
            self.project_milestones = milestones_endpoint.query_async(&self.gitlab).await?;

            let members_endpoint = ProjectMembers { project: pid };
            self.project_members = members_endpoint.query_async(&self.gitlab).await?;
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

    pub async fn select_project(&mut self) {
        if let Some(i) = self.projects_state.selected() {
            if let Some(project) = self.projects.get(i) {
                self.selected_project = Some(project.clone());
                self.show_project_popup = false;
                self.active_selection = MenuSelection::MergeRequests;
                self.menu_state.select(Some(0)); // Start on MRs tab
                self.focus_on_menu = true;
                let _ = self.fetch_merge_requests().await;
                let _ = self.fetch_issues().await;
                let _ = self.fetch_pipelines().await;
                let _ = self.fetch_project_metadata().await;
            }
        }
    }

    pub fn switch_project(&mut self) {
        self.show_project_popup = true;
        self.focus_on_menu = false;
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
            }
            return;
        }

        if self.show_project_popup {
            self.select_project().await;
            return;
        }

        if self.show_pipeline_popup {
            // Enter no longer launches, it's used for navigation/selection within input_mode
            // If not in input_mode, maybe it should focus first field? 
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
                if self.focus_on_menu {
                    self.focus_on_menu = false;
                } else {
                    match self.active_selection {
                        MenuSelection::MergeRequests => {
                            if let Some(i) = self.mrs_state.selected() {
                                if let Some(mr) = self.merge_requests.get(i) {
                                    self.selected_mr = Some(mr.clone());
                                    let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                                    self.discussion_state.select(Some(0)); // Default to first discussion
                                    self.detail_section = DetailSection::Discussions;
                                    self.view_mode = ViewMode::Detail;
                                }
                            }
                        }
                        MenuSelection::Issues => {
                            if let Some(i) = self.issues_state.selected() {
                                if let Some(issue) = self.issues.get(i) {
                                    self.selected_issue = Some(issue.clone());
                                    let _ = self.fetch_issue_discussions(issue.project_id, issue.iid).await;
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
                                    let _ = self.fetch_jobs(pipeline.project_id, pipeline.id).await;
                                    self.jobs_state.select(Some(0));
                                    self.view_mode = ViewMode::Detail;
                                }
                            }
                        }
                    }
                }
            }
            ViewMode::JobLog => {}
            ViewMode::Detail => {
                if let Some(pipeline) = &self.selected_pipeline {
                    if let Some(i) = self.jobs_state.selected() {
                        if let Some(job) = self.jobs.get(i) {
                            self.selected_job = Some(job.clone());
                            self.follow_log = true; // Reset following when opening a new job
                            let _ = self.fetch_job_log(pipeline.project_id, job.id).await;
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
        } else if self.show_project_popup {
            if self.selected_project.is_some() {
                self.show_project_popup = false;
                self.focus_on_menu = true;
                self.refresh().await;
            }
        } else if self.view_mode == ViewMode::JobLog {
            self.view_mode = ViewMode::Detail;
            self.selected_job = None;
            self.job_log.clear();
            self.refresh().await;
        } else if self.view_mode == ViewMode::Detail {
            self.view_mode = ViewMode::List;
            self.selected_mr = None;
            self.selected_issue = None;
            self.selected_pipeline = None;
            self.discussions.clear();
            self.jobs.clear();
            self.refresh().await;
        } else if !self.focus_on_menu {
            self.focus_on_menu = true;
        }
    }

    pub fn open_in_browser(&self) {
        let url = if let Some(mr) = &self.selected_mr {
            Some(mr.web_url.clone())
        } else if let Some(issue) = &self.selected_issue {
            Some(issue.web_url.clone())
        } else if let Some(pipeline) = &self.selected_pipeline {
            self.jobs_state.selected().and_then(|i| self.jobs.get(i)).map(|job| job.web_url.clone()).or(Some(pipeline.web_url.clone()))
        } else if self.view_mode == ViewMode::List {
            match self.active_selection {
                MenuSelection::MergeRequests => {
                    self.mrs_state.selected().and_then(|i| self.merge_requests.get(i)).map(|mr| mr.web_url.clone())
                }
                MenuSelection::Issues => {
                    self.issues_state.selected().and_then(|i| self.issues.get(i)).map(|issue| issue.web_url.clone())
                }
                MenuSelection::Pipelines => {
                    self.pipelines_state.selected().and_then(|i| self.pipelines.get(i)).map(|p| p.web_url.clone())
                }
            }
        } else {
            None
        };

        if let Some(url) = url {
            let _ = std::process::Command::new("powershell")
                .arg("-Command")
                .arg(format!("Start-Process '{}'", url))
                .spawn();
        }
    }

    pub async fn refresh(&mut self) {
        self.last_refresh = std::time::Instant::now();
        if self.show_project_popup {
            let _ = self.fetch_projects().await;
            return;
        }

        match self.view_mode {
            ViewMode::List => {
                match self.active_selection {
                    MenuSelection::MergeRequests => { let _ = self.fetch_merge_requests().await; }
                    MenuSelection::Issues => { let _ = self.fetch_issues().await; }
                    MenuSelection::Pipelines => { let _ = self.fetch_pipelines().await; }
                }
            }
            ViewMode::Detail => {
                if let Some(mr) = &self.selected_mr {
                    let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                } else if let Some(issue) = &self.selected_issue {
                    let _ = self.fetch_issue_discussions(issue.project_id, issue.iid).await;
                } else if let Some(pipeline) = &self.selected_pipeline {
                    let _ = self.fetch_jobs(pipeline.project_id, pipeline.id).await;
                }
            }
            ViewMode::JobLog => {
                if let Some(project) = &self.selected_project {
                    if let Some(job) = &self.selected_job {
                        let _ = self.fetch_job_log(project.id, job.id).await;
                    }
                }
            }
        }
    }

    pub fn get_input_buffer(&self) -> &String {
        match self.input_target {
            InputTarget::PipelineRef => &self.pipeline_ref_input,
            InputTarget::PipelineVariableKey => &self.pipeline_variable_key_input,
            InputTarget::PipelineVariableValue => &self.pipeline_variable_value_input,
            _ => &self.reply_input,
        }
    }

    pub fn get_input_buffer_mut(&mut self) -> &mut String {
        match self.input_target {
            InputTarget::PipelineRef => &mut self.pipeline_ref_input,
            InputTarget::PipelineVariableKey => &mut self.pipeline_variable_key_input,
            InputTarget::PipelineVariableValue => &mut self.pipeline_variable_value_input,
            _ => &mut self.reply_input,
        }
    }
}
