use gitlab::api::AsyncQuery;
use crate::models::*;
use crate::api::*;

impl App {
    pub async fn fetch_projects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        
        tokio::spawn(async move {
            let ep = PagedProjects { membership: true, page: 1, per_page: 50 };
            let res: Result<Result<Vec<Project>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { 
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::Projects { data, page: 1, has_more }); 
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Projects fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_projects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.projects_has_more || self.is_loading { return Ok(()); }
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let next_page = self.projects_page + 1;
        let _ = tx.send(DataUpdate::Loading(true));

        tokio::spawn(async move {
            let ep = PagedProjects { membership: true, page: next_page, per_page: 50 };
            let res: Result<Result<Vec<Project>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => {
                    if !data.is_empty() {
                        let _ = tx.send(DataUpdate::AppendProjects(data));
                    }
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Projects fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_groups(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let ep = Groups { page: Some(1), per_page: Some(50) };
            let res: Result<Result<Vec<Group>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => {
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::Groups { data, page: 1, has_more });
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Groups fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_groups(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.groups_has_more || self.is_loading { return Ok(()); }
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let next_page = self.groups_page + 1;
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let ep = Groups { page: Some(next_page), per_page: Some(50) };
            let res: Result<Result<Vec<Group>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => {
                    if !data.is_empty() {
                        let _ = tx.send(DataUpdate::AppendGroups(data));
                    }
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Groups fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_todos(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        self.todos.clear(); self.filter_lists(); // Prevent stale selection
        self.status_message = Some("Fetching Todos...".into());
        let _ = tx.send(DataUpdate::Loading(true));
        
        tokio::spawn(async move {
            let ep = Todos { page: Some(1), per_page: Some(50) };
            let res: Result<Result<Vec<Todo>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { 
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::Todos { data, page: 1, has_more }); 
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Todos fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_todos(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.todos_has_more || self.is_loading { return Ok(()); }
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let next_page = self.todos_page + 1;
        let _ = tx.send(DataUpdate::Loading(true));

        tokio::spawn(async move {
            let ep = Todos { page: Some(next_page), per_page: Some(50) };
            let res: Result<Result<Vec<Todo>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => {
                    if !data.is_empty() {
                        let _ = tx.send(DataUpdate::AppendTodos(data));
                    }
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Todos fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_global_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        self.merge_requests.clear(); self.filter_lists(); // Prevent stale selection
        self.status_message = Some("Fetching Global Merge Requests...".into());
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let ep = GlobalMergeRequests {
                scope: Some("assigned_to_me".into()),
                state: Some("opened".into()),
                membership: Some(true),
                page: Some(1),
                per_page: Some(50),
            };
            let res: Result<Result<Vec<MergeRequest>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { 
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::GlobalMRs { data, page: 1, has_more }); 
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Global Merge Requests fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_global_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        self.issues.clear(); self.filter_lists(); // Prevent stale selection
        self.status_message = Some("Fetching Global Issues...".into());
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let ep = GlobalIssues {
                scope: Some("assigned_to_me".into()),
                state: Some("opened".into()),
                membership: Some(true),
                page: Some(1),
                per_page: Some(50),
            };
            let res: Result<Result<Vec<Issue>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { 
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::GlobalIssues { data, page: 1, has_more }); 
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Global Issues fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }
}
