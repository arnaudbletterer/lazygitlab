use gitlab::AsyncGitlab;
use gitlab::api::{AsyncClient, RestClient};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::hash::{BuildHasher, RandomState};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{OnceCell, Semaphore};
use tokio::time::Instant;

pub const MAX_IN_FLIGHT: usize = 6;
pub const MAX_ATTEMPTS: u32 = 3;
pub const RETRY_BASE: Duration = Duration::from_millis(200);
pub const DEFAULT_RETRY_AFTER: Duration = Duration::from_secs(10);
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
pub const PER_PAGE: u32 = 100;

#[derive(Debug, Clone)]
pub struct Resp {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub body: Vec<u8>,
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
    client: AsyncGitlab,
    budget: Semaphore,
    gate: Mutex<Option<Instant>>,
    inflight: Mutex<HashMap<String, Arc<OnceCell<Outcome>>>>,
}

/// Request layer for one GitLab host: the per-host budget and the 429 gate live here.
#[derive(Clone)]
pub struct HttpLayer {
    inner: Arc<Inner>,
}

fn jitter(cap: Duration) -> Duration {
    let cap_ms = cap.as_millis().max(1) as u64;
    Duration::from_millis(RandomState::new().hash_one(0u8) % cap_ms)
}

fn retry_after(resp: &Resp) -> Duration {
    resp.header("retry-after")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_RETRY_AFTER)
}

impl HttpLayer {
    pub fn new(client: AsyncGitlab) -> Self {
        Self {
            inner: Arc::new(Inner {
                client,
                budget: Semaphore::new(MAX_IN_FLIGHT),
                gate: Mutex::new(None),
                inflight: Mutex::new(HashMap::new()),
            }),
        }
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
        method: &str,
        url: &str,
        etag: Option<&str>,
        body: Vec<u8>,
    ) -> Result<Resp, String> {
        let _permit = loop {
            while let Some(until) = self.gate_until() {
                tokio::time::sleep_until(until).await;
            }
            let permit = self
                .inner
                .budget
                .acquire()
                .await
                .map_err(|e| e.to_string())?;
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
        let resp = tokio::time::timeout(REQUEST_TIMEOUT, self.inner.client.rest_async(req, body))
            .await
            .map_err(|_| "request timed out".to_string())?
            .map_err(|e| e.to_string())?;
        let (parts, body) = resp.into_parts();
        Ok(Resp {
            status: parts.status,
            headers: parts.headers,
            body: body.to_vec(),
        })
    }

    async fn execute(&self, method: &str, url: &str, etag: Option<&str>, body: Vec<u8>) -> Outcome {
        let attempts = if method == "GET" { MAX_ATTEMPTS } else { 1 };
        let mut last = String::new();
        for attempt in 0..attempts {
            if attempt > 0 {
                tokio::time::sleep(jitter(RETRY_BASE * 2u32.pow(attempt - 1))).await;
            }
            match self.send(method, url, etag, body.clone()).await {
                Ok(resp) if resp.status == http::StatusCode::NOT_MODIFIED => {
                    return Outcome::NotModified;
                }
                Ok(resp) if resp.status == http::StatusCode::TOO_MANY_REQUESTS => {
                    let until = Instant::now() + retry_after(&resp);
                    self.raise_gate(until);
                    return Outcome::RateLimited { until };
                }
                Ok(resp) if resp.status.is_server_error() => {
                    last = format!("server error: {}", resp.status);
                }
                Ok(resp) if resp.status.is_success() => return Outcome::Ok(resp),
                Ok(resp) => return Outcome::Failed(format!("request failed: {}", resp.status)),
                Err(e) => last = e,
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
        let cell = {
            let mut map = self
                .inner
                .inflight
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            map.entry(key.clone()).or_default().clone()
        };
        let outcome = cell
            .get_or_init(|| self.execute("GET", &url, etag, Vec::new()))
            .await
            .clone();
        let mut map = self
            .inner
            .inflight
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if map.get(&key).is_some_and(|c| Arc::ptr_eq(c, &cell)) {
            map.remove(&key);
        }
        outcome
    }

    /// Mutation (POST, PUT, DELETE...): never retried, never coalesced.
    pub async fn send_mutation(&self, method: &str, path: &str, body: Vec<u8>) -> Outcome {
        match self.url(path) {
            Ok(url) => self.execute(method, &url, None, body).await,
            Err(e) => Outcome::Failed(e),
        }
    }

    /// Fetches every page of a list endpoint. `complete` is false if any page failed.
    pub async fn get_all_pages<T>(&self, path: &str) -> Pages<T>
    where
        T: DeserializeOwned,
    {
        let sep = if path.contains('?') { '&' } else { '?' };
        let page_path = |n: u32| format!("{path}{sep}per_page={PER_PAGE}&page={n}");
        let mut out = Pages {
            items: Vec::new(),
            complete: true,
            rate_limited_until: None,
        };
        let first = match self.get(&page_path(1), None).await {
            Outcome::Ok(resp) => resp,
            other => {
                out.fail(&other);
                return out;
            }
        };
        out.absorb(&first);

        if let Some(total) = first.page_header("x-total-pages") {
            let tasks: Vec<_> = (2..=total)
                .map(|n| {
                    let layer = self.clone();
                    let p = page_path(n);
                    tokio::spawn(async move { layer.get(&p, None).await })
                })
                .collect();
            for task in tasks {
                match task.await {
                    Ok(Outcome::Ok(resp)) => {
                        out.absorb(&resp);
                    }
                    Ok(other) => out.fail(&other),
                    Err(e) => out.fail(&Outcome::Failed(e.to_string())),
                }
            }
            return out;
        }

        let mut next = first.page_header("x-next-page");
        while let Some(n) = next {
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
        out
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
