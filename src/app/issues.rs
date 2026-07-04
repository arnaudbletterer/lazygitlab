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
        self.status_message = Some("Fetching Issues...".into());

        let project_ids: Vec<u64> = self.member_project_ids.iter().cloned().collect();

        tokio::spawn(async move {
            if project_ids.is_empty() {
                let _ = tx.send(DataUpdate::GlobalIssues { data: Vec::new(), page: 1, has_more: false });
                let _ = tx.send(DataUpdate::Loading(false));
                return;
            }

            let mut all_issues = Vec::new();
            let mut handles = Vec::new();

            for pid in project_ids {
                let client = c.clone();
                let state = state_str.clone();
                let handle = tokio::spawn(async move {
                    let ep = PagedIssues {
                        project: pid,
                        state: Some(state),
                        page: 1,
                        per_page: 50,
                        order_by: Some("updated_at".into()),
                        sort: Some("desc".into()),
                        with_labels_details: Some(true),
                    };
                    let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
                    match res {
                        Ok(Ok(issues)) => issues,
                        _ => Vec::new(),
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                if let Ok(issues) = handle.await {
                    all_issues.extend(issues);
                }
            }

            // Sort descending by created_at
            all_issues.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            let has_more = all_issues.len() >= 50;
            all_issues.truncate(50);

            let _ = tx.send(DataUpdate::GlobalIssues { data: all_issues, page: 1, has_more });
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_global_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

        let project_ids: Vec<u64> = self.member_project_ids.iter().cloned().collect();

        tokio::spawn(async move {
            if project_ids.is_empty() {
                let _ = tx.send(DataUpdate::Loading(false));
                return;
            }

            let mut all_issues = Vec::new();
            let mut handles = Vec::new();

            for pid in project_ids {
                let client = c.clone();
                let state = state_str.clone();
                let handle = tokio::spawn(async move {
                    let ep = PagedIssues {
                        project: pid,
                        state: Some(state),
                        page: next_page,
                        per_page: 50,
                        order_by: Some("updated_at".into()),
                        sort: Some("desc".into()),
                        with_labels_details: Some(true),
                    };
                    let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
                    match res {
                        Ok(Ok(issues)) => issues,
                        _ => Vec::new(),
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                if let Ok(issues) = handle.await {
                    all_issues.extend(issues);
                }
            }

            // Sort descending by created_at
            all_issues.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            let has_more = all_issues.len() >= 50;
            all_issues.truncate(50);

            let _ = tx.send(DataUpdate::AppendGlobalIssues { data: all_issues, has_more });
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
                let res: Result<Result<Issue, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => {
                        let _ = tx.send(DataUpdate::IssueDetail(data));
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Update Issue timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
            self.input_mode = false;
        }
        Ok(())
    }

    pub async fn update_issue_labels(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &mut self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let updated_labels: Vec<Label> = {
                let labels_list = self.project_label_cache.get(&pid).unwrap_or(&self.project_labels);
                labels_list.iter()
                    .filter(|l| self.selected_label_ids.contains(&l.id()))
                    .cloned()
                    .collect()
            };
            let labels = updated_labels.iter().map(|l| l.name().to_string()).collect::<Vec<_>>().join(",");

            issue.labels = updated_labels.clone();
            if let Some(idx) = self.issues.iter_mut().position(|i| i.iid == iid && i.project_id == pid) {
                self.issues[idx].labels = updated_labels;
            }
            self.filter_lists();

            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            self.show_label_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = EditIssue { project: pid, issue: iid, labels: Some(labels), ..Default::default() };
                let res: Result<Result<Issue, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => {
                        let _ = tx.send(DataUpdate::IssueDetail(data));
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Update Issue labels timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_issue_assignees(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &mut self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let members = self.project_member_cache.get(&pid).unwrap_or(&self.project_members);
            let updated_assignees: Vec<Author> = members.iter()
                .filter(|m| self.selected_assignee_ids.contains(&m.id))
                .cloned()
                .collect();
            issue.assignees = updated_assignees.clone();
            if let Some(idx) = self.issues.iter_mut().position(|i| i.iid == iid && i.project_id == pid) {
                self.issues[idx].assignees = updated_assignees;
            }
            self.filter_lists();

            let assignees = self.selected_assignee_ids.clone();
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            self.show_assignee_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = EditIssue { project: pid, issue: iid, assignee_ids: Some(assignees), ..Default::default() };
                let res: Result<Result<Issue, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => {
                        let _ = tx.send(DataUpdate::IssueDetail(data));
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Update Issue assignees timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_issue_milestone(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &mut self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let mid = self.selected_milestone_id;
            let milestones = self.project_milestone_cache.get(&pid).unwrap_or(&self.project_milestones);
            let updated_milestone = mid.and_then(|id| milestones.iter().find(|m| m.id == id).cloned());
            issue.milestone = updated_milestone.clone();
            if let Some(idx) = self.issues.iter_mut().position(|i| i.iid == iid && i.project_id == pid) {
                self.issues[idx].milestone = updated_milestone;
            }
            self.filter_lists();

            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            self.show_milestone_popup = false;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = EditIssue { project: pid, issue: iid, milestone_id: Some(mid), ..Default::default() };
                let res: Result<Result<Issue, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => {
                        let _ = tx.send(DataUpdate::IssueDetail(data));
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Update Issue milestone timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_issue_start_date(&mut self, date: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &mut self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            issue.start_date = date.clone();
            if let Some(idx) = self.issues.iter_mut().position(|i| i.iid == iid && i.project_id == pid) {
                self.issues[idx].start_date = date.clone();
            }
            self.filter_lists();

            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = EditIssue { project: pid, issue: iid, start_date: Some(date), ..Default::default() };
                let res: Result<Result<Issue, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => {
                        let _ = tx.send(DataUpdate::IssueDetail(data));
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Update Issue start date timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn update_issue_due_date(&mut self, date: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &mut self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            issue.due_date = date.clone();
            if let Some(idx) = self.issues.iter_mut().position(|i| i.iid == iid && i.project_id == pid) {
                self.issues[idx].due_date = date.clone();
            }
            self.filter_lists();

            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = EditIssue { project: pid, issue: iid, due_date: Some(date), ..Default::default() };
                let res: Result<Result<Issue, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => {
                        let _ = tx.send(DataUpdate::IssueDetail(data));
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Update Issue due date timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
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
                let res: Result<Result<Issue, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => {
                        let _ = tx.send(DataUpdate::IssueDetail(data));
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Toggle Issue state timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn toggle_issue_confidential(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &mut self.selected_issue {
            let pid = issue.project_id;
            let iid = issue.iid;
            let current_confidential = issue.confidential;
            let next_confidential = !current_confidential;
            issue.confidential = next_confidential;
            if let Some(idx) = self.issues.iter_mut().position(|i| i.iid == iid && i.project_id == pid) {
                self.issues[idx].confidential = next_confidential;
            }
            self.filter_lists();

            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = EditIssue { project: pid, issue: iid, confidential: Some(next_confidential), ..Default::default() };
                let res: Result<Result<Issue, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => {
                        let _ = tx.send(DataUpdate::IssueDetail(data));
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue update failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Toggle Issue confidentiality timed out".into())); }
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

            // Optimistic insert: build a placeholder Issue with a high fake iid
            // so the user sees it immediately. If the API call fails, the
            // TriggerRefresh (or our fallback below) will remove it.
            let pending_iid = self.issues.iter().map(|i| i.iid).max().unwrap_or(0) + 1_000_000;
            let author = self.current_user.clone().unwrap_or_else(|| Author::new(0, "you".into()));
            let pending_issue = Issue {
                iid: pending_iid,
                project_id: pid,
                title: title.clone(),
                description: Some(description.clone()),
                author: author.clone(),
                assignees: vec![],
                state: "opened".into(),
                web_url: String::new(),
                references: None,
                issue_type: None,
                labels: vec![],
                milestone: None,
                due_date: None,
                start_date: None,
                weight: None,
                task_completion_status: None,
                confidential: false,
                created_at: Some(chrono::Utc::now().to_rfc3339()),
                upvotes: Some(0),
                downvotes: Some(0),
                reactions: vec![],
                participants: vec![],
            };
            self.issues.insert(0, pending_issue.clone());
            self.filter_lists();
            self.status_message = Some(format!("Creating issue '{}'...", title));
            let _ = self.update_preview().await;

            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let ep = CreateIssue { project: pid, title, description: Some(description), assignee_ids: vec![], label_names: vec![] };
                let res: Result<Issue, _> = ep.query_async(&c).await;
                match res {
                    Ok(_) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                    Err(e) => {
                        let _ = tx.send(DataUpdate::Error(format!("Issue creation failed: {}", e)));
                        let _ = tx.send(DataUpdate::RemovePendingIssue { iid: pending_iid });
                    }
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
            if let Ok(Ok(data)) = res {
                let _ = tx.send(DataUpdate::IssueDiscussions { project_id, iid, data });
            }
        });
        Ok(())
    }

    pub async fn resolve_issue_discussion(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(issue) = &self.selected_issue
            && let Some(idx) = self.discussion_state.selected()
            && let Some(disc) = self.discussions.get(idx).cloned()
        {
            let pid = issue.project_id;
            let iid = issue.iid;
            let disc_id = disc.id.clone();
            let current_resolved = disc.resolved.unwrap_or(false);
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            self.status_message = Some(format!("{} issue discussion {}...", if current_resolved { "Reopening" } else { "Resolving" }, disc_id));
            let _ = self.update_preview().await;
            tokio::spawn(async move {
                use gitlab::api::AsyncQuery;
                use crate::api::ResolveIssueDiscussion;
                let endpoint = ResolveIssueDiscussion { project: pid, issue: iid, discussion: disc_id.clone(), resolved: !current_resolved };
                let res: Result<serde_json::Value, _> = endpoint.query_async(&client).await;
                if res.is_ok() {
                    use crate::api::SingleIssue;
                    let issue_ep = SingleIssue { project: pid, issue: iid };
                    if let Ok(Ok(val)) = tokio::time::timeout(std::time::Duration::from_secs(30), issue_ep.query_async(&client)).await
                        && let Ok(updated_issue) = serde_json::from_value::<Issue>(val)
                    {
                        let _ = tx.send(DataUpdate::IssueDetail(updated_issue));
                    }
                    // refresh discussions list
                    let disc_ep = IssueDiscussions { project: pid, issue: iid };
                    let pager = gitlab::api::paged(disc_ep, gitlab::api::Pagination::All);
                    if let Ok(Ok(discs)) = tokio::time::timeout(std::time::Duration::from_secs(30), pager.query_async(&client)).await {
                        let _ = tx.send(DataUpdate::IssueDiscussions { project_id: pid, iid, data: discs });
                    }
                } else {
                    let _ = tx.send(DataUpdate::Error("Issue discussion resolve failed".to_string()));
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        } else {
            self.status_message = Some("No issue discussion selected".into());
        }
        Ok(())
    }

    pub async fn fetch_issue_detail(&mut self, project_id: u64, iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        let c = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let ep = SingleIssue { project: project_id, issue: iid };
            let pep = crate::api::IssueParticipants { project: project_id, issue: iid };
            let pager = gitlab::api::paged(pep, gitlab::api::Pagination::All);
            let (res, pres): (Result<Result<Issue, _>, _>, Result<Result<Vec<Author>, _>, _>) = tokio::join!(
                tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c)),
                tokio::time::timeout(std::time::Duration::from_secs(30), pager.query_async(&c))
            );

            let participants: Vec<Author> = match pres {
                Ok(Ok(parts)) => parts,
                _ => Vec::new(),
            };

            match res {
                Ok(Ok(mut data)) => {
                    data.participants = participants;
                    let _ = tx.send(DataUpdate::IssueDetail(data));
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Issue detail fetch failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Issue detail fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }
}
