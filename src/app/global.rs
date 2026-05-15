use gitlab::api::AsyncQuery;
use crate::models::*;
use crate::api::*;

impl App {
    pub async fn fetch_projects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        self.status_message = Some("Fetching Projects...".into());

        tokio::spawn(async move {
            let ep = PagedProjects { membership: true, page: 1, per_page: 50, simple: Some(true), archived: Some(false), order_by: Some("last_activity_at".into()) };
            let res: Result<Result<Vec<Project>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { 
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::Projects { data, page: 1, has_more }); 
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Projects fetch failed: {}", e))); }
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
            let ep = PagedProjects { membership: true, page: next_page, per_page: 50, simple: Some(true), archived: Some(false), order_by: Some("last_activity_at".into()) };
            let res: Result<Result<Vec<Project>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { if !data.is_empty() { let _ = tx.send(DataUpdate::AppendProjects(data)); } }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Projects fetch failed: {}", e))); }
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
        self.status_message = Some("Fetching Groups...".into());

        tokio::spawn(async move {
            let ep = Groups { page: Some(1), per_page: Some(50), min_access_level: Some(10), top_level_only: None };
            let res: Result<Result<Vec<Group>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { 
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::Groups { data, page: 1, has_more }); 
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Groups fetch failed: {}", e))); }
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
            let ep = Groups { page: Some(next_page), per_page: Some(50), min_access_level: Some(10), top_level_only: None };
            let res: Result<Result<Vec<Group>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { if !data.is_empty() { let _ = tx.send(DataUpdate::AppendGroups(data)); } else { let _ = tx.send(DataUpdate::Loading(false)); } }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Groups fetch failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Groups fetch timed out".into())); }
            }
        });
        Ok(())
    }

    pub async fn fetch_todos(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        self.status_message = Some("Fetching Todos...".into());
        let state_str = match self.todo_filter {
            TodoFilter::Pending => "pending".to_string(),
            TodoFilter::Snoozed => "pending".to_string(), // Snoozed are still pending in GL
            TodoFilter::Done => "done".to_string(),
        };

        tokio::spawn(async move {
            let ep = Todos { page: Some(1), per_page: Some(50), state: Some(state_str) };
            let res: Result<Result<Vec<Todo>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { 
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::Todos { data, page: 1, has_more }); 
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Todos fetch failed: {}", e))); }
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
        let state_str = match self.todo_filter {
            TodoFilter::Pending => "pending".to_string(),
            TodoFilter::Snoozed => "pending".to_string(),
            TodoFilter::Done => "done".to_string(),
        };
        tokio::spawn(async move {
            let ep = Todos { page: Some(next_page), per_page: Some(50), state: Some(state_str) };
            let res: Result<Result<Vec<Todo>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { if !data.is_empty() { let _ = tx.send(DataUpdate::AppendTodos(data)); } else { let _ = tx.send(DataUpdate::Loading(false)); } }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Todos fetch failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Todos fetch timed out".into())); }
            }
        });
        Ok(())
    }

    pub async fn snooze_todo(&mut self, until: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(i) = self.todos_state.selected() {
            if let Some(GroupedItem::Item(todo)) = self.filtered_todos.get(i) {
                let tid = todo.id; let client = self.gitlab.clone(); let tx = self.tx.clone();
                tokio::spawn(async move {
                    let ep = SnoozeTodo { todo_id: tid, snooze_until: until };
                    let _: Result<serde_json::Value, _> = ep.query_async(&client).await;
                    let _ = tx.send(DataUpdate::TriggerRefresh);
                });
            }
        }
        self.show_snooze_popup = false;
        Ok(())
    }

    pub async fn mark_todo_as_done(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(i) = self.todos_state.selected() {
            if let Some(GroupedItem::Item(todo)) = self.filtered_todos.get(i) {
                let tid = todo.id; let client = self.gitlab.clone(); let tx = self.tx.clone();
                tokio::spawn(async move {
                    let ep = MarkTodoAsDone { todo_id: tid };
                    let _: Result<serde_json::Value, _> = ep.query_async(&client).await;
                    let _ = tx.send(DataUpdate::TriggerRefresh);
                });
            }
        }
        Ok(())
    }

    pub async fn fetch_repository_tree(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let c = self.gitlab.clone(); let tx = self.tx.clone(); let pid = project.id;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = RepositoryTree { project: pid, path: None, ref_name: None, recursive: Some(true) };
                let res: Result<Result<Vec<RepositoryTreeItem>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), gitlab::api::paged(ep, gitlab::api::Pagination::All).query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => { let _ = tx.send(DataUpdate::RepositoryTree { project_id: pid, data }); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Repository tree fetch failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Repository tree fetch timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_repository_file(&mut self, file_path: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let c = self.gitlab.clone(); let tx = self.tx.clone(); let pid = project.id;
            let ref_name = project.default_branch.clone().unwrap_or_else(|| "main".to_string());
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = gitlab::api::projects::repository::files::File::builder()
                    .project(pid)
                    .file_path(file_path.clone())
                    .ref_(ref_name)
                    .build()
                    .unwrap();
                let res: Result<Result<RepositoryFile, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => { let _ = tx.send(DataUpdate::RepositoryFile { project_id: pid, file_path, data }); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Repository file fetch failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Repository file fetch timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_commits(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let c = self.gitlab.clone(); let tx = self.tx.clone(); let pid = project.id;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = Commits { project: pid, ref_name: None };
                let res: Result<Result<Vec<Commit>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), gitlab::api::paged(ep, gitlab::api::Pagination::Limit(50)).query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => { let has_more = data.len() == 50; let _ = tx.send(DataUpdate::Commits { project_id: pid, data, page: 1, has_more }); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Commits fetch failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Commits fetch timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_next_commits(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.commits_has_more || self.is_loading { return Ok(()); }
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = Commits { project: pid, ref_name: None };
                let res: Result<Result<Vec<Commit>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), gitlab::api::paged(ep, gitlab::api::Pagination::Limit(50)).query_async(&c)).await;
                match res {
                    Ok(Ok(data)) => { 
                        if !data.is_empty() { let _ = tx.send(DataUpdate::AppendCommits { project_id: pid, data }); }
                        else { let _ = tx.send(DataUpdate::Loading(false)); }
                    }
                    _ => { let _ = tx.send(DataUpdate::Loading(false)); }
                }
            });
        }
        Ok(())
    }

    pub async fn fetch_branches(&mut self, project_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = ProjectBranches { project: project_id };
            let res: Result<Result<Vec<Branch>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), gitlab::api::paged(endpoint, gitlab::api::Pagination::Limit(100)).query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { let _ = tx.send(DataUpdate::Branches { project_id, data, page: 1, has_more: false }); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Branches fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }
    pub fn copy_url(&mut self) {
        let url = if let Some(mr) = &self.selected_mr { 
            Some(mr.web_url.clone()) 
        } else if let Some(issue) = &self.selected_issue { 
            Some(issue.web_url.clone()) 
        } else if let Some(pipeline) = &self.selected_pipeline { 
            Some(pipeline.web_url.clone()) 
        } else if let Some(job) = &self.selected_job { 
            Some(job.web_url.clone()) 
        } else if self.active_selection == MenuSelection::Projects { 
            if let Some(idx) = self.projects_state.selected() { 
                self.filtered_projects.get(idx).and_then(|p| p.web_url.clone()) 
            } else { 
                self.selected_project.as_ref().and_then(|p| p.web_url.clone()) 
            } 
        } else if self.active_selection == MenuSelection::Groups { 
            if let Some(idx) = self.groups_state.selected() { 
                self.filtered_groups.get(idx).and_then(|g| g.web_url.clone()) 
            } else { 
                None 
            } 
        } else if let Some(project) = &self.selected_project { 
            project.web_url.clone() 
        } else { 
            None 
        };

        if let Some(url) = url {
            self.copy_to_clipboard(url);
        }
    }
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
        let mut app = App::new(client);
        app.tx = tx;
        (server, app, rx)
    }

    #[tokio::test]
    async fn test_global_fetching_logic() {
        let (server, mut app, mut rx) = setup_app().await;
        
        // Mock Projects
        Mock::given(path("/api/v4/projects")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([{"id":1,"name":"p","path_with_namespace":"n/p"}]))).mount(&server).await;
        // Mock Groups
        Mock::given(path("/api/v4/groups")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([{"id":1,"name":"g","path":"g","full_path":"g"}]))).mount(&server).await;
        // Mock Todos
        Mock::given(path("/api/v4/todos")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([{"id":1,"project":{"id":1,"name":"p"},"action_name":"a","target_type":"Issue","target_url":"u","body":"b","state":"pending","created_at":"t"}]))).mount(&server).await;

        app.fetch_projects().await.unwrap();
        app.fetch_groups().await.unwrap();
        app.fetch_todos().await.unwrap();

        let _ = tokio::time::timeout(Duration::from_secs(2), async {
            let mut count = 0;
            while count < 3 {
                app.process_updates(&mut rx).await;
                tokio::time::sleep(Duration::from_millis(10)).await;
                count += 1;
            }
        }).await;

        assert!(!app.projects.is_empty());
        assert!(!app.groups.is_empty());
        assert!(!app.todos.is_empty());

        // Test Actions
        app.todos_state.select(Some(0));
        app.filter_lists();
        
        Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
        app.snooze_todo("t".into()).await.unwrap();
        app.mark_todo_as_done().await.unwrap();
    }
}
