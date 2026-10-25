//! Batched GraphQL fetch of the dashboard merge request and issue lists.
//!
//! Versions, from the official GitLab docs: `approvalsLeft` exists only on EE (the
//! GraphQL reference lists it on `MergeRequest` without a CE marker), and the
//! discussion counts need GitLab 17.7 per the maintainers. `detailedMergeStatus`
//! replaces `merge_status` as of GitLab 15.6 in the REST docs; the GraphQL reference carries
//! no introduction note for it, so 15.6 is the practical minimum and servers older than
//! that are expected to answer with an unknown-field error, which maps to `Unsupported`.
//! When the server rejects one of the optional fields (approvals, discussion counts), the
//! query drops exactly that group and retries; the level is remembered per (host, kind) in
//! [`GraphqlState`].

use crate::trace::TracedClient;
use crate::models::{Author, Issue, Label, MergeRequest, Milestone, PipelineMin, References};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

const MAX_BATCH: usize = 20;
const MAX_IN_FLIGHT: usize = 4;
const MAX_FOLLOWUP_PAGES_PER_PROJECT: usize = 3;
const MAX_FOLLOWUP_REQUESTS_PER_CALL: usize = 40;
const DEFAULT_COMPLEXITY_LIMIT: u64 = 250;
const COMPLEXITY_SHRINK_RATIO: f64 = 0.8;
const GROW_AFTER_SUCCESSES: u32 = 5;
const NOT_FOUND_BEFORE_UNSUPPORTED: u32 = 2;
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

const MR_FIELDS_HEAD: &str = "iid title state draft webUrl createdAt updatedAt detailedMergeStatus conflicts sourceBranch targetBranch author{id username name} assignees{nodes{id username}} reviewers{nodes{id username}} labels{nodes{id title color textColor}} milestone{id title} headPipeline{id status} approved";
const ISSUE_FIELDS: &str = "iid title state confidential webUrl createdAt updatedAt dueDate userNotesCount author{id username name} assignees{nodes{id username}} labels{nodes{id title color textColor}} milestone{id title}";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListKind {
    MergeRequests,
    Issues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Level {
    pub approvals: bool,
    pub discussions: bool,
}

impl Level {
    pub const FULL: Level = Level {
        approvals: true,
        discussions: true,
    };

    fn without(self, field: &str) -> Option<Level> {
        match field {
            "approvalsLeft" => Some(Level {
                approvals: false,
                ..self
            }),
            "resolvableDiscussionsCount" | "resolvedDiscussionsCount" => Some(Level {
                discussions: false,
                ..self
            }),
            _ => None,
        }
    }

    fn and(self, o: Level) -> Level {
        Level {
            approvals: self.approvals && o.approvals,
            discussions: self.discussions && o.discussions,
        }
    }
}

fn connection_args(name: &str, ty: &str) -> String {
    format!(
        "query($ids:[ID!], $state:{ty}, $since:Time, $after:String, $first:Int){{ projects(ids:$ids, first:20){{ nodes{{ id fullPath {name}(state:$state, sort:UPDATED_DESC, first:$first, updatedAfter:$since, after:$after){{ pageInfo{{hasNextPage endCursor}} nodes{{ "
    )
}

const QUERY_TAIL: &str = " } } } } queryComplexity{score limit} }";

fn mr_query(level: Level) -> String {
    let mut q = connection_args("mergeRequests", "MergeRequestState");
    q.push_str(MR_FIELDS_HEAD);
    if level.approvals {
        q.push_str(" approvalsLeft");
    }
    if level.discussions {
        q.push_str(" resolvableDiscussionsCount resolvedDiscussionsCount");
    }
    q.push_str(QUERY_TAIL);
    q
}

fn issue_query(_: Level) -> String {
    let mut q = connection_args("issues", "IssuableState");
    q.push_str(ISSUE_FIELDS);
    q.push_str(QUERY_TAIL);
    q
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Complexity {
    pub score: u64,
    pub limit: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphqlError {
    Unsupported,
    RateLimited,
    Failed(String),
}

impl std::fmt::Display for GraphqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "GraphQL lists unsupported by this GitLab"),
            Self::RateLimited => write!(f, "rate limited by GitLab (429)"),
            Self::Failed(m) => write!(f, "GraphQL request failed: {m}"),
        }
    }
}

impl std::error::Error for GraphqlError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListResult<T> {
    pub rows: Vec<T>,
    pub complete: bool,
    pub rate_limited: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MrRow {
    /// `milestone.id` and `head_pipeline.id` are 0 here when the server id could not be
    /// parsed; use `milestone_id` and `pipeline_id` to tell.
    pub mr: MergeRequest,
    pub pipeline_id: Option<u64>,
    pub milestone_id: Option<u64>,
    pub path_with_namespace: String,
    pub updated_at: Option<String>,
    pub approved: Option<bool>,
    pub approvals_left: Option<u64>,
    pub resolvable_discussions: Option<u64>,
    pub resolved_discussions: Option<u64>,
}

impl MrRow {
    pub fn unresolved_discussions(&self) -> Option<u64> {
        Some(
            self.resolvable_discussions?
                .saturating_sub(self.resolved_discussions?),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueRow {
    /// `milestone.id` is 0 here when the server id could not be parsed; use `milestone_id`.
    pub issue: Issue,
    pub milestone_id: Option<u64>,
    pub path_with_namespace: String,
    pub updated_at: Option<String>,
    pub user_notes_count: Option<u64>,
}

#[derive(Clone, Copy)]
struct KindState {
    unsupported: bool,
    size: usize,
    first: usize,
    level: Level,
    last: Option<Complexity>,
    not_found: u32,
    ok_streak: u32,
}

impl Default for KindState {
    fn default() -> Self {
        Self {
            unsupported: false,
            size: MAX_BATCH,
            first: MAX_BATCH,
            level: Level::FULL,
            last: None,
            not_found: 0,
            ok_streak: 0,
        }
    }
}

#[derive(Default)]
pub struct GraphqlState {
    inner: Mutex<HashMap<(String, ListKind), KindState>>,
}

impl GraphqlState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&self) {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }

    fn with<R>(&self, url: &str, kind: ListKind, f: impl FnOnce(&mut KindState) -> R) -> R {
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        f(map.entry((url.to_string(), kind)).or_default())
    }

    fn read<R>(
        &self,
        client: &impl gitlab::api::RestClient,
        kind: ListKind,
        f: impl FnOnce(&KindState) -> R,
        default: R,
    ) -> R {
        match graphql_url(client) {
            Ok(url) => self.with(&url, kind, |k| f(k)),
            Err(_) => default,
        }
    }

    pub fn last_complexity(
        &self,
        client: &impl gitlab::api::RestClient,
        kind: ListKind,
    ) -> Option<Complexity> {
        self.read(client, kind, |k| k.last, None)
    }

    pub fn batch_size(&self, client: &impl gitlab::api::RestClient, kind: ListKind) -> usize {
        self.read(client, kind, |k| k.size, MAX_BATCH)
    }

    pub fn first_size(&self, client: &impl gitlab::api::RestClient, kind: ListKind) -> usize {
        self.read(client, kind, |k| k.first, MAX_BATCH)
    }

    pub fn is_unsupported(&self, client: &impl gitlab::api::RestClient, kind: ListKind) -> bool {
        self.read(client, kind, |k| k.unsupported, false)
    }

    pub fn level(&self, client: &impl gitlab::api::RestClient, kind: ListKind) -> Level {
        self.read(client, kind, |k| k.level, Level::FULL)
    }
}

pub fn graphql_url(client: &impl gitlab::api::RestClient) -> Result<String, GraphqlError> {
    let probe = client
        .rest_endpoint("x")
        .map_err(|e| GraphqlError::Failed(e.to_string()))?;
    let s = probe.as_str();
    let base = s
        .strip_suffix("/api/v4/x")
        .ok_or_else(|| GraphqlError::Failed(format!("unexpected REST endpoint {s}")))?;
    Ok(format!("{base}/api/graphql"))
}

#[derive(Deserialize)]
struct Envelope {
    data: Option<Value>,
    #[serde(default)]
    errors: Vec<GqlMessage>,
}

#[derive(Deserialize)]
struct GqlMessage {
    #[serde(default)]
    message: String,
}

#[derive(Deserialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Deserialize)]
struct Nodes<N> {
    nodes: Vec<Option<N>>,
}

#[derive(Deserialize)]
struct GqlUser {
    id: String,
    username: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Deserialize)]
struct GqlLabel {
    #[serde(default)]
    id: Option<String>,
    title: String,
    #[serde(default)]
    color: String,
    #[serde(default, rename = "textColor")]
    text_color: Option<String>,
}

#[derive(Deserialize)]
struct GqlMilestone {
    #[serde(default)]
    id: Option<String>,
    title: String,
}

#[derive(Deserialize)]
struct GqlPipeline {
    #[serde(default)]
    id: Option<String>,
    status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MrNode {
    iid: String,
    title: String,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    draft: Option<bool>,
    #[serde(default)]
    web_url: String,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
    #[serde(default)]
    detailed_merge_status: Option<String>,
    #[serde(default)]
    conflicts: Option<bool>,
    #[serde(default)]
    source_branch: String,
    #[serde(default)]
    target_branch: String,
    #[serde(default)]
    author: Option<GqlUser>,
    #[serde(default)]
    assignees: Option<Nodes<GqlUser>>,
    #[serde(default)]
    reviewers: Option<Nodes<GqlUser>>,
    #[serde(default)]
    labels: Option<Nodes<GqlLabel>>,
    #[serde(default)]
    milestone: Option<GqlMilestone>,
    #[serde(default)]
    head_pipeline: Option<GqlPipeline>,
    #[serde(default)]
    approved: Option<bool>,
    #[serde(default)]
    approvals_left: Option<u64>,
    #[serde(default)]
    resolvable_discussions_count: Option<u64>,
    #[serde(default)]
    resolved_discussions_count: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IssueNode {
    iid: String,
    title: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    web_url: String,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
    #[serde(default)]
    confidential: Option<bool>,
    #[serde(default)]
    due_date: Option<String>,
    #[serde(default)]
    user_notes_count: Option<u64>,
    #[serde(default)]
    author: Option<GqlUser>,
    #[serde(default)]
    assignees: Option<Nodes<GqlUser>>,
    #[serde(default)]
    labels: Option<Nodes<GqlLabel>>,
    #[serde(default)]
    milestone: Option<GqlMilestone>,
}

fn gid_number(gid: &str) -> Result<u64, String> {
    gid.rsplit('/')
        .next()
        .and_then(|n| n.parse().ok())
        .ok_or_else(|| format!("bad global id {gid}"))
}

fn parsed_id(id: &Option<String>) -> Option<u64> {
    id.as_deref().and_then(|g| gid_number(g).ok())
}

fn author(u: &GqlUser) -> Result<Author, String> {
    let mut a = Author::new(gid_number(&u.id)?, u.username.clone());
    a.name = u.name.clone();
    Ok(a)
}

fn authors(n: &Option<Nodes<GqlUser>>) -> Result<Vec<Author>, String> {
    n.as_ref()
        .map(|n| n.nodes.iter().flatten().map(author).collect())
        .unwrap_or_else(|| Ok(Vec::new()))
}

fn labels(n: &Option<Nodes<GqlLabel>>) -> (Vec<Label>, bool) {
    let mut ok = true;
    let mut out = Vec::new();
    for l in n.iter().flat_map(|n| n.nodes.iter().flatten()) {
        match parsed_id(&l.id) {
            Some(id) => out.push(Label {
                id,
                name: l.title.clone(),
                color: l.color.clone(),
                text_color: l.text_color.clone(),
            }),
            None => ok = false,
        }
    }
    (out, ok)
}

fn milestone(m: &Option<GqlMilestone>) -> Option<Milestone> {
    m.as_ref().map(|m| Milestone {
        id: parsed_id(&m.id).unwrap_or(0),
        title: m.title.clone(),
    })
}

fn author_or_ghost(u: &Option<GqlUser>) -> Result<Author, String> {
    match u {
        Some(u) => author(u),
        None => Ok(Author::new(0, "ghost".into())),
    }
}

fn map_mr(project_id: u64, path: &str, n: MrNode) -> Result<(MrRow, bool), String> {
    let iid: u64 = n.iid.parse().map_err(|_| format!("bad iid {}", n.iid))?;
    let (label_list, labels_ok) = labels(&n.labels);
    let pipeline_id = n.head_pipeline.as_ref().and_then(|p| parsed_id(&p.id));
    let milestone_id = n.milestone.as_ref().and_then(|m| parsed_id(&m.id));
    let mr = MergeRequest {
        iid,
        project_id,
        title: n.title,
        description: None,
        author: author_or_ghost(&n.author)?,
        web_url: n.web_url,
        state: n.state.unwrap_or_default().to_lowercase(),
        draft: n.draft,
        references: Some(References {
            full: format!("{path}!{iid}"),
        }),
        upvotes: None,
        downvotes: None,
        labels: label_list,
        milestone: milestone(&n.milestone),
        assignees: authors(&n.assignees)?,
        reviewers: authors(&n.reviewers)?,
        squash: None,
        created_at: n.created_at,
        updated_at: n.updated_at.clone(),
        diff_refs: None,
        reactions: Vec::new(),
        merge_status: None,
        detailed_merge_status: n.detailed_merge_status.map(|s| s.to_lowercase()),
        has_conflicts: n.conflicts,
        head_pipeline: n.head_pipeline.map(|p| PipelineMin {
            id: pipeline_id.unwrap_or(0),
            status: p.status.to_lowercase(),
        }),
        source_branch: n.source_branch,
        target_branch: n.target_branch,
        participants: Vec::new(),
    };
    let row = MrRow {
        mr,
        pipeline_id,
        milestone_id,
        path_with_namespace: path.to_string(),
        updated_at: n.updated_at,
        approved: n.approved,
        approvals_left: n.approvals_left,
        resolvable_discussions: n.resolvable_discussions_count,
        resolved_discussions: n.resolved_discussions_count,
    };
    Ok((row, labels_ok))
}

fn map_issue(project_id: u64, path: &str, n: IssueNode) -> Result<(IssueRow, bool), String> {
    let iid: u64 = n.iid.parse().map_err(|_| format!("bad iid {}", n.iid))?;
    let (label_list, labels_ok) = labels(&n.labels);
    let milestone_id = n.milestone.as_ref().and_then(|m| parsed_id(&m.id));
    let issue = Issue {
        iid,
        project_id,
        title: n.title,
        description: None,
        author: author_or_ghost(&n.author)?,
        labels: label_list,
        assignees: authors(&n.assignees)?,
        state: n.state.to_lowercase(),
        web_url: n.web_url,
        references: Some(References {
            full: format!("{path}#{iid}"),
        }),
        issue_type: None,
        milestone: milestone(&n.milestone),
        due_date: n.due_date.map(|d| d.chars().take(10).collect()),
        start_date: None,
        weight: None,
        task_completion_status: None,
        confidential: n.confidential.unwrap_or(false),
        created_at: n.created_at,
        updated_at: n.updated_at.clone(),
        upvotes: None,
        downvotes: None,
        reactions: Vec::new(),
        participants: Vec::new(),
    };
    let row = IssueRow {
        issue,
        milestone_id,
        path_with_namespace: path.to_string(),
        updated_at: n.updated_at,
        user_notes_count: n.user_notes_count,
    };
    Ok((row, labels_ok))
}

enum Fail {
    Err(GraphqlError),
    UnknownFields(Vec<String>),
    ComplexityExceeded,
    NotFound,
}

impl From<GraphqlError> for Fail {
    fn from(e: GraphqlError) -> Self {
        Fail::Err(e)
    }
}

enum Validation {
    Fields(Vec<String>),
    Schema,
}

fn classify_validation(errors: &[GqlMessage]) -> Option<Validation> {
    let mut fields = Vec::new();
    for e in errors {
        let m = &e.message;
        let l = m.to_ascii_lowercase();
        if l.contains("unknown type") || l.contains("isn't a defined input type") {
            return Some(Validation::Schema);
        }
        if l.starts_with("field '") && l.contains("' doesn't exist on type") {
            let name = m["Field '".len()..].split('\'').next().unwrap_or("");
            fields.push(name.to_string());
        }
    }
    if fields.is_empty() {
        None
    } else {
        Some(Validation::Fields(fields))
    }
}

type MapFn<N, T> = fn(u64, &str, N) -> Result<(T, bool), String>;

struct Kind<N, T> {
    list: ListKind,
    query: fn(Level) -> String,
    field: &'static str,
    map: MapFn<N, T>,
}

impl<N, T> Clone for Kind<N, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<N, T> Copy for Kind<N, T> {}

struct Request {
    url: String,
    state: String,
    since: Option<String>,
    first: usize,
    followups: AtomicUsize,
    sem: Semaphore,
}

struct Page {
    data: Value,
    complete: bool,
    complexity: Option<Complexity>,
}

async fn post(
    client: &TracedClient,
    req: &Request,
    query: &str,
    ids: &[u64],
    after: Option<&str>,
) -> Result<Page, Fail> {
    use gitlab::api::AsyncClient;
    let gids: Vec<String> = ids
        .iter()
        .map(|i| format!("gid://gitlab/Project/{i}"))
        .collect();
    let body = json!({
        "query": query,
        "variables": {
            "ids": gids,
            "state": req.state,
            "since": req.since,
            "after": after,
            "first": req.first,
        }
    });
    let http_req = http::Request::builder()
        .method("POST")
        .uri(req.url.as_str())
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::ACCEPT, "application/json");
    let _permit = req.sem.acquire().await;
    let resp = tokio::time::timeout(
        REQUEST_TIMEOUT,
        client.rest_async(http_req, body.to_string().into_bytes()),
    )
    .await
    .map_err(|_| GraphqlError::Failed("request timed out".into()))?
    .map_err(|e| GraphqlError::Failed(e.to_string()))?;
    let status = resp.status();
    if status == http::StatusCode::TOO_MANY_REQUESTS {
        return Err(GraphqlError::RateLimited.into());
    }
    if status == http::StatusCode::NOT_FOUND {
        return Err(Fail::NotFound);
    }
    if !status.is_success() {
        return Err(GraphqlError::Failed(format!("HTTP {status}")).into());
    }
    let env: Envelope = serde_json::from_slice(resp.body())
        .map_err(|_| GraphqlError::Failed("response is not GraphQL JSON".into()))?;
    let data = match env.data {
        Some(d) if !d.is_null() => d,
        _ => {
            return Err(match classify_validation(&env.errors) {
                Some(Validation::Schema) => GraphqlError::Unsupported.into(),
                Some(Validation::Fields(f)) => Fail::UnknownFields(f),
                None => {
                    let msg = env
                        .errors
                        .first()
                        .map(|e| e.message.clone())
                        .unwrap_or_else(|| "empty response".into());
                    if msg.to_ascii_lowercase().contains("complexity") {
                        Fail::ComplexityExceeded
                    } else {
                        GraphqlError::Failed(msg).into()
                    }
                }
            });
        }
    };
    let complexity = data.get("queryComplexity").and_then(|c| {
        Some(Complexity {
            score: c.get("score")?.as_u64()?,
            limit: c
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(DEFAULT_COMPLEXITY_LIMIT),
        })
    });
    if let Some(c) = complexity {
        tracing::debug!(score = c.score, limit = c.limit, "graphql query complexity");
    }
    Ok(Page {
        data,
        complete: env.errors.is_empty(),
        complexity,
    })
}

struct BatchOut<T> {
    rows: Vec<T>,
    complete: bool,
    max_score: Option<Complexity>,
    level: Level,
    over_limit: bool,
    rate_limited: bool,
}

fn project_nodes(data: &Value) -> Result<&Vec<Value>, GraphqlError> {
    data.get("projects")
        .and_then(|p| p.get("nodes"))
        .and_then(Value::as_array)
        .ok_or_else(|| GraphqlError::Failed("unexpected GraphQL response shape".into()))
}

fn merge_complexity(acc: &mut Option<Complexity>, c: Option<Complexity>) {
    if let Some(c) = c
        && acc.is_none_or(|a| c.score > a.score)
    {
        *acc = Some(c);
    }
}

fn over_threshold(c: Complexity) -> bool {
    c.score as f64 > COMPLEXITY_SHRINK_RATIO * c.limit as f64
}

type NextPage = Option<(u64, String)>;

fn absorb<N, T>(node: &Value, kind: &Kind<N, T>, out: &mut BatchOut<T>) -> NextPage
where
    N: DeserializeOwned,
{
    let ident = node
        .get("id")
        .and_then(Value::as_str)
        .and_then(|g| gid_number(g).ok());
    let path = node.get("fullPath").and_then(Value::as_str);
    let (Some(pid), Some(path)) = (ident, path) else {
        out.complete = false;
        return None;
    };
    let conn = node.get(kind.field).filter(|c| !c.is_null())?;
    let Some(items) = conn.get("nodes").and_then(Value::as_array) else {
        out.complete = false;
        return None;
    };
    for item in items.iter().filter(|i| !i.is_null()) {
        let row = N::deserialize(item)
            .map_err(|e| e.to_string())
            .and_then(|n| (kind.map)(pid, path, n));
        match row {
            Ok((r, ok)) => {
                out.complete &= ok;
                out.rows.push(r);
            }
            Err(e) => {
                tracing::warn!("skipping unparsable graphql node: {e}");
                out.complete = false;
            }
        }
    }
    let info = conn
        .get("pageInfo")
        .and_then(|p| PageInfo::deserialize(p).ok());
    match info {
        Some(PageInfo {
            has_next_page: true,
            end_cursor: Some(cursor),
        }) => Some((pid, cursor)),
        Some(_) => None,
        None => {
            out.complete = false;
            None
        }
    }
}

async fn run_batch<N, T>(
    client: TracedClient,
    req: Arc<Request>,
    kind: Kind<N, T>,
    start: Level,
    ids: Vec<u64>,
) -> Result<BatchOut<T>, Fail>
where
    N: DeserializeOwned,
{
    let mut level = start;
    let mut retries = 0;
    let page = loop {
        match post(&client, &req, &(kind.query)(level), &ids, None).await {
            Ok(p) => break p,
            Err(Fail::UnknownFields(fields)) if retries < 2 => {
                retries += 1;
                for f in &fields {
                    level = level.without(f).ok_or(GraphqlError::Unsupported)?;
                }
            }
            Err(Fail::UnknownFields(_)) => return Err(GraphqlError::Unsupported.into()),
            Err(e) => return Err(e),
        }
    };
    let query = (kind.query)(level);
    let mut out = BatchOut {
        rows: Vec::new(),
        complete: page.complete,
        max_score: page.complexity,
        level,
        over_limit: false,
        rate_limited: false,
    };
    let mut followups: Vec<(u64, String)> = Vec::new();
    for node in project_nodes(&page.data)? {
        if let Some(next) = absorb(node, &kind, &mut out) {
            followups.push(next);
        }
    }
    for (pid, mut cursor) in followups {
        let mut pages = 0;
        loop {
            if pages >= MAX_FOLLOWUP_PAGES_PER_PROJECT
                || req.followups.fetch_add(1, Ordering::Relaxed) >= MAX_FOLLOWUP_REQUESTS_PER_CALL
            {
                out.complete = false;
                break;
            }
            pages += 1;
            let page = match post(&client, &req, &query, &[pid], Some(&cursor)).await {
                Ok(p) => p,
                Err(Fail::ComplexityExceeded) => {
                    out.complete = false;
                    out.over_limit = true;
                    break;
                }
                Err(Fail::NotFound) => {
                    out.complete = false;
                    break;
                }
                Err(Fail::UnknownFields(_)) => return Err(GraphqlError::Unsupported.into()),
                Err(Fail::Err(GraphqlError::RateLimited)) => {
                    out.complete = false;
                    out.rate_limited = true;
                    return Ok(out);
                }
                Err(Fail::Err(GraphqlError::Failed(m))) => {
                    tracing::warn!("graphql follow-up page failed: {m}");
                    out.complete = false;
                    break;
                }
                Err(e) => return Err(e),
            };
            merge_complexity(&mut out.max_score, page.complexity);
            out.complete &= page.complete;
            let mut next = None;
            for node in project_nodes(&page.data)? {
                if let Some((_, n)) = absorb(node, &kind, &mut out) {
                    next = Some(n);
                }
            }
            match next {
                Some(n) => cursor = n,
                None => break,
            }
        }
    }
    Ok(out)
}

fn shrink(size: usize) -> usize {
    match size {
        s if s > 10 => 10,
        s if s > 5 => 5,
        s => s,
    }
}

fn grow(size: usize) -> usize {
    match size {
        s if s < 10 => 10,
        _ => MAX_BATCH,
    }
}

async fn fetch<N, T, C>(
    client: &C,
    state: &GraphqlState,
    kind: Kind<N, T>,
    project_ids: &[u64],
    list_state: &str,
    since: Option<&str>,
) -> Result<ListResult<T>, GraphqlError>
where
    N: DeserializeOwned + Send + 'static,
    T: Send + 'static,
    C: Clone + Into<TracedClient>,
{
    let client: TracedClient = client.clone().into();
    let client = &client;
    let url = graphql_url(client)?;
    let snapshot = state.with(&url, kind.list, |k| *k);
    if snapshot.unsupported {
        return Err(GraphqlError::Unsupported);
    }
    let req = Arc::new(Request {
        url: url.clone(),
        state: list_state.to_string(),
        since: since.map(str::to_string),
        first: snapshot.first,
        followups: AtomicUsize::new(0),
        sem: Semaphore::new(MAX_IN_FLIGHT),
    });
    let mut set = JoinSet::new();
    for (idx, chunk) in project_ids.chunks(snapshot.size).enumerate() {
        let (client, req, chunk) = (client.clone(), req.clone(), chunk.to_vec());
        let level = snapshot.level;
        set.spawn(async move { (idx, run_batch(client, req, kind, level, chunk).await) });
    }
    let mut results: Vec<(usize, Result<BatchOut<T>, Fail>)> = Vec::new();
    while let Some(r) = set.join_next().await {
        match r {
            Ok(r) => results.push(r),
            Err(e) => results.push((
                usize::MAX,
                Err(Fail::Err(GraphqlError::Failed(e.to_string()))),
            )),
        }
    }
    results.sort_by_key(|(i, _)| *i);

    let mut worst: Option<Complexity> = None;
    let mut unsupported = false;
    let mut rate_limited = false;
    let mut not_found = false;
    let mut any_ok = false;
    let mut rows = Vec::new();
    let mut complete = true;
    let mut over_limit = false;
    let mut level = snapshot.level;
    for (_, r) in results {
        match r {
            Ok(b) => {
                any_ok = true;
                merge_complexity(&mut worst, b.max_score);
                complete &= b.complete;
                over_limit |= b.over_limit;
                rate_limited |= b.rate_limited;
                level = level.and(b.level);
                rows.extend(b.rows);
            }
            Err(Fail::Err(GraphqlError::Unsupported)) | Err(Fail::UnknownFields(_)) => {
                unsupported = true
            }
            Err(Fail::Err(GraphqlError::RateLimited)) => rate_limited = true,
            Err(Fail::NotFound) => {
                not_found = true;
                complete = false;
            }
            Err(Fail::ComplexityExceeded) => {
                over_limit = true;
                complete = false;
            }
            Err(Fail::Err(GraphqlError::Failed(m))) => {
                tracing::warn!("graphql batch failed: {m}");
                complete = false;
            }
        }
    }
    state.with(&url, kind.list, |k| {
        k.level = k.level.and(level);
        if let Some(c) = worst {
            k.last = Some(c);
        }
        if over_limit || worst.is_some_and(over_threshold) {
            k.size = shrink(k.size);
            k.first = shrink(k.first);
            k.ok_streak = 0;
        } else if worst.is_some() {
            k.ok_streak += 1;
            if k.ok_streak >= GROW_AFTER_SUCCESSES {
                k.size = grow(k.size);
                k.first = grow(k.first);
                k.ok_streak = 0;
            }
        }
        if any_ok {
            k.not_found = 0;
        } else if not_found {
            k.not_found += 1;
            if k.not_found >= NOT_FOUND_BEFORE_UNSUPPORTED {
                unsupported = true;
            }
        }
        if unsupported {
            k.unsupported = true;
        }
    });
    if unsupported {
        return Err(GraphqlError::Unsupported);
    }
    if rate_limited && rows.is_empty() {
        return Err(GraphqlError::RateLimited);
    }
    Ok(ListResult {
        rows,
        complete: complete && !rate_limited,
        rate_limited,
    })
}

pub async fn fetch_merge_requests<C: Clone + Into<TracedClient>>(
    client: &C,
    state: &GraphqlState,
    project_ids: &[u64],
    list_state: &str,
    since: Option<&str>,
) -> Result<ListResult<MrRow>, GraphqlError> {
    let kind: Kind<MrNode, MrRow> = Kind {
        list: ListKind::MergeRequests,
        query: mr_query,
        field: "mergeRequests",
        map: map_mr,
    };
    fetch(client, state, kind, project_ids, list_state, since).await
}

pub async fn fetch_issues<C: Clone + Into<TracedClient>>(
    client: &C,
    state: &GraphqlState,
    project_ids: &[u64],
    list_state: &str,
    since: Option<&str>,
) -> Result<ListResult<IssueRow>, GraphqlError> {
    let kind: Kind<IssueNode, IssueRow> = Kind {
        list: ListKind::Issues,
        query: issue_query,
        field: "issues",
        map: map_issue,
    };
    fetch(client, state, kind, project_ids, list_state, since).await
}
