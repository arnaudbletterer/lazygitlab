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
        self.projects_page += 1;
        let endpoint = PagedProjects { membership: true, page: self.projects_page, per_page: 20 };
        let res: Vec<Project> = endpoint.query_async(&self.gitlab).await?;
        if res.is_empty() {
            self.projects_has_more = false;
        } else {
            self.projects.extend(res.clone());
            self.projects_has_more = res.len() >= 20;
            self.filter_lists();
        }
        self.is_loading = false;
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
        self.groups_page += 1;
        let endpoint = Groups {
            page: Some(self.groups_page),
            per_page: Some(20),
        };
        let res: Vec<Group> = endpoint.query_async(&self.gitlab).await?;
        if res.is_empty() {
            self.groups_has_more = false;
        } else {
            self.groups.extend(res.clone());
            self.groups_has_more = res.len() >= 20;
            self.filter_lists();
        }
        self.is_loading = false;
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
        self.todos_page += 1;
        let endpoint = Todos {
            page: Some(self.todos_page),
            per_page: Some(20),
        };
        let res: Vec<Todo> = endpoint.query_async(&self.gitlab).await?;
        
        if res.is_empty() {
            self.todos_has_more = false;
        } else {
            self.todos.extend(res.clone());
            self.todos_has_more = res.len() >= 20;
            self.filter_lists();
        }
        self.is_loading = false;
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
        self.mrs_page += 1;
        let endpoint = GlobalMergeRequests {
            scope: Some("assigned_to_me".into()),
            page: Some(self.mrs_page),
            per_page: Some(20),
        };
        let res: Vec<MergeRequest> = endpoint.query_async(&self.gitlab).await?;
        
        if res.is_empty() {
            self.mrs_has_more = false;
        } else {
            self.merge_requests.extend(res.clone());
            self.mrs_has_more = res.len() >= 20;
            self.filter_lists();
        }
        self.is_loading = false;
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
        self.issues_page += 1;
        let endpoint = GlobalIssues {
            scope: Some("assigned_to_me".into()),
            page: Some(self.issues_page),
            per_page: Some(20),
        };
        let res: Vec<Issue> = endpoint.query_async(&self.gitlab).await?;
        
        if res.is_empty() {
            self.issues_has_more = false;
        } else {
            self.issues.extend(res.clone());
            self.issues_has_more = res.len() >= 20;
            self.filter_lists();
        }
        self.is_loading = false;
        Ok(())
    }
}
