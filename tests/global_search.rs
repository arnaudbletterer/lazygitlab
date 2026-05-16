use lazygitlab::models::{App, SearchResult};
use gitlab::GitlabBuilder;
use tokio::sync::mpsc;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{path, method, query_param, query_param_is_missing};

#[tokio::test]
async fn test_hybrid_search_logic() {
    let server = MockServer::start().await;
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    // Mock for version check (often used by Gitlab client)
    Mock::given(method("GET"))
        .and(path("/api/v4/version"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "version": "15.0.0", "revision": "rev" })))
        .mount(&server)
        .await;

    // Mock for current user
    Mock::given(method("GET"))
        .and(path("/api/v4/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "id": 1, "username": "u" })))
        .mount(&server)
        .await;

    // Mock for projects membership sync
    Mock::given(method("GET"))
        .and(path("/api/v4/projects"))
        .and(query_param("membership", "true"))
        .and(query_param_is_missing("search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": 1, "name": "Project 1" },
            { "id": 2, "name": "Project 2" }
        ])))
        .mount(&server)
        .await;

    // Mock for global search - projects
    Mock::given(method("GET"))
        .and(path("/api/v4/search"))
        .and(query_param("scope", "projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": 1, "name": "Project 1" },
            { "id": 999, "name": "Other" }
        ])))
        .mount(&server)
        .await;

    // Mock for global search - issues
    Mock::given(method("GET"))
        .and(path("/api/v4/search"))
        .and(query_param("scope", "issues"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { 
                "id": 10, "iid": 1, "project_id": 1, "title": "Member Issue", 
                "state": "opened", "web_url": "http://url", "author": { "id": 1, "username": "u" },
                "assignees": [], "labels": []
            }
        ])))
        .mount(&server)
        .await;

    // Mock for global search - merge_requests
    Mock::given(method("GET"))
        .and(path("/api/v4/search"))
        .and(query_param("scope", "merge_requests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { 
                "id": 20, "iid": 1, "project_id": 1, "title": "Member MR", 
                "state": "merged", "web_url": "http://url", "author": { "id": 1, "username": "u" },
                "assignees": [], "labels": []
            }
        ])))
        .mount(&server)
        .await;

    // Mock for direct scoped search (should return empty to not double results in this test)
    Mock::given(method("GET"))
        .and(path("/api/v4/issues"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/merge_requests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    // Mock for direct projects search
    Mock::given(method("GET"))
        .and(path("/api/v4/projects"))
        .and(query_param("membership", "true"))
        .and(query_param("search", "test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t")
        .insecure()
        .build_async()
        .await
        .unwrap();

    let mut app = App::new(client, tx);

    // 1. Trigger membership sync
    app.refresh().await;
    let start = std::time::Instant::now();
    while app.member_project_ids.is_empty() && start.elapsed().as_secs() < 5 {
        app.process_updates(&mut rx).await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(app.member_project_ids.contains(&1));

    // 2. Trigger search
    app.search_query = "test".to_string();
    app.trigger_global_search().await.unwrap();

    let start = std::time::Instant::now();
    while app.search_results.is_empty() && start.elapsed().as_secs() < 5 {
        app.process_updates(&mut rx).await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // 3. Verify results (should only contain the one in member projects)
    assert_eq!(app.search_results.len(), 3);
    
    let mut found_issue = false;
    let mut found_project = false;
    let mut found_mr = false;

    for res in &app.search_results {
        match res {
            SearchResult::Project(p) => { if p.id == 1 { found_project = true; } }
            SearchResult::Issue(i) => { if i.project_id == 1 { found_issue = true; } }
            SearchResult::MergeRequest(m) => { if m.project_id == 1 { found_mr = true; } }
        }
    }

    assert!(found_issue);
    assert!(found_project);
    assert!(found_mr);
}
