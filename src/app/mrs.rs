use gitlab::api::AsyncQuery;
use gitlab::api::projects::merge_requests::MergeRequest as MergeRequestEndpoint;
use crate::models::*;
use crate::api::*;
use crate::app::parse_reactions;

impl App {
    pub async fn fetch_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            self.merge_requests.clear(); self.filter_lists(); // Synchronous clear to prevent jumps
            let _ = tx.send(DataUpdate::Loading(true));
            self.status_message = Some("Fetching Merge Requests...".into());

            tokio::spawn(async move {
                let ep = PagedMergeRequests { project: pid, page: 1, per_page: 50 };
                let res: Result<Result<Vec<MergeRequest>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
                match res {
                    Ok(Ok(data)) => { 
                        let has_more = data.len() == 50;
                        let _ = tx.send(DataUpdate::ProjectMRs { project_id: pid, data, page: 1, has_more }); 
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Merge Requests fetch timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_next_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.mrs_has_more || self.is_loading { return Ok(()); }
        
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let next_page = self.mrs_page + 1;
            let _ = tx.send(DataUpdate::Loading(true));

            tokio::spawn(async move {
                let ep = PagedMergeRequests {
                    project: pid,
                    page: next_page,
                    per_page: 50,
                };
                let res: Result<Result<Vec<MergeRequest>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
                match res {
                    Ok(Ok(data)) => {
                        if !data.is_empty() {
                            let _ = tx.send(DataUpdate::AppendProjectMRs { project_id: pid, data });
                        } else {
                            let _ = tx.send(DataUpdate::Loading(false));
                        }
                    }
                    Ok(Err(e)) => { 
                        let _ = tx.send(DataUpdate::Error(e.to_string())); 
                        let _ = tx.send(DataUpdate::Loading(false));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Merge Requests fetch timed out".into()));
                        let _ = tx.send(DataUpdate::Loading(false));
                    }
                }
            });
        }
        Ok(())
    }

    pub async fn fetch_mr_diffs(&mut self, project_id: u64, mr_iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = MergeRequestDiffs {
            project: project_id,
            merge_request: mr_iid,
        };
        let diffs: Vec<MRDiff> = endpoint.query_async(&self.gitlab).await?;
        self.mr_diffs = diffs;
        self.mr_diff_state.select(Some(0));
        Ok(())
    }

    pub async fn approve_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = ApproveMergeRequest { project: pid, merge_request: iid };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
                if res.is_err() { let _ = tx.send(DataUpdate::Error("Approve MR timed out".into())); }
                let _ = tx.send(DataUpdate::Loading(false));
                let _ = tx.send(DataUpdate::TriggerRefresh);
            });
        }
        Ok(())
    }

    pub async fn unapprove_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = UnapproveMergeRequest { project: pid, merge_request: iid };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
                if res.is_err() { let _ = tx.send(DataUpdate::Error("Unapprove MR timed out".into())); }
                let _ = tx.send(DataUpdate::Loading(false));
                let _ = tx.send(DataUpdate::TriggerRefresh);
            });
        }
        Ok(())
    }

    pub async fn merge_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = AcceptMergeRequest { project: pid, merge_request: iid, squash: false, should_remove_source_branch: true };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
                if res.is_err() { let _ = tx.send(DataUpdate::Error("Merge MR timed out".into())); }
                let _ = tx.send(DataUpdate::Loading(false));
                let _ = tx.send(DataUpdate::TriggerRefresh);
            });
        }
        Ok(())
    }

    pub async fn fetch_discussions(&mut self, project_id: u64, mr_iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            // 1. Fetch single MR to get latest upvotes/downvotes
            let mr_endpoint = MergeRequestEndpoint::builder().project(project_id).merge_request(mr_iid).build().unwrap();
            let mr_res: Result<crate::models::MergeRequest, _> = mr_endpoint.query_async(&client).await;
            
            if let Ok(mut mr) = mr_res {
                // 2. Fetch award emojis
                let emoji_endpoint = MergeRequestAwardEmoji { project: project_id, merge_request: mr_iid };
                let reaction_values: Vec<serde_json::Value> = gitlab::api::paged(emoji_endpoint, gitlab::api::Pagination::All).query_async(&client).await.unwrap_or_default();
                mr.reactions = parse_reactions(reaction_values);
                let _ = tx.send(DataUpdate::MRDetail(mr));
            }

            // 3. Fetch discussions
            let endpoint = MergeRequestDiscussions { project: project_id, merge_request: mr_iid };
            let res: Result<Result<Vec<Discussion>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(mut discussions)) => {
                    for d in discussions.iter_mut() { d.collapsed = true; }
                    
                    // 4. Enrich with diff context
                    let diff_endpoint = MergeRequestDiffs { project: project_id, merge_request: mr_iid };
                    let mr_diffs: Vec<MRDiff> = diff_endpoint.query_async(&client).await.unwrap_or_default();

                    for discussion in &mut discussions {
                        discussion.notes.retain(|n| !n.system);
                        for note in &mut discussion.notes {
                            if let Some(pos) = &note.position {
                                let path = pos.new_path.as_ref().or(pos.old_path.as_ref());
                                if let (Some(path), Some(_target)) = (path, pos.new_line.or(pos.old_line)) {
                                    if let Some(mr_diff) = mr_diffs.iter().find(|d| d.new_path == *path || d.old_path == *path) {
                                        if let Some(context) = parse_diff_context(&mr_diff.diff, pos.new_line, pos.old_line) {
                                            note.context = Some(context);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    discussions.retain(|d| !d.notes.is_empty());
                    let _ = tx.send(DataUpdate::Discussions { project_id, iid: mr_iid, data: discussions });
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Discussions fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn toggle_mr_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let event = if mr.state == "opened" { "close" } else { "reopen" };
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditMergeRequest {
                    project: pid, merge_request: iid, title: None, description: None, state_event: Some(event.to_string()),
                    labels: None, assignee_ids: None, reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: None, target_branch: None,
                };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res {
                    Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Toggle MR state timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let mut edit = EditMergeRequest {
                project: pid, merge_request: iid, title: None, description: None, state_event: None,
                labels: None, assignee_ids: None, reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: None, target_branch: None,
            };

            match self.input_target {
                InputTarget::EditTitle => edit.title = Some(self.reply_input.clone()),
                InputTarget::EditDescription => edit.description = Some(self.reply_input.clone()),
                _ => return Ok(()),
            }

            self.input_mode = false;
            self.reply_input.clear();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res {
                    Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Update MR timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_mr_labels(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let labels = self.project_labels.iter()
                .filter(|l| self.selected_label_ids.contains(&l.id))
                .map(|l| l.name.clone())
                .collect::<Vec<_>>()
                .join(",");

            self.show_label_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditMergeRequest {
                    project: pid, merge_request: iid, title: None, description: None, state_event: None,
                    labels: Some(labels), assignee_ids: None, reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: None, target_branch: None,
                };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res {
                    Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Update MR labels timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_mr_assignees(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let assignee_ids = self.selected_assignee_ids.clone();

            self.show_assignee_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditMergeRequest {
                    project: pid, merge_request: iid, title: None, description: None, state_event: None,
                    labels: None, assignee_ids: Some(assignee_ids), reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: None, target_branch: None,
                };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res {
                    Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Update MR assignees timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_mr_milestone(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let mid = if let Some(i) = self.milestone_state.selected() {
                if i == 0 { None } else { Some(self.project_milestones[i - 1].id) }
            } else { None };

            self.show_milestone_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditMergeRequest {
                    project: pid, merge_request: iid, title: None, description: None, state_event: None,
                    labels: None, assignee_ids: None, reviewer_ids: None, milestone_id: Some(mid), remove_source_branch: None, squash: None, target_branch: None,
                };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res {
                    Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Update MR milestone timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }
}

pub fn parse_diff_context(diff: &str, new_line: Option<u64>, old_line: Option<u64>) -> Option<Vec<(u64, String, LineType, bool)>> {
    let target = new_line.or(old_line)?;
    let mut all_diff_lines = Vec::new();
    let mut current_old_line = 0;
    let mut current_new_line = 0;

    for line in diff.lines() {
        if line.starts_with("@@") {
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
            let l = (LineType::Unchanged, if new_line.is_some() { current_new_line } else { current_old_line });
            current_old_line += 1;
            current_new_line += 1;
            l
        };

        all_diff_lines.push((ln, line.to_string(), ltype));
    }

    if let Some(target_idx) = all_diff_lines.iter().position(|(ln, _, lt)| {
        *ln == target && (
            (new_line.is_some() && *lt == LineType::Addition) ||
            (old_line.is_some() && *lt == LineType::Removal) ||
            (*lt == LineType::Unchanged)
        )
    }) {
        let start = target_idx.saturating_sub(3);
        let end = (target_idx + 4).min(all_diff_lines.len());
        let mut context = Vec::new();
        
        for (i, (ln, content, ltype)) in all_diff_lines[start..end].iter().enumerate() {
            let is_target = (start + i) == target_idx;
            let clean_content = if content.starts_with('+') || content.starts_with('-') || content.starts_with(' ') {
                &content[1..]
            } else {
                content
            };
            context.push((*ln, clean_content.to_string(), ltype.clone(), is_target));
        }
        Some(context)
    } else {
        None
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
        app.tx = tx;
        app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()) });
        (server, app, rx)
    }

    #[tokio::test]
    async fn test_mr_fetching_logic_exhaustive() {
        let (server, mut app, mut rx) = setup_app().await;

        let mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "t", "reactions": []}"##;

        Mock::given(path("/api/v4/projects/1/merge_requests")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([serde_json::from_str::<MergeRequest>(mr_json).unwrap()]))).mount(&server).await;
        Mock::given(path("/api/v4/projects/1/merge_requests/1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::from_str::<MergeRequest>(mr_json).unwrap())).mount(&server).await;
        Mock::given(path("/api/v4/projects/1/merge_requests/1/award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
        Mock::given(path("/api/v4/projects/1/merge_requests/1/discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
        Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
        Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;

        app.fetch_merge_requests().await.unwrap();
        let _ = tokio::time::timeout(Duration::from_secs(2), async {
            while !app.is_loading { app.process_updates(&mut rx).await; tokio::time::sleep(Duration::from_millis(10)).await; }
            while app.is_loading { app.process_updates(&mut rx).await; tokio::time::sleep(Duration::from_millis(10)).await; }
        }).await;
        assert!(!app.merge_requests.is_empty());

        app.selected_mr = Some(app.merge_requests[0].clone());
        app.fetch_discussions(1, 1).await.unwrap();
        let _ = tokio::time::timeout(Duration::from_secs(2), async {
            while app.is_loading { app.process_updates(&mut rx).await; tokio::time::sleep(Duration::from_millis(10)).await; }
        }).await;

        app.approve_mr().await.unwrap();
        app.unapprove_mr().await.unwrap();
        app.merge_mr().await.unwrap();
        app.toggle_mr_state().await.unwrap();
        
        app.input_target = InputTarget::EditTitle;
        app.reply_input = "new".into();
        app.update_mr().await.unwrap();
        
        app.project_labels = vec![Label { id: 1, name: "l".into(), color: "c".into(), text_color: "t".into() }];
        app.selected_label_ids.push(1);
        app.update_mr_labels().await.unwrap();
        
        app.selected_assignee_ids.push(1);
        app.update_mr_assignees().await.unwrap();
        
        app.project_milestones = vec![Milestone { id: 1, title: "m".into() }];
        app.milestone_state.select(Some(1));
        app.update_mr_milestone().await.unwrap();
    }
}
