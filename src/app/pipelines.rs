use gitlab::api::{AsyncQuery, paged, raw};
use crate::models::*;
use crate::api::*;

impl App {
    pub async fn fetch_pipelines(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = ProjectPipelines { project: pid, page: Some(1), per_page: Some(50) };
                let res: Result<Result<Vec<Pipeline>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client)).await;
                match res {
                    Ok(Ok(data)) => { let has_more = data.len() == 50; let _ = tx.send(DataUpdate::ProjectPipelines { project_id: pid, data, page: 1, has_more }); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Pipelines fetch failed: {}", e))); }
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
                Ok(Ok(mut data)) => { 
                    data.sort_by_key(|j| j.id);
                    let _ = tx.send(DataUpdate::Jobs { project_id, pipeline_id, data }); 
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Jobs fetch failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Jobs fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_project_variables(&mut self, project_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = ProjectVariables { project: project_id };
            let res: Result<Result<Vec<ProjectVariable>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(data)) => { let _ = tx.send(DataUpdate::ProjectVariables { project_id, data }); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Project variables fetch failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Project variables fetch timed out".into())); }
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
            let endpoint = JobLog { project: project_id, job: job_id };
            let res: Result<Result<Vec<u8>, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), raw(endpoint).query_async(&client)).await;
            match res {
                Ok(Ok(data)) => {
                    match String::from_utf8(data) {
                        Ok(content) => { let _ = tx.send(DataUpdate::Log { project_id, job_id, data: content }); }
                        Err(e) => { 
                            let content = String::from_utf8_lossy(e.as_bytes()).to_string();
                            let _ = tx.send(DataUpdate::Log { project_id, job_id, data: content });
                        }
                    }
                }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Job log fetch failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Job log fetch timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub fn filter_branches(&mut self) {
        let filter = self.pipeline_ref_input.to_lowercase();
        self.filtered_branches = self.project_branches.iter()
            .filter(|b| b.name.to_lowercase().contains(&filter))
            .cloned()
            .collect();
        if !self.filtered_branches.is_empty() {
            self.branches_state.select(Some(0));
        } else {
            self.branches_state.select(None);
        }
    }

    pub async fn submit_pipeline_launch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let ref_name = self.pipeline_ref_input.clone();
            let vars = self.pipeline_variables.clone();
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));

            tokio::spawn(async move {
                let endpoint = CreatePipeline { 
                    project: pid, 
                    ref_name, 
                    variables: Some(vars.into_iter().map(|v| crate::models::PipelineVariable { key: v.key, value: v.value }).collect()) 
                };
                let res: Result<Result<Pipeline, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
                match res {
                    Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                    Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Launch failed: {}", e))); }
                    Err(_) => { let _ = tx.send(DataUpdate::Error("Launch timed out".into())); }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
            self.show_pipeline_popup = false;
            self.input_mode = false;
        }
        Ok(())
    }

    pub async fn retry_pipeline(&mut self, project_id: u64, pipeline_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = RetryPipeline { project: project_id, pipeline: pipeline_id };
            let res: Result<Result<Pipeline, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Retry failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Retry timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn cancel_pipeline(&mut self, project_id: u64, pipeline_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = CancelPipeline { project: project_id, pipeline: pipeline_id };
            let res: Result<Result<Pipeline, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Cancel failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Cancel timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn retry_job(&mut self, project_id: u64, job_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = RetryJob { project: project_id, job: job_id };
            let res: Result<Result<Job, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Retry failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Retry timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn cancel_job(&mut self, project_id: u64, job_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = CancelJob { project: project_id, job: job_id };
            let res: Result<Result<Job, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Cancel failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Cancel timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn play_job(&mut self, project_id: u64, job_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = PlayJob { project: project_id, job: job_id };
            let res: Result<Result<Job, _>, _> = tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.query_async(&client)).await;
            match res {
                Ok(Ok(_)) => { let _ = tx.send(DataUpdate::TriggerRefresh); }
                Ok(Err(e)) => { let _ = tx.send(DataUpdate::Error(format!("Play failed: {}", e))); }
                Err(_) => { let _ = tx.send(DataUpdate::Error("Play timed out".into())); }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub fn toggle_job_section(&mut self) {}
    pub fn get_visible_job_log_lines_count(&self) -> usize { 0 }
}

pub fn parse_job_log(log: &str) -> Vec<LogLine> {
    log.lines().map(|l| LogLine { timestamp: "".into(), content: l.to_string(), section: None, is_section_start: false, is_section_end: false }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::path;
    use gitlab::GitlabBuilder;

    async fn setup_app() -> (MockServer, App) {
        let server = MockServer::start().await;
        Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
        let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
        (server, App::new(client))
    }

    #[tokio::test]
    async fn test_pipeline_logic() {
        let (_server, mut app) = setup_app().await;
        app.selected_project = Some(Project { id: 1, name: "p".into(), web_url: Some("u".into()), path_with_namespace: Some("p".into()), default_branch: Some("master".into()), namespace: None });
        let _ = app.fetch_pipelines().await;
        let _ = app.fetch_next_pipelines().await;
        let _ = app.fetch_jobs(1, 1).await;
        let _ = app.fetch_project_variables(1).await;
        let _ = app.fetch_pipeline(1, 1).await;
        let _ = app.fetch_job_log(1, 1).await;
        app.pipeline_ref_input = "master".into();
        app.filter_branches();
        let _ = app.submit_pipeline_launch().await;
        let _ = app.retry_pipeline(1, 1).await;
        let _ = app.cancel_pipeline(1, 1).await;
        let _ = app.retry_job(1, 1).await;
        let _ = app.cancel_job(1, 1).await;
        let _ = app.play_job(1, 1).await;
        app.toggle_job_section();
        app.get_visible_job_log_lines_count();
        parse_job_log("hello\nworld");
    }
}
