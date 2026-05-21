use lazygitlab::models::{App, DataUpdate};
use gitlab::GitlabBuilder;
use dotenvy::dotenv;
use std::env;
use tokio::sync::mpsc;

async fn wait_for_loading_finish(app: &mut App, rx: &mut mpsc::UnboundedReceiver<DataUpdate>) {
    let start = std::time::Instant::now();
    while std::time::Instant::now().duration_since(start) < std::time::Duration::from_secs(10) {
        app.process_updates(rx).await;
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

    let client = match GitlabBuilder::new(url.replace("https://", "").replace("http://", ""), token).build_async().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test: Failed to create GitLab client (likely invalid token): {}", e);
            return;
        }
    };

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx);

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
        app.active_context = lazygitlab::models::Context::Project;
        app.active_panel = lazygitlab::models::Panel::Project; // IMPORTANT: Set panel to prevent context reset
        
        // 3. Validate MRs Isolation
        app.active_selection = lazygitlab::models::MenuSelection::MergeRequests;
        app.merge_requests.clear();
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
        app.issues.clear();
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

#[tokio::test]
async fn test_real_updates_on_lazygitlab_test() {
    dotenv().ok();
    let token = match env::var("GITLAB_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Skipping test: GITLAB_TOKEN not set");
            return;
        }
    };
    let url = env::var("GITLAB_URL").unwrap_or_else(|_| "https://gitlab.com".to_string());

    let client = match GitlabBuilder::new(url.replace("https://", "").replace("http://", ""), token).build_async().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test: Failed to create GitLab client (likely invalid token): {}", e);
            return;
        }
    };

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx);

    app.initial_setup().await.expect("Failed initial setup");
    wait_for_loading_finish(&mut app, &mut rx).await;

    let target_proj_path = "abletterer/lazygitlab-test";
    let target_proj = app.projects.iter().find(|p| p.path_with_namespace.as_deref() == Some(target_proj_path)).cloned();
    
    if let Some(project) = target_proj {
        println!("Found project: {}", project.path_with_namespace.as_deref().unwrap_or(""));
        app.selected_project = Some(project.clone());
        app.active_context = lazygitlab::models::Context::Project;
        app.active_panel = lazygitlab::models::Panel::Project;
        
        // Test Issue Update
        app.active_selection = lazygitlab::models::MenuSelection::Issues;
        app.issues.clear();
        app.refresh().await;
        wait_for_loading_finish(&mut app, &mut rx).await;
        app.filter_lists();

        if let Some(lazygitlab::models::GroupedItem::Item(issue)) = app.filtered_issues.first().cloned() {
            app.active_panel = lazygitlab::models::Panel::MainDetail;
            app.selected_issue = Some(issue.clone());
            let original_desc = issue.description.unwrap_or_default();
            let new_desc = format!("{} [E2E TEST]", original_desc);
            
            app.input_target = lazygitlab::models::InputTarget::EditDescription;
            app.textarea = ratatui_textarea::TextArea::from(new_desc.lines());
            app.update_issue().await.expect("Failed to trigger update_issue");
            wait_for_loading_finish(&mut app, &mut rx).await;
            
            let updated_issue = app.selected_issue.as_ref().expect("Should have selected issue");
            assert_eq!(updated_issue.description.as_deref().unwrap_or(""), new_desc, "Issue description did not update optimistically/via API detail response!");
            
            // Revert the description
            app.input_target = lazygitlab::models::InputTarget::EditDescription;
            app.textarea = ratatui_textarea::TextArea::from(original_desc.lines());
            app.update_issue().await.expect("Failed to revert update_issue");
            wait_for_loading_finish(&mut app, &mut rx).await;
            
            let reverted_issue = app.selected_issue.as_ref().expect("Should have selected issue");
            assert_eq!(reverted_issue.description.as_deref().unwrap_or(""), original_desc, "Issue description did not revert!");
        }

        // Test MR Update
        app.active_selection = lazygitlab::models::MenuSelection::MergeRequests;
        app.merge_requests.clear();
        app.refresh().await;
        wait_for_loading_finish(&mut app, &mut rx).await;
        app.filter_lists();

        if let Some(lazygitlab::models::GroupedItem::Item(mr)) = app.filtered_mrs.first().cloned() {
            app.active_panel = lazygitlab::models::Panel::MainDetail;
            app.selected_mr = Some(mr.clone());
            let original_title = mr.title.clone();
            let new_title = format!("{} [E2E TEST]", original_title);
            
            app.input_target = lazygitlab::models::InputTarget::EditTitle;
            app.textarea = ratatui_textarea::TextArea::from(new_title.lines());
            app.update_mr().await.expect("Failed to trigger update_mr");
            wait_for_loading_finish(&mut app, &mut rx).await;
            
            let updated_mr = app.selected_mr.as_ref().expect("Should have selected mr");
            assert_eq!(updated_mr.title, new_title, "MR title did not update optimistically/via API detail response!");
            
            // Revert the title
            app.input_target = lazygitlab::models::InputTarget::EditTitle;
            app.textarea = ratatui_textarea::TextArea::from(original_title.lines());
            app.update_mr().await.expect("Failed to revert update_mr");
            wait_for_loading_finish(&mut app, &mut rx).await;
            
            let reverted_mr = app.selected_mr.as_ref().expect("Should have selected mr");
            assert_eq!(reverted_mr.title, original_title, "MR title did not revert!");

            // Test MR Draft Toggle
            let original_draft = reverted_mr.draft;
            app.toggle_mr_draft().await.expect("Failed to toggle mr draft");
            wait_for_loading_finish(&mut app, &mut rx).await;

            let toggled_mr = app.selected_mr.as_ref().expect("Should have selected mr");
            assert_ne!(toggled_mr.draft, original_draft, "MR draft state did not toggle!");

            // Revert MR Draft Toggle
            app.toggle_mr_draft().await.expect("Failed to revert mr draft toggle");
            wait_for_loading_finish(&mut app, &mut rx).await;
            
            let reverted_draft_mr = app.selected_mr.as_ref().expect("Should have selected mr");
            assert_eq!(reverted_draft_mr.draft, original_draft, "MR draft state did not revert!");
        }

    } else {
        println!("Test project '{}' not found. Cannot run real updates test.", target_proj_path);
    }
}

#[tokio::test]
async fn test_collaboration_on_lazygitlab_test() {
    dotenv().ok();
    let token = match env::var("GITLAB_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Skipping test: GITLAB_TOKEN not set");
            return;
        }
    };
    let url = env::var("GITLAB_URL").unwrap_or_else(|_| "https://gitlab.com".to_string());

    let client = match GitlabBuilder::new(url.replace("https://", "").replace("http://", ""), token).build_async().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test: Failed to create GitLab client (likely invalid token): {}", e);
            return;
        }
    };

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx);

    app.initial_setup().await.expect("Failed initial setup");
    wait_for_loading_finish(&mut app, &mut rx).await;

    let target_proj_path = "abletterer/lazygitlab-test";
    let target_proj = app.projects.iter().find(|p| p.path_with_namespace.as_deref() == Some(target_proj_path)).cloned();
    
    if let Some(project) = target_proj {
        println!("Found project: {}", project.path_with_namespace.as_deref().unwrap_or(""));
        app.selected_project = Some(project.clone());
        app.active_context = lazygitlab::models::Context::Project;
        app.active_panel = lazygitlab::models::Panel::Project;
        
        // 1. Test Label Selection Populating
        app.active_selection = lazygitlab::models::MenuSelection::Issues;
        app.refresh().await;
        wait_for_loading_finish(&mut app, &mut rx).await;
        app.filter_lists();

        if let Some(lazygitlab::models::GroupedItem::Item(issue)) = app.filtered_issues.first().cloned() {
            app.active_panel = lazygitlab::models::Panel::MainDetail;
            app.detail_section = lazygitlab::models::DetailSection::Metadata;
            app.selected_metadata_field = lazygitlab::models::MetadataField::Labels;
            app.selected_issue = Some(issue.clone());

            // Trigger action to open popup
            app.enter_detail_action().await;
            assert!(app.show_label_popup, "Label popup did not open");
            assert!(!app.selected_label_ids.is_empty(), "Existing label IDs not populated in popup state");

            // Toggle a label (just locally for now to verify popup logic)
            let test_label_id = app.selected_label_ids[0];
            app.selected_label_ids.retain(|&id| id != test_label_id);
            app.show_label_popup = false;
        }

        // 2. Test Discussion Reply
        app.active_selection = lazygitlab::models::MenuSelection::MergeRequests;
        app.refresh().await;
        wait_for_loading_finish(&mut app, &mut rx).await;
        app.filter_lists();

        if let Some(lazygitlab::models::GroupedItem::Item(mr)) = app.filtered_mrs.first().cloned() {
            app.active_panel = lazygitlab::models::Panel::MainDetail;
            app.selected_mr = Some(mr.clone());
            app.refresh().await; // Fetch discussions
            wait_for_loading_finish(&mut app, &mut rx).await;

            // Find a discussion that is a thread (not individual note) and has non-system notes
            if let Some(disc_idx) = app.discussions.iter().position(|d| !d.individual_note && d.notes.iter().any(|n| !n.system)) {
                app.detail_section = lazygitlab::models::DetailSection::Discussions;
                app.discussion_state.select(Some(disc_idx));
                
                let test_reply = format!("Automated E2E Reply Test {}", chrono::Local::now());
                app.textarea = ratatui_textarea::TextArea::from(vec![test_reply.clone()]);
                app.post_reply().await.expect("Failed to post reply");
                
                // Poll until the reply appears
                let poll_start = std::time::Instant::now();
                let mut found = false;
                while std::time::Instant::now().duration_since(poll_start) < std::time::Duration::from_secs(15) {
                    app.process_updates(&mut rx).await;
                    found = app.discussions.iter().any(|d| d.notes.iter().any(|n| n.body.contains(&test_reply)));
                    if found { break; }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
                
                assert!(found, "Reply was not found in discussions after posting (timed out)!");
            } else {
                println!("No replyable threaded discussions found in MR !{}. Skipping reply test.", mr.iid);
            }
        }

    } else {
        println!("Test project '{}' not found. Cannot run collaboration test.", target_proj_path);
    }
}

