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
                let ep = PagedIssues { project: pid, page: 1, per_page: 50, state: state_str, order_by: Some("updated_at".into()), sort: Some("desc".into()), with_labels_details: Some(true) };
                let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(60), ep.query_async(&client)).await;
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
                let ep = PagedIssues { project: pid, page: next_page, per_page: 50, state: state_str, order_by: Some("updated_at".into()), sort: Some("desc".into()), with_labels_details: Some(true) };
                let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(60), ep.query_async(&client)).await;
                match res {
                    Ok(Ok(data)) => { if !data.is_empty() { let _ = tx.send(DataUpdate::AppendProjectIssues { project_id: pid, data }); } }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issues fetch failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Issues fetch timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_global_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        self.status_message = Some("Fetching Global Issues...".into());
        let state_str = match self.issue_filter {
            IssueFilter::Opened => "opened".to_string(),
            IssueFilter::Closed => "closed".to_string(),
            IssueFilter::All => "all".to_string(),
        };

        tokio::spawn(async move {
            let ep = GlobalIssues { scope: Some("assigned_to_me".into()), state: Some(state_str), membership: Some(true), page: Some(1), per_page: Some(50), order_by: Some("updated_at".into()), sort: Some("desc".into()), with_labels_details: Some(true) };
            let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(60), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { 
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::GlobalIssues { data, page: 1, has_more }); 
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Global Issues fetch failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Global Issues fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_global_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.issues_has_more || self.is_loading { return Ok(()); }
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let next_page = self.issues_page + 1;
        let _ = tx.send(DataUpdate::Loading(true));
        let state_str = match self.issue_filter {
            IssueFilter::Opened => "opened".to_string(),
            IssueFilter::Closed => "closed".to_string(),
            IssueFilter::All => "all".to_string(),
        };
        tokio::spawn(async move {
            let ep = GlobalIssues { scope: Some("assigned_to_me".into()), state: Some(state_str), membership: Some(true), page: Some(next_page), per_page: Some(50), order_by: Some("updated_at".into()), sort: Some("desc".into()), with_labels_details: Some(true) };
            let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(60), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { if !data.is_empty() { let _ = tx.send(DataUpdate::AppendGlobalIssues(data)); } else { let _ = tx.send(DataUpdate::Loading(false)); } }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Global Issues fetch failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Global Issues fetch timed out".into())); }
            }
        });
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
                    // Filter out system threads entirely before processing
                    discussions.retain(|d| d.notes.first().map_or(false, |n| !n.system));

                    for d in discussions.iter_mut() { d.collapsed = true; d.notes.retain(|n| !n.system); }
                    discussions.retain(|d| !d.notes.is_empty());
                    let _ = tx.send(DataUpdate::IssueDiscussions { project_id, iid: issue_iid, data: discussions });
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue discussions fetch failed: {}", e))); }
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
            let edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: Some(event.to_string()), labels: None, add_labels: None, remove_labels: None, assignee_ids: None, milestone_id: None, weight: None, due_date: None, confidential: None };
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
            match res { 
                Ok(Ok(val)) => { 
                    if let Ok(updated_issue) = serde_json::from_value::<Issue>(val) {
                        let _ = tx.send(DataUpdate::IssueDetail(updated_issue));
                    }
                    let _ = tx.send(DataUpdate::TriggerRefresh); 
                } 
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); } 
                Err(_) => { let _ = tx.send(DataUpdate::Error("Issue update timed out".into())); } 
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn update_issue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid) = if let Some(issue) = &self.selected_issue { (issue.project_id, issue.iid) } else { return Ok(()); };
        let client = self.gitlab.clone(); let tx = self.tx.clone();
        let mut edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: None, labels: None, add_labels: None, remove_labels: None, assignee_ids: None, milestone_id: None, weight: None, due_date: None, confidential: None };
        let body = self.textarea.lines().join("\n");
        match self.input_target {
            InputTarget::EditTitle => { edit.title = Some(body.clone()); if let Some(i) = &mut self.selected_issue { i.title = body.clone(); } }
            InputTarget::EditDescription => { edit.description = Some(body.clone()); if let Some(i) = &mut self.selected_issue { i.description = Some(body.clone()); } }
            _ => return Ok(()),
        }
        self.input_mode = false; self.textarea = ratatui_textarea::TextArea::default(); let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
            match res { 
                Ok(Ok(val)) => { 
                    if let Ok(updated_issue) = serde_json::from_value::<Issue>(val) {
                        let _ = tx.send(DataUpdate::IssueDetail(updated_issue));
                    }
                    let _ = tx.send(DataUpdate::TriggerRefresh); 
                } 
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); } 
                Err(_) => { let _ = tx.send(DataUpdate::Error("Issue update timed out".into())); } 
            }
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
                let edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: None, labels: Some(labels), add_labels: None, remove_labels: None, assignee_ids: None, milestone_id: None, weight: None, due_date: None, confidential: None };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res { 
                    Ok(Ok(val)) => { 
                        if let Ok(updated_issue) = serde_json::from_value::<Issue>(val) {
                            let _ = tx.send(DataUpdate::IssueDetail(updated_issue));
                        }
                        let _ = tx.send(DataUpdate::TriggerRefresh); 
                    } 
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); } 
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Issue update timed out".into())); } 
                }
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
                let edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: None, labels: None, add_labels: None, remove_labels: None, assignee_ids: Some(assignee_ids), milestone_id: None, weight: None, due_date: None, confidential: None };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res { 
                    Ok(Ok(val)) => { 
                        if let Ok(updated_issue) = serde_json::from_value::<Issue>(val) {
                            let _ = tx.send(DataUpdate::IssueDetail(updated_issue));
                        }
                        let _ = tx.send(DataUpdate::TriggerRefresh); 
                    } 
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); } 
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Issue update timed out".into())); } 
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_issue_milestone(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id; let iid = issue.iid; let client = self.gitlab.clone(); let tx = self.tx.clone();
            let mid = if let Some(i) = self.milestone_state.selected() { if i == 0 { None } else { Some(self.project_milestones[i - 1].id) } } else { None };
            self.show_milestone_popup = false; let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: None, labels: None, add_labels: None, remove_labels: None, assignee_ids: None, milestone_id: Some(mid), weight: None, due_date: None, confidential: None };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res { 
                    Ok(Ok(val)) => { 
                        if let Ok(updated_issue) = serde_json::from_value::<Issue>(val) {
                            let _ = tx.send(DataUpdate::IssueDetail(updated_issue));
                        }
                        let _ = tx.send(DataUpdate::TriggerRefresh); 
                    } 
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); } 
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Issue update timed out".into())); } 
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_issue_weight(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue {
            let pid = issue.project_id; let iid = issue.iid; let client = self.gitlab.clone(); let tx = self.tx.clone();
            let weight = self.weight_state.selected().unwrap_or(0) as i64;
            self.show_weight_popup = false; let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditIssue { project: pid, issue: iid, title: None, description: None, state_event: None, labels: None, add_labels: None, remove_labels: None, assignee_ids: None, milestone_id: None, weight: Some(Some(weight)), due_date: None, confidential: None };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res { 
                    Ok(Ok(val)) => { 
                        if let Ok(updated_issue) = serde_json::from_value::<Issue>(val) {
                            let _ = tx.send(DataUpdate::IssueDetail(updated_issue));
                        }
                        let _ = tx.send(DataUpdate::TriggerRefresh); 
                    } 
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); } 
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Issue update timed out".into())); } 
                }
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

    pub async fn create_issue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let title = self.create_issue_title.clone();
            let description = if self.create_issue_description.is_empty() { None } else { Some(self.create_issue_description.clone()) };
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = CreateIssue { project: pid, title, description, assignee_ids: Vec::new(), label_names: Vec::new() };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
                match res {
                    Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue creation failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Issue creation timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
            self.show_create_issue_popup = false;
            self.input_mode = false;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    use gitlab::GitlabBuilder;
    use tokio::sync::mpsc;
    use std::time::Duration;

    async fn setup_app() -> (MockServer, App, mpsc::UnboundedReceiver<DataUpdate>) {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        let (tx, rx) = mpsc::unbounded_channel();
        let mut app = App::new(client);
        app.tx = tx; app.active_panel = Panel::Project; app.active_context = Context::Project; app.active_selection = MenuSelection::Issues; app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()), default_branch: None, namespace: None });
        (server, app, rx)
    }

    #[tokio::test]
    async fn test_issue_fetching_logic_exhaustive() {
        let (server, mut app, mut rx) = setup_app().await;
        let issue_json = r##"{"iid": 1, "project_id": 1, "title": "I", "description": "d", "author": {"id": 1, "username": "u"}, "assignees": [], "state": "opened", "web_url": "u", "labels": [], "milestone": null, "due_date": null, "weight": 1, "confidential": false, "created_at": "t", "upvotes": 0, "downvotes": 0, "references": {"full": "r"}}"##;
        Mock::given(path("/api/v4/projects/1/issues")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([serde_json::from_str::<Issue>(issue_json).unwrap()]))).mount(&server).await;
        Mock::given(path("/api/v4/projects/1/issues/1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<Issue>(issue_json).unwrap())).mount(&server).await;
        Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;

        app.fetch_issues().await.unwrap();
        let _ = tokio::time::timeout(Duration::from_secs(2), async { 
            while !app.is_loading { app.process_updates(&mut rx).await; tokio::time::sleep(Duration::from_millis(10)).await; }
            while app.is_loading { app.process_updates(&mut rx).await; tokio::time::sleep(Duration::from_millis(10)).await; } 
        }).await;
        assert!(!app.issues.is_empty());
        app.selected_issue = Some(app.issues[0].clone());
        
        app.update_issue().await.unwrap();
        app.project_labels = vec![Label { id: 1, name: "l".into(), color: "c".into(), text_color: Some("t".into()) }];
        app.selected_label_ids.push(1);
        app.update_issue_labels().await.unwrap();
        app.update_issue_assignees().await.unwrap();
        app.milestone_state.select(Some(1));
        app.project_milestones = vec![Milestone { id: 1, title: "m".into() }];
        app.update_issue_milestone().await.unwrap();
        app.weight_state.select(Some(5));
        app.update_issue_weight().await.unwrap();
    }
}
