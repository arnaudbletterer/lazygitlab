use crate::api::*;
use crate::models::*;
use gitlab::api::AsyncQuery;

impl App {
    pub async fn post_new_comment(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let body = self.textarea.lines().join("\n");
        if body.is_empty() {
            return Ok(());
        }
        let (pid, iid) = if let Some(mr) = &self.selected_mr {
            (mr.project_id, mr.iid)
        } else {
            return Ok(());
        };
        let mr = self.selected_mr.as_ref().unwrap().clone();

        let mut success = false;

        if self.input_target == InputTarget::NewFileComment {
            if let Some(diff_idx) = self.mr_diff_state.selected()
                && let Some(mr_diff) = self.mr_diffs.get(diff_idx)
            {
                let refs = if let Some(refs) = &mr.diff_refs {
                    refs.clone()
                } else {
                    return Ok(());
                };
                let client = self.gitlab.clone();
                let tx = self.tx.clone();
                let _ = tx.send(DataUpdate::Loading(true));
                let new_path = Some(mr_diff.new_path.clone());
                let old_path = Some(mr_diff.old_path.clone());

                tokio::spawn(async move {
                    let endpoint = CreateMergeRequestDiscussion {
                        project: pid,
                        merge_request: iid,
                        body,
                        base_sha: Some(refs.base_sha.clone()),
                        start_sha: Some(refs.start_sha.clone()),
                        head_sha: Some(refs.head_sha.clone()),
                        new_path,
                        old_path,
                        new_line: None,
                        old_line: None,
                        range_start_new: None,
                        range_start_old: None,
                    };
                    let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                        std::time::Duration::from_secs(30),
                        endpoint.query_async(&client),
                    )
                    .await;
                    match res {
                        Ok(Ok(_)) => {
                            let mr_endpoint = SingleMergeRequest {
                                project: pid,
                                merge_request: iid,
                            };
                            let disc_endpoint = MergeRequestDiscussions {
                                project: pid,
                                merge_request: iid,
                            };
                            let (mr_res, disc_res) = tokio::join!(
                                tokio::time::timeout(
                                    std::time::Duration::from_secs(30),
                                    mr_endpoint.query_async(&client)
                                ),
                                tokio::time::timeout(
                                    std::time::Duration::from_secs(30),
                                    disc_endpoint.query_async(&client)
                                )
                            );
                            if let Ok(Ok(val)) = mr_res
                                && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val)
                            {
                                let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                            }
                            if let Ok(Ok(discs)) = disc_res {
                                let _ = tx.send(DataUpdate::Discussions {
                                    project_id: pid,
                                    iid,
                                    data: discs,
                                });
                            }
                        }
                        Ok(Err(e)) => {
                            let _ =
                                tx.send(DataUpdate::Error(format!("Post comment failed: {}", e)));
                        }
                        Err(_) => {
                            let _ = tx.send(DataUpdate::Error("Post comment timed out".into()));
                        }
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

                if let (
                    Some(DiffDisplayLine::Code(_, old_start, new_start)),
                    Some(DiffDisplayLine::Code(_, old_end, new_end)),
                ) = (
                    mr_diff.display_lines.get(start_idx),
                    mr_diff.display_lines.get(end_idx),
                ) {
                    let new_line = new_end.filter(|&l| l > 0);
                    let old_line = old_end.filter(|&l| l > 0);
                    let rs_new = if start_idx != end_idx {
                        new_start.filter(|&l| l > 0)
                    } else {
                        None
                    };
                    let rs_old = if start_idx != end_idx {
                        old_start.filter(|&l| l > 0)
                    } else {
                        None
                    };

                    if new_line.is_some() || old_line.is_some() {
                        let refs = if let Some(refs) = &mr.diff_refs {
                            refs.clone()
                        } else {
                            return Ok(());
                        };
                        let client = self.gitlab.clone();
                        let tx = self.tx.clone();
                        let _ = tx.send(DataUpdate::Loading(true));
                        let new_path = Some(mr_diff.new_path.clone());
                        let old_path = Some(mr_diff.old_path.clone());

                        tokio::spawn(async move {
                            let endpoint = CreateMergeRequestDiscussion {
                                project: pid,
                                merge_request: iid,
                                body,
                                base_sha: Some(refs.base_sha.clone()),
                                start_sha: Some(refs.start_sha.clone()),
                                head_sha: Some(refs.head_sha.clone()),
                                new_path,
                                old_path,
                                new_line,
                                old_line,
                                range_start_new: rs_new,
                                range_start_old: rs_old,
                            };
                            let res: Result<Result<serde_json::Value, _>, _> =
                                tokio::time::timeout(
                                    std::time::Duration::from_secs(30),
                                    endpoint.query_async(&client),
                                )
                                .await;
                            match res {
                                Ok(Ok(_)) => {
                                    let mr_endpoint = SingleMergeRequest {
                                        project: pid,
                                        merge_request: iid,
                                    };
                                    let disc_endpoint = MergeRequestDiscussions {
                                        project: pid,
                                        merge_request: iid,
                                    };
                                    let (mr_res, disc_res) = tokio::join!(
                                        tokio::time::timeout(
                                            std::time::Duration::from_secs(30),
                                            mr_endpoint.query_async(&client)
                                        ),
                                        tokio::time::timeout(
                                            std::time::Duration::from_secs(30),
                                            disc_endpoint.query_async(&client)
                                        )
                                    );
                                    if let Ok(Ok(val)) = mr_res
                                        && let Ok(updated_mr) =
                                            serde_json::from_value::<MergeRequest>(val)
                                    {
                                        let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                                    }
                                    if let Ok(Ok(discs)) = disc_res {
                                        let _ = tx.send(DataUpdate::Discussions {
                                            project_id: pid,
                                            iid,
                                            data: discs,
                                        });
                                    }
                                }
                                Ok(Err(e)) => {
                                    let _ = tx.send(DataUpdate::Error(format!(
                                        "Post comment failed: {}",
                                        e
                                    )));
                                }
                                Err(_) => {
                                    let _ =
                                        tx.send(DataUpdate::Error("Post comment timed out".into()));
                                }
                            }
                            let _ = tx.send(DataUpdate::Loading(false));
                        });
                        success = true;
                    }
                }
            }
        }

        if !success {
            self.last_error =
                Some("Cannot comment (no file/line selected or invalid context)".into());
        }
        self.input_mode = false;
        self.textarea = ratatui_textarea::TextArea::default();
        self.diff_selection_start = None;
        Ok(())
    }

    pub async fn approve_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            self.status_message = Some(format!("Approving MR !{}...", iid));
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = ApproveMergeRequest {
                    project: pid,
                    merge_request: iid,
                };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    endpoint.query_async(&client),
                )
                .await;
                match res {
                    Ok(Ok(_)) => {
                        let mr_endpoint = SingleMergeRequest {
                            project: pid,
                            merge_request: iid,
                        };
                        if let Ok(Ok(val)) = tokio::time::timeout(
                            std::time::Duration::from_secs(30),
                            mr_endpoint.query_async(&client),
                        )
                        .await
                            && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val)
                        {
                            let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                        }
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!("Approve MR failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Approve MR timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn unapprove_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            self.status_message = Some(format!("Unapproving MR !{}...", iid));
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = UnapproveMergeRequest {
                    project: pid,
                    merge_request: iid,
                };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    endpoint.query_async(&client),
                )
                .await;
                match res {
                    Ok(Ok(_)) => {
                        let mr_endpoint = SingleMergeRequest {
                            project: pid,
                            merge_request: iid,
                        };
                        if let Ok(Ok(val)) = tokio::time::timeout(
                            std::time::Duration::from_secs(30),
                            mr_endpoint.query_async(&client),
                        )
                        .await
                            && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val)
                        {
                            let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                        }
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!("Unapprove MR failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Unapprove MR timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn merge_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let squash = mr.squash.unwrap_or(false);
            self.status_message = Some(format!("Merging MR !{}...", iid));
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = AcceptMergeRequest {
                    project: pid,
                    merge_request: iid,
                    squash,
                    should_remove_source_branch: true,
                };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    endpoint.query_async(&client),
                )
                .await;
                match res {
                    Ok(Ok(_)) => {
                        let mr_endpoint = SingleMergeRequest {
                            project: pid,
                            merge_request: iid,
                        };
                        if let Ok(Ok(val)) = tokio::time::timeout(
                            std::time::Duration::from_secs(30),
                            mr_endpoint.query_async(&client),
                        )
                        .await
                            && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val)
                        {
                            let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                        }
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!("Merge MR failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Merge MR timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn toggle_mr_draft(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid, current_draft, current_title) = if let Some(mr) = &self.selected_mr {
            (
                mr.project_id,
                mr.iid,
                mr.draft.unwrap_or(false),
                mr.title.clone(),
            )
        } else {
            return Ok(());
        };
        self.status_message = Some(if current_draft {
            format!("Marking MR !{} as ready...", iid)
        } else {
            format!("Marking MR !{} as draft...", iid)
        });
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        if let Some(m) = &mut self.selected_mr {
            m.draft = Some(!current_draft);
            let updated = m.clone();
            if let Some(existing) = self
                .merge_requests
                .iter_mut()
                .find(|x| x.iid == updated.iid && x.project_id == updated.project_id)
            {
                *existing = updated;
                self.filter_lists();
            }
        }
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let edit = EditMergeRequest {
                project: pid,
                merge_request: iid,
                title: None,
                description: None,
                state_event: None,
                labels: None,
                assignee_ids: None,
                reviewer_ids: None,
                milestone_id: None,
                remove_source_branch: None,
                squash: None,
                target_branch: None,
                draft: Some(!current_draft),
                wip: None,
            };
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                edit.query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(val)) => {
                    if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) {
                        let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                    }
                }
                Ok(Err(e)) => {
                    let err_str = e.to_string().to_lowercase();
                    if err_str.contains("400") || err_str.contains("bad request") {
                        let retry_edit = EditMergeRequest {
                            project: pid,
                            merge_request: iid,
                            title: None,
                            description: None,
                            state_event: None,
                            labels: None,
                            assignee_ids: None,
                            reviewer_ids: None,
                            milestone_id: None,
                            remove_source_branch: None,
                            squash: None,
                            target_branch: None,
                            draft: None,
                            wip: Some(!current_draft),
                        };
                        let retry_res: Result<Result<serde_json::Value, _>, _> =
                            tokio::time::timeout(
                                std::time::Duration::from_secs(30),
                                retry_edit.query_async(&client),
                            )
                            .await;
                        match retry_res {
                            Ok(Ok(val)) => {
                                if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val)
                                {
                                    let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                                }
                            }
                            Ok(Err(e2)) => {
                                let err_str2 = e2.to_string().to_lowercase();
                                if err_str2.contains("400") || err_str2.contains("bad request") {
                                    let new_title =
                                        toggle_wip_title(&current_title, !current_draft);
                                    let title_edit = EditMergeRequest {
                                        project: pid,
                                        merge_request: iid,
                                        title: Some(new_title),
                                        description: None,
                                        state_event: None,
                                        labels: None,
                                        assignee_ids: None,
                                        reviewer_ids: None,
                                        milestone_id: None,
                                        remove_source_branch: None,
                                        squash: None,
                                        target_branch: None,
                                        draft: None,
                                        wip: None,
                                    };
                                    let title_res: Result<Result<serde_json::Value, _>, _> =
                                        tokio::time::timeout(
                                            std::time::Duration::from_secs(30),
                                            title_edit.query_async(&client),
                                        )
                                        .await;
                                    match title_res {
                                        Ok(Ok(val)) => {
                                            if let Ok(updated_mr) =
                                                serde_json::from_value::<MergeRequest>(val)
                                            {
                                                let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                                            }
                                        }
                                        Ok(Err(e3)) => {
                                            let _ = tx.send(DataUpdate::Error(format!(
                                                "MR update failed: {}",
                                                e3
                                            )));
                                        }
                                        Err(_) => {
                                            let _ = tx.send(DataUpdate::Error(
                                                "MR update timed out".into(),
                                            ));
                                        }
                                    }
                                } else {
                                    let _ = tx.send(DataUpdate::Error(format!(
                                        "MR update failed: {}",
                                        e2
                                    )));
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
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("MR update timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn toggle_mr_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid, current_state) = if let Some(mr) = &self.selected_mr {
            (mr.project_id, mr.iid, mr.state.clone())
        } else {
            return Ok(());
        };
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let event = if current_state == "opened" {
            "close"
        } else {
            "reopen"
        };
        if let Some(m) = &mut self.selected_mr {
            m.state = if current_state == "opened" {
                "closed".to_string()
            } else {
                "opened".to_string()
            };
            let updated = m.clone();
            if let Some(existing) = self
                .merge_requests
                .iter_mut()
                .find(|x| x.iid == updated.iid && x.project_id == updated.project_id)
            {
                *existing = updated;
                self.filter_lists();
            }
        }
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let edit = EditMergeRequest {
                project: pid,
                merge_request: iid,
                title: None,
                description: None,
                state_event: Some(event.to_string()),
                labels: None,
                assignee_ids: None,
                reviewer_ids: None,
                milestone_id: None,
                remove_source_branch: None,
                squash: None,
                target_branch: None,
                draft: None,
                wip: None,
            };
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                edit.query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(val)) => {
                    if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) {
                        let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                    }
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Toggle MR state timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }
}

impl App {
    pub async fn toggle_mr_squash(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid, current_squash) = if let Some(mr) = &self.selected_mr {
            (mr.project_id, mr.iid, mr.squash.unwrap_or(false))
        } else {
            return Ok(());
        };
        self.status_message = Some(if current_squash {
            format!("Disabling squash for MR !{}...", iid)
        } else {
            format!("Enabling squash for MR !{}...", iid)
        });
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        if let Some(m) = &mut self.selected_mr {
            m.squash = Some(!current_squash);
            let updated = m.clone();
            if let Some(existing) = self
                .merge_requests
                .iter_mut()
                .find(|x| x.iid == updated.iid && x.project_id == updated.project_id)
            {
                *existing = updated;
                self.filter_lists();
            }
        }
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let edit = EditMergeRequest {
                project: pid,
                merge_request: iid,
                title: None,
                description: None,
                state_event: None,
                labels: None,
                assignee_ids: None,
                reviewer_ids: None,
                milestone_id: None,
                remove_source_branch: None,
                squash: Some(!current_squash),
                target_branch: None,
                draft: None,
                wip: None,
            };
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                edit.query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(val)) => {
                    if let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val) {
                        let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                    }
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("MR update timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn rebase_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            self.status_message = Some(format!("Rebasing MR !{}...", iid));
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = RebaseMergeRequest {
                    project: pid,
                    merge_request: iid,
                };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    endpoint.query_async(&client),
                )
                .await;
                match res {
                    Ok(Ok(_)) => {
                        let mr_endpoint = SingleMergeRequest {
                            project: pid,
                            merge_request: iid,
                        };
                        if let Ok(Ok(val)) = tokio::time::timeout(
                            std::time::Duration::from_secs(30),
                            mr_endpoint.query_async(&client),
                        )
                        .await
                            && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val)
                        {
                            let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                        }
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!("Rebase MR failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Rebase MR timed out".into()));
                    }
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
            let pid = mr.project_id;
            let iid = mr.iid;
            let disc_id = disc.id.clone();
            let current_resolved = disc.resolved.unwrap_or(false);
            self.status_message = Some(if current_resolved {
                "Reopening discussion thread...".into()
            } else {
                "Resolving discussion thread...".into()
            });
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = ResolveDiscussion {
                    project: pid,
                    merge_request: iid,
                    discussion: disc_id,
                    resolved: !current_resolved,
                };
                let res: Result<serde_json::Value, _> = endpoint.query_async(&client).await;
                match res {
                    Ok(_) => {
                        let mr_endpoint = SingleMergeRequest {
                            project: pid,
                            merge_request: iid,
                        };
                        let disc_endpoint = MergeRequestDiscussions {
                            project: pid,
                            merge_request: iid,
                        };
                        let (mr_res, disc_res) = tokio::join!(
                            tokio::time::timeout(
                                std::time::Duration::from_secs(30),
                                mr_endpoint.query_async(&client)
                            ),
                            tokio::time::timeout(
                                std::time::Duration::from_secs(30),
                                disc_endpoint.query_async(&client)
                            )
                        );
                        if let Ok(Ok(val)) = mr_res
                            && let Ok(updated_mr) = serde_json::from_value::<MergeRequest>(val)
                        {
                            let _ = tx.send(DataUpdate::MRDetail(updated_mr));
                        }
                        if let Ok(Ok(discs)) = disc_res {
                            let _ = tx.send(DataUpdate::Discussions {
                                project_id: pid,
                                iid,
                                data: discs,
                            });
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(DataUpdate::Error(format!(
                            "Resolve discussion failed: {}",
                            e
                        )));
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
            let description = if self.create_mr_description.is_empty() {
                None
            } else {
                Some(self.create_mr_description.clone())
            };

            // Optimistic insert: build a placeholder MR with a high fake iid.
            let pending_iid =
                self.merge_requests.iter().map(|m| m.iid).max().unwrap_or(0) + 1_000_000;
            let author = self
                .current_user
                .clone()
                .unwrap_or_else(|| Author::new(0, "you".into()));
            let pending_mr = MergeRequest {
                iid: pending_iid,
                project_id: pid,
                title: title.clone(),
                description: description.clone(),
                author: author.clone(),
                web_url: String::new(),
                state: "opened".into(),
                references: None,
                upvotes: Some(0),
                downvotes: Some(0),
                labels: vec![],
                milestone: None,
                assignees: vec![],
                reviewers: vec![],
                squash: Some(false),
                created_at: Some(chrono::Utc::now().to_rfc3339()),
                draft: Some(false),
                reactions: vec![],
                diff_refs: None,
                merge_status: None,
                detailed_merge_status: None,
                has_conflicts: None,
                head_pipeline: None,
                source_branch: source_branch.clone(),
                target_branch: target_branch.clone(),
                participants: vec![],
            };
            self.merge_requests.insert(0, pending_mr.clone());
            self.filter_lists();
            self.status_message = Some(format!("Creating MR '{}'...", title));
            let _ = self.update_preview().await;

            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let ep = CreateMergeRequest {
                    project: pid,
                    source_branch,
                    target_branch,
                    title,
                    description,
                };
                let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    ep.query_async(&client),
                )
                .await;
                match res {
                    Ok(Ok(_)) => {
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!("MR creation failed: {}", e)));
                        let _ = tx.send(DataUpdate::RemovePendingMR { iid: pending_iid });
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("MR creation timed out".into()));
                        let _ = tx.send(DataUpdate::RemovePendingMR { iid: pending_iid });
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
            self.show_create_mr_popup = false;
            self.input_mode = false;
        }
        Ok(())
    }
}

pub(super) fn strip_draft_prefix(title: &str) -> String {
    let mut t = title.trim();
    loop {
        let lower = t.to_lowercase();
        if lower.starts_with("draft:") {
            t = t["draft:".len()..].trim_start();
        } else if lower.starts_with("draft -") {
            t = t["draft -".len()..].trim_start();
        } else if lower.starts_with("draft") {
            let rest = &t["draft".len()..];
            if rest.is_empty()
                || rest
                    .chars()
                    .next()
                    .map(|c| c.is_whitespace() || c.is_ascii_punctuation())
                    .unwrap_or(false)
            {
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
            if rest.is_empty()
                || rest
                    .chars()
                    .next()
                    .map(|c| c.is_whitespace() || c.is_ascii_punctuation())
                    .unwrap_or(false)
            {
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

pub(super) fn toggle_wip_title(title: &str, make_draft: bool) -> String {
    let clean = strip_draft_prefix(title);
    if make_draft {
        format!("Draft: {}", clean)
    } else {
        clean
    }
}
