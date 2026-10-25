use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use async_trait::async_trait;
use bytes::Bytes;
use gitlab::api::{ApiError, AsyncClient, RestClient};
use gitlab::{AsyncGitlab, RestError};

pub const ENV_VAR: &str = "LAZYGITLAB_TRACE";

/// Upper bound on trace lines per run (about 8 MB of log), so a long session cannot fill the disk.
pub const MAX_TRACE_LINES: u64 = 50_000;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Emit {
    Yes,
    Notice,
    No,
}

/// `written` is the number of trace lines already emitted: the line after the last allowed one is a single notice.
pub fn emit_allowed(written: u64, max: u64) -> Emit {
    match written {
        n if n < max => Emit::Yes,
        n if n == max => Emit::Notice,
        _ => Emit::No,
    }
}

static REQUESTS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
static LINES: AtomicU64 = AtomicU64::new(0);
static START: OnceLock<Instant> = OnceLock::new();
static ENABLED: OnceLock<bool> = OnceLock::new();

const KNOWN_SEGMENTS: &[&str] = &[
    "all",
    "approvals",
    "approve",
    "award_emoji",
    "blobs",
    "boards",
    "branches",
    "bridges",
    "cancel",
    "changes",
    "closed_by",
    "closes_issues",
    "commits",
    "diffs",
    "discussions",
    "epics",
    "erase",
    "files",
    "groups",
    "issues",
    "jobs",
    "labels",
    "links",
    "lists",
    "mark_as_done",
    "members",
    "merge",
    "merge_requests",
    "milestones",
    "notes",
    "participants",
    "pipeline_schedules",
    "pipelines",
    "play",
    "projects",
    "raw",
    "rebase",
    "related_merge_requests",
    "repository",
    "resolve",
    "retry",
    "search",
    "statuses",
    "subscribe",
    "tags",
    "test_report",
    "todos",
    "trace",
    "tree",
    "unapprove",
    "unsubscribe",
    "user",
    "users",
    "variables",
    "versions",
];

const CONTAINER_SEGMENTS: &[&str] = &["projects", "groups", "users"];

pub fn enabled() -> bool {
    *ENABLED.get_or_init(|| std::env::var(ENV_VAR).is_ok_and(|v| v == "1"))
}

pub fn totals() -> (u64, u64) {
    (
        REQUESTS.load(Ordering::Relaxed),
        BYTES.load(Ordering::Relaxed),
    )
}

pub fn mark_start() {
    START.get_or_init(Instant::now);
}

pub fn elapsed() -> std::time::Duration {
    START.get_or_init(Instant::now).elapsed()
}

pub fn template(path: &str) -> String {
    let rest = match path.find("/api/v4") {
        Some(i) => &path[i + "/api/v4".len()..],
        None => path,
    };
    let mut out = String::new();
    let mut after_container = false;
    for seg in rest.split('/').filter(|s| !s.is_empty()) {
        out.push('/');
        let numeric = seg.bytes().all(|b| b.is_ascii_digit());
        if numeric {
            out.push_str(":id");
            after_container = false;
        } else if after_container {
            out.push_str(":path");
            after_container = false;
        } else if KNOWN_SEGMENTS.contains(&seg) {
            out.push_str(seg);
            after_container = CONTAINER_SEGMENTS.contains(&seg);
        } else {
            out.push_str(":x");
        }
    }
    if out.is_empty() {
        out.push('/');
    }
    out
}

pub fn page_from_query(query: Option<&str>) -> Option<u64> {
    query?
        .split('&')
        .find_map(|kv| kv.strip_prefix("page="))
        .and_then(|v| v.parse().ok())
}

pub struct Record<'a> {
    pub method: &'a str,
    pub endpoint: &'a str,
    pub status: u16,
    pub ms: f64,
    pub bytes: usize,
    pub page: Option<u64>,
    pub not_modified: bool,
}

pub fn format_record(rec: &Record<'_>, t_ms: u64) -> String {
    serde_json::json!({
        "trace": 1,
        "t_ms": t_ms,
        "method": rec.method,
        "endpoint": rec.endpoint,
        "status": rec.status,
        "ms": (rec.ms * 10.0).round() / 10.0,
        "bytes": rec.bytes,
        "page": rec.page,
        "etag_304": rec.not_modified,
    })
    .to_string()
}

#[derive(Clone, Debug)]
pub struct TracedClient {
    inner: AsyncGitlab,
    trace: bool,
    bearer: Option<http::HeaderValue>,
}

impl From<AsyncGitlab> for TracedClient {
    fn from(inner: AsyncGitlab) -> Self {
        Self {
            inner,
            trace: enabled(),
            bearer: None,
        }
    }
}

impl TracedClient {
    pub fn with_trace(mut self, trace: bool) -> Self {
        self.trace = trace;
        self
    }

    /// Sends `Authorization: Bearer <token>` on every request. Used with an
    /// unauthenticated inner client, which skips the connection check the
    /// gitlab crate would otherwise run (a blocking `/user` request) at build time.
    pub fn with_bearer(mut self, token: &str) -> Result<Self, String> {
        let mut value = http::HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_| "token contains invalid characters".to_string())?;
        value.set_sensitive(true);
        self.bearer = Some(value);
        Ok(self)
    }
}

impl RestClient for TracedClient {
    type Error = RestError;

    fn rest_endpoint(&self, endpoint: &str) -> Result<url::Url, ApiError<Self::Error>> {
        self.inner.rest_endpoint(endpoint)
    }

    fn instance_endpoint(&self, endpoint: &str) -> Result<url::Url, ApiError<Self::Error>> {
        self.inner.instance_endpoint(endpoint)
    }
}

#[async_trait]
impl AsyncClient for TracedClient {
    async fn rest_async(
        &self,
        request: http::request::Builder,
        body: Vec<u8>,
    ) -> Result<http::Response<Bytes>, ApiError<<Self as RestClient>::Error>> {
        let request = match &self.bearer {
            Some(value) => request.header(http::header::AUTHORIZATION, value.clone()),
            None => request,
        };
        let meta = if self.trace {
            request.uri_ref().map(|u| {
                (
                    request
                        .method_ref()
                        .map_or_else(|| "GET".to_string(), |m| m.to_string()),
                    template(u.path()),
                    page_from_query(u.query()),
                )
            })
        } else {
            None
        };
        let start = Instant::now();
        let result = self.inner.rest_async(request, body).await;
        let elapsed = start.elapsed();
        let (status, len, response_page) = match &result {
            Ok(resp) => (
                resp.status().as_u16(),
                resp.body().len(),
                resp.headers()
                    .get("x-page")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok()),
            ),
            Err(_) => (0, 0, None),
        };
        REQUESTS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(len as u64, Ordering::Relaxed);
        if self.trace
            && let Some((method, endpoint, request_page)) = meta
        {
            let t_ms = START.get_or_init(Instant::now).elapsed().as_millis() as u64;
            let allowed = emit_allowed(LINES.fetch_add(1, Ordering::Relaxed), MAX_TRACE_LINES);
            if allowed == Emit::No {
                return result;
            }
            if allowed == Emit::Notice {
                tracing::info!(
                    "{}",
                    serde_json::json!({"trace": 2, "t_ms": t_ms, "notice": "trace stopped", "max_lines": MAX_TRACE_LINES})
                );
                return result;
            }
            let record = Record {
                method: &method,
                endpoint: &endpoint,
                status,
                ms: elapsed.as_secs_f64() * 1000.0,
                bytes: len,
                page: response_page.or(request_page),
                not_modified: status == 304,
            };
            tracing::info!("{}", format_record(&record, t_ms));
        }
        result
    }
}
