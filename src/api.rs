use gitlab::api::{Endpoint, Pageable};
use gitlab::api::endpoint_prelude::Method;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct AddDiscussionNote {
    pub project: u64,
    pub merge_request: u64,
    pub discussion: String,
    pub body: String,
}

impl Endpoint for AddDiscussionNote {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/merge_requests/{}/discussions/{}/notes",
            self.project, self.merge_request, self.discussion,
        )
        .into()
    }

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
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/merge_requests/{}/discussions/{}",
            self.project, self.merge_request, self.discussion,
        )
        .into()
    }

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
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/issues/{}/discussions/{}/notes",
            self.project, self.issue, self.discussion,
        )
        .into()
    }

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
    pub milestone_id: Option<Option<u64>>, // Some(Some(id)) to set, Some(None) to clear
    pub due_date: Option<Option<String>>,
    pub weight: Option<Option<i64>>,
    pub confidential: Option<bool>,
}

impl Endpoint for EditIssue {
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}", self.project, self.issue).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        if let Some(t) = &self.title { params.push("title", t); }
        if let Some(d) = &self.description { params.push("description", d); }
        if let Some(s) = &self.state_event { params.push("state_event", s); }
        if let Some(l) = &self.labels { params.push("labels", l); }
        if let Some(a) = &self.assignee_ids { 
            for id in a {
                params.push("assignee_ids[]", *id);
            }
        }
        if let Some(m) = &self.milestone_id {
            match m {
                Some(id) => { params.push("milestone_id", *id); },
                None => { params.push("milestone_id", 0); } // 0 or empty often used to clear in some APIs, GitLab docs say 0 clears it
            }
        }
        if let Some(dd) = &self.due_date {
            match dd {
                Some(val) => { params.push("due_date", val); },
                None => { params.push("due_date", ""); }
            }
        }
        if let Some(w) = &self.weight {
            match w {
                Some(val) => { params.push("weight", *val as u64); },
                None => { params.push("weight", 0); } // Check gitlab docs for clear weight
            }
        }
        if let Some(c) = &self.confidential { params.push("confidential", *c); }
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct IssueDiscussions {
    pub project: u64,
    pub issue: u64,
}

impl Endpoint for IssueDiscussions {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}/discussions", self.project, self.issue).into()
    }
}

#[derive(Debug, Clone)]
pub struct AddIssueNote {
    pub project: u64,
    pub issue: u64,
    pub body: String,
}

impl Endpoint for AddIssueNote {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}/notes", self.project, self.issue).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        params.push("body", &self.body);
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct ResolveIssueDiscussion {
    pub project: u64,
    pub issue: u64,
    pub discussion: String,
    pub resolved: bool,
}

impl Endpoint for ResolveIssueDiscussion {
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/issues/{}/discussions/{}",
            self.project, self.issue, self.discussion,
        )
        .into()
    }

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
    pub milestone_id: Option<Option<u64>>,
    pub remove_source_branch: Option<bool>,
    pub squash: Option<bool>,
}

impl Endpoint for EditMergeRequest {
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/merge_requests/{}", self.project, self.merge_request).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, gitlab::api::BodyError> {
        let mut params = gitlab::api::FormParams::default();
        if let Some(t) = &self.title { params.push("title", t); }
        if let Some(d) = &self.description { params.push("description", d); }
        if let Some(s) = &self.state_event { params.push("state_event", s); }
        if let Some(l) = &self.labels { params.push("labels", l); }
        if let Some(a) = &self.assignee_ids { 
            for id in a {
                params.push("assignee_ids[]", *id);
            }
        }
        if let Some(m) = &self.milestone_id {
            match m {
                Some(id) => { params.push("milestone_id", *id); },
                None => { params.push("milestone_id", 0); }
            }
        }
        if let Some(r) = &self.remove_source_branch { params.push("remove_source_branch", *r); }
        if let Some(s) = &self.squash { params.push("squash", *s); }
        params.into_body()
    }
}

#[derive(Debug, Clone)]
pub struct MergeRequestAwardEmoji {
    pub project: u64,
    pub merge_request: u64,
}

impl Endpoint for MergeRequestAwardEmoji {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/merge_requests/{}/award_emoji", self.project, self.merge_request).into() }
}

impl Pageable for MergeRequestAwardEmoji {}

#[derive(Debug, Clone)]
pub struct IssueAwardEmoji {
    pub project: u64,
    pub issue: u64,
}

impl Endpoint for IssueAwardEmoji {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/issues/{}/award_emoji", self.project, self.issue).into() }
}

impl Pageable for IssueAwardEmoji {}

#[derive(Debug, Clone)]
pub struct ProjectLabels {
    pub project: u64,
}

impl Endpoint for ProjectLabels {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/labels", self.project).into() }
}

#[derive(Debug, Clone)]
pub struct ProjectMilestones {
    pub project: u64,
}

impl Endpoint for ProjectMilestones {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/milestones", self.project).into() }
}

#[derive(Debug, Clone)]
pub struct ProjectMembers {
    pub project: u64,
}

impl Endpoint for ProjectMembers {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/members/all", self.project).into() }
}

#[derive(Debug, Clone)]
pub struct ProjectPipelines {
    pub project: u64,
}

impl Endpoint for ProjectPipelines {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipelines", self.project).into() }
}

impl Pageable for ProjectPipelines {}

#[derive(Debug, Clone)]
pub struct ProjectBranches {
    pub project: u64,
}

impl Endpoint for ProjectBranches {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/repository/branches", self.project).into() }
}

impl Pageable for ProjectBranches {}

#[derive(Debug, Clone)]
pub struct ProjectPipeline {
    pub project: u64,
    pub pipeline: u64,
}

impl Endpoint for ProjectPipeline {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipelines/{}", self.project, self.pipeline).into() }
}

#[derive(Debug, Clone)]
pub struct ProjectPipelineJobs {
    pub project: u64,
    pub pipeline: u64,
}

impl Endpoint for ProjectPipelineJobs {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipelines/{}/jobs", self.project, self.pipeline).into() }
}

impl Pageable for ProjectPipelineJobs {}

#[derive(Debug, Clone)]
pub struct JobTrace {
    pub project: u64,
    pub job: u64,
}

impl Endpoint for JobTrace {
    fn method(&self) -> Method { Method::GET }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/jobs/{}/trace", self.project, self.job).into() }
}

#[derive(Debug, Clone)]
pub struct CreatePipeline {
    pub project: u64,
    pub ref_name: String,
    pub variables: Vec<crate::models::PipelineVariable>,
}

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
pub struct RetryPipeline {
    pub project: u64,
    pub pipeline: u64,
}

impl Endpoint for RetryPipeline {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipelines/{}/retry", self.project, self.pipeline).into() }
}

#[derive(Debug, Clone)]
pub struct CancelPipeline {
    pub project: u64,
    pub pipeline: u64,
}

impl Endpoint for CancelPipeline {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/pipelines/{}/cancel", self.project, self.pipeline).into() }
}

#[derive(Debug, Clone)]
pub struct RetryJob {
    pub project: u64,
    pub job: u64,
}

impl Endpoint for RetryJob {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/jobs/{}/retry", self.project, self.job).into() }
}

#[derive(Debug, Clone)]
pub struct CancelJob {
    pub project: u64,
    pub job: u64,
}

impl Endpoint for CancelJob {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/jobs/{}/cancel", self.project, self.job).into() }
}

#[derive(Debug, Clone)]
pub struct PlayJob {
    pub project: u64,
    pub job: u64,
}

impl Endpoint for PlayJob {
    fn method(&self) -> Method { Method::POST }
    fn endpoint(&self) -> Cow<'static, str> { format!("projects/{}/jobs/{}/play", self.project, self.job).into() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitlab::api::endpoint_prelude::Method;

    #[test]
    fn test_all_endpoints_exhaustive() {
        let ep = AddDiscussionNote { project: 1, merge_request: 2, discussion: "d".into(), body: "b".into() };
        assert_eq!(ep.method(), Method::POST);
        assert_eq!(ep.endpoint(), "projects/1/merge_requests/2/discussions/d/notes");
        assert!(ep.body().is_ok());

        let ep = ResolveDiscussion { project: 1, merge_request: 2, discussion: "d".into(), resolved: true };
        assert_eq!(ep.method(), Method::PUT);
        assert_eq!(ep.endpoint(), "projects/1/merge_requests/2/discussions/d");
        let _ = ep.parameters();

        let ep = AddIssueDiscussionNote { project: 1, issue: 2, discussion: "d".into(), body: "b".into() };
        assert_eq!(ep.method(), Method::POST);
        assert_eq!(ep.endpoint(), "projects/1/issues/2/discussions/d/notes");
        assert!(ep.body().is_ok());

        // Test EditIssue combinations
        let ep = EditIssue { project: 1, issue: 2, title: Some("t".into()), description: Some("d".into()), state_event: Some("s".into()), labels: Some("l".into()), assignee_ids: Some(vec![1, 2]), milestone_id: Some(Some(1)), due_date: Some(Some("d".into())), weight: Some(Some(1)), confidential: Some(true) };
        assert_eq!(ep.method(), Method::PUT);
        assert_eq!(ep.endpoint(), "projects/1/issues/2");
        assert!(ep.body().is_ok());

        let ep2 = EditIssue { project: 1, issue: 2, title: None, description: None, state_event: None, labels: None, assignee_ids: None, milestone_id: Some(None), due_date: Some(None), weight: Some(None), confidential: None };
        assert!(ep2.body().is_ok());

        let ep = IssueDiscussions { project: 1, issue: 2 };
        assert_eq!(ep.method(), Method::GET);
        assert_eq!(ep.endpoint(), "projects/1/issues/2/discussions");

        let ep = EditMergeRequest { project: 1, merge_request: 2, title: Some("t".into()), description: Some("d".into()), state_event: Some("s".into()), labels: Some("l".into()), assignee_ids: Some(vec![1]), milestone_id: Some(Some(1)), remove_source_branch: Some(true), squash: Some(true) };
        assert_eq!(ep.method(), Method::PUT);
        assert_eq!(ep.endpoint(), "projects/1/merge_requests/2");
        assert!(ep.body().is_ok());
        
        let ep2 = EditMergeRequest { project: 1, merge_request: 2, title: None, description: None, state_event: None, labels: None, assignee_ids: None, milestone_id: Some(None), remove_source_branch: None, squash: None };
        assert!(ep2.body().is_ok());

        let ep = IssueAwardEmoji { project: 1, issue: 2 };
        assert_eq!(ep.method(), Method::GET);
        assert_eq!(ep.endpoint(), "projects/1/issues/2/award_emoji");

        let ep = MergeRequestAwardEmoji { project: 1, merge_request: 2 };
        assert_eq!(ep.method(), Method::GET);
        assert_eq!(ep.endpoint(), "projects/1/merge_requests/2/award_emoji");

        let ep = ResolveIssueDiscussion { project: 1, issue: 2, discussion: "d".into(), resolved: true };
        assert_eq!(ep.method(), Method::PUT);
        assert_eq!(ep.endpoint(), "projects/1/issues/2/discussions/d");
        let _ = ep.parameters();

        let ep = ProjectLabels { project: 1 };
        assert_eq!(ep.method(), Method::GET);
        assert_eq!(ep.endpoint(), "projects/1/labels");

        let ep = ProjectMilestones { project: 1 };
        assert_eq!(ep.method(), Method::GET);
        assert_eq!(ep.endpoint(), "projects/1/milestones");

        let ep = ProjectMembers { project: 1 };
        assert_eq!(ep.method(), Method::GET);
        assert_eq!(ep.endpoint(), "projects/1/members/all");

        let ep = ProjectPipelines { project: 1 };
        assert_eq!(ep.method(), Method::GET);
        assert_eq!(ep.endpoint(), "projects/1/pipelines");

        let ep = ProjectPipeline { project: 1, pipeline: 2 };
        assert_eq!(ep.method(), Method::GET);
        assert_eq!(ep.endpoint(), "projects/1/pipelines/2");

        let ep = ProjectPipelineJobs { project: 1, pipeline: 2 };
        assert_eq!(ep.method(), Method::GET);
        assert_eq!(ep.endpoint(), "projects/1/pipelines/2/jobs");

        let ep = ProjectBranches { project: 1 };
        assert_eq!(ep.method(), Method::GET);
        assert_eq!(ep.endpoint(), "projects/1/repository/branches");

        let ep = JobTrace { project: 1, job: 2 };
        assert_eq!(ep.method(), Method::GET);
        assert_eq!(ep.endpoint(), "projects/1/jobs/2/trace");

        let ep = CreatePipeline { project: 1, ref_name: "m".into(), variables: vec![crate::models::PipelineVariable{key:"k".into(), value:"v".into()}] };
        assert_eq!(ep.method(), Method::POST);
        assert_eq!(ep.endpoint(), "projects/1/pipeline");
        assert!(ep.body().is_ok());

        let ep = RetryPipeline { project: 1, pipeline: 2 };
        assert_eq!(ep.method(), Method::POST);
        assert_eq!(ep.endpoint(), "projects/1/pipelines/2/retry");

        let ep = CancelPipeline { project: 1, pipeline: 2 };
        assert_eq!(ep.method(), Method::POST);
        assert_eq!(ep.endpoint(), "projects/1/pipelines/2/cancel");

        let ep = RetryJob { project: 1, job: 2 };
        assert_eq!(ep.method(), Method::POST);
        assert_eq!(ep.endpoint(), "projects/1/jobs/2/retry");

        let ep = CancelJob { project: 1, job: 2 };
        assert_eq!(ep.method(), Method::POST);
        assert_eq!(ep.endpoint(), "projects/1/jobs/2/cancel");

        let ep = PlayJob { project: 1, job: 2 };
        assert_eq!(ep.method(), Method::POST);
        assert_eq!(ep.endpoint(), "projects/1/jobs/2/play");
    }
}
