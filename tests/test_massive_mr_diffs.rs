use lazygitlab::models::{App, MergeRequest};
use gitlab::GitlabBuilder;
use dotenvy::dotenv;
use std::env;
use tokio::sync::mpsc;
use std::time::{Instant, Duration};

#[tokio::test]
async fn test_massive_mr_diff_automatic_fetch() {
    dotenv().ok();
    let token = env::var("GITLAB_TOKEN").expect("GITLAB_TOKEN not set");
    let url = env::var("GITLAB_URL").unwrap_or_else(|_| "https://gitlab.com".to_string());

    let client = GitlabBuilder::new(url.replace("https://", "").replace("http://", ""), token)
        .build_async()
        .await
        .expect("Failed to create GitLab client");

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx.clone());

    let project_id = 82156545;
    let mr_iid = 21; 
    
    let endpoint = gitlab::api::projects::merge_requests::MergeRequest::builder()
        .project(project_id)
        .merge_request(mr_iid)
        .build()
        .unwrap();
    let mr: MergeRequest = gitlab::api::AsyncQuery::query_async(&endpoint, &app.gitlab).await.unwrap();
    app.selected_mr = Some(mr.clone());

    println!("Fetching MR diffs (which should trigger automatic background fetch of missing content)...");
    app.fetch_mr_diffs(project_id, mr_iid).await.expect("Failed to start fetch");
    
    let start = Instant::now();
    let mut all_fetched = false;
    
    // We expect 120 files. 50 are from API, 70 need fetching.
    // 70 files with concurrency 10 might take a few seconds.
    while start.elapsed().as_secs() < 60 {
        if let Ok(update) = rx.try_recv() {
            app.process_update_single(update).await;
        } else {
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            let collapsed_count = app.mr_diffs.iter().filter(|d| d.collapsed).count();
            let total_count = app.mr_diffs.len();
            
            if total_count > 0 && collapsed_count == 0 {
                all_fetched = true;
                break;
            }
        }
    }

    assert!(all_fetched, "Not all files were fetched automatically within 60s. Remaining collapsed: {}", app.mr_diffs.iter().filter(|d| d.collapsed).count());
    
    let last_file = app.mr_diffs.last().unwrap();
    assert!(!last_file.collapsed, "Last file should be uncollapsed");
    assert!(last_file.display_lines.len() > 2, "Last file should have content");
    
    println!("SUCCESS: All {} files were fetched automatically!", app.mr_diffs.len());
}
