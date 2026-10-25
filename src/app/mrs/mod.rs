mod diffs;
mod list;
mod meta;
mod review;

pub(crate) use crate::app::merge::LoadingGuard;

pub use diffs::parse_diff_context;
pub use list::{CACHE_FLUSH_EVERY, MrFetch, fetch_global_mrs, rate_limit_text, state_of};

use crate::api::*;
use crate::models::*;
use gitlab::api::AsyncQuery;

impl App {
    pub async fn fetch_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            self.start_mr_fetch(Some(pid));
        }
        Ok(())
    }

    pub async fn fetch_next_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.mrs_has_more || self.is_loading {
            return Ok(());
        }
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let next_page = self.mrs_page + 1;
            let _ = tx.send(DataUpdate::Loading(true));
            let state_str = match self.mr_filter {
                MergeRequestFilter::Opened
                | MergeRequestFilter::Review
                | MergeRequestFilter::Assigned => Some("opened".to_string()),
                MergeRequestFilter::Merged => Some("merged".to_string()),
                MergeRequestFilter::Closed => Some("closed".to_string()),
                MergeRequestFilter::All => None,
            };
            let assignee_id = if self.mr_filter == MergeRequestFilter::Assigned {
                self.current_user.as_ref().map(|u| u.id)
            } else {
                None
            };
            let reviewer_id = if self.mr_filter == MergeRequestFilter::Review {
                self.current_user.as_ref().map(|u| u.id)
            } else {
                None
            };
            let scope = self.mr_filter;
            tokio::spawn(async move {
                let ep = PagedMergeRequests {
                    project: pid,
                    page: next_page,
                    per_page: 50,
                    state: state_str,
                    order_by: Some("updated_at".into()),
                    sort: Some("desc".into()),
                    with_labels_details: Some(true),
                    assignee_id,
                    reviewer_id,
                };
                let res: Result<Result<Vec<MergeRequest>, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(60),
                    ep.query_async(&client),
                )
                .await;
                match res {
                    Ok(Ok(data)) => {
                        if !data.is_empty() {
                            let _ = tx.send(DataUpdate::ScopedMr(
                                scope,
                                Box::new(DataUpdate::AppendProjectMRs {
                                    project_id: pid,
                                    data,
                                }),
                            ));
                        }
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!("MRs fetch failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("MRs fetch timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_global_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.start_mr_fetch(None);
        Ok(())
    }

    pub async fn fetch_next_global_merge_requests(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.mrs_has_more || self.is_loading {
            return Ok(());
        }
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let next_page = self.mrs_page + 1;
        let state_str = match self.mr_filter {
            MergeRequestFilter::Opened
            | MergeRequestFilter::Review
            | MergeRequestFilter::Assigned => "opened".to_string(),
            MergeRequestFilter::Merged => "merged".to_string(),
            MergeRequestFilter::Closed => "closed".to_string(),
            MergeRequestFilter::All => "all".to_string(),
        };
        let assignee_id = if self.mr_filter == MergeRequestFilter::Assigned {
            self.current_user.as_ref().map(|u| u.id)
        } else {
            None
        };
        let reviewer_id = if self.mr_filter == MergeRequestFilter::Review {
            self.current_user.as_ref().map(|u| u.id)
        } else {
            None
        };

        let _ = tx.send(DataUpdate::Loading(true));

        let project_ids: Vec<u64> = self.member_project_ids.iter().cloned().collect();

        let scope = self.mr_filter;
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
                let assignee_id = assignee_id;
                let reviewer_id = reviewer_id;
                let handle = tokio::spawn(async move {
                    let ep = PagedMergeRequests {
                        project: pid,
                        state: Some(state),
                        page: next_page,
                        per_page: 50,
                        order_by: Some("updated_at".into()),
                        sort: Some("desc".into()),
                        with_labels_details: Some(true),
                        assignee_id,
                        reviewer_id,
                    };
                    let res: Result<Result<Vec<MergeRequest>, _>, _> = tokio::time::timeout(
                        std::time::Duration::from_secs(60),
                        ep.query_async(&c),
                    )
                    .await;
                    match res {
                        Ok(Ok(mrs)) => Some(mrs),
                        _ => None,
                    }
                });
                handles.push(handle);
            }

            let mut complete = true;
            for handle in handles {
                match handle.await {
                    Ok(Some(items)) => all_mrs.extend(items),
                    _ => complete = false,
                }
            }

            Self::sort_mrs(&mut all_mrs);
            let has_more = all_mrs.len() >= 50;

            let _ = tx.send(DataUpdate::ScopedMr(
                scope,
                Box::new(DataUpdate::AppendGlobalMRs {
                    data: all_mrs,
                    has_more,
                    page: next_page,
                    complete,
                }),
            ));
            if !complete {
                let _ = tx.send(DataUpdate::ScopedMr(
                    scope,
                    Box::new(DataUpdate::RefreshFailed),
                ));
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_discussions(
        &mut self,
        project_id: u64,
        mr_iid: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.abort_mr_detail();
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let (prev_participants, prev_reactions) = self
            .selected_mr
            .as_ref()
            .filter(|m| m.iid == mr_iid && m.project_id == project_id)
            .map(|m| (m.participants.clone(), m.reactions.clone()))
            .unwrap_or_default();
        let loading = crate::app::merge::LoadingGuard::start(tx.clone());
        let handle = tokio::spawn(async move {
            let _loading = loading;
            let mr_endpoint = SingleMergeRequest {
                project: project_id,
                merge_request: mr_iid,
            };
            let pep = crate::api::MergeRequestParticipants {
                project: project_id,
                mr: mr_iid,
            };
            let pager = gitlab::api::paged(pep, gitlab::api::Pagination::All);
            let (mr_res, pres): (
                Result<crate::models::MergeRequest, _>,
                Result<Vec<Author>, _>,
            ) = tokio::join!(mr_endpoint.query_async(&client), pager.query_async(&client));

            let participants: Vec<Author> = pres.unwrap_or_default();

            if let Ok(mut mr) = mr_res {
                let emoji_endpoint = MergeRequestAwardEmoji {
                    project: project_id,
                    merge_request: mr_iid,
                };
                let reaction_values: Vec<serde_json::Value> =
                    gitlab::api::paged(emoji_endpoint, gitlab::api::Pagination::All)
                        .query_async(&client)
                        .await
                        .unwrap_or_default();
                mr.reactions = crate::app::parse_reactions(reaction_values);
                mr.participants = participants;
                let _ = tx.send(DataUpdate::MRDetail(mr));
            }
            let endpoint = MergeRequestDiscussions {
                project: project_id,
                merge_request: mr_iid,
            };
            let res: Result<Result<Vec<Discussion>, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                gitlab::api::paged(endpoint, gitlab::api::Pagination::All).query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(mut discussions)) => {
                    discussions.retain(|d| !d.notes.is_empty());
                    let _ = tx.send(DataUpdate::Discussions {
                        project_id,
                        iid: mr_iid,
                        data: discussions,
                    });
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!(
                        "Discussions fetch failed: {}",
                        e
                    )));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Discussions fetch timed out".into()));
                }
            }
        });
        self.mr_detail_task = Some(handle);
        Ok(())
    }

    pub async fn fetch_related_items(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mr) = self.selected_mr.clone() {
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let pid = mr.project_id;
            let iid = mr.iid;
            tokio::spawn(async move {
                let approvals_ep = crate::api::MergeRequestApprovals {
                    project: pid,
                    merge_request: iid,
                };
                let approvals_res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(15),
                    approvals_ep.query_async(&client),
                )
                .await;
                if let Ok(Ok(raw)) = approvals_res
                    && let Ok(info) = serde_json::from_value::<crate::models::ApprovalInfo>(raw)
                {
                    let _ = tx.send(DataUpdate::ApprovalInfo {
                        project_id: pid,
                        iid,
                        info,
                    });
                }
                let related_ep = crate::api::RelatedMergeRequests {
                    project: pid,
                    merge_request: iid,
                };
                let closes_ep = crate::api::MRClosesIssues {
                    project: pid,
                    merge_request: iid,
                };
                let related_pager = gitlab::api::paged(related_ep, gitlab::api::Pagination::All);
                let closes_pager = gitlab::api::paged(closes_ep, gitlab::api::Pagination::All);
                let (related_res, closes_res): (
                    Result<Vec<crate::models::MergeRequest>, _>,
                    Result<Vec<crate::models::Issue>, _>,
                ) = tokio::join!(
                    related_pager.query_async(&client),
                    closes_pager.query_async(&client)
                );
                match related_res {
                    Ok(data) => {
                        let _ = tx.send(DataUpdate::RelatedMRs {
                            project_id: pid,
                            iid,
                            data,
                        });
                    }
                    Err(e) => {
                        tracing::warn!("related MRs fetch failed: {}", e);
                    }
                }
                match closes_res {
                    Ok(data) => {
                        let _ = tx.send(DataUpdate::MRClosesIssues {
                            project_id: pid,
                            iid,
                            data,
                        });
                    }
                    Err(e) => {
                        tracing::warn!("closing issues fetch failed: {}", e);
                    }
                }
            });
        } else if let Some(issue) = self.selected_issue.clone() {
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let pid = issue.project_id;
            let iid = issue.iid;
            tokio::spawn(async move {
                let links_ep = crate::api::IssueLinks {
                    project: pid,
                    issue: iid,
                };
                let closed_by_ep = crate::api::IssueClosedBy {
                    project: pid,
                    issue: iid,
                };
                let links_pager = gitlab::api::paged(links_ep, gitlab::api::Pagination::All);
                let closed_by_pager =
                    gitlab::api::paged(closed_by_ep, gitlab::api::Pagination::All);
                let (links_res, closed_by_res): (
                    Result<Vec<crate::models::Issue>, _>,
                    Result<Vec<crate::models::MergeRequest>, _>,
                ) = tokio::join!(
                    links_pager.query_async(&client),
                    closed_by_pager.query_async(&client)
                );
                match links_res {
                    Ok(data) => {
                        let _ = tx.send(DataUpdate::IssueLinks {
                            project_id: pid,
                            iid,
                            data,
                        });
                    }
                    Err(e) => {
                        tracing::warn!("issue links fetch failed: {}", e);
                    }
                }
                match closed_by_res {
                    Ok(data) => {
                        let _ = tx.send(DataUpdate::IssueClosedBy {
                            project_id: pid,
                            iid,
                            data,
                        });
                    }
                    Err(e) => {
                        tracing::warn!("closed-by MRs fetch failed: {}", e);
                    }
                }
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::review::{strip_draft_prefix, toggle_wip_title};
    use super::*;
    use gitlab::GitlabBuilder;
    use std::time::Duration;
    use tokio::sync::mpsc;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_app() -> (MockServer, App, mpsc::UnboundedReceiver<DataUpdate>) {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id":1,"username":"u"})),
            )
            .mount(&server)
            .await;
        let client = GitlabBuilder::new(server.uri().replace("http://", ""), "t")
            .insecure()
            .build_async()
            .await
            .unwrap();
        let (tx, rx) = mpsc::unbounded_channel();
        let mut app = App::new(client, tx, None);
        app.active_panel = Panel::Project;
        app.project_menu_state.select(Some(1));
        app.active_context = Context::Project;
        app.active_selection = MenuSelection::MergeRequests;
        app.selected_project = Some(Project {
            id: 1,
            name: "p".into(),
            path_with_namespace: Some("n/p".into()),
            web_url: Some("u".into()),
            default_branch: None,
            namespace: None,
        });
        (server, app, rx)
    }

    #[tokio::test]
    async fn test_mr_fetching_logic_exhaustive() {
        let (server, mut app, mut rx) = setup_app().await;
        let mr_json = r##"{"iid": 1, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "t", "reactions": [], "draft": false}"##;
        Mock::given(path("/api/v4/projects/1/merge_requests"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                serde_json::from_str::<MergeRequest>(mr_json).unwrap()
            ])))
            .mount(&server)
            .await;
        Mock::given(path("/api/v4/projects/1/merge_requests/1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::from_str::<MergeRequest>(mr_json).unwrap()),
            )
            .mount(&server)
            .await;
        Mock::given(path("/api/v4/projects/1/merge_requests/1/award_emoji"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&server)
            .await;
        Mock::given(path("/api/v4/projects/1/merge_requests/1/discussions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;
        app.fetch_merge_requests().await.unwrap();
        let _ = tokio::time::timeout(Duration::from_secs(2), async {
            while !app.is_loading {
                app.process_updates(&mut rx).await;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            while app.is_loading {
                app.process_updates(&mut rx).await;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await;
        assert!(!app.merge_requests.is_empty());
        app.selected_mr = Some(app.merge_requests[0].clone());
        app.fetch_discussions(1, 1).await.unwrap();
        let _ = tokio::time::timeout(Duration::from_secs(2), async {
            while app.is_loading {
                app.process_updates(&mut rx).await;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await;
        app.approve_mr().await.unwrap();
        app.unapprove_mr().await.unwrap();
        app.merge_mr().await.unwrap();
        app.toggle_mr_state().await.unwrap();
        app.input_target = InputTarget::EditTitle;
        app.textarea = ratatui_textarea::TextArea::from(vec!["new"]);
        app.update_mr().await.unwrap();
        app.project_labels = vec![Label {
            id: 1,
            name: "l".into(),
            color: "c".into(),
            text_color: Some("t".into()),
        }];
        app.selected_label_ids.push(1);
        app.update_mr_labels().await.unwrap();
        app.selected_assignee_ids.push(1);
        app.update_mr_assignees().await.unwrap();
        app.project_milestones = vec![Milestone {
            id: 1,
            title: "m".into(),
        }];
        app.milestone_state.select(Some(1));
        app.update_mr_milestone().await.unwrap();
        app.resolve_discussion().await.unwrap();
    }

    #[test]
    fn test_draft_prefix_manipulation() {
        assert_eq!(strip_draft_prefix("My MR"), "My MR");
        assert_eq!(strip_draft_prefix("Draft: My MR"), "My MR");
        assert_eq!(strip_draft_prefix("WIP: My MR"), "My MR");
        assert_eq!(strip_draft_prefix("wip - My MR"), "My MR");
        assert_eq!(
            strip_draft_prefix("Drafting new feature"),
            "Drafting new feature"
        );
        assert_eq!(strip_draft_prefix("[Draft] My MR"), "My MR");
        assert_eq!(strip_draft_prefix("(WIP) My MR"), "My MR");

        assert_eq!(toggle_wip_title("My MR", true), "Draft: My MR");
        assert_eq!(toggle_wip_title("Draft: My MR", true), "Draft: My MR");
        assert_eq!(toggle_wip_title("Draft: My MR", false), "My MR");
        assert_eq!(toggle_wip_title("WIP: My MR", false), "My MR");
    }
}
