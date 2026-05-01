use gitlab::api::AsyncQuery;
use crate::models::*;
use crate::api::*;

impl App {
    pub async fn fetch_projects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.projects_page = 1;
        self.projects_has_more = true;
        let endpoint = PagedProjects { membership: true, page: 1, per_page: 20 };
        let res: Vec<Project> = endpoint.query_async(&self.gitlab).await?;
        self.projects_has_more = res.len() == 20;
        self.projects = res;
        self.filter_lists();
        Ok(())
    }

    pub async fn fetch_next_projects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.projects_has_more || self.is_loading { return Ok(()); }
        self.is_loading = true;
        let endpoint = PagedProjects { membership: true, page: self.projects_page + 1, per_page: 20 };
        let res: Result<Vec<Project>, _> = endpoint.query_async(&self.gitlab).await;
        self.is_loading = false;
        
        match res {
            Ok(res) => {
                if res.is_empty() {
                    self.projects_has_more = false;
                } else {
                    self.projects_page += 1;
                    self.projects.extend(res.clone());
                    self.projects_has_more = res.len() >= 20;
                    self.filter_lists();
                }
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }

    pub async fn fetch_groups(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.groups_page = 1;
        self.groups_has_more = true;
        let endpoint = Groups {
            page: Some(1),
            per_page: Some(20),
        };
        let res: Vec<Group> = endpoint.query_async(&self.gitlab).await?;
        self.groups_has_more = res.len() == 20;
        self.groups = res;
        self.filter_lists();
        Ok(())
    }

    pub async fn fetch_next_groups(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.groups_has_more || self.is_loading { return Ok(()); }
        self.is_loading = true;
        let endpoint = Groups {
            page: Some(self.groups_page + 1),
            per_page: Some(20),
        };
        let res: Result<Vec<Group>, _> = endpoint.query_async(&self.gitlab).await;
        self.is_loading = false;

        match res {
            Ok(res) => {
                if res.is_empty() {
                    self.groups_has_more = false;
                } else {
                    self.groups_page += 1;
                    self.groups.extend(res.clone());
                    self.groups_has_more = res.len() >= 20;
                    self.filter_lists();
                }
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }

    pub async fn fetch_todos(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.todos_page = 1;
        self.todos_has_more = true;
        let endpoint = Todos {
            page: Some(1),
            per_page: Some(20),
        };
        let res: Vec<Todo> = endpoint.query_async(&self.gitlab).await?;
        self.todos_has_more = res.len() == 20;
        self.todos = res;
        self.filter_lists();
        Ok(())
    }

    pub async fn fetch_next_todos(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.todos_has_more || self.is_loading { return Ok(()); }
        
        self.is_loading = true;
        let endpoint = Todos {
            page: Some(self.todos_page + 1),
            per_page: Some(20),
        };
        let res: Result<Vec<Todo>, _> = endpoint.query_async(&self.gitlab).await;
        self.is_loading = false;
        
        match res {
            Ok(res) => {
                if res.is_empty() {
                    self.todos_has_more = false;
                } else {
                    self.todos_page += 1;
                    self.todos.extend(res.clone());
                    self.todos_has_more = res.len() >= 20;
                    self.filter_lists();
                }
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }

    pub async fn fetch_global_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.mrs_page = 1;
        self.mrs_has_more = true;
        let endpoint = GlobalMergeRequests {
            scope: Some("assigned_to_me".into()),
            page: Some(1),
            per_page: Some(20),
        };
        let res: Vec<MergeRequest> = endpoint.query_async(&self.gitlab).await?;
        self.mrs_has_more = res.len() == 20;
        self.merge_requests = res;
        self.filter_lists();
        Ok(())
    }

    pub async fn fetch_next_global_merge_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.mrs_has_more || self.is_loading { return Ok(()); }
        
        self.is_loading = true;
        let endpoint = GlobalMergeRequests {
            scope: Some("assigned_to_me".into()),
            page: Some(self.mrs_page + 1),
            per_page: Some(20),
        };
        let res: Result<Vec<MergeRequest>, _> = endpoint.query_async(&self.gitlab).await;
        self.is_loading = false;
        
        match res {
            Ok(res) => {
                if res.is_empty() {
                    self.mrs_has_more = false;
                } else {
                    self.mrs_page += 1;
                    self.merge_requests.extend(res.clone());
                    self.mrs_has_more = res.len() >= 20;
                    self.filter_lists();
                }
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }

    pub async fn fetch_global_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.issues_page = 1;
        self.issues_has_more = true;
        let endpoint = GlobalIssues {
            scope: Some("assigned_to_me".into()),
            page: Some(1),
            per_page: Some(20),
        };
        let res: Vec<Issue> = endpoint.query_async(&self.gitlab).await?;
        self.issues_has_more = res.len() == 20;
        self.issues = res;
        self.filter_lists();
        Ok(())
    }

    pub async fn fetch_next_global_issues(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.issues_has_more || self.is_loading { return Ok(()); }
        
        self.is_loading = true;
        let endpoint = GlobalIssues {
            scope: Some("assigned_to_me".into()),
            page: Some(self.issues_page + 1),
            per_page: Some(20),
        };
        let res: Result<Vec<Issue>, _> = endpoint.query_async(&self.gitlab).await;
        self.is_loading = false;
        
        match res {
            Ok(res) => {
                if res.is_empty() {
                    self.issues_has_more = false;
                } else {
                    self.issues_page += 1;
                    self.issues.extend(res.clone());
                    self.issues_has_more = res.len() >= 20;
                    self.filter_lists();
                }
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{path, method, any};
    use gitlab::GitlabBuilder;
    use tokio_test::block_on;

    async fn setup_test_app() -> (MockServer, App) {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        (server, App::new(client))
    }

    #[test]
    fn test_global_fetching_and_pagination() {
        let (server, mut app) = block_on(setup_test_app());

        block_on(async {
            use wiremock::matchers::query_param;

            // 1. Projects Pagination
            let many_projects: Vec<serde_json::Value> = (0..20).map(|i| serde_json::json!({"id": i, "name": "p", "path_with_namespace": "n/p", "web_url": "u"})).collect();
            Mock::given(path("/api/v4/projects")).and(method("GET")).and(query_param("page", "1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(many_projects))).mount(&server).await;
            Mock::given(path("/api/v4/projects")).and(method("GET")).and(query_param("page", "2")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([{"id": 100, "name": "p2", "path_with_namespace": "n/p2", "web_url": "u"}]))).mount(&server).await;
            Mock::given(path("/api/v4/projects")).and(method("GET")).and(query_param("page", "3")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
            
            app.fetch_projects().await.unwrap();
            assert_eq!(app.projects.len(), 20);
            app.fetch_next_projects().await.unwrap();
            assert_eq!(app.projects.len(), 21);
            app.fetch_next_projects().await.unwrap();
            assert!(!app.projects_has_more);

            // 2. Groups Pagination
            let many_groups: Vec<serde_json::Value> = (0..20).map(|i| serde_json::json!({"id": i, "name": "g", "path": "g", "full_path": "g"})).collect();
            Mock::given(path("/api/v4/groups")).and(method("GET")).and(query_param("page", "1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(many_groups))).mount(&server).await;
            Mock::given(path("/api/v4/groups")).and(method("GET")).and(query_param("page", "2")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
            
            app.fetch_groups().await.unwrap();
            assert_eq!(app.groups.len(), 20);
            app.fetch_next_groups().await.unwrap();
            assert!(!app.groups_has_more);

            // 3. Todos Pagination
            let many_todos: Vec<serde_json::Value> = (0..20).map(|i| serde_json::json!({"id": i, "project": {"id":1,"name":"p","path_with_namespace":"n/p","web_url":"u"}, "action_name": "assigned", "target_type": "Issue", "target_url": "u", "body": "b", "state": "pending", "created_at": "t"})).collect();
            Mock::given(path("/api/v4/todos")).and(method("GET")).and(query_param("page", "1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(many_todos))).mount(&server).await;
            Mock::given(path("/api/v4/todos")).and(method("GET")).and(query_param("page", "2")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
            
            app.fetch_todos().await.unwrap();
            assert_eq!(app.todos.len(), 20);
            app.fetch_next_todos().await.unwrap();
            assert!(!app.todos_has_more);

            // 4. Global MRs Pagination
            let many_mrs: Vec<serde_json::Value> = (0..20).map(|i| serde_json::json!({"iid": i, "project_id": 1, "title": "M", "description": "d", "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "references": {"full": "r"}, "upvotes": 0, "downvotes": 0, "labels": [], "milestone": null, "assignees": [], "created_at": "t", "reactions": []})).collect();
            Mock::given(path("/api/v4/merge_requests")).and(method("GET")).and(query_param("page", "1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(many_mrs))).mount(&server).await;
            Mock::given(path("/api/v4/merge_requests")).and(method("GET")).and(query_param("page", "2")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
            
            app.fetch_global_merge_requests().await.unwrap();
            assert_eq!(app.merge_requests.len(), 20);
            app.fetch_next_global_merge_requests().await.unwrap();
            assert!(!app.mrs_has_more);

            // 5. Global Issues Pagination
            let many_issues: Vec<serde_json::Value> = (0..20).map(|i| serde_json::json!({"iid": i, "project_id": 1, "title": "I", "description": "d", "author": {"id": 1, "username": "u"}, "assignees": [], "state": "opened", "web_url": "u", "labels": [], "milestone": null, "due_date": null, "weight": 1, "confidential": false, "created_at": "t", "upvotes": 0, "downvotes": 0, "references": {"full": "n/p#1"}})).collect();
            Mock::given(path("/api/v4/issues")).and(method("GET")).and(query_param("page", "1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(many_issues))).mount(&server).await;
            Mock::given(path("/api/v4/issues")).and(method("GET")).and(query_param("page", "2")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
            
            app.fetch_global_issues().await.unwrap();
            assert_eq!(app.issues.len(), 20);
            app.fetch_next_global_issues().await.unwrap();
            assert!(!app.issues_has_more);

            // 6. Coverage for early returns (has_more = false)
            app.fetch_next_projects().await.unwrap();
            app.fetch_next_groups().await.unwrap();
            app.fetch_next_todos().await.unwrap();
            app.fetch_next_global_merge_requests().await.unwrap();
            app.fetch_next_global_issues().await.unwrap();

            // 7. Error handling
            server.reset().await;
            Mock::given(any()).respond_with(ResponseTemplate::new(500)).mount(&server).await;
            app.is_loading = false;
            
            // Initial fetches
            let _ = app.fetch_projects().await;
            let _ = app.fetch_groups().await;
            let _ = app.fetch_todos().await;
            let _ = app.fetch_global_merge_requests().await;
            let _ = app.fetch_global_issues().await;
            
            // Next page fetches (with has_more = true)
            app.projects_has_more = true;
            app.is_loading = false;
            let _ = app.fetch_next_projects().await;
            
            app.groups_has_more = true;
            app.is_loading = false;
            let _ = app.fetch_next_groups().await;
            
            app.todos_has_more = true;
            app.is_loading = false;
            let _ = app.fetch_next_todos().await;
            
            app.mrs_has_more = true;
            app.is_loading = false;
            let _ = app.fetch_next_global_merge_requests().await;
            
            app.issues_has_more = true;
            app.is_loading = false;
            let _ = app.fetch_next_global_issues().await;
        });
    }
}
