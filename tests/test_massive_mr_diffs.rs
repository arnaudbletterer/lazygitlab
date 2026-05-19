use lazygitlab::models::{App, DataUpdate, MergeRequest};
use gitlab::GitlabBuilder;
use dotenvy::dotenv;
use std::env;
use tokio::sync::mpsc;
use std::time::{Instant, Duration};

#[tokio::test]
async fn test_massive_mr_diff_fetch() {
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
    let mr_iid = 21; // The one we created
    
    // We need to fetch the MR first to get diff_refs!
    let endpoint = gitlab::api::projects::merge_requests::MergeRequest::builder()
        .project(project_id)
        .merge_request(mr_iid)
        .build()
        .unwrap();
    let mr: MergeRequest = gitlab::api::AsyncQuery::query_async(&endpoint, &app.gitlab).await.unwrap();
    app.selected_mr = Some(mr.clone());

    println!("Fetching MR diffs...");
    app.fetch_mr_diffs(project_id, mr_iid).await.expect("Failed to start fetch");
    
    let start = Instant::now();
    while start.elapsed().as_secs() < 60 {
        if let Ok(update) = rx.try_recv() {
            app.process_update_single(update).await;
        } else {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if app.is_loading == false && start.elapsed().as_secs() > 1 {
                break;
            }
        }
    }

    let files_with_empty_diffs = app.mr_diffs.iter().filter(|d| d.diff.is_empty()).count();
    println!("Files with empty diffs initially: {}", files_with_empty_diffs);
    
    // Now simulate pressing 'E' on the first collapsed file!
    let collapsed_file = app.mr_diffs.iter().find(|d| d.collapsed).cloned();
    if let Some(cf) = collapsed_file {
        println!("Fetching single diff for collapsed file: {}", cf.new_path);
        app.fetch_single_file_diff(project_id, mr_iid, cf.old_path.clone(), cf.new_path.clone()).await.unwrap();
        
        let start = Instant::now();
        while start.elapsed().as_secs() < 60 {
            if let Ok(update) = rx.try_recv() {
                app.process_update_single(update).await;
            } else {
                tokio::time::sleep(Duration::from_millis(100)).await;
                if app.is_loading == false && start.elapsed().as_secs() > 1 {
                    break;
                }
            }
        }
        
        let fetched_file = app.mr_diffs.iter().find(|d| d.new_path == cf.new_path).unwrap();
        assert!(!fetched_file.collapsed, "File should no longer be collapsed");
        assert!(!fetched_file.diff.is_empty(), "File should have a diff string now");
        println!("Successfully fetched missing diff!");
    } else {
        println!("No collapsed files found to test.");
    }
}
