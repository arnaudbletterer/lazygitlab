use gitlab::GitlabBuilder;
use lazygitlab::{App, Context};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn app_for(server: &MockServer) -> App {
    Mock::given(method("GET"))
        .and(path("/api/v4/user"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"})),
        )
        .mount(server)
        .await;
    let client = GitlabBuilder::new(server.uri().replace("http://", ""), "t")
        .insecure()
        .build_async()
        .await
        .unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    App::new(client, tx, None)
}

fn project_body() -> serde_json::Value {
    serde_json::json!({
        "id": 7,
        "name": "repo",
        "path_with_namespace": "group/repo",
        "web_url": "https://gitlab.example/group/repo",
        "default_branch": "main"
    })
}

#[tokio::test]
async fn preselect_by_path_sets_project_context() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/group%2Frepo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_body()))
        .mount(&server)
        .await;
    let mut app = app_for(&server).await;
    app.preselect_project("group/repo").await.unwrap();
    assert_eq!(app.selected_project.as_ref().map(|p| p.id), Some(7));
    assert_eq!(app.active_context, Context::Project);
}

#[tokio::test]
async fn preselect_by_id_works() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_body()))
        .mount(&server)
        .await;
    let mut app = app_for(&server).await;
    app.preselect_project("7").await.unwrap();
    assert_eq!(app.selected_project.as_ref().map(|p| p.id), Some(7));
}

#[tokio::test]
async fn preselect_unknown_project_reports_error_and_keeps_dashboard() {
    let server = MockServer::start().await;
    let mut app = app_for(&server).await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/nope%2Fnone"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({"message": "404 Project Not Found"})),
        )
        .mount(&server)
        .await;
    let err = app.preselect_project("nope/none").await.unwrap_err();
    assert!(err.contains("nope/none"));
    assert!(app.selected_project.is_none());
}
