//! Popup keyboard handling. Extracted from `App::handle_key` so the
//! central keymap stays small. State stays on `App`; this file just
//! houses the keyboard branches that fire when a popup is open.

use chrono::Timelike;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::models::*;

impl App {
    /// Returns true if the key was handled by a popup branch.
    pub(crate) async fn handle_popup_key(&mut self, processed_key: KeyEvent) -> bool {
        // 1.1 Help Popup
        if self.show_help_popup {
            if self.show_help_filter {
                match processed_key.code {
                    KeyCode::Esc | KeyCode::Char('/') => {
                        self.show_help_filter = false;
                    }
                    KeyCode::Enter => {
                        if let Some(item) = self
                            .help_state
                            .selected()
                            .and_then(|i| self.filtered_help.get(i))
                            .cloned()
                        {
                            self.execute_help_action(item.action).await;
                        }
                    }
                    KeyCode::Down => {
                        if !self.filtered_help.is_empty() {
                            let i = (self.help_state.selected().unwrap_or(0) + 1)
                                % self.filtered_help.len();
                            self.help_state.select(Some(i));
                        }
                    }
                    KeyCode::Up => {
                        if !self.filtered_help.is_empty() {
                            let i = if self.help_state.selected().unwrap_or(0) == 0 {
                                self.filtered_help.len() - 1
                            } else {
                                self.help_state.selected().unwrap() - 1
                            };
                            self.help_state.select(Some(i));
                        }
                    }
                    KeyCode::Backspace => {
                        self.help_filter_input.pop();
                        self.filter_help();
                    }
                    KeyCode::Char(c) if c != '/' => {
                        self.help_filter_input.push(c);
                        self.filter_help();
                    }
                    _ => {}
                }
                return true;
            }
            if processed_key.code == KeyCode::Esc || processed_key.code == KeyCode::Char('?') {
                self.show_help_popup = false;
                return true;
            }
            match processed_key.code {
                KeyCode::Char('/') => {
                    self.show_help_filter = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.filtered_help.is_empty() {
                        let i = (self.help_state.selected().unwrap_or(0) + 1)
                            % self.filtered_help.len();
                        self.help_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.filtered_help.is_empty() {
                        let i = if self.help_state.selected().unwrap_or(0) == 0 {
                            self.filtered_help.len() - 1
                        } else {
                            self.help_state.selected().unwrap() - 1
                        };
                        self.help_state.select(Some(i));
                    }
                }
                KeyCode::Enter => {
                    if let Some(item) = self
                        .help_state
                        .selected()
                        .and_then(|i| self.filtered_help.get(i))
                        .cloned()
                    {
                        self.execute_help_action(item.action).await;
                    }
                }
                _ => {}
            }
            return true;
        }

        // 1.2 Calendar Popup (Snooze Date Selection)
        if self.show_calendar_popup {
            use chrono::Duration;
            match processed_key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.calendar_date += Duration::weeks(1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.calendar_date -= Duration::weeks(1);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.calendar_date -= Duration::days(1);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.calendar_date += Duration::days(1);
                }
                KeyCode::Enter => {
                    match self.input_target {
                        InputTarget::SnoozeUntil => {
                            let snooze_until =
                                format!("{}T09:00:00Z", self.calendar_date.format("%Y-%m-%d"));
                            let _ = self.snooze_todo(snooze_until).await;
                        }
                        InputTarget::EditIssueStartDate => {
                            let date_str = self.calendar_date.format("%Y-%m-%d").to_string();
                            let _ = self.update_issue_start_date(Some(date_str)).await;
                        }
                        InputTarget::EditIssueDueDate => {
                            let date_str = self.calendar_date.format("%Y-%m-%d").to_string();
                            let _ = self.update_issue_due_date(Some(date_str)).await;
                        }
                        _ => {}
                    }
                    self.show_calendar_popup = false;
                }
                KeyCode::Char('c') | KeyCode::Char('x') | KeyCode::Backspace | KeyCode::Delete => {
                    match self.input_target {
                        InputTarget::EditIssueStartDate => {
                            let _ = self.update_issue_start_date(None).await;
                        }
                        InputTarget::EditIssueDueDate => {
                            let _ = self.update_issue_due_date(None).await;
                        }
                        _ => {}
                    }
                    self.show_calendar_popup = false;
                }
                KeyCode::Esc => {
                    self.show_calendar_popup = false;
                }
                _ => {}
            }
            let _ = self.update_preview().await;
            return true;
        }

        // 1.3 Snooze Presets Popup
        if self.show_snooze_popup {
            match processed_key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    let i = (self.snooze_state.selected().unwrap_or(0) + 1) % 5;
                    self.snooze_state.select(Some(i));
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let i = if self.snooze_state.selected().unwrap_or(0) == 0 {
                        4
                    } else {
                        self.snooze_state.selected().unwrap() - 1
                    };
                    self.snooze_state.select(Some(i));
                }
                KeyCode::Enter => {
                    let selected = self.snooze_state.selected().unwrap_or(0);
                    let snooze_until = match selected {
                        0 => (chrono::Local::now() + chrono::Duration::hours(1)).to_rfc3339(),
                        1 => (chrono::Local::now() + chrono::Duration::days(1))
                            .with_hour(9)
                            .unwrap()
                            .with_minute(0)
                            .unwrap()
                            .with_second(0)
                            .unwrap()
                            .to_rfc3339(),
                        2 => (chrono::Local::now() + chrono::Duration::days(3))
                            .with_hour(9)
                            .unwrap()
                            .with_minute(0)
                            .unwrap()
                            .with_second(0)
                            .unwrap()
                            .to_rfc3339(),
                        3 => (chrono::Local::now() + chrono::Duration::weeks(1))
                            .with_hour(9)
                            .unwrap()
                            .with_minute(0)
                            .unwrap()
                            .with_second(0)
                            .unwrap()
                            .to_rfc3339(),
                        4 => {
                            self.show_snooze_popup = false;
                            self.show_calendar_popup = true;
                            self.input_target = InputTarget::SnoozeUntil;
                            return true;
                        }
                        _ => (chrono::Local::now() + chrono::Duration::hours(1)).to_rfc3339(),
                    };
                    let _ = self.snooze_todo(snooze_until).await;
                    self.show_snooze_popup = false;
                }
                KeyCode::Esc => {
                    self.show_snooze_popup = false;
                }
                _ => {}
            }
            let _ = self.update_preview().await;
            return true;
        }

        // 1.4 Global Search Popup
        if self.show_search {
            if self.input_target == InputTarget::Search && self.input_mode {
                match processed_key.code {
                    KeyCode::Esc => {
                        self.input_mode = false;
                        self.show_search = false;
                        self.search_query.clear();
                        self.search_results.clear();
                    }
                    KeyCode::Enter => {
                        self.input_mode = false;
                    }
                    KeyCode::Char(c) => {
                        self.search_query.push(c);
                        let _ = self.trigger_global_search().await;
                    }
                    KeyCode::Backspace => {
                        self.search_query.pop();
                        let _ = self.trigger_global_search().await;
                    }
                    _ => {}
                }
                return true;
            }
            match processed_key.code {
                KeyCode::Esc => {
                    self.show_search = false;
                    self.search_results.clear();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.search_results.is_empty() {
                        let i = (self.search_state.selected().unwrap_or(0) + 1)
                            % self.search_results.len();
                        self.search_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.search_results.is_empty() {
                        let i = if self.search_state.selected().unwrap_or(0) == 0 {
                            self.search_results.len() - 1
                        } else {
                            self.search_state.selected().unwrap() - 1
                        };
                        self.search_state.select(Some(i));
                    }
                }
                KeyCode::Enter => {
                    if let Some(res) = self
                        .search_state
                        .selected()
                        .and_then(|idx| self.search_results.get(idx))
                        .cloned()
                    {
                        match res {
                            SearchResult::Project(p) => {
                                self.selected_project = Some(p);
                                self.active_context = Context::Project;
                                self.active_panel = Panel::Project;
                                self.project_menu_state.select(Some(0));
                                self.active_selection = MenuSelection::MergeRequests;
                                self.show_search = false;
                                let _ = self.refresh().await;
                            }
                            SearchResult::Issue(i) => {
                                self.selected_issue = Some(i);
                                self.selected_mr = None;
                                self.discussions.clear();
                                self.mr_diffs.clear();
                                self.is_fetching_diffs = false;
                                self.view_mode = ViewMode::Detail;
                                self.show_search = false;
                            }
                            SearchResult::MergeRequest(m) => {
                                self.selected_mr = Some(m);
                                self.selected_issue = None;
                                self.discussions.clear();
                                self.mr_diffs.clear();
                                self.is_fetching_diffs = false;
                                self.view_mode = ViewMode::Detail;
                                self.show_search = false;
                            }
                        }
                    }
                }
                KeyCode::Char('/') => {
                    self.input_mode = true;
                    self.input_target = InputTarget::Search;
                }
                _ => {}
            }
            let _ = self.update_preview().await;
            return true;
        }

        // 1.5 Selection Popups (Simple List/Selection Popups)
        if self.show_schedules_popup {
            match processed_key.code {
                KeyCode::Esc => {
                    self.show_schedules_popup = false;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.pipeline_schedules.is_empty() {
                        let i = (self.schedules_state.selected().unwrap_or(0) + 1)
                            % self.pipeline_schedules.len();
                        self.schedules_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.pipeline_schedules.is_empty() {
                        let i = if self.schedules_state.selected().unwrap_or(0) == 0 {
                            self.pipeline_schedules.len() - 1
                        } else {
                            self.schedules_state.selected().unwrap() - 1
                        };
                        self.schedules_state.select(Some(i));
                    }
                }
                KeyCode::Enter => {
                    if let Some(s) = self
                        .schedules_state
                        .selected()
                        .and_then(|i| self.pipeline_schedules.get(i))
                    {
                        let id = s.id;
                        self.show_schedules_popup = false;
                        let _ = self.run_pipeline_schedule(id).await;
                    }
                }
                _ => {}
            }
            return true;
        }

        if self.show_project_popup
            || self.show_label_popup
            || self.show_assignee_popup
            || self.show_reviewer_popup
            || self.show_milestone_popup
            || self.show_target_branch_popup
            || self.show_tree_branch_popup
        {
            match processed_key.code {
                KeyCode::Esc => {
                    self.show_project_popup = false;
                    self.show_label_popup = false;
                    self.show_assignee_popup = false;
                    self.show_reviewer_popup = false;
                    self.show_milestone_popup = false;
                    self.show_target_branch_popup = false;
                    self.show_create_branch_popup = false;
                    self.show_tree_branch_popup = false;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.show_project_popup {
                        self.move_down();
                    } else if self.show_label_popup {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        let labels = self
                            .project_label_cache
                            .get(&pid)
                            .unwrap_or(&self.project_labels);
                        if !labels.is_empty() {
                            let i = (self.label_state.selected().unwrap_or(0) + 1) % labels.len();
                            self.label_state.select(Some(i));
                        }
                    } else if self.show_assignee_popup {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        let members = self
                            .project_member_cache
                            .get(&pid)
                            .unwrap_or(&self.project_members);
                        if !members.is_empty() {
                            let i =
                                (self.assignee_state.selected().unwrap_or(0) + 1) % members.len();
                            self.assignee_state.select(Some(i));
                        }
                    } else if self.show_reviewer_popup {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        let members = self
                            .project_member_cache
                            .get(&pid)
                            .unwrap_or(&self.project_members);
                        if !members.is_empty() {
                            let i =
                                (self.reviewer_state.selected().unwrap_or(0) + 1) % members.len();
                            self.reviewer_state.select(Some(i));
                        }
                    } else if self.show_milestone_popup {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        let milestones = self
                            .project_milestone_cache
                            .get(&pid)
                            .unwrap_or(&self.project_milestones);
                        let i = (self.milestone_state.selected().unwrap_or(0) + 1)
                            % (milestones.len() + 1);
                        self.milestone_state.select(Some(i));
                    } else if self.show_target_branch_popup && !self.project_branches.is_empty() {
                        let i = (self.target_branch_state.selected().unwrap_or(0) + 1)
                            % self.project_branches.len();
                        self.target_branch_state.select(Some(i));
                    } else if self.show_tree_branch_popup && !self.project_branches.is_empty() {
                        let i = (self.tree_branch_state.selected().unwrap_or(0) + 1)
                            % self.project_branches.len();
                        self.tree_branch_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.show_project_popup {
                        self.move_up();
                    } else if self.show_label_popup {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        let labels = self
                            .project_label_cache
                            .get(&pid)
                            .unwrap_or(&self.project_labels);
                        if !labels.is_empty() {
                            let i = if self.label_state.selected().unwrap_or(0) == 0 {
                                labels.len() - 1
                            } else {
                                self.label_state.selected().unwrap() - 1
                            };
                            self.label_state.select(Some(i));
                        }
                    } else if self.show_assignee_popup {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        let members = self
                            .project_member_cache
                            .get(&pid)
                            .unwrap_or(&self.project_members);
                        if !members.is_empty() {
                            let i = if self.assignee_state.selected().unwrap_or(0) == 0 {
                                members.len() - 1
                            } else {
                                self.assignee_state.selected().unwrap() - 1
                            };
                            self.assignee_state.select(Some(i));
                        }
                    } else if self.show_reviewer_popup {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        let members = self
                            .project_member_cache
                            .get(&pid)
                            .unwrap_or(&self.project_members);
                        if !members.is_empty() {
                            let i = if self.reviewer_state.selected().unwrap_or(0) == 0 {
                                members.len() - 1
                            } else {
                                self.reviewer_state.selected().unwrap() - 1
                            };
                            self.reviewer_state.select(Some(i));
                        }
                    } else if self.show_milestone_popup {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        let milestones = self
                            .project_milestone_cache
                            .get(&pid)
                            .unwrap_or(&self.project_milestones);
                        let i = if self.milestone_state.selected().unwrap_or(0) == 0 {
                            milestones.len()
                        } else {
                            self.milestone_state.selected().unwrap() - 1
                        };
                        self.milestone_state.select(Some(i));
                    } else if self.show_target_branch_popup && !self.project_branches.is_empty() {
                        let i = if self.target_branch_state.selected().unwrap_or(0) == 0 {
                            self.project_branches.len() - 1
                        } else {
                            self.target_branch_state.selected().unwrap() - 1
                        };
                        self.target_branch_state.select(Some(i));
                    } else if self.show_tree_branch_popup && !self.project_branches.is_empty() {
                        let i = if self.tree_branch_state.selected().unwrap_or(0) == 0 {
                            self.project_branches.len() - 1
                        } else {
                            self.tree_branch_state.selected().unwrap() - 1
                        };
                        self.tree_branch_state.select(Some(i));
                    }
                }
                KeyCode::Char(' ') => {
                    if self.show_label_popup {
                        self.toggle_popup_label();
                    } else if self.show_assignee_popup {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        let members = self
                            .project_member_cache
                            .get(&pid)
                            .unwrap_or(&self.project_members);
                        if let Some(member) =
                            self.assignee_state.selected().and_then(|i| members.get(i))
                        {
                            let id = member.id;
                            if self.selected_assignee_ids.contains(&id) {
                                self.selected_assignee_ids.retain(|&x| x != id);
                            } else {
                                self.selected_assignee_ids.push(id);
                            }
                        }
                    } else if self.show_reviewer_popup {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        let members = self
                            .project_member_cache
                            .get(&pid)
                            .unwrap_or(&self.project_members);
                        if let Some(member) =
                            self.reviewer_state.selected().and_then(|i| members.get(i))
                        {
                            let id = member.id;
                            if self.selected_reviewer_ids.contains(&id) {
                                self.selected_reviewer_ids.retain(|&x| x != id);
                            } else {
                                self.selected_reviewer_ids.push(id);
                            }
                        }
                    }
                }
                KeyCode::Enter => {
                    if self.show_project_popup {
                        let _ = self.select_project().await;
                    } else if self.show_label_popup {
                        if self.selected_issue.is_some() {
                            let _ = self.update_issue_labels().await;
                        } else if self.selected_mr.is_some() {
                            let _ = self.update_mr_labels().await;
                        }
                    } else if self.show_assignee_popup {
                        if self.selected_issue.is_some() {
                            let _ = self.update_issue_assignees().await;
                        } else if self.selected_mr.is_some() {
                            let _ = self.update_mr_assignees().await;
                        }
                        self.selected_assignee_ids.clear();
                        self.show_assignee_popup = false;
                    } else if self.show_reviewer_popup {
                        if self.selected_mr.is_some() {
                            let _ = self.update_mr_reviewers().await;
                        }
                    } else if self.show_milestone_popup {
                        let pid = self
                            .selected_mr
                            .as_ref()
                            .map(|m| m.project_id)
                            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
                            .unwrap_or(0);
                        let milestones = self
                            .project_milestone_cache
                            .get(&pid)
                            .unwrap_or(&self.project_milestones);
                        let selected = self.milestone_state.selected().unwrap_or(0);
                        self.selected_milestone_id = if selected == 0 {
                            None
                        } else {
                            milestones.get(selected - 1).map(|m| m.id)
                        };
                        if self.selected_issue.is_some() {
                            let _ = self.update_issue_milestone().await;
                        } else if self.selected_mr.is_some() {
                            let _ = self.update_mr_milestone().await;
                        }
                    } else if self.show_target_branch_popup {
                        let _ = self.update_mr_target_branch().await;
                    } else if self.show_tree_branch_popup {
                        if let Some(branch) = self
                            .tree_branch_state
                            .selected()
                            .and_then(|i| self.project_branches.get(i))
                        {
                            let name = branch.name.clone();
                            self.show_tree_branch_popup = false;
                            self.view_mode = ViewMode::List;
                            let _ = self.select_repository_ref(name).await;
                        } else {
                            self.show_tree_branch_popup = false;
                        }
                    }
                }
                _ => {}
            }
            let _ = self.update_preview().await;
            return true;
        }

        // 1.55 CI Variables editor
        if self.show_ci_vars_popup {
            if self.ci_var_editing.is_some() {
                match processed_key.code {
                    KeyCode::Esc => {
                        self.ci_var_editing = None;
                        self.input_mode = false;
                        self.pipeline_variable_key_input.clear();
                        self.pipeline_variable_value_input.clear();
                    }
                    KeyCode::Tab => {
                        if self.input_target == InputTarget::PipelineVariableKey {
                            self.pipeline_variable_key_input = self.textarea.lines().join("\n");
                            self.input_target = InputTarget::PipelineVariableValue;
                            self.textarea = ratatui_textarea::TextArea::from(
                                self.pipeline_variable_value_input.lines(),
                            );
                        } else {
                            self.pipeline_variable_value_input = self.textarea.lines().join("\n");
                            self.input_target = InputTarget::PipelineVariableKey;
                            self.textarea = ratatui_textarea::TextArea::from(
                                self.pipeline_variable_key_input.lines(),
                            );
                        }
                    }
                    KeyCode::Enter
                        if processed_key.modifiers.contains(KeyModifiers::CONTROL)
                            || processed_key.modifiers.contains(KeyModifiers::SHIFT) =>
                    {
                        if self.input_target == InputTarget::PipelineVariableKey {
                            self.pipeline_variable_key_input = self.textarea.lines().join("\n");
                        } else if self.input_target == InputTarget::PipelineVariableValue {
                            self.pipeline_variable_value_input = self.textarea.lines().join("\n");
                        }
                        let key_name = self.pipeline_variable_key_input.trim().to_string();
                        let value = self.pipeline_variable_value_input.clone();
                        let update_existing = self
                            .ci_var_editing
                            .as_deref()
                            .map(|k| !k.is_empty())
                            .unwrap_or(false);
                        self.ci_var_editing = None;
                        self.input_mode = false;
                        self.pipeline_variable_key_input.clear();
                        self.pipeline_variable_value_input.clear();
                        if key_name.is_empty() {
                            self.status_message = Some("Variable key must not be empty".into());
                        } else {
                            let _ = self
                                .upsert_project_variable(key_name, value, update_existing)
                                .await;
                        }
                    }
                    _ => {
                        self.textarea.input(processed_key);
                        if self.input_target == InputTarget::PipelineVariableKey {
                            self.pipeline_variable_key_input = self.textarea.lines().join("\n");
                        } else if self.input_target == InputTarget::PipelineVariableValue {
                            self.pipeline_variable_value_input = self.textarea.lines().join("\n");
                        }
                    }
                }
                return true;
            }
            match processed_key.code {
                KeyCode::Esc => {
                    self.show_ci_vars_popup = false;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.project_variables.is_empty() {
                        let i = (self.ci_vars_state.selected().unwrap_or(0) + 1)
                            % self.project_variables.len();
                        self.ci_vars_state.select(Some(i));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.project_variables.is_empty() {
                        let i = if self.ci_vars_state.selected().unwrap_or(0) == 0 {
                            self.project_variables.len() - 1
                        } else {
                            self.ci_vars_state.selected().unwrap() - 1
                        };
                        self.ci_vars_state.select(Some(i));
                    }
                }
                KeyCode::Enter => {
                    if let Some(v) = self
                        .ci_vars_state
                        .selected()
                        .and_then(|i| self.project_variables.get(i))
                    {
                        let key_name = v.key.clone();
                        self.pipeline_variable_key_input = key_name.clone();
                        self.pipeline_variable_value_input = v.value.clone();
                        self.ci_var_editing = Some(key_name.clone());
                        self.input_mode = true;
                        self.input_target = InputTarget::PipelineVariableValue;
                        self.textarea = ratatui_textarea::TextArea::from(
                            self.pipeline_variable_value_input.lines(),
                        );
                    }
                }
                KeyCode::Char('d') => {
                    if let Some(v) = self
                        .ci_vars_state
                        .selected()
                        .and_then(|i| self.project_variables.get(i))
                    {
                        let key_name = v.key.clone();
                        let _ = self.delete_project_variable(key_name).await;
                    }
                }
                KeyCode::Char('n') => {
                    self.ci_var_editing = Some(String::new());
                    self.input_mode = true;
                    self.input_target = InputTarget::PipelineVariableKey;
                    self.pipeline_variable_key_input.clear();
                    self.pipeline_variable_value_input.clear();
                    self.textarea = ratatui_textarea::TextArea::default();
                }
                _ => {}
            }
            return true;
        }

        // 1.6 Confirmation Popups
        if self.show_confirm_merge_popup
            || self.show_confirm_rebase_popup
            || self.show_confirm_delete_branch_popup
        {
            match processed_key.code {
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.show_confirm_merge_popup = false;
                    self.show_confirm_rebase_popup = false;
                    self.show_confirm_delete_branch_popup = false;
                    self.pending_delete_branch = None;
                }
                KeyCode::Char('s') | KeyCode::Char('S') if self.show_confirm_merge_popup => {
                    self.merge_squash = !self.merge_squash;
                }
                KeyCode::Char('d') | KeyCode::Char('D') if self.show_confirm_merge_popup => {
                    self.merge_remove_source_branch = !self.merge_remove_source_branch;
                }
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if self.show_confirm_merge_popup {
                        self.show_confirm_merge_popup = false;
                        let _ = self.merge_mr().await;
                    } else if self.show_confirm_rebase_popup {
                        self.show_confirm_rebase_popup = false;
                        let _ = self.rebase_mr().await;
                    } else if self.show_confirm_delete_branch_popup {
                        self.show_confirm_delete_branch_popup = false;
                        if let Some((pid, name)) = self.pending_delete_branch.take() {
                            let _ = self.delete_branch(pid, name).await;
                        }
                    }
                }
                _ => {}
            }
            let _ = self.update_preview().await;
            return true;
        }

        // 1.7 Creation Forms (Create Issue/MR)
        if self.show_create_issue_popup
            || self.show_create_mr_popup
            || self.show_create_branch_popup
        {
            match processed_key.code {
                KeyCode::Esc => {
                    self.show_create_issue_popup = false;
                    self.show_create_mr_popup = false;
                    self.show_create_branch_popup = false;
                    self.input_mode = false;
                }
                KeyCode::Tab => {
                    if self.show_create_issue_popup {
                        if self.input_target == InputTarget::CreateIssueTitle {
                            self.create_issue_title = self.textarea.lines().join("\n");
                            self.input_target = InputTarget::CreateIssueDescription;
                            self.textarea = ratatui_textarea::TextArea::from(
                                self.create_issue_description.lines(),
                            );
                        } else {
                            self.create_issue_description = self.textarea.lines().join("\n");
                            self.input_target = InputTarget::CreateIssueTitle;
                            self.textarea =
                                ratatui_textarea::TextArea::from(self.create_issue_title.lines());
                        }
                    } else if self.show_create_mr_popup {
                        match self.input_target {
                            InputTarget::CreateMRTitle => {
                                self.create_mr_title = self.textarea.lines().join("\n");
                                self.input_target = InputTarget::CreateMRSourceBranch;
                                self.textarea = ratatui_textarea::TextArea::from(
                                    self.create_mr_source_branch.lines(),
                                );
                            }
                            InputTarget::CreateMRSourceBranch => {
                                self.create_mr_source_branch = self.textarea.lines().join("\n");
                                self.input_target = InputTarget::CreateMRTargetBranch;
                                self.textarea = ratatui_textarea::TextArea::from(
                                    self.create_mr_target_branch.lines(),
                                );
                            }
                            InputTarget::CreateMRTargetBranch => {
                                self.create_mr_target_branch = self.textarea.lines().join("\n");
                                self.input_target = InputTarget::CreateMRDescription;
                                self.textarea = ratatui_textarea::TextArea::from(
                                    self.create_mr_description.lines(),
                                );
                            }
                            InputTarget::CreateMRDescription => {
                                self.create_mr_description = self.textarea.lines().join("\n");
                                self.input_target = InputTarget::CreateMRTitle;
                                self.textarea =
                                    ratatui_textarea::TextArea::from(self.create_mr_title.lines());
                            }
                            _ => {}
                        }
                    } else if self.show_create_branch_popup {
                        if self.input_target == InputTarget::CreateBranchName {
                            self.create_branch_name = self.textarea.lines().join("\n");
                            self.input_target = InputTarget::CreateBranchRef;
                            self.textarea = ratatui_textarea::TextArea::from(
                                self.create_branch_ref_input.lines(),
                            );
                            self.show_create_branch_ref_dropdown =
                                !self.project_branches.is_empty();
                            if self.show_create_branch_ref_dropdown {
                                self.create_branch_state.select(Some(0));
                            }
                        } else {
                            self.create_branch_ref_input = self.textarea.lines().join("\n");
                            self.input_target = InputTarget::CreateBranchName;
                            self.textarea =
                                ratatui_textarea::TextArea::from(self.create_branch_name.lines());
                            self.show_create_branch_ref_dropdown = false;
                        }
                    }
                    self.textarea.move_cursor(ratatui_textarea::CursorMove::End);
                }
                KeyCode::Enter if processed_key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.show_create_issue_popup {
                        if self.input_target == InputTarget::CreateIssueTitle {
                            self.create_issue_title = self.textarea.lines().join("\n");
                        } else {
                            self.create_issue_description = self.textarea.lines().join("\n");
                        }
                        let _ = self.create_issue().await;
                    } else if self.show_create_mr_popup {
                        match self.input_target {
                            InputTarget::CreateMRTitle => {
                                self.create_mr_title = self.textarea.lines().join("\n")
                            }
                            InputTarget::CreateMRSourceBranch => {
                                self.create_mr_source_branch = self.textarea.lines().join("\n")
                            }
                            InputTarget::CreateMRTargetBranch => {
                                self.create_mr_target_branch = self.textarea.lines().join("\n")
                            }
                            InputTarget::CreateMRDescription => {
                                self.create_mr_description = self.textarea.lines().join("\n")
                            }
                            _ => {}
                        }
                        let _ = self.create_mr().await;
                    } else if self.show_create_branch_popup {
                        if self.input_target == InputTarget::CreateBranchName {
                            self.create_branch_name = self.textarea.lines().join("\n");
                        } else if self.input_target == InputTarget::CreateBranchRef {
                            self.create_branch_ref_input = self.textarea.lines().join("\n");
                        }
                        if let (Some(p), name, ref_name) = (
                            &self.selected_project,
                            self.create_branch_name.trim().to_string(),
                            self.create_branch_ref_input.trim().to_string(),
                        ) {
                            if name.is_empty() {
                                self.status_message = Some("Branch name cannot be empty".into());
                            } else if ref_name.is_empty() {
                                self.status_message = Some("Reference cannot be empty".into());
                            } else {
                                let pid = p.id;
                                self.show_create_branch_popup = false;
                                self.input_mode = false;
                                let _ = self.create_branch(pid, name, ref_name).await;
                                return true;
                            }
                        } else {
                            self.status_message = Some("No project selected".into());
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j')
                    if self.show_create_branch_popup
                        && self.input_target == InputTarget::CreateBranchRef
                        && self.show_create_branch_ref_dropdown
                        && !self.project_branches.is_empty() =>
                {
                    let i = (self.create_branch_state.selected().unwrap_or(0) + 1)
                        % self.project_branches.len();
                    self.create_branch_state.select(Some(i));
                    if let Some(b) = self.project_branches.get(i) {
                        self.create_branch_ref_input = b.name.clone();
                    }
                }
                KeyCode::Up | KeyCode::Char('k')
                    if self.show_create_branch_popup
                        && self.input_target == InputTarget::CreateBranchRef
                        && self.show_create_branch_ref_dropdown
                        && !self.project_branches.is_empty() =>
                {
                    let i = if self.create_branch_state.selected().unwrap_or(0) == 0 {
                        self.project_branches.len() - 1
                    } else {
                        self.create_branch_state.selected().unwrap() - 1
                    };
                    self.create_branch_state.select(Some(i));
                    if let Some(b) = self.project_branches.get(i) {
                        self.create_branch_ref_input = b.name.clone();
                    }
                }
                _ => {
                    self.textarea.input(processed_key);
                }
            }
            let _ = self.update_preview().await;
            return true;
        }

        // 1.8 Pipeline Popup
        if self.show_pipeline_popup {
            match processed_key.code {
                KeyCode::Tab => match self.input_target {
                    InputTarget::PipelineRef => {
                        self.input_target = InputTarget::PipelineVariableKey
                    }
                    InputTarget::PipelineVariableKey => {
                        self.input_target = InputTarget::PipelineVariableValue
                    }
                    InputTarget::PipelineVariableValue => {
                        self.input_target = InputTarget::PipelineVariableList
                    }
                    InputTarget::PipelineVariableList => {
                        self.input_target = InputTarget::PipelineRef
                    }
                    _ => self.input_target = InputTarget::PipelineRef,
                },
                KeyCode::Char('r') if processed_key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let _ = self.submit_pipeline_launch().await;
                }
                KeyCode::Enter if processed_key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let _ = self.submit_pipeline_launch().await;
                }
                KeyCode::Char('p')
                    if self.input_target == InputTarget::PipelineVariableList
                        || (self.input_target == InputTarget::PipelineVariableKey
                            && self.pipeline_variable_key_input.is_empty()) =>
                {
                    let _ = self.submit_pipeline_launch().await;
                }
                KeyCode::Enter => match self.input_target {
                    InputTarget::PipelineRef => {
                        if self.show_branches_dropdown {
                            if let Some(b) = self
                                .branches_state
                                .selected()
                                .and_then(|idx| self.filtered_branches.get(idx))
                            {
                                self.pipeline_ref_input = b.name.clone();
                            }
                            self.show_branches_dropdown = false;
                            self.input_target = InputTarget::PipelineVariableKey;
                        } else {
                            self.input_target = InputTarget::PipelineVariableKey;
                        }
                        self.label_state.select(Some(0));
                    }
                    InputTarget::PipelineVariableKey => {
                        let suggested_vars: Vec<&ProjectVariable> = self
                            .project_variables
                            .iter()
                            .filter(|v| {
                                v.key
                                    .to_lowercase()
                                    .contains(&self.pipeline_variable_key_input.to_lowercase())
                            })
                            .collect();
                        if let Some(v) = self
                            .label_state
                            .selected()
                            .and_then(|idx| suggested_vars.get(idx))
                        {
                            self.pipeline_variable_key_input = v.key.clone();
                        }
                        self.input_target = InputTarget::PipelineVariableValue;
                    }
                    InputTarget::PipelineVariableValue
                        if !self.pipeline_variable_key_input.is_empty() =>
                    {
                        self.pipeline_variables.push(PipelineVariable {
                            key: self.pipeline_variable_key_input.clone(),
                            value: self.pipeline_variable_value_input.clone(),
                        });
                        self.pipeline_variable_key_input.clear();
                        self.pipeline_variable_value_input.clear();
                        self.input_target = InputTarget::PipelineVariableKey;
                    }
                    _ => {}
                },
                KeyCode::Delete | KeyCode::Backspace
                    if self.input_target == InputTarget::PipelineVariableList =>
                {
                    if let Some(idx) = self
                        .pipeline_variables_state
                        .selected()
                        .filter(|&idx| idx < self.pipeline_variables.len())
                    {
                        self.pipeline_variables.remove(idx);
                    }
                }
                KeyCode::Esc => {
                    self.show_pipeline_popup = false;
                    self.input_mode = false;
                }
                _ => match self.input_target {
                    InputTarget::PipelineRef => match processed_key.code {
                        KeyCode::Char(c) => {
                            self.pipeline_ref_input.push(c);
                            self.show_branches_dropdown = true;
                            self.filter_branches();
                        }
                        KeyCode::Backspace => {
                            self.pipeline_ref_input.pop();
                            self.filter_branches();
                        }
                        KeyCode::Down => {
                            if !self.filtered_branches.is_empty() {
                                let i = (self.branches_state.selected().unwrap_or(0) + 1)
                                    % self.filtered_branches.len();
                                self.branches_state.select(Some(i));
                            }
                        }
                        KeyCode::Up if !self.filtered_branches.is_empty() => {
                            let i = (self.branches_state.selected().unwrap_or(0)
                                + self.filtered_branches.len().saturating_sub(1))
                                % self.filtered_branches.len();
                            self.branches_state.select(Some(i));
                        }
                        _ => {}
                    },
                    InputTarget::PipelineVariableKey => {
                        let suggested_vars: Vec<&ProjectVariable> = self
                            .project_variables
                            .iter()
                            .filter(|v| {
                                v.key
                                    .to_lowercase()
                                    .contains(&self.pipeline_variable_key_input.to_lowercase())
                            })
                            .collect();
                        match processed_key.code {
                            KeyCode::Char(c) => {
                                self.pipeline_variable_key_input.push(c);
                                self.label_state.select(Some(0));
                            }
                            KeyCode::Backspace => {
                                self.pipeline_variable_key_input.pop();
                                self.label_state.select(Some(0));
                            }
                            KeyCode::Down => {
                                if !suggested_vars.is_empty() {
                                    let i = (self.label_state.selected().unwrap_or(0) + 1)
                                        % suggested_vars.len();
                                    self.label_state.select(Some(i));
                                }
                            }
                            KeyCode::Up if !suggested_vars.is_empty() => {
                                let i = (self.label_state.selected().unwrap_or(0)
                                    + suggested_vars.len().saturating_sub(1))
                                    % suggested_vars.len();
                                self.label_state.select(Some(i));
                            }
                            _ => {}
                        }
                    }
                    InputTarget::PipelineVariableValue => {
                        if let KeyCode::Char(c) = processed_key.code {
                            self.pipeline_variable_value_input.push(c);
                        } else if let KeyCode::Backspace = processed_key.code {
                            self.pipeline_variable_value_input.pop();
                        }
                    }
                    _ => {}
                },
            }
            let _ = self.update_preview().await;
            return true;
        }

        false
    }
}
