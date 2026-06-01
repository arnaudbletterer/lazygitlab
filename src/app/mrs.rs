use gitlab::api::AsyncQuery;
use crate::models::*;
use crate::api::*;

impl App {
    pub async fn fetch_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            self.status_message = Some("Fetching Merge Requests...".into());

            let state_str = match self.mr_filter {
                MergeRequestFilter::Opened => Some("opened".to_string()),
                MergeRequestFilter::Merged => Some("merged".to_string()),
                MergeRequestFilter::Closed => Some("closed".to_string()),
                MergeRequestFilter::All => None,
            };

            tokio::spawn(async move {
                let ep = PagedMergeRequests { project: pid, page: 1, per_page: 50, state: state_str, order_by: Some("updated_at".into()), sort: Some("desc".into()), with_labels_details: Some(true) };
                let res: Result<Result<Vec<MergeRequest>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(60), ep.query_async(&client)).await;
                match res {
                    Ok(Ok(data)) => { 
                        let has_more = data.len() == 50;
                        let _ = tx.send(DataUpdate::ProjectMRs { project_id: pid, data, page: 1, has_more }); 
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("MRs fetch failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("MRs fetch timed out".into())); }
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
            self.mrs_page = next_page;
            let _ = tx.send(DataUpdate::Loading(true));            let state_str = match self.mr_filter {
                MergeRequestFilter::Opened => Some("opened".to_string()),
                MergeRequestFilter::Merged => Some("merged".to_string()),
                MergeRequestFilter::Closed => Some("closed".to_string()),
                MergeRequestFilter::All => None,
            };
            tokio::spawn(async move {
                let ep = PagedMergeRequests { project: pid, page: next_page, per_page: 50, state: state_str, order_by: Some("updated_at".into()), sort: Some("desc".into()), with_labels_details: Some(true) };
                let res: Result<Result<Vec<MergeRequest>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(60), ep.query_async(&client)).await;
                match res {
                    Ok(Ok(data)) => { if !data.is_empty() { let _ = tx.send(DataUpdate::AppendProjectMRs { project_id: pid, data }); } }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("MRs fetch failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("MRs fetch timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_global_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let state_str = match self.mr_filter {
            MergeRequestFilter::Opened => "opened".to_string(),
            MergeRequestFilter::Merged => "merged".to_string(),
            MergeRequestFilter::Closed => "closed".to_string(),
            MergeRequestFilter::All => "all".to_string(),
        };

        let _ = tx.send(DataUpdate::Loading(true));
        self.status_message = Some("Fetching Merge Requests...".into());

        let project_ids: Vec<u64> = self.member_project_ids.iter().cloned().collect();

        tokio::spawn(async move {
            if project_ids.is_empty() {
                let _ = tx.send(DataUpdate::GlobalMRs { data: Vec::new(), page: 1, has_more: false });
                let _ = tx.send(DataUpdate::Loading(false));
                return;
            }

            let mut all_mrs = Vec::new();
            let mut handles = Vec::new();

            for pid in project_ids {
                let c = client.clone();
                let state = state_str.clone();
                let handle = tokio::spawn(async move {
                    let ep = PagedMergeRequests {
                        project: pid,
                        state: Some(state),
                        page: 1,
                        per_page: 50,
                        order_by: Some("created_at".into()),
                        sort: Some("desc".into()),
                        with_labels_details: Some(true),
                    };
                    let res: Result<Result<Vec<MergeRequest>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(60), ep.query_async(&c)).await;
                    match res {
                        Ok(Ok(mrs)) => mrs,
                        _ => Vec::new(),
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                if let Ok(mrs) = handle.await {
                    all_mrs.extend(mrs);
                }
            }

            // Sort descending by created_at
            all_mrs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            let has_more = all_mrs.len() >= 50;
            all_mrs.truncate(50);

            let _ = tx.send(DataUpdate::GlobalMRs { data: all_mrs, page: 1, has_more });
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_global_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.mrs_has_more || self.is_loading { return Ok(()); }
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let next_page = self.mrs_page + 1;
        self.mrs_page = next_page;
        let state_str = match self.mr_filter {
            MergeRequestFilter::Opened => "opened".to_string(),
            MergeRequestFilter::Merged => "merged".to_string(),
            MergeRequestFilter::Closed => "closed".to_string(),
            MergeRequestFilter::All => "all".to_string(),
        };

        let _ = tx.send(DataUpdate::Loading(true));

        let project_ids: Vec<u64> = self.member_project_ids.iter().cloned().collect();

        tokio::spawn(async move {
            if project_ids.is_empty() {
                let _ = tx.send(DataUpdate::Loading(false));
                return;
            }

            let mut all_mrs = Vec::new();
            let mut handles = Vec::new();

            for pid in project_ids {
                let c = client.clone();
                let state = state_str.clone();
                let handle = tokio::spawn(async move {
                    let ep = PagedMergeRequests {
                        project: pid,
                        state: Some(state),
                        page: next_page,
                        per_page: 50,
                        order_by: Some("created_at".into()),
                        sort: Some("desc".into()),
                        with_labels_details: Some(true),
                    };
                    let res: Result<Result<Vec<MergeRequest>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(60), ep.query_async(&c)).await;
                    match res {
                        Ok(Ok(mrs)) => mrs,
                        _ => Vec::new(),
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                if let Ok(mrs) = handle.await {
                    all_mrs.extend(mrs);
                }
            }

            // Sort descending by created_at
            all_mrs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            let has_more = all_mrs.len() >= 50;
            all_mrs.truncate(50);

            let _ = tx.send(DataUpdate::AppendGlobalMRs { data: all_mrs, has_more });
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_discussions(&mut self, project_id: u64, mr_iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone(); let tx = self.tx.clone(); let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let mr_endpoint = SingleMergeRequest { project: project_id, merge_request: mr_iid };
            let pep = crate::api::MergeRequestParticipants { project: project_id, mr: mr_iid };
            let pager = gitlab::api::paged(pep, gitlab::api::Pagination::All);
            let (mr_res, pres): (Result<crate::models::MergeRequest, _>, Result<Vec<Author>, _>) = tokio::join!(
                mr_endpoint.query_async(&client),
                pager.query_async(&client)
            );

            let participants: Vec<Author> = pres.unwrap_or_default();

            if let Ok(mut mr) = mr_res {
                let emoji_endpoint = MergeRequestAwardEmoji { project: project_id, merge_request: mr_iid };
                let reaction_values: Vec<serde_json::Value> = gitlab::api::paged(emoji_endpoint, gitlab::api::Pagination::All).query_async(&client).await.unwrap_or_default();
                mr.reactions = crate::app::parse_reactions(reaction_values);
                mr.participants = participants;
                let _ = tx.send(DataUpdate::MRDetail(mr));
            }
            let endpoint = MergeRequestDiscussions { project: project_id, merge_request: mr_iid };
            let res: Result<Result<Vec<Discussion>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), gitlab::api::paged(endpoint, gitlab::api::Pagination::All).query_async(&client)).await;
            match res {
                Ok(Ok(mut discussions)) => {
                    discussions.retain(|d| !d.notes.is_empty());
                    let _ = tx.send(DataUpdate::Discussions { project_id, iid: mr_iid, data: discussions });
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Discussions fetch failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Discussions fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_mr_diffs(&mut self, project_id: u64, mr_iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.is_fetching_diffs = true;
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = MergeRequestDiffs { project: project_id, merge_request: mr_iid };
            let res: Result<Result<Vec<MRDiff>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), gitlab::api::paged(endpoint, gitlab::api::Pagination::All).query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { let _ = tx.send(DataUpdate::Diffs { project_id, iid: mr_iid, data }); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Diffs fetch failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Diffs fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }


    pub async fn fetch_single_file_diff(&mut self, project_id: u64, iid: u64, old_path: String, new_path: String) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        
        let mr = if let Some(mr) = &self.selected_mr {
            mr.clone()
        } else { return Ok(()); };

        let _ = tx.send(DataUpdate::Loading(true));
        let sem = self.diff_fetch_semaphore.clone();
        tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let mut current_mr = mr;
            if current_mr.diff_refs.is_none() {
                let mr_endpoint = crate::api::SingleMergeRequest { project: project_id, merge_request: iid };
                if let Ok(full_mr) = mr_endpoint.query_async(&client).await {
                    current_mr = full_mr;
                    let _ = tx.send(DataUpdate::MRDetail(current_mr.clone()));
                }
            }

            let refs = if let Some(r) = &current_mr.diff_refs { r.clone() } else {
                let _ = tx.send(DataUpdate::Error("Could not retrieve MR diff refs (SHAs)".into()));
                let _ = tx.send(DataUpdate::Loading(false));
                return;
            };

            let base_sha = refs.base_sha;
            let head_sha = refs.head_sha;
            
            let old_endpoint = gitlab::api::projects::repository::files::File::builder().project(project_id).file_path(old_path.clone()).ref_(base_sha).build().unwrap();
            let new_endpoint = gitlab::api::projects::repository::files::File::builder().project(project_id).file_path(new_path.clone()).ref_(head_sha).build().unwrap();
            
            let old_res: Result<crate::models::RepositoryFile, _> = gitlab::api::AsyncQuery::query_async(&old_endpoint, &client).await;
            let new_res: Result<crate::models::RepositoryFile, _> = gitlab::api::AsyncQuery::query_async(&new_endpoint, &client).await;
            
            use base64::{Engine as _, engine::general_purpose};
            
            let old_text = if let Ok(f) = old_res {
                let clean = f.content.replace(['\n', '\r'], "");
                if let Ok(dec) = general_purpose::STANDARD.decode(&clean) { String::from_utf8_lossy(&dec).into_owned() } else { String::new() }
            } else { String::new() };
            
            let new_text = if let Ok(f) = new_res {
                let clean = f.content.replace(['\n', '\r'], "");
                if let Ok(dec) = general_purpose::STANDARD.decode(&clean) { String::from_utf8_lossy(&dec).into_owned() } else { String::new() }
            } else { String::new() };
            
            let diff = similar::TextDiff::from_lines(&old_text, &new_text).unified_diff().to_string();
            
            let _ = tx.send(DataUpdate::SingleDiffFetched { project_id, iid, old_path, new_path, diff });
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_all_missing_diffs(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid) = if let Some(mr) = &self.selected_mr { (mr.project_id, mr.iid) } else { return Ok(()); };
        let missing: Vec<(String, String)> = self.mr_diffs.iter().filter(|d| d.collapsed && d.diff.is_empty()).map(|d| (d.old_path.clone(), d.new_path.clone())).collect();
        
        if missing.is_empty() { return Ok(()); }
        
        for (old, new) in missing {
            let _ = self.fetch_single_file_diff(pid, iid, old, new).await;
        }
        Ok(())
    }
    pub async fn post_new_comment(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let body = self.textarea.lines().join("\n");
        if body.is_empty() { return Ok(()); }
        let (pid, iid) = if let Some(mr) = &self.selected_mr { (mr.project_id, mr.iid) } else { return Ok(()); };
        let mr = self.selected_mr.as_ref().unwrap().clone();
        
        let mut success = false;

        if self.input_target == InputTarget::NewFileComment {
            if let Some(diff_idx) = self.mr_diff_state.selected()
                && let Some(mr_diff) = self.mr_diffs.get(diff_idx)
            {
                let refs = if let Some(refs) = &mr.diff_refs { refs.clone() } else { return Ok(()); };
                let client = self.gitlab.clone(); let tx = self.tx.clone(); let _ = tx.send(DataUpdate::Loading(true));
                let new_path = Some(mr_diff.new_path.clone());
                let old_path = Some(mr_diff.old_path.clone());

                tokio::spawn(async move {
                    let endpoint = CreateMergeRequestDiscussion {
                        project: pid, merge_request: iid, body,
                        base_sha: Some(refs.base_sha.clone()),
                        start_sha: Some(refs.start_sha.clone()),
                        head_sha: Some(refs.head_sha.clone()),
                        new_path, old_path,
                        new_line: None, old_line: None,
                        range_start_new: None,
                        range_start_old: None,
                    };
                    let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
                    match res {
                        Ok(Ok(_)) => {
                            let mr_endpoint = SingleMergeRequest { project: pid, merge_request: iid };
                            let disc_endpoint = MergeRequestDiscussions { project: pid, merge_request: iid };
                            let (mr_res, disc_res) = tokio::join!(
                                tokio::time::timeout(std::time::Duration::from_secs(30), mr_endpoint.query_async(&client)),
                                tokio::time::timeout(std::time::Duration::from_secs(30), disc_endpoint.query_async(&client))
                            );
                            if let Ok(Ok(val)) = mr_res 
                                && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) 
                            {
                                let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                            }
                            if let Ok(Ok(discs)) = disc_res {
                                let _ = tx.send(DataUpdate::Discussions { project_id: pid, iid, data: discs });
                            }
                        }
                        Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Post comment failed: {}", e))); }
                        Err(_) => { let _ = tx.send(DataUpdate::Error("Post comment timed out".into())); }
                    }
                    let _ = tx.send(DataUpdate::Loading(false));
                });
                success = true;
            }
        } else {
            if let Some(diff_idx) = self.mr_diff_state.selected()
                && let Some(mr_diff) = self.mr_diffs.get(diff_idx)
                && let Some(line_idx) = self.diff_line_state.selected()
            {
                let start_idx = self.diff_selection_start.unwrap_or(line_idx).min(line_idx);
                let end_idx = self.diff_selection_start.unwrap_or(line_idx).max(line_idx);

                if let (Some(DiffDisplayLine::Code(_, old_start, new_start)), Some(DiffDisplayLine::Code(_, old_end, new_end))) = 
                    (mr_diff.display_lines.get(start_idx), mr_diff.display_lines.get(end_idx)) 
                {
                    let new_line = new_end.filter(|&l| l > 0);
                    let old_line = old_end.filter(|&l| l > 0);
                    let rs_new = if start_idx != end_idx { new_start.filter(|&l| l > 0) } else { None };
                    let rs_old = if start_idx != end_idx { old_start.filter(|&l| l > 0) } else { None };

                    if new_line.is_some() || old_line.is_some() {
                        let refs = if let Some(refs) = &mr.diff_refs { refs.clone() } else { return Ok(()); };
                        let client = self.gitlab.clone(); let tx = self.tx.clone(); let _ = tx.send(DataUpdate::Loading(true));
                        let new_path = Some(mr_diff.new_path.clone());
                        let old_path = Some(mr_diff.old_path.clone());

                        tokio::spawn(async move {
                            let endpoint = CreateMergeRequestDiscussion {
                                project: pid, merge_request: iid, body,
                                base_sha: Some(refs.base_sha.clone()),
                                start_sha: Some(refs.start_sha.clone()),
                                head_sha: Some(refs.head_sha.clone()),
                                new_path, old_path, new_line, old_line,
                                range_start_new: rs_new,
                                range_start_old: rs_old,
                            };
                            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
                            match res {
                                Ok(Ok(_)) => {
                                    let mr_endpoint = SingleMergeRequest { project: pid, merge_request: iid };
                                    let disc_endpoint = MergeRequestDiscussions { project: pid, merge_request: iid };
                                    let (mr_res, disc_res) = tokio::join!(
                                        tokio::time::timeout(std::time::Duration::from_secs(30), mr_endpoint.query_async(&client)),
                                        tokio::time::timeout(std::time::Duration::from_secs(30), disc_endpoint.query_async(&client))
                                    );
                                    if let Ok(Ok(val)) = mr_res 
                                        && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) 
                                    {
                                        let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                                    }
                                    if let Ok(Ok(discs)) = disc_res {
                                        let _ = tx.send(DataUpdate::Discussions { project_id: pid, iid, data: discs });
                                    }
                                }
                                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Post comment failed: {}", e))); }
                                Err(_) => { let _ = tx.send(DataUpdate::Error("Post comment timed out".into())); }
                            }
                            let _ = tx.send(DataUpdate::Loading(false));
                        });
                        success = true;
                    }
                }
            }
        }

        if !success {
            self.last_error = Some("Cannot comment (no file/line selected or invalid context)".into());
        }
        self.input_mode = false;
        self.textarea = ratatui_textarea::TextArea::default();
        self.diff_selection_start = None;
        Ok(())
    }

    pub async fn approve_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id; let iid = mr.iid; let client = self.gitlab.clone(); let tx = self.tx.clone(); let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = ApproveMergeRequest { project: pid, merge_request: iid };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
                match res {
                    Ok(Ok(_)) => {
                        let mr_endpoint = SingleMergeRequest { project: pid, merge_request: iid };
                        if let Ok(Ok(val)) = tokio::time::timeout(std::time::Duration::from_secs(30), mr_endpoint.query_async(&client)).await 
                            && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) 
                        {
                            let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                        }
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Approve MR failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Approve MR timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn unapprove_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id; let iid = mr.iid; let client = self.gitlab.clone(); let tx = self.tx.clone(); let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = UnapproveMergeRequest { project: pid, merge_request: iid };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
                match res {
                    Ok(Ok(_)) => {
                        let mr_endpoint = SingleMergeRequest { project: pid, merge_request: iid };
                        if let Ok(Ok(val)) = tokio::time::timeout(std::time::Duration::from_secs(30), mr_endpoint.query_async(&client)).await 
                            && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) 
                        {
                            let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                        }
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Unapprove MR failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Unapprove MR timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn merge_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id; let iid = mr.iid; let squash = mr.squash.unwrap_or(false); let client = self.gitlab.clone(); let tx = self.tx.clone(); let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = AcceptMergeRequest { project: pid, merge_request: iid, squash, should_remove_source_branch: true };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
                match res {
                    Ok(Ok(_)) => {
                        let mr_endpoint = SingleMergeRequest { project: pid, merge_request: iid };
                        if let Ok(Ok(val)) = tokio::time::timeout(std::time::Duration::from_secs(30), mr_endpoint.query_async(&client)).await 
                            && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) 
                        {
                            let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                        }
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Merge MR failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Merge MR timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn toggle_mr_draft(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid, current_draft, current_title) = if let Some(mr) = &self.selected_mr { 
            (mr.project_id, mr.iid, mr.draft.unwrap_or(false), mr.title.clone()) 
        } else { 
            return Ok(()); 
        };
        let client = self.gitlab.clone(); let tx = self.tx.clone();
        if let Some(m) = &mut self.selected_mr { 
            m.draft = Some(!current_draft); 
            let updated = m.clone();
            if let Some(existing) = self.merge_requests.iter_mut().find(|x| x.iid == updated.iid && x.project_id == updated.project_id) {
                *existing = updated;
                self.filter_lists();
            }
        }
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let edit = EditMergeRequest { project: pid, merge_request: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: None, reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: None, target_branch: None, draft: Some(!current_draft), wip: None };
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
            match res { 
                Ok(Ok(val)) => { 
                    if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) {
                        let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                    }
                } 
                Ok(Err(e)) => {
                    let err_str = e.to_string().to_lowercase();
                    if err_str.contains("400") || err_str.contains("bad request") {
                        let retry_edit = EditMergeRequest { project: pid, merge_request: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: None, reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: None, target_branch: None, draft: None, wip: Some(!current_draft) };
                        let retry_res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), retry_edit.query_async(&client)).await;
                        match retry_res {
                            Ok(Ok(val)) => {
                                if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) {
                                    let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                                }
                            }
                            Ok(Err(e2)) => {
                                let err_str2 = e2.to_string().to_lowercase();
                                if err_str2.contains("400") || err_str2.contains("bad request") {
                                    let new_title = toggle_wip_title(&current_title, !current_draft);
                                    let title_edit = EditMergeRequest { project: pid, merge_request: iid, title: Some(new_title), description: None, state_event: None, labels: None, assignee_ids: None, reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: None, target_branch: None, draft: None, wip: None };
                                    let title_res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), title_edit.query_async(&client)).await;
                                    match title_res {
                                        Ok(Ok(val)) => {
                                            if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) {
                                                let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                                            }
                                        }
                                        Ok(Err(e3)) => {
                                            let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e3)));
                                        }
                                        Err(_) => {
                                            let _ = tx.send(DataUpdate::Error("MR update timed out".into()));
                                        }
                                    }
                                } else {
                                    let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e2)));
                                }
                            }
                            Err(_) => {
                                let _ = tx.send(DataUpdate::Error("MR update timed out".into()));
                            }
                        }
                    } else {
                        let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e)));
                    }
                } 
                Err(_) => { let _ = tx.send(DataUpdate::Error("MR update timed out".into())); } 
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn toggle_mr_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid, current_state) = if let Some(mr) = &self.selected_mr { (mr.project_id, mr.iid, mr.state.clone()) } else { return Ok(()); };
        let client = self.gitlab.clone(); let tx = self.tx.clone(); let event = if current_state == "opened" { "close" } else { "reopen" };
        if let Some(m) = &mut self.selected_mr { 
            m.state = if current_state == "opened" { "closed".to_string() } else { "opened".to_string() }; 
            let updated = m.clone();
            if let Some(existing) = self.merge_requests.iter_mut().find(|x| x.iid == updated.iid && x.project_id == updated.project_id) {
                *existing = updated;
                self.filter_lists();
            }
        }
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let edit = EditMergeRequest { project: pid, merge_request: iid, title: None, description: None, state_event: Some(event.to_string()), labels: None, assignee_ids: None, reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: None, target_branch: None, draft: None, wip: None };
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
            match res { 
                Ok(Ok(val)) => { 
                    if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) { 
                        let _ = tx.send(DataUpdate::MRDetail(updated_mr)); 
                    } 
                } 
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e))); } 
                Err(_) => { let _ = tx.send(DataUpdate::Error("Toggle MR state timed out".into())); } 
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn update_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid) = if let Some(mr) = &self.selected_mr { (mr.project_id, mr.iid) } else { return Ok(()); };
        let client = self.gitlab.clone(); let tx = self.tx.clone();
        let mut edit = EditMergeRequest { project: pid, merge_request: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: None, reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: None, target_branch: None, draft: None, wip: None };
        let body = self.textarea.lines().join("\n");
        match self.input_target {
            InputTarget::EditTitle => { 
                edit.title = Some(body.clone()); 
                if let Some(m) = &mut self.selected_mr { 
                    m.title = body.clone(); 
                    let updated = m.clone();
                    if let Some(existing) = self.merge_requests.iter_mut().find(|x| x.iid == updated.iid && x.project_id == updated.project_id) {
                        *existing = updated;
                        self.filter_lists();
                    }
                } 
            }
            InputTarget::EditDescription => { 
                edit.description = Some(body.clone()); 
                if let Some(m) = &mut self.selected_mr { 
                    m.description = Some(body.clone()); 
                    let updated = m.clone();
                    if let Some(existing) = self.merge_requests.iter_mut().find(|x| x.iid == updated.iid && x.project_id == updated.project_id) {
                        *existing = updated;
                        self.filter_lists();
                    }
                } 
            }
            _ => return Ok(()),
        }
        self.input_mode = false; self.textarea = ratatui_textarea::TextArea::default(); let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
            match res { 
                Ok(Ok(val)) => { 
                    if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) { 
                        let _ = tx.send(DataUpdate::MRDetail(updated_mr)); 
                    } 
                } 
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e))); } 
                Err(_) => { let _ = tx.send(DataUpdate::Error("Update MR timed out".into())); } 
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn update_mr_labels(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &mut self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let updated_labels: Vec<Label> = {
                let labels_list = self.project_label_cache.get(&pid).unwrap_or(&self.project_labels);
                labels_list.iter().filter(|l| self.selected_label_ids.contains(&l.id())).cloned().collect()
            };
            let labels = updated_labels.iter().map(|l| l.name().to_string()).collect::<Vec<_>>().join(",");

            mr.labels = updated_labels.clone();
            if let Some(idx) = self.merge_requests.iter_mut().position(|m| m.iid == iid && m.project_id == pid) {
                self.merge_requests[idx].labels = updated_labels;
            }
            self.filter_lists();

            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            self.show_label_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditMergeRequest { project: pid, merge_request: iid, title: None, description: None, state_event: None, labels: Some(labels), assignee_ids: None, reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: None, target_branch: None, draft: None, wip: None };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res { Ok(Ok(val)) => { if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) { let _ = tx.send(DataUpdate::MRDetail(updated_mr)); } let _ = tx.send(DataUpdate::TriggerRefresh); } Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e))); } Err(_) => { let _ = tx.send(DataUpdate::Error("Update MR labels timed out".into())); } }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_mr_assignees(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &mut self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let members = self.project_member_cache.get(&pid).unwrap_or(&self.project_members);
            let updated_assignees: Vec<Author> = members.iter().filter(|m| self.selected_assignee_ids.contains(&m.id)).cloned().collect();
            mr.assignees = updated_assignees.clone();
            if let Some(idx) = self.merge_requests.iter_mut().position(|m| m.iid == iid && m.project_id == pid) {
                self.merge_requests[idx].assignees = updated_assignees;
            }
            self.filter_lists();

            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let assignee_ids = self.selected_assignee_ids.clone();
            self.show_assignee_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditMergeRequest { project: pid, merge_request: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: Some(assignee_ids), reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: None, target_branch: None, draft: None, wip: None };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res { Ok(Ok(val)) => { if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) { let _ = tx.send(DataUpdate::MRDetail(updated_mr)); } let _ = tx.send(DataUpdate::TriggerRefresh); } Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e))); } Err(_) => { let _ = tx.send(DataUpdate::Error("Update MR assignees timed out".into())); } }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_mr_reviewers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &mut self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let members = self.project_member_cache.get(&pid).unwrap_or(&self.project_members);
            let updated_reviewers: Vec<Author> = members.iter().filter(|m| self.selected_reviewer_ids.contains(&m.id)).cloned().collect();
            mr.reviewers = updated_reviewers.clone();
            if let Some(idx) = self.merge_requests.iter_mut().position(|m| m.iid == iid && m.project_id == pid) {
                self.merge_requests[idx].reviewers = updated_reviewers;
            }
            self.filter_lists();

            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let reviewer_ids = self.selected_reviewer_ids.clone();
            self.show_reviewer_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditMergeRequest { project: pid, merge_request: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: None, reviewer_ids: Some(reviewer_ids), milestone_id: None, remove_source_branch: None, squash: None, target_branch: None, draft: None, wip: None };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res { Ok(Ok(val)) => { if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) { let _ = tx.send(DataUpdate::MRDetail(updated_mr)); } let _ = tx.send(DataUpdate::TriggerRefresh); } Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e))); } Err(_) => { let _ = tx.send(DataUpdate::Error("Update MR reviewers timed out".into())); } }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_mr_milestone(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &mut self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let mid = if let Some(i) = self.milestone_state.selected() { if i == 0 { None } else { Some(self.project_milestones[i - 1].id) } } else { None };
            let milestones = self.project_milestone_cache.get(&pid).unwrap_or(&self.project_milestones);
            let updated_milestone = mid.and_then(|id| milestones.iter().find(|m| m.id == id).cloned());
            mr.milestone = updated_milestone.clone();
            if let Some(idx) = self.merge_requests.iter_mut().position(|m| m.iid == iid && m.project_id == pid) {
                self.merge_requests[idx].milestone = updated_milestone;
            }
            self.filter_lists();

            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            self.show_milestone_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditMergeRequest { project: pid, merge_request: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: None, reviewer_ids: None, milestone_id: Some(mid), remove_source_branch: None, squash: None, target_branch: None, draft: None, wip: None };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                match res { Ok(Ok(val)) => { if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) { let _ = tx.send(DataUpdate::MRDetail(updated_mr)); } let _ = tx.send(DataUpdate::TriggerRefresh); } Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e))); } Err(_) => { let _ = tx.send(DataUpdate::Error("Update MR milestone timed out".into())); } }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_mr_target_branch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &mut self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let branch_name = if let Some(i) = self.target_branch_state.selected() {
                if i < self.project_branches.len() {
                    Some(self.project_branches[i].name.clone())
                } else {
                    None
                }
            } else {
                None
            };
            self.show_target_branch_popup = false;
            if let Some(branch) = branch_name {
                mr.target_branch = branch.clone();
                if let Some(idx) = self.merge_requests.iter_mut().position(|m| m.iid == iid && m.project_id == pid) {
                    self.merge_requests[idx].target_branch = branch.clone();
                }
                self.mr_diffs.clear();
                self.mr_diff_state = ratatui::widgets::ListState::default();
                self.filter_lists();

                let client = self.gitlab.clone();
                let tx = self.tx.clone();
                let _ = tx.send(DataUpdate::Loading(true));
                tokio::spawn(async move {
                    let edit = EditMergeRequest { project: pid, merge_request: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: None, reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: None, target_branch: Some(branch), draft: None, wip: None };
                    let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
                    match res {
                        Ok(Ok(val)) => {
                            if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) {
                                let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                            }
                            let _ = tx.send(DataUpdate::TriggerRefresh);
                        }
                        Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e))); }
                        Err(_) => { let _ = tx.send(DataUpdate::Error("Update MR target branch timed out".into())); }
                    }
                    let _ = tx.send(DataUpdate::Loading(false));
                });
            }
        }
        Ok(())
    }

    pub async fn toggle_mr_squash(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid, current_squash) = if let Some(mr) = &self.selected_mr { (mr.project_id, mr.iid, mr.squash.unwrap_or(false)) } else { return Ok(()); };
        let client = self.gitlab.clone(); let tx = self.tx.clone();
        if let Some(m) = &mut self.selected_mr { 
            m.squash = Some(!current_squash); 
            let updated = m.clone();
            if let Some(existing) = self.merge_requests.iter_mut().find(|x| x.iid == updated.iid && x.project_id == updated.project_id) {
                *existing = updated;
                self.filter_lists();
            }
        }
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let edit = EditMergeRequest { project: pid, merge_request: iid, title: None, description: None, state_event: None, labels: None, assignee_ids: None, reviewer_ids: None, milestone_id: None, remove_source_branch: None, squash: Some(!current_squash), target_branch: None, draft: None, wip: None };
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), edit.query_async(&client)).await;
            match res { 
                Ok(Ok(val)) => { 
                    if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) {
                        let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                    }
                } 
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e))); } 
                Err(_) => { let _ = tx.send(DataUpdate::Error("MR update timed out".into())); } 
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn rebase_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id; let iid = mr.iid; let client = self.gitlab.clone(); let tx = self.tx.clone(); let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = RebaseMergeRequest { project: pid, merge_request: iid };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
                match res {
                    Ok(Ok(_)) => {
                        let mr_endpoint = SingleMergeRequest { project: pid, merge_request: iid };
                        if let Ok(Ok(val)) = tokio::time::timeout(std::time::Duration::from_secs(30), mr_endpoint.query_async(&client)).await 
                            && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) 
                        {
                            let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                        }
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Rebase MR failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Rebase MR timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn resolve_discussion(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr
            && let Some(idx) = self.discussion_state.selected()
            && let Some(disc) = self.discussions.get(idx)
        {
                    let pid = mr.project_id; let iid = mr.iid; let disc_id = disc.id.clone(); let current_resolved = disc.resolved.unwrap_or(false);
                    let client = self.gitlab.clone(); let tx = self.tx.clone(); let _ = tx.send(DataUpdate::Loading(true));
                    tokio::spawn(async move {
                        let endpoint = ResolveDiscussion { project: pid, merge_request: iid, discussion: disc_id, resolved: !current_resolved };
                        let res: Result<serde_json::Value, _> = endpoint.query_async(&client).await;
                        if res.is_ok() {
                            let mr_endpoint = SingleMergeRequest { project: pid, merge_request: iid };
                            let disc_endpoint = MergeRequestDiscussions { project: pid, merge_request: iid };
                            let (mr_res, disc_res) = tokio::join!(
                                tokio::time::timeout(std::time::Duration::from_secs(30), mr_endpoint.query_async(&client)),
                                tokio::time::timeout(std::time::Duration::from_secs(30), disc_endpoint.query_async(&client))
                            );
                            if let Ok(Ok(val)) = mr_res 
                                && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) 
                            {
                                let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                            }
                            if let Ok(Ok(discs)) = disc_res {
                                let _ = tx.send(DataUpdate::Discussions { project_id: pid, iid, data: discs });
                            }
                        }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
        }
        Ok(())
    }

    pub async fn create_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let title = self.create_mr_title.clone();
            let source_branch = self.create_mr_source_branch.clone();
            let target_branch = self.create_mr_target_branch.clone();
            let description = if self.create_mr_description.is_empty() { None } else { Some(self.create_mr_description.clone()) };
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = CreateMergeRequest { project: pid, source_branch, target_branch, title, description };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
                match res {
                    Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("MR creation failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("MR creation timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
            self.show_create_mr_popup = false;
            self.input_mode = false;
        }
        Ok(())
    }
}

fn strip_draft_prefix(title: &str) -> String {
    let mut t = title.trim();
    loop {
        let lower = t.to_lowercase();
        if lower.starts_with("draft:") {
            t = t["draft:".len()..].trim_start();
        } else if lower.starts_with("draft -") {
            t = t["draft -".len()..].trim_start();
        } else if lower.starts_with("draft") {
            let rest = &t["draft".len()..];
            if rest.is_empty() || rest.chars().next().map(|c| c.is_whitespace() || c.is_ascii_punctuation()).unwrap_or(false) {
                t = rest.trim_start();
            } else {
                break;
            }
        } else if lower.starts_with("wip:") {
            t = t["wip:".len()..].trim_start();
        } else if lower.starts_with("wip -") {
            t = t["wip -".len()..].trim_start();
        } else if lower.starts_with("wip") {
            let rest = &t["wip".len()..];
            if rest.is_empty() || rest.chars().next().map(|c| c.is_whitespace() || c.is_ascii_punctuation()).unwrap_or(false) {
                t = rest.trim_start();
            } else {
                break;
            }
        } else if lower.starts_with("[draft]") {
            t = t["[draft]".len()..].trim_start();
        } else if lower.starts_with("[wip]") {
            t = t["[wip]".len()..].trim_start();
        } else if lower.starts_with("(draft)") {
            t = t["(draft)".len()..].trim_start();
        } else if lower.starts_with("(wip)") {
            t = t["(wip)".len()..].trim_start();
        } else {
            break;
        }
    }
    t.to_string()
}

fn toggle_wip_title(title: &str, make_draft: bool) -> String {
    let clean = strip_draft_prefix(title);
    if make_draft {
        format!("Draft: {}", clean)
    } else {
        clean
    }
}

pub fn parse_diff_context(diff: &str, new_line: Option<u64>, old_line: Option<u64>) -> Option<Vec<(u64, String, LineType, bool)>> {
    let target = new_line.or(old_line)?;
    let mut all_diff_lines = Vec::new();
    let mut current_old_line = 0; let mut current_new_line = 0;
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
        let (ltype, ln) = if line.starts_with('+') { let l = (LineType::Addition, current_new_line); current_new_line += 1; l }
        else if line.starts_with('-') { let l = (LineType::Removal, current_old_line); current_old_line += 1; l }
        else { let l = (LineType::Unchanged, if new_line.is_some() { current_new_line } else { current_old_line }); current_old_line += 1; current_new_line += 1; l };
        all_diff_lines.push((ln, line.to_string(), ltype));
    }
    if let Some(target_idx) = all_diff_lines.iter().position(|(ln, _, lt)| *ln == target && ((new_line.is_some() && *lt == LineType::Addition) || (old_line.is_some() && *lt == LineType::Removal) || (*lt == LineType::Unchanged))) {
        let start = target_idx.saturating_sub(3); let end = (target_idx + 4).min(all_diff_lines.len());
        let mut context = Vec::new();
        for (i, (ln, content, ltype)) in all_diff_lines[start..end].iter().enumerate() {
            let is_target = (start + i) == target_idx;
            let clean_content = if content.starts_with('+') || content.starts_with('-') || content.starts_with(' ') { &content[1..] } else { content };
            context.push((*ln, clean_content.to_string(), ltype.clone(), is_target));
        }
        Some(context)
    } else { None }
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
        let mut app = App::new(client, tx);
        app.active_panel = Panel::Project; app.project_menu_state.select(Some(1)); app.active_context = Context::Project; app.active_selection = MenuSelection::MergeRequests; app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()), default_branch: None, namespace: None });
        (server, app, rx)
    }

    #[tokio::test]
    async fn test_mr_fetching_logic_exhaustive() {
        let (server, mut app, mut rx) = setup_app().await;
        let mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "t", "reactions": [], "draft": false}"##;
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
        app.selected_mr = Some(app.merge_requests[0].clone()); app.fetch_discussions(1, 1).await.unwrap();
        let _ = tokio::time::timeout(Duration::from_secs(2), async { while app.is_loading { app.process_updates(&mut rx).await; tokio::time::sleep(Duration::from_millis(10)).await; } }).await;
        app.approve_mr().await.unwrap(); app.unapprove_mr().await.unwrap(); app.merge_mr().await.unwrap(); app.toggle_mr_state().await.unwrap(); app.input_target = InputTarget::EditTitle; app.textarea = ratatui_textarea::TextArea::from(vec!["new"]); app.update_mr().await.unwrap(); app.project_labels = vec![Label { id: 1, name: "l".into(), color: "c".into(), text_color: Some("t".into()) }]; app.selected_label_ids.push(1); app.update_mr_labels().await.unwrap(); app.selected_assignee_ids.push(1); app.update_mr_assignees().await.unwrap(); app.project_milestones = vec![Milestone { id: 1, title: "m".into() }]; app.milestone_state.select(Some(1)); app.update_mr_milestone().await.unwrap(); app.resolve_discussion().await.unwrap();
    }

    #[test]
    fn test_draft_prefix_manipulation() {
        assert_eq!(strip_draft_prefix("My MR"), "My MR");
        assert_eq!(strip_draft_prefix("Draft: My MR"), "My MR");
        assert_eq!(strip_draft_prefix("WIP: My MR"), "My MR");
        assert_eq!(strip_draft_prefix("wip - My MR"), "My MR");
        assert_eq!(strip_draft_prefix("Drafting new feature"), "Drafting new feature");
        assert_eq!(strip_draft_prefix("[Draft] My MR"), "My MR");
        assert_eq!(strip_draft_prefix("(WIP) My MR"), "My MR");

        assert_eq!(toggle_wip_title("My MR", true), "Draft: My MR");
        assert_eq!(toggle_wip_title("Draft: My MR", true), "Draft: My MR");
        assert_eq!(toggle_wip_title("Draft: My MR", false), "My MR");
        assert_eq!(toggle_wip_title("WIP: My MR", false), "My MR");
    }
}
