use crate::models::*;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

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
    key: impl Fn(&T) -> Key,
    after: impl Fn(&T, &T) -> bool,
) -> bool {
    if !keep_tail {
        *existing = fresh;
        return false;
    }
    let fresh_keys: HashSet<Key> = fresh.iter().map(&key).collect();
    let tail: Vec<T> = match fresh.last() {
        Some(last) => std::mem::take(existing)
            .into_iter()
            .filter(|e| !fresh_keys.contains(&key(e)) && after(e, last))
            .collect(),
        None => Vec::new(),
    };
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
    key: impl Fn(&T) -> Key,
    after: impl Fn(&T, &T) -> bool,
) -> bool {
    let keep_tail = *page > 1;
    let old = existing.clone();
    let tail_kept = first_page_merge(existing, fresh, keep_tail, key, after);
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
    first_page_merge(&mut answered, fresh, keep_tail, &key, |a, b| {
        cmp(a, b) == Ordering::Greater
    });
    answered.extend(kept_failed);
    answered.sort_by(cmp);
    *existing = answered;
}

fn mr_cmp(a: &MergeRequest, b: &MergeRequest) -> Ordering {
    b.created_at
        .cmp(&a.created_at)
        .then(a.project_id.cmp(&b.project_id))
        .then(b.iid.cmp(&a.iid))
}

fn issue_cmp(a: &Issue, b: &Issue) -> Ordering {
    b.created_at
        .cmp(&a.created_at)
        .then(a.project_id.cmp(&b.project_id))
        .then(b.iid.cmp(&a.iid))
}

fn pipeline_after(a: &Pipeline, b: &Pipeline) -> bool {
    (&a.created_at, a.id) < (&b.created_at, b.id)
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
            .sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));
        self.cross_pipelines.truncate(50);
        self.filter_lists();
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
            |t| (t.id, 0),
            |a, b| (&a.created_at, a.id) < (&b.created_at, b.id),
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
            |m| (m.project_id, m.iid),
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
            |i| (i.project_id, i.iid),
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
            |p| (p.project_id, p.id),
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
            |p| (p.project_id, p.id),
            pipeline_after,
        );
        if changed {
            self.filter_lists();
        }
    }
}
