use crate::api::*;
use crate::models::*;
use gitlab::api::AsyncQuery;

impl App {
    pub async fn fetch_projects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        self.status_message = Some("Fetching Projects...".into());

        tokio::spawn(async move {
            let ep = PagedProjects {
                membership: true,
                search: None,
                page: 1,
                per_page: 50,
                simple: Some(true),
                archived: Some(false),
                order_by: Some("last_activity_at".into()),
            };
            let res: Result<Result<Vec<Project>, _>, _> =
                tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client))
                    .await;
            match res {
                Ok(Ok(data)) => {
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::Projects {
                        data,
                        page: 1,
                        has_more,
                    });
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Projects fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Projects fetch timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_projects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.projects_has_more || self.is_loading {
            return Ok(());
        }
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let next_page = self.projects_page + 1;
        let _ = tx.send(DataUpdate::Loading(true));

        tokio::spawn(async move {
            let ep = PagedProjects {
                membership: true,
                search: None,
                page: next_page,
                per_page: 50,
                simple: Some(true),
                archived: Some(false),
                order_by: Some("last_activity_at".into()),
            };
            let res: Result<Result<Vec<Project>, _>, _> =
                tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client))
                    .await;
            match res {
                Ok(Ok(data)) => {
                    if !data.is_empty() {
                        let _ = tx.send(DataUpdate::AppendProjects(data));
                    }
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Projects fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Projects fetch timed out".into()));
                }
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
            let ep = Groups {
                page: Some(1),
                per_page: Some(50),
                min_access_level: Some(10),
                top_level_only: None,
            };
            let res: Result<Result<Vec<Group>, _>, _> =
                tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client))
                    .await;
            match res {
                Ok(Ok(data)) => {
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::Groups {
                        data,
                        page: 1,
                        has_more,
                    });
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Groups fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Groups fetch timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_groups(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.groups_has_more || self.is_loading {
            return Ok(());
        }
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let next_page = self.groups_page + 1;
        let _ = tx.send(DataUpdate::Loading(true));

        tokio::spawn(async move {
            let ep = Groups {
                page: Some(next_page),
                per_page: Some(50),
                min_access_level: Some(10),
                top_level_only: None,
            };
            let res: Result<Result<Vec<Group>, _>, _> =
                tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client))
                    .await;
            match res {
                Ok(Ok(data)) => {
                    if !data.is_empty() {
                        let _ = tx.send(DataUpdate::AppendGroups(data));
                    } else {
                        let _ = tx.send(DataUpdate::Loading(false));
                    }
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Groups fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Groups fetch timed out".into()));
                }
            }
        });
        Ok(())
    }

    pub async fn fetch_todos_conditional(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.todos_state_param();
        let polled = poll_todos(&self.gitlab, state, self.todos_etag.clone())
            .await
            .map_err(|e| e.to_string())?;
        if let Some((etag, data)) = polled {
            if etag.is_some() {
                self.todos_etag = etag;
            }
            let has_more = data.len() == 50;
            let _ = self.tx.send(DataUpdate::Todos {
                data,
                page: 1,
                has_more,
            });
        }
        Ok(())
    }

    fn todos_state_param(&self) -> &'static str {
        match self.todo_filter {
            TodoFilter::Pending | TodoFilter::Snoozed => "pending",
            TodoFilter::Done => "done",
        }
    }

    /// Non-blocking variant of `fetch_todos_conditional`: the request runs on a
    /// spawned task and reports through `DataUpdate`. No-op while a poll is in flight.
    pub fn spawn_todos_poll(&mut self) {
        if self.todos_poll_in_flight {
            return;
        }
        self.todos_poll_in_flight = true;
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let state = self.todos_state_param();
        let etag = self.todos_etag.clone();
        tokio::spawn(async move {
            match poll_todos(&client, state, etag).await {
                Ok(Some((new_etag, data))) => {
                    if let Some(e) = new_etag {
                        let _ = tx.send(DataUpdate::TodosEtag(e));
                    }
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::Todos {
                        data,
                        page: 1,
                        has_more,
                    });
                    let _ = tx.send(DataUpdate::TodosPollDone { failed: false });
                }
                Ok(None) => {
                    let _ = tx.send(DataUpdate::TodosPollDone { failed: false });
                }
                Err(e) => {
                    tracing::debug!("conditional todos poll failed, falling back: {}", e);
                    let _ = tx.send(DataUpdate::TodosPollDone { failed: true });
                }
            }
        });
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
            let ep = Todos {
                page: Some(1),
                per_page: Some(50),
                state: Some(state_str),
            };
            let res: Result<Result<Vec<Todo>, _>, _> =
                tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client))
                    .await;
            match res {
                Ok(Ok(data)) => {
                    let has_more = data.len() == 50;
                    let _ = tx.send(DataUpdate::Todos {
                        data,
                        page: 1,
                        has_more,
                    });
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Todos fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Todos fetch timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_todos(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.todos_has_more || self.is_loading {
            return Ok(());
        }
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
            let ep = Todos {
                page: Some(next_page),
                per_page: Some(50),
                state: Some(state_str),
            };
            let res: Result<Result<Vec<Todo>, _>, _> =
                tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client))
                    .await;
            match res {
                Ok(Ok(data)) => {
                    if !data.is_empty() {
                        let _ = tx.send(DataUpdate::AppendTodos(data));
                    } else {
                        let _ = tx.send(DataUpdate::Loading(false));
                    }
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Todos fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Todos fetch timed out".into()));
                }
            }
        });
        Ok(())
    }

    pub async fn snooze_todo(&mut self, until: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(i) = self.todos_state.selected()
            && let Some(GroupedItem::Item(todo)) = self.filtered_todos.get(i)
        {
            let tid = todo.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let ep = SnoozeTodo {
                    todo_id: tid,
                    snooze_until: until,
                };
                let _: Result<serde_json::Value, _> = ep.query_async(&client).await;
                let _ = tx.send(DataUpdate::TriggerRefresh);
            });
        }
        self.show_snooze_popup = false;
        Ok(())
    }

    pub async fn mark_todo_as_done(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(i) = self.todos_state.selected()
            && let Some(GroupedItem::Item(todo)) = self.filtered_todos.get(i)
        {
            let tid = todo.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let ep = MarkTodoAsDone { todo_id: tid };
                let _: Result<serde_json::Value, _> = ep.query_async(&client).await;
                let _ = tx.send(DataUpdate::TriggerRefresh);
            });
        }
        Ok(())
    }

    pub async fn fetch_repository_tree(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let pid = project.id;
            let ref_name = self.repository_ref.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = RepositoryTree {
                    project: pid,
                    path: None,
                    ref_name,
                    recursive: Some(true),
                };
                let res: Result<Result<Vec<RepositoryTreeItem>, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    gitlab::api::paged(ep, gitlab::api::Pagination::All).query_async(&c),
                )
                .await;
                match res {
                    Ok(Ok(data)) => {
                        let _ = tx.send(DataUpdate::RepositoryTree {
                            project_id: pid,
                            data,
                        });
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!(
                            "Repository tree fetch failed: {}",
                            e
                        )));
                    }
                    Err(_) => {
                        let _ =
                            tx.send(DataUpdate::Error("Repository tree fetch timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn select_repository_ref(
        &mut self,
        name: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.repository_ref = Some(name.clone());
        self.repository_tree.clear();
        self.selected_file_content = None;
        self.file_scroll = 0;
        self.repository_tree_state = tui_tree_widget::TreeState::default();
        self.status_message = Some(format!("Browsing tree at '{}'", name));
        self.fetch_repository_tree().await
    }

    pub async fn fetch_repository_file(
        &mut self,
        file_path: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let pid = project.id;
            let ref_name = match &self.repository_ref {
                Some(r) => r.clone(),
                None => project
                    .default_branch
                    .clone()
                    .unwrap_or_else(|| "main".to_string()),
            };
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let endpoint = gitlab::api::projects::repository::files::File::builder()
                    .project(pid)
                    .file_path(file_path.clone())
                    .ref_(ref_name)
                    .build()
                    .unwrap();
                let res: Result<Result<RepositoryFile, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    endpoint.query_async(&c),
                )
                .await;
                match res {
                    Ok(Ok(data)) => {
                        let _ = tx.send(DataUpdate::RepositoryFile {
                            project_id: pid,
                            file_path,
                            data,
                        });
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!(
                            "Repository file fetch failed: {}",
                            e
                        )));
                    }
                    Err(_) => {
                        let _ =
                            tx.send(DataUpdate::Error("Repository file fetch timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_commits(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let pid = project.id;
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = Commits {
                    project: pid,
                    ref_name: None,
                };
                let res: Result<Result<Vec<Commit>, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    gitlab::api::paged(ep, gitlab::api::Pagination::Limit(50)).query_async(&c),
                )
                .await;
                match res {
                    Ok(Ok(data)) => {
                        let has_more = data.len() == 50;
                        let _ = tx.send(DataUpdate::Commits {
                            project_id: pid,
                            data,
                            page: 1,
                            has_more,
                        });
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!("Commits fetch failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Commits fetch timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_next_commits(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.commits_has_more || self.is_loading {
            return Ok(());
        }
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let c = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = Commits {
                    project: pid,
                    ref_name: None,
                };
                let res: Result<Result<Vec<Commit>, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    gitlab::api::paged(ep, gitlab::api::Pagination::Limit(50)).query_async(&c),
                )
                .await;
                match res {
                    Ok(Ok(data)) if !data.is_empty() => {
                        let _ = tx.send(DataUpdate::AppendCommits {
                            project_id: pid,
                            data,
                        });
                    }
                    _ => {
                        let _ = tx.send(DataUpdate::Loading(false));
                    }
                }
            });
        }
        Ok(())
    }

    pub async fn fetch_branches(
        &mut self,
        project_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = ProjectBranches {
                project: project_id,
            };
            let res: Result<Result<Vec<Branch>, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                gitlab::api::paged(endpoint, gitlab::api::Pagination::Limit(100))
                    .query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(data)) => {
                    let _ = tx.send(DataUpdate::Branches {
                        project_id,
                        data,
                        page: 1,
                        has_more: false,
                    });
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(e.to_string()));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Branches fetch timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }
    pub async fn fetch_membership_project_ids(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_syncing_membership = true;
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let ep = PagedProjects {
                membership: true,
                search: None,
                page: 1,
                per_page: 100,
                simple: Some(true),
                archived: Some(false),
                order_by: None,
            };
            let res: Result<Vec<serde_json::Value>, _> =
                gitlab::api::paged(ep, gitlab::api::Pagination::All)
                    .query_async(&client)
                    .await;
            match res {
                Ok(data) => {
                    let mut ids = std::collections::HashSet::new();
                    for val in data {
                        if let Some(id) = val.get("id").and_then(|v| v.as_u64()) {
                            ids.insert(id);
                        }
                    }
                    tracing::info!("Membership sync complete: {} projects found", ids.len());
                    let _ = tx.send(DataUpdate::MemberProjectIds(ids));
                }
                Err(e) => {
                    tracing::error!("Membership sync failed: {}", e);
                    let _ = tx.send(DataUpdate::MemberProjectIds(
                        std::collections::HashSet::new(),
                    ));
                }
            }
        });
        Ok(())
    }

    pub fn copy_url(&mut self) {
        let url = if self.view_mode == ViewMode::Board {
            self.get_current_board_item_url()
        } else if let Some(mr) = &self.selected_mr {
            Some(mr.web_url.clone())
        } else if let Some(issue) = &self.selected_issue {
            Some(issue.web_url.clone())
        } else if let Some(pipeline) = &self.selected_pipeline {
            Some(pipeline.web_url.clone())
        } else if let Some(job) = &self.selected_job {
            Some(job.web_url.clone())
        } else if self.active_selection == MenuSelection::Projects {
            if let Some(idx) = self.projects_state.selected() {
                self.filtered_projects
                    .get(idx)
                    .and_then(|p| p.web_url.clone())
            } else {
                self.selected_project
                    .as_ref()
                    .and_then(|p| p.web_url.clone())
            }
        } else if self.active_selection == MenuSelection::Groups {
            if let Some(idx) = self.groups_state.selected() {
                self.filtered_groups
                    .get(idx)
                    .and_then(|g| g.web_url.clone())
            } else {
                None
            }
        } else if let Some(project) = &self.selected_project {
            project.web_url.clone()
        } else {
            None
        };

        if let Some(u) = url {
            self.copy_to_clipboard(u);
        }
    }

    pub fn open_in_browser(&self) {
        let url = if self.view_mode == ViewMode::Board {
            self.get_current_board_item_url()
        } else if let Some(mr) = &self.selected_mr {
            Some(mr.web_url.clone())
        } else if let Some(issue) = &self.selected_issue {
            Some(issue.web_url.clone())
        } else if let Some(pipeline) = &self.selected_pipeline {
            Some(pipeline.web_url.clone())
        } else if let Some(job) = &self.selected_job {
            Some(job.web_url.clone())
        } else if self.active_selection == MenuSelection::Projects {
            if let Some(idx) = self.projects_state.selected() {
                self.filtered_projects
                    .get(idx)
                    .and_then(|p| p.web_url.clone())
            } else {
                self.selected_project
                    .as_ref()
                    .and_then(|p| p.web_url.clone())
            }
        } else if self.active_selection == MenuSelection::Groups {
            if let Some(idx) = self.groups_state.selected() {
                self.filtered_groups
                    .get(idx)
                    .and_then(|g| g.web_url.clone())
            } else {
                None
            }
        } else if let Some(project) = &self.selected_project {
            project.web_url.clone()
        } else {
            None
        };

        #[cfg(test)]
        {
            let _ = url;
        }

        #[cfg(not(test))]
        if let Some(url) = url {
            if url == "u" {
                return;
            }
            if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with('/')
            {
                return;
            }
            #[cfg(target_os = "windows")]
            let _ = std::process::Command::new("cmd")
                .arg("/c")
                .arg("start")
                .arg(&url)
                .spawn();
            #[cfg(target_os = "macos")]
            let _ = std::process::Command::new("open").arg(&url).spawn();
            #[cfg(target_os = "linux")]
            let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
        }
    }
}

#[cfg(test)]
mod tests {
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
        let app = App::new(client, tx, None);
        (server, app, rx)
    }

    #[tokio::test]
    async fn test_global_fetching_logic() {
        let (server, mut app, mut rx) = setup_app().await;

        // Mock Projects
        Mock::given(path("/api/v4/projects"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!([{"id":1,"name":"p","path_with_namespace":"n/p"}]),
            ))
            .mount(&server)
            .await;
        // Mock Groups
        Mock::given(path("/api/v4/groups"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(
                    serde_json::json!([{"id":1,"name":"g","path":"g","full_path":"g"}]),
                ),
            )
            .mount(&server)
            .await;
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
        })
        .await;

        assert!(!app.projects.is_empty());
        assert!(!app.groups.is_empty());
        assert!(!app.todos.is_empty());

        // Test Actions
        app.todos_state.select(Some(0));
        app.filter_lists();

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;
        app.snooze_todo("t".into()).await.unwrap();
        app.mark_todo_as_done().await.unwrap();
    }
}

type PolledTodos = Option<(Option<String>, Vec<Todo>)>;

/// One conditional GET of `/todos`. `Ok(None)` means 304 Not Modified.
async fn poll_todos(
    client: &gitlab::AsyncGitlab,
    state: &str,
    etag: Option<String>,
) -> Result<PolledTodos, Box<dyn std::error::Error + Send + Sync>> {
    use gitlab::api::{AsyncClient, RestClient};
    let url = client.rest_endpoint(&format!("todos?per_page=50&state={}", state))?;
    let mut req = http::Request::builder().method("GET").uri(url.as_str());
    if let Some(etag) = etag {
        req = req.header(http::header::IF_NONE_MATCH, etag);
    }
    let resp = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        client.rest_async(req, Vec::new()),
    )
    .await??;
    if resp.status() == http::StatusCode::NOT_MODIFIED {
        return Ok(None);
    }
    if !resp.status().is_success() {
        return Err(format!("todos poll failed: {}", resp.status()).into());
    }
    let new_etag = resp
        .headers()
        .get(http::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let data: Vec<Todo> = serde_json::from_slice(resp.body())?;
    Ok(Some((new_etag, data)))
}
