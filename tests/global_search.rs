use lazygitlab::models::{App, SearchResult};
use gitlab::GitlabBuilder;
use tokio::sync::mpsc;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{path, method, query_param};

#[tokio::test]
async fn test_global_search_logic() {
    let server = MockServer::start().await;

    // Mock for user (required by App::new logic usually)
    Mock::given(method("GET"))
        .and(path("/api/v4/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "id": 1, "username": "abletterer" })))
        .mount(&server)
        .await;
    
    // Mock for projects search
    Mock::given(method("GET"))
        .and(path("/api/v4/search"))
        .and(query_param("scope", "projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": 1, "name": "Test Project", "path_with_namespace": "ns/test-proj" }
        ])))
        .mount(&server)
        .await;

    // Mock for issues search
    Mock::given(method("GET"))
        .and(path("/api/v4/search"))
        .and(query_param("scope", "issues"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { 
                "id": 10, "iid": 1, "project_id": 1, "title": "Test Issue", 
                "state": "opened", "web_url": "http://url", "author": { "id": 1, "username": "u" },
                "assignees": [], "labels": []
            }
        ])))
        .mount(&server)
        .await;

    // Mock for MRs search
    Mock::given(method("GET"))
        .and(path("/api/v4/search"))
        .and(query_param("scope", "merge_requests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { 
                "id": 20, "iid": 2, "project_id": 1, "title": "Test MR", 
                "state": "opened", "web_url": "http://url", "author": { "id": 1, "username": "u" },
                "target_branch": "main", "source_branch": "feat", "draft": false, "labels": [],
                "created_at": "2023-01-01T00:00:00Z", "updated_at": "2023-01-01T00:00:00Z",
                "assignees": []
            }
        ])))
        .mount(&server)
        .await;

    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t")
        .insecure()
        .build_async()
        .await
        .unwrap();

    let mut app = App::new(client);
    let (tx, mut rx) = mpsc::unbounded_channel();
    app.tx = tx;

    app.search_query = "test".to_string();
    app.trigger_global_search().await.unwrap();

    // Process updates
    let start = std::time::Instant::now();
    while app.search_results.len() < 3 && start.elapsed().as_secs() < 10 {
        app.process_updates(&mut rx).await;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    println!("Results found: {}", app.search_results.len());
    for (i, res) in app.search_results.iter().enumerate() {
        println!("Result {}: {:?}", i, res);
    }

    assert_eq!(app.search_results.len(), 3);
    
    let has_project = app.search_results.iter().any(|r| matches!(r, SearchResult::Project(_)));
    let has_issue = app.search_results.iter().any(|r| matches!(r, SearchResult::Issue(_)));
    let has_mr = app.search_results.iter().any(|r| matches!(r, SearchResult::MergeRequest(_)));

    assert!(has_project);
    assert!(has_issue);
    assert!(has_mr);
}
