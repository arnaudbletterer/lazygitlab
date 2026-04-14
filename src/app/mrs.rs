use gitlab::{api::{self, AsyncQuery}};
use crate::models::*;
use crate::api::*;

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
        let mut reactions = Vec::new();
        for val in reaction_values {
            if let Some(name) = val.get("name").and_then(|v| v.as_str())
                .or_else(|| val.get("emoji").and_then(|v| v.as_str()))
                .or_else(|| val.get("emoji_name").and_then(|v| v.as_str())) {
                reactions.push(AwardEmoji { name: name.to_string() });
            }
        }
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
