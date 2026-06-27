use serde_json::Value;
use std::time::Duration;
use tokio::sync::mpsc;

use gitlab::api::AsyncQuery;

use lazygitlab::models::{App, DataUpdate};
use lazygitlab::api::{
    PagedIssues, PagedMergeRequests, ProjectLabels, ProjectMembersAll, ProjectMilestones, Todos,
};

mod common;

async fn try_build_client() -> Option<gitlab::AsyncGitlab> {
    let _ = dotenvy::dotenv();
    let token = match std::env::var("GITLAB_TOKEN") {
        Ok(t) if !t.trim().is_empty() => t,
        _ => return None,
    };
    let url = std::env::var("GITLAB_URL").unwrap_or_else(|_| "https://gitlab.com".to_string());
    match gitlab::GitlabBuilder::new(url.replace("https://", "").replace("http://", ""), token)
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
    while start.elapsed() < Duration::from_secs(10) {
        app.process_updates(rx).await;
        if !app.is_loading {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn fetch_json<E>(client: &gitlab::AsyncGitlab, endpoint: E) -> Value
where
    E: gitlab::api::AsyncQuery<Value, gitlab::AsyncGitlab>,
{
    AsyncQuery::query_async(&endpoint, client).await.expect("API query failed")
}

async fn build_app(client: gitlab::AsyncGitlab) -> (App, mpsc::UnboundedReceiver<DataUpdate>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (App::new(client, tx), rx)
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_data_update_round_trip_project_issues() {
    if common::skip_if_live_unavailable("live_full_data_update_round_trip_project_issues") {
        return;
    }
    let Some(client) = try_build_client().await else { return; };
    let ground = common::test_ground();
    let endpoint = PagedIssues {
        project: ground.project.id,
        page: 1,
        per_page: 5,
        state: Some("all".into()),
        order_by: None,
        sort: None,
        with_labels_details: Some(true),
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("issues response must be array");
    assert!(!arr.is_empty(), "expected at least one issue in test ground");

    let (mut app, mut rx) = build_app(client).await;
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: None,
        default_branch: None,
        namespace: None,
    });
    app.active_context = lazygitlab::models::Context::Project;
    app.active_panel = lazygitlab::models::Panel::Main;

    for issue_json in arr.iter().take(3) {
        let issue: lazygitlab::models::Issue =
            serde_json::from_value(issue_json.clone()).expect("deserialize issue");
        let original_iid = issue.iid;
        let original_title = issue.title.clone();
        let original_labels: Vec<(u64, String)> =
            issue.labels.iter().map(|l| (l.id(), l.name().to_string())).collect();
        let _ = app.tx.send(DataUpdate::ProjectIssues {
            project_id: ground.project.id,
            data: vec![issue],
            page: 1,
            has_more: false,
        });
        wait_for_loading_finish(&mut app, &mut rx).await;

        let stored = app
            .issues
            .iter()
            .find(|i| i.iid == original_iid)
            .unwrap_or_else(|| panic!("issue !{original_iid} not present in app after DataUpdate"));
        assert_eq!(stored.title, original_title, "issue !{original_iid} title drift after DataUpdate");
        for (id, name) in &original_labels {
            assert!(
                stored.labels.iter().any(|l| l.id() == *id && l.name() == name),
                "issue !{original_iid} lost label id={id} name={name} after DataUpdate"
            );
        }
    }
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_data_update_round_trip_project_mrs() {
    if common::skip_if_live_unavailable("live_full_data_update_round_trip_project_mrs") {
        return;
    }
    let Some(client) = try_build_client().await else { return; };
    let ground = common::test_ground();
    let endpoint = PagedMergeRequests {
        project: ground.project.id,
        page: 1,
        per_page: 5,
        state: Some("all".into()),
        order_by: None,
        sort: None,
        with_labels_details: Some(true),
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("mrs response must be array");
    assert!(!arr.is_empty(), "expected at least one MR in test ground");

    let (mut app, mut rx) = build_app(client).await;
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: None,
        default_branch: None,
        namespace: None,
    });
    app.active_context = lazygitlab::models::Context::Project;
    app.active_panel = lazygitlab::models::Panel::Main;

    for mr_json in arr.iter().take(3) {
        let mr: lazygitlab::models::MergeRequest =
            serde_json::from_value(mr_json.clone()).expect("deserialize mr");
        let original_iid = mr.iid;
        let original_title = mr.title.clone();
        let original_state = mr.state.clone();
        let original_source = mr.source_branch.clone();
        let original_target = mr.target_branch.clone();
        let _ = app.tx.send(DataUpdate::ProjectMRs {
            project_id: ground.project.id,
            data: vec![mr],
            page: 1,
            has_more: false,
        });
        wait_for_loading_finish(&mut app, &mut rx).await;

        let stored = app
            .merge_requests
            .iter()
            .find(|m| m.iid == original_iid)
            .unwrap_or_else(|| panic!("MR !{original_iid} not present in app after DataUpdate"));
        assert_eq!(stored.title, original_title, "MR !{original_iid} title drift");
        assert_eq!(stored.state, original_state, "MR !{original_iid} state drift");
        assert_eq!(stored.source_branch, original_source, "MR !{original_iid} source_branch drift");
        assert_eq!(stored.target_branch, original_target, "MR !{original_iid} target_branch drift");
    }
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_data_update_metadata_round_trip() {
    if common::skip_if_live_unavailable("live_full_data_update_metadata_round_trip") {
        return;
    }
    let Some(client) = try_build_client().await else { return; };
    let ground = common::test_ground();

    let label_value: Value = AsyncQuery::query_async(
        &ProjectLabels { project: ground.project.id },
        &client,
    )
    .await
    .expect("labels fetch");
    let label_arr = label_value.as_array().expect("labels array");
    assert!(!label_arr.is_empty(), "expected at least one label");
    let first_label = &label_arr[0];
    let label_id = first_label
        .as_object()
        .and_then(|o| o.get("id"))
        .and_then(|v| v.as_u64())
        .expect("label id");
    let label_name = first_label
        .as_object()
        .and_then(|o| o.get("name"))
        .and_then(|v| v.as_str())
        .expect("label name")
        .to_string();

    let member_value: Value = AsyncQuery::query_async(
        &ProjectMembersAll { project: ground.project.id },
        &client,
    )
    .await
    .expect("members fetch");
    let member_arr = member_value.as_array().expect("members array");
    assert!(!member_arr.is_empty(), "expected at least one member");
    let first_member = &member_arr[0];
    let member_id = first_member
        .as_object()
        .and_then(|o| o.get("id"))
        .and_then(|v| v.as_u64())
        .expect("member id");
    let member_username = first_member
        .as_object()
        .and_then(|o| o.get("username"))
        .and_then(|v| v.as_str())
        .expect("member username")
        .to_string();

    let milestone_value: Value = AsyncQuery::query_async(
        &ProjectMilestones { project: ground.project.id },
        &client,
    )
    .await
    .expect("milestones fetch");
    let milestone_arr = milestone_value.as_array().expect("milestones array");
    let (milestone_id, milestone_title) = if milestone_arr.is_empty() {
        (0, String::new())
    } else {
        let m = &milestone_arr[0];
        let id = m
            .as_object()
            .and_then(|o| o.get("id"))
            .and_then(|v| v.as_u64())
            .expect("milestone id");
        let title = m
            .as_object()
            .and_then(|o| o.get("title"))
            .and_then(|v| v.as_str())
            .expect("milestone title")
            .to_string();
        (id, title)
    };

    let (mut app, mut rx) = build_app(client).await;
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: None,
        default_branch: None,
        namespace: None,
    });

    let labels: Vec<lazygitlab::models::Label> = label_arr
        .iter()
        .map(|v| serde_json::from_value(v.clone()).expect("label"))
        .collect();
    let members: Vec<lazygitlab::models::Author> = member_arr
        .iter()
        .map(|v| serde_json::from_value(v.clone()).expect("member"))
        .collect();
    let milestones: Vec<lazygitlab::models::Milestone> = milestone_arr
        .iter()
        .map(|v| serde_json::from_value(v.clone()).expect("milestone"))
        .collect();
    let _ = app.tx.send(DataUpdate::Metadata {
        project_id: ground.project.id,
        labels,
        milestones,
        members,
    });
    wait_for_loading_finish(&mut app, &mut rx).await;

    let labels_in_cache = app
        .project_label_cache
        .get(&ground.project.id)
        .cloned()
        .unwrap_or_default();
    assert!(
        labels_in_cache.iter().any(|l| l.id() == label_id && l.name() == label_name),
        "label id={label_id} name={label_name} not in cache after DataUpdate::Metadata"
    );

    let members_in_cache = app
        .project_member_cache
        .get(&ground.project.id)
        .cloned()
        .unwrap_or_default();
    assert!(
        members_in_cache.iter().any(|m| m.id == member_id && m.username == member_username),
        "member id={member_id} username={member_username} not in cache after DataUpdate::Metadata"
    );

    if milestone_id != 0 {
        let milestones_in_cache = app
            .project_milestone_cache
            .get(&ground.project.id)
            .cloned()
            .unwrap_or_default();
        assert!(
            milestones_in_cache.iter().any(|m| m.id == milestone_id && m.title == milestone_title),
            "milestone id={milestone_id} title={milestone_title} not in cache after DataUpdate::Metadata"
        );
    }
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_data_update_todos_round_trip() {
    if common::skip_if_live_unavailable("live_full_data_update_todos_round_trip") {
        return;
    }
    let Some(client) = try_build_client().await else { return; };
    let endpoint = Todos {
        page: Some(1),
        per_page: Some(5),
        state: Some("pending".into()),
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("todos response must be array");
    if arr.is_empty() {
        eprintln!("[live_full_data_update_todos_round_trip] no pending todos, skipping");
        return;
    }

    let (mut app, mut rx) = build_app(client).await;
    for todo_json in arr.iter().take(3) {
        let todo: lazygitlab::models::Todo =
            serde_json::from_value(todo_json.clone()).expect("deserialize todo");
        let original_id = todo.id;
        let original_body = todo.body.clone();
        let _ = app.tx.send(DataUpdate::Todos {
            data: vec![todo],
            page: 1,
            has_more: false,
        });
        wait_for_loading_finish(&mut app, &mut rx).await;
        let stored = app
            .todos
            .iter()
            .find(|t| t.id == original_id)
            .unwrap_or_else(|| panic!("todo {} not present in app after DataUpdate", original_id));
        assert_eq!(stored.body, original_body, "todo {} body drift", original_id);
    }
}
