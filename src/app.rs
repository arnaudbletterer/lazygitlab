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

#[derive(Debug, Clone)]
pub struct AddIssueDiscussionNote {
    pub project: u64,
    pub issue: u64,
    pub discussion: String,
    pub body: String,
}

impl Endpoint for AddIssueDiscussionNote {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/issues/{}/discussions/{}/notes",
            self.project, self.issue, self.discussion,
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
pub struct EditIssue {
    pub project: u64,
    pub issue: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub state_event: Option<String>,
}

impl Endpoint for EditIssue {
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}", self.project, self.issue).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        if let Some(t) = &self.title { params.push("title", t); }
        if let Some(d) = &self.description { params.push("description", d); }
        if let Some(s) = &self.state_event { params.push("state_event", s); }
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct IssueDiscussions {
    pub project: u64,
    pub issue: u64,
}

impl Endpoint for IssueDiscussions {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}/discussions", self.project, self.issue).into()
    }
}

#[derive(Debug, Clone)]
pub struct AddIssueNote {
    pub project: u64,
    pub issue: u64,
    pub body: String,
}

impl Endpoint for AddIssueNote {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}/notes", self.project, self.issue).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("body", &self.body);
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct ResolveIssueDiscussion {
    pub project: u64,
    pub issue: u64,
    pub discussion: String,
    pub resolved: bool,
}

impl Endpoint for ResolveIssueDiscussion {
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/issues/{}/discussions/{}",
            self.project, self.issue, self.discussion,
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
                                // Loop back to metadata?
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
                            MetadataField::Title => MetadataField::Description,
                            MetadataField::Description => {
                                self.detail_section = DetailSection::Discussions;
                                if !self.discussions.is_empty() {
                                    self.discussion_state.select(Some(self.discussions.len() - 1));
                                }
                                MetadataField::Title // Dummy value to reset
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

    pub async fn toggle_issue_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let iid = issue.iid;
            let project_id = issue.project_id;
            self.is_loading = true;
            let event = if issue.state == "opened" { "close" } else { "reopen" };
            let endpoint = EditIssue {
                project: project_id,
                issue: iid,
                title: None,
                description: None,
                state_event: Some(event.to_string()),
            };
            let res: Result<serde_json::Value, _> = endpoint.query_async(&self.gitlab).await;
            if res.is_ok() {
                let _ = self.fetch_issues().await;
                if let Some(updated) = self.issues.iter().find(|i| i.iid == iid) {
                    self.selected_issue = Some(updated.clone());
                }
            }
            self.is_loading = false;
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
        }
        Ok(())
    }

    pub async fn update_issue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let iid = issue.iid;
            let project_id = issue.project_id;
            self.is_loading = true;
            let mut edit = EditIssue {
                project: project_id,
                issue: iid,
                title: None,
                description: None,
                state_event: None,
            };

            match self.input_target {
                InputTarget::EditTitle => edit.title = Some(self.reply_input.clone()),
                InputTarget::EditDescription => edit.description = Some(self.reply_input.clone()),
                _ => { self.is_loading = false; return Ok(()); }
            }

            let res: Result<serde_json::Value, _> = edit.query_async(&self.gitlab).await;
            if res.is_ok() {
                self.input_mode = false;
                self.reply_input.clear();
                
                // Refresh everything
                let _ = self.fetch_issues().await;
                let _ = self.fetch_issue_discussions(project_id, iid).await;
                if let Some(updated) = self.issues.iter().find(|i| i.iid == iid) {
                    self.selected_issue = Some(updated.clone());
                }
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

    pub async fn fetch_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mrs: Vec<MergeRequest> = if let Some(project) = &self.selected_project {
            let endpoint = api::projects::merge_requests::MergeRequests::builder()
                .project(project.id)
                .build()?;
            endpoint.query_async(&self.gitlab).await?
        } else {
            Vec::new()
        };
        
        self.merge_requests = mrs;
        Ok(())
    }

    pub async fn fetch_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let issues: Vec<Issue> = if let Some(project) = &self.selected_project {
            let endpoint = api::projects::issues::Issues::builder()
                .project(project.id)
                .with_labels_details(true)
                .build()?;
            endpoint.query_async(&self.gitlab).await?
        } else {
            Vec::new()
        };
        
        self.issues = issues;
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
            discussion.notes.retain(|n| !n.system);
            
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

        discussions.retain(|d| !d.notes.is_empty());
        self.discussions = discussions;
        Ok(())
    }

    pub async fn fetch_issue_discussions(&mut self, project_id: u64, issue_iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = IssueDiscussions {
            project: project_id,
            issue: issue_iid,
        };
        let mut discussions: Vec<Discussion> = endpoint.query_async(&self.gitlab).await?;
        
        for d in discussions.iter_mut() {
            d.collapsed = true;
            d.notes.retain(|n| !n.system);
        }

        discussions.retain(|d| !d.notes.is_empty());
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
                    // The endpoint for resolving issue threads is slightly different in API structure but ResolveDiscussion struct is for MRs.
                    // We need an Issue equivalent or a more generic one.
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
                                }
                            }
                            MetadataField::Description => {
                                if let Some(issue) = &self.selected_issue {
                                    self.input_mode = true;
                                    self.input_target = InputTarget::EditDescription;
                                    self.reply_input = issue.description.clone().unwrap_or_default();
                                }
                            }
                        }
                    }
                    DetailSection::Discussions => {
                        self.input_mode = true;
                        self.input_target = InputTarget::Reply;
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

