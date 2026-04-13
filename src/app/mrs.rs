use gitlab::{api::{self, AsyncQuery}};
use crate::models::*;

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
}
