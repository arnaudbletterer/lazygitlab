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
                let ep = ProjectBoardLists { project: pid, board: board_id };
                let res: Result<Result<Vec<BoardList>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), paged(ep, gitlab::api::Pagination::Limit(100)).query_async(&client)).await;
                match res {
                    Ok(Ok(data)) => { let _ = tx.send(DataUpdate::BoardLists { project_id: pid, board_id, data }); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Board lists fetch timed out".into())); }
                }
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
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = ProjectBoardListIssues { project: pid, board: board_id, list: list_id };
                // Increase timeout to 60s for stress test projects
                let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(60), paged(ep, gitlab::api::Pagination::Limit(50)).query_async(&client)).await;
                match res {
                    Ok(Ok(data)) => { let _ = tx.send(DataUpdate::BoardIssues { project_id: pid, board_id, list_id, data }); }
                    Ok(Err(e)) => { 
                        let err_msg = e.to_string();
                        // If it's a 404, maybe the list was deleted or is special
                        if err_msg.contains("404") {
                            let _ = tx.send(DataUpdate::BoardIssues { project_id: pid, board_id, list_id, data: Vec::new() });
                        } else {
                            let _ = tx.send(DataUpdate::Error(format!("List {} error: {}", list_id, err_msg))); 
                        }
                    }
                    Err(_) => { let _ = tx.send(DataUpdate::Error(format!("List {} fetch timed out", list_id))); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
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
