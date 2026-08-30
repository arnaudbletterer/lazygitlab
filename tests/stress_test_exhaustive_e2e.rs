use dotenvy::dotenv;
use gitlab::GitlabBuilder;
use lazygitlab::models::{
    App, Context, DataUpdate, IssueFilter, MenuSelection, MergeRequestFilter, Panel,
};
use std::env;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

mod common;

async fn wait_for_loading_finish(app: &mut App, rx: &mut mpsc::UnboundedReceiver<DataUpdate>) {
    let start = Instant::now();
    tokio::time::sleep(Duration::from_millis(300)).await;
    app.process_updates(rx).await;

    while Instant::now().duration_since(start) < Duration::from_secs(30) {
        app.process_updates(rx).await;
        if !app.is_loading {
            tokio::time::sleep(Duration::from_millis(200)).await;
            app.process_updates(rx).await;
            if !app.is_loading {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_labels(
    app: &mut App,
    rx: &mut mpsc::UnboundedReceiver<DataUpdate>,
    count: usize,
) {
    let start = Instant::now();
    while app.project_labels.len() < count && start.elapsed().as_secs() < 15 {
        app.process_updates(rx).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn try_build_client() -> Option<gitlab::AsyncGitlab> {
    dotenv().ok();
    let token = match env::var("GITLAB_TOKEN") {
        Ok(t) if !t.trim().is_empty() => t,
        _ => return None,
    };
    let url = env::var("GITLAB_URL").unwrap_or_else(|_| "https://gitlab.com".to_string());
    match GitlabBuilder::new(url.replace("https://", "").replace("http://", ""), token)
        .build_async()
        .await
    {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("Skipping test: Failed to create GitLab client: {}", e);
            None
        }
    }
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_exhaustive_test_ground() {
    if common::skip_if_live_unavailable("live_full_exhaustive_test_ground") {
        return;
    }
    let ground = common::test_ground();
    let Some(client) = try_build_client().await else {
        return;
    };

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx.clone(), None);

    println!("Starting initial setup...");
    app.initial_setup().await.expect("Failed initial setup");
    wait_for_loading_finish(&mut app, &mut rx).await;

    let project = app
        .projects
        .iter()
        .find(|p| {
            p.path_with_namespace.as_deref() == Some(ground.project.path_with_namespace.as_str())
        })
        .cloned()
        .unwrap_or_else(|| {
            panic!(
                "Test ground project {} not found",
                ground.project.path_with_namespace
            )
        });

    println!(
        "Selected project: {} (ID: {})",
        ground.project.path_with_namespace, project.id
    );
    app.selected_project = Some(project.clone());
    app.active_context = Context::Project;
    app.active_panel = Panel::Project;

    println!("Fetching labels...");
    app.trigger_project_metadata_fetch(project.id);
    wait_for_labels(&mut app, &mut rx, ground.counts.labels).await;
    assert!(
        app.project_labels.len() >= ground.counts.labels,
        "Should have at least {} labels, found {}",
        ground.counts.labels,
        app.project_labels.len()
    );
    println!("SUCCESS: Fetched {} labels", app.project_labels.len());

    println!("Fetching issues...");
    app.active_selection = MenuSelection::Issues;
    app.issue_filter = IssueFilter::All;
    app.refresh().await;
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(
        app.issues.len() >= ground.counts.issues,
        "Should have at least {} issues, found {}",
        ground.counts.issues,
        app.issues.len()
    );
    println!("SUCCESS: Fetched {} issues", app.issues.len());

    println!("Fetching MRs...");
    app.active_selection = MenuSelection::MergeRequests;
    app.mr_filter = MergeRequestFilter::All;
    app.refresh().await;
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(
        app.merge_requests.len() >= ground.counts.merge_requests,
        "Should have at least {} MRs (All), found {}",
        ground.counts.merge_requests,
        app.merge_requests.len()
    );
    println!("SUCCESS: Fetched {} MRs", app.merge_requests.len());

    println!("Fetching repository tree...");
    app.active_selection = MenuSelection::Repository;
    app.refresh().await;
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(
        !app.repository_tree.is_empty(),
        "Should have files in repository"
    );

    let deep_root = ground.extras.deep_tree_root.as_deref().unwrap_or("dir-0");
    let deep_file_path =
        format!("{deep_root}/dir-1/dir-2/dir-3/dir-4/dir-5/dir-6/dir-7/dir-8/dir-9/file-0.txt");
    assert!(
        app.repository_tree
            .iter()
            .any(|it| it.path == deep_file_path),
        "Deep file not found in recursive tree fetch (looking for {})",
        deep_file_path
    );

    println!("Fetching deep file content...");
    app.repository_tree_state
        .select(vec![deep_file_path.to_string()]);
    app.fetch_repository_file(deep_file_path.to_string())
        .await
        .expect("Failed to fetch file content");
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(
        app.selected_file_content.is_some(),
        "File content should be fetched"
    );
    println!("SUCCESS: Navigated deep file tree and fetched content");

    if let Some(expected_board_id) = ground.extras.board_id {
        println!("Fetching boards...");
        app.active_selection = MenuSelection::Boards;
        app.refresh().await;
        wait_for_loading_finish(&mut app, &mut rx).await;
        assert!(
            !app.project_boards.is_empty(),
            "Should have at least one board"
        );

        let board = app
            .project_boards
            .iter()
            .find(|b| b.id == expected_board_id)
            .cloned()
            .unwrap_or_else(|| app.project_boards[0].clone());
        app.selected_board = Some(board.clone());
        println!("Found board: {} (ID: {})", board.name, board.id);
        println!("Fetching board lists for board ID {}...", board.id);
        app.fetch_board_lists(board.id)
            .await
            .expect("Failed to fetch board lists");
        wait_for_loading_finish(&mut app, &mut rx).await;

        if app.board_lists.is_empty() {
            println!("Board lists are empty. Last error: {:?}", app.last_error);
        }
        assert!(!app.board_lists.is_empty(), "Should have board lists");
        println!(
            "SUCCESS: Fetched board and lists (count: {})",
            app.board_lists.len()
        );
    } else {
        println!("SKIP: no board_id in test_ground.json (re-run populate to seed a board)");
    }

    println!("Fetching pipelines...");
    app.active_selection = MenuSelection::Pipelines;
    app.refresh().await;
    wait_for_loading_finish(&mut app, &mut rx).await;
    if let Some(err) = &app.last_error {
        panic!("Error fetching pipelines: {}", err);
    }
    println!(
        "SUCCESS: Fetched pipelines (count: {})",
        app.pipelines.len()
    );

    println!("Fetching branches...");
    app.fetch_branches(project.id)
        .await
        .expect("Failed to fetch branches");
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert!(
        app.project_branches.len() >= ground.counts.branches,
        "Should have at least {} branches, found {}",
        ground.counts.branches,
        app.project_branches.len()
    );
    println!("SUCCESS: Fetched {} branches", app.project_branches.len());

    println!("ALL EXHAUSTIVE TESTS PASSED SUCCESSFULLY!");
}
