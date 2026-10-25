use std::time::Duration;
use tokio::sync::mpsc;

use gitlab::GitlabBuilder;
use lazygitlab::models::{App, DataUpdate};

mod common;

async fn try_build_client() -> Option<gitlab::AsyncGitlab> {
    let _ = dotenvy::dotenv();
    let token = match std::env::var("GITLAB_TOKEN") {
        Ok(t) if !t.trim().is_empty() => t,
        _ => return None,
    };
    let url = std::env::var("GITLAB_URL").unwrap_or_else(|_| "https://gitlab.com".to_string());
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

async fn wait_for_loading_finish(app: &mut App, rx: &mut mpsc::UnboundedReceiver<DataUpdate>) {
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(20) {
        app.process_updates(rx).await;
        if !app.is_loading {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_create_issue_optimistic_insert() {
    if common::skip_if_live_unavailable("live_full_create_issue_optimistic_insert") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: Some(ground.project.web_url.clone().unwrap_or_default()),
        default_branch: Some(ground.project.default_branch.clone()),
        namespace: None,
    });
    app.active_context = lazygitlab::models::Context::Project;
    app.create_issue_title = "Test issue from optimistic insert".into();
    app.create_issue_description = "Body".into();
    let initial_issue_count = app.issues.len();

    let _ = app.create_issue().await;
    assert_eq!(
        app.issues.len(),
        initial_issue_count + 1,
        "the pending issue should be inserted before the API call returns"
    );
    let pending = app
        .issues
        .iter()
        .find(|i| i.title == "Test issue from optimistic insert")
        .expect("the pending issue should be in the local list");
    assert!(
        pending.web_url.is_empty(),
        "pending issue should have empty web_url"
    );

    wait_for_loading_finish(&mut app, &mut rx).await;
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_remove_pending_issue_data_update() {
    if common::skip_if_live_unavailable("live_full_remove_pending_issue_data_update") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: Some(ground.project.web_url.clone().unwrap_or_default()),
        default_branch: Some(ground.project.default_branch.clone()),
        namespace: None,
    });

    let fake_iid = u64::MAX - 1;
    app.issues.push(lazygitlab::models::Issue {
        updated_at: None,
        iid: fake_iid,
        project_id: ground.project.id,
        title: "Synthetic".into(),
        description: None,
        author: lazygitlab::models::Author::new(1, "u".into()),
        assignees: vec![],
        state: "opened".into(),
        web_url: "u".into(),
        references: None,
        issue_type: None,
        labels: vec![],
        milestone: None,
        due_date: None,
        start_date: None,
        weight: None,
        task_completion_status: None,
        confidential: false,
        created_at: Some("t".into()),
        upvotes: Some(0),
        downvotes: Some(0),
        reactions: vec![],
        participants: vec![],
    });
    app.filter_lists();
    let before = app.issues.len();
    assert_eq!(app.issues.iter().filter(|i| i.iid == fake_iid).count(), 1);

    let _ = app
        .tx
        .send(DataUpdate::RemovePendingIssue { iid: fake_iid });
    wait_for_loading_finish(&mut app, &mut rx).await;
    assert_eq!(
        app.issues.len(),
        before - 1,
        "the pending issue should be removed"
    );
    assert_eq!(app.issues.iter().filter(|i| i.iid == fake_iid).count(), 0);
}
