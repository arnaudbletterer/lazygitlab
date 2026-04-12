use gitlab::{AsyncGitlab, api::{self, AsyncQuery, Endpoint}};
use gitlab::api::endpoint_prelude::Method;
use std::borrow::Cow;
use crate::models::*;
use ratatui::widgets::ListState;
use crate::ui::Theme;

#[derive(Debug, Clone)]
pub struct AddDiscussionNote {
    pub project: u64,
    pub merge_request: u64,
    pub discussion: String,
    pub body: String,
}

impl Endpoint for AddDiscussionNote {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/merge_requests/{}/discussions/{}/notes",
            self.project, self.merge_request, self.discussion,
        )
        .into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("body", &self.body);
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct ResolveDiscussion {
    pub project: u64,
    pub merge_request: u64,
    pub discussion: String,
    pub resolved: bool,
}

impl Endpoint for ResolveDiscussion {
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/merge_requests/{}/discussions/{}",
            self.project, self.merge_request, self.discussion,
        )
        .into()
    }

    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("resolved", self.resolved);
        params
    }
}

impl App {
    pub fn new(gitlab: AsyncGitlab) -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        
        let mut projects_state = ListState::default();
        projects_state.select(Some(0));

        let mut mrs_state = ListState::default();
        mrs_state.select(Some(0));

        let mut discussion_state = ListState::default();
        discussion_state.select(Some(0));

        Self {
            should_quit: false,
            menu_state,
            menu_items: vec![
                ("Projects".to_string(), MenuSelection::Projects),
                ("Merge Requests".to_string(), MenuSelection::MergeRequests),
                ("Issues".to_string(), MenuSelection::Issues),
                ("Pipelines".to_string(), MenuSelection::Pipelines),
            ],
            active_selection: MenuSelection::Projects,
            gitlab,
            projects: Vec::new(),
            projects_state,
            merge_requests: Vec::new(),
            mrs_state,
            focus_on_menu: true,
            view_mode: ViewMode::List,
            selected_mr: None,
            discussions: Vec::new(),
            discussion_state,
            input_mode: false,
            reply_input: String::new(),
            theme: Theme::default(),
        }
    }

    pub fn move_down(&mut self) {
        if self.input_mode { return; }
        if self.focus_on_menu {
            let i = match self.menu_state.selected() {
                Some(i) => if i >= self.menu_items.len() - 1 { 0 } else { i + 1 },
                None => 0,
            };
            self.menu_state.select(Some(i));
            self.active_selection = self.menu_items[i].1;
        } else {
            match self.view_mode {
                ViewMode::List => {
                    match self.active_selection {
                        MenuSelection::Projects => {
                            if !self.projects.is_empty() {
                                let i = match self.projects_state.selected() {
                                    Some(i) => if i >= self.projects.len() - 1 { 0 } else { i + 1 },
                                    None => 0,
                                };
                                self.projects_state.select(Some(i));
                            }
                        }
                        MenuSelection::MergeRequests => {
                            if !self.merge_requests.is_empty() {
                                let i = match self.mrs_state.selected() {
                                    Some(i) => if i >= self.merge_requests.len() - 1 { 0 } else { i + 1 },
                                    None => 0,
                                };
                                self.mrs_state.select(Some(i));
                            }
                        }
                        _ => {}
                    }
                }
                ViewMode::Detail => {
                    if !self.discussions.is_empty() {
                        let i = match self.discussion_state.selected() {
                            Some(i) => if i >= self.discussions.len() - 1 { 0 } else { i + 1 },
                            None => 0,
                        };
                        self.discussion_state.select(Some(i));
                    }
                }
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.input_mode { return; }
        if self.focus_on_menu {
            let i = match self.menu_state.selected() {
                Some(i) => if i == 0 { self.menu_items.len() - 1 } else { i - 1 },
                None => 0,
            };
            self.menu_state.select(Some(i));
            self.active_selection = self.menu_items[i].1;
        } else {
            match self.view_mode {
                ViewMode::List => {
                    match self.active_selection {
                        MenuSelection::Projects => {
                            if !self.projects.is_empty() {
                                let i = match self.projects_state.selected() {
                                    Some(i) => if i == 0 { self.projects.len() - 1 } else { i - 1 },
                                    None => 0,
                                };
                                self.projects_state.select(Some(i));
                            }
                        }
                        MenuSelection::MergeRequests => {
                            if !self.merge_requests.is_empty() {
                                let i = match self.mrs_state.selected() {
                                    Some(i) => if i == 0 { self.merge_requests.len() - 1 } else { i - 1 },
                                    None => 0,
                                };
                                self.mrs_state.select(Some(i));
                            }
                        }
                        _ => {}
                    }
                }
                ViewMode::Detail => {
                    if !self.discussions.is_empty() {
                        let i = match self.discussion_state.selected() {
                            Some(i) => if i == 0 { self.discussions.len() - 1 } else { i - 1 },
                            None => 0,
                        };
                        self.discussion_state.select(Some(i));
                    }
                }
            }
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus_on_menu = !self.focus_on_menu;
    }

    pub async fn fetch_projects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = api::projects::Projects::builder()
            .membership(true)
            .build()?;
        let projects: Vec<Project> = endpoint.query_async(&self.gitlab).await?;
        self.projects = projects;
        Ok(())
    }

    pub async fn fetch_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = api::merge_requests::MergeRequests::builder()
            .scope(api::merge_requests::MergeRequestScope::CreatedByMe)
            .build()?;
        let mrs: Vec<MergeRequest> = endpoint.query_async(&self.gitlab).await?;
        self.merge_requests = mrs;
        Ok(())
    }

    pub async fn fetch_discussions(&mut self, project_id: u64, mr_iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        use gitlab::api::projects::merge_requests::MergeRequestDiffs;

        let endpoint = api::projects::merge_requests::discussions::MergeRequestDiscussions::builder()
            .project(project_id)
            .merge_request(mr_iid)
            .build()?;
        let mut discussions: Vec<Discussion> = endpoint.query_async(&self.gitlab).await?;
        
        // All collapsed by default
        for d in discussions.iter_mut() {
            d.collapsed = true;
        }

        // Set them immediately so we see them even if enrichment fails
        self.discussions = discussions.clone();

        let diff_endpoint = MergeRequestDiffs::builder()
            .project(project_id)
            .merge_request(mr_iid)
            .build()?;
        
        // If diffs fail, we still have the discussions
        let mr_diffs: Vec<MRDiff> = match diff_endpoint.query_async(&self.gitlab).await {
            Ok(d) => d,
            Err(_) => return Ok(()), 
        };

        for discussion in &mut discussions {
            for note in &mut discussion.notes {
                if let Some(pos) = &note.position {
                    let path = pos.new_path.as_ref().or(pos.old_path.as_ref());
                    if let (Some(path), Some(target)) = (path, pos.new_line.or(pos.old_line)) {
                        if let Some(mr_diff) = mr_diffs.iter().find(|d| d.new_path == *path || d.old_path == *path) {
                            let mut context = Vec::new();
                            let mut current_old_line = 0;
                            let mut current_new_line = 0;
                            
                            let mut all_diff_lines = Vec::new();

                            for line in mr_diff.diff.lines() {
                                if line.starts_with("@@") {
                                    // Parse @@ -old_start,old_len +new_start,new_len @@
                                    let parts: Vec<&str> = line.split_whitespace().collect();
                                    if parts.len() >= 3 {
                                        let old_info = parts[1].trim_start_matches('-');
                                        let new_info = parts[2].trim_start_matches('+');
                                        current_old_line = old_info.split(',').next().unwrap_or("0").parse().unwrap_or(0);
                                        current_new_line = new_info.split(',').next().unwrap_or("0").parse().unwrap_or(0);
                                    }
                                    continue;
                                }

                                let (ltype, ln) = if line.starts_with('+') {
                                    let l = (LineType::Addition, current_new_line);
                                    current_new_line += 1;
                                    l
                                } else if line.starts_with('-') {
                                    let l = (LineType::Removal, current_old_line);
                                    current_old_line += 1;
                                    l
                                } else {
                                    let l = (LineType::Unchanged, if pos.new_line.is_some() { current_new_line } else { current_old_line });
                                    current_old_line += 1;
                                    current_new_line += 1;
                                    l
                                };

                                all_diff_lines.push((ln, line.to_string(), ltype));
                            }

                            // Find the target line in all_diff_lines
                            if let Some(target_idx) = all_diff_lines.iter().position(|(ln, _, lt)| {
                                *ln == target && (
                                    (pos.new_line.is_some() && *lt == LineType::Addition) ||
                                    (pos.old_line.is_some() && *lt == LineType::Removal) ||
                                    (*lt == LineType::Unchanged)
                                )
                            }) {
                                let start = target_idx.saturating_sub(3);
                                let end = (target_idx + 4).min(all_diff_lines.len());
                                
                                for (i, (ln, content, ltype)) in all_diff_lines[start..end].iter().enumerate() {
                                    let is_target = (start + i) == target_idx;
                                    let clean_content = if content.starts_with('+') || content.starts_with('-') {
                                        &content[1..]
                                    } else if content.starts_with(' ') {
                                        &content[1..]
                                    } else {
                                        content
                                    };
                                    context.push((*ln, clean_content.to_string(), ltype.clone(), is_target));
                                }
                            }
                            
                            if !context.is_empty() {
                                note.context = Some(context);
                            }
                        }
                    }
                }
            }
        }

        self.discussions = discussions;
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
        }
        Ok(())
    }

    pub async fn enter(&mut self) {
        if self.input_mode {
            let _ = self.post_reply().await;
            return;
        }

        if self.focus_on_menu {
            self.focus_on_menu = false;
        } else {
            match self.view_mode {
                ViewMode::List => {
                    match self.active_selection {
                        MenuSelection::MergeRequests => {
                            if let Some(i) = self.mrs_state.selected() {
                                if let Some(mr) = self.merge_requests.get(i) {
                                    self.selected_mr = Some(mr.clone());
                                    let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                                    self.discussion_state.select(Some(0)); // Default to first discussion
                                    self.view_mode = ViewMode::Detail;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                ViewMode::Detail => {
                    self.input_mode = true;
                }
            }
        }
    }

    pub fn back(&mut self) {
        if self.input_mode {
            self.input_mode = false;
            self.reply_input.clear();
        } else if self.view_mode == ViewMode::Detail {
            self.view_mode = ViewMode::List;
            self.selected_mr = None;
            self.discussions.clear();
        } else if !self.focus_on_menu {
            self.focus_on_menu = true;
        }
    }

    pub fn open_in_vscode(&self) {
        if let Some(mr) = &self.selected_mr {
            let _ = std::process::Command::new("powershell")
                .arg("-Command")
                .arg(format!("Start-Process '{}'", mr.web_url))
                .spawn();
        }
    }

    pub async fn refresh(&mut self) {
        match self.active_selection {
            MenuSelection::Projects => { let _ = self.fetch_projects().await; }
            MenuSelection::MergeRequests => { 
                if self.view_mode == ViewMode::Detail {
                    if let Some(mr) = &self.selected_mr {
                        let _ = self.fetch_discussions(mr.project_id, mr.iid).await;
                    }
                } else {
                    let _ = self.fetch_merge_requests().await; 
                }
            }
            _ => {}
        }
    }
}
