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
        self.selected_label_ids = current
            .iter()
            .map(|l| l.id())
            .filter(|id| *id != 0)
            .collect();
        self.selected_label_names = current.iter().map(|l| l.name().to_string()).collect();
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
        if label.id != 0 {
            if self.selected_label_ids.contains(&label.id) {
                self.selected_label_ids.retain(|&x| x != label.id);
            } else {
                self.selected_label_ids.push(label.id);
            }
        }
        if self.selected_label_names.iter().any(|n| n == &label.name) {
            self.selected_label_names.retain(|n| n != &label.name);
        } else {
            self.selected_label_names.push(label.name.clone());
        }
    }

    /// Labels are changed by name, and only those the popup can show are touched:
    /// labels on the item that are missing from the popup list are preserved.
    pub fn label_change(&self, current: &[Label]) -> LabelChange {
        let mut add = Vec::new();
        let mut remove = Vec::new();
        let mut resulting: Vec<Label> = current.to_vec();
        for l in self.popup_labels() {
            let checked = self.label_checked(l);
            let present = current.iter().any(|c| c.name == l.name);
            if checked && !present {
                add.push(l.name.clone());
                resulting.push(l.clone());
            } else if !checked && present {
                remove.push(l.name.clone());
                resulting.retain(|c| c.name != l.name);
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

    pub fn snapshot_mr_edit(&mut self, project_id: u64, iid: u64) {
        let row = self
            .merge_requests
            .iter()
            .find(|m| m.project_id == project_id && m.iid == iid)
            .cloned();
        let snap = row.or_else(|| {
            self.selected_mr
                .clone()
                .filter(|m| m.project_id == project_id && m.iid == iid)
        });
        if let Some(snap) = snap {
            self.mr_edit_snapshots.insert((project_id, iid), snap);
        }
    }

    pub fn snapshot_issue_edit(&mut self, project_id: u64, iid: u64) {
        let row = self
            .issues
            .iter()
            .find(|i| i.project_id == project_id && i.iid == iid)
            .cloned();
        let snap = row.or_else(|| {
            self.selected_issue
                .clone()
                .filter(|i| i.project_id == project_id && i.iid == iid)
        });
        if let Some(snap) = snap {
            self.issue_edit_snapshots.insert((project_id, iid), snap);
        }
    }

    pub fn rollback_mr_edit(&mut self, project_id: u64, iid: u64) {
        let Some(prev) = self.mr_edit_snapshots.remove(&(project_id, iid)) else {
            return;
        };
        if let Some(sel) = &mut self.selected_mr
            && sel.project_id == project_id
            && sel.iid == iid
        {
            *sel = prev.clone();
        }
        if let Some(row) = self
            .merge_requests
            .iter_mut()
            .find(|m| m.project_id == project_id && m.iid == iid)
        {
            *row = prev;
        }
        self.filter_lists();
    }

    pub fn rollback_issue_edit(&mut self, project_id: u64, iid: u64) {
        let Some(prev) = self.issue_edit_snapshots.remove(&(project_id, iid)) else {
            return;
        };
        if let Some(sel) = &mut self.selected_issue
            && sel.project_id == project_id
            && sel.iid == iid
        {
            *sel = prev.clone();
        }
        if let Some(row) = self
            .issues
            .iter_mut()
            .find(|i| i.project_id == project_id && i.iid == iid)
        {
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
        self.project_branches.clear();
        self.selected_file_content = None;
        self.file_scroll = 0;
        self.file_highlight_cache = None;
    }
}

impl App {
    pub fn snapshot_selected_mr(&mut self) {
        if let Some((p, i)) = self.selected_mr.as_ref().map(|m| (m.project_id, m.iid)) {
            self.snapshot_mr_edit(p, i);
        }
    }

    pub fn snapshot_selected_issue(&mut self) {
        if let Some((p, i)) = self.selected_issue.as_ref().map(|m| (m.project_id, m.iid)) {
            self.snapshot_issue_edit(p, i);
        }
    }
}

/// A response confirms a pending edit when it is newer than the pre-edit copy
/// (or when timestamps are unknown and there is nothing to compare).
pub fn confirms_edit(before: &Option<String>, response: &Option<String>) -> bool {
    unknown_ts(before, response) || newer(response, before)
}
