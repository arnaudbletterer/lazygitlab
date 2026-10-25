use crate::models::*;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use tokio::sync::mpsc::UnboundedSender;

/// Holds one `loading_tasks` count; a page guard also frees its in-flight key after the page's update.
pub(crate) struct LoadingGuard {
    tx: UnboundedSender<DataUpdate>,
    page: Option<&'static str>,
}

impl LoadingGuard {
    pub(crate) fn new(tx: UnboundedSender<DataUpdate>) -> Self {
        Self { tx, page: None }
    }

    pub(crate) fn start(tx: UnboundedSender<DataUpdate>) -> Self {
        let _ = tx.send(DataUpdate::Loading(true));
        Self { tx, page: None }
    }

    fn page(tx: UnboundedSender<DataUpdate>, key: &'static str) -> Self {
        let _ = tx.send(DataUpdate::Loading(true));
        Self {
            tx,
            page: Some(key),
        }
    }
}

impl Drop for LoadingGuard {
    fn drop(&mut self) {
        if let Some(key) = self.page {
            let _ = self.tx.send(DataUpdate::PageDone(key));
        }
        let _ = self.tx.send(DataUpdate::Loading(false));
    }
}

/// Rows after the last row still present in a full fresh page stay loaded (no server order to compare).
#[allow(clippy::too_many_arguments)]
fn replace_positional<T: Clone + PartialEq, K: Hash + Eq>(
    existing: &mut Vec<T>,
    fresh: Vec<T>,
    page: &mut u64,
    has_more: &mut bool,
    new_page: u64,
    new_has_more: bool,
    page_size: usize,
    key: impl Fn(&T) -> K,
) -> bool {
    let fresh_keys: HashSet<K> = fresh.iter().map(&key).collect();
    let anchor = existing.iter().rposition(|e| fresh_keys.contains(&key(e)));
    let tail: Vec<T> = match anchor {
        Some(i) if *page > 1 && fresh.len() >= page_size => existing[i + 1..]
            .iter()
            .filter(|e| !fresh_keys.contains(&key(e)))
            .cloned()
            .collect(),
        _ => Vec::new(),
    };
    let kept = !tail.is_empty();
    let mut next = fresh;
    next.extend(tail);
    let mut changed = *existing != next;
    *existing = next;
    if !kept && (*page != new_page || *has_more != new_has_more) {
        *page = new_page;
        *has_more = new_has_more;
        changed = true;
    }
    changed
}

type Key = (u64, u64);

fn upsert_by_key<T>(existing: &mut Vec<T>, fresh: Vec<T>, key: impl Fn(&T) -> Key) {
    let mut index: HashMap<Key, usize> = existing
        .iter()
        .enumerate()
        .map(|(i, e)| (key(e), i))
        .collect();
    for item in fresh {
        let k = key(&item);
        match index.get(&k) {
            Some(&i) => existing[i] = item,
            None => {
                index.insert(k, existing.len());
                existing.push(item);
            }
        }
    }
}

/// A complete first-page pass replaces the window it fetched. When later pages
/// are loaded (`keep_tail`), rows that sort at or above the last fresh row but
/// were not returned are gone (merged, closed, deleted); older rows stay.
/// Returns whether any older rows were kept.
fn first_page_merge<T>(
    existing: &mut Vec<T>,
    fresh: Vec<T>,
    keep_tail: bool,
    page_size: usize,
    key: impl Fn(&T) -> Key,
    project_of: impl Fn(&T) -> u64,
    after: impl Fn(&T, &T) -> bool,
) -> bool {
    if !keep_tail {
        *existing = fresh;
        return false;
    }
    let fresh_keys: HashSet<Key> = fresh.iter().map(&key).collect();
    let mut last_of_project: HashMap<u64, &T> = HashMap::new();
    let mut count_of_project: HashMap<u64, usize> = HashMap::new();
    for item in &fresh {
        last_of_project.insert(project_of(item), item);
        *count_of_project.entry(project_of(item)).or_default() += 1;
    }
    let tail: Vec<T> = std::mem::take(existing)
        .into_iter()
        .filter(|e| {
            !fresh_keys.contains(&key(e))
                && count_of_project
                    .get(&project_of(e))
                    .is_some_and(|c| *c >= page_size)
                && last_of_project
                    .get(&project_of(e))
                    .is_some_and(|last| after(e, last))
        })
        .collect();
    let kept = !tail.is_empty();
    *existing = fresh;
    existing.extend(tail);
    kept
}

#[allow(clippy::too_many_arguments)]
fn replace_window<T: Clone + PartialEq>(
    existing: &mut Vec<T>,
    fresh: Vec<T>,
    page: &mut u64,
    has_more: &mut bool,
    new_page: u64,
    new_has_more: bool,
    page_size: usize,
    key: impl Fn(&T) -> Key,
    project_of: impl Fn(&T) -> u64,
    after: impl Fn(&T, &T) -> bool,
) -> bool {
    let keep_tail = *page > 1;
    let old = existing.clone();
    let tail_kept = first_page_merge(
        existing, fresh, keep_tail, page_size, key, project_of, after,
    );
    let mut changed = old != *existing;
    if !tail_kept && (*page != new_page || *has_more != new_has_more) {
        *page = new_page;
        *has_more = new_has_more;
        changed = true;
    }
    changed
}

/// Replaces the rows of every project that answered and keeps the rows of the
/// projects that failed.
fn replace_partial<T>(
    existing: &mut Vec<T>,
    fresh: Vec<T>,
    failed: &[u64],
    keep_tail: bool,
    project_of: impl Fn(&T) -> u64,
    key: impl Fn(&T) -> Key,
    cmp: impl Fn(&T, &T) -> Ordering,
) {
    let (kept_failed, mut answered): (Vec<T>, Vec<T>) = std::mem::take(existing)
        .into_iter()
        .partition(|e| failed.contains(&project_of(e)));
    first_page_merge(
        &mut answered,
        fresh,
        keep_tail,
        50,
        &key,
        &project_of,
        |a, b| cmp(a, b) == Ordering::Greater,
    );
    answered.extend(kept_failed);
    answered.sort_by(cmp);
    *existing = answered;
}

fn mr_cmp(a: &MergeRequest, b: &MergeRequest) -> Ordering {
    let (ta, tb) = (
        a.updated_at.as_deref().or(a.created_at.as_deref()),
        b.updated_at.as_deref().or(b.created_at.as_deref()),
    );
    tb.cmp(&ta)
        .then(a.project_id.cmp(&b.project_id))
        .then(b.iid.cmp(&a.iid))
}

fn issue_cmp(a: &Issue, b: &Issue) -> Ordering {
    let (ta, tb) = (
        a.updated_at.as_deref().or(a.created_at.as_deref()),
        b.updated_at.as_deref().or(b.created_at.as_deref()),
    );
    tb.cmp(&ta)
        .then(a.project_id.cmp(&b.project_id))
        .then(b.iid.cmp(&a.iid))
}

/// Pipelines are listed by id desc. A round that is never complete never
/// removes rows, so pipelines deleted on the server linger until it completes.
fn pipeline_after(a: &Pipeline, b: &Pipeline) -> bool {
    a.id < b.id
}

impl App {
    pub(crate) fn sort_mrs(list: &mut [MergeRequest]) {
        list.sort_by(mr_cmp);
    }

    pub(crate) fn sort_issues(list: &mut [Issue]) {
        list.sort_by(issue_cmp);
    }

    /// Called whenever a list is cleared for a filter, tab or project switch.
    pub(crate) fn reset_list_paging(&mut self) {
        self.mrs_page = 1;
        self.mrs_has_more = true;
        self.issues_page = 1;
        self.issues_has_more = true;
        self.pipelines_page = 1;
        self.pipelines_has_more = true;
        self.todos_page = 1;
        self.todos_has_more = true;
    }

    pub(super) fn append_mrs(
        &mut self,
        data: Vec<MergeRequest>,
        has_more: bool,
        page: u64,
        complete: bool,
    ) {
        upsert_by_key(&mut self.merge_requests, data, |m| (m.project_id, m.iid));
        Self::sort_mrs(&mut self.merge_requests);
        if complete {
            self.mrs_page = page;
            self.mrs_has_more = has_more;
        }
        self.filter_lists();
    }

    pub(super) fn append_issues(
        &mut self,
        data: Vec<Issue>,
        has_more: bool,
        page: u64,
        complete: bool,
    ) {
        upsert_by_key(&mut self.issues, data, |i| (i.project_id, i.iid));
        Self::sort_issues(&mut self.issues);
        if complete {
            self.issues_page = page;
            self.issues_has_more = has_more;
        }
        self.filter_lists();
    }

    pub(super) fn replace_mrs_partial(&mut self, data: Vec<MergeRequest>, failed: &[u64]) {
        let keep_tail = self.mrs_page > 1;
        replace_partial(
            &mut self.merge_requests,
            data,
            failed,
            keep_tail,
            |m| m.project_id,
            |m| (m.project_id, m.iid),
            mr_cmp,
        );
        self.filter_lists();
    }

    pub(super) fn replace_issues_partial(&mut self, data: Vec<Issue>, failed: &[u64]) {
        let keep_tail = self.issues_page > 1;
        replace_partial(
            &mut self.issues,
            data,
            failed,
            keep_tail,
            |i| i.project_id,
            |i| (i.project_id, i.iid),
            issue_cmp,
        );
        self.filter_lists();
    }

    pub(super) fn upsert_cross_pipelines(&mut self, data: Vec<Pipeline>) {
        upsert_by_key(&mut self.cross_pipelines, data, |p| (p.project_id, p.id));
        self.cross_pipelines
            .sort_by_key(|p| std::cmp::Reverse(p.id));
        self.cross_pipelines.truncate(50);
        self.filter_lists();
    }

    /// Marks a list's next page as in flight; `None` while one already is.
    pub(crate) fn begin_page(&mut self, key: &'static str) -> Option<LoadingGuard> {
        if !self.page_in_flight.insert(key) {
            return None;
        }
        Some(LoadingGuard::page(self.tx.clone(), key))
    }

    pub(super) fn replace_projects(&mut self, data: Vec<Project>, page: u64, has_more: bool) {
        for p in &data {
            self.project_cache.insert(p.id, p.clone());
        }
        let changed = replace_positional(
            &mut self.projects,
            data,
            &mut self.projects_page,
            &mut self.projects_has_more,
            page,
            has_more,
            crate::app::global::PROJECTS_PER_PAGE as usize,
            |p| p.id,
        );
        if changed {
            self.filter_lists();
        }
    }

    pub(super) fn replace_groups(&mut self, data: Vec<Group>, page: u64, has_more: bool) {
        let changed = replace_positional(
            &mut self.groups,
            data,
            &mut self.groups_page,
            &mut self.groups_has_more,
            page,
            has_more,
            crate::app::global::GROUPS_PER_PAGE as usize,
            |g| g.id,
        );
        if changed {
            self.filter_lists();
        }
    }

    pub(super) fn replace_commits(&mut self, data: Vec<Commit>, page: u64, has_more: bool) {
        let changed = replace_positional(
            &mut self.commits,
            data,
            &mut self.commits_page,
            &mut self.commits_has_more,
            page,
            has_more,
            crate::app::global::COMMITS_PER_PAGE as usize,
            |c| c.id.clone(),
        );
        if changed {
            self.filter_lists();
        }
    }

    pub(super) fn apply_todos(&mut self, data: Vec<Todo>, page: u64, has_more: bool) {
        for t in &data {
            if let Some(ref p) = t.project {
                self.project_cache.insert(p.id, p.clone());
            }
        }
        let changed = replace_window(
            &mut self.todos,
            data,
            &mut self.todos_page,
            &mut self.todos_has_more,
            page,
            has_more,
            50,
            |t| (t.id, 0),
            |_| 0,
            |a, b| a.id < b.id,
        );
        if changed {
            self.filter_lists();
        }
    }

    pub(super) fn replace_mrs(&mut self, data: Vec<MergeRequest>, page: u64, has_more: bool) {
        self.last_list_refresh_ok = Some(chrono::Local::now());
        let changed = replace_window(
            &mut self.merge_requests,
            data,
            &mut self.mrs_page,
            &mut self.mrs_has_more,
            page,
            has_more,
            50,
            |m| (m.project_id, m.iid),
            |m| m.project_id,
            |a, b| mr_cmp(a, b) == Ordering::Greater,
        );
        if changed {
            self.filter_lists();
        }
    }

    pub(super) fn replace_issues(&mut self, data: Vec<Issue>, page: u64, has_more: bool) {
        self.last_list_refresh_ok = Some(chrono::Local::now());
        let changed = replace_window(
            &mut self.issues,
            data,
            &mut self.issues_page,
            &mut self.issues_has_more,
            page,
            has_more,
            50,
            |i| (i.project_id, i.iid),
            |i| i.project_id,
            |a, b| issue_cmp(a, b) == Ordering::Greater,
        );
        if changed {
            self.filter_lists();
        }
    }

    pub(super) fn replace_pipelines(&mut self, data: Vec<Pipeline>, page: u64, has_more: bool) {
        self.last_list_refresh_ok = Some(chrono::Local::now());
        let changed = replace_window(
            &mut self.pipelines,
            data,
            &mut self.pipelines_page,
            &mut self.pipelines_has_more,
            page,
            has_more,
            50,
            |p| (p.project_id, p.id),
            |p| p.project_id,
            pipeline_after,
        );
        if changed {
            self.filter_lists();
        }
    }

    pub(super) fn replace_cross_pipelines(
        &mut self,
        data: Vec<Pipeline>,
        page: u64,
        has_more: bool,
    ) {
        self.last_list_refresh_ok = Some(chrono::Local::now());
        let changed = replace_window(
            &mut self.cross_pipelines,
            data,
            &mut self.cross_pipelines_page,
            &mut self.cross_pipelines_has_more,
            page,
            has_more,
            20,
            |p| (p.project_id, p.id),
            |p| p.project_id,
            pipeline_after,
        );
        if changed {
            self.filter_lists();
        }
    }
}

fn upsert_changed(existing: &mut Vec<MergeRequest>, fresh: Vec<MergeRequest>) -> bool {
    let mut index: HashMap<Key, usize> = existing
        .iter()
        .enumerate()
        .map(|(i, e)| ((e.project_id, e.iid), i))
        .collect();
    let mut changed = false;
    for item in fresh {
        let k = (item.project_id, item.iid);
        match index.get(&k) {
            Some(&i) => {
                if existing[i] != item {
                    existing[i] = item;
                    changed = true;
                }
            }
            None => {
                index.insert(k, existing.len());
                existing.push(item);
                changed = true;
            }
        }
    }
    if changed {
        existing.sort_by(mr_cmp);
    }
    changed
}

impl App {
    /// Applies one merge request list result and reports whether the list changed.
    /// `replace` swaps the rows of every project that answered; rows of `failed`
    /// projects stay. Without `replace` rows are only upserted, never removed, and
    /// an identical row is not touched.
    pub(super) fn merge_mr_rows(
        &mut self,
        rows: Vec<MergeRequest>,
        failed: &[u64],
        truncated: &[u64],
        replace: bool,
    ) -> bool {
        if !replace {
            return upsert_changed(&mut self.merge_requests, rows);
        }
        let (answered, failed_rows): (Vec<_>, Vec<_>) = rows
            .into_iter()
            .partition(|m| !failed.contains(&m.project_id));
        let mut oldest: HashMap<u64, &MergeRequest> = HashMap::new();
        for m in answered.iter().filter(|m| truncated.contains(&m.project_id)) {
            let slot = oldest.entry(m.project_id).or_insert(m);
            if mr_cmp(m, slot) == Ordering::Greater {
                *slot = m;
            }
        }
        let fresh_keys: HashSet<Key> = answered.iter().map(|m| (m.project_id, m.iid)).collect();
        let older: Vec<MergeRequest> = self
            .merge_requests
            .iter()
            .filter(|m| {
                oldest
                    .get(&m.project_id)
                    .is_some_and(|last| mr_cmp(m, last) == Ordering::Greater)
                    && !fresh_keys.contains(&(m.project_id, m.iid))
            })
            .cloned()
            .collect();
        let mut next = answered;
        next.extend(older);
        next.extend(
            self.merge_requests
                .iter()
                .filter(|m| failed.contains(&m.project_id))
                .cloned(),
        );
        upsert_by_key(&mut next, failed_rows, |m| (m.project_id, m.iid));
        Self::sort_mrs(&mut next);
        if next == self.merge_requests {
            return false;
        }
        self.merge_requests = next;
        true
    }
}

impl App {
    pub(super) fn remove_mr_keys(&mut self, keys: &[Key]) -> bool {
        let before = self.merge_requests.len();
        self.merge_requests
            .retain(|m| !keys.contains(&(m.project_id, m.iid)));
        self.merge_requests.len() != before
    }
}
