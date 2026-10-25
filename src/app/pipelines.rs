use crate::api::*;
use crate::models::*;
use gitlab::api::{AsyncQuery, paged, raw};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Semaphore;

/// Largest job log kept in memory; older output is dropped.
pub const JOB_LOG_MAX_BYTES: usize = 8 * 1024 * 1024;

struct InflightGuard(Arc<AtomicBool>);

impl Drop for InflightGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

/// Decodes a raw log, keeping only the last `JOB_LOG_MAX_BYTES` (cut at a line start).
pub fn tail_log(data: &[u8]) -> String {
    if data.len() <= JOB_LOG_MAX_BYTES {
        return String::from_utf8_lossy(data).into_owned();
    }
    let mut start = data.len() - JOB_LOG_MAX_BYTES;
    if let Some(nl) = data[start..].iter().position(|b| *b == b'\n') {
        start += nl + 1;
    }
    format!(
        "[log truncated: showing the last {} MB]\n{}",
        JOB_LOG_MAX_BYTES / (1024 * 1024),
        String::from_utf8_lossy(&data[start..])
    )
}

struct Mutation {
    method: crate::http::Method,
    path: String,
    body: Vec<u8>,
    pending: String,
    done: String,
    failed: &'static str,
    follow_up: FollowUp,
}

enum FollowUp {
    Refresh,
    Branches(u64),
}

fn encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Upper bound on simultaneous per-project requests for the cross-project
/// Pipelines tab.
const CROSS_PIPELINES_MAX_IN_FLIGHT: usize = 8;

/// One bounded fan-out round. Projects still unfinished at `deadline` are
/// abandoned and whatever arrived is returned.
async fn fetch_cross_round(
    client: crate::trace::TracedClient,
    project_ids: Vec<u64>,
    deadline: std::time::Duration,
) -> (Vec<Pipeline>, bool) {
    let expected = project_ids.len();
    let succeeded = Arc::new(std::sync::atomic::AtomicUsize::new(0));
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
            let ok = succeeded.clone();
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
                    ok.fetch_add(1, Ordering::AcqRel);
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
    let complete = succeeded.load(Ordering::Acquire) == expected;
    (std::mem::take(&mut *guard), complete)
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
                let (mut all, complete) =
                    fetch_cross_round(client.clone(), project_ids.clone(), deadline).await;
                all.sort_by_key(|p| std::cmp::Reverse(p.id));
                all.truncate(50);
                if complete {
                    let _ = tx.send(DataUpdate::GlobalPipelines {
                        data: all,
                        page: 1,
                        has_more: false,
                    });
                } else {
                    if !all.is_empty() {
                        let _ = tx.send(DataUpdate::MergeGlobalPipelines(all));
                    }
                    let _ = tx.send(DataUpdate::RefreshFailed);
                }
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
        if self
            .job_log_inflight
            .swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            return Ok(());
        }
        self.job_log_seq += 1;
        let generation = self.job_log_gen;
        let seq = self.job_log_seq;
        let inflight = self.job_log_inflight.clone();
        let client = self.gitlab.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let _inflight = InflightGuard(inflight);
            let _loading = crate::app::mrs::LoadingGuard::new(tx.clone());
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
                Ok(Ok(data)) => {
                    let parsed = tokio::task::spawn_blocking(move || {
                        let raw = tail_log(&data);
                        let lines = parse_job_log(&raw);
                        (raw, lines)
                    })
                    .await;
                    if let Ok((raw, lines)) = parsed {
                        let _ = tx.send(DataUpdate::JobLog {
                            project_id,
                            job_id,
                            generation,
                            seq,
                            raw,
                            lines,
                        });
                    }
                }
                Ok(Err(e)) => {
                    let _ = tx.send(DataUpdate::Error(format!("Job log fetch failed: {}", e)));
                }
                Err(_) => {
                    let _ = tx.send(DataUpdate::Error("Job log fetch timed out".into()));
                }
            }
        });
        Ok(())
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
        let Some(pid) = self.selected_project.as_ref().map(|p| p.id) else {
            self.status_message = Some("No project selected".into());
            return Ok(());
        };
        let ref_name = self.pipeline_ref_input.clone();
        if ref_name.is_empty() {
            self.status_message = Some("Ref name cannot be empty".into());
            return Ok(());
        }
        let variables: Vec<serde_json::Value> = self
            .pipeline_variables
            .iter()
            .map(|v| serde_json::json!({"key": v.key, "value": v.value}))
            .collect();
        let body = serde_json::json!({"ref": ref_name, "variables": variables});
        self.spawn_mutation(Mutation {
            method: crate::http::Method::POST,
            path: format!("projects/{pid}/pipeline"),
            body: body.to_string().into_bytes(),
            pending: format!("Launching pipeline on {ref_name}..."),
            done: format!("Pipeline launched on {ref_name}"),
            failed: "Launch failed",
            follow_up: FollowUp::Refresh,
        });
        self.show_pipeline_popup = false;
        self.input_mode = false;
        Ok(())
    }

    pub async fn retry_pipeline(
        &mut self,
        project_id: u64,
        pipeline_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.spawn_mutation(Mutation {
            method: crate::http::Method::POST,
            path: format!("projects/{project_id}/pipelines/{pipeline_id}/retry"),
            body: Vec::new(),
            pending: format!("Retrying pipeline #{pipeline_id}..."),
            done: format!("Pipeline #{pipeline_id} retry triggered"),
            failed: "Retry failed",
            follow_up: FollowUp::Refresh,
        });
        Ok(())
    }

    pub async fn cancel_pipeline(
        &mut self,
        project_id: u64,
        pipeline_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.spawn_mutation(Mutation {
            method: crate::http::Method::POST,
            path: format!("projects/{project_id}/pipelines/{pipeline_id}/cancel"),
            body: Vec::new(),
            pending: format!("Cancelling pipeline #{pipeline_id}..."),
            done: format!("Pipeline #{pipeline_id} cancelled"),
            failed: "Cancel failed",
            follow_up: FollowUp::Refresh,
        });
        Ok(())
    }

    pub async fn create_branch(
        &mut self,
        project_id: u64,
        branch: String,
        ref_name: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if branch.is_empty() {
            self.status_message = Some("Branch name cannot be empty".into());
            return Err("Branch name cannot be empty".into());
        }
        if ref_name.is_empty() {
            self.status_message = Some("Reference cannot be empty".into());
            return Err("Reference cannot be empty".into());
        }
        self.spawn_mutation(Mutation {
            method: crate::http::Method::POST,
            path: format!(
                "projects/{project_id}/repository/branches?branch={}&ref={}",
                encode_component(&branch),
                encode_component(&ref_name)
            ),
            body: Vec::new(),
            pending: format!("Creating branch {branch} from {ref_name}..."),
            done: format!("Branch {branch} created"),
            failed: "Create failed",
            follow_up: FollowUp::Branches(project_id),
        });
        Ok(())
    }

    pub async fn delete_branch(
        &mut self,
        project_id: u64,
        branch: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if branch.is_empty() {
            self.status_message = Some("No branch selected".into());
            return Err("No branch selected".into());
        }
        self.spawn_mutation(Mutation {
            method: crate::http::Method::DELETE,
            path: format!(
                "projects/{project_id}/repository/branches/{}",
                encode_component(&branch)
            ),
            body: Vec::new(),
            pending: format!("Deleting branch {branch}..."),
            done: format!("Branch {branch} deleted"),
            failed: "Delete failed",
            follow_up: FollowUp::Branches(project_id),
        });
        Ok(())
    }

    fn spawn_mutation(&mut self, m: Mutation) {
        self.status_message = Some(m.pending);
        let http = self.http.clone();
        let tx = self.tx.clone();
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let _loading = crate::app::mrs::LoadingGuard::new(tx.clone());
            match http.send_mutation(m.method, &m.path, m.body).await {
                crate::http::Outcome::Ok(_) | crate::http::Outcome::NotModified => {
                    match m.follow_up {
                        FollowUp::Refresh => {
                            let _ = tx.send(DataUpdate::TriggerRefresh);
                        }
                        FollowUp::Branches(project_id) => {
                            let path = format!("projects/{project_id}/repository/branches");
                            let pages = http.get_pages_capped::<Branch>(&path, 1).await;
                            if pages.complete {
                                let _ = tx.send(DataUpdate::Branches {
                                    project_id,
                                    data: pages.items,
                                    page: 1,
                                    has_more: false,
                                });
                            }
                        }
                    }
                    let _ = tx.send(DataUpdate::StatusMessage(m.done));
                }
                crate::http::Outcome::RateLimited { until } => {
                    let _ = tx.send(DataUpdate::Error(crate::http::rate_limit_text(until)));
                }
                crate::http::Outcome::Failed(e) => {
                    let msg = format!("{}: {e}", m.failed);
                    let _ = tx.send(DataUpdate::StatusMessage(msg.clone()));
                    let _ = tx.send(DataUpdate::Error(msg));
                }
            }
        });
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
