use gitlab::api::{Endpoint, Pageable};
use gitlab::api::endpoint_prelude::Method;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct MergeRequestDiscussions { pub project: u64, pub merge_request: u64 }
impl Endpoint for MergeRequestDiscussions {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/discussions", self.project, self.merge_request).into() }
}

#[derive(Debug, Clone)]
pub struct CurrentUser;
impl Endpoint for CurrentUser {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "user".into() }
}

#[derive(Debug, Clone)]
pub struct AddDiscussionNote {
    pub project: u64,
    pub merge_request: u64,
    pub discussion: String,
    pub body: String,
}
impl Endpoint for AddDiscussionNote {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/discussions/{}/notes", self.project, self.merge_request, self.discussion).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("body", &self.body);
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
pub struct EditIssue {
    pub project: u64,
    pub issue: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub state_event: Option<String>,
    pub labels: Option<String>,
    pub assignee_ids: Option<Vec<u64>>,
    pub milestone_id: Option<Option<u64>>,
    pub due_date: Option<Option<String>>,
    pub weight: Option<Option<i64>>,
    pub confidential: Option<bool>,
}
impl Endpoint for EditIssue {
    fn method(&self) -> Method { Method::PUT }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/issues/{}", self.project, self.issue).into() }
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
        if let Some(m) = &self.milestone_id {
            match m {
                Some(id) => { params.push("milestone_id", *id); },
                None => { params.push("milestone_id", 0); }
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
}
impl Endpoint for EditMergeRequest {
    fn method(&self) -> Method { Method::PUT }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}", self.project, self.merge_request).into() }
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
}

#[derive(Debug, Clone)]
pub struct ProjectMilestones { pub project: u64 }
impl Endpoint for ProjectMilestones {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/milestones", self.project).into() }
}

#[derive(Debug, Clone)]
pub struct ProjectMembers { pub project: u64 }
impl Endpoint for ProjectMembers {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/members/all", self.project).into() }
}

#[derive(Debug, Clone)]
pub struct Groups { pub page: Option<u64>, pub per_page: Option<u64> }
impl Endpoint for Groups {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "groups".into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(p) = self.page { params.push("page", p); }
        if let Some(pp) = self.per_page { params.push("per_page", pp); }
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
pub struct PagedIssues { pub project: u64, pub page: u64, pub per_page: u64 }
impl Endpoint for PagedIssues {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/issues", self.project).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("page", self.page);
        params.push("per_page", self.per_page);
        params.push("with_labels_details", true);
        params
    }
}
impl Pageable for PagedIssues {}

#[derive(Debug, Clone)]
pub struct PagedMergeRequests { pub project: u64, pub page: u64, pub per_page: u64 }
impl Endpoint for PagedMergeRequests {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests", self.project).into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("page", self.page);
        params.push("per_page", self.per_page);
        params
    }
}
impl Pageable for PagedMergeRequests {}

#[derive(Debug, Clone)]
pub struct PagedProjects { pub membership: bool, pub page: u64, pub per_page: u64 }
impl Endpoint for PagedProjects {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "projects".into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        params.push("membership", self.membership);
        params.push("page", self.page);
        params.push("per_page", self.per_page);
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
pub struct GlobalIssues { pub scope: Option<String>, pub state: Option<String>, pub membership: Option<bool>, pub page: Option<u64>, pub per_page: Option<u64> }
impl Endpoint for GlobalIssues {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "issues".into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(s) = &self.scope { params.push("scope", s); }
        if let Some(s) = &self.state { params.push("state", s); }
        if let Some(m) = self.membership { params.push("membership", m); }
        if let Some(p) = self.page { params.push("page", p); }
        if let Some(pp) = self.per_page { params.push("per_page", pp); }
        params
    }
}
impl Pageable for GlobalIssues {}

#[derive(Debug, Clone)]
pub struct GlobalMergeRequests { pub scope: Option<String>, pub state: Option<String>, pub membership: Option<bool>, pub page: Option<u64>, pub per_page: Option<u64> }
impl Endpoint for GlobalMergeRequests {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "merge_requests".into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(s) = &self.scope { params.push("scope", s); }
        if let Some(s) = &self.state { params.push("state", s); }
        if let Some(m) = self.membership { params.push("membership", m); }
        if let Some(p) = self.page { params.push("page", p); }
        if let Some(pp) = self.per_page { params.push("per_page", pp); }
        params
    }
}
impl Pageable for GlobalMergeRequests {}

#[derive(Debug, Clone)]
pub struct Todos { pub page: Option<u64>, pub per_page: Option<u64> }
impl Endpoint for Todos {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { "todos".into() }
    fn parameters(&self) -> gitlab::api::QueryParams<'_> {
        let mut params = gitlab::api::QueryParams::default();
        if let Some(p) = self.page { params.push("page", p); }
        if let Some(pp) = self.per_page { params.push("per_page", pp); }
        params
    }
}
impl Pageable for Todos {}

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
pub struct JobTrace { pub project: u64, pub job: u64 }
impl Endpoint for JobTrace {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/jobs/{}/trace", self.project, self.job).into() }
}

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
pub struct CreatePipeline { pub project: u64, pub ref_name: String, pub variables: Vec<crate::models::PipelineVariable> }
impl Endpoint for CreatePipeline {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipeline", self.project).into() }
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("ref", &self.ref_name);
        for (i, var) in self.variables.iter().enumerate() {
            params.push(format!("variables[{}][key]", i), &var.key);
            params.push(format!("variables[{}][value]", i), &var.value);
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

#[cfg(test)]
mod tests {
    use super::*;
    use gitlab::api::Endpoint;

    #[test]
    fn test_all_endpoints_exhaustive() {
        let mr_disc = MergeRequestDiscussions { project: 1, merge_request: 2 };
        assert_eq!(mr_disc.method(), Method::GET);
        assert_eq!(mr_disc.endpoint(), "projects/1/merge_requests/2/discussions");

        let curr_user = CurrentUser;
        assert_eq!(curr_user.method(), Method::GET);
        assert_eq!(curr_user.endpoint(), "user");

        let add_disc = AddDiscussionNote { project: 1, merge_request: 2, discussion: "d".into(), body: "b".into() };
        assert_eq!(add_disc.method(), Method::POST);
        assert_eq!(add_disc.endpoint(), "projects/1/merge_requests/2/discussions/d/notes");
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
            assignee_ids: Some(vec![1]), milestone_id: Some(Some(1)),
            due_date: Some(Some("2021-01-01".into())), weight: Some(Some(1)), confidential: Some(true)
        };
        assert_eq!(edit_issue.method(), Method::PUT);
        assert!(edit_issue.body().is_ok());

        let edit_issue_nulls = EditIssue {
            project: 1, issue: 2, title: None, description: None,
            state_event: None, labels: None, assignee_ids: Some(vec![]),
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
            squash: Some(true), target_branch: Some("b".into())
        };
        assert_eq!(edit_mr.method(), Method::PUT);
        assert!(edit_mr.body().is_ok());

        let edit_mr_nulls = EditMergeRequest {
            project: 1, merge_request: 2, title: None, description: None,
            state_event: None, labels: None, assignee_ids: Some(vec![]),
            reviewer_ids: Some(vec![]), milestone_id: Some(None),
            remove_source_branch: None, squash: None, target_branch: None
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

        let p_projs = PagedProjects { membership: true, page: 1, per_page: 1 };
        assert_eq!(p_projs.method(), Method::GET);
        let _ = p_projs.parameters();

        let groups = Groups { page: Some(1), per_page: Some(1) };
        assert_eq!(groups.method(), Method::GET);
        let _ = groups.parameters();

        let group_projs = GroupProjects { group: 1, page: Some(1), per_page: Some(1) };
        assert_eq!(group_projs.method(), Method::GET);
        let _ = group_projs.parameters();

        let p_mrs = PagedMergeRequests { project: 1, page: 1, per_page: 1 };
        assert_eq!(p_mrs.method(), Method::GET);
        let _ = p_mrs.parameters();

        let p_issues = PagedIssues { project: 1, page: 1, per_page: 1 };
        assert_eq!(p_issues.method(), Method::GET);
        let _ = p_issues.parameters();

        let g_mrs = GlobalMergeRequests { scope: Some("a".into()), state: Some("a".into()), membership: Some(true), page: Some(1), per_page: Some(1) };
        assert_eq!(g_mrs.method(), Method::GET);
        let _ = g_mrs.parameters();

        let g_issues = GlobalIssues { scope: Some("a".into()), state: Some("a".into()), membership: Some(true), page: Some(1), per_page: Some(1) };
        assert_eq!(g_issues.method(), Method::GET);
        let _ = g_issues.parameters();

        let todos = Todos { page: Some(1), per_page: Some(1) };
        assert_eq!(todos.method(), Method::GET);
        let _ = todos.parameters();

        let cancel_job = CancelJob { project: 1, job: 2 };
        assert_eq!(cancel_job.method(), Method::POST);

        let retry_job = RetryJob { project: 1, job: 2 };
        assert_eq!(retry_job.method(), Method::POST);

        let play_job = PlayJob { project: 1, job: 2 };
        assert_eq!(play_job.method(), Method::POST);

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

        let cancel_pipe = CancelPipeline { project: 1, pipeline: 2 };
        assert_eq!(cancel_pipe.method(), Method::POST);

        let retry_pipe = RetryPipeline { project: 1, pipeline: 2 };
        assert_eq!(retry_pipe.method(), Method::POST);

        let create_pipe = CreatePipeline { project: 1, ref_name: "b".into(), variables: vec![crate::models::PipelineVariable{key:"k".into(), value:"v".into()}] };
        assert_eq!(create_pipe.method(), Method::POST);
        assert!(create_pipe.body().is_ok());

        let proj_lbls = ProjectLabels { project: 1 };
        assert_eq!(proj_lbls.method(), Method::GET);
        
        let proj_mstones = ProjectMilestones { project: 1 };
        assert_eq!(proj_mstones.method(), Method::GET);

        let proj_members = ProjectMembers { project: 1 };
        assert_eq!(proj_members.method(), Method::GET);

        let add_note = AddNote { project: 1, merge_request: 1, body: "b".into() };
        assert_eq!(add_note.method(), Method::POST);
        assert!(add_note.body().is_ok());

        let proj_branches = ProjectBranches { project: 1 };
        assert_eq!(proj_branches.method(), Method::GET);

        let proj_pipe = ProjectPipeline { project: 1, pipeline: 1 };
        assert_eq!(proj_pipe.method(), Method::GET);

        let approve_mr = ApproveMergeRequest { project: 1, merge_request: 1 };
        assert_eq!(approve_mr.method(), Method::POST);

        let unapprove_mr = UnapproveMergeRequest { project: 1, merge_request: 1 };
        assert_eq!(unapprove_mr.method(), Method::POST);

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

    }
}
