use gitlab::{api::{self, AsyncQuery}};
use crate::models::*;
use crate::api::*;
use crate::app::parse_reactions;

impl App {
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

    pub async fn fetch_discussions(&mut self, project_id: u64, mr_iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        use gitlab::api::projects::merge_requests::{MergeRequestDiffs, MergeRequest};

        // Fetch single MR to get latest upvotes/downvotes
        let mr_endpoint = MergeRequest::builder()
            .project(project_id)
            .merge_request(mr_iid)
            .build()?;
        let mr: crate::models::MergeRequest = mr_endpoint.query_async(&self.gitlab).await?;
        self.selected_mr = Some(mr.clone());

        // Fetch award emojis
        let emoji_endpoint = MergeRequestAwardEmoji {
            project: project_id,
            merge_request: mr_iid,
        };
        let reaction_values: Vec<serde_json::Value> = gitlab::api::paged(emoji_endpoint, gitlab::api::Pagination::All).query_async(&self.gitlab).await?;
        let reactions = parse_reactions(reaction_values);
        if let Some(mr) = &mut self.selected_mr {
            mr.reactions = reactions;
        }

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
        self.discussions = discussions;
        Ok(())
    }

    pub async fn update_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let iid = mr.iid;
            let project_id = mr.project_id;
            self.is_loading = true;
            let mut edit = EditMergeRequest {
                project: project_id,
                merge_request: iid,
                title: None,
                description: None,
                state_event: None,
                labels: None,
                assignee_ids: None,
                milestone_id: None,
                remove_source_branch: None,
                squash: None,
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
                let _ = self.fetch_merge_requests().await;
                let _ = self.fetch_discussions(project_id, iid).await;
            }
            self.is_loading = false;
        }
        Ok(())
    }

    pub async fn update_mr_labels(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let iid = mr.iid;
            let project_id = mr.project_id;
            self.is_loading = true;
            
            let labels = self.project_labels.iter()
                .filter(|l| self.selected_label_ids.contains(&l.id))
                .map(|l| l.name.clone())
                .collect::<Vec<_>>()
                .join(",");

            let edit = EditMergeRequest {
                project: project_id,
                merge_request: iid,
                title: None,
                description: None,
                state_event: None,
                labels: Some(labels),
                assignee_ids: None,
                milestone_id: None,
                remove_source_branch: None,
                squash: None,
            };

            let res: Result<serde_json::Value, _> = edit.query_async(&self.gitlab).await;
            if res.is_ok() {
                self.show_label_popup = false;
                let _ = self.fetch_merge_requests().await;
                let _ = self.fetch_discussions(project_id, iid).await;
            }
            self.is_loading = false;
        }
        Ok(())
    }

    pub async fn update_mr_assignees(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let iid = mr.iid;
            let project_id = mr.project_id;
            self.is_loading = true;
            
            let edit = EditMergeRequest {
                project: project_id,
                merge_request: iid,
                title: None,
                description: None,
                state_event: None,
                labels: None,
                assignee_ids: Some(self.selected_assignee_ids.clone()),
                milestone_id: None,
                remove_source_branch: None,
                squash: None,
            };

            let res: Result<serde_json::Value, _> = edit.query_async(&self.gitlab).await;
            if res.is_ok() {
                self.show_assignee_popup = false;
                let _ = self.fetch_merge_requests().await;
                let _ = self.fetch_discussions(project_id, iid).await;
            }
            self.is_loading = false;
        }
        Ok(())
    }

    pub async fn update_mr_milestone(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let iid = mr.iid;
            let project_id = mr.project_id;
            self.is_loading = true;
            
            let mid = if let Some(i) = self.milestone_state.selected() {
                if i == 0 { None } else { Some(self.project_milestones[i - 1].id) }
            } else {
                None
            };

            let edit = EditMergeRequest {
                project: project_id,
                merge_request: iid,
                title: None,
                description: None,
                state_event: None,
                labels: None,
                assignee_ids: None,
                milestone_id: Some(mid),
                remove_source_branch: None,
                squash: None,
            };

            let res: Result<serde_json::Value, _> = edit.query_async(&self.gitlab).await;
            if res.is_ok() {
                self.show_milestone_popup = false;
                let _ = self.fetch_merge_requests().await;
                let _ = self.fetch_discussions(project_id, iid).await;
            }
            self.is_loading = false;
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
    use gitlab::GitlabBuilder;
    use tokio_test::block_on;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::path;

    async fn setup_app() -> (MockServer, App) {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        (server, App::new(client))
    }

    #[test]
    fn test_parse_diff_context_exhaustive() {
        let diff = "@@ -1,2 +1,3 @@\n unchanged\n-removed\n+added\n";
        let res1 = parse_diff_context(diff, Some(2), None).unwrap();
        assert_eq!(res1[2].1, "added");
        assert!(res1[2].3);
        let res2 = parse_diff_context(diff, None, Some(2)).unwrap();
        assert_eq!(res2[1].1, "removed");
        assert!(res2[1].3);
        assert!(parse_diff_context(diff, Some(99), None).is_none());
    }

    #[test]
    fn test_mr_updates_logic_exhaustive() {
        let (server, mut app) = block_on(setup_app());
        
        // Test fetch without project
        app.selected_project = None;
        let _ = block_on(app.fetch_merge_requests());
        assert!(app.merge_requests.is_empty());

        app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: "n/p".into() });
        let mr_json = r#"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "2026-04-25T12:00:00Z"}"#;
        app.selected_mr = Some(serde_json::from_str(mr_json).unwrap());

        block_on(async {
            use wiremock::matchers::method;
            Mock::given(path("/api/v4/projects/1/merge_requests/1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(serde_json::from_str::<MergeRequest>(mr_json).unwrap()))).mount(&server).await;
            Mock::given(path("/api/v4/projects/1/merge_requests/1/award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
            Mock::given(path("/api/v4/projects/1/merge_requests/1/discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"id": "d1", "notes": [{"id": 1, "body": "b", "author": {"id": 1, "username": "u"}, "system": false, "created_at": "t", "position": {"new_path": "f", "new_line": 1}}]}
            ]))).mount(&server).await;
            Mock::given(path("/api/v4/projects/1/merge_requests/1/diffs")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"old_path": "f", "new_path": "f", "diff": "@@ -1,1 +1,1 @@\n-old\n+new\n"}
            ]))).mount(&server).await;

            Mock::given(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
            Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;

            let _ = app.fetch_merge_requests().await;
            let _ = app.fetch_discussions(1, 1).await;
            
            app.input_target = InputTarget::EditTitle;
            let _ = app.update_mr().await;
            app.input_target = InputTarget::EditDescription;
            let _ = app.update_mr().await;
            
            let _ = app.update_mr_labels().await;
            let _ = app.update_mr_assignees().await;
            
            app.milestone_state.select(Some(0));
            let _ = app.update_mr_milestone().await;
        });
    }
}
