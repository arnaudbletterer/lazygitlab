use crate::api::*;
use crate::models::*;
use gitlab::api::{AsyncQuery, paged, raw};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Semaphore;

/// Upper bound on simultaneous per-project requests for the cross-project
/// Pipelines tab.
const CROSS_PIPELINES_MAX_IN_FLIGHT: usize = 8;

/// One bounded fan-out round. Projects still unfinished at `deadline` are
/// abandoned and whatever arrived is returned.
async fn fetch_cross_round(
    client: gitlab::AsyncGitlab,
    project_ids: Vec<u64>,
    deadline: std::time::Duration,
) -> Vec<Pipeline> {
    let results = Arc::new(std::sync::Mutex::new(Vec::new()));
    let sem = Arc::new(Semaphore::new(CROSS_PIPELINES_MAX_IN_FLIGHT));
    // Dropping the JoinSet on deadline aborts unfinished requests.
    let mut set = tokio::task::JoinSet::new();
    let run = async {
        for pid in project_ids {
            let Ok(permit) = sem.clone().acquire_owned().await else {
                break;
            };
            let c = client.clone();
            let out = results.clone();
            set.spawn(async move {
                let _permit = permit;
                let ep = ProjectPipelines {
                    project: pid,
                    page: Some(1),
                    per_page: Some(20),
                };
                let res: Result<Result<Vec<Pipeline>, _>, _> =
                    tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&c))
                        .await;
                if let Ok(Ok(pipes)) = res {
                    out.lock().unwrap().extend(pipes);
                }
            });
        }
        while set.join_next().await.is_some() {}
    };
    if tokio::time::timeout(deadline, run).await.is_err() {
        tracing::warn!("cross-project pipelines refresh hit its deadline; showing partial results");
    }
    set.abort_all();
    let mut guard = results.lock().unwrap();
    std::mem::take(&mut *guard)
}

/// Clears the in-flight flag when the refresh task ends, even on panic.
struct InFlightGuard(Arc<AtomicBool>);

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl App {
    pub async fn fetch_pipelines(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            let _ = tx.send(DataUpdate::Loading(true));
            tokio::spawn(async move {
                let ep = ProjectPipelines {
                    project: pid,
                    page: Some(1),
                    per_page: Some(50),
                };
                let res: Result<Result<Vec<Pipeline>, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    ep.query_async(&client),
                )
                .await;
                match res {
                    Ok(Ok(data)) => {
                        let has_more = data.len() == 50;
                        let _ = tx.send(DataUpdate::ProjectPipelines {
                            project_id: pid,
                            data,
                            page: 1,
                            has_more,
                        });
                    }
                    Ok(Err(e)) => {
                        let _ =
                            tx.send(DataUpdate::Error(format!("Pipelines fetch failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Pipelines fetch timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_pipeline_schedules(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.selected_project {
            let pid = project.id;
            if !self.pipeline_schedules.is_empty() {
                return Ok(());
            }
            let client = self.gitlab.clone();
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let ep = ProjectPipelineSchedules { project: pid };
                let res: Result<Result<Vec<PipelineSchedule>, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    paged(ep, gitlab::api::Pagination::All).query_async(&client),
                )
                .await;
                match res {
                    Ok(Ok(data)) => {
                        let _ = tx.send(DataUpdate::PipelineSchedules {
                            project_id: pid,
                            data,
                        });
                    }
                    Ok(Err(e)) => {
                        let _ =
                            tx.send(DataUpdate::Error(format!("Schedules fetch failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Schedules fetch timed out".into()));
                    }
                }
            });
        }
        Ok(())
    }

    pub async fn run_pipeline_schedule(
        &mut self,
        schedule_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(project) = self.selected_project.as_ref().map(|p| p.id) else {
            return Ok(());
        };
        self.status_message = Some(format!("Triggering schedule #{}...", schedule_id));
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let ep = RunPipelineSchedule {
                project,
                schedule: schedule_id,
            };
            let res: Result<Result<serde_json::Value, _>, _> =
                tokio::time::timeout(std::time::Duration::from_secs(30), ep.query_async(&client))
                    .await;
            match res {
                Ok(Ok(_)) => {
                    let _ = tx.send(DataUpdate::StatusMessage(format!(
                        "Schedule #{} triggered",
                        schedule_id
                    )));
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Schedule trigger failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Schedule trigger timed out".into()));
                }
            }
        });
        Ok(())
    }

    pub async fn delete_project_variable(
        &mut self,
        key: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(pid) = self.selected_project.as_ref().map(|p| p.id) else {
            return Ok(());
        };
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let ep = DeleteProjectVariable {
                project: pid,
                key: key.clone(),
            };
            let res: Result<Vec<u8>, _> = gitlab::api::raw(ep).query_async(&client).await;
            match res {
                Ok(_) => {
                    let _ = tx.send(DataUpdate::VariableDeleted { key });
                }
                Err(e) => {
                    let _ = tx.send(DataUpdate::Error(format!("Variable delete failed: {}", e)));
                }
            }
        });
        Ok(())
    }

    pub async fn upsert_project_variable(
        &mut self,
        key: String,
        value: String,
        update_existing: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(pid) = self.selected_project.as_ref().map(|p| p.id) else {
            return Ok(());
        };
        self.status_message = Some(if update_existing {
            format!("Updating variable {}...", key)
        } else {
            format!("Creating variable {}...", key)
        });
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let res: Result<serde_json::Value, _> = if update_existing {
                UpdateProjectVariable {
                    project: pid,
                    key: key.clone(),
                    value: value.clone(),
                }
                .query_async(&client)
                .await
            } else {
                CreateProjectVariable {
                    project: pid,
                    key: key.clone(),
                    value: value.clone(),
                }
                .query_async(&client)
                .await
            };
            match res {
                Ok(_) => {
                    let _ = tx.send(DataUpdate::VariableSaved { key, value });
                }
                Err(e) => {
                    let _ = tx.send(DataUpdate::Error(format!("Variable save failed: {}", e)));
                }
            }
        });
        Ok(())
    }

    /// Background (polled) refresh: skipped while one is already in flight.
    pub async fn fetch_cross_pipelines(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.start_cross_pipelines(false)
    }

    /// Manual refresh: if one is in flight, queue exactly one rerun.
    pub async fn fetch_cross_pipelines_manual(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.start_cross_pipelines(true)
    }

    fn start_cross_pipelines(&mut self, manual: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut project_ids: Vec<u64> = self.member_project_ids.iter().copied().collect();
        project_ids.sort_unstable();
        if project_ids.is_empty() {
            return Ok(());
        }
        // Never overlap refreshes: the adaptive runner can tick faster than a
        // slow fan-out completes.
        if self.cross_pipelines_in_flight.swap(true, Ordering::AcqRel) {
            if manual {
                self.cross_pipelines_rerun.store(true, Ordering::Release);
                self.status_message =
                    Some("Pipelines refresh in progress, will refresh again when done".into());
            }
            return Ok(());
        }
        let guard = InFlightGuard(self.cross_pipelines_in_flight.clone());
        let rerun = self.cross_pipelines_rerun.clone();
        let deadline = self.cross_pipelines_deadline;
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        self.status_message = Some("Fetching pipelines across projects...".into());
        tokio::spawn(async move {
            let _guard = guard;
            loop {
                let mut all =
                    fetch_cross_round(client.clone(), project_ids.clone(), deadline).await;
                all.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                all.truncate(50);
                let _ = tx.send(DataUpdate::GlobalPipelines {
                    data: all,
                    page: 1,
                    has_more: false,
                });
                if !rerun.swap(false, Ordering::AcqRel) {
                    break;
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_next_pipelines(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.pipelines_has_more || self.is_loading {
            return Ok(());
        }

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
                let res: Result<Result<Vec<Pipeline>, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    ep.query_async(&client),
                )
                .await;
                match res {
                    Ok(Ok(data)) => {
                        if !data.is_empty() {
                            let _ = tx.send(DataUpdate::AppendProjectPipelines {
                                project_id: pid,
                                data,
                            });
                        }
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(e.to_string()));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Pipelines fetch timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
        }
        Ok(())
    }

    pub async fn fetch_jobs(
        &mut self,
        project_id: u64,
        pipeline_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = ProjectPipelineJobs {
                project: project_id,
                pipeline: pipeline_id,
            };
            let res: Result<Result<Vec<Job>, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                paged(endpoint, gitlab::api::Pagination::Limit(100)).query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(mut data)) => {
                    data.sort_by_key(|j| j.id);
                    let _ = tx.send(DataUpdate::Jobs {
                        project_id,
                        pipeline_id,
                        data,
                    });
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Jobs fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Jobs fetch timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_jobs_sync(
        &mut self,
        project_id: u64,
        pipeline_id: u64,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        use crate::api::ProjectPipelineJobs;
        use gitlab::api::{AsyncQuery, Pagination, paged};
        let endpoint = ProjectPipelineJobs {
            project: project_id,
            pipeline: pipeline_id,
        };
        let lp = paged(endpoint, Pagination::Limit(100));
        let res = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            AsyncQuery::query_async(&lp, &self.gitlab),
        )
        .await;
        match res {
            Ok(Ok(mut data)) => {
                data.sort_by_key(|j: &Job| j.id);
                self.jobs = data.clone();
                self.jobs_state
                    .select(if self.jobs.is_empty() { None } else { Some(0) });
                self.is_loading = false;
                let _ = self.update_preview().await;
                Ok(self.jobs.len())
            }
            Ok(Err(e)) => {
                self.last_error = Some(format!("Jobs fetch failed: {}", e));
                self.is_loading = false;
                Err(e.into())
            }
            Err(_) => {
                self.last_error = Some("Jobs fetch timed out".into());
                self.is_loading = false;
                Err("Jobs fetch timed out".into())
            }
        }
    }

    pub async fn fetch_project_variables(
        &mut self,
        project_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = ProjectVariables {
                project: project_id,
            };
            let res: Result<Result<Vec<ProjectVariable>, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                endpoint.query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(data)) => {
                    let _ = tx.send(DataUpdate::ProjectVariables { project_id, data });
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!(
                        "Project variables fetch failed: {}",
                        e
                    )));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error(
                        "Project variables fetch timed out".into(),
                    ));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_pipeline(
        &mut self,
        project_id: u64,
        pipeline_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = ProjectPipeline {
                project: project_id,
                pipeline: pipeline_id,
            };
            let res: Result<Result<Pipeline, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                endpoint.query_async(&client),
            )
            .await;
            if let Ok(Ok(_pipeline)) = res {
                let _ = tx.send(DataUpdate::TriggerRefresh);
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_job_log(
        &mut self,
        project_id: u64,
        job_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = JobLog {
                project: project_id,
                job: job_id,
            };
            let res: Result<Result<Vec<u8>, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                raw(endpoint).query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(data)) => match String::from_utf8(data) {
                    Ok(content) => {
                        let _ = tx.send(DataUpdate::Log {
                            project_id,
                            job_id,
                            data: content,
                        });
                    }
                    Err(e) => {
                        let content = String::from_utf8_lossy(e.as_bytes()).to_string();
                        let _ = tx.send(DataUpdate::Log {
                            project_id,
                            job_id,
                            data: content,
                        });
                    }
                },
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Job log fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Job log fetch timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn fetch_job_log_sync(
        &mut self,
        project_id: u64,
        job_id: u64,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        use crate::api::JobLog;
        use gitlab::api::{AsyncQuery, raw};
        let endpoint = JobLog {
            project: project_id,
            job: job_id,
        };
        let res = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            raw(endpoint).query_async(&self.gitlab),
        )
        .await;
        match res {
            Ok(Ok(data)) => {
                let content = String::from_utf8_lossy(&data).to_string();
                self.job_log = content;
                self.parse_job_log();
                self.job_log_cursor = 0;
                self.job_log_selection_start = None;
                self.follow_log = matches!(
                    self.selected_job.as_ref().map(|j| j.status.as_str()),
                    Some("running" | "pending")
                );
                self.is_loading = false;
                let _ = self.update_preview().await;
                Ok(self.job_log_lines.len())
            }
            Ok(Err(e)) => {
                self.last_error = Some(format!("Job log fetch failed: {}", e));
                self.is_loading = false;
                Err(e.into())
            }
            Err(_) => {
                self.last_error = Some("Job log fetch timed out".into());
                self.is_loading = false;
                Err("Job log fetch timed out".into())
            }
        }
    }

    pub fn filter_branches(&mut self) {
        let filter = self.pipeline_ref_input.to_lowercase();
        self.filtered_branches = self
            .project_branches
            .iter()
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
                    variables: Some(
                        vars.into_iter()
                            .map(|v| crate::models::PipelineVariable {
                                key: v.key,
                                value: v.value,
                            })
                            .collect(),
                    ),
                };
                let res: Result<Result<Pipeline, _>, _> = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    endpoint.query_async(&client),
                )
                .await;
                match res {
                    Ok(Ok(_)) => {
                        let _ = tx.send(DataUpdate::TriggerRefresh);
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(DataUpdate::Error(format!("Launch failed: {}", e)));
                    }
                    Err(_) => {
                        let _ = tx.send(DataUpdate::Error("Launch timed out".into()));
                    }
                }
                let _ = tx.send(DataUpdate::Loading(false));
            });
            self.show_pipeline_popup = false;
            self.input_mode = false;
        }
        Ok(())
    }

    pub async fn submit_pipeline_launch_sync(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let project = match &self.selected_project {
            Some(p) => p.clone(),
            None => {
                self.status_message = Some("No project selected".into());
                return Ok(());
            }
        };
        let pid = project.id;
        let ref_name = self.pipeline_ref_input.clone();
        let vars = self.pipeline_variables.clone();

        if ref_name.is_empty() {
            self.status_message = Some("Ref name cannot be empty".into());
            return Ok(());
        }

        self.status_message = Some(format!("Launching pipeline on {}...", ref_name));
        let _ = self.update_preview().await;

        use crate::api::CreatePipeline;
        use gitlab::api::AsyncQuery;
        let endpoint = CreatePipeline {
            project: pid,
            ref_name,
            variables: Some(
                vars.into_iter()
                    .map(|v| crate::models::PipelineVariable {
                        key: v.key,
                        value: v.value,
                    })
                    .collect(),
            ),
        };
        let res: Result<Result<crate::models::Pipeline, _>, _> = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            AsyncQuery::query_async(&endpoint, &self.gitlab),
        )
        .await;
        self.show_pipeline_popup = false;
        self.input_mode = false;
        match res {
            Ok(Ok(p)) => {
                self.status_message = Some(format!("Pipeline #{} launched", p.id));
                let _ = self.refresh().await;
                Ok(())
            }
            Ok(Err(e)) => {
                self.status_message = Some(format!("Launch failed: {}", e));
                self.last_error = Some(format!("Launch failed: {}", e));
                Err(e.into())
            }
            Err(_) => {
                self.status_message = Some("Launch timed out".into());
                self.last_error = Some("Launch timed out".into());
                Err("Launch timed out".into())
            }
        }
    }

    pub async fn create_branch_sync(
        &mut self,
        project_id: u64,
        branch: String,
        ref_name: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::api::CreateBranch;
        use gitlab::api::AsyncQuery;
        if branch.is_empty() {
            self.status_message = Some("Branch name cannot be empty".into());
            return Err("Branch name cannot be empty".into());
        }
        if ref_name.is_empty() {
            self.status_message = Some("Reference cannot be empty".into());
            return Err("Reference cannot be empty".into());
        }
        self.status_message = Some(format!("Creating branch {branch} from {ref_name}..."));
        let _ = self.update_preview().await;
        let endpoint = CreateBranch {
            project: project_id,
            branch: branch.clone(),
            ref_name: ref_name.clone(),
        };
        let res: Result<Result<crate::models::Branch, _>, _> = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            AsyncQuery::query_async(&endpoint, &self.gitlab),
        )
        .await;
        match res {
            Ok(Ok(b)) => {
                self.status_message = Some(format!("Branch {} created", b.name));
                self.project_branches.push(b);
                if self.branches_state.selected().is_none() && !self.project_branches.is_empty() {
                    self.branches_state.select(Some(0));
                }
                Ok(())
            }
            Ok(Err(e)) => {
                self.status_message = Some(format!("Create failed: {}", e));
                self.last_error = Some(format!("Create branch failed: {}", e));
                Err(e.into())
            }
            Err(_) => {
                self.status_message = Some("Create timed out".into());
                self.last_error = Some("Create branch timed out".into());
                Err("Create branch timed out".into())
            }
        }
    }

    pub async fn delete_branch_sync(
        &mut self,
        project_id: u64,
        branch: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::api::DeleteBranch;
        use gitlab::api::AsyncQuery;
        if branch.is_empty() {
            self.status_message = Some("No branch selected".into());
            return Err("No branch selected".into());
        }
        self.status_message = Some(format!("Deleting branch {branch}..."));
        let _ = self.update_preview().await;
        let endpoint = DeleteBranch {
            project: project_id,
            branch: branch.clone(),
        };
        let res: Result<Result<serde_json::Value, _>, _> = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            AsyncQuery::query_async(&endpoint, &self.gitlab),
        )
        .await;
        match res {
            Ok(Ok(_)) => {
                self.status_message = Some(format!("Branch {branch} deleted"));
                self.project_branches.retain(|b| b.name != branch);
                if let Some(idx) = self.branches_state.selected() {
                    if idx >= self.project_branches.len() && !self.project_branches.is_empty() {
                        self.branches_state
                            .select(Some(self.project_branches.len() - 1));
                    } else if self.project_branches.is_empty() {
                        self.branches_state.select(None);
                    }
                }
                Ok(())
            }
            Ok(Err(e)) => {
                self.status_message = Some(format!("Delete failed: {}", e));
                self.last_error = Some(format!("Delete branch failed: {}", e));
                Err(e.into())
            }
            Err(_) => {
                self.status_message = Some("Delete timed out".into());
                self.last_error = Some("Delete branch timed out".into());
                Err("Delete branch timed out".into())
            }
        }
    }

    pub async fn retry_pipeline(
        &mut self,
        project_id: u64,
        pipeline_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = RetryPipeline {
                project: project_id,
                pipeline: pipeline_id,
            };
            let res: Result<Result<Pipeline, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                endpoint.query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(_)) => {
                    let _ = tx.send(DataUpdate::TriggerRefresh);
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Retry failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Retry timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn cancel_pipeline(
        &mut self,
        project_id: u64,
        pipeline_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = CancelPipeline {
                project: project_id,
                pipeline: pipeline_id,
            };
            let res: Result<Result<Pipeline, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                endpoint.query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(_)) => {
                    let _ = tx.send(DataUpdate::TriggerRefresh);
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Cancel failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Cancel timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn cancel_pipeline_sync(
        &mut self,
        project_id: u64,
        pipeline_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::api::CancelPipeline;
        use gitlab::api::AsyncQuery;
        self.status_message = Some(format!("Cancelling pipeline #{}...", pipeline_id));
        let _ = self.update_preview().await;
        let endpoint = CancelPipeline {
            project: project_id,
            pipeline: pipeline_id,
        };
        let res: Result<Result<crate::models::Pipeline, _>, _> = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            AsyncQuery::query_async(&endpoint, &self.gitlab),
        )
        .await;
        match res {
            Ok(Ok(_)) => {
                self.status_message = Some(format!("Pipeline #{} cancelled", pipeline_id));
                let _ = self.refresh().await;
                Ok(())
            }
            Ok(Err(e)) => {
                self.status_message = Some(format!("Cancel failed: {}", e));
                self.last_error = Some(format!("Cancel failed: {}", e));
                Err(e.into())
            }
            Err(_) => {
                self.status_message = Some("Cancel timed out".into());
                self.last_error = Some("Cancel timed out".into());
                Err("Cancel timed out".into())
            }
        }
    }

    pub async fn retry_pipeline_sync(
        &mut self,
        project_id: u64,
        pipeline_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::api::RetryPipeline;
        use gitlab::api::AsyncQuery;
        self.status_message = Some(format!("Retrying pipeline #{}...", pipeline_id));
        let _ = self.update_preview().await;
        let endpoint = RetryPipeline {
            project: project_id,
            pipeline: pipeline_id,
        };
        let res: Result<Result<crate::models::Pipeline, _>, _> = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            AsyncQuery::query_async(&endpoint, &self.gitlab),
        )
        .await;
        match res {
            Ok(Ok(_)) => {
                self.status_message = Some(format!("Pipeline #{} retry triggered", pipeline_id));
                let _ = self.refresh().await;
                Ok(())
            }
            Ok(Err(e)) => {
                self.status_message = Some(format!("Retry failed: {}", e));
                self.last_error = Some(format!("Retry failed: {}", e));
                Err(e.into())
            }
            Err(_) => {
                self.status_message = Some("Retry timed out".into());
                self.last_error = Some("Retry timed out".into());
                Err("Retry timed out".into())
            }
        }
    }

    pub async fn retry_job(
        &mut self,
        project_id: u64,
        job_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = RetryJob {
                project: project_id,
                job: job_id,
            };
            let res: Result<Result<Job, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                endpoint.query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(_)) => {
                    let _ = tx.send(DataUpdate::TriggerRefresh);
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Retry failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Retry timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn cancel_job(
        &mut self,
        project_id: u64,
        job_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = CancelJob {
                project: project_id,
                job: job_id,
            };
            let res: Result<Result<Job, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                endpoint.query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(_)) => {
                    let _ = tx.send(DataUpdate::TriggerRefresh);
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Cancel failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Cancel timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub async fn play_job(
        &mut self,
        project_id: u64,
        job_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let endpoint = PlayJob {
                project: project_id,
                job: job_id,
            };
            let res: Result<Result<Job, _>, _> = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                endpoint.query_async(&client),
            )
            .await;
            match res {
                Ok(Ok(_)) => {
                    let _ = tx.send(DataUpdate::TriggerRefresh);
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Play failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Play timed out".into()));
                }
            }
            let _ = tx.send(DataUpdate::Loading(false));
        });
        Ok(())
    }

    pub fn toggle_job_section(&mut self) {
        if let Some(line) = self.job_log_lines.get(self.job_log_cursor)
            && let Some(section) = &line.section
        {
            if self.collapsed_sections.contains(section) {
                self.collapsed_sections.remove(section);
            } else {
                self.collapsed_sections.insert(section.clone());
            }
            self.bump_job_log_epoch();
        }
    }

    pub fn get_visible_job_log_lines_count(&self) -> usize {
        let mut count = 0;
        for line in &self.job_log_lines {
            let is_collapsed = line
                .section
                .as_ref()
                .map(|s| self.collapsed_sections.contains(s))
                .unwrap_or(false);
            if is_collapsed && !line.is_section_start {
                continue;
            }
            if line.is_section_end && line.content.trim().is_empty() {
                continue;
            }
            count += 1;
        }
        count
    }
}

static TIMESTAMP_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+").unwrap()
});
static SECTION_START_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"section_start:(\d+):([^\r\n]+)").unwrap());
static SECTION_END_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"section_end:(\d+):([^\r\n]+)").unwrap());
static LOG_NOISE_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\x1b\[0K").unwrap());

pub fn parse_job_log(log: &str) -> Vec<LogLine> {
    let timestamp_re = &*TIMESTAMP_RE;
    let section_start_re = &*SECTION_START_RE;
    let section_end_re = &*SECTION_END_RE;
    let noise_re = &*LOG_NOISE_RE;

    let mut current_section: Option<String> = None;
    let mut out = Vec::new();

    for l in log.lines() {
        if l.is_empty() {
            continue;
        }

        let mut timestamp = String::new();
        let mut content = l.to_string();

        if let Some(caps) = timestamp_re.captures(&content) {
            timestamp = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            content = timestamp_re.replace(&content, "").to_string();
        }

        let mut is_section_start = false;
        let mut is_section_end = false;
        let mut line_section: Option<String> = None;

        if let Some(caps) = section_start_re.captures(&content) {
            is_section_start = true;
            let name = caps.get(2).map(|m| {
                m.as_str()
                    .split('[')
                    .next()
                    .unwrap_or(m.as_str())
                    .trim()
                    .to_string()
            });
            current_section = name.clone();
            line_section = name;
            content = section_start_re.replace(&content, "").to_string();
        } else if let Some(caps) = section_end_re.captures(&content) {
            is_section_end = true;
            let name = caps.get(2).map(|m| {
                m.as_str()
                    .split('[')
                    .next()
                    .unwrap_or(m.as_str())
                    .trim()
                    .to_string()
            });
            line_section = name.clone();
            content = section_end_re.replace(&content, "").to_string();
            current_section = None;
        }

        content = content
            .trim_start_matches(|c: char| c.is_ascii_digit() || c == 'O' || c == '+')
            .trim_start()
            .to_string();
        content = noise_re.replace_all(&content, "").to_string();

        let effective_section = if is_section_start || is_section_end {
            line_section
        } else {
            current_section.clone()
        };

        if content.is_empty() && !is_section_start && !is_section_end {
            continue;
        }

        out.push(LogLine {
            timestamp,
            content,
            section: effective_section,
            is_section_start,
            is_section_end,
        });
    }
    out
}

static ANSI_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\x1b\[[0-9;]*[mK]").unwrap());

pub fn strip_ansi(text: &str) -> String {
    ANSI_RE.replace_all(text, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitlab::GitlabBuilder;
    use wiremock::matchers::path;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_app() -> (MockServer, App) {
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
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        (server, App::new(client, tx, None))
    }

    #[tokio::test]
    async fn test_pipeline_logic() {
        let (_server, mut app) = setup_app().await;
        app.selected_project = Some(Project {
            id: 1,
            name: "p".into(),
            web_url: Some("u".into()),
            path_with_namespace: Some("p".into()),
            default_branch: Some("master".into()),
            namespace: None,
        });
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
