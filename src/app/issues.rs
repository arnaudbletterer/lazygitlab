use gitlab::{api::{self, AsyncQuery}};
use crate::models::*;
use crate::api::*;
use crate::app::parse_reactions;

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
        // Fetch single issue to get latest upvotes/downvotes
        let issue_endpoint = api::projects::issues::Issues::builder()
            .project(project_id)
            .iids([issue_iid].iter().copied())
            .with_labels_details(true)
            .build()?;
        let issues: Vec<crate::models::Issue> = issue_endpoint.query_async(&self.gitlab).await?;
        if let Some(issue) = issues.into_iter().next() {
            self.selected_issue = Some(issue);
        }

        // Fetch award emojis
        let emoji_endpoint = IssueAwardEmoji {
            project: project_id,
            issue: issue_iid,
        };
        let reaction_values: Vec<serde_json::Value> = gitlab::api::paged(emoji_endpoint, gitlab::api::Pagination::All).query_async(&self.gitlab).await?;
        let reactions = parse_reactions(reaction_values);
        
        if let Some(selected) = &mut self.selected_issue {
            if selected.iid == issue_iid {
                selected.reactions = reactions;
            }
        }

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
                labels: None,
                assignee_ids: None,
                milestone_id: None,
                due_date: None,
                weight: None,
                confidential: None,
            };
            let res: Result<serde_json::Value, _> = endpoint.query_async(&self.gitlab).await;
            if res.is_ok() {
                let _ = self.fetch_issues().await;
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
                labels: None,
                assignee_ids: None,
                milestone_id: None,
                due_date: None,
                weight: None,
                confidential: None,
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
            }
            self.is_loading = false;
        }
        Ok(())
    }

    pub async fn update_issue_labels(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let iid = issue.iid;
            let project_id = issue.project_id;
            self.is_loading = true;
            
            let labels = self.project_labels.iter()
                .filter(|l| self.selected_label_ids.contains(&l.id))
                .map(|l| l.name.clone())
                .collect::<Vec<_>>()
                .join(",");

            let edit = EditIssue {
                project: project_id,
                issue: iid,
                title: None,
                description: None,
                state_event: None,
                labels: Some(labels),
                assignee_ids: None,
                milestone_id: None,
                due_date: None,
                weight: None,
                confidential: None,
            };

            let res: Result<serde_json::Value, _> = edit.query_async(&self.gitlab).await;
            if res.is_ok() {
                self.show_label_popup = false;
                let _ = self.fetch_issues().await;
                let _ = self.fetch_issue_discussions(project_id, iid).await;
            }
            self.is_loading = false;
        }
        Ok(())
    }

    pub async fn update_issue_assignees(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let iid = issue.iid;
            let project_id = issue.project_id;
            self.is_loading = true;
            
            let edit = EditIssue {
                project: project_id,
                issue: iid,
                title: None,
                description: None,
                state_event: None,
                labels: None,
                assignee_ids: Some(self.selected_assignee_ids.clone()),
                milestone_id: None,
                due_date: None,
                weight: None,
                confidential: None,
            };

            let res: Result<serde_json::Value, _> = edit.query_async(&self.gitlab).await;
            if res.is_ok() {
                self.show_assignee_popup = false;
                let _ = self.fetch_issues().await;
                let _ = self.fetch_issue_discussions(project_id, iid).await;
            }
            self.is_loading = false;
        }
        Ok(())
    }

    pub async fn update_issue_milestone(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let iid = issue.iid;
            let project_id = issue.project_id;
            self.is_loading = true;
            
            let mid = if let Some(i) = self.milestone_state.selected() {
                if i == 0 {
                    None // "None" selected
                } else {
                    Some(self.project_milestones[i - 1].id)
                }
            } else {
                None
            };

            let edit = EditIssue {
                project: project_id,
                issue: iid,
                title: None,
                description: None,
                state_event: None,
                labels: None,
                assignee_ids: None,
                milestone_id: Some(mid),
                due_date: None,
                weight: None,
                confidential: None,
            };

            let res: Result<serde_json::Value, _> = edit.query_async(&self.gitlab).await;
            if res.is_ok() {
                self.show_milestone_popup = false;
                let _ = self.fetch_issues().await;
                let _ = self.fetch_issue_discussions(project_id, iid).await;
            }
            self.is_loading = false;
        }
        Ok(())
    }

    pub async fn update_issue_due_date(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let iid = issue.iid;
            let project_id = issue.project_id;
            self.is_loading = true;
            
            let edit = EditIssue {
                project: project_id,
                issue: iid,
                title: None,
                description: None,
                state_event: None,
                labels: None,
                assignee_ids: None,
                milestone_id: None,
                due_date: Some(Some(self.calendar_date.format("%Y-%m-%d").to_string())),
                weight: None,
                confidential: None,
            };

            let res: Result<serde_json::Value, _> = edit.query_async(&self.gitlab).await;
            if res.is_ok() {
                self.show_calendar_popup = false;
                let _ = self.fetch_issues().await;
                let _ = self.fetch_issue_discussions(project_id, iid).await;
            }
            self.is_loading = false;
        }
        Ok(())
    }

    pub async fn clear_due_date(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let iid = issue.iid;
            let project_id = issue.project_id;
            self.is_loading = true;
            
            let edit = EditIssue {
                project: project_id,
                issue: iid,
                title: None,
                description: None,
                state_event: None,
                labels: None,
                assignee_ids: None,
                milestone_id: None,
                due_date: Some(None),
                weight: None,
                confidential: None,
            };

            let res: Result<serde_json::Value, _> = edit.query_async(&self.gitlab).await;
            if res.is_ok() {
                self.show_calendar_popup = false;
                let _ = self.fetch_issues().await;
                let _ = self.fetch_issue_discussions(project_id, iid).await;
            }
            self.is_loading = false;
        }
        Ok(())
    }

    pub async fn update_issue_weight(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let iid = issue.iid;
            let project_id = issue.project_id;
            self.is_loading = true;
            
            let weight = self.weight_state.selected().map(|w| w as i64);

            let edit = EditIssue {
                project: project_id,
                issue: iid,
                title: None,
                description: None,
                state_event: None,
                labels: None,
                assignee_ids: None,
                milestone_id: None,
                due_date: None,
                weight: Some(weight),
                confidential: None,
            };

            let res: Result<serde_json::Value, _> = edit.query_async(&self.gitlab).await;
            if res.is_ok() {
                self.show_weight_popup = false;
                let _ = self.fetch_issues().await;
                let _ = self.fetch_issue_discussions(project_id, iid).await;
            }
            self.is_loading = false;
        }
        Ok(())
    }

    pub async fn toggle_confidential(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let iid = issue.iid;
            let project_id = issue.project_id;
            self.is_loading = true;
            
            let edit = EditIssue {
                project: project_id,
                issue: iid,
                title: None,
                description: None,
                state_event: None,
                labels: None,
                assignee_ids: None,
                milestone_id: None,
                due_date: None,
                weight: None,
                confidential: Some(!issue.confidential),
            };

            let res: Result<serde_json::Value, _> = edit.query_async(&self.gitlab).await;
            if res.is_ok() {
                let _ = self.fetch_issues().await;
                let _ = self.fetch_issue_discussions(project_id, iid).await;
            }
            self.is_loading = false;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitlab::GitlabBuilder;
    use tokio_test::block_on;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    async fn setup_app() -> (MockServer, App) {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        (server, App::new(client))
    }

    #[test]
    fn test_issue_updates_logic_exhaustive() {
        let (server, mut app) = block_on(setup_app());
        
        // Test fetch_issues without project
        app.selected_project = None;
        let _ = block_on(app.fetch_issues());
        assert!(app.issues.is_empty());

        app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: "n/p".into() });
        let issue_json = r#"{"iid": 1, "project_id": 1, "title": "I", "description": "d", "author": {"id": 1, "username": "u"}, "assignees": [], "state": "opened", "web_url": "u", "labels": [], "milestone": null, "due_date": null, "weight": 5, "confidential": false, "created_at": "2026-04-25T12:00:00Z", "upvotes": 0, "downvotes": 0}"#;
        app.selected_issue = Some(serde_json::from_str(issue_json).unwrap());

        block_on(async {
            // Mocks for fetch_issue_discussions
            Mock::given(path("/api/v4/projects/1/issues/1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(serde_json::from_str::<Issue>(issue_json).unwrap()))).mount(&server).await;
            Mock::given(path("/api/v4/projects/1/issues/1/award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
            Mock::given(path("/api/v4/projects/1/issues/1/discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"id": "1", "notes": [{"id": 1, "body": "n", "author": {"id": 1, "username": "u"}, "system": false, "created_at": "t"}]}
            ]))).mount(&server).await;

            Mock::given(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
            Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;

            let _ = app.fetch_issues().await;
            let _ = app.fetch_issue_discussions(1, 1).await;
            let _ = app.update_issue().await;
            let _ = app.update_issue_labels().await;
            let _ = app.update_issue_assignees().await;
            let _ = app.update_issue_milestone().await;
            
            // Set some selections for date/weight
            app.calendar_date = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
            let _ = app.update_issue_due_date().await;
            
            app.weight_state.select(Some(5));
            let _ = app.update_issue_weight().await;
            
            let _ = app.toggle_confidential().await;
            let _ = app.toggle_issue_state().await;

            // Test toggle when closed
            if let Some(i) = &mut app.selected_issue { i.state = "closed".into(); }
            let _ = app.toggle_issue_state().await;
        });
    }
}
