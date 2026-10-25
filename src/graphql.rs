use crate::models::{Author, Issue, Label, MergeRequest, Milestone, PipelineMin, References};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

const MAX_BATCH: usize = 20;
const MAX_IN_FLIGHT: usize = 4;
const MAX_PAGES_PER_PROJECT: usize = 5;
const COMPLEXITY_THRESHOLD: u64 = 200;
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

const MR_QUERY: &str = "query($ids:[ID!], $state:MergeRequestState, $since:Time, $after:String){ projects(ids:$ids, first:20){ nodes{ id fullPath mergeRequests(state:$state, sort:UPDATED_DESC, first:20, updatedAfter:$since, after:$after){ pageInfo{hasNextPage endCursor} nodes{ iid title state draft webUrl createdAt updatedAt detailedMergeStatus conflicts sourceBranch targetBranch author{id username name} assignees{nodes{id username}} reviewers{nodes{id username}} labels{nodes{title color textColor}} milestone{title} headPipeline{status} approved approvalsLeft resolvableDiscussionsCount resolvedDiscussionsCount } } } } queryComplexity{score limit} }";

const ISSUE_QUERY: &str = "query($ids:[ID!], $state:IssuableState, $since:Time, $after:String){ projects(ids:$ids, first:20){ nodes{ id fullPath issues(state:$state, sort:UPDATED_DESC, first:20, updatedAfter:$since, after:$after){ pageInfo{hasNextPage endCursor} nodes{ iid title state webUrl createdAt updatedAt dueDate userNotesCount author{id username name} assignees{nodes{id username}} labels{nodes{title color textColor}} milestone{title} } } } } queryComplexity{score limit} }";

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MrRow {
    pub mr: MergeRequest,
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
    pub issue: Issue,
    pub path_with_namespace: String,
    pub updated_at: Option<String>,
    pub user_notes_count: Option<u64>,
}

struct HostState {
    unsupported: bool,
    batch: usize,
    last: Option<Complexity>,
}

static HOSTS: LazyLock<Mutex<HashMap<String, HostState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn with_host<R>(key: &str, f: impl FnOnce(&mut HostState) -> R) -> R {
    let mut map = HOSTS.lock().unwrap_or_else(|e| e.into_inner());
    let st = map.entry(key.to_string()).or_insert(HostState {
        unsupported: false,
        batch: MAX_BATCH,
        last: None,
    });
    f(st)
}

pub fn graphql_url(client: &gitlab::AsyncGitlab) -> Result<String, GraphqlError> {
    use gitlab::api::RestClient;
    let probe = client
        .rest_endpoint("x")
        .map_err(|e| GraphqlError::Failed(e.to_string()))?;
    let s = probe.as_str();
    let base = s
        .strip_suffix("/api/v4/x")
        .ok_or_else(|| GraphqlError::Failed(format!("unexpected REST endpoint {s}")))?;
    Ok(format!("{base}/api/graphql"))
}

pub fn last_complexity(client: &gitlab::AsyncGitlab) -> Option<Complexity> {
    let url = graphql_url(client).ok()?;
    with_host(&url, |h| h.last)
}

pub fn current_batch_size(client: &gitlab::AsyncGitlab) -> usize {
    match graphql_url(client) {
        Ok(url) => with_host(&url, |h| h.batch),
        Err(_) => MAX_BATCH,
    }
}

pub fn is_marked_unsupported(client: &gitlab::AsyncGitlab) -> bool {
    match graphql_url(client) {
        Ok(url) => with_host(&url, |h| h.unsupported),
        Err(_) => false,
    }
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
struct Connection<N> {
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
    nodes: Vec<Option<N>>,
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
    title: String,
    #[serde(default)]
    color: String,
    #[serde(default, rename = "textColor")]
    text_color: Option<String>,
}

#[derive(Deserialize)]
struct GqlMilestone {
    title: String,
}

#[derive(Deserialize)]
struct GqlPipeline {
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

fn labels(n: &Option<Nodes<GqlLabel>>) -> Vec<Label> {
    n.as_ref()
        .map(|n| {
            n.nodes
                .iter()
                .flatten()
                .map(|l| Label {
                    id: 0,
                    name: l.title.clone(),
                    color: l.color.clone(),
                    text_color: l.text_color.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn milestone(m: &Option<GqlMilestone>) -> Option<Milestone> {
    m.as_ref().map(|m| Milestone {
        id: 0,
        title: m.title.clone(),
    })
}

fn author_or_ghost(u: &Option<GqlUser>) -> Result<Author, String> {
    match u {
        Some(u) => author(u),
        None => Ok(Author::new(0, "ghost".into())),
    }
}

fn map_mr(project_id: u64, path: &str, n: MrNode) -> Result<MrRow, String> {
    let iid: u64 = n.iid.parse().map_err(|_| format!("bad iid {}", n.iid))?;
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
        labels: labels(&n.labels),
        milestone: milestone(&n.milestone),
        assignees: authors(&n.assignees)?,
        reviewers: authors(&n.reviewers)?,
        squash: None,
        created_at: n.created_at,
        diff_refs: None,
        reactions: Vec::new(),
        merge_status: None,
        detailed_merge_status: n.detailed_merge_status.map(|s| s.to_lowercase()),
        has_conflicts: n.conflicts,
        head_pipeline: n.head_pipeline.map(|p| PipelineMin {
            id: 0,
            status: p.status.to_lowercase(),
        }),
        source_branch: n.source_branch,
        target_branch: n.target_branch,
        participants: Vec::new(),
    };
    Ok(MrRow {
        mr,
        path_with_namespace: path.to_string(),
        updated_at: n.updated_at,
        approved: n.approved,
        approvals_left: n.approvals_left,
        resolvable_discussions: n.resolvable_discussions_count,
        resolved_discussions: n.resolved_discussions_count,
    })
}

fn map_issue(project_id: u64, path: &str, n: IssueNode) -> Result<IssueRow, String> {
    let iid: u64 = n.iid.parse().map_err(|_| format!("bad iid {}", n.iid))?;
    let issue = Issue {
        iid,
        project_id,
        title: n.title,
        description: None,
        author: author_or_ghost(&n.author)?,
        labels: labels(&n.labels),
        assignees: authors(&n.assignees)?,
        state: n.state.to_lowercase(),
        web_url: n.web_url,
        references: Some(References {
            full: format!("{path}#{iid}"),
        }),
        issue_type: None,
        milestone: milestone(&n.milestone),
        due_date: n.due_date,
        start_date: None,
        weight: None,
        task_completion_status: None,
        confidential: false,
        created_at: n.created_at,
        upvotes: None,
        downvotes: None,
        reactions: Vec::new(),
        participants: Vec::new(),
    };
    Ok(IssueRow {
        issue,
        path_with_namespace: path.to_string(),
        updated_at: n.updated_at,
        user_notes_count: n.user_notes_count,
    })
}

fn is_unsupported_message(m: &str) -> bool {
    let l = m.to_ascii_lowercase();
    l.contains("doesn't exist")
        || l.contains("does not exist")
        || l.contains("unknown field")
        || l.contains("unknown type")
        || l.contains("unknown argument")
        || l.contains("undefined field")
        || l.contains("isn't a defined input type")
        || l.contains("not defined")
}

struct Kind<N, T> {
    query: &'static str,
    field: &'static str,
    map: fn(u64, &str, N) -> Result<T, String>,
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
}

struct Page {
    data: Value,
    complete: bool,
    complexity: Option<Complexity>,
}

async fn post(
    client: &gitlab::AsyncGitlab,
    req: &Request,
    query: &str,
    ids: &[u64],
    after: Option<&str>,
) -> Result<Page, GraphqlError> {
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
        }
    });
    let http_req = http::Request::builder()
        .method("POST")
        .uri(req.url.as_str())
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::ACCEPT, "application/json");
    let resp = tokio::time::timeout(
        REQUEST_TIMEOUT,
        client.rest_async(http_req, body.to_string().into_bytes()),
    )
    .await
    .map_err(|_| GraphqlError::Failed("request timed out".into()))?
    .map_err(|e| GraphqlError::Failed(e.to_string()))?;
    let status = resp.status();
    if status == http::StatusCode::TOO_MANY_REQUESTS {
        return Err(GraphqlError::RateLimited);
    }
    if status == http::StatusCode::NOT_FOUND || status == http::StatusCode::BAD_REQUEST {
        return Err(GraphqlError::Unsupported);
    }
    if !status.is_success() {
        return Err(GraphqlError::Failed(format!("HTTP {status}")));
    }
    let env: Envelope =
        serde_json::from_slice(resp.body()).map_err(|_| GraphqlError::Unsupported)?;
    if env
        .errors
        .iter()
        .any(|e| is_unsupported_message(&e.message))
    {
        return Err(GraphqlError::Unsupported);
    }
    let Some(data) = env.data else {
        let msg = env
            .errors
            .first()
            .map(|e| e.message.clone())
            .unwrap_or_else(|| "empty response".into());
        return Err(GraphqlError::Failed(msg));
    };
    let complexity = data.get("queryComplexity").and_then(|c| {
        Some(Complexity {
            score: c.get("score")?.as_u64()?,
            limit: c.get("limit")?.as_u64()?,
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
}

fn project_nodes(data: &Value) -> Result<&Vec<Value>, GraphqlError> {
    data.get("projects")
        .and_then(|p| p.get("nodes"))
        .and_then(Value::as_array)
        .ok_or(GraphqlError::Unsupported)
}

fn merge_complexity(acc: &mut Option<Complexity>, c: Option<Complexity>) {
    if let Some(c) = c
        && acc.is_none_or(|a| c.score > a.score)
    {
        *acc = Some(c);
    }
}

type NextPage = Option<(u64, String)>;

fn absorb<N, T>(
    node: &Value,
    kind: &Kind<N, T>,
    out: &mut BatchOut<T>,
) -> Result<NextPage, GraphqlError>
where
    N: DeserializeOwned,
{
    if node.is_null() {
        return Ok(None);
    }
    let id = node
        .get("id")
        .and_then(Value::as_str)
        .ok_or(GraphqlError::Unsupported)?;
    let pid = gid_number(id).map_err(|_| GraphqlError::Unsupported)?;
    let path = node
        .get("fullPath")
        .and_then(Value::as_str)
        .ok_or(GraphqlError::Unsupported)?;
    let Some(conn) = node.get(kind.field).filter(|c| !c.is_null()) else {
        return Ok(None);
    };
    let conn: Connection<N> =
        serde_json::from_value(conn.clone()).map_err(|_| GraphqlError::Unsupported)?;
    for n in conn.nodes.into_iter().flatten() {
        out.rows
            .push((kind.map)(pid, path, n).map_err(|_| GraphqlError::Unsupported)?);
    }
    if conn.page_info.has_next_page
        && let Some(cursor) = conn.page_info.end_cursor
    {
        return Ok(Some((pid, cursor)));
    }
    Ok(None)
}

async fn run_batch<N, T>(
    client: gitlab::AsyncGitlab,
    req: Arc<Request>,
    kind: Kind<N, T>,
    ids: Vec<u64>,
) -> Result<BatchOut<T>, GraphqlError>
where
    N: DeserializeOwned,
{
    let mut out = BatchOut {
        rows: Vec::new(),
        complete: true,
        max_score: None,
    };
    let page = post(&client, &req, kind.query, &ids, None).await?;
    merge_complexity(&mut out.max_score, page.complexity);
    out.complete &= page.complete;
    let mut followups: Vec<(u64, String)> = Vec::new();
    for node in project_nodes(&page.data)? {
        if let Some(next) = absorb(node, &kind, &mut out)? {
            followups.push(next);
        }
    }
    for (pid, mut cursor) in followups {
        let mut pages = 1;
        loop {
            if pages >= MAX_PAGES_PER_PROJECT {
                out.complete = false;
                break;
            }
            pages += 1;
            let page = post(&client, &req, kind.query, &[pid], Some(&cursor)).await?;
            merge_complexity(&mut out.max_score, page.complexity);
            out.complete &= page.complete;
            let mut next = None;
            for node in project_nodes(&page.data)? {
                if let Some((_, n)) = absorb(node, &kind, &mut out)? {
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

async fn fetch<N, T>(
    client: &gitlab::AsyncGitlab,
    kind: Kind<N, T>,
    project_ids: &[u64],
    state: &str,
    since: Option<&str>,
) -> Result<ListResult<T>, GraphqlError>
where
    N: DeserializeOwned + Send + 'static,
    T: Send + 'static,
{
    let url = graphql_url(client)?;
    let batch = with_host(&url, |h| if h.unsupported { None } else { Some(h.batch) })
        .ok_or(GraphqlError::Unsupported)?;
    let req = Arc::new(Request {
        url: url.clone(),
        state: state.to_string(),
        since: since.map(str::to_string),
    });
    let sem = Arc::new(Semaphore::new(MAX_IN_FLIGHT));
    let mut set = JoinSet::new();
    for (idx, chunk) in project_ids.chunks(batch).enumerate() {
        let (client, req, sem, chunk) = (client.clone(), req.clone(), sem.clone(), chunk.to_vec());
        set.spawn(async move {
            let _permit = sem.acquire_owned().await;
            (idx, run_batch(client, req, kind, chunk).await)
        });
    }
    let mut results: Vec<(usize, Result<BatchOut<T>, GraphqlError>)> = Vec::new();
    while let Some(r) = set.join_next().await {
        match r {
            Ok(r) => results.push(r),
            Err(e) => {
                results.push((usize::MAX, Err(GraphqlError::Failed(e.to_string()))));
            }
        }
    }
    results.sort_by_key(|(i, _)| *i);

    let mut worst: Option<Complexity> = None;
    let mut fatal: Option<GraphqlError> = None;
    let mut rows = Vec::new();
    let mut complete = true;
    for (_, r) in results {
        match r {
            Ok(b) => {
                merge_complexity(&mut worst, b.max_score);
                complete &= b.complete;
                rows.extend(b.rows);
            }
            Err(GraphqlError::Unsupported) => fatal = Some(GraphqlError::Unsupported),
            Err(GraphqlError::RateLimited) => {
                if fatal != Some(GraphqlError::Unsupported) {
                    fatal = Some(GraphqlError::RateLimited);
                }
            }
            Err(e) => {
                tracing::warn!("graphql batch failed: {e}");
                complete = false;
            }
        }
    }
    with_host(&url, |h| {
        if let Some(c) = worst {
            h.last = Some(c);
            if c.score > COMPLEXITY_THRESHOLD {
                h.batch = match h.batch {
                    b if b > 10 => 10,
                    b if b > 5 => 5,
                    b => b,
                };
            }
        }
        if fatal == Some(GraphqlError::Unsupported) {
            h.unsupported = true;
        }
    });
    if let Some(e) = fatal {
        return Err(e);
    }
    Ok(ListResult { rows, complete })
}

pub async fn fetch_merge_requests(
    client: &gitlab::AsyncGitlab,
    project_ids: &[u64],
    state: &str,
    since: Option<&str>,
) -> Result<ListResult<MrRow>, GraphqlError> {
    let kind: Kind<MrNode, MrRow> = Kind {
        query: MR_QUERY,
        field: "mergeRequests",
        map: map_mr,
    };
    fetch(client, kind, project_ids, state, since).await
}

pub async fn fetch_issues(
    client: &gitlab::AsyncGitlab,
    project_ids: &[u64],
    state: &str,
    since: Option<&str>,
) -> Result<ListResult<IssueRow>, GraphqlError> {
    let kind: Kind<IssueNode, IssueRow> = Kind {
        query: ISSUE_QUERY,
        field: "issues",
        map: map_issue,
    };
    fetch(client, kind, project_ids, state, since).await
}
