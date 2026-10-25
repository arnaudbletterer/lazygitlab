use std::io::Write;
use std::sync::{Arc, Mutex};

use gitlab::GitlabBuilder;
use gitlab::api::{AsyncClient, RestClient};
use lazygitlab::trace::{TracedClient, template};
use tracing_subscriber::fmt::MakeWriter;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TOKEN: &str = "glpat-sekrit-token-0123456789";
const PROJECT: &str = "acme%2Fsecret-proj";
const QUERY_VALUE: &str = "topsecretquery";

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Capture {
    type Writer = Capture;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

async fn mount_user(server: &MockServer) {
    Mock::given(path("/api/v4/user"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"})),
        )
        .mount(server)
        .await;
}

async fn get(client: &TracedClient, endpoint: &str, etag: Option<&str>) -> u16 {
    let url = client.rest_endpoint(endpoint).unwrap();
    let mut req = http::Request::builder().method("GET").uri(url.as_str());
    if let Some(etag) = etag {
        req = req.header(http::header::IF_NONE_MATCH, etag);
    }
    client
        .rest_async(req, Vec::new())
        .await
        .unwrap()
        .status()
        .as_u16()
}

#[tokio::test]
async fn trace_lines_carry_no_host_token_project_or_query_value() {
    let server = MockServer::start().await;
    mount_user(&server).await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v4/projects/{PROJECT}/merge_requests")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("X-Page", "2")
                .set_body_string("[]"),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/todos"))
        .respond_with(ResponseTemplate::new(304))
        .mount(&server)
        .await;

    let host = server.uri().replace("http://", "");
    let client: TracedClient = GitlabBuilder::new(host.clone(), TOKEN)
        .insecure()
        .build_async()
        .await
        .unwrap()
        .into();
    let client = client.with_trace(true);

    let capture = Capture::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(capture.clone())
        .with_ansi(false)
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let endpoint = format!("projects/{PROJECT}/merge_requests?search={QUERY_VALUE}&page=2");
    assert_eq!(get(&client, &endpoint, None).await, 200);
    assert_eq!(
        get(&client, "todos?state=pending", Some("\"abc\"")).await,
        304
    );

    let out = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
    let lines: Vec<&str> = out.lines().filter(|l| l.contains("\"trace\":1")).collect();
    assert_eq!(lines.len(), 2, "unexpected trace output: {out}");
    assert!(out.contains("\"endpoint\":\"/projects/:path/merge_requests\""));
    assert!(out.contains("\"endpoint\":\"/todos\""));
    assert!(out.contains("\"page\":2"));
    assert!(out.contains("\"etag_304\":true"));

    let port = host.rsplit(':').next().unwrap();
    for forbidden in [
        host.as_str(),
        "127.0.0.1",
        port,
        TOKEN,
        "sekrit",
        "secret-proj",
        "acme",
        QUERY_VALUE,
        "pending",
        "abc",
    ] {
        assert!(
            !out.contains(forbidden),
            "trace output leaked {forbidden:?}: {out}"
        );
    }
}

#[tokio::test]
async fn trace_is_silent_unless_enabled() {
    let server = MockServer::start().await;
    mount_user(&server).await;
    Mock::given(path("/api/v4/todos"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[]"))
        .mount(&server)
        .await;
    let client: TracedClient = GitlabBuilder::new(server.uri().replace("http://", ""), TOKEN)
        .insecure()
        .build_async()
        .await
        .unwrap()
        .into();
    let client = client.with_trace(false);

    let capture = Capture::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(capture.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);
    get(&client, "todos", None).await;
    let out = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
    assert!(!out.contains("\"trace\":1"));
}

#[test]
fn templates_replace_ids_and_unknown_segments() {
    assert_eq!(
        template("/api/v4/projects/42/merge_requests/7/discussions"),
        "/projects/:id/merge_requests/:id/discussions"
    );
    assert_eq!(
        template("/api/v4/projects/acme%2Fsecret-proj/pipelines"),
        "/projects/:path/pipelines"
    );
    assert_eq!(
        template("/gitlab/api/v4/projects/9/repository/branches/feat%2Fx"),
        "/projects/:id/repository/branches/:x"
    );
    assert_eq!(template("/api/v4/user"), "/user");
    assert_eq!(
        template("/api/v4/groups/team/projects"),
        "/groups/:path/projects"
    );
}
