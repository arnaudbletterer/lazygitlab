use crate::api::*;
use crate::models::*;
use gitlab::api::AsyncQuery;

impl App {
    pub async fn update_mr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (pid, iid) = if let Some(mr) = &self.selected_mr {
            (mr.project_id, mr.iid)
        } else {
            return Ok(());
        };
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let mut edit = EditMergeRequest {
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
            wip: None,
        };
        let body = self.textarea.lines().join("\n");
        match self.input_target {
            InputTarget::EditTitle => {
                edit.title = Some(body.clone());
                if let Some(m) = &mut self.selected_mr {
                    m.title = body.clone();
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
            }
            InputTarget::EditDescription => {
                edit.description = Some(body.clone());
                if let Some(m) = &mut self.selected_mr {
                    m.description = Some(body.clone());
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
            }
            _ => return Ok(()),
        }
        self.input_mode = false;
        self.textarea = ratatui_textarea::TextArea::default();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
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
                    let _ = tx.send(DataUpdate::Error("Update MR timed out".into()));
                }
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
                let labels_list = self
                    .project_label_cache
                    .get(&pid)
                    .unwrap_or(&self.project_labels);
                labels_list
                    .iter()
                    .filter(|l| self.selected_label_ids.contains(&l.id()))
                    .cloned()
                    .collect()
            };
            let labels = updated_labels
                .iter()
                .map(|l| l.name().to_string())
                .collect::<Vec<_>>()
                .join(",");

            mr.labels = updated_labels.clone();
            if let Some(idx) = self
                .merge_requests
                .iter_mut()
                .position(|m| m.iid == iid && m.project_id == pid)
            {
                self.merge_requests[idx].labels = updated_labels;
            }
            self.filter_lists();

            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            self.show_label_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditMergeRequest {
                    project: pid,
                    merge_request: iid,
                    title: None,
                    description: None,
                    state_event: None,
                    labels: Some(labels),
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
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Update MR labels timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_mr_assignees(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &mut self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let members = self
                .project_member_cache
                .get(&pid)
                .unwrap_or(&self.project_members);
            let updated_assignees: Vec<Author> = members
                .iter()
                .filter(|m| self.selected_assignee_ids.contains(&m.id))
                .cloned()
                .collect();
            mr.assignees = updated_assignees.clone();
            if let Some(idx) = self
                .merge_requests
                .iter_mut()
                .position(|m| m.iid == iid && m.project_id == pid)
            {
                self.merge_requests[idx].assignees = updated_assignees;
            }
            self.filter_lists();

            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let assignee_ids = self.selected_assignee_ids.clone();
            self.show_assignee_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let edit = EditMergeRequest {
                    project: pid,
                    merge_request: iid,
                    title: None,
                    description: None,
                    state_event: None,
                    labels: None,
                    assignee_ids: Some(assignee_ids),
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
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Update MR assignees timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_mr_reviewers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &mut self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let members = self
                .project_member_cache
                .get(&pid)
                .unwrap_or(&self.project_members);
            let updated_reviewers: Vec<Author> = members
                .iter()
                .filter(|m| self.selected_reviewer_ids.contains(&m.id))
                .cloned()
                .collect();
            mr.reviewers = updated_reviewers.clone();
            if let Some(idx) = self
                .merge_requests
                .iter_mut()
                .position(|m| m.iid == iid && m.project_id == pid)
            {
                self.merge_requests[idx].reviewers = updated_reviewers;
            }
            self.filter_lists();

            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let reviewer_ids = self.selected_reviewer_ids.clone();
            self.show_reviewer_popup = false;
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
                    reviewer_ids: Some(reviewer_ids),
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
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Update MR reviewers timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_mr_milestone(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = &mut self.selected_mr {
            let pid = mr.project_id;
            let iid = mr.iid;
            let mid = if let Some(i) = self.milestone_state.selected() {
                if i == 0 {
                    None
                } else {
                    Some(self.project_milestones[i - 1].id)
                }
            } else {
                None
            };
            let milestones = self
                .project_milestone_cache
                .get(&pid)
                .unwrap_or(&self.project_milestones);
            let updated_milestone =
                mid.and_then(|id| milestones.iter().find(|m| m.id == id).cloned());
            mr.milestone = updated_milestone.clone();
            if let Some(idx) = self
                .merge_requests
                .iter_mut()
                .position(|m| m.iid == iid && m.project_id == pid)
            {
                self.merge_requests[idx].milestone = updated_milestone;
            }
            self.filter_lists();

            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            self.show_milestone_popup = false;
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
                    milestone_id: Some(mid),
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
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Update MR milestone timed out".into()));
                    }
                }
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
                if let Some(idx) = self
                    .merge_requests
                    .iter_mut()
                    .position(|m| m.iid == iid && m.project_id == pid)
                {
                    self.merge_requests[idx].target_branch = branch.clone();
                }
                self.mr_diffs.clear();
                self.mr_diff_state = ratatui::widgets::ListState::default();
                self.filter_lists();

                let client = self.gitlab.clone();
                let tx = self.tx.clone();
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
                        target_branch: Some(branch),
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
                            let _ = tx.send(DataUpdate::TriggerRefresh);
                        }
                        Ok(Err(e)) => {
                            let _ = tx.send(DataUpdate::Error(format!("MR update failed: {}", e)));
                        }
                        Err(_) => {
                            let _ = tx.send(DataUpdate::Error(
                                "Update MR target branch timed out".into(),
                            ));
                        }
                    }
                    let _ = tx.send(DataUpdate::Loading(false));
                });
            }
        }
        Ok(())
    }
}
