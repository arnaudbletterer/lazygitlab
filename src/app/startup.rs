use crate::models::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// A fan-out that never reported back (its task died) stops blocking new ones after this.
const FANOUT_STALE: Duration = Duration::from_secs(65);
/// How long Home waits before prefetching merge requests, so a screen opened
/// right away gets the network first.
pub const HOME_PREFETCH_DELAY: Duration = Duration::from_millis(400);
/// A membership sync that has not answered after this long is given up.
pub const MEMBERSHIP_TIMEOUT: Duration = Duration::from_secs(20);
const MEMBERSHIP_RETRY_BASE: Duration = Duration::from_secs(10);
const MEMBERSHIP_RETRY_MAX: Duration = Duration::from_secs(60);

/// Key of a fan-out marker: global lists and the lists of one project never share one.
pub(crate) fn issues_fanout_key(project: Option<u64>, filter: IssueFilter) -> String {
    match project {
        Some(pid) => format!("issues:project:{pid}:{filter:?}"),
        None => format!("issues:global:{filter:?}"),
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ScopedKind {
    Result,
    NextPage,
    Failed,
    Other,
}

struct ClearOnDrop(std::sync::Arc<AtomicBool>);

impl Drop for ClearOnDrop {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl App {
    /// Loads the cached member project ids of `host` so that lists can start fetching at once.
    pub fn enable_membership_cache(&mut self, dir: std::path::PathBuf, host: &str) {
        if let Some(record) = crate::cache::load_membership(&dir, host) {
            self.member_project_ids = record.ids.into_iter().collect();
            self.cached_user_id = Some(record.user_id);
        }
        self.membership_cache = Some((dir, host.to_string()));
    }

    pub(crate) fn save_membership_cache(&self) {
        let (Some((dir, host)), Some(user)) = (&self.membership_cache, &self.current_user) else {
            return;
        };
        if self.is_syncing_membership || !self.membership_sync_attempted {
            return;
        }
        let (dir, host, user_id, ids) = (
            dir.clone(),
            host.clone(),
            user.id,
            self.member_project_ids.clone(),
        );
        tokio::task::spawn_blocking(move || {
            if let Err(e) = crate::cache::save_membership(&dir, &host, user_id, &ids) {
                tracing::debug!("membership cache not saved: {}", e);
            }
        });
    }

    /// Drops ids that came from the cache when they belong to another user.
    /// Ids from a membership sync that already answered are never touched.
    pub(crate) fn on_current_user(&mut self, id: u64) {
        if self
            .cached_user_id
            .take()
            .is_some_and(|cached| cached != id)
        {
            self.member_project_ids.clear();
            self.membership_sync_attempted = false;
            self.filter_lists();
        }
        self.save_membership_cache();
    }

    /// Fetches `/user` once: later calls do nothing while one is in flight or the user is known.
    pub(crate) fn start_user_fetch(&mut self, counted: bool) {
        if self.current_user.is_some()
            || self.auth_failed
            || self.user_fetch_in_flight.swap(true, Ordering::AcqRel)
        {
            return;
        }
        let guard = ClearOnDrop(self.user_fetch_in_flight.clone());
        let c = self.gitlab.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let _guard = guard;
            match fetch_current_user(&c).await {
                Ok(user) => {
                    let _ = tx.send(DataUpdate::CurrentUser(user));
                }
                Err((error, auth)) => {
                    let _ = tx.send(DataUpdate::UserFetchFailed { error, auth });
                }
            }
            if counted {
                let _ = tx.send(DataUpdate::Loading(false));
            }
        });
    }

    pub(crate) fn on_user_fetch_failed(&mut self, error: &str, auth: bool) {
        let friendly =
            crate::redaction::redact_secrets(&crate::errors::friendly_error(error)).into_owned();
        if self.exit_on_connect_failure {
            self.fatal_error = Some(format!("Could not connect to GitLab: {friendly}"));
            self.should_quit = true;
        } else if auth {
            self.auth_failed = true;
            self.last_error = Some(format!("{friendly}. Press q to quit or r to retry"));
            self.last_error_since = None;
        } else {
            self.last_error = Some(friendly);
            self.last_error_since = Some(Instant::now());
        }
    }

    /// `r` after an authentication failure: one more attempt, background refreshes stay off until it works.
    pub(crate) async fn retry_after_auth_failure(&mut self) {
        self.auth_failed = false;
        self.last_error = None;
        self.last_error_since = None;
        self.start_user_fetch(false);
        self.refresh().await;
    }

    fn fanout_in_flight(&self, key: &str) -> bool {
        self.fanouts_started
            .get(key)
            .is_some_and(|t| t.elapsed() < FANOUT_STALE)
    }

    pub(crate) fn clear_loading_projects_status(&mut self) {
        if self.status_message.as_deref() == Some("Loading projects...") {
            self.status_message = None;
        }
    }

    /// Marks a fan-out as finished. Returns true when a follow-up was requested meanwhile.
    pub(crate) fn fanout_done(&mut self, key: &str) -> bool {
        self.fanouts_started.remove(key);
        self.fanout_ids.remove(key);
        self.fanout_rerun.remove(key)
    }

    /// Counts a failed membership sync. The first failure retries on the next tick,
    /// later ones back off up to a minute.
    pub(crate) fn note_membership_failure(&mut self) {
        self.membership_failures += 1;
        self.membership_retry_at = (self.membership_failures > 1).then(|| {
            let exp = (self.membership_failures - 2).min(3);
            Instant::now() + (MEMBERSHIP_RETRY_BASE * 2u32.pow(exp)).min(MEMBERSHIP_RETRY_MAX)
        });
    }

    pub(crate) fn membership_retry_due(&self) -> bool {
        self.membership_retry_at
            .is_none_or(|at| Instant::now() >= at)
    }

    /// Classifies a scoped fan-out message. A fan-out ends with the first data message
    /// (full or partial) or failure that carries its id; the key's id is dropped when it
    /// ends, so a later message of the same fan-out, or one of a superseded or unrelated
    /// fetch (id 0), ends nothing and every fan-out ends exactly once.
    pub(crate) fn scoped_ends_fanout(&mut self, key: &str, id: u64, kind: ScopedKind) -> bool {
        match kind {
            ScopedKind::Result | ScopedKind::Failed => {
                id != 0 && self.fanout_ids.get(key) == Some(&id)
            }
            ScopedKind::NextPage | ScopedKind::Other => false,
        }
    }

    fn begin_fanout(&mut self, key: String, rerun_if_busy: bool) -> bool {
        if self.member_project_ids.is_empty() {
            if self.is_syncing_membership {
                self.status_message = Some("Loading projects...".into());
            }
            return false;
        }
        if self.fanout_in_flight(&key) {
            if rerun_if_busy {
                self.fanout_rerun.insert(key);
            }
            return false;
        }
        self.fanout_seq += 1;
        self.fanout_ids.insert(key.clone(), self.fanout_seq);
        self.fanouts_started.insert(key, Instant::now());
        true
    }

    /// Global issues fan-out, skipped while one for the same filter is in flight.
    /// With `rerun_if_busy` a follow-up runs when the running one ends.
    pub(crate) async fn fetch_global_issues_guarded(&mut self, rerun_if_busy: bool) {
        let key = issues_fanout_key(None, self.issue_filter);
        if self.begin_fanout(key, rerun_if_busy) {
            let _ = self.fetch_global_issues().await;
        }
    }

    pub(crate) async fn fetch_global_mrs_guarded(&mut self, rerun_if_busy: bool) {
        let key = format!("mr:{:?}", self.mr_filter);
        if self.begin_fanout(key, rerun_if_busy) {
            let _ = self.fetch_global_merge_requests().await;
        }
    }

    /// Starts the fetch of the cross-project list the user is looking at.
    /// `ids_changed` asks for a follow-up when a fan-out over the old ids is still running.
    pub(crate) async fn fetch_active_global_list(&mut self, ids_changed: bool) {
        let on_list = matches!(self.active_context, Context::Dashboard | Context::Explore)
            && self.view_mode == ViewMode::List
            && self.active_panel != Panel::MainDetail;
        if !on_list {
            return;
        }
        match self.active_selection {
            MenuSelection::Issues => self.fetch_global_issues_guarded(ids_changed).await,
            MenuSelection::MergeRequests => self.fetch_global_mrs_guarded(ids_changed).await,
            MenuSelection::Pipelines => {
                let _ = if ids_changed {
                    self.fetch_cross_pipelines_manual().await
                } else {
                    self.fetch_cross_pipelines().await
                };
            }
            MenuSelection::Home => self.schedule_home_prefetch(),
            _ => {}
        }
    }

    pub(crate) fn schedule_home_prefetch(&mut self) {
        if self.home_prefetch_due.is_none()
            && self.merge_requests.is_empty()
            && !self.member_project_ids.is_empty()
        {
            self.home_prefetch_due = Some(Instant::now() + HOME_PREFETCH_DELAY);
        }
    }

    /// Time until the next deferred task is due, for the event loop's wait.
    pub fn next_deferred_in(&self) -> Option<Duration> {
        self.home_prefetch_due
            .map(|due| due.saturating_duration_since(Instant::now()))
    }

    /// Runs deferred work that came due. The Home prefetch is dropped when the user left Home.
    pub async fn run_deferred(&mut self) {
        let Some(due) = self.home_prefetch_due else {
            return;
        };
        if Instant::now() < due {
            return;
        }
        self.home_prefetch_due = None;
        let on_home = self.active_context == Context::Dashboard
            && self.active_selection == MenuSelection::Home
            && self.view_mode == ViewMode::List;
        if on_home {
            self.fetch_global_mrs_guarded(false).await;
        }
    }
}

/// `/user` with its HTTP status in hand: only a 401 or 403 answer is an authentication failure.
async fn fetch_current_user(client: &crate::trace::TracedClient) -> Result<Author, (String, bool)> {
    use gitlab::api::{AsyncClient, RestClient};
    let url = client
        .rest_endpoint("user")
        .map_err(|e| (e.to_string(), false))?;
    let req = http::Request::builder().method("GET").uri(url.as_str());
    let resp = tokio::time::timeout(Duration::from_secs(30), client.rest_async(req, Vec::new()))
        .await
        .map_err(|_| ("request timed out".to_string(), false))?
        .map_err(|e| (e.to_string(), false))?;
    let status = resp.status();
    if !status.is_success() {
        let auth = matches!(status.as_u16(), 401 | 403);
        return Err((format!("request failed ({status})"), auth));
    }
    serde_json::from_slice(resp.body())
        .map_err(|e| (format!("could not parse JSON response: {e}"), false))
}
