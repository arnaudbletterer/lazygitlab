use gitlab::api::{Endpoint, Pageable};
use gitlab::api::endpoint_prelude::Method;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct MergeRequestDiscussions { pub project: u64, pub merge_request: u64 }
impl Endpoint for MergeRequestDiscussions {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/discussions", self.project, self.merge_request).into() }
}
impl Pageable for MergeRequestDiscussions {}

#[derive(Debug, Clone)]
pub struct CurrentUser;
impl Endpoint for CurrentUser {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "user".into() }
}

#[derive(Debug, Clone)]
pub struct CreateMergeRequestDiscussion {
    pub project: u64,
    pub merge_request: u64,
    pub body: String,
    pub base_sha: Option<String>,
    pub start_sha: Option<String>,
    pub head_sha: Option<String>,
    pub new_path: Option<String>,
    pub old_path: Option<String>,
    pub new_line: Option<u64>,
    pub old_line: Option<u64>,
    pub range_start_new: Option<u64>,
    pub range_start_old: Option<u64>,
}
impl Endpoint for CreateMergeRequestDiscussion {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/discussions", self.project, self.merge_request).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("body", &self.body);
        if let Some(base) = &self.base_sha {
            params.push("position[base_sha]", base);
            params.push("position[start_sha]", self.start_sha.as_ref().unwrap());
            params.push("position[head_sha]", self.head_sha.as_ref().unwrap());
            params.push("position[position_type]", "text");
            if let Some(path) = &self.new_path { params.push("position[new_path]", path); }
            if let Some(path) = &self.old_path { params.push("position[old_path]", path); }
            
            if let Some(line) = self.new_line { params.push("position[new_line]", line); }
            if let Some(line) = self.old_line { params.push("position[old_line]", line); }

            if self.range_start_new.is_some() || self.range_start_old.is_some() {
                // Multi-line range
                if let Some(rs_new) = self.range_start_new {
                    params.push("position[line_range][start][new_line]", rs_new);
                }
                if let Some(rs_old) = self.range_start_old {
                    params.push("position[line_range][start][old_line]", rs_old);
                }
                let start_type = if self.range_start_old.is_some() && self.range_start_new.is_some() {
                    "text"
                } else if self.range_start_new.is_some() {
                    "new"
                } else {
                    "old"
                };
                params.push("position[line_range][start][type]", start_type);
                
                if let Some(nl) = self.new_line {
                    params.push("position[line_range][end][new_line]", nl);
                }
                if let Some(ol) = self.old_line {
                    params.push("position[line_range][end][old_line]", ol);
                }
                let end_type = if self.old_line.is_some() && self.new_line.is_some() {
                    "text"
                } else if self.new_line.is_some() {
                    "new"
                } else {
                    "old"
                };
                params.push("position[line_range][end][type]", end_type);
            }
        }
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct ResolveDiscussion {
    pub project: u64,
    pub merge_request: u64,
    pub discussion: String,
    pub resolved: bool,
}
impl Endpoint for ResolveDiscussion {
    fn method(&self) -> Method { Method::PUT }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/discussions/{}", self.project, self.merge_request, self.discussion).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("resolved", self.resolved);
        params
    }
}

#[derive(Debug, Clone)]
pub struct AddMergeRequestDiscussionNote {
    pub project: u64,
    pub merge_request: u64,
    pub discussion: String,
    pub body: String,
}
impl Endpoint for AddMergeRequestDiscussionNote {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/discussions/{}/notes", self.project, self.merge_request, self.discussion).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("body", &self.body);
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct AddIssueDiscussionNote {
    pub project: u64,
    pub issue: u64,
    pub discussion: String,
    pub body: String,
}
impl Endpoint for AddIssueDiscussionNote {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/issues/{}/discussions/{}/notes", self.project, self.issue, self.discussion).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("body", &self.body);
        params.into_body()
    }
}

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct EditIssue {
    pub project: u64,
    pub issue: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub state_event: Option<String>,
    pub labels: Option<String>,
    pub add_labels: Option<String>,
    pub remove_labels: Option<String>,
    pub assignee_ids: Option<Vec<u64>>,
    pub milestone_id: Option<Option<u64>>,
    pub due_date: Option<Option<String>>,
    pub weight: Option<Option<i64>>,
    pub confidential: Option<bool>,
}
impl EditIssue {
    pub fn builder() -> Self { Self::default() }
    pub fn project(&mut self, p: u64) -> &mut Self { self.project = p; self }
    pub fn issue(&mut self, i: u64) -> &mut Self { self.issue = i; self }
    pub fn title(&mut self, t: String) -> &mut Self { self.title = Some(t); self }
    pub fn description(&mut self, d: String) -> &mut Self { self.description = Some(d); self }
    pub fn build(&mut self) -> Result<Self, String> { Ok(std::mem::take(self)) }
}
impl Endpoint for EditIssue {
    fn method(&self) -> Method { Method::PUT }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/issues/{}", self.project, self.issue).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("with_labels_details", true);
        params
    }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        if let Some(t) = &self.title { params.push("title", t); }
        if let Some(d) = &self.description { params.push("description", d); }
        if let Some(s) = &self.state_event { params.push("state_event", s); }
        if let Some(l) = &self.labels {
            if l.is_empty() { params.push("labels", ""); }
            else { params.push("labels", l); }
        }
        if let Some(l) = &self.add_labels { params.push("add_labels", l); }
        if let Some(l) = &self.remove_labels { params.push("remove_labels", l); }
        if let Some(a) = &self.assignee_ids {
            if a.is_empty() { params.push("assignee_ids", "0"); }
            else { for id in a { params.push("assignee_ids[]", *id); } }
        }
        if let Some(m) = &self.milestone_id {
            match m {
                Some(id) => { params.push("milestone_id", *id); },
                None => { params.push("milestone_id", "0"); }
            }
        }
        if let Some(dd) = &self.due_date {
            match dd { Some(val) => { params.push("due_date", val); }, None => { params.push("due_date", ""); } }
        }
        if let Some(w) = &self.weight {
            match w { Some(val) => { params.push("weight", *val as u64); }, None => { params.push("weight", ""); } }
        }
        if let Some(c) = &self.confidential { params.push("confidential", *c); }
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct SingleIssue { pub project: u64, pub issue: u64 }
impl Endpoint for SingleIssue {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/issues/{}", self.project, self.issue).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("with_labels_details", true);
        params
    }
}

#[derive(Debug, Clone)]
pub struct IssueDiscussions { pub project: u64, pub issue: u64 }
impl Endpoint for IssueDiscussions {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/issues/{}/discussions", self.project, self.issue).into() }
}
impl Pageable for IssueDiscussions {}

#[derive(Debug, Clone)]
pub struct AddIssueNote { pub project: u64, pub issue: u64, pub body: String }
impl Endpoint for AddIssueNote {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/issues/{}/notes", self.project, self.issue).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("body", &self.body);
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct ResolveIssueDiscussion { pub project: u64, pub issue: u64, pub discussion: String, pub resolved: bool }
impl Endpoint for ResolveIssueDiscussion {
    fn method(&self) -> Method { Method::PUT }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/issues/{}/discussions/{}", self.project, self.issue, self.discussion).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("resolved", self.resolved);
        params
    }
}

#[derive(Debug, Clone)]
pub struct EditMergeRequest {
    pub project: u64,
    pub merge_request: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub state_event: Option<String>,
    pub labels: Option<String>,
    pub assignee_ids: Option<Vec<u64>>,
    pub reviewer_ids: Option<Vec<u64>>,
    pub milestone_id: Option<Option<u64>>,
    pub remove_source_branch: Option<bool>,
    pub squash: Option<bool>,
    pub target_branch: Option<String>,
    pub draft: Option<bool>,
}
impl Endpoint for EditMergeRequest {
    fn method(&self) -> Method { Method::PUT }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}", self.project, self.merge_request).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("with_labels_details", true);
        params
    }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        if let Some(t) = &self.title { params.push("title", t); }
        if let Some(d) = &self.description { params.push("description", d); }
        if let Some(s) = &self.state_event { params.push("state_event", s); }
        if let Some(l) = &self.labels { params.push("labels", l); }
        if let Some(a) = &self.assignee_ids {
            if a.is_empty() { params.push("assignee_ids[]", 0); }
            else { for id in a { params.push("assignee_ids[]", *id); } }
        }
        if let Some(r) = &self.reviewer_ids {
            if r.is_empty() { params.push("reviewer_ids[]", 0); }
            else { for id in r { params.push("reviewer_ids[]", *id); } }
        }
        if let Some(m) = &self.milestone_id {
            match m { Some(id) => { params.push("milestone_id", *id); }, None => { params.push("milestone_id", 0); } }
        }
        if let Some(r) = &self.remove_source_branch { params.push("remove_source_branch", *r); }
        if let Some(s) = &self.squash { params.push("squash", *s); }
        if let Some(t) = &self.target_branch { params.push("target_branch", t); }
        if let Some(d) = &self.draft { params.push("draft", *d); }
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct MergeRequestAwardEmoji { pub project: u64, pub merge_request: u64 }
impl Endpoint for MergeRequestAwardEmoji {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/award_emoji", self.project, self.merge_request).into() }
}
impl Pageable for MergeRequestAwardEmoji {}

#[derive(Debug, Clone)]
pub struct IssueAwardEmoji { pub project: u64, pub issue: u64 }
impl Endpoint for IssueAwardEmoji {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/issues/{}/award_emoji", self.project, self.issue).into() }
}
impl Pageable for IssueAwardEmoji {}

#[derive(Debug, Clone)]
pub struct ProjectLabels { pub project: u64 }
impl Endpoint for ProjectLabels {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/labels", self.project).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("include_ancestor_groups", true);
        params
    }
}
impl Pageable for ProjectLabels {}

#[derive(Debug, Clone)]
pub struct ProjectMilestones { pub project: u64 }
impl Endpoint for ProjectMilestones {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/milestones", self.project).into() }
}
impl Pageable for ProjectMilestones {}

#[derive(Debug, Clone)]
pub struct ProjectMembers { pub project: u64 }
impl Endpoint for ProjectMembers {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/members/all", self.project).into() }
}
impl Pageable for ProjectMembers {}

#[derive(Debug, Clone)]
pub struct Groups { pub page: Option<u64>, pub per_page: Option<u64>, pub min_access_level: Option<u64>, pub top_level_only: Option<bool> }
impl Endpoint for Groups {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "groups".into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(p) = self.page { params.push("page", p); }
        if let Some(pp) = self.per_page { params.push("per_page", pp); }
        if let Some(m) = self.min_access_level { params.push("min_access_level", m); }
        if let Some(t) = self.top_level_only { params.push("top_level_only", t); }
        params
    }
}
impl Pageable for Groups {}

#[derive(Debug, Clone)]
pub struct ProjectPipelines { pub project: u64, pub page: Option<u64>, pub per_page: Option<u64> }
impl Endpoint for ProjectPipelines {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipelines", self.project).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(p) = self.page { params.push("page", p); }
        if let Some(pp) = self.per_page { params.push("per_page", pp); }
        params
    }
}
impl Pageable for ProjectPipelines {}

#[derive(Debug, Clone)]
pub struct PagedIssues { pub project: u64, pub page: u64, pub per_page: u64, pub state: Option<String>, pub order_by: Option<String>, pub sort: Option<String>, pub with_labels_details: Option<bool> }
impl Endpoint for PagedIssues {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/issues", self.project).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("page", self.page);
        params.push("per_page", self.per_page);
        if let Some(s) = &self.state { params.push("state", s); }
        if let Some(o) = &self.order_by { params.push("order_by", o); }
        if let Some(s) = &self.sort { params.push("sort", s); }
        if let Some(w) = self.with_labels_details { params.push("with_labels_details", w); }
        else { params.push("with_labels_details", true); }
        params
    }
}
impl Pageable for PagedIssues {}

#[derive(Debug, Clone)]
pub struct PagedMergeRequests { pub project: u64, pub page: u64, pub per_page: u64, pub state: Option<String>, pub order_by: Option<String>, pub sort: Option<String>, pub with_labels_details: Option<bool> }
impl Endpoint for PagedMergeRequests {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests", self.project).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("page", self.page);
        params.push("per_page", self.per_page);
        if let Some(s) = &self.state { params.push("state", s); }
        if let Some(o) = &self.order_by { params.push("order_by", o); }
        if let Some(s) = &self.sort { params.push("sort", s); }
        if let Some(w) = self.with_labels_details { params.push("with_labels_details", w); }
        else { params.push("with_labels_details", true); }
        params
    }
}
impl Pageable for PagedMergeRequests {}

#[derive(Debug, Clone)]
pub struct PagedProjects { 
    pub membership: bool, 
    pub search: Option<String>,
    pub page: u64, 
    pub per_page: u64,
    pub simple: Option<bool>,
    pub archived: Option<bool>,
    pub order_by: Option<String>,
}
impl Endpoint for PagedProjects {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "projects".into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("membership", self.membership);
        if let Some(s) = &self.search { params.push("search", s); }
        params.push("page", self.page);
        params.push("per_page", self.per_page);
        if let Some(s) = self.simple { params.push("simple", s); }
        if let Some(a) = self.archived { params.push("archived", a); }
        if let Some(o) = &self.order_by { params.push("order_by", o); }
        params
    }
}
impl Pageable for PagedProjects {}

#[derive(Debug, Clone)]
pub struct AddNote { pub project: u64, pub merge_request: u64, pub body: String }
impl Endpoint for AddNote {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/notes", self.project, self.merge_request).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("body", &self.body);
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct GlobalIssues { 
    pub scope: Option<String>, 
    pub state: Option<String>, 
    pub membership: Option<bool>, 
    pub search: Option<String>,
    pub author_id: Option<u64>,
    pub assignee_id: Option<u64>,
    pub page: Option<u64>, 
    pub per_page: Option<u64>, 
    pub order_by: Option<String>, 
    pub sort: Option<String>,
    pub with_labels_details: Option<bool>,
}
impl Endpoint for GlobalIssues {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "issues".into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(s) = &self.scope { params.push("scope", s); }
        if let Some(s) = &self.state { params.push("state", s); }
        if let Some(m) = self.membership { params.push("membership", m); }
        if let Some(s) = &self.search { params.push("search", s); }
        if let Some(id) = self.author_id { params.push("author_id", id); }
        if let Some(id) = self.assignee_id { params.push("assignee_id", id); }
        if let Some(p) = self.page { params.push("page", p); }
        if let Some(pp) = self.per_page { params.push("per_page", pp); }
        if let Some(o) = &self.order_by { params.push("order_by", o); }
        if let Some(s) = &self.sort { params.push("sort", s); }
        if let Some(w) = self.with_labels_details { params.push("with_labels_details", w); }
        params
    }
}
impl Pageable for GlobalIssues {}

#[derive(Debug, Clone)]
pub struct GlobalMergeRequests { 
    pub scope: Option<String>, 
    pub state: Option<String>, 
    pub membership: Option<bool>, 
    pub search: Option<String>,
    pub author_id: Option<u64>,
    pub assignee_id: Option<u64>,
    pub reviewer_id: Option<u64>,
    pub page: Option<u64>, 
    pub per_page: Option<u64>, 
    pub order_by: Option<String>, 
    pub sort: Option<String>,
    pub with_labels_details: Option<bool>,
}
impl Endpoint for GlobalMergeRequests {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "merge_requests".into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(s) = &self.scope { params.push("scope", s); }
        if let Some(s) = &self.state { params.push("state", s); }
        if let Some(m) = self.membership { params.push("membership", m); }
        if let Some(s) = &self.search { params.push("search", s); }
        if let Some(id) = self.author_id { params.push("author_id", id); }
        if let Some(id) = self.assignee_id { params.push("assignee_id", id); }
        if let Some(id) = self.reviewer_id { params.push("reviewer_id", id); }
        if let Some(p) = self.page { params.push("page", p); }
        if let Some(pp) = self.per_page { params.push("per_page", pp); }
        if let Some(o) = &self.order_by { params.push("order_by", o); }
        if let Some(s) = &self.sort { params.push("sort", s); }
        if let Some(w) = self.with_labels_details { params.push("with_labels_details", w); }
        params
    }
}
impl Pageable for GlobalMergeRequests {}

#[derive(Debug, Clone)]
pub struct Todos { pub page: Option<u64>, pub per_page: Option<u64>, pub state: Option<String> }
impl Endpoint for Todos {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "todos".into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(p) = self.page { params.push("page", p); }
        if let Some(pp) = self.per_page { params.push("per_page", pp); }
        if let Some(s) = &self.state { params.push("state", s); }
        params
    }
}
impl Pageable for Todos {}

#[derive(Debug, Clone)]
pub struct MarkTodoAsDone { pub todo_id: u64 }
impl Endpoint for MarkTodoAsDone {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("todos/{}/mark_as_done", self.todo_id).into() }
}

#[derive(Debug, Clone)]
pub struct SnoozeTodo { pub todo_id: u64, pub snooze_until: String }
impl Endpoint for SnoozeTodo {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("todos/{}/snooze", self.todo_id).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("snooze_until", &self.snooze_until);
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct MergeRequestDiffs { pub project: u64, pub merge_request: u64 }
impl Endpoint for MergeRequestDiffs {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/diffs", self.project, self.merge_request).into() }
}
impl Pageable for MergeRequestDiffs {}

#[derive(Debug, Clone)]
pub struct MergeRequestNotes { pub project: u64, pub merge_request: u64 }
impl Endpoint for MergeRequestNotes {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/notes", self.project, self.merge_request).into() }
}
impl Pageable for MergeRequestNotes {}

#[derive(Debug, Clone)]
pub struct PipelineVariables { pub project: u64, pub pipeline: u64 }
impl Endpoint for PipelineVariables {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipelines/{}/variables", self.project, self.pipeline).into() }
}

#[derive(Debug, Clone)]
pub struct SingleMergeRequest { pub project: u64, pub merge_request: u64 }
impl Endpoint for SingleMergeRequest {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}", self.project, self.merge_request).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("with_labels_details", true);
        params
    }
}

#[derive(Debug, Clone)]
pub struct RebaseMergeRequest { pub project: u64, pub merge_request: u64 }
impl Endpoint for RebaseMergeRequest {
    fn method(&self) -> Method { Method::PUT }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/rebase", self.project, self.merge_request).into() }
}

#[derive(Debug, Clone)]
pub struct ApproveMergeRequest { pub project: u64, pub merge_request: u64 }
impl Endpoint for ApproveMergeRequest {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/approve", self.project, self.merge_request).into() }
}

#[derive(Debug, Clone)]
pub struct UnapproveMergeRequest { pub project: u64, pub merge_request: u64 }
impl Endpoint for UnapproveMergeRequest {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/unapprove", self.project, self.merge_request).into() }
}

#[derive(Debug, Clone)]
pub struct AcceptMergeRequest { pub project: u64, pub merge_request: u64, pub squash: bool, pub should_remove_source_branch: bool }
impl Endpoint for AcceptMergeRequest {
    fn method(&self) -> Method { Method::PUT }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/merge", self.project, self.merge_request).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("squash", self.squash);
        params.push("should_remove_source_branch", self.should_remove_source_branch);
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct CreateIssue { pub project: u64, pub title: String, pub description: Option<String>, pub assignee_ids: Vec<u64>, pub label_names: Vec<String> }
impl Endpoint for CreateIssue {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/issues", self.project).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("with_labels_details", true);
        params
    }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("title", &self.title);
        if let Some(d) = &self.description { params.push("description", d); }
        for id in &self.assignee_ids { params.push("assignee_ids[]", *id); }
        if !self.label_names.is_empty() { params.push("labels", self.label_names.join(",")); }
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct CreateMergeRequest { pub project: u64, pub source_branch: String, pub target_branch: String, pub title: String, pub description: Option<String> }
impl Endpoint for CreateMergeRequest {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests", self.project).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("with_labels_details", true);
        params
    }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("source_branch", &self.source_branch);
        params.push("target_branch", &self.target_branch);
        params.push("title", &self.title);
        if let Some(d) = &self.description { params.push("description", d); }
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct ProjectBranches { pub project: u64 }
impl Endpoint for ProjectBranches {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/repository/branches", self.project).into() }
}
impl Pageable for ProjectBranches {}

#[derive(Debug, Clone)]
pub struct ProjectPipeline { pub project: u64, pub pipeline: u64 }
impl Endpoint for ProjectPipeline {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipelines/{}", self.project, self.pipeline).into() }
}

#[derive(Debug, Clone)]
pub struct ProjectPipelineJobs { pub project: u64, pub pipeline: u64 }
impl Endpoint for ProjectPipelineJobs {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipelines/{}/jobs", self.project, self.pipeline).into() }
}
impl Pageable for ProjectPipelineJobs {}

#[derive(Debug, Clone)]
pub struct SearchProjects { pub query: String }
impl Endpoint for SearchProjects {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "projects".into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("search", &self.query);
        params
    }
}

#[derive(Debug, Clone)]
pub struct GroupProjects { pub group: u64, pub page: Option<u64>, pub per_page: Option<u64> }
impl Endpoint for GroupProjects {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("groups/{}/projects", self.group).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(p) = self.page { params.push("page", p); }
        if let Some(pp) = self.per_page { params.push("per_page", pp); }
        params
    }
}

#[derive(Debug, Clone)]
pub struct CreatePipeline { pub project: u64, pub ref_name: String, pub variables: Option<Vec<crate::models::PipelineVariable>> }
impl Endpoint for CreatePipeline {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipeline", self.project).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("ref", &self.ref_name);
        if let Some(vars) = &self.variables {
            for (i, var) in vars.iter().enumerate() {
                params.push(format!("variables[{}][key]", i), &var.key);
                params.push(format!("variables[{}][value]", i), &var.value);
            }
        }
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct RetryPipeline { pub project: u64, pub pipeline: u64 }
impl Endpoint for RetryPipeline {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipelines/{}/retry", self.project, self.pipeline).into() }
}

#[derive(Debug, Clone)]
pub struct CancelPipeline { pub project: u64, pub pipeline: u64 }
impl Endpoint for CancelPipeline {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipelines/{}/cancel", self.project, self.pipeline).into() }
}

#[derive(Debug, Clone)]
pub struct RetryJob { pub project: u64, pub job: u64 }
impl Endpoint for RetryJob {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/jobs/{}/retry", self.project, self.job).into() }
}

#[derive(Debug, Clone)]
pub struct CancelJob { pub project: u64, pub job: u64 }
impl Endpoint for CancelJob {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/jobs/{}/cancel", self.project, self.job).into() }
}

#[derive(Debug, Clone)]
pub struct PlayJob { pub project: u64, pub job: u64 }
impl Endpoint for PlayJob {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/jobs/{}/play", self.project, self.job).into() }
}

#[derive(Debug, Clone)]
pub struct CreateGroup { pub name: String, pub path: String, pub parent_id: Option<u64> }
impl Endpoint for CreateGroup {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { "groups".into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("name", &self.name);
        params.push("path", &self.path);
        if let Some(id) = self.parent_id { params.push("parent_id", id); }
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct CreateMilestone {
    pub project: u64,
    pub title: String,
}
impl Endpoint for CreateMilestone {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/milestones", self.project).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("title", &self.title);
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct CreateLabel {
    pub project: u64,
    pub name: String,
    pub color: String,
}
impl Endpoint for CreateLabel {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/labels", self.project).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("name", &self.name);
        params.push("color", &self.color);
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct CreateFile {
    pub project: u64,
    pub file_path: String,
    pub branch: String,
    pub content: String,
    pub commit_message: String,
}
impl Endpoint for CreateFile {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/repository/files/{}", self.project, self.file_path.replace("/", "%2F")).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("branch", &self.branch);
        params.push("content", &self.content);
        params.push("commit_message", &self.commit_message);
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct CreateBranch {
    pub project: u64,
    pub branch: String,
    pub ref_name: String,
}
impl Endpoint for CreateBranch {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/repository/branches", self.project).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("branch", &self.branch);
        params.push("ref", &self.ref_name);
        params
    }
}
#[derive(Debug, Clone)]
pub struct CreateProject {
    pub name: String,
    pub path: String,
    pub namespace_id: Option<u64>,
}
impl Endpoint for CreateProject {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { "projects".into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("name", &self.name);
        params.push("path", &self.path);
        if let Some(id) = self.namespace_id { params.push("namespace_id", id); }
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct JobLog { pub project: u64, pub job: u64 }
impl Endpoint for JobLog {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/jobs/{}/trace", self.project, self.job).into() }
}

#[derive(Debug, Clone)]
pub struct JobTrace { pub project: u64, pub job: u64 }
impl Endpoint for JobTrace {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/jobs/{}/trace", self.project, self.job).into() }
}

#[derive(Debug, Clone)]
pub struct Branches { pub project: u64 }
impl Endpoint for Branches {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/repository/branches", self.project).into() }
}
impl Pageable for Branches {}

#[derive(Debug, Clone)]
pub struct Commits { pub project: u64, pub ref_name: Option<String> }
impl Endpoint for Commits {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/repository/commits", self.project).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(r) = &self.ref_name { params.push("ref_name", r); }
        params
    }
}
impl Pageable for Commits {}

#[derive(Debug, Clone)]
pub struct RepositoryTree { pub project: u64, pub path: Option<String>, pub ref_name: Option<String>, pub recursive: Option<bool> }
impl Endpoint for RepositoryTree {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/repository/tree", self.project).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(p) = &self.path { params.push("path", p); }
        if let Some(r) = &self.ref_name { params.push("ref", r); }
        if let Some(r) = self.recursive { params.push("recursive", r); }
        params
    }
}
impl Pageable for RepositoryTree {}

#[derive(Debug, Clone)]
pub struct RawFile { pub project: u64, pub file_path: String, pub ref_name: Option<String> }
impl Endpoint for RawFile {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/repository/files/{}/raw", self.project, self.file_path.replace("/", "%2F")).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(r) = &self.ref_name { params.push("ref", r); }
        params
    }
}

#[derive(Debug, Clone)]
pub struct GlobalSearch { 
    pub scope: String, 
    pub search: String,
    pub per_page: Option<u64>,
    pub page: Option<u64>,
}
impl Endpoint for GlobalSearch {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "search".into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("scope", &self.scope);
        params.push("search", &self.search);
        if let Some(pp) = self.per_page { params.push("per_page", pp); }
        if let Some(p) = self.page { params.push("page", p); }
        params
    }
}
impl Pageable for GlobalSearch {}

#[derive(Debug, Clone)]
pub struct ProjectBoards { pub project: u64 }
impl Endpoint for ProjectBoards {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/boards", self.project).into() }
}
impl Pageable for ProjectBoards {}

#[derive(Debug, Clone)]
pub struct ProjectBoardLists { pub project: u64, pub board: u64 }
impl Endpoint for ProjectBoardLists {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/boards/{}/lists", self.project, self.board).into() }
}
impl Pageable for ProjectBoardLists {}

#[derive(Debug, Clone)]
pub struct ProjectBoardDetails { pub project: u64, pub board: u64 }
impl Endpoint for ProjectBoardDetails {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/boards/{}", self.project, self.board).into() }
}

#[derive(Debug, Clone)]
pub struct ProjectBoardListIssues { pub project: u64, pub board: u64, pub list: u64 }
impl Endpoint for ProjectBoardListIssues {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/boards/{}/lists/{}/issues", self.project, self.board, self.list).into() }
}
impl Pageable for ProjectBoardListIssues {}

#[derive(Debug, Clone)]
pub struct ProjectVariables { pub project: u64 }
impl Endpoint for ProjectVariables {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/variables", self.project).into() }
}

#[derive(Debug, Clone)]
pub struct GroupEpics { pub group: u64 }
impl Endpoint for GroupEpics {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("groups/{}/epics", self.group).into() }
}
impl Pageable for GroupEpics {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_endpoints_exhaustive() {
        let mr_disc = MergeRequestDiscussions { project: 1, merge_request: 2 };
        assert_eq!(mr_disc.method(), Method::GET);
        assert_eq!(mr_disc.endpoint(), "projects/1/merge_requests/2/discussions");

        let curr_user = CurrentUser;
        assert_eq!(curr_user.method(), Method::GET);
        assert_eq!(curr_user.endpoint(), "user");

        let add_disc = CreateMergeRequestDiscussion { project: 1, merge_request: 2, body: "b".into(), base_sha: None, start_sha: None, head_sha: None, new_path: None, old_path: None, new_line: None, old_line: None, range_start_new: None, range_start_old: None };
        assert_eq!(add_disc.method(), Method::POST);
        assert_eq!(add_disc.endpoint(), "projects/1/merge_requests/2/discussions");
        assert!(add_disc.body().is_ok());

        let res_disc = ResolveDiscussion { project: 1, merge_request: 2, discussion: "d".into(), resolved: true };
        assert_eq!(res_disc.method(), Method::PUT);
        assert_eq!(res_disc.endpoint(), "projects/1/merge_requests/2/discussions/d");
        let _ = res_disc.parameters();

        let add_issue_disc = AddIssueDiscussionNote { project: 1, issue: 2, discussion: "d".into(), body: "b".into() };
        assert_eq!(add_issue_disc.method(), Method::POST);
        assert_eq!(add_issue_disc.endpoint(), "projects/1/issues/2/discussions/d/notes");
        assert!(add_issue_disc.body().is_ok());

        let edit_issue = EditIssue {
            project: 1, issue: 2, title: Some("t".into()), description: Some("d".into()),
            state_event: Some("close".into()), labels: Some("l".into()),
            add_labels: None, remove_labels: None,
            assignee_ids: Some(vec![1]), milestone_id: Some(Some(1)),
            due_date: Some(Some("2021-01-01".into())), weight: Some(Some(1)), confidential: Some(true)
        };
        assert_eq!(edit_issue.method(), Method::PUT);
        assert!(edit_issue.body().is_ok());

        let edit_issue_nulls = EditIssue {
            project: 1, issue: 2, title: None, description: None,
            state_event: None, labels: None, add_labels: None, remove_labels: None,
            assignee_ids: Some(vec![]),
            milestone_id: Some(None), due_date: Some(None), weight: Some(None), confidential: None
        };
        assert!(edit_issue_nulls.body().is_ok());

        let single_issue = SingleIssue { project: 1, issue: 2 };
        assert_eq!(single_issue.method(), Method::GET);
        let _ = single_issue.parameters();

        let issue_disc = IssueDiscussions { project: 1, issue: 2 };
        assert_eq!(issue_disc.method(), Method::GET);

        let add_issue_note = AddIssueNote { project: 1, issue: 2, body: "b".into() };
        assert_eq!(add_issue_note.method(), Method::POST);
        assert!(add_issue_note.body().is_ok());

        let res_issue_disc = ResolveIssueDiscussion { project: 1, issue: 2, discussion: "d".into(), resolved: true };
        assert_eq!(res_issue_disc.method(), Method::PUT);
        let _ = res_issue_disc.parameters();

        let edit_mr = EditMergeRequest {
            project: 1, merge_request: 2, title: Some("t".into()), description: Some("d".into()),
            state_event: Some("close".into()), labels: Some("l".into()),
            assignee_ids: Some(vec![1]), reviewer_ids: Some(vec![2]),
            milestone_id: Some(Some(1)), remove_source_branch: Some(true),
            squash: Some(true), target_branch: Some("b".into()),
            draft: Some(true),
        };
        assert_eq!(edit_mr.method(), Method::PUT);
        assert!(edit_mr.body().is_ok());

        let edit_mr_nulls = EditMergeRequest {
            project: 1, merge_request: 2, title: None, description: None,
            state_event: None, labels: None, assignee_ids: Some(vec![]),
            reviewer_ids: Some(vec![]), milestone_id: Some(None),
            remove_source_branch: None, squash: None, target_branch: None,
            draft: None,
        };
        assert!(edit_mr_nulls.body().is_ok());

        let mr_notes = MergeRequestNotes { project: 1, merge_request: 2 };
        assert_eq!(mr_notes.method(), Method::GET);

        let mr_diffs = MergeRequestDiffs { project: 1, merge_request: 2 };
        assert_eq!(mr_diffs.method(), Method::GET);

        let p_pipelines = ProjectPipelines { project: 1, page: Some(1), per_page: Some(1) };
        assert_eq!(p_pipelines.method(), Method::GET);
        let _ = p_pipelines.parameters();

        let p_jobs = ProjectPipelineJobs { project: 1, pipeline: 2 };
        assert_eq!(p_jobs.method(), Method::GET);

        let job_trace = JobTrace { project: 1, job: 2 };
        assert_eq!(job_trace.method(), Method::GET);

        let pipe_vars = PipelineVariables { project: 1, pipeline: 2 };
        assert_eq!(pipe_vars.method(), Method::GET);

        let search_projs = SearchProjects { query: "q".into() };
        assert_eq!(search_projs.method(), Method::GET);
        let _ = search_projs.parameters();

        let p_projs = PagedProjects { membership: true, search: None, page: 1, per_page: 1, simple: Some(true), archived: Some(false), order_by: None };
        assert_eq!(p_projs.method(), Method::GET);
        let _ = p_projs.parameters();

        let groups = Groups { page: Some(1), per_page: Some(1), min_access_level: Some(10), top_level_only: None };
        assert_eq!(groups.method(), Method::GET);
        let _ = groups.parameters();

        let group_projs = GroupProjects { group: 1, page: Some(1), per_page: Some(1) };
        assert_eq!(group_projs.method(), Method::GET);
        let _ = group_projs.parameters();

        let p_mrs = PagedMergeRequests { project: 1, page: 1, per_page: 1, state: Some("opened".into()), order_by: None, sort: None, with_labels_details: None };
        assert_eq!(p_mrs.method(), Method::GET);
        let _ = p_mrs.parameters();

        let p_issues = PagedIssues { project: 1, page: 1, per_page: 1, state: Some("opened".into()), order_by: None, sort: None, with_labels_details: None };
        assert_eq!(p_issues.method(), Method::GET);
        let _ = p_issues.parameters();

        let g_mrs = GlobalMergeRequests { scope: Some("a".into()), state: Some("a".into()), membership: Some(true), search: None, author_id: None, assignee_id: None, reviewer_id: None, page: Some(1), per_page: Some(1), order_by: None, sort: None, with_labels_details: None };
        assert_eq!(g_mrs.method(), Method::GET);
        let _ = g_mrs.parameters();

        let g_issues = GlobalIssues { scope: Some("a".into()), state: Some("a".into()), membership: Some(true), search: None, author_id: None, assignee_id: None, page: Some(1), per_page: Some(1), order_by: None, sort: None, with_labels_details: None };
        assert_eq!(g_issues.method(), Method::GET);
        let _ = g_issues.parameters();

        let todos = Todos { page: Some(1), per_page: Some(1), state: Some("pending".into()) };
        assert_eq!(todos.method(), Method::GET);
        let _ = todos.parameters();

        let mark_done = MarkTodoAsDone { todo_id: 1 };
        assert_eq!(mark_done.method(), Method::POST);

        let snooze = SnoozeTodo { todo_id: 1, snooze_until: "2026-05-03T12:00:00Z".into() };
        assert_eq!(snooze.method(), Method::POST);
        assert!(snooze.body().is_ok());

        let create_group = CreateGroup { name: "n".into(), path: "p".into(), parent_id: Some(1) };
        assert!(create_group.body().is_ok());

        let create_ms = CreateMilestone { project: 1, title: "t".into() };
        assert!(create_ms.body().is_ok());

        let create_lbl = CreateLabel { project: 1, name: "n".into(), color: "c".into() };
        assert!(create_lbl.body().is_ok());

        let create_file = CreateFile { project: 1, file_path: "f".into(), branch: "b".into(), content: "c".into(), commit_message: "m".into() };
        assert!(create_file.body().is_ok());

        let create_br = CreateBranch { project: 1, branch: "b".into(), ref_name: "r".into() };
        assert_eq!(create_br.method(), Method::POST);
        let _ = create_br.parameters();

        let create_proj = CreateProject { name: "n".into(), path: "p".into(), namespace_id: Some(1) };
        assert!(create_proj.body().is_ok());

        let create_pipe = CreatePipeline { project: 1, ref_name: "b".into(), variables: Some(vec![crate::models::PipelineVariable{key:"k".into(), value:"v".into()}]) };
        assert_eq!(create_pipe.method(), Method::POST);
        assert!(create_pipe.body().is_ok());

        let add_note = AddNote { project: 1, merge_request: 1, body: "b".into() };
        assert_eq!(add_note.method(), Method::POST);
        assert!(add_note.body().is_ok());

        let approve_mr = ApproveMergeRequest { project: 1, merge_request: 1 };
        assert_eq!(approve_mr.method(), Method::POST);

        let unapprove_mr = UnapproveMergeRequest { project: 1, merge_request: 1 };
        assert_eq!(unapprove_mr.method(), Method::POST);

        let rebase_mr = RebaseMergeRequest { project: 1, merge_request: 1 };
        assert_eq!(rebase_mr.method(), Method::PUT);

        let accept_mr = AcceptMergeRequest { project: 1, merge_request: 1, squash: true, should_remove_source_branch: true };
        assert_eq!(accept_mr.method(), Method::PUT);
        assert!(accept_mr.body().is_ok());

        let create_mr = CreateMergeRequest { project: 1, source_branch: "s".into(), target_branch: "t".into(), title: "t".into(), description: Some("d".into()) };
        assert_eq!(create_mr.method(), Method::POST);
        assert!(create_mr.body().is_ok());

        let create_issue = CreateIssue { project: 1, title: "t".into(), description: Some("d".into()), assignee_ids: vec![1], label_names: vec!["l".into()] };
        assert_eq!(create_issue.method(), Method::POST);
        assert!(create_issue.body().is_ok());

        let mr_award_emoji = MergeRequestAwardEmoji { project: 1, merge_request: 1 };
        assert_eq!(mr_award_emoji.method(), Method::GET);

        let issue_award_emoji = IssueAwardEmoji { project: 1, issue: 1 };
        assert_eq!(issue_award_emoji.method(), Method::GET);
        
        let branch = Branches { project: 1 };
        assert_eq!(branch.method(), Method::GET);

        let job_log = JobLog { project: 1, job: 1 };
        assert_eq!(job_log.method(), Method::GET);

        let play_job = PlayJob { project: 1, job: 1 };
        assert_eq!(play_job.method(), Method::POST);

        let retry_job = RetryJob { project: 1, job: 1 };
        assert_eq!(retry_job.method(), Method::POST);

        let cancel_job = CancelJob { project: 1, job: 1 };
        assert_eq!(cancel_job.method(), Method::POST);

        let cancel_pipe = CancelPipeline { project: 1, pipeline: 1 };
        assert_eq!(cancel_pipe.method(), Method::POST);

        let retry_pipe = RetryPipeline { project: 1, pipeline: 1 };
        assert_eq!(retry_pipe.method(), Method::POST);

        let p_boards = ProjectBoards { project: 1 };
        assert_eq!(p_boards.method(), Method::GET);
        assert_eq!(p_boards.endpoint(), "projects/1/boards");

        let p_board_lists = ProjectBoardLists { project: 1, board: 2 };
        assert_eq!(p_board_lists.method(), Method::GET);
        assert_eq!(p_board_lists.endpoint(), "projects/1/boards/2/lists");

        let p_board_issues = ProjectBoardListIssues { project: 1, board: 2, list: 3 };
        assert_eq!(p_board_issues.method(), Method::GET);
        assert_eq!(p_board_issues.endpoint(), "projects/1/boards/2/lists/3/issues");

        let g_epics = GroupEpics { group: 1 };
        assert_eq!(g_epics.method(), Method::GET);
        assert_eq!(g_epics.endpoint(), "groups/1/epics");
    }
}
