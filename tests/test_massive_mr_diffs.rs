use lazygitlab::models::{App, DataUpdate, MergeRequest};
use gitlab::GitlabBuilder;
use dotenvy::dotenv;
use std::env;
use tokio::sync::mpsc;
use std::time::{Instant, Duration};

#[tokio::test]
async fn test_massive_mr_diff_last_file() {
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

    println!("Total MR diffs: {}", app.mr_diffs.len());
    
    let files_with_empty_diff_and_not_collapsed = app.mr_diffs.iter().filter(|d| d.diff.is_empty() && !d.collapsed).collect::<Vec<_>>();
    println!("Files with empty diff and NOT collapsed: {}", files_with_empty_diff_and_not_collapsed.len());
    for f in &files_with_empty_diff_and_not_collapsed {
        println!("  - {}", f.new_path);
    }
    
    let last_file = app.mr_diffs.last().cloned().expect("No files in MR");
    println!("Last file: {}, Collapsed: {}, Diff len: {}", last_file.new_path, last_file.collapsed, last_file.diff.len());
    
    if last_file.diff.is_empty() {
        assert!(last_file.collapsed, "Last file {} has empty diff but collapsed is false!", last_file.new_path);
        assert!(!last_file.display_lines.is_empty(), "Last file {} has empty display_lines!", last_file.new_path);
        
        println!("Uncollapsing LAST file: {}", last_file.new_path);
        app.fetch_single_file_diff(project_id, mr_iid, last_file.old_path.clone(), last_file.new_path.clone()).await.unwrap();
        
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
        
        let uncollapsed_file = app.mr_diffs.iter().find(|d| d.new_path == last_file.new_path).unwrap();
        assert!(!uncollapsed_file.collapsed, "Last file should no longer be collapsed");
        assert!(uncollapsed_file.display_lines.len() > 2, "Uncollapsed last file should have many lines, found {}", uncollapsed_file.display_lines.len());
        println!("SUCCESS: Last file uncollapsed correctly.");
    } else {
        println!("Last file was not collapsed.");
    }
}
