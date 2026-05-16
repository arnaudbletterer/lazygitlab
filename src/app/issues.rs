use gitlab::api::AsyncQuery;
use crate::models::*;
use crate::api::*;

impl App {
    pub async fn fetch_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let state_str = match self.issue_filter {
                IssueFilter::Opened => "opened".to_string(),
                IssueFilter::Closed => "closed".to_string(),
                IssueFilter::All => "all".to_string(),
            };
            let _ = tx.send(DataUpdate::Loading(true));
            self.status_message = Some("Fetching Issues...".into());
            tokio::spawn(async move {
                let ep = PagedIssues { project: pid, state: Some(state_str), page: 1, per_page: 50, order_by: Some("updated_at".into()), sort: Some("desc".into()), with_labels_details: Some(true) };
                let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => { 
                        let has_more = data.len() == 50;
                        let _ = tx.send(DataUpdate::ProjectIssues { project_id: pid, data, page: 1, has_more }); 
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issues fetch failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Issues fetch timed out".into())); }
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
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let next_page = self.issues_page + 1;
            let state_str = match self.issue_filter {
                IssueFilter::Opened => "opened".to_string(),
                IssueFilter::Closed => "closed".to_string(),
                IssueFilter::All => "all".to_string(),
            };
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = PagedIssues { project: pid, state: Some(state_str), page: next_page, per_page: 50, order_by: Some("updated_at".into()), sort: Some("desc".into()), with_labels_details: Some(true) };
                let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => { if !data.is_empty() { let _ = tx.send(DataUpdate::AppendProjectIssues { project_id: pid, data }); } else { let _ = tx.send(DataUpdate::Loading(false)); } }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issues fetch failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Issues fetch timed out".into())); }
                }
            });
        }
        Ok(())
    }

    pub async fn fetch_global_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let c = self.gitlab.clone();
        let tx = self.tx.clone();
        let state_str = match self.issue_filter {
            IssueFilter::Opened => "opened".to_string(),
            IssueFilter::Closed => "closed".to_string(),
            IssueFilter::All => "all".to_string(),
        };
        let _ = tx.send(DataUpdate::Loading(true));
        let _ = tx.send(DataUpdate::ClearGlobalIssues);
        self.status_message = Some("Fetching Involved Issues...".into());

        tokio::spawn(async move {
            let scopes = vec!["assigned_to_me", "created_by_me"];
            let mut handles = Vec::new();

            for scope in scopes {
                let c_inner = c.clone();
                let tx_inner = tx.clone();
                let s_inner = state_str.clone();
                let scope_val = scope.to_string();
                
                handles.push(tokio::spawn(async move {
                    let ep = GlobalIssues { scope: Some(scope_val), state: Some(s_inner), membership: Some(true), search: None, author_id: None, assignee_id: None, page: Some(1), per_page: Some(50), order_by: Some("updated_at".into()), sort: Some("desc".into()), with_labels_details: Some(false) };
                    let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c_inner)).await;
                    if let Ok(Ok(data)) = res {
                        if !data.is_empty() { let _ = tx_inner.send(DataUpdate::AppendGlobalIssues(data)); }
                    }
                }));
            }

            for h in handles { let _ = h.await; }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_global_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Pagination for multi-scope is complex. For now, we'll just fetch more of both.
        if !self.issues_has_more || self.is_loading { return Ok(()); }
        let c = self.gitlab.clone();
        let tx = self.tx.clone();
        let next_page = self.issues_page + 1;
        self.issues_page = next_page;
        let state_str = match self.issue_filter {
            IssueFilter::Opened => "opened".to_string(),
            IssueFilter::Closed => "closed".to_string(),
            IssueFilter::All => "all".to_string(),
        };
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let scopes = vec!["assigned_to_me", "created_by_me"];
            let mut handles = Vec::new();
            for scope in scopes {
                let c_inner = c.clone(); let tx_inner = tx.clone(); let s_inner = state_str.clone(); let scope_val = scope.to_string();
                handles.push(tokio::spawn(async move {
                    let ep = GlobalIssues { scope: Some(scope_val), state: Some(s_inner), membership: Some(true), search: None, author_id: None, assignee_id: None, page: Some(next_page), per_page: Some(50), order_by: Some("updated_at".into()), sort: Some("desc".into()), with_labels_details: Some(false) };
                    let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c_inner)).await;
                    if let Ok(Ok(data)) = res {
                        if !data.is_empty() { let _ = tx_inner.send(DataUpdate::AppendGlobalIssues(data)); }
                    }
                }));
            }
            for h in handles { let _ = h.await; }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn update_issue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let body = self.textarea.lines().join("\n");
        if body.is_empty() { return Ok(()); }
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let target = self.input_target;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let mut ep_builder = EditIssue::builder();
                ep_builder.project(pid).issue(iid);
                match target {
                    InputTarget::EditTitle => { ep_builder.title(body); }
                    InputTarget::EditDescription => { ep_builder.description(body); }
                    _ => return,
                }
                let ep = ep_builder.build().unwrap();
                let res: Result<Issue, _> = ep.query_async(&c).await;
                match res {
                    Ok(data) => { let _ = tx.send(DataUpdate::IssueDetail(data)); }
                    _ => {}
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
            self.input_mode = false;
        }
        Ok(())
    }

    pub async fn update_issue_labels(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let _pid = issue.project_id;
            let _iid = issue.iid;
            let _labels = self.selected_label_ids.clone();
            let _c = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                // To update labels we need to provide the full list of names
                // But we only have IDs. Actually GitLab Edit Issue accepts label names or a comma-separated string.
                // We'll use the names from the cache.
            });
        }
        self.show_label_popup = false;
        Ok(())
    }

    pub async fn update_issue_assignees(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let assignees = self.selected_assignee_ids.clone();
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = EditIssue { project: pid, issue: iid, assignee_ids: Some(assignees), ..Default::default() };
                let res: Result<Issue, _> = ep.query_async(&c).await;
                match res {
                    Ok(data) => { let _ = tx.send(DataUpdate::IssueDetail(data)); }
                    _ => {}
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        self.show_assignee_popup = false;
        Ok(())
    }

    pub async fn update_issue_milestone(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let mid = self.selected_milestone_id;
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = EditIssue { project: pid, issue: iid, milestone_id: Some(mid), ..Default::default() };
                let res: Result<Issue, _> = ep.query_async(&c).await;
                match res {
                    Ok(data) => { let _ = tx.send(DataUpdate::IssueDetail(data)); }
                    _ => {}
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        self.show_milestone_popup = false;
        Ok(())
    }

    pub async fn update_issue_weight(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let weight = self.weight_state.selected().map(|i| i as i64);
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = EditIssue { project: pid, issue: iid, weight: Some(weight), ..Default::default() };
                let res: Result<Issue, _> = ep.query_async(&c).await;
                match res {
                    Ok(data) => { let _ = tx.send(DataUpdate::IssueDetail(data)); }
                    _ => {}
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        self.show_weight_popup = false;
        Ok(())
    }

    pub async fn toggle_issue_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let new_state = if issue.state == "opened" { "close" } else { "reopen" };
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = EditIssue { project: pid, issue: iid, state_event: Some(new_state.to_string()), ..Default::default() };
                let res: Result<Issue, _> = ep.query_async(&c).await;
                match res {
                    Ok(data) => { let _ = tx.send(DataUpdate::IssueDetail(data)); }
                    _ => {}
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn create_issue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let title = self.create_issue_title.clone();
            let description = self.create_issue_description.clone();
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = CreateIssue { project: pid, title, description: Some(description), assignee_ids: vec![], label_names: vec![] };
                let res: Result<Issue, _> = ep.query_async(&c).await;
                match res {
                    Ok(_) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                    _ => {}
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
            self.show_create_issue_popup = false;
            self.input_mode = false;
        }
        Ok(())
    }

    pub async fn fetch_issue_discussions(&mut self, project_id: u64, iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        let c = self.gitlab.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let ep = IssueDiscussions { project: project_id, issue: iid };
            let res: Result<Result<Vec<Discussion>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), gitlab::api::paged(ep, gitlab::api::Pagination::All).query_async(&c)).await;
            match res {
                Ok(Ok(data)) => { let _ = tx.send(DataUpdate::IssueDiscussions { project_id, iid, data }); }
                _ => {}
            }
        });
        Ok(())
    }

    pub async fn fetch_issue_detail(&mut self, project_id: u64, iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        let c = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let ep = SingleIssue { project: project_id, issue: iid };
            let res: Result<Result<Issue, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c)).await;
            match res {
                Ok(Ok(data)) => { let _ = tx.send(DataUpdate::IssueDetail(data)); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue detail fetch failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Issue detail fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }
}
