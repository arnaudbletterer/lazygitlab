use gitlab::api::{AsyncQuery, paged};
use crate::models::*;
use crate::api::*;
use regex::Regex;

impl App {
    pub async fn fetch_pipelines(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            self.status_message = Some("Fetching Pipelines...".into());

            tokio::spawn(async move {
                let ep = ProjectPipelines { project: pid, page: Some(1), per_page: Some(50) };
                let res: Result<Result<Vec<Pipeline>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
                match res {
                    Ok(Ok(data)) => { 
                        let has_more = data.len() == 50;
                        let _ = tx.send(DataUpdate::ProjectPipelines { project_id: pid, data, page: 1, has_more }); 
                    }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Pipelines fetch timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_next_pipelines(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.pipelines_has_more || self.is_loading { return Ok(()); }

        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let next_page = self.pipelines_page + 1;
            let _ = tx.send(DataUpdate::Loading(true));

            tokio::spawn(async move {
                let ep = ProjectPipelines { 
                    project: pid,
                    page: Some(next_page),
                    per_page: Some(50),
                };
                let res: Result<Result<Vec<Pipeline>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
                match res {
                    Ok(Ok(data)) => {
                        if !data.is_empty() {
                            let _ = tx.send(DataUpdate::AppendProjectPipelines { project_id: pid, data });
                        } else {
                            let _ = tx.send(DataUpdate::Loading(false));
                        }
                    }
                    Ok(Err(e)) => { 
                        let _ = tx.send(DataUpdate::Error(e.to_string())); 
                        let _ = tx.send(DataUpdate::Loading(false));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Pipelines fetch timed out".into()));
                        let _ = tx.send(DataUpdate::Loading(false));
                    }
                }
            });
        }
        Ok(())
    }

    pub async fn fetch_jobs(&mut self, project_id: u64, pipeline_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = ProjectPipelineJobs { project: project_id, pipeline: pipeline_id };
            let res: Result<Result<Vec<Job>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), paged(endpoint, gitlab::api::Pagination::Limit(100)).query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { let _ = tx.send(DataUpdate::Jobs { project_id, pipeline_id, data }); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Jobs fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_pipeline(&mut self, project_id: u64, pipeline_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = ProjectPipeline { project: project_id, pipeline: pipeline_id };
            let res: Result<Result<Pipeline, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_pipeline)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                _ => {}
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_job_log(&mut self, project_id: u64, job_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = JobTrace { project: project_id, job: job_id };
            let res: Result<Result<Vec<u8>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), gitlab::api::raw(endpoint).query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { let _ = tx.send(DataUpdate::Log { project_id, job_id, data: String::from_utf8_lossy(&data).to_string() }); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Job log fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_branches(&mut self, project_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = ProjectBranches { project: project_id };
            let _res: Result<Result<Vec<Branch>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), paged(endpoint, gitlab::api::Pagination::Limit(100)).query_async(&client)).await;
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub fn filter_branches(&mut self) {
        self.filtered_branches = filter_branch_names(&self.project_branches, &self.pipeline_ref_input);
        self.branches_state.select(if self.filtered_branches.is_empty() { None } else { Some(0) });
    }

    pub async fn show_pipeline_launch_popup(&mut self) {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let _ = self.fetch_branches(pid).await;
            self.show_pipeline_popup = true;
            self.pipeline_ref_input.clear();
            self.pipeline_variables.clear();
            self.pipeline_variable_key_input.clear();
            self.pipeline_variable_value_input.clear();
            self.input_mode = true;
            self.input_target = InputTarget::PipelineRef;
            self.cursor_index = 0;
            self.show_branches_dropdown = true;
            self.filter_branches();
        }
    }

    pub fn add_pipeline_variable(&mut self) {
        if !self.pipeline_variable_key_input.is_empty() {
            self.pipeline_variables.push(PipelineVariable {
                key: self.pipeline_variable_key_input.clone(),
                value: self.pipeline_variable_value_input.clone(),
            });
            self.pipeline_variable_key_input.clear();
            self.pipeline_variable_value_input.clear();
            self.pipeline_variables_state.select(Some(self.pipeline_variables.len().saturating_sub(1)));
        }
    }

    pub fn remove_pipeline_variable(&mut self) {
        if let Some(i) = self.pipeline_variables_state.selected() {
            if i < self.pipeline_variables.len() {
                self.pipeline_variables.remove(i);
                if self.pipeline_variables.is_empty() {
                    self.pipeline_variables_state.select(None);
                } else {
                    self.pipeline_variables_state.select(Some(i.min(self.pipeline_variables.len() - 1)));
                }
            }
        }
    }

    pub async fn submit_pipeline_launch(&mut self) {
        if let Some(project) = &self.selected_project {
            let project_id = project.id;
            let ref_name = self.pipeline_ref_input.clone();
            let variables = self.pipeline_variables.clone();
            let _ = self.create_pipeline(project_id, &ref_name, variables).await;
            self.show_pipeline_popup = false;
            self.input_mode = false;
        }
    }

    pub async fn create_pipeline(&mut self, project_id: u64, ref_name: &str, variables: Vec<PipelineVariable>) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        let endpoint = CreatePipeline { 
            project: project_id, 
            ref_name: ref_name.to_string(),
            variables,
        };
        tokio::spawn(async move {
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Create pipeline timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn retry_pipeline(&mut self, project_id: u64, pipeline_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        let endpoint = RetryPipeline { project: project_id, pipeline: pipeline_id };
        tokio::spawn(async move {
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Retry pipeline timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn cancel_pipeline(&mut self, project_id: u64, pipeline_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        let endpoint = CancelPipeline { project: project_id, pipeline: pipeline_id };
        tokio::spawn(async move {
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Cancel pipeline timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn retry_job(&mut self, project_id: u64, job_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        let endpoint = RetryJob { project: project_id, job: job_id };
        tokio::spawn(async move {
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Retry job timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn cancel_job(&mut self, project_id: u64, job_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        let endpoint = CancelJob { project: project_id, job: job_id };
        tokio::spawn(async move {
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Cancel job timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn play_job(&mut self, project_id: u64, job_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        let endpoint = PlayJob { project: project_id, job: job_id };
        tokio::spawn(async move {
            let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(e.to_string())); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Play job timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub fn toggle_job_section(&mut self) {
        let mut visible_count = 0;
        let mut target_section = None;
        for line in &self.job_log_lines {
            let is_collapsed = line.section.as_ref().map(|s| self.collapsed_sections.contains(s)).unwrap_or(false);
            if visible_count == self.job_log_scroll {
                if let Some(s) = &line.section { target_section = Some(s.clone()); break; }
            }
            if line.is_section_start { if is_collapsed { visible_count += 1; continue; } }
            if !is_collapsed || line.is_section_start { visible_count += 1; }
        }
        if let Some(s) = target_section {
            if self.collapsed_sections.contains(&s) { self.collapsed_sections.remove(&s); }
            else { self.collapsed_sections.insert(s); }
        }
    }

    pub fn get_visible_job_log_lines_count(&self) -> u16 {
        let mut count = 0;
        for line in &self.job_log_lines {
            let is_collapsed = line.section.as_ref().map(|s| self.collapsed_sections.contains(s)).unwrap_or(false);
            if is_collapsed && !line.is_section_start { continue; }
            if line.is_section_end && line.content.trim().is_empty() { continue; }
            count += 1;
        }
        count
    }
}

pub fn parse_job_log(raw_log: &str) -> Vec<LogLine> {
    let prefix_re = Regex::new(r"^([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]+Z)\s[0-9]{2}O\+?\s?").unwrap();
    let section_start_re = Regex::new(r"section_start:[0-9]+:([a-z0-9_]+)\r?").unwrap();
    let section_end_re = Regex::new(r"section_end:[0-9]+:([a-z0-9_]+)\r?").unwrap();
    let clear_line_re = Regex::new(r"\x1b\[0K").unwrap();

    let mut lines = Vec::new();
    let mut current_section: Option<String> = None;

    for line in raw_log.lines() {
        let mut timestamp = String::new();
        let mut content = line.to_string();
        if let Some(caps) = prefix_re.captures(line) {
            timestamp = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            content = prefix_re.replace(line, "").to_string();
        }
        content = clear_line_re.replace_all(&content, "").to_string();
        let is_section_start = if let Some(caps) = section_start_re.captures(&content) {
            current_section = caps.get(1).map(|m| m.as_str().to_string());
            content = section_start_re.replace(&content, "").to_string();
            true
        } else { false };

        let is_section_end = if let Some(_caps) = section_end_re.captures(&content) {
            content = section_end_re.replace(&content, "").to_string();
            lines.push(LogLine { timestamp: timestamp.clone(), content: content.clone(), section: current_section.clone(), is_section_start, is_section_end: true });
            current_section = None;
            continue;
        } else { false };

        lines.push(LogLine { timestamp, content, section: current_section.clone(), is_section_start, is_section_end });
    }
    lines
}

pub fn filter_branch_names(branches: &[Branch], search: &str) -> Vec<String> {
    let search = search.to_lowercase();
    branches.iter().map(|b| b.name.clone()).filter(|name| name.to_lowercase().contains(&search)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_job_log_exhaustive() {
        let log = "section_start:1:s1\nline1\nsection_end:2:s1\n\x1b[0Kclear\n2026-04-25T12:00:00.000Z 01O timestamped";
        let lines = parse_job_log(log);
        assert_eq!(lines.len(), 5);
        assert!(lines[0].is_section_start);
        assert_eq!(lines[0].section, Some("s1".into()));
        assert_eq!(lines[1].section, Some("s1".into()));
        assert!(lines[2].is_section_end);
        assert_eq!(lines[2].section, Some("s1".into()));
        assert_eq!(lines[3].content, "clear");
        assert_eq!(lines[4].timestamp, "2026-04-25T12:00:00.000Z");
    }

    #[test]
    fn test_filter_branch_names_logic() {
        let branches = vec![Branch { name: "main".into() }, Branch { name: "dev".into() }];
        assert_eq!(filter_branch_names(&branches, "MA"), vec!["main"]);
        assert_eq!(filter_branch_names(&branches, "xyz"), Vec::<String>::new());
    }

    use gitlab::GitlabBuilder;
    use tokio_test::block_on;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    use tokio::sync::mpsc;
    use std::time::Duration;

    async fn setup_app() -> (MockServer, App, mpsc::UnboundedReceiver<DataUpdate>) {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        let (tx, rx) = mpsc::unbounded_channel();
        let mut app = App::new(client);
        app.tx = tx;
        app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()) });
        (server, app, rx)
    }

    #[tokio::test]
    async fn test_pipeline_actions_exhaustive() {
        let (server, mut app, mut rx) = setup_app().await;

        Mock::given(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
        Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;

        app.fetch_pipelines().await.unwrap();
        let _ = tokio::time::timeout(Duration::from_secs(2), async {
            while !app.is_loading { app.process_updates(&mut rx).await; tokio::time::sleep(Duration::from_millis(10)).await; }
            while app.is_loading { app.process_updates(&mut rx).await; tokio::time::sleep(Duration::from_millis(10)).await; }
        }).await;

        app.fetch_jobs(1, 1).await.unwrap();
        app.fetch_pipeline(1, 1).await.unwrap();
        app.fetch_job_log(1, 1).await.unwrap();
        app.fetch_branches(1).await.unwrap();
        let _ = tokio::time::timeout(Duration::from_secs(2), async {
            while app.is_loading { app.process_updates(&mut rx).await; tokio::time::sleep(Duration::from_millis(10)).await; }
        }).await;

        app.show_pipeline_launch_popup().await;
        app.pipeline_variable_key_input = "K".into();
        app.pipeline_variable_value_input = "V".into();
        app.add_pipeline_variable();
        app.pipeline_variables_state.select(Some(0));
        app.remove_pipeline_variable();
        app.submit_pipeline_launch().await;
        
        app.create_pipeline(1, "m", vec![]).await.unwrap();
        app.retry_pipeline(1, 1).await.unwrap();
        app.cancel_pipeline(1, 1).await.unwrap();
        
        app.selected_pipeline = Some(Pipeline { id: 1, project_id: 1, status: "s".into(), source: "s".into(), ref_name: "m".into(), web_url: "u".into(), created_at: "t".into(), started_at: None, duration: None });
        app.retry_job(1, 1).await.unwrap();
        app.cancel_job(1, 1).await.unwrap();
        app.play_job(1, 1).await.unwrap();
        
        app.job_log_lines = vec![LogLine { timestamp: "t".into(), content: "c1".into(), section: Some("s1".into()), is_section_start: true, is_section_end: false }];
        app.job_log_scroll = 0;
        app.toggle_job_section();
        app.get_visible_job_log_lines_count();
        app.pipeline_ref_input = "xyz".into();
        app.filter_branches();
    }
}
