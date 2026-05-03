use gitlab::api::AsyncQuery;
use crate::models::*;
use crate::api::*;

impl App {
    pub async fn fetch_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            self.status_message = Some("Fetching Issues...".into());

            let state_str = match self.issue_filter {
                IssueFilter::Opened => Some("opened".to_string()),
                IssueFilter::Closed => Some("closed".to_string()),
                IssueFilter::All => None,
            };

            tokio::spawn(async move {
                let ep = PagedIssues { project: pid, page: 1, per_page: 50, state: state_str };
                let res: Result<Vec<Issue>, _> = ep.query_async(&client).await;
                if let Ok(data) = res { 
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::ProjectIssues { project_id: pid, data, page: 1, has_more }); 
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_next_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.issues_has_more || self.is_loading { return Ok(()); }
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let next_page = self.issues_page + 1;
            let _ = tx.send(DataUpdate::Loading(true));
            let state_str = match self.issue_filter {
                IssueFilter::Opened => Some("opened".to_string()),
                IssueFilter::Closed => Some("closed".to_string()),
                IssueFilter::All => None,
            };
            tokio::spawn(async move {
                let ep = PagedIssues { project: pid, page: next_page, per_page: 50, state: state_str };
                let res: Result<Vec<Issue>, _> = ep.query_async(&client).await;
                match res {
                    Ok(data) => { if !data.is_empty() { let _ = tx.send(DataUpdate::AppendProjectIssues { project_id: pid, data }); } else { let _ = tx.send(DataUpdate::Loading(false)); } }
                    Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); let _ = tx.send(DataUpdate::Loading(false)); }
                }
            });
        }
        Ok(())
    }

    pub async fn fetch_issue_discussions(&mut self, project_id: u64, issue_iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let issue_endpoint = SingleIssue { project: project_id, issue: issue_iid };
            let issue_res: Result<crate::models::Issue, _> = issue_endpoint.query_async(&client).await;
            if let Ok(mut issue) = issue_res {
                let emoji_endpoint = IssueAwardEmoji { project: project_id, issue: issue_iid };
                let reaction_values: Vec<serde_json::Value> = gitlab::api::paged(emoji_endpoint, gitlab::api::Pagination::All).query_async(&client).await.unwrap_or_default();
                issue.reactions = crate::app::parse_reactions(reaction_values);
                let _ = tx.send(DataUpdate::IssueDetail(issue));
            }
            let endpoint = IssueDiscussions { project: project_id, issue: issue_iid };
            let res: Result<Result<Vec<Discussion>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(mut discussions)) => {
                    for d in discussions.iter_mut() { d.collapsed = true; }
                    discussions.retain(|d| !d.notes.is_empty());
                    let _ = tx.send(DataUpdate::IssueDiscussions { project_id, iid: issue_iid, data: discussions });
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Issue discussions fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn toggle_issue_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid, current_state) = if let Some(issue) = &self.selected_issue { (issue.project_id, issue.iid, issue.state.clone()) } else { return Ok(()); };
        let client = self.gitlab.clone(); let tx = self.tx.clone(); let event = if current_state == "opened" { "close" } else { "reopen" };
        if let Some(i) = &mut self.selected_issue { i.state = if current_state == "opened" { "closed".to_string() } else { "opened".to_string() }; }
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: Some(event.to_string()), labels: None, assignee_ids: None, milestone_id: None, due_date: None, weight: None, confidential: None };
            let _: Result<serde_json::Value, _> = edit.query_async(&client).await;
            let _ = tx.send(DataUpdate::TriggerRefresh);
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn update_issue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid) = if let Some(issue) = &self.selected_issue { (issue.project_id, issue.iid) } else { return Ok(()); };
        let client = self.gitlab.clone(); let tx = self.tx.clone();
        let mut edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: None, milestone_id: None, due_date: None, weight: None, confidential: None };
        match self.input_target {
            InputTarget::EditTitle => { edit.title = Some(self.reply_input.clone()); if let Some(i) = &mut self.selected_issue { i.title = self.reply_input.clone(); } }
            InputTarget::EditDescription => { edit.description = Some(self.reply_input.clone()); if let Some(i) = &mut self.selected_issue { i.description = Some(self.reply_input.clone()); } }
            _ => return Ok(()),
        }
        self.input_mode = false; let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let _: Result<serde_json::Value, _> = edit.query_async(&client).await;
            let _ = tx.send(DataUpdate::TriggerRefresh);
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn update_issue_labels(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id; let iid = issue.iid; let client = self.gitlab.clone(); let tx = self.tx.clone();
            let labels = self.project_labels.iter().filter(|l| self.selected_label_ids.contains(&l.id())).map(|l| l.name().to_string()).collect::<Vec<_>>().join(",");
            self.show_label_popup = false; let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: None, labels: Some(labels), assignee_ids: None, milestone_id: None, due_date: None, weight: None, confidential: None };
                let _: Result<serde_json::Value, _> = edit.query_async(&client).await;
                let _ = tx.send(DataUpdate::TriggerRefresh);
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_issue_assignees(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id; let iid = issue.iid; let client = self.gitlab.clone(); let tx = self.tx.clone();
            let assignee_ids = self.selected_assignee_ids.clone();
            self.show_assignee_popup = false; let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: Some(assignee_ids), milestone_id: None, due_date: None, weight: None, confidential: None };
                let _: Result<serde_json::Value, _> = edit.query_async(&client).await;
                let _ = tx.send(DataUpdate::TriggerRefresh);
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_issue_milestone(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id; let iid = issue.iid; let client = self.gitlab.clone(); let tx = self.tx.clone();
            let mid = self.selected_milestone_id;
            self.show_milestone_popup = false; let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: None, milestone_id: Some(mid), due_date: None, weight: None, confidential: None };
                let _: Result<serde_json::Value, _> = edit.query_async(&client).await;
                let _ = tx.send(DataUpdate::TriggerRefresh);
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_issue_weight(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id; let iid = issue.iid; let client = self.gitlab.clone(); let tx = self.tx.clone();
            let weight = self.weight_state.selected().map(|w| w as i64);
            self.show_weight_popup = false; let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: None, milestone_id: None, due_date: None, weight: Some(weight), confidential: None };
                let _: Result<serde_json::Value, _> = edit.query_async(&client).await;
                let _ = tx.send(DataUpdate::TriggerRefresh);
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_issue_due_date(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id; let iid = issue.iid; let client = self.gitlab.clone(); let tx = self.tx.clone();
            let due_date = format!("{}", self.calendar_date.format("%Y-%m-%d"));
            self.show_calendar_popup = false; let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: None, milestone_id: None, due_date: Some(Some(due_date)), weight: None, confidential: None };
                let _: Result<serde_json::Value, _> = edit.query_async(&client).await;
                let _ = tx.send(DataUpdate::TriggerRefresh);
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn resolve_issue_discussion(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            if let Some(idx) = self.discussion_state.selected() {
                if let Some(disc) = self.discussions.get(idx) {
                    let pid = issue.project_id; let iid = issue.iid; let disc_id = disc.id.clone(); let current_resolved = disc.resolved.unwrap_or(false);
                    let client = self.gitlab.clone(); let tx = self.tx.clone(); let _ = tx.send(DataUpdate::Loading(true));
                    tokio::spawn(async move {
                        let endpoint = ResolveIssueDiscussion { project: pid, issue: iid, discussion: disc_id, resolved: !current_resolved };
                        let res: Result<serde_json::Value, _> = endpoint.query_async(&client).await;
                        if res.is_ok() { let _ = tx.send(DataUpdate::TriggerRefresh); }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
                }
            }
        }
        Ok(())
    }
}
