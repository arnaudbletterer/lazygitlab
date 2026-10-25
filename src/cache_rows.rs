use crate::models::{
    Author, Issue, Label, MergeRequest, Milestone, Pipeline, PipelineMin, Project, References, Todo,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CachedItem {
    Mr(CachedMr),
    Issue(CachedIssue),
    Pipeline(CachedPipeline),
    Todo(CachedTodo),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedUser {
    pub id: u64,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedLabel {
    pub title: String,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedMilestone {
    pub id: u64,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedMr {
    pub iid: u64,
    pub project_id: u64,
    pub project_path: Option<String>,
    pub title: String,
    pub state: String,
    pub draft: Option<bool>,
    pub author: CachedUser,
    pub assignees: Vec<CachedUser>,
    pub reviewers: Vec<CachedUser>,
    pub labels: Vec<CachedLabel>,
    pub milestone: Option<CachedMilestone>,
    pub source_branch: String,
    pub target_branch: String,
    pub pipeline_id: Option<u64>,
    pub pipeline_status: Option<String>,
    pub approvals_left: Option<u64>,
    pub unresolved_count: Option<u64>,
    pub web_path: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedIssue {
    pub iid: u64,
    pub project_id: u64,
    pub project_path: Option<String>,
    pub title: String,
    pub state: String,
    pub author: CachedUser,
    pub assignees: Vec<CachedUser>,
    pub labels: Vec<CachedLabel>,
    pub milestone: Option<CachedMilestone>,
    pub issue_type: Option<String>,
    pub confidential: bool,
    pub web_path: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedPipeline {
    pub id: u64,
    pub project_id: u64,
    pub status: String,
    pub source: String,
    pub ref_name: String,
    pub web_path: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedTodoProject {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedTodo {
    pub id: u64,
    pub project: Option<CachedTodoProject>,
    pub action_name: String,
    pub target_type: String,
    pub target_title: String,
    pub target_path: String,
    pub state: String,
    pub created_at: String,
}

fn url_path(url: &str) -> String {
    match url.find("://") {
        Some(i) => {
            let rest = &url[i + 3..];
            rest.find('/')
                .map(|j| rest[j..].to_string())
                .unwrap_or_default()
        }
        None if url.starts_with('/') => url.to_string(),
        None => String::new(),
    }
}

fn join_url(host: &str, path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    format!("{}{path}", host.trim_end_matches('/'))
}

fn project_path(references: &Option<References>) -> Option<String> {
    let full = &references.as_ref()?.full;
    let end = full.find(['!', '#'])?;
    Some(full[..end].to_string())
}

fn rebuild_reference(path: &Option<String>, sigil: char, iid: u64) -> Option<References> {
    path.as_ref().map(|p| References {
        full: format!("{p}{sigil}{iid}"),
    })
}

impl From<&Author> for CachedUser {
    fn from(a: &Author) -> Self {
        Self {
            id: a.id,
            username: a.username.clone(),
        }
    }
}

impl From<&CachedUser> for Author {
    fn from(u: &CachedUser) -> Self {
        Author::new(u.id, u.username.clone())
    }
}

impl From<&Label> for CachedLabel {
    fn from(l: &Label) -> Self {
        Self {
            title: l.name.clone(),
            color: l.color.clone(),
        }
    }
}

impl From<&CachedLabel> for Label {
    fn from(l: &CachedLabel) -> Self {
        Label {
            id: 0,
            name: l.title.clone(),
            color: l.color.clone(),
            text_color: None,
        }
    }
}

impl From<&Milestone> for CachedMilestone {
    fn from(m: &Milestone) -> Self {
        Self {
            id: m.id,
            title: m.title.clone(),
        }
    }
}

impl From<&CachedMilestone> for Milestone {
    fn from(m: &CachedMilestone) -> Self {
        Milestone {
            id: m.id,
            title: m.title.clone(),
        }
    }
}

impl From<&MergeRequest> for CachedMr {
    fn from(m: &MergeRequest) -> Self {
        Self {
            iid: m.iid,
            project_id: m.project_id,
            project_path: project_path(&m.references),
            title: m.title.clone(),
            state: m.state.clone(),
            draft: m.draft,
            author: (&m.author).into(),
            assignees: m.assignees.iter().map(Into::into).collect(),
            reviewers: m.reviewers.iter().map(Into::into).collect(),
            labels: m.labels.iter().map(Into::into).collect(),
            milestone: m.milestone.as_ref().map(Into::into),
            source_branch: m.source_branch.clone(),
            target_branch: m.target_branch.clone(),
            pipeline_id: m.head_pipeline.as_ref().map(|p| p.id),
            pipeline_status: m.head_pipeline.as_ref().map(|p| p.status.clone()),
            approvals_left: None,
            unresolved_count: None,
            web_path: url_path(&m.web_url),
            created_at: m.created_at.clone(),
            updated_at: None,
        }
    }
}

impl CachedMr {
    pub fn to_model(&self, host: &str) -> MergeRequest {
        MergeRequest {
            iid: self.iid,
            project_id: self.project_id,
            title: self.title.clone(),
            description: None,
            author: (&self.author).into(),
            web_url: join_url(host, &self.web_path),
            state: self.state.clone(),
            draft: self.draft,
            references: rebuild_reference(&self.project_path, '!', self.iid),
            upvotes: None,
            downvotes: None,
            labels: self.labels.iter().map(Into::into).collect(),
            milestone: self.milestone.as_ref().map(Into::into),
            assignees: self.assignees.iter().map(Into::into).collect(),
            reviewers: self.reviewers.iter().map(Into::into).collect(),
            squash: None,
            created_at: self.created_at.clone(),
            diff_refs: None,
            reactions: Vec::new(),
            merge_status: None,
            detailed_merge_status: None,
            has_conflicts: None,
            head_pipeline: match (self.pipeline_id, &self.pipeline_status) {
                (Some(id), Some(status)) => Some(PipelineMin {
                    id,
                    status: status.clone(),
                }),
                _ => None,
            },
            source_branch: self.source_branch.clone(),
            target_branch: self.target_branch.clone(),
            participants: Vec::new(),
        }
    }
}

impl From<&Issue> for CachedIssue {
    fn from(i: &Issue) -> Self {
        Self {
            iid: i.iid,
            project_id: i.project_id,
            project_path: project_path(&i.references),
            title: i.title.clone(),
            state: i.state.clone(),
            author: (&i.author).into(),
            assignees: i.assignees.iter().map(Into::into).collect(),
            labels: i.labels.iter().map(Into::into).collect(),
            milestone: i.milestone.as_ref().map(Into::into),
            issue_type: i.issue_type.clone(),
            confidential: i.confidential,
            web_path: url_path(&i.web_url),
            created_at: i.created_at.clone(),
            updated_at: None,
            due_date: i.due_date.clone(),
        }
    }
}

impl CachedIssue {
    pub fn to_model(&self, host: &str) -> Issue {
        Issue {
            iid: self.iid,
            project_id: self.project_id,
            title: self.title.clone(),
            description: None,
            author: (&self.author).into(),
            labels: self.labels.iter().map(Into::into).collect(),
            assignees: self.assignees.iter().map(Into::into).collect(),
            state: self.state.clone(),
            web_url: join_url(host, &self.web_path),
            references: rebuild_reference(&self.project_path, '#', self.iid),
            issue_type: self.issue_type.clone(),
            milestone: self.milestone.as_ref().map(Into::into),
            due_date: self.due_date.clone(),
            start_date: None,
            weight: None,
            task_completion_status: None,
            confidential: self.confidential,
            created_at: self.created_at.clone(),
            upvotes: None,
            downvotes: None,
            reactions: Vec::new(),
            participants: Vec::new(),
        }
    }
}

impl From<&Pipeline> for CachedPipeline {
    fn from(p: &Pipeline) -> Self {
        Self {
            id: p.id,
            project_id: p.project_id,
            status: p.status.clone(),
            source: p.source.clone(),
            ref_name: p.ref_name.clone(),
            web_path: url_path(&p.web_url),
            created_at: p.created_at.clone(),
            started_at: p.started_at.clone(),
            duration: p.duration,
        }
    }
}

impl CachedPipeline {
    pub fn to_model(&self, host: &str) -> Pipeline {
        Pipeline {
            id: self.id,
            project_id: self.project_id,
            status: self.status.clone(),
            source: self.source.clone(),
            ref_name: self.ref_name.clone(),
            web_url: join_url(host, &self.web_path),
            created_at: self.created_at.clone(),
            started_at: self.started_at.clone(),
            duration: self.duration,
        }
    }
}

impl From<&Todo> for CachedTodo {
    fn from(t: &Todo) -> Self {
        Self {
            id: t.id,
            project: t.project.as_ref().map(|p| CachedTodoProject {
                id: p.id,
                name: p.name.clone(),
                path_with_namespace: p.path_with_namespace.clone(),
            }),
            action_name: t.action_name.clone(),
            target_type: t.target_type.clone(),
            target_title: t.body.clone(),
            target_path: url_path(&t.target_url),
            state: t.state.clone(),
            created_at: t.created_at.clone(),
        }
    }
}

impl CachedTodo {
    pub fn to_model(&self, host: &str) -> Todo {
        Todo {
            id: self.id,
            project: self.project.as_ref().map(|p| Project {
                id: p.id,
                name: p.name.clone(),
                path_with_namespace: p.path_with_namespace.clone(),
                web_url: p
                    .path_with_namespace
                    .as_ref()
                    .map(|path| join_url(host, &format!("/{path}"))),
                default_branch: None,
                namespace: None,
            }),
            action_name: self.action_name.clone(),
            target_type: self.target_type.clone(),
            target_url: join_url(host, &self.target_path),
            body: self.target_title.clone(),
            state: self.state.clone(),
            created_at: self.created_at.clone(),
            group: None,
        }
    }
}
