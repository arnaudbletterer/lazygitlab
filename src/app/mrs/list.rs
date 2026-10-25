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
const PROGRESS_EVERY: Duration = Duration::from_millis(100);
const REST_PROJECTS_IN_FLIGHT: usize = 6;
const CURSOR_MARGIN_SECS: i64 = 60;

#[derive(Debug, Clone)]
pub struct MrFetch {
    pub scope: MergeRequestFilter,
    pub project: Option<u64>,
    pub project_ids: Vec<u64>,
    pub user_id: Option<u64>,
    pub since: Option<String>,
    pub fetch_gen: u64,
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

/// An incremental pass asks for every state, so a row that left the scope shows up
/// as changed and can be dropped locally.
fn query_state(q: &MrFetch) -> &'static str {
    if q.since.is_some() {
        "all"
    } else {
        state_of(q.scope)
    }
}

fn state_matches(mr: &MergeRequest, scope: MergeRequestFilter) -> bool {
    match scope {
        MergeRequestFilter::All => true,
        _ => mr.state == state_of(scope),
    }
}

fn since_with_margin(cursor: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(cursor)
        .map(|t| {
            (t - chrono::Duration::seconds(CURSOR_MARGIN_SECS))
                .with_timezone(&chrono::Utc)
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string()
        })
        .unwrap_or_else(|_| cursor.to_string())
}

fn encode(v: &str) -> String {
    v.replace('+', "%2B")
}

fn rest_path(q: &MrFetch, pid: u64) -> String {
    let mut path = format!(
        "projects/{pid}/merge_requests?state={}&order_by=updated_at&sort=desc&with_labels_details=true",
        query_state(q)
    );
    match (q.scope, q.user_id) {
        _ if q.since.is_some() => {}
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

pub(super) fn matches_scope(
    mr: &MergeRequest,
    scope: MergeRequestFilter,
    user: Option<u64>,
) -> bool {
    state_matches(mr, scope) && involves(mr, scope, user)
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
    progress: Option<&tokio::sync::mpsc::UnboundedSender<DataUpdate>>,
) -> MrListUpdate {
    let mut out = MrListUpdate {
        scope: Some(q.scope),
        project: q.project,
        incremental: q.since.is_some(),
        complete: true,
        user_id: q.user_id,
        fetch_gen: q.fetch_gen,
        ..Default::default()
    };
    if q.project.is_none() {
        match graphql::fetch_merge_requests(
            client,
            gql,
            &q.project_ids,
            query_state(q),
            q.since.as_deref(),
        )
        .await
        {
            Ok(res) if !(res.rows.is_empty() && !res.complete && !res.rate_limited) => {
                for row in res.rows {
                    if q.since.is_none() && !involves(&row.mr, q.scope, q.user_id) {
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
    rest_fan_out(http, q, progress, &mut out).await;
    out
}

async fn rest_fan_out(
    http: &HttpLayer,
    q: &MrFetch,
    progress: Option<&tokio::sync::mpsc::UnboundedSender<DataUpdate>>,
    out: &mut MrListUpdate,
) {
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
    let mut unsent: Vec<MergeRequest> = Vec::new();
    let mut last_sent = tokio::time::Instant::now();
    while let Some(joined) = set.join_next().await {
        if let Some(pid) = pending.next() {
            spawn(&mut set, pid);
        }
        match joined {
            Ok(r) => {
                if progress.is_some() && q.since.is_none() {
                    unsent.extend(r.1.items.iter().cloned());
                    if !unsent.is_empty() && last_sent.elapsed() >= PROGRESS_EVERY {
                        send_progress(progress, q, std::mem::take(&mut unsent));
                        last_sent = tokio::time::Instant::now();
                    }
                }
                results.push(r)
            }
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

/// Ends one list fetch: always releases the loading marker and, when the task died
/// before sending its result (abort or panic), reports the failure so the in-flight
/// marker is released.
struct FetchGuard {
    tx: tokio::sync::mpsc::UnboundedSender<DataUpdate>,
    scope: MergeRequestFilter,
    project: Option<u64>,
    fetch_gen: u64,
    sent: bool,
}

impl FetchGuard {
    fn send(&mut self, update: MrListUpdate) {
        self.sent = true;
        let _ = self.tx.send(DataUpdate::MrList(Box::new(update)));
    }
}

impl Drop for FetchGuard {
    fn drop(&mut self) {
        if !self.sent {
            let _ = self.tx.send(DataUpdate::MrList(Box::new(MrListUpdate {
                scope: Some(self.scope),
                project: self.project,
                complete: false,
                error: Some("list fetch ended unexpectedly".into()),
                fetch_gen: self.fetch_gen,
                ..Default::default()
            })));
        }
        let _ = self.tx.send(DataUpdate::Loading(false));
    }
}

fn send_progress(
    tx: Option<&tokio::sync::mpsc::UnboundedSender<DataUpdate>>,
    q: &MrFetch,
    rows: Vec<MergeRequest>,
) {
    let Some(tx) = tx else {
        return;
    };
    let meta = rows
        .iter()
        .map(|m| ((m.project_id, m.iid), meta_of(m)))
        .collect();
    let _ = tx.send(DataUpdate::MrList(Box::new(MrListUpdate {
        scope: Some(q.scope),
        project: q.project,
        rows,
        meta,
        progress: true,
        user_id: q.user_id,
        fetch_gen: q.fetch_gen,
        ..Default::default()
    })));
}

fn scope_name(scope: MergeRequestFilter) -> String {
    format!("{scope:?}").to_lowercase()
}

pub fn rate_limit_text(secs: u64) -> String {
    http::rate_limit_message(secs)
}

impl App {
    /// Turns on the persistent list cache for `host`. The user id comes from the cache
    /// file written last time until `/user` answers; without one nothing is cached.
    pub fn enable_cache(&mut self, dir: &std::path::Path, host: &str) {
        let user = cache::discover_user_id(dir, host);
        self.cache_host = host.to_string();
        self.list_cache = Some(Arc::new(ListCache::load(dir, host, user)));
    }

    /// Returns true when the user differs from the one the shown rows were loaded for:
    /// the rows are dropped and the caller refetches.
    pub fn on_user_known(&mut self, id: u64) -> bool {
        let Some(id) = NonZeroU64::new(id) else {
            return false;
        };
        let previous_shown = self.current_user.as_ref().map(|u| u.id);
        let previous = match &self.list_cache {
            Some(cache) => {
                let previous = cache.last_user_id();
                cache.invalidate_user(id);
                previous
            }
            None => None,
        };
        let changed =
            previous.is_some_and(|p| p != id) || previous_shown.is_some_and(|p| p != id.get());
        if changed {
            self.merge_requests.clear();
            self.issues.clear();
            self.todos.clear();
            self.todos_etag = None;
            self.selected_mr = None;
            self.selected_issue = None;
            self.mr_edit_snapshots.clear();
            self.issue_edit_snapshots.clear();
            self.mr_meta.clear();
            self.mrs_cached_at = None;
            self.mrs_stale = false;
            self.mrs_sync = MrSync::default();
            self.mrs_force_full = true;
            self.filter_lists();
        }
        changed
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
        if !self.merge_requests.is_empty() {
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

    pub fn write_mr_cache(&self, key: &str, cursor: Option<String>) {
        let Some(cache) = &self.list_cache else {
            return;
        };
        let items = self
            .merge_requests
            .iter()
            .filter(|m| !super::super::edits::is_pending_iid(m.iid))
            .map(|m| {
                let row = self
                    .mr_edit_snapshots
                    .get(&(m.project_id, m.iid))
                    .and_then(|s| s.row.as_ref())
                    .unwrap_or(m);
                CachedItem::Mr(row.into())
            })
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
                .is_some_and(|t| t.elapsed().is_ok_and(|e| e < FULL_RECONCILE));
        let since = if incremental {
            self.mrs_sync.cursor.as_deref().map(since_with_margin)
        } else {
            None
        };
        let q = MrFetch {
            scope: self.mr_filter,
            project,
            project_ids,
            user_id,
            since,
            fetch_gen: self.mrs_fetch_gen + 1,
        };
        let fanout_key = format!("mr:{:?}", self.mr_filter);
        if let Some(task) = self.mrs_fetch_task.take() {
            task.abort();
            if let Some(old) = self.mrs_fetch_key.take()
                && old != fanout_key
            {
                self.fanout_done(&old);
            }
        }
        self.mrs_fetch_gen += 1;
        self.mrs_fetch_key = Some(fanout_key);
        self.mrs_in_flight = Some(key);
        let (client, http, gql, tx) = (
            self.gitlab.clone(),
            self.http.clone(),
            self.graphql.clone(),
            self.tx.clone(),
        );
        let _ = tx.send(DataUpdate::Loading(true));
        let mut guard = FetchGuard {
            tx: tx.clone(),
            scope: q.scope,
            project,
            fetch_gen: q.fetch_gen,
            sent: false,
        };
        self.mrs_fetch_task = Some(tokio::spawn(async move {
            let update = fetch_global_mrs(&client, &http, &gql, &q, Some(&tx)).await;
            guard.send(update);
        }));
    }

    /// Returns false when the update belongs to a superseded fetch and was ignored.
    pub(crate) fn apply_mr_list(&mut self, mut u: MrListUpdate) -> bool {
        let Some(scope) = u.scope else {
            return false;
        };
        if u.fetch_gen != 0 && u.fetch_gen != self.mrs_fetch_gen {
            return false;
        }
        let key = {
            let saved = self.mr_filter;
            self.mr_filter = scope;
            let key = self.mr_scope_key(u.project);
            self.mr_filter = saved;
            key
        };
        if !u.progress && self.mrs_in_flight.as_deref() == Some(key.as_str()) {
            self.mrs_in_flight = None;
        }
        if scope != self.mr_filter || self.current_mr_project() != Some(u.project) {
            return true;
        }
        if let (Some(fetched_for), Some(me)) = (u.user_id, self.current_user.as_ref().map(|c| c.id))
            && fetched_for != me
        {
            return !u.progress;
        }
        for mr in &mut u.rows {
            self.fix_mr_labels(mr);
        }
        if u.progress {
            for (k, m) in u.meta.drain(..) {
                self.mr_meta.insert(k, m);
            }
            if self.merge_mr_rows(u.rows, &[], false) {
                self.mrs_mutations += 1;
                self.filter_lists();
            }
            return false;
        }
        let fetched_max = u
            .rows
            .iter()
            .filter_map(|m| m.updated_at.as_deref())
            .max()
            .map(str::to_string);
        let mut dropped: Vec<(u64, u64)> = Vec::new();
        if u.incremental {
            let user = self.current_user.as_ref().map(|c| c.id).or_else(|| {
                self.list_cache
                    .as_ref()
                    .and_then(|c| c.last_user_id())
                    .map(NonZeroU64::get)
            });
            let (keep, gone): (Vec<_>, Vec<_>) = u
                .rows
                .into_iter()
                .partition(|m| matches_scope(m, scope, user));
            dropped = gone.iter().map(|m| (m.project_id, m.iid)).collect();
            u.rows = keep;
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
        if !dropped.is_empty() {
            for k in &dropped {
                self.mr_meta.remove(k);
            }
            changed |= self.remove_mr_keys(&dropped);
        }
        if changed {
            self.mrs_mutations += 1;
            self.mrs_page = 1;
            self.mrs_has_more = false;
            self.filter_lists();
        }
        let max_updated = match (&self.mrs_sync.cursor, fetched_max) {
            (Some(prev), Some(new)) if u.incremental => Some(new.max(prev.clone())),
            (Some(prev), None) if u.incremental => Some(prev.clone()),
            (_, new) => new,
        };
        if u.complete && u.error.is_none() && u.rate_limited_secs.is_none() {
            self.mrs_stale = false;
            self.mrs_cached_at = None;
            self.last_list_refresh_ok = Some(chrono::Local::now());
            if !u.incremental {
                self.mrs_sync.cursor = max_updated.clone();
                self.mrs_sync.last_full = Some(std::time::SystemTime::now());
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
            } else if u.rate_limited_secs.is_none() {
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
        true
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
