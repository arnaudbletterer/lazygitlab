use gitlab::api::Endpoint;
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
