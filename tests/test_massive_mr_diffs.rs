use lazygitlab::models::{App, DataUpdate, MergeRequest};
use gitlab::GitlabBuilder;
use dotenvy::dotenv;
use std::env;
use tokio::sync::mpsc;
use std::time::{Instant, Duration};

#[tokio::test]
async fn test_massive_mr_diff_enrichment() {
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

    println!("Fetching MR diffs...");
    app.fetch_mr_diffs(project_id, mr_iid).await.expect("Failed to start fetch");
    
    let start = Instant::now();
    while start.elapsed().as_secs() < 30 {
        if let Ok(update) = rx.try_recv() {
            app.process_update_single(update).await;
        } else {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if app.is_loading == false && start.elapsed().as_secs() > 1 {
                break;
            }
        }
    }

    let collapsed_files = app.mr_diffs.iter().filter(|d| d.collapsed).collect::<Vec<_>>();
    println!("Found {} collapsed files", collapsed_files.len());
    
    assert!(collapsed_files.len() > 0, "Should have collapsed files for this stress MR");
    
    for file in &collapsed_files {
        if file.display_lines.is_empty() {
             panic!("Collapsed file {} has 0 display lines! It should have placeholders.", file.new_path);
        }
    }
    println!("SUCCESS: Collapsed files have placeholders.");

    // Now test uncollapsing!
    let cf = collapsed_files[0].clone();
    println!("Uncollapsing file: {}", cf.new_path);
    app.fetch_single_file_diff(project_id, mr_iid, cf.old_path.clone(), cf.new_path.clone()).await.unwrap();
    
    let start = Instant::now();
    while start.elapsed().as_secs() < 30 {
        if let Ok(update) = rx.try_recv() {
            app.process_update_single(update).await;
        } else {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if app.is_loading == false && start.elapsed().as_secs() > 1 {
                break;
            }
        }
    }
    
    let uncollapsed_file = app.mr_diffs.iter().find(|d| d.new_path == cf.new_path).unwrap();
    assert!(!uncollapsed_file.collapsed, "File should no longer be collapsed");
    assert!(uncollapsed_file.display_lines.len() > 2, "Uncollapsed file should have many lines, found {}", uncollapsed_file.display_lines.len());
    
    println!("SUCCESS: Uncollapsing works and updates display_lines correctly.");
}
