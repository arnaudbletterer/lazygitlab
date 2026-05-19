use lazygitlab::models::{App, DataUpdate, Context, Panel, MenuSelection, MergeRequestFilter};
use gitlab::GitlabBuilder;
use dotenvy::dotenv;
use std::env;
use tokio::sync::mpsc;
use std::time::{Instant, Duration};

async fn wait_for_loading_finish(app: &mut App, rx: &mut mpsc::UnboundedReceiver<DataUpdate>) {
    let start = Instant::now();
    // Wait for at least one update to ensure any spawned tasks had a chance to send Loading(true)
    tokio::time::sleep(Duration::from_millis(300)).await;
    app.process_updates(rx).await;
    
    while Instant::now().duration_since(start) < Duration::from_secs(30) {
        app.process_updates(rx).await;
        if !app.is_loading {
            // Check again after a small delay to catch staggered task starts
            tokio::time::sleep(Duration::from_millis(200)).await;
            app.process_updates(rx).await;
            if !app.is_loading { return; }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_labels(app: &mut App, rx: &mut mpsc::UnboundedReceiver<DataUpdate>, count: usize) {
    let start = Instant::now();
    while app.project_labels.len() < count && start.elapsed().as_secs() < 15 {
        app.process_updates(rx).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::test]
async fn test_exhaustive_stress_project() {
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

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx.clone());

    // 1. Initial Setup
    println!("Starting initial setup...");
    app.initial_setup().await.expect("Failed initial setup");
    wait_for_loading_finish(&mut app, &mut rx).await;

    // 2. Select Stress Test Project
    let target_path = "abletterer/lazygitlab-stress-test";
    let project = app.projects.iter().find(|p| p.path_with_namespace.as_deref() == Some(target_path))
        .cloned()
        .expect("Stress test project not found");
    
    println!("Selected project: {} (ID: {})", target_path, project.id);
    app.selected_project = Some(project.clone());
    app.active_context = Context::Project;
    app.active_panel = Panel::Project;

    // 3. Verify Labels (Stress: 30 labels)
    println!("Fetching labels...");
    app.trigger_project_metadata_fetch(project.id);
    wait_for_labels(&mut app, &mut rx, 30).await;
    assert!(app.project_labels.len() >= 30, "Should have at least 30 labels, found {}", app.project_labels.len());
    println!("SUCCESS: Fetched {} labels", app.project_labels.len());

    // 4. Verify Issues (Stress: 50 issues)
    println!("Fetching issues...");
    app.active_selection = MenuSelection::Issues;
    app.refresh().await;
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(app.issues.len() >= 50, "Should have at least 50 issues, found {}", app.issues.len());
    println!("SUCCESS: Fetched {} issues", app.issues.len());

    // 5. Verify MRs (Stress: 20 MRs)
    println!("Fetching MRs...");
    app.active_selection = MenuSelection::MergeRequests;
    app.mr_filter = MergeRequestFilter::All;
    app.refresh().await;
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(app.merge_requests.len() >= 20, "Should have at least 20 MRs (All), found {}", app.merge_requests.len());
    println!("SUCCESS: Fetched {} MRs", app.merge_requests.len());

    // 6. Verify Repository / Files (Stress: Deep Tree)
    println!("Fetching repository tree...");
    app.active_selection = MenuSelection::Repository;
    app.refresh().await;
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(!app.repository_tree.is_empty(), "Should have files in repository");
    
    let deep_file_path = "dir-0/dir-1/dir-2/dir-3/dir-4/dir-5/dir-6/dir-7/dir-8/dir-9/file-0.txt";
    assert!(app.repository_tree.iter().any(|it| it.path == deep_file_path), "Deep file not found in recursive tree fetch");
    
    println!("Fetching deep file content...");
    app.repository_tree_state.select(vec![deep_file_path.to_string()]);
    app.fetch_repository_file(deep_file_path.to_string()).await.expect("Failed to fetch file content");
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(app.selected_file_content.is_some(), "File content should be fetched");
    println!("SUCCESS: Navigated deep file tree and fetched content");

    // 7. Verify Boards (Phase 3)
    println!("Fetching boards...");
    app.active_selection = MenuSelection::Boards;
    app.refresh().await;
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(!app.project_boards.is_empty(), "Should have at least one board");
    
    let board = app.project_boards[0].clone();
    app.selected_board = Some(board.clone());
    println!("Found board: {} (ID: {})", board.name, board.id);
    println!("Fetching board lists for board ID {}...", board.id);
    app.fetch_board_lists(board.id).await.expect("Failed to fetch board lists");
    wait_for_loading_finish(&mut app, &mut rx).await;
    
    if app.board_lists.is_empty() {
        println!("Board lists are empty. Last error: {:?}", app.last_error);
    }
    assert!(!app.board_lists.is_empty(), "Should have board lists");
    println!("SUCCESS: Fetched board and lists (count: {})", app.board_lists.len());

    // 8. Verify Pipelines
    println!("Fetching pipelines...");
    app.active_selection = MenuSelection::Pipelines;
    app.refresh().await;
    wait_for_loading_finish(&mut app, &mut rx).await;
    if let Some(err) = &app.last_error {
        panic!("Error fetching pipelines: {}", err);
    }
    println!("SUCCESS: Fetched pipelines (count: {})", app.pipelines.len());

    // 9. Verify Branches
    println!("Fetching branches...");
    app.fetch_branches(project.id).await.expect("Failed to fetch branches");
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(app.project_branches.len() >= 20, "Should have at least 20 branches, found {}", app.project_branches.len());
    println!("SUCCESS: Fetched {} branches", app.project_branches.len());
    
    println!("ALL STRESS TESTS PASSED SUCCESSFULLY!");
}
