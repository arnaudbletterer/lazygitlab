use lazygitlab::models::{App, DataUpdate};
use gitlab::GitlabBuilder;
use dotenvy::dotenv;
use std::env;
use tokio::sync::mpsc;

async fn wait_for_loading_finish(app: &mut App, rx: &mut mpsc::UnboundedReceiver<DataUpdate>) {
    let start = std::time::Instant::now();
    while std::time::Instant::now().duration_since(start) < std::time::Duration::from_secs(10) {
        app.process_updates(rx);
        if !app.is_loading {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

#[tokio::test]
async fn test_remote_validation_and_isolation() {
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
    let (tx, mut rx) = mpsc::unbounded_channel();
    app.tx = tx;

    // 1. Validate Initial Setup (Global Dashboard)
    app.initial_setup().await.expect("Failed initial setup");
    wait_for_loading_finish(&mut app, &mut rx).await;

    if let Some(err) = &app.last_error {
        panic!("Background error during initial setup: {}", err);
    }

    assert!(!app.projects.is_empty(), "Should fetch projects (found 0). Make sure membership projects exist on the account.");
    println!("Fetched {} projects globally", app.projects.len());

    // 2. Find a stress-test project
    let target_proj_path = "abletterer/complex-stress-alpha-1";
    let target_proj = app.projects.iter().find(|p| p.path_with_namespace.as_deref() == Some(target_proj_path)).cloned();
    
    if let Some(project) = target_proj {
        println!("Found project: {}", project.path_with_namespace.as_deref().unwrap_or(""));
        app.selected_project = Some(project.clone());
        
        // 3. Validate MRs Isolation
        app.active_selection = lazygitlab::models::MenuSelection::MergeRequests;
        app.refresh().await;
        wait_for_loading_finish(&mut app, &mut rx).await;
        
        app.filter_lists();
        for item in &app.filtered_mrs {
            if let lazygitlab::models::GroupedItem::Item(mr) = item {
                assert_eq!(mr.project_id, project.id, "MR {} does not belong to selected project", mr.iid);
            }
        }
        println!("Validated {} MRs are isolated to project {}", app.filtered_mrs.len(), project.path_with_namespace.as_deref().unwrap_or(""));

        // 4. Validate Issues Isolation
        app.active_selection = lazygitlab::models::MenuSelection::Issues;
        app.refresh().await;
        wait_for_loading_finish(&mut app, &mut rx).await;

        app.filter_lists();
        for item in &app.filtered_issues {
            if let lazygitlab::models::GroupedItem::Item(issue) = item {
                assert_eq!(issue.project_id, project.id, "Issue {} does not belong to selected project", issue.iid);
            }
        }
        println!("Validated {} issues are isolated to project {}", app.filtered_issues.len(), project.path_with_namespace.as_deref().unwrap_or(""));

        // 5. Faithful Representation Check (API comparison)
        assert!(app.issues.len() >= 5, "Should have at least 5 issues for project, found {}", app.issues.len());
        
        if let Some(lazygitlab::models::GroupedItem::Item(first_issue)) = app.filtered_issues.first() {
            assert!(first_issue.title.contains("Stress Issue") || first_issue.title.contains("Issue for workflow"), "Issue title format mismatch: {}", first_issue.title);
        }

    } else {
        println!("Stress test project '{}' not found. Ensure populate_test_data was run.", target_proj_path);
    }
}
