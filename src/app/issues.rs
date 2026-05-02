use gitlab::api::AsyncQuery;
use crate::models::*;
use crate::api::*;

impl App {
    pub async fn fetch_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            self.issues.clear(); self.filter_lists(); // Synchronous clear to prevent jumps
            let _ = tx.send(DataUpdate::Loading(true));
            self.status_message = Some("Fetching Issues...".into());

            tokio::spawn(async move {
                let ep = PagedIssues { project: pid, page: 1, per_page: 50 };
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

    pub async fn fetch_issue_discussions(&mut self, project_id: u64, issue_iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let ep_issue = SingleIssue { project: project_id, issue: issue_iid };
            if let Ok(mut issue) = AsyncQuery::<Issue, _>::query_async(&ep_issue, &client).await {
                let emoji_endpoint = IssueAwardEmoji { project: project_id, issue: issue_iid };
                if let Ok(reactions) = AsyncQuery::<Vec<serde_json::Value>, _>::query_async(&gitlab::api::paged(emoji_endpoint, gitlab::api::Pagination::All), &client).await {
                    issue.reactions = crate::app::parse_reactions(reactions);
                }
                let _ = tx.send(DataUpdate::IssueDetail(issue));
            }

            let endpoint = IssueDiscussions { project: project_id, issue: issue_iid };
            if let Ok(mut discussions) = AsyncQuery::<Vec<Discussion>, _>::query_async(&endpoint, &client).await {
                for d in discussions.iter_mut() { d.collapsed = true; d.notes.retain(|n| !n.system); }
                discussions.retain(|d| !d.notes.is_empty());
                let _ = tx.send(DataUpdate::IssueDiscussions { project_id, iid: issue_iid, data: discussions });
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn update_issue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let mut edit = EditIssue {
                project: pid, issue: iid, title: None, description: None, state_event: None,
                labels: None, assignee_ids: None, milestone_id: None, due_date: None, weight: None, confidential: None,
            };

            match self.input_target {
                InputTarget::EditTitle => edit.title = Some(self.reply_input.clone()),
                InputTarget::EditDescription => edit.description = Some(self.reply_input.clone()),
                _ => return Ok(()),
            }

            self.input_mode = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let _: Result<serde_json::Value, _> = edit.query_async(&client).await;
                let _ = tx.send(DataUpdate::TriggerRefresh);
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_issue_due_date(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let due_date = self.calendar_date.format("%Y-%m-%d").to_string();

            self.show_calendar_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditIssue {
                    project: pid, issue: iid, title: None, description: None, state_event: None,
                    labels: None, assignee_ids: None, milestone_id: None, due_date: Some(Some(due_date)), weight: None, confidential: None,
                };
                let _: Result<serde_json::Value, _> = edit.query_async(&client).await;
                let _ = tx.send(DataUpdate::TriggerRefresh);
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_issue_weight(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let weight = self.weight_state.selected().map(|w| w as i64);

            self.show_weight_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditIssue {
                    project: pid, issue: iid, title: None, description: None, state_event: None,
                    labels: None, assignee_ids: None, milestone_id: None, due_date: None, weight: Some(weight), confidential: None,
                };
                let _: Result<serde_json::Value, _> = edit.query_async(&client).await;
                let _ = tx.send(DataUpdate::TriggerRefresh);
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn toggle_issue_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let event = if issue.state == "opened" { "close" } else { "reopen" };
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditIssue {
                    project: pid, issue: iid, title: None, description: None, state_event: Some(event.to_string()),
                    labels: None, assignee_ids: None, milestone_id: None, due_date: None, weight: None, confidential: None,
                };
                let _: Result<serde_json::Value, _> = edit.query_async(&client).await;
                let _ = tx.send(DataUpdate::TriggerRefresh);
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }
    pub async fn fetch_next_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    pub async fn toggle_confidential(&mut self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    pub async fn clear_due_date(&mut self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    pub async fn update_issue_milestone(&mut self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    pub async fn update_issue_assignees(&mut self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    pub async fn update_issue_labels(&mut self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
}
