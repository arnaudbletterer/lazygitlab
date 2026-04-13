pub mod issues;
pub mod mrs;

use gitlab::{AsyncGitlab, api::{self, AsyncQuery}};
use crate::models::*;
use crate::api::*;
use ratatui::widgets::ListState;
use crate::ui::Theme;

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
            focus_on_menu: false,
            view_mode: ViewMode::List,
            detail_section: DetailSection::Discussions,
            selected_metadata_field: MetadataField::Title,
            selected_mr: None,
            selected_issue: None,
            discussions: Vec::new(),
            discussion_state,
            input_mode: false,
            input_target: InputTarget::Reply,
            is_loading: false,
            reply_input: String::new(),
            cursor_index: 0,
            theme: Theme::default(),
        }
    }

    pub fn move_down(&mut self) {
        if self.input_mode || self.is_loading { return; }
        
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
                        _ => {}
                    }
                }
            }
            ViewMode::Detail => {
                match self.detail_section {
                    DetailSection::Metadata => {
                        self.selected_metadata_field = match self.selected_metadata_field {
                            MetadataField::Title => MetadataField::Description,
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
        if self.input_mode || self.is_loading { return; }
        
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
                        _ => {}
                    }
                }
            }
            ViewMode::Detail => {
                match self.detail_section {
                    DetailSection::Metadata => {
                        self.selected_metadata_field = match self.selected_metadata_field {
                            MetadataField::Title => {
                                self.detail_section = DetailSection::Discussions;
                                if !self.discussions.is_empty() {
                                    self.discussion_state.select(Some(self.discussions.len() - 1));
                                }
                                MetadataField::Title // Dummy value to reset
                            }
                            MetadataField::Description => MetadataField::Title,
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
                InputTarget::EditTitle | InputTarget::EditDescription => { let _ = self.update_issue().await; }
            }
            return;
        }

        if self.show_project_popup {
            self.select_project().await;
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
                        _ => {}
                    }
                }
            }
            ViewMode::Detail => {
                match self.detail_section {
                    DetailSection::Metadata => {
                        match self.selected_metadata_field {
                            MetadataField::Title => {
                                if let Some(issue) = &self.selected_issue {
                                    self.input_mode = true;
                                    self.input_target = InputTarget::EditTitle;
                                    self.reply_input = issue.title.clone();
                                    self.cursor_index = self.reply_input.chars().count();
                                }
                            }
                            MetadataField::Description => {
                                if let Some(issue) = &self.selected_issue {
                                    self.input_mode = true;
                                    self.input_target = InputTarget::EditDescription;
                                    self.reply_input = issue.description.clone().unwrap_or_default();
                                    self.cursor_index = self.reply_input.chars().count();
                                }
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

    pub fn back(&mut self) {
        if self.input_mode {
            self.input_mode = false;
            self.reply_input.clear();
        } else if self.show_project_popup {
            if self.selected_project.is_some() {
                self.show_project_popup = false;
                self.focus_on_menu = true;
            }
        } else if self.view_mode == ViewMode::Detail {
            self.view_mode = ViewMode::List;
            self.selected_mr = None;
            self.selected_issue = None;
            self.discussions.clear();
        } else if !self.focus_on_menu {
            self.focus_on_menu = true;
        }
    }

    pub fn open_in_vscode(&self) {
        let url = if let Some(mr) = &self.selected_mr {
            Some(&mr.web_url)
        } else if let Some(issue) = &self.selected_issue {
            Some(&issue.web_url)
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
        if self.show_project_popup {
            let _ = self.fetch_projects().await;
            return;
        }

        match self.view_mode {
            ViewMode::List => {
                match self.active_selection {
                    MenuSelection::MergeRequests => { let _ = self.fetch_merge_requests().await; }
                    MenuSelection::Issues => { let _ = self.fetch_issues().await; }
                    _ => {}
                }
            }
            ViewMode::Detail => {
                if let Some(mr) = &self.selected_mr {
                    let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                } else if let Some(issue) = &self.selected_issue {
                    let _ = self.fetch_issue_discussions(issue.project_id, issue.iid).await;
                }
            }
        }
    }
}
