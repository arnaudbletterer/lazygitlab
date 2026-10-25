use lazygitlab::client::builder;
use lazygitlab::config::Transport;
use lazygitlab::http::{HttpLayer, Outcome};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

const TOKEN: &str = "glpat-requestLayer";

async fn layer(server: &MockServer) -> HttpLayer {
    Mock::given(path("/api/v4/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":1,"username":"u"})))
        .mount(server)
        .await;
    let client = builder(
        server.uri().replace("http://", ""),
        TOKEN.into(),
        Transport::PlainHttp,
    )
    .build_async()
    .await
    .expect("client");
    HttpLayer::new(client)
}

fn page_of(req: &Request) -> u64 {
    req.url
        .query_pairs()
        .find(|(k, _)| k == "page")
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(1)
}

struct Paged {
    total: u64,
    delay: Duration,
    total_header: bool,
    fail_page: Option<u64>,
    arrivals: Arc<Mutex<Vec<Instant>>>,
}

impl Respond for Paged {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        self.arrivals.lock().unwrap().push(Instant::now());
        let p = page_of(req);
        if Some(p) == self.fail_page {
            return ResponseTemplate::new(500).set_delay(self.delay);
        }
        let mut t = ResponseTemplate::new(200)
            .set_body_json(json!([p * 10, p * 10 + 1]))
            .set_delay(self.delay);
        if self.total_header {
            t = t.append_header("X-Total-Pages", self.total.to_string().as_str());
        }
        if p < self.total {
            t = t.append_header("X-Next-Page", (p + 1).to_string().as_str());
        }
        t
    }
}

async fn mount_paged(server: &MockServer, p: Paged) {
    Mock::given(method("GET"))
        .and(path("/api/v4/things"))
        .respond_with(p)
        .mount(server)
        .await;
}

fn paged(
    total: u64,
    delay_ms: u64,
    total_header: bool,
    fail_page: Option<u64>,
) -> (Paged, Arc<Mutex<Vec<Instant>>>) {
    let arrivals = Arc::new(Mutex::new(Vec::new()));
    (
        Paged {
            total,
            delay: Duration::from_millis(delay_ms),
            total_header,
            fail_page,
            arrivals: arrivals.clone(),
        },
        arrivals,
    )
}

#[tokio::test]
async fn pages_are_fetched_in_parallel_within_budget() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    let (p, arrivals) = paged(10, 100, true, None);
    mount_paged(&server, p).await;

    let start = Instant::now();
    let pages = http.get_all_pages::<u64>("things").await;
    let elapsed = start.elapsed();
    eprintln!("10 pages at 100 ms: {elapsed:?}");

    assert!(pages.complete);
    assert_eq!(pages.items.len(), 20);
    assert!(elapsed < Duration::from_millis(900), "{elapsed:?}");

    let mut times = arrivals.lock().unwrap().clone();
    times.sort();
    let window = Duration::from_millis(80);
    let max_in_window = times
        .iter()
        .map(|t| {
            times
                .iter()
                .filter(|u| **u >= *t && **u - *t < window)
                .count()
        })
        .max()
        .unwrap();
    assert!(max_in_window <= 6, "{max_in_window} in flight");
    assert!(max_in_window >= 2, "pages were not parallel");
}

#[tokio::test]
async fn missing_total_pages_falls_back_to_sequential() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    let (p, arrivals) = paged(4, 50, false, None);
    mount_paged(&server, p).await;

    let pages = http.get_all_pages::<u64>("things").await;
    assert!(pages.complete);
    assert_eq!(pages.items.len(), 8);
    let times = arrivals.lock().unwrap().clone();
    assert!(
        times
            .windows(2)
            .all(|w| w[1] - w[0] >= Duration::from_millis(45))
    );
}

#[tokio::test]
async fn failing_page_marks_incomplete_but_keeps_other_rows() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    let (p, _) = paged(5, 0, true, Some(3));
    mount_paged(&server, p).await;

    let pages = http.get_all_pages::<u64>("things").await;
    assert!(!pages.complete);
    assert_eq!(pages.items.len(), 8);
    assert!(!pages.items.contains(&30));
}

#[tokio::test]
async fn etag_sends_if_none_match_and_maps_304() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    Mock::given(method("GET"))
        .and(path("/api/v4/todos"))
        .and(header("if-none-match", "\"v1\""))
        .respond_with(ResponseTemplate::new(304))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/todos"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!([]))
                .append_header("ETag", "\"v1\""),
        )
        .mount(&server)
        .await;

    let Outcome::Ok(resp) = http.get("todos", None).await else {
        panic!("expected 200");
    };
    let etag = resp.etag().expect("etag");
    assert!(matches!(
        http.get("todos", Some(&etag)).await,
        Outcome::NotModified
    ));
}

#[tokio::test]
async fn identical_simultaneous_gets_share_one_request() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    Mock::given(method("GET"))
        .and(path("/api/v4/same"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!([1]))
                .set_delay(Duration::from_millis(200)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (a, b) = tokio::join!(http.get("same", None), http.get("same", None));
    assert!(matches!(a, Outcome::Ok(_)));
    assert!(matches!(b, Outcome::Ok(_)));
    server.verify().await;
}

struct FlakyThenOk(AtomicUsize);

impl Respond for FlakyThenOk {
    fn respond(&self, _: &Request) -> ResponseTemplate {
        if self.0.fetch_add(1, Ordering::SeqCst) < 2 {
            ResponseTemplate::new(500)
        } else {
            ResponseTemplate::new(200).set_body_json(json!([1]))
        }
    }
}

#[tokio::test]
async fn get_retries_after_two_server_errors() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    Mock::given(method("GET"))
        .and(path("/api/v4/flaky"))
        .respond_with(FlakyThenOk(AtomicUsize::new(0)))
        .expect(3)
        .mount(&server)
        .await;

    assert!(matches!(http.get("flaky", None).await, Outcome::Ok(_)));
    server.verify().await;
}

#[tokio::test]
async fn post_is_never_retried() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    Mock::given(method("POST"))
        .and(path("/api/v4/things"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&server)
        .await;

    let out = http
        .send_mutation(http::Method::POST, "things", b"{}".to_vec())
        .await;
    assert!(matches!(out, Outcome::Failed(_)));
    server.verify().await;
}

#[tokio::test]
async fn rate_limit_pauses_unrelated_requests() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    Mock::given(method("GET"))
        .and(path("/api/v4/limited"))
        .respond_with(ResponseTemplate::new(429).append_header("Retry-After", "1"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/other"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let start = Instant::now();
    let Outcome::RateLimited { until } = http.get("limited", None).await else {
        panic!("expected rate limit");
    };
    assert!(until > tokio::time::Instant::now());
    assert!(matches!(http.get("other", None).await, Outcome::Ok(_)));
    assert!(
        start.elapsed() >= Duration::from_millis(950),
        "{:?}",
        start.elapsed()
    );
}

#[tokio::test]
async fn requests_carry_bearer_token() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    Mock::given(method("GET"))
        .and(path("/api/v4/auth"))
        .and(header("authorization", format!("Bearer {TOKEN}").as_str()))
        .and(query_param("x", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(Value::Array(vec![])))
        .expect(1)
        .mount(&server)
        .await;

    assert!(matches!(http.get("auth?x=1", None).await, Outcome::Ok(_)));
    server.verify().await;
}

async fn mount_status(server: &MockServer, p: &str, t: ResponseTemplate) {
    Mock::given(method("GET"))
        .and(path(format!("/api/v4/{p}")))
        .respond_with(t)
        .mount(server)
        .await;
}

fn left(until: tokio::time::Instant) -> Duration {
    until.saturating_duration_since(tokio::time::Instant::now())
}

#[tokio::test]
async fn retry_after_is_capped_and_never_panics() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    mount_status(
        &server,
        "huge",
        ResponseTemplate::new(429).append_header("Retry-After", "18446744073709551615"),
    )
    .await;
    let Outcome::RateLimited { until } = http.get("huge", None).await else {
        panic!("expected rate limit");
    };
    assert!(left(until) <= Duration::from_secs(120));
}

#[tokio::test]
async fn retry_after_accepts_http_date() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    let when = (chrono::Utc::now() + chrono::Duration::seconds(3)).to_rfc2822();
    mount_status(
        &server,
        "dated",
        ResponseTemplate::new(429).append_header("Retry-After", when.as_str()),
    )
    .await;
    let Outcome::RateLimited { until } = http.get("dated", None).await else {
        panic!("expected rate limit");
    };
    assert!(left(until) <= Duration::from_secs(4), "{:?}", left(until));
}

#[tokio::test]
async fn unparseable_retry_after_defaults_to_ten_seconds() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    mount_status(
        &server,
        "junk",
        ResponseTemplate::new(429).append_header("Retry-After", "soon"),
    )
    .await;
    let Outcome::RateLimited { until } = http.get("junk", None).await else {
        panic!("expected rate limit");
    };
    let l = left(until);
    assert!(
        l > Duration::from_secs(8) && l <= Duration::from_secs(10),
        "{l:?}"
    );
}

#[tokio::test]
async fn mutation_fails_fast_while_gate_is_up() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    mount_status(
        &server,
        "limited",
        ResponseTemplate::new(429).append_header("Retry-After", "30"),
    )
    .await;
    assert!(matches!(
        http.get("limited", None).await,
        Outcome::RateLimited { .. }
    ));
    let start = Instant::now();
    let out = http
        .send_mutation(http::Method::POST, "things", b"{}".to_vec())
        .await;
    assert!(matches!(out, Outcome::RateLimited { .. }));
    assert!(start.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
async fn mutation_rejects_get() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    let out = http
        .send_mutation(http::Method::GET, "things", Vec::new())
        .await;
    assert!(matches!(out, Outcome::Failed(_)));
}

#[tokio::test]
async fn huge_total_pages_is_capped_and_incomplete() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    mount_status(
        &server,
        "things",
        ResponseTemplate::new(200)
            .set_body_json(json!([1]))
            .append_header("X-Total-Pages", "4294967295"),
    )
    .await;
    let pages = http.get_all_pages::<u64>("things").await;
    assert!(!pages.complete);
    assert_eq!(pages.items.len(), 200);
}

#[tokio::test]
async fn repeated_next_page_does_not_loop() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    mount_status(
        &server,
        "things",
        ResponseTemplate::new(200)
            .set_body_json(json!([1]))
            .append_header("X-Next-Page", "1"),
    )
    .await;
    let pages = http.get_all_pages::<u64>("things").await;
    assert!(!pages.complete);
    assert_eq!(pages.items.len(), 1);
}

struct Endless;

impl Respond for Endless {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        ResponseTemplate::new(200)
            .set_body_json(json!([1]))
            .append_header("X-Next-Page", (page_of(req) + 1).to_string().as_str())
    }
}

#[tokio::test]
async fn endless_next_page_stops_at_cap() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    Mock::given(method("GET"))
        .and(path("/api/v4/things"))
        .respond_with(Endless)
        .mount(&server)
        .await;
    let pages = http.get_all_pages::<u64>("things").await;
    assert!(!pages.complete);
    assert_eq!(pages.items.len(), 200);
}

#[tokio::test]
async fn cancelled_first_coalescer_does_not_block_the_next() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    mount_status(
        &server,
        "slow",
        ResponseTemplate::new(200)
            .set_body_json(json!([1]))
            .set_delay(Duration::from_millis(200)),
    )
    .await;
    let h2 = http.clone();
    let first = tokio::spawn(async move { h2.get("slow", None).await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    first.abort();
    let _ = first.await;
    assert!(matches!(http.get("slow", None).await, Outcome::Ok(_)));
}

struct LimitedPage3;

impl Respond for LimitedPage3 {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        if page_of(req) == 3 {
            return ResponseTemplate::new(429).append_header("Retry-After", "2");
        }
        ResponseTemplate::new(200)
            .set_body_json(json!([1]))
            .append_header("X-Total-Pages", "4")
    }
}

#[tokio::test]
async fn rate_limit_on_page_three_is_surfaced() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    Mock::given(method("GET"))
        .and(path("/api/v4/things"))
        .respond_with(LimitedPage3)
        .mount(&server)
        .await;
    let pages = http.get_all_pages::<u64>("things").await;
    assert!(!pages.complete);
    assert!(pages.rate_limited_until.is_some());
    assert_eq!(pages.items.len(), 3);
}

#[tokio::test]
async fn gitlab_error_message_is_kept() {
    let server = MockServer::start().await;
    let http = layer(&server).await;
    mount_status(
        &server,
        "nope",
        ResponseTemplate::new(404).set_body_json(json!({"message": "404 Project Not Found"})),
    )
    .await;
    let Outcome::Failed(msg) = http.get("nope", None).await else {
        panic!("expected failure");
    };
    assert!(msg.contains("404 Project Not Found"), "{msg}");
}
