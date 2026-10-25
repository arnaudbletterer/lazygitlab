use crate::cache::{self, CacheEntry, CachedItem, ListCache};
use crate::graphql::{self, GraphqlError, GraphqlState};
use crate::http::{self, HttpLayer, Pages};
use crate::models::*;
use crate::trace::TracedClient;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;

pub const FULL_RECONCILE: Duration = Duration::from_secs(600);
pub const CACHE_FLUSH_EVERY: Duration = Duration::from_secs(30);
const REST_MAX_PAGES: u32 = 5;
const REST_PROJECTS_IN_FLIGHT: usize = 12;

#[derive(Debug, Clone)]
pub struct MrFetch {
    pub scope: MergeRequestFilter,
    pub project: Option<u64>,
    pub project_ids: Vec<u64>,
    pub user_id: Option<u64>,
    pub since: Option<String>,
}

pub fn state_of(scope: MergeRequestFilter) -> &'static str {
    match scope {
        MergeRequestFilter::Opened | MergeRequestFilter::Review | MergeRequestFilter::Assigned => {
            "opened"
        }
        MergeRequestFilter::Merged => "merged",
        MergeRequestFilter::Closed => "closed",
        MergeRequestFilter::All => "all",
    }
}

fn encode(v: &str) -> String {
    v.replace('+', "%2B")
}

fn rest_path(q: &MrFetch, pid: u64) -> String {
    let mut path = format!(
        "projects/{pid}/merge_requests?state={}&order_by=updated_at&sort=desc&with_labels_details=true",
        state_of(q.scope)
    );
    match (q.scope, q.user_id) {
        (MergeRequestFilter::Assigned, Some(id)) => path.push_str(&format!("&assignee_id={id}")),
        (MergeRequestFilter::Review, Some(id)) => path.push_str(&format!("&reviewer_id={id}")),
        _ => {}
    }
    if let Some(since) = &q.since {
        path.push_str(&format!("&updated_after={}", encode(since)));
    }
    path
}

fn meta_of(mr: &MergeRequest) -> MrMeta {
    MrMeta {
        pipeline_id: mr
            .head_pipeline
            .as_ref()
            .map(|p| p.id)
            .filter(|id| *id != 0),
        milestone_id: mr.milestone.as_ref().map(|m| m.id).filter(|id| *id != 0),
    }
}

fn secs_until(until: tokio::time::Instant) -> u64 {
    until
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs()
        .max(1)
}

fn involves(mr: &MergeRequest, scope: MergeRequestFilter, user: Option<u64>) -> bool {
    match (scope, user) {
        (MergeRequestFilter::Assigned, Some(id)) => mr.assignees.iter().any(|a| a.id == id),
        (MergeRequestFilter::Review, Some(id)) => mr.reviewers.iter().any(|a| a.id == id),
        _ => true,
    }
}

/// Single entry point of the merge request lists. The dashboard list asks GraphQL first
/// (one query per batch of projects, in parallel) and falls back to REST when the server
/// does not support it or answers nothing. REST goes through the shared request layer, so
/// its concurrency is bounded by the layer budget. Never touches app state: the caller
/// applies the returned update under the merge contract.
pub async fn fetch_global_mrs(
    client: &TracedClient,
    http: &HttpLayer,
    gql: &GraphqlState,
    q: &MrFetch,
) -> MrListUpdate {
    let mut out = MrListUpdate {
        scope: Some(q.scope),
        project: q.project,
        incremental: q.since.is_some(),
        complete: true,
        ..Default::default()
    };
    if q.project.is_none() {
        match graphql::fetch_merge_requests(
            client,
            gql,
            &q.project_ids,
            state_of(q.scope),
            q.since.as_deref(),
        )
        .await
        {
            Ok(res) if !(res.rows.is_empty() && !res.complete && !res.rate_limited) => {
                for row in res.rows {
                    if !involves(&row.mr, q.scope, q.user_id) {
                        continue;
                    }
                    out.meta.push((
                        (row.mr.project_id, row.mr.iid),
                        MrMeta {
                            pipeline_id: row.pipeline_id,
                            milestone_id: row.milestone_id,
                        },
                    ));
                    out.rows.push(row.mr);
                }
                out.complete = res.complete;
                if res.rate_limited {
                    out.rate_limited_secs = Some(http::DEFAULT_RETRY_AFTER.as_secs());
                }
                return out;
            }
            Ok(_) | Err(GraphqlError::Unsupported) => {}
            Err(GraphqlError::RateLimited) => {
                out.complete = false;
                out.rate_limited_secs = Some(http::DEFAULT_RETRY_AFTER.as_secs());
                return out;
            }
            Err(GraphqlError::Failed(m)) => {
                out.complete = false;
                out.error = Some(m);
                return out;
            }
        }
    }
    rest_fan_out(http, q, &mut out).await;
    out
}

async fn rest_fan_out(http: &HttpLayer, q: &MrFetch, out: &mut MrListUpdate) {
    let mut pending = q.project_ids.iter().copied();
    let mut set: JoinSet<(u64, Pages<MergeRequest>)> = JoinSet::new();
    let spawn = |set: &mut JoinSet<(u64, Pages<MergeRequest>)>, pid: u64| {
        let (http, path) = (http.clone(), rest_path(q, pid));
        set.spawn(async move { (pid, http.get_pages_capped(&path, REST_MAX_PAGES).await) });
    };
    for pid in pending.by_ref().take(REST_PROJECTS_IN_FLIGHT) {
        spawn(&mut set, pid);
    }
    let mut results = Vec::new();
    while let Some(joined) = set.join_next().await {
        if let Some(pid) = pending.next() {
            spawn(&mut set, pid);
        }
        match joined {
            Ok(r) => results.push(r),
            Err(e) => {
                out.complete = false;
                out.error = Some(e.to_string());
            }
        }
    }
    results.sort_by_key(|(pid, _)| *pid);
    for (pid, pages) in results {
        if !pages.complete {
            out.complete = false;
            out.failed_projects.push(pid);
        }
        if let Some(until) = pages.rate_limited_until {
            let s = secs_until(until);
            out.rate_limited_secs = Some(out.rate_limited_secs.map_or(s, |c| c.max(s)));
        }
        for mr in pages.items {
            out.meta.push(((mr.project_id, mr.iid), meta_of(&mr)));
            out.rows.push(mr);
        }
    }
    if q.project_ids.is_empty() {
        out.complete = true;
    }
}

pub(super) struct LoadingGuard(tokio::sync::mpsc::UnboundedSender<DataUpdate>);

impl LoadingGuard {
    pub(super) fn new(tx: tokio::sync::mpsc::UnboundedSender<DataUpdate>) -> Self {
        Self(tx)
    }
}

impl Drop for LoadingGuard {
    fn drop(&mut self) {
        let _ = self.0.send(DataUpdate::Loading(false));
    }
}

fn scope_name(scope: MergeRequestFilter) -> String {
    format!("{scope:?}").to_lowercase()
}

pub fn rate_limit_text(secs: u64) -> String {
    format!("Rate limited by GitLab, retrying in {secs}s")
}

impl App {
    /// Turns on the persistent list cache for `host`. The user id comes from the cache
    /// file written last time until `/user` answers; without one nothing is cached.
    pub fn enable_cache(&mut self, dir: &std::path::Path, host: &str) {
        let user = cache::discover_user_id(dir, host);
        self.cache_host = host.to_string();
        self.list_cache = Some(Arc::new(ListCache::load(dir, host, user)));
    }

    pub(crate) fn on_user_known(&mut self, id: u64) {
        let Some(id) = NonZeroU64::new(id) else {
            return;
        };
        let Some(cache) = &self.list_cache else {
            return;
        };
        let previous = cache.last_user_id();
        cache.invalidate_user(id);
        if previous.is_some_and(|p| p != id) && self.mrs_cached_at.take().is_some() {
            self.merge_requests.clear();
            self.mrs_sync = MrSync::default();
            self.filter_lists();
        }
    }

    pub fn flush_cache(&self) {
        if let Some(cache) = &self.list_cache
            && let Err(e) = cache.flush()
        {
            tracing::warn!("list cache flush failed: {e}");
        }
    }

    pub fn flush_cache_in_background(&self) {
        if let Some(cache) = self.list_cache.clone() {
            tokio::task::spawn_blocking(move || {
                if let Err(e) = cache.flush() {
                    tracing::warn!("list cache flush failed: {e}");
                }
            });
        }
    }

    fn mr_scope_key(&self, project: Option<u64>) -> String {
        let scope = match project {
            Some(pid) => format!("project:{pid}"),
            None => "global".to_string(),
        };
        cache::query_key(
            "mr",
            &scope_name(self.mr_filter),
            state_of(self.mr_filter),
            &scope,
        )
    }

    fn current_mr_project(&self) -> Option<Option<u64>> {
        match self.active_context {
            Context::Dashboard => Some(None),
            Context::Project => self.selected_project.as_ref().map(|p| Some(p.id)),
            Context::Explore => None,
        }
    }

    fn paint_cached_mrs(&mut self, key: &str) {
        if !self.merge_requests.is_empty() || self.mrs_cached_at.is_some() {
            return;
        }
        let Some(entry) = self.list_cache.as_ref().and_then(|c| c.get(key)) else {
            return;
        };
        let mut rows: Vec<MergeRequest> = entry
            .items
            .iter()
            .filter_map(|i| match i {
                CachedItem::Mr(m) => Some(m.to_model(&self.cache_host)),
                _ => None,
            })
            .collect();
        if rows.is_empty() {
            return;
        }
        Self::sort_mrs(&mut rows);
        self.merge_requests = rows;
        self.mrs_cached_at = Some(entry.fetched_at);
        self.mrs_page = 1;
        self.mrs_has_more = false;
        self.mrs_sync = MrSync {
            key: key.to_string(),
            ..Default::default()
        };
        self.filter_lists();
    }

    fn write_mr_cache(&self, key: &str, cursor: Option<String>) {
        let Some(cache) = &self.list_cache else {
            return;
        };
        let items = self
            .merge_requests
            .iter()
            .map(|m| CachedItem::Mr(m.into()))
            .collect();
        cache.put(key, CacheEntry::new(items, None, cursor));
    }

    pub fn mr_pipeline_id(&self, mr: &MergeRequest) -> Option<u64> {
        match self.mr_meta.get(&(mr.project_id, mr.iid)) {
            Some(m) => m.pipeline_id,
            None => mr
                .head_pipeline
                .as_ref()
                .map(|p| p.id)
                .filter(|id| *id != 0),
        }
    }

    pub fn mr_milestone_known(&self, mr: &MergeRequest) -> bool {
        mr.milestone.is_none()
            || match self.mr_meta.get(&(mr.project_id, mr.iid)) {
                Some(m) => m.milestone_id.is_some(),
                None => mr.milestone.as_ref().is_some_and(|m| m.id != 0),
            }
    }

    /// Starts a merge request list fetch for the dashboard (`None`) or one project.
    /// Cached rows are painted first when the list is empty.
    pub(super) fn start_mr_fetch(&mut self, project: Option<u64>) {
        let key = self.mr_scope_key(project);
        self.status_message = Some("Fetching Merge Requests...".into());
        self.paint_cached_mrs(&key);
        let project_ids: Vec<u64> = match project {
            Some(pid) => vec![pid],
            None => {
                let mut ids: Vec<u64> = self.member_project_ids.iter().copied().collect();
                ids.sort_unstable();
                ids
            }
        };
        if project.is_none() && project_ids.is_empty() {
            return;
        }
        let user_id = self.current_user.as_ref().map(|u| u.id).or_else(|| {
            self.list_cache
                .as_ref()
                .and_then(|c| c.last_user_id())
                .map(NonZeroU64::get)
        });
        let needs_user = matches!(
            self.mr_filter,
            MergeRequestFilter::Review | MergeRequestFilter::Assigned
        );
        if needs_user && user_id.is_none() {
            return;
        }
        let force = std::mem::take(&mut self.mrs_force_full);
        if !force && self.mrs_in_flight.as_deref() == Some(key.as_str()) {
            return;
        }
        if self.mrs_sync.key != key {
            self.mrs_sync = MrSync {
                key: key.clone(),
                ..Default::default()
            };
        }
        let incremental = !force
            && !self.merge_requests.is_empty()
            && self.mrs_cached_at.is_none()
            && self
                .mrs_sync
                .last_full
                .is_some_and(|t| t.elapsed() < FULL_RECONCILE);
        let since = if incremental {
            self.mrs_sync.cursor.clone()
        } else {
            None
        };
        let q = MrFetch {
            scope: self.mr_filter,
            project,
            project_ids,
            user_id,
            since,
        };
        self.mrs_in_flight = Some(key);
        let (client, http, gql, tx) = (
            self.gitlab.clone(),
            self.http.clone(),
            self.graphql.clone(),
            self.tx.clone(),
        );
        let _ = tx.send(DataUpdate::Loading(true));
        tokio::spawn(async move {
            let _guard = LoadingGuard::new(tx.clone());
            let update = fetch_global_mrs(&client, &http, &gql, &q).await;
            let _ = tx.send(DataUpdate::MrList(Box::new(update)));
        });
    }

    pub(crate) fn apply_mr_list(&mut self, mut u: MrListUpdate) {
        let Some(scope) = u.scope else {
            return;
        };
        let key = {
            let saved = self.mr_filter;
            self.mr_filter = scope;
            let key = self.mr_scope_key(u.project);
            self.mr_filter = saved;
            key
        };
        if self.mrs_in_flight.as_deref() == Some(key.as_str()) {
            self.mrs_in_flight = None;
        }
        if scope != self.mr_filter || self.current_mr_project() != Some(u.project) {
            return;
        }
        for mr in &mut u.rows {
            self.fix_mr_labels(mr);
        }
        let had_rows = !u.rows.is_empty();
        let replace = !u.incremental && (u.complete || !u.failed_projects.is_empty());
        let replace_all = replace && u.complete;
        let failed = if replace_all {
            Vec::new()
        } else {
            u.failed_projects.clone()
        };
        let apply = had_rows || (replace && u.error.is_none());
        let mut changed = false;
        if apply {
            for (k, m) in u.meta.drain(..) {
                self.mr_meta.insert(k, m);
            }
            changed = self.merge_mr_rows(u.rows, &failed, replace);
        }
        if changed {
            self.mrs_mutations += 1;
            self.mrs_page = 1;
            self.mrs_has_more = false;
            self.filter_lists();
        }
        let max_updated = self
            .merge_requests
            .iter()
            .filter_map(|m| m.updated_at.as_deref())
            .max()
            .map(str::to_string);
        if u.complete && u.error.is_none() && u.rate_limited_secs.is_none() {
            self.mrs_stale = false;
            self.mrs_cached_at = None;
            self.last_list_refresh_ok = Some(chrono::Local::now());
            if !u.incremental {
                self.mrs_sync.cursor = max_updated.clone();
                self.mrs_sync.last_full = Some(std::time::Instant::now());
            } else if max_updated.is_some() {
                self.mrs_sync.cursor = max_updated.clone();
            }
            if changed || !u.incremental {
                self.write_mr_cache(&key, self.mrs_sync.cursor.clone());
            }
            if self
                .status_message
                .as_deref()
                .is_some_and(|m| m.starts_with("Fetching") || m.starts_with("Rate limited"))
            {
                self.status_message = None;
            }
        } else {
            self.mrs_stale = true;
            if !u.incremental {
                self.mrs_sync.cursor = None;
                self.mrs_sync.last_full = None;
            }
            let when = self
                .last_list_refresh_ok
                .map(|t| format!(", showing data from {}", t.format("%H:%M")))
                .unwrap_or_default();
            self.status_message = Some(match u.rate_limited_secs {
                Some(secs) => format!("{}{when}", rate_limit_text(secs)),
                None => format!("refresh failed{when}"),
            });
        }
    }

    /// Cancels the detail fetch of the row the cursor left.
    pub(crate) fn abort_mr_detail(&mut self) {
        self.pending_mr_detail = None;
        if let Some(task) = self.mr_detail_task.take() {
            task.abort();
        }
    }

    /// Fires the debounced detail fetch once the cursor has rested on its row.
    pub async fn fire_pending_detail(&mut self) {
        let Some((pid, iid, since)) = self.pending_mr_detail else {
            return;
        };
        let rested =
            since.elapsed() >= Duration::from_millis(250) || self.active_panel == Panel::MainDetail;
        if !rested {
            return;
        }
        self.pending_mr_detail = None;
        if self
            .selected_mr
            .as_ref()
            .is_some_and(|m| m.project_id == pid && m.iid == iid)
        {
            let _ = self.fetch_discussions(pid, iid).await;
        }
    }
}
