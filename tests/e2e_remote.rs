use lazygitlab::models::App;
use gitlab::GitlabBuilder;
use dotenvy::dotenv;
use std::env;

#[tokio::test]
async fn test_remote_fetch_projects() {
    dotenv().ok();
    let token = match env::var("GITLAB_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Skipping test: GITLAB_TOKEN not set");
            return;
        }
    };
    let url = env::var("GITLAB_URL").unwrap_or_else(|_| "https://gitlab.com".to_string());

    let client = GitlabBuilder::new(url.replace("https://", "").replace("http://", ""), token)
        .build_async()
        .await
        .expect("Failed to create GitLab client");

    let mut app = App::new(client);
    
    // Test project fetching
    app.fetch_projects().await.expect("Failed to fetch projects");
    assert!(!app.projects.is_empty(), "Projects list should not be empty");

    // Try to find the test repository
    let test_repo = app.projects.iter().find(|p| p.path_with_namespace == "abletterer/lazygitlab-test");
    if let Some(project) = test_repo {
        println!("Found test repo: {}", project.id);
        app.selected_project = Some(project.clone());
        
        // Test fetching issues
        app.fetch_issues().await.expect("Failed to fetch issues");
        println!("Fetched {} issues", app.issues.len());

        // Test fetching MRs
        app.fetch_merge_requests().await.expect("Failed to fetch MRs");
        println!("Fetched {} MRs", app.merge_requests.len());

        // Test fetching pipelines
        app.fetch_pipelines().await.expect("Failed to fetch pipelines");
        println!("Fetched {} pipelines", app.pipelines.len());
    } else {
        println!("Test repo 'abletterer/lazygitlab-test' not found in visible projects");
    }
}
