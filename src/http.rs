use crate::trace::TracedClient;
use gitlab::api::{AsyncClient, RestClient};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::hash::{BuildHasher, RandomState};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{Notify, Semaphore, watch};
use tokio::task::JoinSet;
use tokio::time::Instant;

pub use ::http::Method;

pub const MAX_IN_FLIGHT: usize = 6;
pub const MAX_ATTEMPTS: u32 = 3;
pub const RETRY_BASE: Duration = Duration::from_millis(200);
pub const DEFAULT_RETRY_AFTER: Duration = Duration::from_secs(10);
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
pub const PER_PAGE: u32 = 100;
pub const MAX_PAGES: u32 = 200;
pub const MAX_RETRY_AFTER: Duration = Duration::from_secs(120);
pub const MAX_BODY_BYTES: usize = 32 * 1024 * 1024;
pub const PAGES_DEADLINE: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct Resp {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub body: Arc<[u8]>,
}

impl Resp {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    pub fn etag(&self) -> Option<String> {
        self.header("etag").map(str::to_string)
    }

    fn page_header(&self, name: &str) -> Option<u32> {
        self.header(name).and_then(|v| v.trim().parse().ok())
    }
}

/// Text for the status line when the server asks the client to back off.
pub fn rate_limit_text(until: Instant) -> String {
    let secs = until
        .saturating_duration_since(Instant::now())
        .as_secs()
        .max(1);
    rate_limit_message(secs)
}

pub fn rate_limit_message(secs: u64) -> String {
    format!("Rate limited by GitLab, retrying in {secs}s")
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Ok(Resp),
    NotModified,
    RateLimited { until: Instant },
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct Pages<T> {
    pub items: Vec<T>,
    pub complete: bool,
    pub rate_limited_until: Option<Instant>,
}

struct Inner {
    client: TracedClient,
    budget: Semaphore,
    gate: Mutex<Option<Instant>>,
    inflight: Mutex<HashMap<String, Inflight>>,
}

/// One coalesced GET. The task that runs it stops when its last waiter goes away,
/// so an aborted caller does not keep a budget permit for the whole timeout.
struct Inflight {
    rx: watch::Receiver<Option<Outcome>>,
    waiters: usize,
    abandon: Arc<Notify>,
}

/// Request layer for one GitLab host: the per-host budget and the 429 gate live here.
/// The host is trusted: the gitlab client buffers each response before `MAX_BODY_BYTES` is checked.
#[derive(Clone)]
pub struct HttpLayer {
    inner: Arc<Inner>,
    pages_deadline: Duration,
}

fn jitter(cap: Duration) -> Duration {
    let cap_ms = cap.as_millis().max(1) as u64;
    Duration::from_millis(RandomState::new().hash_one(0u8) % cap_ms)
}

/// Only delta-seconds and RFC 2822 HTTP-dates (`Wed, 21 Oct 2015 07:28:00 GMT`) are parsed.
fn retry_after(resp: &Resp) -> Duration {
    let parsed = resp.header("retry-after").and_then(|v| {
        let v = v.trim();
        if let Ok(secs) = v.parse::<u64>() {
            return Some(Duration::from_secs(secs));
        }
        let date = chrono::DateTime::parse_from_rfc2822(v).ok()?;
        let delta = date.with_timezone(&chrono::Utc) - chrono::Utc::now();
        Some(delta.to_std().unwrap_or(Duration::ZERO))
    });
    parsed.unwrap_or(DEFAULT_RETRY_AFTER).min(MAX_RETRY_AFTER)
}

fn error_message(resp: &Resp) -> String {
    let detail = serde_json::from_slice::<serde_json::Value>(&resp.body)
        .ok()
        .and_then(|v| {
            ["message", "error"].iter().find_map(|k| {
                v.get(k)
                    .map(|m| m.as_str().map_or_else(|| m.to_string(), str::to_string))
            })
        })
        .unwrap_or_default();
    let raw = format!("request failed ({}) {}", resp.status, detail);
    crate::errors::friendly_error(raw.trim())
}

enum SendError {
    Gated(Instant),
    Other(String),
}

impl HttpLayer {
    pub fn new(client: impl Into<TracedClient>) -> Self {
        Self {
            inner: Arc::new(Inner {
                client: client.into(),
                budget: Semaphore::new(MAX_IN_FLIGHT),
                gate: Mutex::new(None),
                inflight: Mutex::new(HashMap::new()),
            }),
            pages_deadline: PAGES_DEADLINE,
        }
    }

    pub fn with_pages_deadline(mut self, deadline: Duration) -> Self {
        self.pages_deadline = deadline;
        self
    }

    fn url(&self, path: &str) -> Result<String, String> {
        self.inner
            .client
            .rest_endpoint(path)
            .map(|u| u.to_string())
            .map_err(|e| e.to_string())
    }

    fn gate_until(&self) -> Option<Instant> {
        let mut gate = self.inner.gate.lock().unwrap_or_else(|e| e.into_inner());
        match *gate {
            Some(t) if t > Instant::now() => Some(t),
            _ => {
                *gate = None;
                None
            }
        }
    }

    fn raise_gate(&self, until: Instant) {
        let mut gate = self.inner.gate.lock().unwrap_or_else(|e| e.into_inner());
        if gate.is_none_or(|t| t < until) {
            *gate = Some(until);
        }
    }

    /// Single choke point: every upstream request goes through here, once per attempt.
    async fn send(
        &self,
        method: &http::Method,
        url: &str,
        etag: Option<&str>,
        body: Vec<u8>,
        fail_fast: bool,
    ) -> Result<Resp, SendError> {
        async {
            let _permit = loop {
                while let Some(until) = self.gate_until() {
                    if fail_fast {
                        return Err(SendError::Gated(until));
                    }
                    tokio::time::sleep_until(until).await;
                }
                let permit = self
                    .inner
                    .budget
                    .acquire()
                    .await
                    .map_err(|e| SendError::Other(e.to_string()))?;
                if self.gate_until().is_none() {
                    break permit;
                }
            };
            let mut req = http::Request::builder().method(method).uri(url);
            if let Some(etag) = etag {
                req = req.header(http::header::IF_NONE_MATCH, etag);
            }
            if !body.is_empty() {
                req = req.header(http::header::CONTENT_TYPE, "application/json");
            }
            let resp =
                tokio::time::timeout(REQUEST_TIMEOUT, self.inner.client.rest_async(req, body))
                    .await
                    .map_err(|_| SendError::Other("request timed out".to_string()))?
                    .map_err(|e| SendError::Other(e.to_string()))?;
            let (parts, body) = resp.into_parts();
            if body.len() > MAX_BODY_BYTES {
                return Err(SendError::Other("response body too large".to_string()));
            }
            Ok(Resp {
                status: parts.status,
                headers: parts.headers,
                body: Arc::from(&body[..]),
            })
        }
        .await
    }

    async fn execute(
        &self,
        method: &http::Method,
        url: &str,
        etag: Option<&str>,
        body: Vec<u8>,
    ) -> Outcome {
        let idempotent = method == http::Method::GET;
        let attempts = if idempotent { MAX_ATTEMPTS } else { 1 };
        let mut last = String::new();
        for attempt in 0..attempts {
            if attempt > 0 {
                tokio::time::sleep(jitter(RETRY_BASE * 2u32.pow(attempt - 1))).await;
            }
            match self
                .send(method, url, etag, body.clone(), !idempotent)
                .await
            {
                Ok(resp) if resp.status == http::StatusCode::NOT_MODIFIED => {
                    return Outcome::NotModified;
                }
                Ok(resp) if resp.status == http::StatusCode::TOO_MANY_REQUESTS => {
                    let until = Instant::now() + retry_after(&resp);
                    self.raise_gate(until);
                    return Outcome::RateLimited { until };
                }
                Ok(resp) if resp.status.is_server_error() => last = error_message(&resp),
                Ok(resp) if resp.status.is_success() => return Outcome::Ok(resp),
                Ok(resp) => return Outcome::Failed(error_message(&resp)),
                Err(SendError::Gated(until)) => return Outcome::RateLimited { until },
                Err(SendError::Other(e)) => last = crate::errors::friendly_error(&e),
            }
        }
        Outcome::Failed(last)
    }

    /// Conditional, coalesced, retried GET of an API path (relative to `/api/v4/`).
    pub async fn get(&self, path: &str, etag: Option<&str>) -> Outcome {
        let url = match self.url(path) {
            Ok(u) => u,
            Err(e) => return Outcome::Failed(e),
        };
        let key = format!("{}\n{}", url, etag.unwrap_or(""));
        let (mut rx, token) = {
            let mut map = self
                .inner
                .inflight
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(entry) = map.get_mut(&key) {
                entry.waiters += 1;
                (entry.rx.clone(), entry.abandon.clone())
            } else {
                let (tx, rx) = watch::channel(None);
                let abandon = Arc::new(Notify::new());
                map.insert(
                    key.clone(),
                    Inflight {
                        rx: rx.clone(),
                        waiters: 1,
                        abandon: abandon.clone(),
                    },
                );
                let layer = self.clone();
                let etag = etag.map(str::to_string);
                let task_token = abandon.clone();
                let task_key = key.clone();
                tokio::spawn(async move {
                    let _cleanup = InflightCleanup {
                        layer: &layer,
                        key: task_key,
                        token: task_token.clone(),
                    };
                    tokio::select! {
                        outcome = layer.execute(&http::Method::GET, &url, etag.as_deref(), Vec::new()) => {
                            let _ = tx.send(Some(outcome));
                        }
                        _ = task_token.notified() => {}
                    }
                });
                (rx, abandon)
            }
        };
        let _waiter = Waiter {
            layer: self,
            key,
            token,
        };
        match rx.wait_for(|v| v.is_some()).await {
            Ok(v) => v
                .clone()
                .unwrap_or(Outcome::Failed("request failed".to_string())),
            Err(_) => Outcome::Failed("request task ended unexpectedly".to_string()),
        }
    }

    /// Mutation (POST, PUT, DELETE...): never retried, never coalesced, fails fast while rate limited.
    pub async fn send_mutation(&self, method: http::Method, path: &str, body: Vec<u8>) -> Outcome {
        if method == http::Method::GET || method == http::Method::HEAD {
            return Outcome::Failed("send_mutation does not accept GET or HEAD".to_string());
        }
        match self.url(path) {
            Ok(url) => self.execute(&method, &url, None, body).await,
            Err(e) => Outcome::Failed(e),
        }
    }

    /// Fetches every page of a list endpoint. `complete` is false if any page failed
    /// or the overall deadline passed; rows fetched so far are still returned.
    pub async fn get_all_pages<T>(&self, path: &str) -> Pages<T>
    where
        T: DeserializeOwned,
    {
        self.get_pages_capped(path, MAX_PAGES).await
    }

    /// Like `get_all_pages` but stops after `max_pages` pages. Hitting a cap below
    /// `MAX_PAGES` is a deliberate window, so it does not make the result incomplete.
    pub async fn get_pages_capped<T>(&self, path: &str, max_pages: u32) -> Pages<T>
    where
        T: DeserializeOwned,
    {
        let max_pages = max_pages.clamp(1, MAX_PAGES);
        let mut out = Pages {
            items: Vec::new(),
            complete: true,
            rate_limited_until: None,
        };
        let mut fetched: Vec<(u32, Outcome)> = Vec::new();
        let run = self.fill_pages(path, max_pages, &mut out, &mut fetched);
        if tokio::time::timeout(self.pages_deadline, run)
            .await
            .is_err()
        {
            out.complete = false;
        }
        fetched.sort_by_key(|(n, _)| *n);
        for (_, outcome) in fetched {
            match outcome {
                Outcome::Ok(resp) => {
                    out.absorb(&resp);
                }
                other => out.fail(&other),
            }
        }
        out
    }

    async fn fill_pages<T: DeserializeOwned>(
        &self,
        path: &str,
        max_pages: u32,
        out: &mut Pages<T>,
        fetched: &mut Vec<(u32, Outcome)>,
    ) {
        let sep = if path.contains('?') { '&' } else { '?' };
        let page_path = |n: u32| format!("{path}{sep}per_page={PER_PAGE}&page={n}");
        let first = match self.get(&page_path(1), None).await {
            Outcome::Ok(resp) => resp,
            other => {
                out.fail(&other);
                return;
            }
        };
        out.absorb(&first);

        if let Some(total) = first.page_header("x-total-pages") {
            let last_page = if total > max_pages {
                if max_pages >= MAX_PAGES {
                    out.complete = false;
                }
                max_pages
            } else {
                total
            };
            let mut tasks = JoinSet::new();
            for n in 2..=last_page {
                let layer = self.clone();
                let p = page_path(n);
                tasks.spawn(async move { (n, layer.get(&p, None).await) });
            }
            while let Some(joined) = tasks.join_next().await {
                match joined {
                    Ok(pair) => fetched.push(pair),
                    Err(e) => out.fail(&Outcome::Failed(e.to_string())),
                }
            }
            return;
        }

        let mut current = 1;
        let mut next = first.page_header("x-next-page");
        while let Some(n) = next {
            if current >= max_pages && max_pages < MAX_PAGES {
                break;
            }
            if n <= current || current >= MAX_PAGES {
                out.complete = false;
                break;
            }
            current = n;
            match self.get(&page_path(n), None).await {
                Outcome::Ok(resp) => {
                    next = resp.page_header("x-next-page");
                    if !out.absorb(&resp) {
                        break;
                    }
                }
                other => {
                    out.fail(&other);
                    break;
                }
            }
        }
    }
}

struct InflightCleanup<'a> {
    layer: &'a HttpLayer,
    key: String,
    token: Arc<Notify>,
}

impl Drop for InflightCleanup<'_> {
    fn drop(&mut self) {
        let mut map = self
            .layer
            .inner
            .inflight
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if map
            .get(&self.key)
            .is_some_and(|e| Arc::ptr_eq(&e.abandon, &self.token))
        {
            map.remove(&self.key);
        }
    }
}

/// One caller waiting on a coalesced GET. The last one to leave stops the request.
struct Waiter<'a> {
    layer: &'a HttpLayer,
    key: String,
    token: Arc<Notify>,
}

impl Drop for Waiter<'_> {
    fn drop(&mut self) {
        let mut map = self
            .layer
            .inner
            .inflight
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let Some(entry) = map.get_mut(&self.key) else {
            return;
        };
        if !Arc::ptr_eq(&entry.abandon, &self.token) {
            return;
        }
        entry.waiters -= 1;
        if entry.waiters == 0 {
            map.remove(&self.key);
            self.token.notify_one();
        }
    }
}

impl<T: DeserializeOwned> Pages<T> {
    fn fail(&mut self, outcome: &Outcome) {
        self.complete = false;
        if let Outcome::RateLimited { until } = outcome {
            self.rate_limited_until = Some(*until);
        }
    }

    fn absorb(&mut self, resp: &Resp) -> bool {
        match serde_json::from_slice::<Vec<T>>(&resp.body) {
            Ok(items) => {
                self.items.extend(items);
                true
            }
            Err(_) => {
                self.complete = false;
                false
            }
        }
    }
}
