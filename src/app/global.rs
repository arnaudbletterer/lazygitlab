use gitlab::api::AsyncQuery;
use crate::models::*;
use crate::api::*;

impl App {
    pub async fn fetch_projects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        self.is_loading = true;
        
        tokio::spawn(async move {
            let ep = PagedProjects { membership: true, page: 1, per_page: 20 };
            let res: Result<Vec<Project>, _> = ep.query_async(&client).await;
            match res {
                Ok(data) => { 
                    let has_more = data.len() == 20;
                    let _ = tx.send(DataUpdate::Projects { data, page: 1, has_more }); 
                }
                Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
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
        self.is_loading = true;

        tokio::spawn(async move {
            let ep = PagedProjects { membership: true, page: next_page, per_page: 20 };
            let res: Result<Vec<Project>, _> = ep.query_async(&client).await;
            match res {
                Ok(data) => {
                    if !data.is_empty() {
                        let _ = tx.send(DataUpdate::AppendProjects(data));
                    }
                }
                Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_groups(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        self.is_loading = true;
        tokio::spawn(async move {
            let ep = Groups { page: Some(1), per_page: Some(20) };
            let res: Result<Vec<Group>, _> = ep.query_async(&client).await;
            match res {
                Ok(data) => {
                    let has_more = data.len() == 20;
                    let _ = tx.send(DataUpdate::Groups { data, page: 1, has_more });
                }
                Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
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
        self.is_loading = true;
        tokio::spawn(async move {
            let ep = Groups { page: Some(next_page), per_page: Some(20) };
            let res: Result<Vec<Group>, _> = ep.query_async(&client).await;
            match res {
                Ok(data) => {
                    if !data.is_empty() {
                        let _ = tx.send(DataUpdate::AppendGroups(data));
                    }
                }
                Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_todos(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        self.is_loading = true;
        
        tokio::spawn(async move {
            let ep = Todos { page: Some(1), per_page: Some(20) };
            let res: Result<Vec<Todo>, _> = ep.query_async(&client).await;
            match res {
                Ok(data) => { 
                    let has_more = data.len() == 20;
                    let _ = tx.send(DataUpdate::Todos { data, page: 1, has_more }); 
                }
                Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
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
        self.is_loading = true;

        tokio::spawn(async move {
            let ep = Todos { page: Some(next_page), per_page: Some(20) };
            let res: Result<Vec<Todo>, _> = ep.query_async(&client).await;
            match res {
                Ok(data) => {
                    if !data.is_empty() {
                        let _ = tx.send(DataUpdate::AppendTodos(data));
                    }
                }
                Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_global_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        self.is_loading = true;
        tokio::spawn(async move {
            let ep = GlobalMergeRequests {
                scope: Some("all".into()),
                state: Some("opened".into()),
                membership: Some(true),
                page: Some(1),
                per_page: Some(20),
            };
            let res: Result<Vec<MergeRequest>, _> = ep.query_async(&client).await;
            match res {
                Ok(data) => { 
                    let has_more = data.len() == 20;
                    let _ = tx.send(DataUpdate::GlobalMRs { data, page: 1, has_more }); 
                }
                Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_global_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.mrs_has_more || self.is_loading { return Ok(()); }
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let next_page = self.mrs_page + 1;
        self.is_loading = true;
        tokio::spawn(async move {
            let ep = GlobalMergeRequests {
                scope: Some("all".into()),
                state: Some("opened".into()),
                membership: Some(true),
                page: Some(next_page),
                per_page: Some(20),
            };
            let res: Result<Vec<MergeRequest>, _> = ep.query_async(&client).await;
            match res {
                Ok(data) => {
                    if !data.is_empty() {
                        let _ = tx.send(DataUpdate::AppendGlobalMRs(data));
                    }
                }
                Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_global_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        self.is_loading = true;
        tokio::spawn(async move {
            let ep = GlobalIssues {
                scope: Some("all".into()),
                state: Some("opened".into()),
                membership: Some(true),
                page: Some(1),
                per_page: Some(20),
            };
            let res: Result<Vec<Issue>, _> = ep.query_async(&client).await;
            match res {
                Ok(data) => { 
                    let has_more = data.len() == 20;
                    let _ = tx.send(DataUpdate::GlobalIssues { data, page: 1, has_more }); 
                }
                Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_global_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.issues_has_more || self.is_loading { return Ok(()); }
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let next_page = self.issues_page + 1;
        self.is_loading = true;
        tokio::spawn(async move {
            let ep = GlobalIssues {
                scope: Some("all".into()),
                state: Some("opened".into()),
                membership: Some(true),
                page: Some(next_page),
                per_page: Some(20),
            };
            let res: Result<Vec<Issue>, _> = ep.query_async(&client).await;
            match res {
                Ok(data) => {
                    if !data.is_empty() {
                        let _ = tx.send(DataUpdate::AppendGlobalIssues(data));
                    }
                }
                Err(e) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{path, method};
    use gitlab::GitlabBuilder;
    use tokio_test::block_on;
    use tokio::sync::mpsc;

    async fn setup_test_app() -> (MockServer, App, mpsc::UnboundedReceiver<DataUpdate>) {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        let (tx, rx) = mpsc::unbounded_channel();
        let mut app = App::new(client);
        app.tx = tx;
        (server, app, rx)
    }

    #[test]
    fn test_global_fetching_and_pagination() {
        let (server, mut app, mut rx) = block_on(setup_test_app());

        block_on(async {
            use wiremock::matchers::query_param;

            // Projects
            let many_projects: Vec<serde_json::Value> = (0..20).map(|i| serde_json::json!({"id": i, "name": "p", "path_with_namespace": "n/p", "web_url": "u"})).collect();
            Mock::given(path("/api/v4/projects")).and(method("GET")).and(query_param("page", "1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(many_projects))).mount(&server).await;
            
            app.fetch_projects().await.unwrap();
            let start = std::time::Instant::now();
            while std::time::Instant::now().duration_since(start) < std::time::Duration::from_secs(2) {
                app.process_updates(&mut rx);
                if !app.is_loading && !app.projects.is_empty() { break; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            assert_eq!(app.projects.len(), 20);

            // Groups
            let many_groups: Vec<serde_json::Value> = (0..20).map(|i| serde_json::json!({"id": i, "name": "g", "path": "g", "full_path": "g"})).collect();
            Mock::given(path("/api/v4/groups")).and(method("GET")).and(query_param("page", "1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(many_groups))).mount(&server).await;
            
            app.fetch_groups().await.unwrap();
            let start = std::time::Instant::now();
            while std::time::Instant::now().duration_since(start) < std::time::Duration::from_secs(2) {
                app.process_updates(&mut rx);
                if !app.is_loading && !app.groups.is_empty() { break; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            assert_eq!(app.groups.len(), 20);

            // Todos
            let many_todos: Vec<serde_json::Value> = (0..20).map(|i| serde_json::json!({"id": i, "project": {"id":1,"name":"p","path_with_namespace":"n/p","web_url":"u"}, "action_name": "assigned", "target_type": "Issue", "target_url": "u", "body": "b", "state": "pending", "created_at": "t"})).collect();
            Mock::given(path("/api/v4/todos")).and(method("GET")).and(query_param("page", "1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(many_todos))).mount(&server).await;
            
            app.fetch_todos().await.unwrap();
            let start = std::time::Instant::now();
            while std::time::Instant::now().duration_since(start) < std::time::Duration::from_secs(2) {
                app.process_updates(&mut rx);
                if !app.is_loading && !app.todos.is_empty() { break; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            assert_eq!(app.todos.len(), 20);
            Mock::given(path("/api/v4/projects")).and(method("GET")).and(query_param("page", "2")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(many_projects))).mount(&server).await;
            app.projects_has_more = true; app.projects_page = 1; app.is_loading = false;
            app.fetch_next_projects().await.unwrap();
            let start = std::time::Instant::now();
            while std::time::Instant::now().duration_since(start) < std::time::Duration::from_secs(2) {
                app.process_updates(&mut rx);
                if app.projects.len() > 20 { break; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            assert!(app.projects.len() > 20);

            Mock::given(path("/api/v4/groups")).and(method("GET")).and(query_param("page", "2")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(vec![serde_json::json!({"id":2,"name":"g2","path":"g2","full_path":"g2"})]))).mount(&server).await;
            app.groups_has_more = true; app.groups_page = 1; app.is_loading = false;
            app.fetch_next_groups().await.unwrap();
            let start = std::time::Instant::now();
            while std::time::Instant::now().duration_since(start) < std::time::Duration::from_secs(2) {
                app.process_updates(&mut rx);
                if app.groups.len() > 1 { break; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            assert!(app.groups.len() > 1);

            Mock::given(path("/api/v4/todos")).and(method("GET")).and(query_param("page", "2")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(vec![serde_json::json!({"id":2,"project":{"id":1,"name":"p","path_with_namespace":"n/p","web_url":"u"},"action_name":"a","target_type":"i","target_url":"u","body":"b","state":"p","created_at":"t"})]))).mount(&server).await;
            app.todos_has_more = true; app.todos_page = 1; app.is_loading = false;
            app.fetch_next_todos().await.unwrap();
            let start = std::time::Instant::now();
            while std::time::Instant::now().duration_since(start) < std::time::Duration::from_secs(2) {
                app.process_updates(&mut rx);
                if app.todos.len() > 20 { break; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            assert!(app.todos.len() > 20);

            Mock::given(path("/api/v4/merge_requests")).and(method("GET")).and(query_param("page", "2")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(vec![serde_json::json!({"iid":2,"project_id":1,"title":"m","author":{"id":1,"username":"u"},"web_url":"u","state":"o","references":{"full":"r"},"upvotes":0,"downvotes":0,"labels":[],"milestone":null,"assignees":[],"created_at":"t"})]))).mount(&server).await;
            app.mrs_has_more = true; app.mrs_page = 1; app.is_loading = false;
            app.fetch_next_global_merge_requests().await.unwrap();
            let start = std::time::Instant::now();
            while std::time::Instant::now().duration_since(start) < std::time::Duration::from_secs(2) {
                app.process_updates(&mut rx);
                if app.merge_requests.len() > 0 { break; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            
            Mock::given(path("/api/v4/issues")).and(method("GET")).and(query_param("page", "2")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(vec![serde_json::json!({"iid":2,"project_id":1,"title":"i","author":{"id":1,"username":"u"},"assignees":[],"state":"o","web_url":"u","references":{"full":"r"},"labels":[],"milestone":null,"due_date":null,"weight":null,"confidential":false,"created_at":"t","upvotes":0,"downvotes":0})]))).mount(&server).await;
            app.issues_has_more = true; app.issues_page = 1; app.is_loading = false;
            app.fetch_next_global_issues().await.unwrap();
            let start = std::time::Instant::now();
            while std::time::Instant::now().duration_since(start) < std::time::Duration::from_secs(2) {
                app.process_updates(&mut rx);
                if app.issues.len() > 0 { break; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        });
    }
}
