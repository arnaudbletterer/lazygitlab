use crate::models::{
    App, Context, DataUpdate, DetailSection, GroupedItem, MenuSelection, MetadataField, Panel,
    ViewMode,
};

impl App {
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
                let i = (self.branches_state.selected().unwrap_or(0) + 1)
                    % self.filtered_branches.len();
                self.branches_state.select(Some(i));
            }
            return;
        }
        match self.active_panel {
            Panel::Dashboard => {
                let current = self.dashboard_menu_state.selected().unwrap_or(0);
                if current >= 4 {
                    self.active_panel = Panel::Explore;
                    self.explore_menu_state.select(Some(0));
                } else {
                    self.dashboard_menu_state.select(Some(current + 1));
                }
            }
            Panel::Explore => {
                let current = self.explore_menu_state.selected().unwrap_or(0);
                if current >= 1 {
                    if self.selected_project.is_some() {
                        self.active_panel = Panel::Project;
                        self.project_menu_state.select(Some(0));
                    } else {
                        self.active_panel = Panel::Dashboard;
                        self.dashboard_menu_state.select(Some(0));
                    }
                } else {
                    self.explore_menu_state.select(Some(current + 1));
                }
            }
            Panel::Project => {
                let current = self.project_menu_state.selected().unwrap_or(0);
                let max_idx = if let Some(p) = &self.selected_project {
                    if p.namespace.as_ref().is_some_and(|n| n.kind == "group") {
                        7
                    } else {
                        6
                    }
                } else {
                    0
                };
                if current >= max_idx {
                    self.active_panel = Panel::Dashboard;
                    self.dashboard_menu_state.select(Some(0));
                } else {
                    self.project_menu_state.select(Some(current + 1));
                }
            }
            Panel::Main => {
                match self.active_selection {
                    MenuSelection::Home => {}
                    MenuSelection::Todos => {
                        if !self.filtered_todos.is_empty() {
                            let i = (self.todos_state.selected().unwrap_or(0) + 1)
                                % self.filtered_todos.len();
                            self.todos_state.select(Some(i));
                        }
                    }
                    MenuSelection::MergeRequests => {
                        if !self.filtered_mrs.is_empty() {
                            let i = (self.mrs_state.selected().unwrap_or(0) + 1)
                                % self.filtered_mrs.len();
                            self.mrs_state.select(Some(i));
                        }
                    }
                    MenuSelection::Issues => {
                        if !self.filtered_issues.is_empty() {
                            let i = (self.issues_state.selected().unwrap_or(0) + 1)
                                % self.filtered_issues.len();
                            self.issues_state.select(Some(i));
                        }
                    }
                    MenuSelection::Pipelines => {
                        if !self.pipelines.is_empty() {
                            let i = (self.pipelines_state.selected().unwrap_or(0) + 1)
                                % self.pipelines.len();
                            self.pipelines_state.select(Some(i));
                        }
                    }
                    MenuSelection::Projects => {
                        if !self.projects.is_empty() {
                            let i = (self.projects_state.selected().unwrap_or(0) + 1)
                                % self.projects.len();
                            self.projects_state.select(Some(i));
                        }
                    }
                    MenuSelection::Groups => {
                        if !self.groups.is_empty() {
                            let i =
                                (self.groups_state.selected().unwrap_or(0) + 1) % self.groups.len();
                            self.groups_state.select(Some(i));
                        }
                    }
                    MenuSelection::Repository => {
                        self.repository_tree_state.key_down();
                    }
                    MenuSelection::Branches => {
                        if !self.project_branches.is_empty() {
                            let i = (self.branches_state.selected().unwrap_or(0) + 1)
                                % self.project_branches.len();
                            self.branches_state.select(Some(i));
                        }
                    }
                    MenuSelection::Commits => {
                        if !self.commits.is_empty() {
                            let i = (self.commits_state.selected().unwrap_or(0) + 1)
                                % self.commits.len();
                            self.commits_state.select(Some(i));
                        }
                    }
                    MenuSelection::Boards => {
                        if !self.project_boards.is_empty() {
                            let i = (self.project_boards_state.selected().unwrap_or(0) + 1)
                                % self.project_boards.len();
                            self.project_boards_state.select(Some(i));
                        }
                    }
                    MenuSelection::Epics => {
                        if self.selected_epic.is_some() && !self.filtered_epic_issues.is_empty() {
                            let i = (self.epic_issues_state.selected().unwrap_or(0) + 1)
                                % self.filtered_epic_issues.len();
                            self.epic_issues_state.select(Some(i));
                        } else if !self.epics.is_empty() {
                            let i =
                                (self.epics_state.selected().unwrap_or(0) + 1) % self.epics.len();
                            self.epics_state.select(Some(i));
                        }
                    }
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
                    DetailSection::Metadata => match self.selected_metadata_field {
                        MetadataField::Title => {
                            if self.selected_mr.is_some() {
                                self.selected_metadata_field = MetadataField::TargetBranch;
                            } else {
                                self.selected_metadata_field = MetadataField::Labels;
                            }
                        }
                        MetadataField::TargetBranch => {
                            self.selected_metadata_field = MetadataField::Labels;
                        }
                        MetadataField::Labels => {
                            self.selected_metadata_field = MetadataField::Assignees;
                        }
                        MetadataField::Assignees => {
                            if self.selected_mr.is_some() {
                                self.selected_metadata_field = MetadataField::Reviewers;
                            } else if self.selected_issue.is_some() {
                                self.selected_metadata_field = MetadataField::Milestone;
                            }
                        }
                        MetadataField::Reviewers => {
                            self.selected_metadata_field = MetadataField::Milestone;
                        }
                        MetadataField::Milestone => {
                            if self.selected_issue.is_some() {
                                self.selected_metadata_field = MetadataField::StartDate;
                            }
                        }
                        MetadataField::StartDate => {
                            if self.selected_issue.is_some() {
                                self.selected_metadata_field = MetadataField::DueDate;
                            }
                        }
                        MetadataField::DueDate => {
                            if self.selected_issue.is_some() {
                                self.selected_metadata_field = MetadataField::Confidential;
                            }
                        }
                        MetadataField::Confidential => {
                            // Clamped at bottom
                        }
                        _ => {}
                    },
                    DetailSection::Description => {
                        self.description_scroll = self.description_scroll.saturating_add(1);
                    }
                    DetailSection::Discussions => {
                        if !self.discussions.is_empty() {
                            let i = (self.discussion_state.selected().unwrap_or(0) + 1)
                                % self.discussions.len();
                            self.discussion_state.select(Some(i));
                        }
                    }
                    DetailSection::Changes => {
                        if !self.mr_diffs.is_empty() {
                            let i = (self.mr_diff_state.selected().unwrap_or(0) + 1)
                                % self.mr_diffs.len();
                            self.mr_diff_state.select(Some(i));
                        }
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
                            if p.namespace.as_ref().is_some_and(|n| n.kind == "group") {
                                7
                            } else {
                                6
                            }
                        } else {
                            0
                        };
                        self.project_menu_state.select(Some(max_idx));
                    } else {
                        self.active_panel = Panel::Explore;
                        self.explore_menu_state.select(Some(1));
                    }
                } else {
                    self.dashboard_menu_state.select(Some(current - 1));
                }
            }
            Panel::Explore => {
                let current = self.explore_menu_state.selected().unwrap_or(0);
                if current == 0 {
                    self.active_panel = Panel::Dashboard;
                    self.dashboard_menu_state.select(Some(4));
                } else {
                    self.explore_menu_state.select(Some(current - 1));
                }
            }
            Panel::Project => {
                let current = self.project_menu_state.selected().unwrap_or(0);
                if current == 0 {
                    self.active_panel = Panel::Explore;
                    self.explore_menu_state.select(Some(1));
                } else {
                    self.project_menu_state.select(Some(current - 1));
                }
            }
            Panel::Main => match self.active_selection {
                MenuSelection::Home => {}
                MenuSelection::Todos => {
                    if !self.filtered_todos.is_empty() {
                        let i = if self.todos_state.selected().unwrap_or(0) == 0 {
                            self.filtered_todos.len() - 1
                        } else {
                            self.todos_state.selected().unwrap() - 1
                        };
                        self.todos_state.select(Some(i));
                    }
                }
                MenuSelection::MergeRequests => {
                    if !self.filtered_mrs.is_empty() {
                        let i = if self.mrs_state.selected().unwrap_or(0) == 0 {
                            self.filtered_mrs.len() - 1
                        } else {
                            self.mrs_state.selected().unwrap() - 1
                        };
                        self.mrs_state.select(Some(i));
                    }
                }
                MenuSelection::Issues => {
                    if !self.filtered_issues.is_empty() {
                        let i = if self.issues_state.selected().unwrap_or(0) == 0 {
                            self.filtered_issues.len() - 1
                        } else {
                            self.issues_state.selected().unwrap() - 1
                        };
                        self.issues_state.select(Some(i));
                    }
                }
                MenuSelection::Pipelines => {
                    if !self.pipelines.is_empty() {
                        let i = if self.pipelines_state.selected().unwrap_or(0) == 0 {
                            self.pipelines.len() - 1
                        } else {
                            self.pipelines_state.selected().unwrap() - 1
                        };
                        self.pipelines_state.select(Some(i));
                    }
                }
                MenuSelection::Projects => {
                    if !self.projects.is_empty() {
                        let i = if self.projects_state.selected().unwrap_or(0) == 0 {
                            self.projects.len() - 1
                        } else {
                            self.projects_state.selected().unwrap() - 1
                        };
                        self.projects_state.select(Some(i));
                    }
                }
                MenuSelection::Groups => {
                    if !self.groups.is_empty() {
                        let i = if self.groups_state.selected().unwrap_or(0) == 0 {
                            self.groups.len() - 1
                        } else {
                            self.groups_state.selected().unwrap() - 1
                        };
                        self.groups_state.select(Some(i));
                    }
                }
                MenuSelection::Repository => {
                    self.repository_tree_state.key_up();
                }
                MenuSelection::Branches => {
                    if !self.project_branches.is_empty() {
                        let i = if self.branches_state.selected().unwrap_or(0) == 0 {
                            self.project_branches.len() - 1
                        } else {
                            self.branches_state.selected().unwrap() - 1
                        };
                        self.branches_state.select(Some(i));
                    }
                }
                MenuSelection::Commits => {
                    if !self.commits.is_empty() {
                        let i = if self.commits_state.selected().unwrap_or(0) == 0 {
                            self.commits.len() - 1
                        } else {
                            self.commits_state.selected().unwrap() - 1
                        };
                        self.commits_state.select(Some(i));
                    }
                }
                MenuSelection::Boards => {
                    if !self.project_boards.is_empty() {
                        let i = if self.project_boards_state.selected().unwrap_or(0) == 0 {
                            self.project_boards.len() - 1
                        } else {
                            self.project_boards_state.selected().unwrap() - 1
                        };
                        self.project_boards_state.select(Some(i));
                    }
                }
                MenuSelection::Epics => {
                    if self.selected_epic.is_some() && !self.filtered_epic_issues.is_empty() {
                        let i = if self.epic_issues_state.selected().unwrap_or(0) == 0 {
                            self.filtered_epic_issues.len() - 1
                        } else {
                            self.epic_issues_state.selected().unwrap() - 1
                        };
                        self.epic_issues_state.select(Some(i));
                    } else if !self.epics.is_empty() {
                        let i = if self.epics_state.selected().unwrap_or(0) == 0 {
                            self.epics.len() - 1
                        } else {
                            self.epics_state.selected().unwrap() - 1
                        };
                        self.epics_state.select(Some(i));
                    }
                }
            },
            Panel::MainDetail => {
                if self.active_selection == MenuSelection::Pipelines {
                    if !self.jobs.is_empty() {
                        let i = if self.jobs_state.selected().unwrap_or(0) == 0 {
                            self.jobs.len() - 1
                        } else {
                            self.jobs_state.selected().unwrap() - 1
                        };
                        self.jobs_state.select(Some(i));
                    }
                    return;
                }
                match self.detail_section {
                    DetailSection::Metadata => {
                        match self.selected_metadata_field {
                            MetadataField::Title => {
                                // Do nothing (it is the top sub-panel of Tab 0)
                            }
                            MetadataField::TargetBranch => {
                                self.selected_metadata_field = MetadataField::Title
                            }
                            MetadataField::Labels => {
                                if self.selected_mr.is_some() {
                                    self.selected_metadata_field = MetadataField::TargetBranch;
                                } else {
                                    self.selected_metadata_field = MetadataField::Title;
                                }
                            }
                            MetadataField::Assignees => {
                                self.selected_metadata_field = MetadataField::Labels
                            }
                            MetadataField::Reviewers => {
                                self.selected_metadata_field = MetadataField::Assignees
                            }
                            MetadataField::Milestone => {
                                if self.selected_mr.is_some() {
                                    self.selected_metadata_field = MetadataField::Reviewers;
                                } else {
                                    self.selected_metadata_field = MetadataField::Assignees;
                                }
                            }
                            MetadataField::StartDate => {
                                self.selected_metadata_field = MetadataField::Milestone
                            }
                            MetadataField::DueDate => {
                                self.selected_metadata_field = MetadataField::StartDate
                            }
                            MetadataField::Confidential => {
                                self.selected_metadata_field = MetadataField::DueDate
                            }
                            _ => self.selected_metadata_field = MetadataField::Title,
                        }
                    }
                    DetailSection::Description => {
                        self.description_scroll = self.description_scroll.saturating_sub(1);
                    }
                    DetailSection::Discussions => {
                        if !self.discussions.is_empty() {
                            let i = if self.discussion_state.selected().unwrap_or(0) == 0 {
                                self.discussions.len() - 1
                            } else {
                                self.discussion_state.selected().unwrap() - 1
                            };
                            self.discussion_state.select(Some(i));
                        }
                    }
                    DetailSection::Changes => {
                        if !self.mr_diffs.is_empty() {
                            let i = if self.mr_diff_state.selected().unwrap_or(0) == 0 {
                                self.mr_diffs.len() - 1
                            } else {
                                self.mr_diff_state.selected().unwrap() - 1
                            };
                            self.mr_diff_state.select(Some(i));
                        }
                    }
                }
            }
        }
    }

    pub fn cycle_detail_tab(&mut self, forward: bool) {
        let has_changes = self.selected_mr.is_some();
        let max_tabs = if has_changes { 3 } else { 2 };
        let current_idx = match self.detail_section {
            DetailSection::Description | DetailSection::Metadata => 0,
            DetailSection::Discussions => 1,
            DetailSection::Changes => 2,
        };
        let next_idx = if forward {
            (current_idx + 1) % max_tabs
        } else {
            (current_idx + max_tabs - 1) % max_tabs
        };
        self.detail_section = match next_idx {
            0 => {
                self.selected_metadata_field = MetadataField::Title;
                DetailSection::Metadata
            }
            1 => {
                if self.discussion_state.selected().is_none() && !self.discussions.is_empty() {
                    self.discussion_state.select(Some(0));
                }
                DetailSection::Discussions
            }
            _ => {
                if self.mr_diff_state.selected().is_none() && !self.mr_diffs.is_empty() {
                    self.mr_diff_state.select(Some(0));
                }
                DetailSection::Changes
            }
        };
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
                MenuSelection::Todos => self.filtered_todos.get(i).and_then(|item| {
                    if let GroupedItem::Header(h) = item {
                        Some(h.clone())
                    } else {
                        None
                    }
                }),
                MenuSelection::MergeRequests => self.filtered_mrs.get(i).and_then(|item| {
                    if let GroupedItem::Header(h) = item {
                        Some(h.clone())
                    } else {
                        None
                    }
                }),
                MenuSelection::Issues => self.filtered_issues.get(i).and_then(|item| {
                    if let GroupedItem::Header(h) = item {
                        Some(h.clone())
                    } else {
                        None
                    }
                }),
                _ => None,
            };

            if let Some(h) = header {
                let key = format!("{}:{}", s, h);
                if self.collapsed_groups.contains(&key) {
                    self.collapsed_groups.remove(&key);
                } else {
                    self.collapsed_groups.insert(key);
                }
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
                if self.toggle_group_collapse() {
                    return;
                }
                match self.active_selection {
                    MenuSelection::Pipelines => {
                        if let Some(p) = self
                            .pipelines_state
                            .selected()
                            .and_then(|i| self.filtered_pipelines.get(i))
                        {
                            self.selected_pipeline = Some(p.clone());
                            self.active_panel = Panel::MainDetail;
                            self.detail_section = DetailSection::Metadata;
                            let _ = self.fetch_jobs(p.project_id, p.id).await;
                        }
                    }
                    MenuSelection::Projects => {
                        if let Some(i) = self.projects_state.selected() {
                            self.selected_project = Some(self.filtered_projects[i].clone());
                            self.active_panel = Panel::Project;
                            self.active_context = Context::Project;
                            self.project_menu_state.select(Some(0));
                            self.active_selection = MenuSelection::MergeRequests;
                            let _ = self.update_preview().await;
                        }
                    }
                    MenuSelection::Repository => {
                        if let Some((selected_path, item)) = self
                            .repository_tree_state
                            .selected()
                            .last()
                            .and_then(|path| {
                                self.repository_tree
                                    .iter()
                                    .find(|i| &i.path == path)
                                    .map(|item| (path, item))
                            })
                        {
                            if item.r#type == "blob" {
                                self.view_mode = ViewMode::Tree;
                                self.file_scroll = 0;
                                let _ = self.fetch_repository_file(selected_path.clone()).await;
                            } else {
                                self.repository_tree_state.toggle_selected();
                            }
                        }
                    }
                    MenuSelection::Epics => {
                        if self.selected_epic.is_none() {
                            if let Some(epic) = self
                                .epics_state
                                .selected()
                                .and_then(|i| self.epics.get(i))
                                .cloned()
                            {
                                let (group_id, iid) = (epic.group_id, epic.iid);
                                self.selected_epic = Some(epic);
                                self.epic_issues.clear();
                                self.epic_issues_state.select(None);
                                if let Some(gid) = group_id {
                                    let _ = self.fetch_epic_issues(gid, iid).await;
                                }
                            }
                        } else if let Some(issue) = self
                            .epic_issues_state
                            .selected()
                            .and_then(|i| self.filtered_epic_issues.get(i))
                            .cloned()
                        {
                            self.selected_issue = Some(issue);
                            self.selected_mr = None;
                            self.discussions.clear();
                            self.active_panel = Panel::MainDetail;
                            self.detail_section = DetailSection::Metadata;
                        }
                    }
                    MenuSelection::Boards => {
                        if let Some(board) = self
                            .project_boards_state
                            .selected()
                            .and_then(|i| self.project_boards.get(i))
                            .cloned()
                        {
                            self.selected_board = Some(board.clone());
                            self.view_mode = ViewMode::Board;
                            self.board_lists.clear();
                            self.board_issues.clear();
                            let _ = self.fetch_board_lists(board.id).await;
                        }
                    }
                    MenuSelection::Branches => {
                        if let Some(branch) = self
                            .branches_state
                            .selected()
                            .and_then(|i| self.filtered_branches.get(i))
                            .cloned()
                        {
                            let name = branch.name.clone();
                            self.active_selection = MenuSelection::Repository;
                            self.project_menu_state.select(Some(3));
                            self.view_mode = ViewMode::List;
                            self.active_panel = Panel::Main;
                            let _ = self.select_repository_ref(name).await;
                        }
                    }
                    MenuSelection::MergeRequests if self.selected_mr.is_some() => {
                        self.view_mode = ViewMode::Diff;
                        if self.mr_diff_state.selected().is_none() {
                            self.mr_diff_state.select(Some(0));
                        }
                        let (pid, iid) = {
                            let m = self.selected_mr.as_ref().unwrap();
                            (m.project_id, m.iid)
                        };
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
                    if self.mr_diff_state.selected().is_none() {
                        self.mr_diff_state.select(Some(0));
                    }
                    let (pid, iid) = {
                        let m = self.selected_mr.as_ref().unwrap();
                        (m.project_id, m.iid)
                    };
                    let _ = self.fetch_mr_diffs(pid, iid).await;
                } else if self.active_selection == MenuSelection::Pipelines {
                    // If the user pressed Enter before the jobs finished loading, fetch
                    // them in the background and open the first job once they arrive.
                    if self.jobs.is_empty()
                        && let Some(pipeline) = &self.selected_pipeline.clone()
                    {
                        self.status_message = Some("Loading pipeline jobs...".into());
                        self.open_job_when_loaded = Some((pipeline.project_id, pipeline.id));
                        let _ = self.fetch_jobs(pipeline.project_id, pipeline.id).await;
                    } else {
                        self.open_selected_job().await;
                    }
                } else {
                    self.enter_detail_action().await;
                }
            }
        }
        let _ = self.update_preview().await;
    }

    /// Opens the log of the selected job. The log is fetched on a spawned task
    /// and delivered through `DataUpdate::Log`.
    pub async fn open_selected_job(&mut self) {
        let Some(job) = self
            .jobs_state
            .selected()
            .and_then(|i| self.jobs.get(i))
            .cloned()
        else {
            if self.jobs.is_empty() {
                self.status_message = Some("No jobs in this pipeline.".into());
            }
            return;
        };
        self.selected_job = Some(job.clone());
        self.view_mode = ViewMode::JobLog;
        self.job_log_scroll = 0;
        self.job_log.clear();
        self.parse_job_log();
        self.job_log_cursor = 0;
        self.job_log_selection_start = None;
        self.follow_log = matches!(job.status.as_str(), "running" | "pending");
        let project_id = self
            .selected_pipeline
            .as_ref()
            .map(|p| p.project_id)
            .or_else(|| self.selected_project.as_ref().map(|p| p.id));
        self.job_log_gen += 1;
        self.job_log_seq = 0;
        self.job_log_applied_seq = 0;
        self.job_log_inflight = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.job_log_key = project_id.map(|p| (p, job.id));
        if let Some(project_id) = project_id {
            let _ = self.fetch_job_log(project_id, job.id).await;
        }
    }

    pub async fn go_back(&mut self) {
        self.open_job_when_loaded = None;
        match self.active_panel {
            Panel::MainDetail => {
                self.active_panel = Panel::Main;
            }
            Panel::Main => {
                if self.active_selection == MenuSelection::Epics && self.selected_epic.is_some() {
                    self.selected_epic = None;
                    self.epic_issues.clear();
                    return;
                }
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
}
