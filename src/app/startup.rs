use crate::models::*;
use gitlab::api::AsyncQuery;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// A fan-out that never reported back (its task died) stops blocking new ones after this.
const FANOUT_STALE: Duration = Duration::from_secs(90);
/// How long Home waits before prefetching merge requests, so a screen opened
/// right away gets the network first.
pub const HOME_PREFETCH_DELAY: Duration = Duration::from_millis(400);

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

    /// Drops cached ids that belong to another user once `/user` answers.
    pub(crate) fn on_current_user(&mut self, id: u64) {
        if self.cached_user_id.is_some_and(|cached| cached != id) {
            self.member_project_ids.clear();
            self.filter_lists();
        }
        self.cached_user_id = Some(id);
        self.save_membership_cache();
    }

    /// Fetches `/user` once: later calls do nothing while one is in flight or the user is known.
    pub(crate) fn start_user_fetch(&mut self, counted: bool) {
        if self.current_user.is_some() || self.user_fetch_in_flight.swap(true, Ordering::AcqRel) {
            return;
        }
        let guard = ClearOnDrop(self.user_fetch_in_flight.clone());
        let c = self.gitlab.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let _guard = guard;
            let res: Result<Author, _> = crate::api::CurrentUser.query_async(&c).await;
            match res {
                Ok(user) => {
                    let _ = tx.send(DataUpdate::CurrentUser(user));
                }
                Err(e) => {
                    let _ = tx.send(DataUpdate::Error(format!(
                        "Could not connect to GitLab: {}",
                        crate::errors::friendly_error(&e.to_string())
                    )));
                }
            }
            if counted {
                let _ = tx.send(DataUpdate::Loading(false));
            }
        });
    }

    fn fanout_in_flight(&self, key: &str) -> bool {
        self.fanouts_started
            .get(key)
            .is_some_and(|t| t.elapsed() < FANOUT_STALE)
    }

    pub(crate) fn fanout_done(&mut self, key: &str) {
        self.fanouts_started.remove(key);
    }

    /// Global issues fan-out, skipped while one for the same filter is in flight.
    pub(crate) async fn fetch_global_issues_guarded(&mut self) {
        if self.member_project_ids.is_empty() {
            return;
        }
        let key = format!("issues:{:?}", self.issue_filter);
        if self.fanout_in_flight(&key) {
            return;
        }
        self.fanouts_started.insert(key, Instant::now());
        let _ = self.fetch_global_issues().await;
    }

    /// Global merge request fan-out, skipped while one for the same filter is in flight.
    pub(crate) async fn fetch_global_mrs_guarded(&mut self) {
        if self.member_project_ids.is_empty() {
            return;
        }
        let key = format!("mr:{:?}", self.mr_filter);
        if self.fanout_in_flight(&key) {
            return;
        }
        self.fanouts_started.insert(key, Instant::now());
        let _ = self.fetch_global_merge_requests().await;
    }

    /// Starts the fetch of the cross-project list the user is looking at.
    pub(crate) async fn fetch_active_global_list(&mut self) {
        let on_list = matches!(self.active_context, Context::Dashboard | Context::Explore)
            && self.view_mode == ViewMode::List
            && self.active_panel != Panel::MainDetail;
        if !on_list {
            return;
        }
        match self.active_selection {
            MenuSelection::Issues => self.fetch_global_issues_guarded().await,
            MenuSelection::MergeRequests => self.fetch_global_mrs_guarded().await,
            MenuSelection::Pipelines => {
                let _ = self.fetch_cross_pipelines().await;
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
            self.fetch_global_mrs_guarded().await;
        }
    }
}
