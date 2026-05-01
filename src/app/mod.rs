pub mod issues;
pub mod mrs;
pub mod pipelines;
pub mod global;

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
                ("Dashboard (Todos)".to_string(), MenuSelection::Todos),
                ("Merge Requests".to_string(), MenuSelection::MergeRequests),
                ("Issues".to_string(), MenuSelection::Issues),
                ("Pipelines".to_string(), MenuSelection::Pipelines),
                ("Explore Projects".to_string(), MenuSelection::Projects),
                ("Explore Groups".to_string(), MenuSelection::Groups),
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
            focus_on_menu: false,
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
                if self.focus_on_menu {
                    let i = match self.menu_state.selected() {
                        Some(i) => if i >= self.menu_items.len() - 1 { 0 } else { i + 1 },
                        None => 0,
                    };
                    self.menu_state.select(Some(i));
                    self.active_selection = self.menu_items[i].1;
                } else {
                    match self.active_selection {
                        MenuSelection::Todos => {
                            if !self.todos.is_empty() {
                                let i = match self.todos_state.selected() {
                                    Some(i) => {
                                        if i >= self.todos.len().saturating_sub(1) {
                                            if self.todos_has_more {
                                                let _ = self.fetch_next_todos();
                                            }
                                            0
                                        } else {
                                            if i >= self.todos.len().saturating_sub(5) && self.todos_has_more {
                                                // Trigger fetch next page
                                            }
                                            i + 1
                                        }
                                    },
                                    None => 0,
                                };
                                self.todos_state.select(Some(i));
                            }
                        }
                        MenuSelection::MergeRequests => {
                            if !self.merge_requests.is_empty() {
                                let i = match self.mrs_state.selected() {
                                    Some(i) => {
                                        if i >= self.merge_requests.len().saturating_sub(1) {
                                            if self.mrs_has_more {
                                                let _ = self.fetch_next_merge_requests(); // This should be handled in a way that doesn't block or we use block_on if we're in a sync context but App::handle_key is async
                                            }
                                            0
                                        } else {
                                            if i >= self.merge_requests.len().saturating_sub(5) && self.mrs_has_more {
                                                // Trigger fetch next page when 5 items from the end
                                                // Since move_down is sync, we might need to handle this carefully.
                                                // Actually, handle_key is async, but move_down is called from it.
                                            }
                                            i + 1
                                        }
                                    },
                                    None => 0,
                                };
                                self.mrs_state.select(Some(i));
                            }
                        }
                        MenuSelection::Issues => {
                            if !self.issues.is_empty() {
                                let i = match self.issues_state.selected() {
                                    Some(i) => if i >= self.issues.len().saturating_sub(1) { 0 } else { i + 1 },
                                    None => 0,
                                };
                                self.issues_state.select(Some(i));
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
                                    Some(i) => {
                                        if i >= self.projects.len().saturating_sub(1) {
                                            if self.projects_has_more {
                                                let _ = self.fetch_next_projects();
                                            }
                                            0
                                        } else {
                                            if i >= self.projects.len().saturating_sub(5) && self.projects_has_more {
                                                // Trigger fetch
                                            }
                                            i + 1
                                        }
                                    }
                                    None => 0,
                                };
                                self.projects_state.select(Some(i));
                            }
                        }
                        MenuSelection::Groups => {
                            if !self.groups.is_empty() {
                                let i = match self.groups_state.selected() {
                                    Some(i) => {
                                        if i >= self.groups.len().saturating_sub(1) {
                                            if self.groups_has_more {
                                                let _ = self.fetch_next_groups();
                                            }
                                            0
                                        } else {
                                            if i >= self.groups.len().saturating_sub(5) && self.groups_has_more {
                                                // Trigger fetch
                                            }
                                            i + 1
                                        }
                                    }
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
                if self.focus_on_menu {
                    let i = match self.menu_state.selected() {
                        Some(i) => if i == 0 { self.menu_items.len() - 1 } else { i - 1 },
                        None => 0,
                    };
                    self.menu_state.select(Some(i));
                    self.active_selection = self.menu_items[i].1;
                } else {
                    match self.active_selection {
                        MenuSelection::Todos => {
                            if !self.todos.is_empty() {
                                let i = match self.todos_state.selected() {
                                    Some(i) => if i == 0 { self.todos.len().saturating_sub(1) } else { i - 1 },
                                    None => 0,
                                };
                                self.todos_state.select(Some(i));
                            }
                        }
                        MenuSelection::MergeRequests => {
                            if !self.merge_requests.is_empty() {
                                let i = match self.mrs_state.selected() {
                                    Some(i) => if i == 0 { self.merge_requests.len().saturating_sub(1) } else { i - 1 },
                                    None => 0,
                                };
                                self.mrs_state.select(Some(i));
                            }
                        }
                        MenuSelection::Issues => {
                            if !self.issues.is_empty() {
                                let i = match self.issues_state.selected() {
                                    Some(i) => if i == 0 { self.issues.len().saturating_sub(1) } else { i - 1 },
                                    None => 0,
                                };
                                self.issues_state.select(Some(i));
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
                self.menu_state.select(Some(1)); // Merge Requests is the second item
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
                if self.focus_on_menu {
                    self.focus_on_menu = false;
                } else {
                    match self.active_selection {
                        MenuSelection::MergeRequests => {
                            if let Some(i) = self.mrs_state.selected() {
                                if let Some(mr) = self.merge_requests.get(i) {
                                    self.selected_mr = Some(mr.clone());
                                    let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                                    self.discussion_state.select(Some(0));
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
                        MenuSelection::Todos => {
                            if let Some(i) = self.todos_state.selected() {
                                if self.todos.get(i).is_some() {
                                    self.open_in_browser();
                                }
                            }
                        }
                        MenuSelection::Projects => {
                            if let Some(i) = self.projects_state.selected() {
                                if let Some(project) = self.projects.get(i) {
                                    self.selected_project = Some(project.clone());
                                    self.active_selection = MenuSelection::MergeRequests;
                                    let _ = self.fetch_merge_requests().await;
                                    let _ = self.fetch_issues().await;
                                    let _ = self.fetch_pipelines().await;
                                    let _ = self.fetch_project_metadata().await;
                                    self.focus_on_menu = true;
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
                if let Some(pipeline) = &self.selected_pipeline {
                    if let Some(i) = self.jobs_state.selected() {
                        if let Some(job) = self.jobs.get(i) {
                            self.selected_job = Some(job.clone());
                            self.follow_log = true;
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
            if self.show_filter_input {
                self.show_filter_input = false;
                self.filter_input.clear();
            }
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
        let url = if let Some(mr) = &self.selected_mr { Some(mr.web_url.clone()) }
        else if let Some(issue) = &self.selected_issue { Some(issue.web_url.clone()) }
        else if let Some(pipeline) = &self.selected_pipeline {
            self.jobs_state.selected().and_then(|i| self.jobs.get(i)).map(|job| job.web_url.clone()).or(Some(pipeline.web_url.clone()))
        } else if self.view_mode == ViewMode::List {
            match self.active_selection {
                MenuSelection::MergeRequests => self.mrs_state.selected().and_then(|i| self.merge_requests.get(i)).map(|mr| mr.web_url.clone()),
                MenuSelection::Issues => self.issues_state.selected().and_then(|i| self.issues.get(i)).map(|issue| issue.web_url.clone()),
                MenuSelection::Pipelines => self.pipelines_state.selected().and_then(|i| self.pipelines.get(i)).map(|p| p.web_url.clone()),
                MenuSelection::Todos => self.todos_state.selected().and_then(|i| self.todos.get(i)).map(|t| t.target_url.clone()),
                MenuSelection::Projects => self.projects_state.selected().and_then(|i| self.projects.get(i)).map(|p| p.web_url.clone()),
                MenuSelection::Groups => self.groups_state.selected().and_then(|i| self.groups.get(i)).map(|g| format!("https://gitlab.com/groups/{}", g.full_path)), // Simplified URL
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

    pub async fn refresh(&mut self) {
        self.last_refresh = std::time::Instant::now();
        if self.show_project_popup { let _ = self.fetch_projects().await; return; }

        if self.selected_project.is_none() {
            match self.active_selection {
                MenuSelection::Todos => { let _ = self.fetch_todos().await; }
                MenuSelection::MergeRequests => { let _ = self.fetch_global_merge_requests().await; }
                MenuSelection::Issues => { let _ = self.fetch_global_issues().await; }
                _ => {}
            }
            return;
        }

        match self.view_mode {
            ViewMode::List => match self.active_selection {
                MenuSelection::MergeRequests => { let _ = self.fetch_merge_requests().await; }
                MenuSelection::Issues => { let _ = self.fetch_issues().await; }
                MenuSelection::Pipelines => { let _ = self.fetch_pipelines().await; }
                _ => {}
            },
            ViewMode::Detail => {
                if let Some(mr) = &self.selected_mr { let _ = self.fetch_discussions(mr.project_id, mr.iid).await; }
                else if let Some(issue) = &self.selected_issue { let _ = self.fetch_issue_discussions(issue.project_id, issue.iid).await; }
                else if let Some(pipeline) = &self.selected_pipeline { let _ = self.fetch_jobs(pipeline.project_id, pipeline.id).await; }
            },
            ViewMode::JobLog => if let Some(project) = &self.selected_project { if let Some(job) = &self.selected_job { let _ = self.fetch_job_log(project.id, job.id).await; } },
            ViewMode::Diff => if let Some(mr) = &self.selected_mr { let _ = self.fetch_mr_diffs(mr.project_id, mr.iid).await; }
        }
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
            let _ = self.fetch_issues().await;
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
            let _ = self.fetch_merge_requests().await;
        }
        Ok(())
    }

    pub async fn check_pagination(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.view_mode != ViewMode::List || self.focus_on_menu { return Ok(()); }
        
        match self.active_selection {
            MenuSelection::MergeRequests => {
                if let Some(i) = self.mrs_state.selected() {
                    if i >= self.merge_requests.len().saturating_sub(5) && self.mrs_has_more {
                        self.fetch_next_merge_requests().await?;
                    }
                }
            }
            MenuSelection::Issues => {
                if let Some(i) = self.issues_state.selected() {
                    if i >= self.issues.len().saturating_sub(5) && self.issues_has_more {
                        self.fetch_next_issues().await?;
                    }
                }
            }
            MenuSelection::Pipelines => {
                if let Some(i) = self.pipelines_state.selected() {
                    if i >= self.pipelines.len().saturating_sub(5) && self.pipelines_has_more {
                        self.fetch_next_pipelines().await?;
                    }
                }
            }
            MenuSelection::Todos => {
                if let Some(i) = self.todos_state.selected() {
                    if i >= self.todos.len().saturating_sub(5) && self.todos_has_more {
                        self.fetch_next_todos().await?;
                    }
                }
            }
            MenuSelection::Projects => {
                if let Some(i) = self.projects_state.selected() {
                    if i >= self.projects.len().saturating_sub(5) && self.projects_has_more {
                        self.fetch_next_projects().await?;
                    }
                }
            }
            MenuSelection::Groups => {
                if let Some(i) = self.groups_state.selected() {
                    if i >= self.groups.len().saturating_sub(5) && self.groups_has_more {
                        self.fetch_next_groups().await?;
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
                if self.view_mode == ViewMode::JobLog {
                    self.page_down();
                } else {
                    self.move_down();
                    let _ = self.check_pagination().await;
                }
            }
            MouseEventKind::ScrollUp => {
                if self.view_mode == ViewMode::JobLog {
                    self.page_up();
                } else {
                    self.move_up();
                }
            }
            _ => {}
        }
    }

    pub fn filter_lists(&mut self) {
        let filter = self.filter_input.to_lowercase();
        
        self.filtered_todos = self.todos.iter()
            .filter(|t| t.body.to_lowercase().contains(&filter) || t.project.name.to_lowercase().contains(&filter) || t.action_name.to_lowercase().contains(&filter))
            .cloned()
            .collect();

        self.filtered_projects = self.projects.iter()
            .filter(|p| p.name.to_lowercase().contains(&filter) || p.path_with_namespace.to_lowercase().contains(&filter))
            .cloned()
            .collect();

        self.filtered_groups = self.groups.iter()
            .filter(|g| g.name.to_lowercase().contains(&filter) || g.full_path.to_lowercase().contains(&filter))
            .cloned()
            .collect();

        self.filtered_mrs = self.merge_requests.iter()
            .filter(|mr| mr.title.to_lowercase().contains(&filter) || mr.author.username.to_lowercase().contains(&filter) || mr.references.full.to_lowercase().contains(&filter))
            .cloned()
            .collect();
            
        self.filtered_issues = self.issues.iter()
            .filter(|i| i.title.to_lowercase().contains(&filter) || i.author.username.to_lowercase().contains(&filter) || format!("#{}", i.iid).contains(&filter))
            .cloned()
            .collect();
            
        self.filtered_pipelines = self.pipelines.iter()
            .filter(|p| p.ref_name.to_lowercase().contains(&filter) || p.status.to_lowercase().contains(&filter) || format!("#{}", p.id).contains(&filter))
            .cloned()
            .collect();

        // Reset selections if they are out of bounds or None
        if self.filtered_todos.is_empty() {
            self.todos_state.select(None);
        } else if self.todos_state.selected().map_or(true, |i| i >= self.filtered_todos.len()) {
            self.todos_state.select(Some(0));
        }

        if self.filtered_mrs.is_empty() {
            self.mrs_state.select(None);
        } else if self.mrs_state.selected().map_or(true, |i| i >= self.filtered_mrs.len()) {
            self.mrs_state.select(Some(0));
        }

        if self.filtered_issues.is_empty() {
            self.issues_state.select(None);
        } else if self.issues_state.selected().map_or(true, |i| i >= self.filtered_issues.len()) {
            self.issues_state.select(Some(0));
        }

        if self.filtered_pipelines.is_empty() {
            self.pipelines_state.select(None);
        } else if self.pipelines_state.selected().map_or(true, |i| i >= self.filtered_pipelines.len()) {
            self.pipelines_state.select(Some(0));
        }
    }

    pub async fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        if self.show_help_popup {
            if self.show_help_filter {
                match key.code {
                    KeyCode::Enter => {
                        self.show_help_filter = false;
                        if let Some(i) = self.help_state.selected() {
                            if let Some(item) = self.filtered_help.get(i) {
                                let action = item.action;
                                self.execute_help_action(action).await;
                            }
                        }
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
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = match self.help_state.selected() {
                            Some(i) => if i >= self.filtered_help.len().saturating_sub(1) { 0 } else { i + 1 },
                            None => 0,
                        };
                        self.help_state.select(Some(i));
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let i = match self.help_state.selected() {
                            Some(i) => if i == 0 { self.filtered_help.len().saturating_sub(1) } else { i - 1 },
                            None => 0,
                        };
                        self.help_state.select(Some(i));
                    }
                    KeyCode::Enter => {
                        if let Some(i) = self.help_state.selected() {
                            if let Some(item) = self.filtered_help.get(i) {
                                let action = item.action;
                                self.execute_help_action(action).await;
                            }
                        }
                    }
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
                    if let Some(mr) = self.selected_mr.clone().or_else(|| {
                        if self.active_selection == MenuSelection::MergeRequests {
                            self.mrs_state.selected().and_then(|i| self.filtered_mrs.get(i).cloned())
                        } else { None }
                    }) {
                        self.view_mode = ViewMode::Diff;
                        let _ = self.fetch_mr_diffs(mr.project_id, mr.iid).await;
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
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_down();
                    let _ = self.check_pagination().await;
                }
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
                        MenuSelection::Issues => {
                            self.show_create_issue_popup = true;
                            self.input_mode = true;
                            self.input_target = InputTarget::CreateIssueTitle;
                            self.create_issue_title.clear();
                            self.create_issue_description.clear();
                            self.cursor_index = 0;
                        }
                        MenuSelection::MergeRequests => {
                            self.show_create_mr_popup = true;
                            self.input_mode = true;
                            self.input_target = InputTarget::CreateMRTitle;
                            self.create_mr_title.clear();
                            self.create_mr_description.clear();
                            self.create_mr_source_branch.clear();
                            self.create_mr_target_branch.clear();
                            self.cursor_index = 0;
                        }
                        _ => {}
                    }
                }
                KeyCode::Char('p') if self.view_mode == ViewMode::List && self.active_selection == MenuSelection::Pipelines => { self.show_pipeline_launch_popup().await; }
                KeyCode::Char('p') if self.view_mode == ViewMode::Detail && self.selected_pipeline.is_some() => {
                    if let Some(proj) = &self.selected_project {
                        if let Some(i) = self.jobs_state.selected() {
                            if let Some(job) = self.jobs.get(i) {
                                if job.status == "manual" { let _ = self.play_job(proj.id, job.id).await; }
                            }
                        }
                    }
                }
                KeyCode::Char('r') if self.view_mode == ViewMode::Detail && self.selected_pipeline.is_some() => {
                    if let Some(proj) = &self.selected_project {
                        if let Some(i) = self.jobs_state.selected() {
                            if let Some(job) = self.jobs.get(i) { let _ = self.retry_job(proj.id, job.id).await; }
                        } else if let Some(pipeline) = &self.selected_pipeline { let _ = self.retry_pipeline(proj.id, pipeline.id).await; }
                    }
                }
                KeyCode::Char('c') if self.view_mode == ViewMode::List && self.active_selection == MenuSelection::Pipelines => {
                    if let Some(proj) = &self.selected_project {
                        if let Some(i) = self.pipelines_state.selected() {
                            if let Some(pipeline) = self.pipelines.get(i) { let _ = self.cancel_pipeline(proj.id, pipeline.id).await; }
                        }
                    }
                }
                KeyCode::Char('c') if self.view_mode == ViewMode::Detail && self.selected_pipeline.is_some() => {
                    if let Some(proj) = &self.selected_project {
                        if let Some(i) = self.jobs_state.selected() {
                            if let Some(job) = self.jobs.get(i) { let _ = self.cancel_job(proj.id, job.id).await; }
                        } else if let Some(pipeline) = &self.selected_pipeline { let _ = self.cancel_pipeline(proj.id, pipeline.id).await; }
                    }
                }
                KeyCode::Char(' ') => self.toggle_discussion(),
                KeyCode::Enter => self.enter().await,
                KeyCode::Backspace | KeyCode::Esc => self.back().await,
                KeyCode::Char('v') | KeyCode::Char('o') => self.open_in_browser(),
                _ => {}
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use gitlab::GitlabBuilder;
    use tokio_test::block_on;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{path, method, any};
    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};

    async fn setup_test_app() -> (MockServer, App) {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        (server, App::new(client))
    }

    fn key(code: KeyCode) -> KeyEvent { KeyEvent { code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE } }
    fn key_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent { KeyEvent { code, modifiers, kind: KeyEventKind::Press, state: KeyEventState::NONE } }

    #[test]
    fn test_exhaustive_app_logic() {
        let (server, mut app) = block_on(setup_test_app());
        app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: "n/p".into(), web_url: "u".into() });

        block_on(async {
            Mock::given(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
            Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
            Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;

            let keys = [
                KeyCode::Down, KeyCode::Up, KeyCode::Left, KeyCode::Right, KeyCode::Enter, KeyCode::Esc, KeyCode::Tab, KeyCode::Backspace, 
                KeyCode::Delete, KeyCode::Home, KeyCode::End, KeyCode::PageUp, KeyCode::PageDown,
                KeyCode::Char('j'), KeyCode::Char('k'), KeyCode::Char('h'), KeyCode::Char('l'),
                KeyCode::Char('p'), KeyCode::Char('r'), KeyCode::Char('c'), KeyCode::Char('s'),
                KeyCode::Char('n'), KeyCode::Char(' '), KeyCode::Char('v'), KeyCode::Char('o'), KeyCode::Char('q'),
                KeyCode::Char('?'), KeyCode::Char('/'), KeyCode::Char('d'), KeyCode::Char('a'), KeyCode::Char('m'), KeyCode::Char('C')
            ];

            // Populate lists and select first item
            app.project_labels = vec![Label { id: 1, name: "l".into(), color: "c".into(), text_color: "t".into() }];
            app.label_state.select(Some(0));
            app.project_members = vec![Author { id: 1, username: "u".into() }];
            app.assignee_state.select(Some(0));
            app.project_milestones = vec![Milestone { id: 1, title: "m".into() }];
            app.milestone_state.select(Some(0));
            app.pipeline_variables = vec![PipelineVariable { key: "k".into(), value: "v".into() }];
            app.pipeline_variables_state.select(Some(0));
            app.filtered_branches = vec!["b".into()];
            app.branches_state.select(Some(0));
            app.projects = vec![Project { id: 1, name: "p".into(), path_with_namespace: "n/p".into(), web_url: "u".into() }];
            app.projects_state.select(Some(0));
            app.merge_requests = vec![serde_json::from_str(r#"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z"}"#).unwrap()];
            app.mrs_state.select(Some(0));
            app.issues = vec![serde_json::from_str(r#"{"iid": 1, "project_id": 1, "title": "I", "description": "d", "author": {"id": 1, "username": "u"}, "assignees": [], "state": "opened", "web_url": "u", "labels": [], "milestone": null, "due_date": null, "weight": 5, "confidential": false, "created_at": "2026-04-25T12:00:00Z", "upvotes": 0, "downvotes": 0}"#).unwrap()];
            app.issues_state.select(Some(0));
            app.pipelines = vec![serde_json::from_str(r#"{"id": 1, "project_id": 1, "status": "success", "source": "push", "ref": "m", "web_url": "u", "created_at": "t"}"#).unwrap()];
            app.pipelines_state.select(Some(0));
            app.jobs = vec![serde_json::from_str(r#"{"id": 1, "name": "j", "stage": "s", "status": "success", "created_at": "t", "web_url": "u", "tag_list": [], "allow_failure": false}"#).unwrap()];
            app.jobs_state.select(Some(0));
            app.discussions = vec![serde_json::from_str(r#"{"id": "1", "notes": [{"id":1,"body":"b","author":{"id":1,"username":"u"},"system":false,"created_at":"t"}]}"#).unwrap()];
            app.discussion_state.select(Some(0));
            app.todos = vec![serde_json::from_str(r#"{"id": 1, "project": {"id":1,"name":"p","path_with_namespace":"n/p","web_url":"u"}, "action_name": "assigned", "target_type": "Issue", "target_url": "u", "body": "b", "state": "pending", "created_at": "t"}"#).unwrap()];
            app.todos_state.select(Some(0));

            // 1. Popups exhaustive keys
            let popups = [
                (|a: &mut App| a.show_project_popup = true),
                (|a: &mut App| a.show_calendar_popup = true),
                (|a: &mut App| a.show_label_popup = true),
                (|a: &mut App| a.show_assignee_popup = true),
                (|a: &mut App| a.show_milestone_popup = true),
                (|a: &mut App| a.show_weight_popup = true),
                (|a: &mut App| { a.show_pipeline_popup = true; a.input_target = InputTarget::PipelineVariableList; }),
                (|a: &mut App| { a.show_pipeline_popup = true; a.show_branches_dropdown = true; }),
                (|a: &mut App| a.show_help_popup = true),
                (|a: &mut App| { a.show_help_popup = true; a.show_help_filter = true; }),
                (|a: &mut App| a.show_create_issue_popup = true),
                (|a: &mut App| a.show_create_mr_popup = true),
            ];

            for setup in popups {
                app.show_project_popup = false; app.show_calendar_popup = false; app.show_label_popup = false;
                app.show_assignee_popup = false; app.show_milestone_popup = false; app.show_weight_popup = false; app.show_pipeline_popup = false;
                app.show_help_popup = false; app.show_help_filter = false; app.show_create_issue_popup = false; app.show_create_mr_popup = false;
                app.input_mode = false;
                setup(&mut app);
                for k in &keys { app.handle_key(key(k.clone())).await; }
            }

            // 2. Test Input Mode with all targets
            app.input_mode = true;
            let targets = [
                InputTarget::Reply, InputTarget::NewComment, InputTarget::EditTitle, InputTarget::EditDescription,
                InputTarget::PipelineRef, InputTarget::PipelineRefSearch, InputTarget::PipelineVariableKey,
                InputTarget::PipelineVariableValue, InputTarget::PipelineVariableList, InputTarget::Filter,
                InputTarget::CreateIssueTitle, InputTarget::CreateIssueDescription,
                InputTarget::CreateMRTitle, InputTarget::CreateMRDescription,
                InputTarget::CreateMRSourceBranch, InputTarget::CreateMRTargetBranch,
            ];
            for target in targets {
                app.input_target = target;
                app.cursor_index = 0;
                for k in &keys { app.handle_key(key(k.clone())).await; }
                app.handle_key(key_mod(KeyCode::Enter, KeyModifiers::SHIFT)).await;
            }
            app.input_mode = false;

            // 3. Test ViewModes and Selections
            for mode in [ViewMode::List, ViewMode::Detail, ViewMode::JobLog, ViewMode::Diff] {
                app.view_mode = mode;
                for sel in [MenuSelection::MergeRequests, MenuSelection::Issues, MenuSelection::Pipelines, MenuSelection::Todos] {
                    app.active_selection = sel;
                    for focus in [true, false] {
                        app.focus_on_menu = focus;
                        for k in &keys { app.handle_key(key(k.clone())).await; }
                    }
                }
            }

            // 4. Metadata fields detail navigation
            app.view_mode = ViewMode::Detail;
            app.selected_mr = Some(serde_json::from_str(r#"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z"}"#).unwrap());
            app.selected_issue = None;
            for field in [MetadataField::Title, MetadataField::Labels, MetadataField::Assignees, MetadataField::Milestone, MetadataField::Description] {
                app.selected_metadata_field = field;
                app.detail_section = DetailSection::Metadata;
                app.move_down(); app.move_up();
            }
            app.selected_issue = Some(serde_json::from_str(r#"{"iid": 1, "project_id": 1, "title": "I", "description": "d", "author": {"id": 1, "username": "u"}, "assignees": [], "state": "opened", "web_url": "u", "labels": [], "milestone": null, "due_date": null, "weight": 5, "confidential": false, "created_at": "2026-04-25T12:00:00Z", "upvotes": 0, "downvotes": 0}"#).unwrap());
            app.selected_mr = None;
            for field in [MetadataField::Milestone, MetadataField::DueDate, MetadataField::Weight, MetadataField::Confidential, MetadataField::Description] {
                app.selected_metadata_field = field;
                app.detail_section = DetailSection::Metadata;
                app.move_down(); app.move_up();
            }

            // 5. Action methods
            let _ = app.fetch_projects().await;
            let _ = app.fetch_merge_requests().await;
            let _ = app.fetch_issues().await;
            let _ = app.fetch_pipelines().await;
            let _ = app.fetch_project_metadata().await;
            let _ = app.fetch_todos().await;
            let _ = app.fetch_global_merge_requests().await;
            let _ = app.fetch_global_issues().await;
            let _ = app.fetch_mr_diffs(1, 1).await;
            let _ = app.update_issue().await;
            let _ = app.update_mr().await;
            let _ = app.approve_mr().await;
            let _ = app.unapprove_mr().await;
            let _ = app.merge_mr().await;
            let _ = app.toggle_issue_state().await;
            let _ = app.toggle_mr_state().await;
            let _ = app.toggle_confidential().await;
            let _ = app.submit_pipeline_launch().await;
            let _ = app.refresh().await;
            let _ = app.create_issue().await;
            let _ = app.create_mr().await;
            app.open_in_browser();
            app.switch_project();
            app.back().await;
            app.toggle_focus();
            app.page_down(); app.page_up();
            app.toggle_discussion();
            app.filter_lists();
            app.toggle_help();
            app.filter_help();
            app.execute_help_action(HelpAction::None).await;
        });
    }

    #[test]
    fn test_parse_reactions_logic() {
        use serde_json::json;
        let vals = vec![json!({"name": "a"}), json!({"emoji": "b"}), json!({"emoji_name": "c"})];
        let res = parse_reactions(vals);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].name, "a");
    }
}
