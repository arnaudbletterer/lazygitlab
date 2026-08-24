use gitlab::api::{AsyncQuery, paged};
use crate::models::*;
use crate::api::*;

impl App {
    pub async fn fetch_boards(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let pid = project.id;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = ProjectBoards { project: pid };
                let res: Result<Result<Vec<Board>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), paged(ep, gitlab::api::Pagination::Limit(100)).query_async(&client)).await;
                match res {
                    Ok(Ok(data)) => { let _ = tx.send(DataUpdate::Boards { project_id: pid, data }); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Boards fetch timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_board_lists(&mut self, board_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let pid = project.id;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                // 1. Fetch the label lists
                let ep = ProjectBoardLists { project: pid, board: board_id };
                let res: Result<Result<Vec<BoardList>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), paged(ep, gitlab::api::Pagination::Limit(100)).query_async(&client)).await;
                
                // 2. Fetch full board details to check for default columns (Backlog/Closed)
                let det_ep = ProjectBoardDetails { project: pid, board: board_id };
                let det_res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), det_ep.query_async(&client)).await;

                let mut all_lists = match res {
                    Ok(Ok(data)) => data,
                    _ => Vec::new(),
                };

                if let Ok(Ok(det)) = det_res {
                    // GitLab board details can include "backlog_list" and "closed_list"
                    // These are not in the 'lists' endpoint.
                    
                    if let Some(backlog) = det.get("backlog_list")
                        && let Ok(bl) = serde_json::from_value::<BoardList>(backlog.clone())
                        && !all_lists.iter().any(|l| l.id == bl.id)
                    {
                        all_lists.insert(0, bl);
                    }
                    if let Some(closed) = det.get("closed_list")
                        && let Ok(cl) = serde_json::from_value::<BoardList>(closed.clone())
                        && !all_lists.iter().any(|l| l.id == cl.id)
                    {
                        all_lists.push(cl);
                    }
                }

                let _ = tx.send(DataUpdate::BoardLists { project_id: pid, board_id, data: all_lists });
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_board_list_issues(&mut self, board_id: u64, list_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let pid = project.id;
            
            // Check if this is a default list (Backlog/Closed) by looking at its list_type in our state
            let _list_type = self.board_lists.iter().find(|l| l.id == list_id).map(|l| l.list_type.clone()).unwrap_or_default();
            
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let mut data = Vec::new();
                
                let ep = ProjectBoardListIssues { project: pid, board: board_id, list: list_id };
                // Increase timeout to 60s for stress test projects
                let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(60), paged(ep, gitlab::api::Pagination::Limit(50)).query_async(&client)).await;
                
                match res {
                    Ok(Ok(d)) => { data = d; }
                    Ok(Err(e)) => { 
                        let err_msg = e.to_string();
                        // Special handling for backlog/closed if needed, but the endpoint should work
                        if !err_msg.contains("404") {
                            let _ = tx.send(DataUpdate::Error(format!("List {} error: {}", list_id, err_msg))); 
                        }
                    }
                    Err(_) => { let _ = tx.send(DataUpdate::Error(format!("List {} fetch timed out", list_id))); }
                }
                
                let _ = tx.send(DataUpdate::BoardIssues { project_id: pid, board_id, list_id, data });
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_epic_issues(&mut self, group_id: u64, epic_iid: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let ep = GroupEpicIssues { group: group_id, epic_iid };
            let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), paged(ep, gitlab::api::Pagination::All).query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { let _ = tx.send(DataUpdate::EpicIssues { group_id, epic_iid, data }); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Epic issues fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_epics(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            if let Some(namespace) = &project.namespace {
                if namespace.kind == "group" {
                    let group_id = namespace.id;
                    let client = self.gitlab.clone();
                    let tx = self.tx.clone();
                    let _ = tx.send(DataUpdate::Loading(true));
                    tokio::spawn(async move {
                        let ep = GroupEpics { group: group_id };
                        let res: Result<Result<Vec<Epic>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), paged(ep, gitlab::api::Pagination::Limit(100)).query_async(&client)).await;
                        match res {
                            Ok(Ok(data)) => { let _ = tx.send(DataUpdate::Epics { group_id, data }); }
                            Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                            Err(_) => { let _ = tx.send(DataUpdate::Error("Epics fetch timed out".into())); }
                        }
                        let _ = tx.send(DataUpdate::Loading(false));
                    });
                } else {
                    let _ = self.tx.send(DataUpdate::Error(format!("Namespace kind is {}, not group", namespace.kind)));
                }
            } else {
                let _ = self.tx.send(DataUpdate::Error("Project has no namespace".into()));
            }
        }
        Ok(())
    }
}
