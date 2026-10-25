use super::is_selectable_diff_line;
use crate::models::{
    App, Context, DetailSection, DiffDisplayLine, HelpAction, HelpItem, InputTarget, IssueFilter,
    MenuSelection, MergeRequestFilter, Panel, Pipeline, PipelineFilter, TodoFilter, ViewMode,
};

impl App {
    pub fn get_available_actions(&self) -> Vec<HelpItem> {
        let mut items = vec![
            HelpItem {
                shortcut: "q".into(),
                description: "Quit application".into(),
                action: HelpAction::Quit,
                category: "Global".into(),
            },
            HelpItem {
                shortcut: "?".into(),
                description: "Show/hide help".into(),
                action: HelpAction::ToggleHelp,
                category: "Global".into(),
            },
            HelpItem {
                shortcut: "r".into(),
                description: "Refresh current view".into(),
                action: HelpAction::Refresh,
                category: "Global".into(),
            },
            HelpItem {
                shortcut: "p".into(),
                description: "Switch project".into(),
                action: HelpAction::SwitchProject,
                category: "Global".into(),
            },
            HelpItem {
                shortcut: "o".into(),
                description: "Open in browser".into(),
                action: HelpAction::OpenBrowser,
                category: "Global".into(),
            },
            HelpItem {
                shortcut: "y".into(),
                description: "Copy URL/Content to clipboard".into(),
                action: HelpAction::CopyUrl,
                category: "Global".into(),
            },
            HelpItem {
                shortcut: "f".into(),
                description: "Global Search".into(),
                action: HelpAction::Search,
                category: "Global".into(),
            },
        ];
        if self.view_mode == ViewMode::Diff {
            items.push(HelpItem {
                shortcut: "h / l / Enter".into(),
                description: "Switch focus between files and diff lines".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "j / k".into(),
                description: "Navigate files or diff lines".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "Shift + j/k".into(),
                description: "Select multi-line range".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "[ / ]".into(),
                description: "Jump to prev / next file or hunk".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "n / N / p".into(),
                description: "Jump to next / prev discussion thread".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "c".into(),
                description: "Add inline comment on selected line/range".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "C".into(),
                description: "Add file-level comment".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "r".into(),
                description: "Reply to selected inline discussion".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "Space".into(),
                description: "Collapse / expand discussion thread".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "s".into(),
                description: "Resolve / unresolve discussion thread".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "A".into(),
                description: "Approve Merge Request".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "R".into(),
                description: "Unapprove Merge Request".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "M".into(),
                description: "Merge Merge Request".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "D".into(),
                description: "Toggle Draft state".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "E".into(),
                description: "Rebase Merge Request".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "S".into(),
                description: "Toggle Squash on merge".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "y".into(),
                description: "Copy selected line(s) to clipboard".into(),
                action: HelpAction::None,
                category: "Diff".into(),
            });
            items.push(HelpItem {
                shortcut: "e / E".into(),
                description: "Expand collapsed diff file".into(),
                action: HelpAction::ExpandCollapsedDiff,
                category: "Diff".into(),
            });
        }
        if self.view_mode == ViewMode::Tree {
            items.push(HelpItem {
                shortcut: "b".into(),
                description: "Browse another branch".into(),
                action: HelpAction::None,
                category: "Repository".into(),
            });
        }
        if self.selected_mr.is_some() {
            items.push(HelpItem {
                shortcut: "Y".into(),
                description: "Copy source branch name".into(),
                action: HelpAction::None,
                category: "Merge Request".into(),
            });
            if self
                .selected_mr
                .as_ref()
                .is_some_and(|m| self.mr_pipeline_id(m).is_some())
            {
                items.push(HelpItem {
                    shortcut: "P".into(),
                    description: "Open head pipeline jobs".into(),
                    action: HelpAction::None,
                    category: "Merge Request".into(),
                });
            }
            items.push(HelpItem {
                shortcut: "A".into(),
                description: "Approve Merge Request".into(),
                action: HelpAction::None,
                category: "Merge Request".into(),
            });
            items.push(HelpItem {
                shortcut: "R".into(),
                description: "Unapprove Merge Request".into(),
                action: HelpAction::None,
                category: "Merge Request".into(),
            });
            items.push(HelpItem {
                shortcut: "M".into(),
                description: "Merge Merge Request".into(),
                action: HelpAction::None,
                category: "Merge Request".into(),
            });
            items.push(HelpItem {
                shortcut: "D".into(),
                description: "Toggle Draft state".into(),
                action: HelpAction::None,
                category: "Merge Request".into(),
            });
            items.push(HelpItem {
                shortcut: "E".into(),
                description: "Rebase Merge Request".into(),
                action: HelpAction::None,
                category: "Merge Request".into(),
            });
            items.push(HelpItem {
                shortcut: "S".into(),
                description: "Toggle Squash on merge".into(),
                action: HelpAction::None,
                category: "Merge Request".into(),
            });
            items.push(HelpItem {
                shortcut: "c".into(),
                description: "Toggle Open/Closed state".into(),
                action: HelpAction::None,
                category: "Merge Request".into(),
            });
        }
        if (self.selected_mr.is_some() || self.selected_issue.is_some())
            && self.active_panel == Panel::MainDetail
        {
            items.push(HelpItem {
                shortcut: "Tab / [ / ]".into(),
                description: "Cycle detail tabs (Overview, Discussions, Changes)".into(),
                action: HelpAction::None,
                category: "Detail".into(),
            });
            items.push(HelpItem {
                shortcut: "m".into(),
                description: "Focus Metadata Inspector".into(),
                action: HelpAction::None,
                category: "Detail".into(),
            });
            items.push(HelpItem {
                shortcut: "d".into(),
                description: "Focus Description preview".into(),
                action: HelpAction::None,
                category: "Detail".into(),
            });
            items.push(HelpItem {
                shortcut: "e / Enter".into(),
                description: "Edit selected field or description".into(),
                action: HelpAction::None,
                category: "Detail".into(),
            });
        }
        if (self.selected_mr.is_some() || self.selected_issue.is_some())
            && self.active_panel == Panel::MainDetail
            && self.detail_section == DetailSection::Discussions
            && !self.discussions.is_empty()
        {
            items.push(HelpItem {
                shortcut: "Space".into(),
                description: "Collapse / expand discussion thread".into(),
                action: HelpAction::None,
                category: "Discussion".into(),
            });
            items.push(HelpItem {
                shortcut: "r / Enter".into(),
                description: "Reply to selected discussion".into(),
                action: HelpAction::None,
                category: "Discussion".into(),
            });
            items.push(HelpItem {
                shortcut: "s / S".into(),
                description: "Resolve / unresolve discussion".into(),
                action: HelpAction::None,
                category: "Discussion".into(),
            });
        }
        if self.active_selection == MenuSelection::Pipelines && self.selected_pipeline.is_some() {
            items.push(HelpItem {
                shortcut: "n".into(),
                description: "Launch new pipeline".into(),
                action: HelpAction::None,
                category: "Pipeline".into(),
            });
            items.push(HelpItem {
                shortcut: "C".into(),
                description: "Cancel selected pipeline".into(),
                action: HelpAction::None,
                category: "Pipeline".into(),
            });
            items.push(HelpItem {
                shortcut: "R".into(),
                description: "Retry selected pipeline".into(),
                action: HelpAction::None,
                category: "Pipeline".into(),
            });
        }
        if self.active_selection == MenuSelection::Pipelines
            && self.active_context == Context::Project
        {
            items.push(HelpItem {
                shortcut: "S".into(),
                description: "Pipeline schedules (Enter = run now)".into(),
                action: HelpAction::None,
                category: "Pipeline".into(),
            });
            items.push(HelpItem {
                shortcut: "v".into(),
                description: "Manage CI variables".into(),
                action: HelpAction::None,
                category: "Pipeline".into(),
            });
        }
        if self.view_mode == ViewMode::JobLog {
            items.push(HelpItem {
                shortcut: "y".into(),
                description: "Copy line or selection".into(),
                action: HelpAction::None,
                category: "Job Log".into(),
            });
            items.push(HelpItem {
                shortcut: "Shift + j/k".into(),
                description: "Extend selection".into(),
                action: HelpAction::None,
                category: "Job Log".into(),
            });
            items.push(HelpItem {
                shortcut: "Tab / Space / l".into(),
                description: "Toggle section collapse".into(),
                action: HelpAction::None,
                category: "Job Log".into(),
            });
            items.push(HelpItem {
                shortcut: "/".into(),
                description: "Search in log (Enter jumps to next match)".into(),
                action: HelpAction::Filter,
                category: "Job Log".into(),
            });
            items.push(HelpItem {
                shortcut: "n / N".into(),
                description: "Jump to next / previous search match".into(),
                action: HelpAction::None,
                category: "Job Log".into(),
            });
            items.push(HelpItem {
                shortcut: "f".into(),
                description: "Toggle auto-follow log".into(),
                action: HelpAction::None,
                category: "Job Log".into(),
            });
            items.push(HelpItem {
                shortcut: "G / Home".into(),
                description: "Jump to end / start of log".into(),
                action: HelpAction::None,
                category: "Job Log".into(),
            });
            items.push(HelpItem {
                shortcut: "PgUp / PgDn".into(),
                description: "Page through log".into(),
                action: HelpAction::None,
                category: "Job Log".into(),
            });
            if let Some(job) = &self.selected_job {
                if matches!(job.status.as_str(), "failed" | "canceled") {
                    items.push(HelpItem {
                        shortcut: "r / R".into(),
                        description: "Retry job".into(),
                        action: HelpAction::None,
                        category: "Job Log".into(),
                    });
                }
                if matches!(
                    job.status.as_str(),
                    "failed" | "manual" | "pending" | "created" | "skipped"
                ) {
                    items.push(HelpItem {
                        shortcut: "p / P".into(),
                        description: "Play job".into(),
                        action: HelpAction::None,
                        category: "Job Log".into(),
                    });
                }
                if matches!(
                    job.status.as_str(),
                    "running" | "pending" | "manual" | "created"
                ) {
                    items.push(HelpItem {
                        shortcut: "c / C".into(),
                        description: "Cancel job".into(),
                        action: HelpAction::None,
                        category: "Job Log".into(),
                    });
                }
            }
        }
        items.push(HelpItem {
            shortcut: "I".into(),
            description: "Toggle show involved-only items".into(),
            action: HelpAction::ToggleInvolvedOnly,
            category: "Filter".into(),
        });
        if self.active_selection == MenuSelection::Issues {
            items.push(HelpItem {
                shortcut: "n".into(),
                description: "Create new issue".into(),
                action: HelpAction::CreateIssue,
                category: "Issue".into(),
            });
            items.push(HelpItem {
                shortcut: "c".into(),
                description: "Toggle Open/Closed state".into(),
                action: HelpAction::ToggleIssueState,
                category: "Issue".into(),
            });
            items.push(HelpItem {
                shortcut: "V".into(),
                description: "Group issues by milestone / project".into(),
                action: HelpAction::None,
                category: "Filter".into(),
            });
        }
        if self.active_selection == MenuSelection::MergeRequests {
            items.push(HelpItem {
                shortcut: "n".into(),
                description: "Create new merge request".into(),
                action: HelpAction::CreateMR,
                category: "Merge Request".into(),
            });
        }
        if self.active_selection == MenuSelection::Todos {
            items.push(HelpItem {
                shortcut: "d".into(),
                description: "Mark To-Do as done".into(),
                action: HelpAction::MarkDone,
                category: "To-Do".into(),
            });
            items.push(HelpItem {
                shortcut: "s".into(),
                description: "Snooze To-Do".into(),
                action: HelpAction::Snooze,
                category: "To-Do".into(),
            });
        }
        if self.active_selection == MenuSelection::Branches {
            items.push(HelpItem {
                shortcut: "n".into(),
                description: "Create new branch".into(),
                action: HelpAction::None,
                category: "Branch".into(),
            });
            items.push(HelpItem {
                shortcut: "D".into(),
                description: "Delete selected branch".into(),
                action: HelpAction::None,
                category: "Branch".into(),
            });
        }
        if self.active_selection == MenuSelection::Repository {
            items.push(HelpItem {
                shortcut: "b".into(),
                description: "Switch branch".into(),
                action: HelpAction::None,
                category: "Repository".into(),
            });
            items.push(HelpItem {
                shortcut: "Enter / Space".into(),
                description: "Expand/collapse directory or open file".into(),
                action: HelpAction::None,
                category: "Repository".into(),
            });
        }
        items.push(HelpItem {
            shortcut: "1/2/3".into(),
            description: "Focus Sidebar/List/Detail".into(),
            action: HelpAction::None,
            category: "Navigation".into(),
        });
        items.push(HelpItem {
            shortcut: "h/l (Left/Right)".into(),
            description: "Move focus between columns".into(),
            action: HelpAction::None,
            category: "Navigation".into(),
        });
        items.push(HelpItem {
            shortcut: "j/k (Up/Down)".into(),
            description: "Navigate items/fields".into(),
            action: HelpAction::None,
            category: "Navigation".into(),
        });
        items.push(HelpItem {
            shortcut: "[ / ]".into(),
            description: "Cycle filter segments / detail tabs".into(),
            action: HelpAction::None,
            category: "Navigation".into(),
        });
        items.push(HelpItem {
            shortcut: "Enter".into(),
            description: "Select / Open Detail".into(),
            action: HelpAction::Select,
            category: "Navigation".into(),
        });
        items.push(HelpItem {
            shortcut: "Esc".into(),
            description: "Go back / Close panel".into(),
            action: HelpAction::Back,
            category: "Navigation".into(),
        });
        items.push(HelpItem {
            shortcut: "Tab".into(),
            description: "Next Panel / Tab".into(),
            action: HelpAction::ToggleFocus,
            category: "Navigation".into(),
        });
        items.push(HelpItem {
            shortcut: "g h".into(),
            description: "Go to Home / Launchpad".into(),
            action: HelpAction::GotoHome,
            category: "Jump".into(),
        });
        items.push(HelpItem {
            shortcut: "g t".into(),
            description: "Go to Todos".into(),
            action: HelpAction::GotoTodos,
            category: "Jump".into(),
        });
        items.push(HelpItem {
            shortcut: "g m".into(),
            description: "Go to Merge Requests".into(),
            action: HelpAction::GotoMRs,
            category: "Jump".into(),
        });
        items.push(HelpItem {
            shortcut: "g i".into(),
            description: "Go to Issues".into(),
            action: HelpAction::GotoIssues,
            category: "Jump".into(),
        });
        items.push(HelpItem {
            shortcut: "g p".into(),
            description: "Go to Pipelines".into(),
            action: HelpAction::GotoPipelines,
            category: "Jump".into(),
        });
        items.push(HelpItem {
            shortcut: "g r".into(),
            description: "Go to Repository".into(),
            action: HelpAction::GotoRepository,
            category: "Jump".into(),
        });
        items.push(HelpItem {
            shortcut: "g b".into(),
            description: "Go to Branches".into(),
            action: HelpAction::GotoBranches,
            category: "Jump".into(),
        });
        if self.view_mode == ViewMode::List {
            items.push(HelpItem {
                shortcut: "/".into(),
                description: "Filter current list".into(),
                action: HelpAction::Filter,
                category: "List".into(),
            });
        }
        if self.view_mode == ViewMode::Board {
            items.push(HelpItem {
                shortcut: "Shift + H/L".into(),
                description: "Move issue between columns".into(),
                action: HelpAction::None,
                category: "Kanban".into(),
            });
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
            HelpAction::Back => {
                let _ = self.go_back().await;
            }
            HelpAction::Select => {
                let _ = self.go_forward().await;
            }
            HelpAction::ToggleFocus => {
                self.active_panel = match self.active_panel {
                    Panel::Dashboard | Panel::Explore | Panel::Project => Panel::Main,
                    Panel::Main => Panel::MainDetail,
                    Panel::MainDetail => match self.active_context {
                        Context::Dashboard => Panel::Dashboard,
                        Context::Explore => Panel::Explore,
                        Context::Project => Panel::Project,
                    },
                };
            }
            HelpAction::Filter => {
                self.input_mode = true;
                self.input_target = InputTarget::Filter;
                self.show_filter_input = true;
            }
            HelpAction::Search => {
                self.show_search = true;
                self.input_mode = true;
                self.input_target = InputTarget::Search;
                self.search_query.clear();
            }
            HelpAction::Refresh => {
                let _ = self.refresh().await;
            }
            HelpAction::OpenBrowser => self.open_in_browser(),
            HelpAction::CopyUrl => self.copy_url(),
            HelpAction::GotoHome => {
                self.active_context = Context::Dashboard;
                self.active_selection = MenuSelection::Home;
                self.dashboard_menu_state.select(Some(0));
                self.active_panel = Panel::Main;
                self.view_mode = ViewMode::List;
            }
            HelpAction::GotoTodos => {
                self.active_context = Context::Dashboard;
                self.active_selection = MenuSelection::Todos;
                self.dashboard_menu_state.select(Some(1));
                self.active_panel = Panel::Main;
                let _ = self.refresh().await;
            }
            HelpAction::GotoMRs => {
                if self.selected_project.is_some() {
                    self.active_context = Context::Project;
                    self.active_selection = MenuSelection::MergeRequests;
                    self.project_menu_state.select(Some(0));
                } else {
                    self.active_context = Context::Dashboard;
                    self.active_selection = MenuSelection::MergeRequests;
                    self.dashboard_menu_state.select(Some(2));
                }
                self.active_panel = Panel::Main;
                let _ = self.refresh().await;
            }
            HelpAction::GotoIssues => {
                if self.selected_project.is_some() {
                    self.active_context = Context::Project;
                    self.active_selection = MenuSelection::Issues;
                    self.project_menu_state.select(Some(1));
                } else {
                    self.active_context = Context::Dashboard;
                    self.active_selection = MenuSelection::Issues;
                    self.dashboard_menu_state.select(Some(3));
                }
                self.active_panel = Panel::Main;
                let _ = self.refresh().await;
            }
            HelpAction::GotoPipelines => {
                if self.selected_project.is_some() {
                    self.active_context = Context::Project;
                    self.active_selection = MenuSelection::Pipelines;
                    self.project_menu_state.select(Some(2));
                    self.active_panel = Panel::Main;
                    let _ = self.refresh().await;
                } else {
                    self.active_context = Context::Dashboard;
                    self.active_selection = MenuSelection::Pipelines;
                    self.dashboard_menu_state.select(Some(4));
                    self.active_panel = Panel::Main;
                    let _ = self.refresh().await;
                }
            }

            HelpAction::GotoRepository => {
                if self.selected_project.is_some() {
                    self.active_context = Context::Project;
                    self.active_selection = MenuSelection::Repository;
                    self.project_menu_state.select(Some(3));
                    self.active_panel = Panel::Main;
                    let _ = self.refresh().await;
                }
            }
            HelpAction::GotoBranches => {
                if self.selected_project.is_some() {
                    self.active_context = Context::Project;
                    self.active_selection = MenuSelection::Branches;
                    self.project_menu_state.select(Some(4));
                    self.active_panel = Panel::Main;
                    let _ = self.refresh().await;
                }
            }
            HelpAction::SwitchProject => {
                let _ = self.switch_project().await;
            }
            HelpAction::ToggleInvolvedOnly => {
                self.show_involved_only = !self.show_involved_only;
                self.filter_lists();
            }
            _ => {}
        }
        self.show_help_popup = false;
        let _ = self.update_preview().await;
    }

    pub fn filter_help(&mut self) {
        let all = self.get_available_actions();
        if self.help_filter_input.is_empty() {
            self.filtered_help = all;
        } else {
            let filter = self.help_filter_input.to_lowercase();
            self.filtered_help = all
                .into_iter()
                .filter(|item| {
                    item.description.to_lowercase().contains(&filter)
                        || item.shortcut.to_lowercase().contains(&filter)
                        || item.category.to_lowercase().contains(&filter)
                })
                .collect();
        }
        if self.filtered_help.is_empty() {
            self.help_state.select(None);
        } else if self.help_state.selected().is_none()
            || self.help_state.selected().unwrap() >= self.filtered_help.len()
        {
            self.help_state.select(Some(0));
        }
    }

    pub async fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        self.last_input = std::time::Instant::now();

        // --- 0. PRE-PROCESSING ---
        let mut processed_key = key;
        if matches!(processed_key.code, KeyCode::Char(_))
            && processed_key
                .modifiers
                .contains(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            processed_key
                .modifiers
                .remove(KeyModifiers::CONTROL | KeyModifiers::ALT);
        }

        // --- 1. POPUP FOCUS (Highest Priority, return early) ---

        // Popup keyboard handling lives in  pp::popups. The 1.1-1.8
        // sections used to live inline here (~330 lines); they now
        // delegate to App::handle_popup_key which returns true if the
        // key was handled by a popup branch.
        if self.handle_popup_key(processed_key).await {
            return;
        }

        if self.auth_failed
            && !self.input_mode
            && !self.show_search
            && !self.pending_g_prefix
            && processed_key.code == KeyCode::Char('r')
            && processed_key.modifiers == KeyModifiers::NONE
        {
            self.retry_after_auth_failure().await;
            return;
        }

        // --- 2. ABSOLUTE HIGH-PRIORITY GLOBAL KEYS (only when not in input mode) ---
        if !self.input_mode {
            let is_question_mark = processed_key.code == KeyCode::Char('?')
                || (processed_key.code == KeyCode::Char('/')
                    && processed_key.modifiers.contains(KeyModifiers::SHIFT))
                || (processed_key.code == KeyCode::Char(',')
                    && processed_key.modifiers.contains(KeyModifiers::SHIFT));

            if is_question_mark {
                self.execute_help_action(HelpAction::ToggleHelp).await;
                return;
            }
            if processed_key.code == KeyCode::Char('q') {
                self.should_quit = true;
                return;
            }
        }

        // --- 3. G-PREFIX NAVIGATION ---
        if self.pending_g_prefix {
            self.pending_g_prefix = false;
            match processed_key.code {
                KeyCode::Char('h') => {
                    self.active_context = Context::Dashboard;
                    self.active_selection = MenuSelection::Home;
                    self.active_panel = Panel::Main;
                    self.dashboard_menu_state.select(Some(0));
                    self.view_mode = ViewMode::List;
                }
                KeyCode::Char('t') => {
                    self.active_context = Context::Dashboard;
                    self.active_selection = MenuSelection::Todos;
                    self.active_panel = Panel::Main;
                    self.dashboard_menu_state.select(Some(1));
                    let _ = self.refresh().await;
                }
                KeyCode::Char('m') => {
                    if self.active_context == Context::Project {
                        self.active_selection = MenuSelection::MergeRequests;
                        self.project_menu_state.select(Some(0));
                    } else {
                        self.active_context = Context::Dashboard;
                        self.active_selection = MenuSelection::MergeRequests;
                        self.dashboard_menu_state.select(Some(2));
                    }
                    self.active_panel = Panel::Main;
                    let _ = self.refresh().await;
                }
                KeyCode::Char('i') => {
                    if self.active_context == Context::Project {
                        self.active_selection = MenuSelection::Issues;
                        self.project_menu_state.select(Some(1));
                    } else {
                        self.active_context = Context::Dashboard;
                        self.active_selection = MenuSelection::Issues;
                        self.dashboard_menu_state.select(Some(3));
                    }
                    self.active_panel = Panel::Main;
                    let _ = self.refresh().await;
                }
                KeyCode::Char('p') => {
                    if self.selected_project.is_some() {
                        self.active_context = Context::Project;
                        self.active_selection = MenuSelection::Pipelines;
                        self.project_menu_state.select(Some(2));
                        self.active_panel = Panel::Main;
                        let _ = self.refresh().await;
                    } else {
                        self.active_context = Context::Dashboard;
                        self.active_selection = MenuSelection::Pipelines;
                        self.dashboard_menu_state.select(Some(4));
                        self.active_panel = Panel::Main;
                        let _ = self.refresh().await;
                    }
                }
                KeyCode::Char('r') => {
                    if self.selected_project.is_some() {
                        self.active_context = Context::Project;
                        self.active_selection = MenuSelection::Repository;
                        self.project_menu_state.select(Some(3));
                        self.active_panel = Panel::Main;
                        let _ = self.refresh().await;
                    }
                }
                KeyCode::Char('b') if self.selected_project.is_some() => {
                    self.active_context = Context::Project;
                    self.active_selection = MenuSelection::Branches;
                    self.project_menu_state.select(Some(4));
                    self.active_panel = Panel::Main;
                    let _ = self.refresh().await;
                }
                _ => {}
            }
            return;
        }

        // --- 4. SPECIALIZED VIEW MODES ---

        if !self.input_mode {
            // JobLog Mode
            if self.view_mode == ViewMode::JobLog {
                match processed_key.code {
                    KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') => {
                        self.toggle_job_section();
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Esc => {
                        if self.job_log_selection_start.is_some() {
                            self.job_log_selection_start = None;
                        } else {
                            self.view_mode = ViewMode::List;
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.view_mode = ViewMode::List;
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !self.job_log_lines.is_empty() {
                            let current = self.job_log_cursor;
                            self.job_log_cursor = (self.job_log_cursor + 1)
                                .min(self.job_log_lines.len().saturating_sub(1));
                            if processed_key.modifiers.contains(KeyModifiers::SHIFT) {
                                if self.job_log_selection_start.is_none() {
                                    self.job_log_selection_start = Some(current);
                                }
                            } else {
                                self.job_log_selection_start = None;
                            }

                            if self.job_log_cursor
                                >= self.job_log_scroll as usize
                                    + self.job_log_viewport_height as usize
                            {
                                self.job_log_scroll = self.job_log_scroll.saturating_add(1);
                            }
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.follow_log = false;
                        if !self.job_log_lines.is_empty() {
                            let current = self.job_log_cursor;
                            self.job_log_cursor = self.job_log_cursor.saturating_sub(1);
                            if processed_key.modifiers.contains(KeyModifiers::SHIFT) {
                                if self.job_log_selection_start.is_none() {
                                    self.job_log_selection_start = Some(current);
                                }
                            } else {
                                self.job_log_selection_start = None;
                            }

                            if (self.job_log_cursor as u16) < self.job_log_scroll {
                                self.job_log_scroll = self.job_log_cursor as u16;
                            }
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Char('y') => {
                        let text = if let Some(start) = self.job_log_selection_start {
                            let (s, e) = if start <= self.job_log_cursor {
                                (start, self.job_log_cursor)
                            } else {
                                (self.job_log_cursor, start)
                            };
                            self.job_log_lines[s..=e]
                                .iter()
                                .map(|l| l.content.clone())
                                .collect::<Vec<String>>()
                                .join("\n")
                        } else if !self.job_log_lines.is_empty() {
                            self.job_log_lines[self.job_log_cursor].content.clone()
                        } else {
                            String::new()
                        };
                        self.copy_to_clipboard(text);
                        self.job_log_selection_start = None;
                        return;
                    }
                    KeyCode::Char('G') | KeyCode::End => {
                        if !self.job_log_lines.is_empty() {
                            self.job_log_cursor = self.job_log_lines.len() - 1;
                            self.job_log_scroll = self
                                .job_log_cursor
                                .saturating_sub(self.job_log_viewport_height as usize / 2)
                                as u16;
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Home => {
                        self.follow_log = false;
                        self.job_log_cursor = 0;
                        self.job_log_scroll = 0;
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::PageDown => {
                        if !self.job_log_lines.is_empty() {
                            let page = (self.job_log_viewport_height as usize).max(1);
                            self.job_log_cursor =
                                (self.job_log_cursor + page).min(self.job_log_lines.len() - 1);
                            self.ensure_job_log_cursor_visible();
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::PageUp => {
                        self.follow_log = false;
                        let page = (self.job_log_viewport_height as usize).max(1);
                        self.job_log_cursor = self.job_log_cursor.saturating_sub(page);
                        self.ensure_job_log_cursor_visible();
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Char('/') => {
                        self.input_mode = true;
                        self.input_target = InputTarget::JobLogSearch;
                        self.job_log_search.clear();
                        return;
                    }
                    KeyCode::Char('n') => {
                        self.job_log_search_next();
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Char('N') => {
                        self.job_log_search_prev();
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Char('f') | KeyCode::Char('F') => {
                        self.follow_log = !self.follow_log;
                        self.status_message = Some(if self.follow_log {
                            "Auto-follow enabled".into()
                        } else {
                            "Auto-follow disabled".into()
                        });
                        if self.follow_log && !self.job_log_lines.is_empty() {
                            self.job_log_cursor = self.job_log_lines.len() - 1;
                            self.ensure_job_log_cursor_visible();
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        if let Some((j, p)) = self
                            .selected_job
                            .as_ref()
                            .zip(self.selected_project.as_ref())
                        {
                            let _ = self.retry_job(p.id, j.id).await;
                        }
                        return;
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        if let Some((j, p)) = self
                            .selected_job
                            .as_ref()
                            .zip(self.selected_project.as_ref())
                        {
                            let _ = self.cancel_job(p.id, j.id).await;
                        }
                        return;
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        if let Some((j, p)) = self
                            .selected_job
                            .as_ref()
                            .zip(self.selected_project.as_ref())
                        {
                            let _ = self.play_job(p.id, j.id).await;
                        }
                        return;
                    }
                    _ => {}
                }
                return;
            }

            // Tree Mode
            if self.view_mode == ViewMode::Tree {
                match processed_key.code {
                    KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => {
                        self.view_mode = ViewMode::List;
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let line_count = if let Some(file) = &self.selected_file_content {
                            use base64::{Engine as _, engine::general_purpose};
                            let clean_content = file.content.replace(['\n', '\r'], "");
                            if let Ok(decoded) = general_purpose::STANDARD.decode(&clean_content) {
                                if let Ok(text) = String::from_utf8(decoded) {
                                    text.lines().count()
                                } else {
                                    0
                                }
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        let max_scroll =
                            line_count.saturating_sub(self.file_viewport_height as usize) as u16;
                        if self.file_scroll < max_scroll {
                            self.file_scroll = self.file_scroll.saturating_add(1);
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.file_scroll = self.file_scroll.saturating_sub(1);
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Char('y') => {
                        if let Some(file) = &self.selected_file_content {
                            use base64::{Engine as _, engine::general_purpose};
                            let clean_content = file.content.replace(['\n', '\r'], "");
                            if let Ok(text) = general_purpose::STANDARD
                                .decode(&clean_content)
                                .map_err(|e| e.to_string())
                                .and_then(|decoded| {
                                    String::from_utf8(decoded).map_err(|e| e.to_string())
                                })
                            {
                                self.copy_to_clipboard(text);
                            }
                        }
                        return;
                    }
                    KeyCode::Char('b') => {
                        if self.project_branches.is_empty()
                            && let Some(p) = &self.selected_project
                        {
                            let _ = self.fetch_branches(p.id).await;
                        }
                        let current = self.repository_ref.clone();
                        let idx = current
                            .and_then(|r| self.project_branches.iter().position(|b| b.name == r))
                            .unwrap_or(0);
                        self.tree_branch_state.select(Some(idx));
                        self.show_tree_branch_popup = true;
                        return;
                    }
                    _ => {}
                }
                return;
            }

            // Diff Mode
            if self.view_mode == ViewMode::Diff {
                match processed_key.code {
                    KeyCode::Esc => {
                        self.view_mode = ViewMode::List;
                        self.diff_selection_start = None;
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        if self.diff_line_state.selected().is_some() {
                            self.diff_line_state.select(None);
                            self.diff_selection_start = None;
                        } else {
                            self.view_mode = ViewMode::List;
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter
                        if self.diff_line_state.selected().is_none() =>
                    {
                        if let Some(mr_diff) = self
                            .mr_diff_state
                            .selected()
                            .and_then(|i| self.mr_diffs.get(i))
                            && !mr_diff.display_lines.is_empty()
                        {
                            let idx = mr_diff
                                .display_lines
                                .iter()
                                .position(is_selectable_diff_line)
                                .unwrap_or(0);
                            self.diff_line_state.select(Some(idx));
                            self.diff_selection_start = None;
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(current) = self.diff_line_state.selected() {
                            if let Some(mr_diff) = self
                                .mr_diff_state
                                .selected()
                                .and_then(|idx| self.mr_diffs.get(idx))
                            {
                                let line_count = mr_diff.display_lines.len();
                                if line_count > 0 {
                                    let mut next = current;
                                    for step in 1..=line_count {
                                        let candidate = (current + step) % line_count;
                                        if is_selectable_diff_line(
                                            &mr_diff.display_lines[candidate],
                                        ) {
                                            next = candidate;
                                            break;
                                        }
                                    }
                                    if processed_key.modifiers.contains(KeyModifiers::SHIFT) {
                                        if self.diff_selection_start.is_none() {
                                            self.diff_selection_start = Some(current);
                                        }
                                    } else {
                                        self.diff_selection_start = None;
                                    }
                                    self.diff_line_state.select(Some(next));
                                }
                            }
                        } else {
                            let i = (self.mr_diff_state.selected().unwrap_or(0) + 1)
                                % self.mr_diffs.len().max(1);
                            self.mr_diff_state.select(Some(i));
                            self.diff_line_state.select(None);
                            self.diff_selection_start = None;
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Some(current) = self.diff_line_state.selected() {
                            if let Some(mr_diff) = self
                                .mr_diff_state
                                .selected()
                                .and_then(|idx| self.mr_diffs.get(idx))
                            {
                                let line_count = mr_diff.display_lines.len();
                                if line_count > 0 {
                                    let mut prev = current;
                                    for step in 1..=line_count {
                                        let candidate = if current >= step {
                                            current - step
                                        } else {
                                            line_count - (step - current)
                                        };
                                        if is_selectable_diff_line(
                                            &mr_diff.display_lines[candidate],
                                        ) {
                                            prev = candidate;
                                            break;
                                        }
                                    }
                                    if processed_key.modifiers.contains(KeyModifiers::SHIFT) {
                                        if self.diff_selection_start.is_none() {
                                            self.diff_selection_start = Some(current);
                                        }
                                    } else {
                                        self.diff_selection_start = None;
                                    }
                                    self.diff_line_state.select(Some(prev));
                                }
                            }
                        } else {
                            let i = if self.mr_diff_state.selected().unwrap_or(0) == 0 {
                                self.mr_diffs.len().saturating_sub(1)
                            } else {
                                self.mr_diff_state.selected().unwrap() - 1
                            };
                            self.mr_diff_state.select(Some(i));
                            self.diff_line_state.select(None);
                            self.diff_selection_start = None;
                        }
                        let _ = self.update_preview().await;
                        return;
                    }
                    KeyCode::Char('e') => {
                        if let Some((mr_diff, mr)) = self
                            .mr_diff_state
                            .selected()
                            .and_then(|idx| self.mr_diffs.get(idx))
                            .filter(|d| d.collapsed)
                            .cloned()
                            .zip(self.selected_mr.as_ref())
                        {
                            let _ = self
                                .fetch_single_file_diff(
                                    mr.project_id,
                                    mr.iid,
                                    mr_diff.old_path,
                                    mr_diff.new_path,
                                )
                                .await;
                        }
                        return;
                    }
                    KeyCode::Char('y') => {
                        if let Some(mr_diff) = self
                            .mr_diff_state
                            .selected()
                            .and_then(|idx| self.mr_diffs.get(idx))
                        {
                            let text = if let Some(start) = self.diff_selection_start {
                                let (s, e) =
                                    if start <= self.diff_line_state.selected().unwrap_or(0) {
                                        (start, self.diff_line_state.selected().unwrap_or(0))
                                    } else {
                                        (self.diff_line_state.selected().unwrap_or(0), start)
                                    };
                                mr_diff.display_lines[s..=e]
                                    .iter()
                                    .filter_map(|l| match l {
                                        DiffDisplayLine::Code(c, ..) => Some(c.clone()),
                                        _ => None,
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            } else if let Some(idx) = self.diff_line_state.selected() {
                                match &mr_diff.display_lines[idx] {
                                    DiffDisplayLine::Code(c, ..) => c.clone(),
                                    _ => String::new(),
                                }
                            } else {
                                String::new()
                            };
                            self.copy_to_clipboard(text);
                            self.status_message = Some("Copied diff content to clipboard".into());
                            self.diff_selection_start = None;
                        }
                        return;
                    }
                    KeyCode::Char('c') => {
                        let selected_valid = self
                            .mr_diff_state
                            .selected()
                            .zip(self.diff_line_state.selected())
                            .and_then(|(sel_idx, line_idx)| {
                                self.mr_diffs
                                    .get(sel_idx)
                                    .map(|mr_diff| (mr_diff, line_idx))
                            });

                        if let Some((mr_diff, line_idx)) = selected_valid {
                            let current_line = mr_diff.display_lines.get(line_idx);
                            let start_line = self
                                .diff_selection_start
                                .and_then(|start_idx| mr_diff.display_lines.get(start_idx));

                            let is_valid_line = |dl: &DiffDisplayLine| {
                                if let DiffDisplayLine::Code(_, old_line, new_line) = dl {
                                    old_line.is_some_and(|l| l > 0)
                                        || new_line.is_some_and(|l| l > 0)
                                } else {
                                    false
                                }
                            };

                            let current_valid = current_line.is_some_and(is_valid_line);
                            let start_valid = start_line.is_none_or(is_valid_line);

                            if current_valid && start_valid {
                                self.input_mode = true;
                                self.input_target = InputTarget::NewComment;
                                self.textarea = ratatui_textarea::TextArea::default();
                                return;
                            } else {
                                self.last_error = Some(
                                    "Cannot comment on this line (no valid line number)".into(),
                                );
                            }
                        }
                    }
                    KeyCode::Char('C') => {
                        if self.selected_mr.is_some() && self.mr_diff_state.selected().is_some() {
                            self.input_mode = true;
                            self.input_target = InputTarget::NewFileComment;
                            self.textarea = ratatui_textarea::TextArea::default();
                            return;
                        }
                    }
                    KeyCode::Char('r') => {
                        if let Some(pos) = self
                            .mr_diff_state
                            .selected()
                            .zip(self.diff_line_state.selected())
                            .and_then(|(sel_idx, line_idx)| {
                                self.mr_diffs
                                    .get(sel_idx)
                                    .and_then(|mr_diff| mr_diff.display_lines.get(line_idx))
                            })
                            .and_then(|line| match line {
                                DiffDisplayLine::Discussion(disc) => Some(disc.id.clone()),
                                _ => None,
                            })
                            .and_then(|disc_id| {
                                self.discussions.iter().position(|d| d.id == disc_id)
                            })
                        {
                            self.discussion_state.select(Some(pos));
                            self.input_mode = true;
                            self.input_target = InputTarget::Reply;
                            self.textarea = ratatui_textarea::TextArea::default();
                            return;
                        }
                    }
                    KeyCode::Char(' ') => {
                        if let Some(DiffDisplayLine::Discussion(disc)) = self
                            .mr_diff_state
                            .selected()
                            .zip(self.diff_line_state.selected())
                            .and_then(|(sel_idx, line_idx)| {
                                self.mr_diffs
                                    .get_mut(sel_idx)
                                    .and_then(|mr_diff| mr_diff.display_lines.get_mut(line_idx))
                            })
                        {
                            disc.collapsed = !disc.collapsed;
                            if let Some(main_disc) =
                                self.discussions.iter_mut().find(|d| d.id == disc.id)
                            {
                                main_disc.collapsed = disc.collapsed;
                            }
                        }
                    }
                    KeyCode::Char('s') => {
                        if let Some(DiffDisplayLine::Discussion(disc)) = self
                            .mr_diff_state
                            .selected()
                            .zip(self.diff_line_state.selected())
                            .and_then(|(sel_idx, line_idx)| {
                                self.mr_diffs
                                    .get_mut(sel_idx)
                                    .and_then(|mr_diff| mr_diff.display_lines.get_mut(line_idx))
                            })
                        {
                            disc.collapsed = true;
                            if let Some(pos) = self.discussions.iter().position(|d| d.id == disc.id)
                            {
                                if let Some(main_disc) = self.discussions.get_mut(pos) {
                                    main_disc.collapsed = true;
                                }
                                self.discussion_state.select(Some(pos));
                                let _ = self.resolve_discussion().await;
                            }
                        }
                    }
                    KeyCode::Char(']') => {
                        if let Some(file_idx) = self.mr_diff_state.selected() {
                            if let Some(current_line) = self.diff_line_state.selected() {
                                let mut found_hunk = None;
                                if let Some(mr_diff) = self.mr_diffs.get(file_idx) {
                                    for i in (current_line + 1)..mr_diff.display_lines.len() {
                                        if let DiffDisplayLine::Code(line, ..) =
                                            &mr_diff.display_lines[i]
                                            && line.starts_with("@@")
                                        {
                                            let target = (i + 1..mr_diff.display_lines.len())
                                                .find(|&idx| {
                                                    is_selectable_diff_line(
                                                        &mr_diff.display_lines[idx],
                                                    )
                                                })
                                                .unwrap_or(i);
                                            found_hunk = Some(target);
                                            break;
                                        }
                                    }
                                }
                                if let Some(target) = found_hunk {
                                    self.diff_line_state.select(Some(target));
                                    self.diff_selection_start = None;
                                } else if file_idx + 1 < self.mr_diffs.len() {
                                    self.mr_diff_state.select(Some(file_idx + 1));
                                    let first_sel = self.mr_diffs[file_idx + 1]
                                        .display_lines
                                        .iter()
                                        .position(is_selectable_diff_line)
                                        .unwrap_or(0);
                                    self.diff_line_state.select(Some(first_sel));
                                    self.diff_selection_start = None;
                                }
                            } else {
                                let next = (file_idx + 1) % self.mr_diffs.len().max(1);
                                self.mr_diff_state.select(Some(next));
                                self.diff_line_state.select(None);
                                self.diff_selection_start = None;
                            }
                            let _ = self.update_preview().await;
                            return;
                        }
                    }
                    KeyCode::Char('[') => {
                        if let Some(file_idx) = self.mr_diff_state.selected() {
                            if let Some(current_line) = self.diff_line_state.selected() {
                                let mut found_hunk = None;
                                if let Some(mr_diff) = self.mr_diffs.get(file_idx) {
                                    for i in (0..current_line).rev() {
                                        if let DiffDisplayLine::Code(line, ..) =
                                            &mr_diff.display_lines[i]
                                            && line.starts_with("@@")
                                        {
                                            let target = (i + 1..mr_diff.display_lines.len())
                                                .find(|&idx| {
                                                    is_selectable_diff_line(
                                                        &mr_diff.display_lines[idx],
                                                    )
                                                })
                                                .unwrap_or(i);
                                            found_hunk = Some(target);
                                            break;
                                        }
                                    }
                                }
                                if let Some(target) = found_hunk {
                                    self.diff_line_state.select(Some(target));
                                    self.diff_selection_start = None;
                                } else if file_idx > 0 {
                                    self.mr_diff_state.select(Some(file_idx - 1));
                                    let last_sel = self.mr_diffs[file_idx - 1]
                                        .display_lines
                                        .iter()
                                        .rposition(is_selectable_diff_line)
                                        .unwrap_or(0);
                                    self.diff_line_state.select(Some(last_sel));
                                    self.diff_selection_start = None;
                                }
                            } else {
                                let prev = if file_idx == 0 {
                                    self.mr_diffs.len().saturating_sub(1)
                                } else {
                                    file_idx - 1
                                };
                                self.mr_diff_state.select(Some(prev));
                                self.diff_line_state.select(None);
                                self.diff_selection_start = None;
                            }
                            let _ = self.update_preview().await;
                            return;
                        }
                    }
                    KeyCode::Char('n') => {
                        if let Some(file_idx) = self.mr_diff_state.selected() {
                            let current = self.diff_line_state.selected().unwrap_or(0);
                            for fi in file_idx..self.mr_diffs.len() {
                                let start = if fi == file_idx { current + 1 } else { 0 };
                                for (i, dl) in self.mr_diffs[fi]
                                    .display_lines
                                    .iter()
                                    .enumerate()
                                    .skip(start)
                                {
                                    if let DiffDisplayLine::Discussion(_) = dl {
                                        self.mr_diff_state.select(Some(fi));
                                        self.diff_line_state.select(Some(i));
                                        let _ = self.update_preview().await;
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char('N') | KeyCode::Char('p') => {
                        if let Some(file_idx) = self.mr_diff_state.selected() {
                            let current = self.diff_line_state.selected().unwrap_or(0);
                            for fi in (0..=file_idx).rev() {
                                let start = if fi == file_idx && current > 0 {
                                    current - 1
                                } else {
                                    self.mr_diffs[fi].display_lines.len().saturating_sub(1)
                                };
                                for (i, dl) in self.mr_diffs[fi]
                                    .display_lines
                                    .iter()
                                    .enumerate()
                                    .take(start + 1)
                                    .rev()
                                {
                                    if let DiffDisplayLine::Discussion(_) = dl {
                                        self.mr_diff_state.select(Some(fi));
                                        self.diff_line_state.select(Some(i));
                                        let _ = self.update_preview().await;
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char('A') => {
                        if self.selected_mr.is_some() {
                            let _ = self.approve_mr().await;
                        }
                        return;
                    }
                    KeyCode::Char('R') => {
                        if self.selected_mr.is_some() {
                            let _ = self.unapprove_mr().await;
                        }
                        return;
                    }
                    KeyCode::Char('M') => {
                        self.open_confirm_merge_popup();
                        return;
                    }
                    KeyCode::Char('D') => {
                        if self.selected_mr.is_some() {
                            let _ = self.toggle_mr_draft().await;
                        }
                        return;
                    }
                    KeyCode::Char('E') => {
                        if self.selected_mr.is_some() {
                            self.show_confirm_rebase_popup = true;
                        }
                        return;
                    }
                    KeyCode::Char('S') => {
                        if self.selected_mr.is_some() {
                            let _ = self.toggle_mr_squash().await;
                        }
                        return;
                    }
                    KeyCode::Char('w') | KeyCode::Char('W') => {
                        self.diff_side_by_side = !self.diff_side_by_side;
                        self.status_message = Some(if self.diff_side_by_side {
                            "Diff view: Side-by-Side".into()
                        } else {
                            "Diff view: Unified".into()
                        });
                        let _ = self.update_preview().await;
                        return;
                    }
                    _ => {}
                }
                return;
            }

            // Board Mode
            if self.view_mode == ViewMode::Board {
                match processed_key.code {
                    KeyCode::Esc => {
                        self.view_mode = ViewMode::List;
                        self.selected_board = None;
                        return;
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        let current = self.board_lists_state.selected().unwrap_or(0);
                        let next = if current == 0 {
                            self.board_lists.len().saturating_sub(1)
                        } else {
                            current - 1
                        };
                        self.board_lists_state.select(Some(next));
                        if let Some(list) = self.board_lists.get(next) {
                            let count = self
                                .filtered_board_issues
                                .get(&list.id)
                                .map(|v| v.len())
                                .unwrap_or(0);
                            let state = self.board_issue_states.entry(list.id).or_default();
                            if count > 0 && state.selected().is_none() {
                                state.select(Some(0));
                            }
                        }
                        return;
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        let current = self.board_lists_state.selected().unwrap_or(0);
                        let next = (current + 1) % self.board_lists.len().max(1);
                        self.board_lists_state.select(Some(next));
                        if let Some(list) = self.board_lists.get(next) {
                            let count = self
                                .filtered_board_issues
                                .get(&list.id)
                                .map(|v| v.len())
                                .unwrap_or(0);
                            let state = self.board_issue_states.entry(list.id).or_default();
                            if count > 0 && state.selected().is_none() {
                                state.select(Some(0));
                            }
                        }
                        return;
                    }
                    KeyCode::Char('H') => {
                        let _ = self.move_board_issue(-1).await;
                        return;
                    }
                    KeyCode::Char('L') => {
                        let _ = self.move_board_issue(1).await;
                        return;
                    }
                    KeyCode::Char('c') => {
                        if let Some(list) = self
                            .board_lists_state
                            .selected()
                            .and_then(|idx| self.board_lists.get(idx))
                        {
                            let state = self.board_issue_states.entry(list.id).or_default();
                            let issue_idx = state.selected().unwrap_or(0);
                            if let Some(issue) = self
                                .filtered_board_issues
                                .get(&list.id)
                                .and_then(|v| v.get(issue_idx))
                                .cloned()
                            {
                                self.selected_issue = Some(issue);
                                let _ = self.toggle_issue_state().await;
                            }
                        }
                        return;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(list) = self
                            .board_lists_state
                            .selected()
                            .and_then(|idx| self.board_lists.get(idx))
                        {
                            let issues = self
                                .filtered_board_issues
                                .get(&list.id)
                                .map(|v| v.len())
                                .unwrap_or(0);
                            if issues > 0 {
                                let state = self.board_issue_states.entry(list.id).or_default();
                                let next = match state.selected() {
                                    Some(c) => (c + 1) % issues,
                                    None => 0,
                                };
                                state.select(Some(next));
                            }
                        }
                        return;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Some(list) = self
                            .board_lists_state
                            .selected()
                            .and_then(|idx| self.board_lists.get(idx))
                        {
                            let issues = self
                                .filtered_board_issues
                                .get(&list.id)
                                .map(|v| v.len())
                                .unwrap_or(0);
                            if issues > 0 {
                                let state = self.board_issue_states.entry(list.id).or_default();
                                let next = match state.selected() {
                                    Some(0) | None => issues - 1,
                                    Some(c) => c - 1,
                                };
                                state.select(Some(next));
                            }
                        }
                        return;
                    }
                    KeyCode::Enter => {
                        if let Some(list) = self
                            .board_lists_state
                            .selected()
                            .and_then(|idx| self.board_lists.get(idx))
                        {
                            let state = self.board_issue_states.entry(list.id).or_default();
                            let issue_idx = state.selected().unwrap_or(0);
                            if let Some(issue) = self
                                .filtered_board_issues
                                .get(&list.id)
                                .and_then(|v| v.get(issue_idx))
                                .cloned()
                            {
                                self.selected_issue = Some(issue);
                                self.view_mode = ViewMode::Detail;
                                self.detail_section = DetailSection::Metadata;
                            }
                        }
                        return;
                    }
                    _ => {}
                }
                return;
            }
        }

        // --- 5. INPUT MODES ---

        if self.input_mode {
            if self.input_target == InputTarget::JobLogSearch {
                match processed_key.code {
                    KeyCode::Esc => {
                        self.input_mode = false;
                        self.job_log_search.clear();
                    }
                    KeyCode::Enter => {
                        self.input_mode = false;
                        let query = self.job_log_search.clone();
                        self.job_log_search_next();
                        if !query.is_empty() && self.status_message.is_none() {
                            self.status_message = Some(format!("Found '{}'", query));
                        }
                    }
                    KeyCode::Char(c) => {
                        self.job_log_search.push(c);
                    }
                    KeyCode::Backspace => {
                        self.job_log_search.pop();
                    }
                    _ => {}
                }
                return;
            }

            if self.input_target == InputTarget::Filter {
                match processed_key.code {
                    KeyCode::Esc => {
                        self.input_mode = false;
                        self.show_filter_input = false;
                        self.filter_input.clear();
                        self.filter_lists();
                    }
                    KeyCode::Down => {
                        self.move_down();
                    }
                    KeyCode::Up => {
                        self.move_up();
                    }
                    KeyCode::Enter => {
                        self.input_mode = false;
                        self.show_filter_input = false;
                        let _ = self.update_preview().await;
                    }
                    KeyCode::Char(c) => {
                        self.filter_input.push(c);
                        self.filter_lists();
                    }
                    KeyCode::Backspace => {
                        self.filter_input.pop();
                        self.filter_lists();
                    }
                    _ => {}
                }
            } else {
                // Default textarea input modes (EditTitle, EditDescription, Reply, NewComment, SnoozeUntil)
                match processed_key.code {
                    KeyCode::Enter
                        if processed_key.modifiers.contains(KeyModifiers::CONTROL)
                            || processed_key.modifiers.contains(KeyModifiers::SHIFT) =>
                    {
                        self.enter().await;
                    }
                    KeyCode::Char('\n') | KeyCode::Char('\r')
                        if processed_key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        self.enter().await;
                    }
                    KeyCode::Enter
                        if matches!(
                            self.input_target,
                            InputTarget::EditTitle | InputTarget::SnoozeUntil
                        ) =>
                    {
                        self.enter().await;
                    }
                    KeyCode::Enter => {
                        self.textarea.input(processed_key);
                    }
                    KeyCode::Esc => {
                        self.input_mode = false;
                    }
                    _ => {
                        self.textarea.input(processed_key);
                    }
                }
            }
            return;
        }

        // --- 6. DEFAULT NAVIGATION (Background) ---

        // Panel-specific keys (like Job Retry in Pipelines)
        if self.active_panel == Panel::MainDetail
            && self.active_selection == MenuSelection::Pipelines
        {
            match processed_key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    if let Some((j, p)) = self
                        .jobs_state
                        .selected()
                        .and_then(|i| self.jobs.get(i).cloned())
                        .zip(self.selected_project.as_ref())
                    {
                        let _ = self.retry_job(p.id, j.id).await;
                    }
                    return;
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    if let Some((j, p)) = self
                        .jobs_state
                        .selected()
                        .and_then(|i| self.jobs.get(i).cloned())
                        .zip(self.selected_project.as_ref())
                    {
                        let _ = self.cancel_job(p.id, j.id).await;
                    }
                    return;
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    if let Some((j, p)) = self
                        .jobs_state
                        .selected()
                        .and_then(|i| self.jobs.get(i).cloned())
                        .zip(self.selected_project.as_ref())
                    {
                        let _ = self.play_job(p.id, j.id).await;
                    }
                    return;
                }
                _ => {}
            }
        }

        // Home view launchpad interactive navigation
        if self.active_panel == Panel::Main && self.active_selection == MenuSelection::Home {
            match processed_key.code {
                KeyCode::Left | KeyCode::Char('h') => {
                    self.home_card_index = self.home_card_index.saturating_sub(1);
                    return;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.home_card_index = (self.home_card_index + 1).min(7);
                    return;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.home_card_index = self.home_card_index.saturating_sub(2);
                    return;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.home_card_index = (self.home_card_index + 2).min(7);
                    return;
                }
                KeyCode::Enter => {
                    self.execute_home_card_action().await;
                    return;
                }
                KeyCode::Char('m') => {
                    self.active_context = Context::Dashboard;
                    self.active_selection = MenuSelection::MergeRequests;
                    self.dashboard_menu_state.select(Some(2));
                    let _ = self.refresh().await;
                    return;
                }
                KeyCode::Char('t') => {
                    self.active_context = Context::Dashboard;
                    self.active_selection = MenuSelection::Todos;
                    self.dashboard_menu_state.select(Some(1));
                    let _ = self.refresh().await;
                    return;
                }
                KeyCode::Char('i') => {
                    self.active_context = Context::Dashboard;
                    self.active_selection = MenuSelection::Issues;
                    self.dashboard_menu_state.select(Some(3));
                    let _ = self.refresh().await;
                    return;
                }
                KeyCode::Char('b') => {
                    self.active_selection = MenuSelection::Boards;
                    self.view_mode = ViewMode::Board;
                    let _ = self.refresh().await;
                    return;
                }
                KeyCode::Char('p') => {
                    self.active_context = Context::Dashboard;
                    self.active_selection = MenuSelection::Pipelines;
                    self.dashboard_menu_state.select(Some(4));
                    let _ = self.refresh().await;
                    return;
                }
                KeyCode::Char('r') => {
                    self.active_context = Context::Project;
                    self.active_selection = MenuSelection::Repository;
                    let _ = self.refresh().await;
                    return;
                }
                KeyCode::Char('P') => {
                    self.show_project_popup = true;
                    return;
                }
                KeyCode::Char('?') => {
                    self.show_help_popup = true;
                    return;
                }
                _ => {}
            }
        }

        match processed_key.code {
            KeyCode::Tab => {
                if self.active_panel == Panel::MainDetail {
                    self.cycle_detail_tab(true);
                } else {
                    self.active_panel = match self.active_panel {
                        Panel::Dashboard | Panel::Explore | Panel::Project => Panel::Main,
                        Panel::Main => match self.active_context {
                            Context::Dashboard => Panel::Dashboard,
                            Context::Explore => Panel::Explore,
                            Context::Project => Panel::Project,
                        },
                        _ => Panel::Main,
                    };
                }
                let _ = self.update_preview().await;
            }
            KeyCode::BackTab => {
                if self.active_panel == Panel::MainDetail {
                    self.cycle_detail_tab(false);
                } else {
                    self.active_panel = match self.active_panel {
                        Panel::Main => match self.active_context {
                            Context::Dashboard => Panel::Dashboard,
                            Context::Explore => Panel::Explore,
                            Context::Project => Panel::Project,
                        },
                        Panel::Dashboard | Panel::Explore | Panel::Project => Panel::Main,
                        _ => Panel::Main,
                    };
                }
                let _ = self.update_preview().await;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let is_item_selected = self.selected_mr.is_some()
                    || self.selected_issue.is_some()
                    || self.selected_pipeline.is_some();
                self.active_panel = match self.active_panel {
                    Panel::Dashboard | Panel::Explore | Panel::Project => Panel::Main,
                    Panel::Main => {
                        if is_item_selected {
                            Panel::MainDetail
                        } else {
                            Panel::Main
                        }
                    }
                    Panel::MainDetail => Panel::MainDetail,
                };
                let _ = self.update_preview().await;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.active_panel = match self.active_panel {
                    Panel::MainDetail => Panel::Main,
                    Panel::Main => match self.active_context {
                        Context::Dashboard => Panel::Dashboard,
                        Context::Explore => Panel::Explore,
                        Context::Project => Panel::Project,
                    },
                    _ => self.active_panel,
                };
                let _ = self.update_preview().await;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                let _ = self.update_preview().await;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                let _ = self.update_preview().await;
            }
            KeyCode::Enter => {
                let _ = self.update_preview().await;
                self.go_forward().await;
            }
            KeyCode::Esc => {
                if !self.filter_input.is_empty() {
                    self.filter_input.clear();
                    self.filter_lists();
                } else {
                    self.go_back().await;
                }
            }
            KeyCode::Char('y') => {
                self.copy_url();
            }
            KeyCode::Char('P') => {
                let head = self.selected_mr.as_ref().and_then(|mr| {
                    self.mr_pipeline_id(mr)?;
                    mr.head_pipeline.clone()
                });
                if let Some(head) = head {
                    let pid = self
                        .selected_mr
                        .as_ref()
                        .map(|mr| mr.project_id)
                        .unwrap_or(0);
                    let source_branch = self
                        .selected_mr
                        .as_ref()
                        .map(|mr| mr.source_branch.clone())
                        .unwrap_or_default();
                    self.selected_issue = None;
                    self.selected_mr = None;
                    self.clear_related_items();
                    self.discussions.clear();
                    self.selected_pipeline = Some(Pipeline {
                        id: head.id,
                        project_id: pid,
                        status: head.status,
                        source: String::new(),
                        ref_name: source_branch,
                        web_url: String::new(),
                        created_at: String::new(),
                        started_at: None,
                        duration: None,
                    });
                    let pipeline_id = self.selected_pipeline.as_ref().map(|p| p.id).unwrap_or(0);
                    self.active_context = Context::Project;
                    self.active_selection = MenuSelection::Pipelines;
                    self.project_menu_state.select(Some(2));
                    self.active_panel = Panel::MainDetail;
                    self.detail_section = DetailSection::Metadata;
                    self.view_mode = ViewMode::List;
                    let _ = self.fetch_jobs(pid, pipeline_id).await;
                }
            }
            KeyCode::Char('Y') => {
                if let Some(mr) = &self.selected_mr {
                    let branch = mr.source_branch.clone();
                    self.copy_to_clipboard(branch);
                } else if let Some(issue) = &self.selected_issue {
                    let url = issue.web_url.clone();
                    self.copy_to_clipboard(url);
                }
            }
            KeyCode::Char('o') => {
                self.open_in_browser();
            }
            KeyCode::Char('v') => {
                if self.active_selection == MenuSelection::Pipelines
                    && self.active_context == Context::Project
                {
                    if let Some(p) = &self.selected_project {
                        let pid = p.id;
                        if self.project_variable_cache.contains_key(&pid) {
                            self.project_variables = self
                                .project_variable_cache
                                .get(&pid)
                                .cloned()
                                .unwrap_or_default();
                        } else {
                            let _ = self.fetch_project_variables(pid).await;
                        }
                    }
                    self.ci_vars_state.select(Some(0));
                    self.show_ci_vars_popup = true;
                } else if self.selected_mr.is_some() {
                    self.view_mode = ViewMode::Diff;
                    if self.mr_diff_state.selected().is_none() {
                        self.mr_diff_state.select(Some(0));
                    }
                    let (pid, iid) = {
                        let m = self.selected_mr.as_ref().unwrap();
                        (m.project_id, m.iid)
                    };
                    let _ = self.fetch_mr_diffs(pid, iid).await;
                    let _ = self.update_preview().await;
                }
            }
            KeyCode::Char('1') => {
                if !matches!(
                    self.active_panel,
                    Panel::Dashboard | Panel::Explore | Panel::Project
                ) {
                    self.active_panel = match self.active_context {
                        Context::Dashboard => Panel::Dashboard,
                        Context::Explore => Panel::Explore,
                        Context::Project => Panel::Project,
                    };
                }
                let _ = self.update_preview().await;
            }
            KeyCode::Char('2') => {
                self.active_panel = Panel::Main;
                let _ = self.update_preview().await;
            }
            KeyCode::Char('3') => {
                self.active_panel = Panel::MainDetail;
                let _ = self.update_preview().await;
            }
            KeyCode::Char('b') => {
                if self.active_selection == MenuSelection::Repository {
                    if self.project_branches.is_empty()
                        && let Some(p) = &self.selected_project
                    {
                        let _ = self.fetch_branches(p.id).await;
                    }
                    let current = self.repository_ref.clone();
                    let idx = current
                        .and_then(|r| self.project_branches.iter().position(|b| b.name == r))
                        .unwrap_or(0);
                    self.tree_branch_state.select(Some(idx));
                    self.show_tree_branch_popup = true;
                }
            }
            KeyCode::Char('p') => self.switch_project().await,
            KeyCode::Char('r') => {
                if self.active_panel == Panel::MainDetail
                    && self.detail_section == DetailSection::Discussions
                    && self.discussion_state.selected().is_some()
                {
                    self.input_mode = true;
                    self.input_target = InputTarget::Reply;
                    self.textarea = ratatui_textarea::TextArea::default();
                } else {
                    let _ = self.refresh().await;
                }
            }
            KeyCode::Char(' ') => {
                if self.active_selection == MenuSelection::Repository
                    && self.active_panel == Panel::Main
                {
                    self.go_forward().await;
                } else if self.active_panel == Panel::MainDetail
                    && self.detail_section == DetailSection::Discussions
                    && let Some(disc) = self
                        .discussion_state
                        .selected()
                        .and_then(|idx| self.discussions.get_mut(idx))
                {
                    disc.collapsed = !disc.collapsed;
                }
            }
            KeyCode::Char('/') => {
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
            KeyCode::Char('f') => {
                self.show_search = true;
                self.input_mode = true;
                self.input_target = InputTarget::Search;
                self.search_query.clear();
                self.search_results.clear();
            }
            KeyCode::Char('[') => {
                if self.active_panel == Panel::MainDetail {
                    self.cycle_detail_tab(false);
                    let _ = self.update_preview().await;
                } else {
                    match self.active_selection {
                        MenuSelection::Todos => {
                            self.todo_filter = match self.todo_filter {
                                TodoFilter::Pending => TodoFilter::Done,
                                TodoFilter::Snoozed => TodoFilter::Pending,
                                TodoFilter::Done => TodoFilter::Snoozed,
                            }
                        }
                        MenuSelection::MergeRequests => {
                            self.mr_filter = match self.mr_filter {
                                MergeRequestFilter::Opened => MergeRequestFilter::All,
                                MergeRequestFilter::All => MergeRequestFilter::Closed,
                                MergeRequestFilter::Closed => MergeRequestFilter::Merged,
                                MergeRequestFilter::Merged => MergeRequestFilter::Assigned,
                                MergeRequestFilter::Assigned => MergeRequestFilter::Review,
                                MergeRequestFilter::Review => MergeRequestFilter::Opened,
                            }
                        }
                        MenuSelection::Issues => {
                            self.issue_filter = match self.issue_filter {
                                IssueFilter::Opened => IssueFilter::All,
                                IssueFilter::Closed => IssueFilter::Opened,
                                IssueFilter::All => IssueFilter::Closed,
                            }
                        }
                        MenuSelection::Pipelines => {
                            self.pipeline_filter = match self.pipeline_filter {
                                PipelineFilter::All => PipelineFilter::Passed,
                                PipelineFilter::Passed => PipelineFilter::Failed,
                                PipelineFilter::Failed => PipelineFilter::Running,
                                PipelineFilter::Running => PipelineFilter::All,
                            };
                            self.filter_lists();
                        }
                        _ => {}
                    }
                    if self.active_selection != MenuSelection::Pipelines {
                        self.merge_requests.clear();
                        self.issues.clear();
                        self.reset_list_paging();
                        self.todos.clear();
                        let _ = self.refresh().await;
                    }
                }
            }
            KeyCode::Char(']') => {
                if self.active_panel == Panel::MainDetail {
                    self.cycle_detail_tab(true);
                    let _ = self.update_preview().await;
                } else {
                    match self.active_selection {
                        MenuSelection::Todos => {
                            self.todo_filter = match self.todo_filter {
                                TodoFilter::Pending => TodoFilter::Snoozed,
                                TodoFilter::Snoozed => TodoFilter::Done,
                                TodoFilter::Done => TodoFilter::Pending,
                            }
                        }
                        MenuSelection::MergeRequests => {
                            self.mr_filter = match self.mr_filter {
                                MergeRequestFilter::Opened => MergeRequestFilter::Review,
                                MergeRequestFilter::Review => MergeRequestFilter::Assigned,
                                MergeRequestFilter::Assigned => MergeRequestFilter::Merged,
                                MergeRequestFilter::Merged => MergeRequestFilter::Closed,
                                MergeRequestFilter::Closed => MergeRequestFilter::All,
                                MergeRequestFilter::All => MergeRequestFilter::Opened,
                            }
                        }
                        MenuSelection::Issues => {
                            self.issue_filter = match self.issue_filter {
                                IssueFilter::Opened => IssueFilter::Closed,
                                IssueFilter::Closed => IssueFilter::All,
                                IssueFilter::All => IssueFilter::Opened,
                            }
                        }
                        MenuSelection::Pipelines => {
                            self.pipeline_filter = match self.pipeline_filter {
                                PipelineFilter::All => PipelineFilter::Running,
                                PipelineFilter::Running => PipelineFilter::Failed,
                                PipelineFilter::Failed => PipelineFilter::Passed,
                                PipelineFilter::Passed => PipelineFilter::All,
                            };
                            self.filter_lists();
                        }
                        _ => {}
                    }
                    if self.active_selection != MenuSelection::Pipelines {
                        self.merge_requests.clear();
                        self.issues.clear();
                        self.reset_list_paging();
                        self.todos.clear();
                        let _ = self.refresh().await;
                    }
                }
            }
            KeyCode::Char('c') => {
                if self.selected_issue.is_some() {
                    let _ = self.toggle_issue_state().await;
                } else if self.selected_mr.is_some() {
                    let _ = self.toggle_mr_state().await;
                }
            }
            KeyCode::Char('I') => {
                self.execute_help_action(HelpAction::ToggleInvolvedOnly)
                    .await;
            }
            KeyCode::Char('V') => {
                if self.active_selection == MenuSelection::Issues {
                    self.issue_group_by_milestone = !self.issue_group_by_milestone;
                    self.status_message = Some(if self.issue_group_by_milestone {
                        "Grouping issues by milestone".into()
                    } else {
                        "Grouping issues by project".into()
                    });
                    self.filter_lists();
                }
            }
            KeyCode::Char('A') => {
                if self.selected_mr.is_some() {
                    let _ = self.approve_mr().await;
                }
            }
            KeyCode::Char('M') => {
                self.open_confirm_merge_popup();
            }
            KeyCode::Char('C') => {
                if self.active_panel == Panel::MainDetail
                    && self.detail_section == DetailSection::Changes
                    && self.selected_mr.is_some()
                    && self.mr_diff_state.selected().is_some()
                {
                    self.input_mode = true;
                    self.input_target = InputTarget::NewFileComment;
                    self.textarea = ratatui_textarea::TextArea::default();
                } else if let Some(p) = (self.active_selection == MenuSelection::Pipelines)
                    .then(|| {
                        self.pipelines_state
                            .selected()
                            .and_then(|i| self.filtered_pipelines.get(i))
                    })
                    .flatten()
                {
                    let _ = self.cancel_pipeline_sync(p.project_id, p.id).await;
                }
            }
            KeyCode::Char('R') => {
                if let Some(p) = (self.active_selection == MenuSelection::Pipelines)
                    .then(|| {
                        self.pipelines_state
                            .selected()
                            .and_then(|i| self.filtered_pipelines.get(i))
                    })
                    .flatten()
                {
                    let _ = self.retry_pipeline_sync(p.project_id, p.id).await;
                } else if self.selected_mr.is_some() {
                    let _ = self.unapprove_mr().await;
                }
            }
            KeyCode::Char('D') => {
                if self.active_selection == MenuSelection::Branches
                    && !self.project_branches.is_empty()
                {
                    if let (Some(p), Some(branch)) = (
                        &self.selected_project,
                        self.branches_state
                            .selected()
                            .and_then(|i| self.project_branches.get(i)),
                    ) {
                        self.pending_delete_branch = Some((p.id, branch.name.clone()));
                        self.show_confirm_delete_branch_popup = true;
                    } else {
                        self.status_message = Some("No branch selected".into());
                    }
                } else if self.selected_mr.is_some() {
                    let _ = self.toggle_mr_draft().await;
                }
            }
            KeyCode::Char('E') => {
                if self.selected_mr.is_some() {
                    self.show_confirm_rebase_popup = true;
                }
            }
            KeyCode::Char('S') => {
                if self.active_selection == MenuSelection::Pipelines
                    && self.active_context == Context::Project
                {
                    let _ = self.fetch_pipeline_schedules().await;
                    self.show_schedules_popup = true;
                } else if self.selected_mr.is_some() {
                    let _ = self.toggle_mr_squash().await;
                }
            }
            KeyCode::Char('s') => {
                if self.active_selection == MenuSelection::Todos {
                    self.show_snooze_popup = true;
                    self.snooze_state.select(Some(0));
                } else {
                    if self.selected_mr.is_some() {
                        if let Some(disc) = self
                            .discussion_state
                            .selected()
                            .and_then(|idx| self.discussions.get_mut(idx))
                        {
                            disc.collapsed = true;
                        }
                        let _ = self.resolve_discussion().await;
                    } else if self.selected_issue.is_some() {
                        if let Some(disc) = self
                            .discussion_state
                            .selected()
                            .and_then(|idx| self.discussions.get_mut(idx))
                        {
                            disc.collapsed = true;
                        }
                        let _ = self.resolve_issue_discussion().await;
                    }
                }
            }
            KeyCode::Char('d') => {
                if self.active_selection == MenuSelection::Todos && self.active_panel == Panel::Main
                {
                    let _ = self.mark_todo_as_done().await;
                } else if self.active_panel == Panel::MainDetail {
                    self.detail_section = DetailSection::Description;
                }
            }
            KeyCode::Char('m') => {
                if self.active_panel == Panel::MainDetail {
                    self.detail_section = DetailSection::Metadata;
                }
            }
            KeyCode::Char('e') => {
                if self.active_panel == Panel::MainDetail {
                    match self.detail_section {
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
                        DetailSection::Metadata => {
                            self.enter_detail_action().await;
                        }
                        DetailSection::Changes => {
                            if let Some((mr_diff, mr)) = self
                                .mr_diff_state
                                .selected()
                                .and_then(|idx| self.mr_diffs.get(idx))
                                .filter(|d| d.collapsed)
                                .cloned()
                                .zip(self.selected_mr.as_ref())
                            {
                                let _ = self
                                    .fetch_single_file_diff(
                                        mr.project_id,
                                        mr.iid,
                                        mr_diff.old_path,
                                        mr_diff.new_path,
                                    )
                                    .await;
                            }
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Char('n') => {
                if self.show_create_issue_popup
                    || self.show_create_mr_popup
                    || self.show_pipeline_popup
                    || self.show_project_popup
                    || self.show_help_popup
                    || self.show_search
                    || self.show_confirm_merge_popup
                    || self.show_confirm_rebase_popup
                {
                    return;
                }
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
                            self.create_mr_target_branch = p
                                .default_branch
                                .clone()
                                .unwrap_or_else(|| "main".to_string());
                        }
                    }
                    MenuSelection::Pipelines => {
                        self.show_pipeline_popup = true;
                        self.input_mode = true;
                        self.input_target = InputTarget::PipelineRef;
                        if let Some(p) = &self.selected_project {
                            self.pipeline_ref_input = self
                                .selected_pipeline
                                .as_ref()
                                .map(|pipe| pipe.ref_name.clone())
                                .unwrap_or_else(|| {
                                    p.default_branch
                                        .clone()
                                        .unwrap_or_else(|| "main".to_string())
                                });
                            let pid = p.id;
                            if self.project_variable_cache.contains_key(&pid) {
                                self.project_variables =
                                    self.project_variable_cache.get(&pid).cloned().unwrap();
                            } else {
                                let _ = self.fetch_project_variables(pid).await;
                            }
                        }
                    }
                    MenuSelection::Branches => {
                        self.show_create_branch_popup = true;
                        self.input_mode = true;
                        self.input_target = InputTarget::CreateBranchName;
                        self.create_branch_name.clear();
                        self.create_branch_ref_input = self
                            .selected_pipeline
                            .as_ref()
                            .map(|p| p.ref_name.clone())
                            .or_else(|| {
                                self.selected_project
                                    .as_ref()
                                    .and_then(|p| p.default_branch.clone())
                            })
                            .unwrap_or_else(|| "main".to_string());
                        self.show_create_branch_ref_dropdown = false;
                        self.textarea =
                            ratatui_textarea::TextArea::from(self.create_branch_name.lines());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        let _ = self.update_preview().await;
    }

    pub async fn execute_home_card_action(&mut self) {
        match self.home_card_index {
            0 => {
                self.active_context = Context::Dashboard;
                self.active_selection = MenuSelection::MergeRequests;
                self.dashboard_menu_state.select(Some(2));
                let _ = self.refresh().await;
            }
            1 => {
                self.active_context = Context::Dashboard;
                self.active_selection = MenuSelection::Todos;
                self.dashboard_menu_state.select(Some(1));
                let _ = self.refresh().await;
            }
            2 => {
                self.active_context = Context::Dashboard;
                self.active_selection = MenuSelection::Issues;
                self.dashboard_menu_state.select(Some(3));
                let _ = self.refresh().await;
            }
            3 => {
                self.active_selection = MenuSelection::Boards;
                self.view_mode = ViewMode::Board;
                let _ = self.refresh().await;
            }
            4 => {
                self.active_context = Context::Dashboard;
                self.active_selection = MenuSelection::Pipelines;
                self.dashboard_menu_state.select(Some(4));
                let _ = self.refresh().await;
            }
            5 => {
                self.active_context = Context::Project;
                self.active_selection = MenuSelection::Repository;
                let _ = self.refresh().await;
            }
            6 => {
                self.show_project_popup = true;
            }
            7 => {
                self.show_help_popup = true;
            }
            _ => {}
        }
    }
}
