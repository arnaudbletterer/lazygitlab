use dotenvy::dotenv;
use gitlab::GitlabBuilder;
use lazygitlab::models::App;
use std::env;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

mod common;

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
async fn live_full_massive_mr_diff_automatic_fetch() {
    if common::skip_if_live_unavailable("live_full_massive_mr_diff_automatic_fetch") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };

    let ground = common::test_ground();
    let project_id = ground.project.id;
    let mr_iid = match ground.massive_mr.as_ref().and_then(|m| m.iid) {
        Some(iid) => iid,
        None => {
            eprintln!(
                "Skipping: massive_mr.iid is null in data/test_ground.json. \
                 Re-run `python scripts/populate_test_ground.py --with-massive-mr` to seed a 120-file MR."
            );
            return;
        }
    };
    let expected_files = ground
        .massive_mr
        .as_ref()
        .map(|m| m.files_count)
        .unwrap_or(0) as usize;

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx.clone());

    let endpoint = gitlab::api::projects::merge_requests::MergeRequest::builder()
        .project(project_id)
        .merge_request(mr_iid)
        .build()
        .unwrap();
    let mr: lazygitlab::models::MergeRequest =
        gitlab::api::AsyncQuery::query_async(&endpoint, &app.gitlab)
            .await
            .unwrap();
    app.selected_mr = Some(mr.clone());

    println!(
        "Fetching MR !{} diffs (project {}, expecting ~{} files)...",
        mr_iid, project_id, expected_files
    );
    app.fetch_mr_diffs(project_id, mr_iid)
        .await
        .expect("Failed to start fetch");

    let start = Instant::now();
    let mut all_fetched = false;

    while start.elapsed().as_secs() < 120 {
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

    assert!(
        all_fetched,
        "Not all files were fetched automatically within 120s. Remaining collapsed: {}",
        app.mr_diffs.iter().filter(|d| d.collapsed).count()
    );

    let last_file = app.mr_diffs.last().unwrap();
    assert!(!last_file.collapsed, "Last file should be uncollapsed");
    assert!(
        last_file.display_lines.len() > 2,
        "Last file should have content"
    );

    println!(
        "SUCCESS: All {} files were fetched automatically!",
        app.mr_diffs.len()
    );
}
