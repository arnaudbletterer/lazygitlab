use gitlab::{api::{self, AsyncQuery}};
use crate::models::*;
use crate::api::*;

impl App {
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
}
