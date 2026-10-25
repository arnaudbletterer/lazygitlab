use crate::models::*;
use std::time::Duration;

pub const PENDING_IID_BASE: u64 = 1_000_000;
pub const METADATA_TTL: Duration = Duration::from_secs(600);

pub fn is_pending_iid(iid: u64) -> bool {
    iid >= PENDING_IID_BASE
}

fn parse_ts(s: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    chrono::DateTime::parse_from_rfc3339(s).ok()
}

/// True when `a` is strictly more recent than `b`. Unknown timestamps are never newer.
pub fn newer(a: &Option<String>, b: &Option<String>) -> bool {
    match (a.as_deref(), b.as_deref()) {
        (Some(a), Some(b)) => match (parse_ts(a), parse_ts(b)) {
            (Some(x), Some(y)) => x > y,
            _ => a > b,
        },
        _ => false,
    }
}

fn unknown_ts(a: &Option<String>, b: &Option<String>) -> bool {
    a.is_none() || b.is_none()
}

/// Merge an incoming full MR (edit response, detail fetch) into what we already hold.
/// The fresher side wins for its fields; participants and reactions that the
/// incoming copy does not carry are never erased. `edit_pending` protects an
/// optimistic edit from a response that is not newer than the local copy.
pub fn merge_mr_detail(old: &MergeRequest, new: MergeRequest, edit_pending: bool) -> MergeRequest {
    let new_wins = newer(&new.updated_at, &old.updated_at)
        || (!edit_pending && !newer(&old.updated_at, &new.updated_at));
    let mut base = if new_wins { new.clone() } else { old.clone() };
    let other = if new_wins { old } else { &new };
    if base.participants.is_empty() {
        base.participants = other.participants.clone();
    }
    if base.reactions.is_empty() {
        base.reactions = other.reactions.clone();
    }
    if base.description.is_none() {
        base.description = other.description.clone();
    }
    if base.diff_refs.is_none() {
        base.diff_refs = other.diff_refs.clone();
    }
    base
}

/// Refresh the detail copy from a list row. The row only overrides the fields
/// it carries when it is newer than the detail (or when timestamps are unknown).
pub fn apply_mr_row(selected: &mut MergeRequest, row: &MergeRequest) {
    if newer(&selected.updated_at, &row.updated_at) {
        return;
    }
    if !unknown_ts(&selected.updated_at, &row.updated_at) && selected.updated_at == row.updated_at {
        return;
    }
    selected.title = row.title.clone();
    selected.state = row.state.clone();
    selected.labels = row.labels.clone();
    selected.milestone = row.milestone.clone();
    selected.assignees = row.assignees.clone();
    if row.updated_at.is_some() {
        selected.updated_at = row.updated_at.clone();
    }
}

pub fn merge_issue_detail(old: &Issue, new: Issue, edit_pending: bool) -> Issue {
    let new_wins = newer(&new.updated_at, &old.updated_at)
        || (!edit_pending && !newer(&old.updated_at, &new.updated_at));
    let (mut base, other) = if new_wins {
        (new, old.clone())
    } else {
        (old.clone(), new)
    };
    if base.participants.is_empty() {
        base.participants = other.participants;
    }
    if base.reactions.is_empty() {
        base.reactions = other.reactions;
    }
    if base.description.is_none() {
        base.description = other.description;
    }
    base
}

pub fn apply_issue_row(selected: &mut Issue, row: &Issue) {
    if newer(&selected.updated_at, &row.updated_at) {
        return;
    }
    if !unknown_ts(&selected.updated_at, &row.updated_at) && selected.updated_at == row.updated_at {
        return;
    }
    selected.title = row.title.clone();
    selected.state = row.state.clone();
    selected.labels = row.labels.clone();
    selected.milestone = row.milestone.clone();
    selected.assignees = row.assignees.clone();
    selected.weight = row.weight;
    selected.due_date = row.due_date.clone();
    if row.updated_at.is_some() {
        selected.updated_at = row.updated_at.clone();
    }
}

pub struct LabelChange {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub resulting: Vec<Label>,
}

fn label_scope(name: &str) -> Option<&str> {
    name.rsplit_once("::").map(|(scope, _)| scope)
}

/// State kept while edits of one item are in flight: the pre-edit row and detail
/// copies (the oldest, taken by the first edit), how many edits are running and
/// how many were started since the snapshot.
#[derive(Debug, Clone)]
pub struct EditState<T> {
    pub row: Option<T>,
    pub detail: Option<T>,
    pub inflight: usize,
    pub total: usize,
    pub distrust: bool,
}

/// Dropped when an edit task ends (success, failure, early return or panic).
pub struct EditGuard {
    tx: tokio::sync::mpsc::UnboundedSender<DataUpdate>,
    issue: bool,
    project_id: u64,
    iid: u64,
}

impl Drop for EditGuard {
    fn drop(&mut self) {
        let _ = self.tx.send(DataUpdate::EditSettled {
            issue: self.issue,
            project_id: self.project_id,
            iid: self.iid,
        });
    }
}

impl App {
    fn popup_project_id(&self) -> u64 {
        self.selected_mr
            .as_ref()
            .map(|m| m.project_id)
            .or_else(|| self.selected_issue.as_ref().map(|i| i.project_id))
            .unwrap_or(0)
    }

    fn popup_labels(&self) -> &[Label] {
        self.project_label_cache
            .get(&self.popup_project_id())
            .unwrap_or(&self.project_labels)
    }

    pub fn label_checked(&self, label: &Label) -> bool {
        self.selected_label_names.iter().any(|n| n == &label.name)
    }

    pub fn open_label_popup(&mut self) {
        let current: Vec<Label> = if let Some(i) = &self.selected_issue {
            i.labels.clone()
        } else if let Some(m) = &self.selected_mr {
            m.labels.clone()
        } else {
            Vec::new()
        };
        self.show_label_popup = true;
        self.selected_label_names = current.iter().map(|l| l.name().to_string()).collect();
        self.label_popup_baseline = self.selected_label_names.clone();
        self.label_state.select(Some(0));
    }

    pub fn toggle_popup_label(&mut self) {
        let Some(label) = self
            .label_state
            .selected()
            .and_then(|i| self.popup_labels().get(i))
            .cloned()
        else {
            return;
        };
        if self.selected_label_names.iter().any(|n| n == &label.name) {
            self.selected_label_names.retain(|n| n != &label.name);
        } else {
            self.selected_label_names.push(label.name.clone());
        }
    }

    /// The change is the difference between the popup selection and the labels
    /// the item had when the popup was opened, so labels changed meanwhile by
    /// someone else and labels missing from the popup list are never touched.
    /// The optimistic result applies that delta to the item's current labels.
    pub fn label_change(&self, current: &[Label]) -> LabelChange {
        let baseline = &self.label_popup_baseline;
        let mut add = Vec::new();
        let mut remove = Vec::new();
        let mut added_labels = Vec::new();
        for l in self.popup_labels() {
            let checked = self.label_checked(l);
            let was = baseline.iter().any(|n| n == &l.name);
            if checked && !was {
                add.push(l.name.clone());
                added_labels.push(l.clone());
            } else if !checked && was {
                remove.push(l.name.clone());
            }
        }
        let mut resulting: Vec<Label> = current.to_vec();
        resulting.retain(|c| !remove.contains(&c.name));
        for l in added_labels {
            if let Some(scope) = label_scope(&l.name) {
                resulting.retain(|c| label_scope(&c.name) != Some(scope));
            }
            if !resulting.iter().any(|c| c.name == l.name) {
                resulting.push(l);
            }
        }
        LabelChange {
            add,
            remove,
            resulting,
        }
    }

    pub fn metadata_fresh(&self, project_id: u64) -> bool {
        self.metadata_fetched_at
            .get(&project_id)
            .is_some_and(|t| t.elapsed() < METADATA_TTL)
    }

    pub fn begin_mr_edit(&mut self, project_id: u64, iid: u64) -> EditGuard {
        let row = self
            .merge_requests
            .iter()
            .find(|m| m.project_id == project_id && m.iid == iid)
            .cloned();
        let detail = self
            .selected_mr
            .clone()
            .filter(|m| m.project_id == project_id && m.iid == iid);
        let state = self
            .mr_edit_snapshots
            .entry((project_id, iid))
            .or_insert(EditState {
                row,
                detail,
                inflight: 0,
                total: 0,
                distrust: false,
            });
        state.inflight += 1;
        state.total += 1;
        EditGuard {
            tx: self.tx.clone(),
            issue: false,
            project_id,
            iid,
        }
    }

    pub fn begin_issue_edit(&mut self, project_id: u64, iid: u64) -> EditGuard {
        let row = self
            .issues
            .iter()
            .find(|i| i.project_id == project_id && i.iid == iid)
            .cloned();
        let detail = self
            .selected_issue
            .clone()
            .filter(|i| i.project_id == project_id && i.iid == iid);
        let state = self
            .issue_edit_snapshots
            .entry((project_id, iid))
            .or_insert(EditState {
                row,
                detail,
                inflight: 0,
                total: 0,
                distrust: false,
            });
        state.inflight += 1;
        state.total += 1;
        EditGuard {
            tx: self.tx.clone(),
            issue: true,
            project_id,
            iid,
        }
    }

    pub fn begin_selected_mr_edit(&mut self) -> Option<EditGuard> {
        let (p, i) = self.selected_mr.as_ref().map(|m| (m.project_id, m.iid))?;
        Some(self.begin_mr_edit(p, i))
    }

    pub fn begin_selected_issue_edit(&mut self) -> Option<EditGuard> {
        let (p, i) = self
            .selected_issue
            .as_ref()
            .map(|m| (m.project_id, m.iid))?;
        Some(self.begin_issue_edit(p, i))
    }

    pub fn settle_edit(&mut self, issue: bool, project_id: u64, iid: u64) {
        let key = (project_id, iid);
        if issue {
            if let Some(s) = self.issue_edit_snapshots.get_mut(&key) {
                s.inflight = s.inflight.saturating_sub(1);
                if s.inflight == 0 {
                    self.issue_edit_snapshots.remove(&key);
                }
            }
        } else if let Some(s) = self.mr_edit_snapshots.get_mut(&key) {
            s.inflight = s.inflight.saturating_sub(1);
            if s.inflight == 0 {
                self.mr_edit_snapshots.remove(&key);
            }
        }
    }

    pub fn mr_edit_pending(&self, project_id: u64, iid: u64) -> bool {
        self.mr_edit_snapshots
            .get(&(project_id, iid))
            .is_some_and(|s| !s.distrust)
    }

    pub fn issue_edit_pending(&self, project_id: u64, iid: u64) -> bool {
        self.issue_edit_snapshots
            .get(&(project_id, iid))
            .is_some_and(|s| !s.distrust)
    }

    /// A failed edit restores the pre-edit copies only when it was the sole edit
    /// of the item and certainly not applied; otherwise the server copy is
    /// refetched.
    pub async fn edit_failed(&mut self, issue: bool, project_id: u64, iid: u64, error: String) {
        let key = (project_id, iid);
        let uncertain = error.contains("timed out");
        let sole = if issue {
            self.issue_edit_snapshots.get(&key).map(|s| s.total == 1)
        } else {
            self.mr_edit_snapshots.get(&key).map(|s| s.total == 1)
        }
        .unwrap_or(false);
        let message = if uncertain {
            format!("{error}; result uncertain, refreshing")
        } else {
            error
        };
        if sole && !uncertain {
            if issue {
                self.rollback_issue_edit(project_id, iid);
            } else {
                self.rollback_mr_edit(project_id, iid);
            }
        } else {
            if issue {
                if let Some(s) = self.issue_edit_snapshots.get_mut(&key) {
                    s.distrust = true;
                }
                let _ = self.fetch_issue_detail(project_id, iid).await;
            } else {
                if let Some(s) = self.mr_edit_snapshots.get_mut(&key) {
                    s.distrust = true;
                }
                let _ = self.fetch_discussions(project_id, iid).await;
            }
            let _ = self.tx.send(DataUpdate::TriggerRefresh);
        }
        Box::pin(self.process_update_single(DataUpdate::Error(message))).await;
    }

    pub fn rollback_mr_edit(&mut self, project_id: u64, iid: u64) {
        let Some(state) = self.mr_edit_snapshots.get(&(project_id, iid)).cloned() else {
            return;
        };
        if let (Some(sel), Some(prev)) = (&mut self.selected_mr, state.detail)
            && sel.project_id == project_id
            && sel.iid == iid
        {
            *sel = prev;
        }
        if let (Some(row), Some(prev)) = (
            self.merge_requests
                .iter_mut()
                .find(|m| m.project_id == project_id && m.iid == iid),
            state.row,
        ) {
            *row = prev;
        }
        self.filter_lists();
    }

    pub fn rollback_issue_edit(&mut self, project_id: u64, iid: u64) {
        let Some(state) = self.issue_edit_snapshots.get(&(project_id, iid)).cloned() else {
            return;
        };
        if let (Some(sel), Some(prev)) = (&mut self.selected_issue, state.detail)
            && sel.project_id == project_id
            && sel.iid == iid
        {
            *sel = prev;
        }
        if let (Some(row), Some(prev)) = (
            self.issues
                .iter_mut()
                .find(|i| i.project_id == project_id && i.iid == iid),
            state.row,
        ) {
            *row = prev;
        }
        self.filter_lists();
    }

    pub fn reset_project_scope(&mut self) {
        self.project_labels.clear();
        self.project_milestones.clear();
        self.project_members.clear();
        self.project_variables.clear();
        self.pipeline_schedules.clear();
        self.schedules_state.select(None);
        self.project_boards.clear();
        self.project_boards_state.select(None);
        self.selected_board = None;
        self.board_lists.clear();
        self.board_issues.clear();
        self.filtered_board_issues.clear();
        self.board_issue_states.clear();
        self.epics.clear();
        self.epics_state.select(None);
        self.selected_epic = None;
        self.epic_issues.clear();
        self.filtered_epic_issues.clear();
        self.commits.clear();
        self.commits_state.select(None);
        self.commits_page = 1;
        self.commits_has_more = false;
        self.mr_meta.clear();
        self.mrs_sync = MrSync::default();
        self.mr_edit_snapshots.clear();
        self.issue_edit_snapshots.clear();
        self.repository_ref = None;
        self.repository_tree.clear();
        self.repository_tree_state = Default::default();
        self.project_branches.clear();
        self.filtered_branches.clear();
        self.branches_state.select(None);
        self.pending_delete_branch = None;
        self.selected_file_content = None;
        self.file_scroll = 0;
        self.file_highlight_cache = None;
    }
}
