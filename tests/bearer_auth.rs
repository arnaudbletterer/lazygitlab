use gitlab::api::{AsyncQuery, Endpoint};
use lazygitlab::client::builder;
use lazygitlab::config::Transport;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const TOKEN: &str = "glpat-bearerTestToken123";

fn host(server: &MockServer) -> String {
    server.uri().replace("http://", "")
}

fn user_ok() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))
}

#[tokio::test]
async fn requests_use_bearer_and_never_private_token() {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user"))
        .and(header("authorization", format!("Bearer {TOKEN}").as_str()))
        .respond_with(user_ok())
        .expect(3)
        .mount(&server)
        .await;
    let client = builder(host(&server), TOKEN.into(), Transport::PlainHttp)
        .build_async()
        .await
        .expect("client");
    let _: serde_json::Value = lazygitlab::CurrentUser
        .query_async(&client)
        .await
        .expect("query");

    let req = http::Request::get(format!("{}/api/v4/user", server.uri()).as_str());
    let _ = gitlab::api::AsyncClient::rest_async(&client, req, Vec::new()).await;

    for r in server.received_requests().await.unwrap() {
        assert!(!r.headers.contains_key("private-token"), "{:?}", r.headers);
        assert!(r.headers.contains_key("authorization"));
    }
    let _ = lazygitlab::CurrentUser.method();
}

#[tokio::test]
async fn cross_host_redirect_does_not_leak_the_token() {
    let target = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(user_ok())
        .mount(&target)
        .await;
    let origin = MockServer::start().await;
    let location = format!("{}/steal", target.uri());
    Mock::given(path("/api/v4/user"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", location.as_str()))
        .mount(&origin)
        .await;

    let client = builder(host(&origin), TOKEN.into(), Transport::PlainHttp)
        .build_async()
        .await;
    drop(client);

    let seen: Vec<Request> = target.received_requests().await.unwrap();
    assert!(!seen.is_empty(), "redirect was not followed");
    for r in seen {
        assert!(!r.headers.contains_key("private-token"), "{:?}", r.headers);
        assert!(!r.headers.contains_key("authorization"), "{:?}", r.headers);
        let dump = format!("{:?}", r.headers);
        assert!(!dump.contains(TOKEN), "{dump}");
    }
}
