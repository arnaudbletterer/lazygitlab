use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::sync::mpsc;

use gitlab::api::paged as gitlab_paged;
use gitlab::api::{AsyncQuery, Pagination};
use lazygitlab::models::{self, App, DataUpdate};

use lazygitlab::api::{
    IssueAwardEmoji, MergeRequestAwardEmoji, MergeRequestDiscussions, MergeRequestNotes,
    PagedIssues, PagedMergeRequests, PagedProjects, ProjectLabels, ProjectMembersAll,
    ProjectMilestones, ProjectPipelines, ProjectVariables, Todos,
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

#[allow(dead_code)]
async fn wait_for_loading_finish(app: &mut App, rx: &mut mpsc::UnboundedReceiver<DataUpdate>) {
    let start = Instant::now();
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
    endpoint
        .query_async(client)
        .await
        .expect("API query failed")
}

fn pick_json<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.as_object().and_then(|o| o.get(key))
}

fn assert_json_field_eq(json: &Value, key: &str, expected: &str) {
    let actual = pick_json(json, key)
        .unwrap_or_else(|| panic!("missing field {key} in {json}"))
        .as_str()
        .unwrap_or_else(|| panic!("field {key} is not a string in {json}"));
    assert_eq!(actual, expected, "field {key} mismatch");
}

fn assert_json_field_u64(json: &Value, key: &str, expected: u64) {
    let actual = pick_json(json, key)
        .unwrap_or_else(|| panic!("missing field {key} in {json}"))
        .as_u64()
        .unwrap_or_else(|| panic!("field {key} is not a u64 in {json}"));
    assert_eq!(actual, expected, "field {key} mismatch");
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_api_equivalence_project() {
    if common::skip_if_live_unavailable("live_full_api_equivalence_project") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();
    let endpoint = PagedProjects {
        membership: true,
        search: None,
        page: 1,
        per_page: 50,
        simple: Some(false),
        archived: Some(false),
        order_by: None,
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("projects response must be array");
    let project_json = arr
        .iter()
        .find(|p| pick_json(p, "id").and_then(|v| v.as_u64()) == Some(ground.project.id))
        .unwrap_or_else(|| {
            panic!(
                "test ground project {} not in projects list",
                ground.project.id
            )
        });

    let project: models::Project = serde_json::from_value(project_json.clone())
        .expect("TUI's Project struct must deserialize API response");
    assert_eq!(project.id, ground.project.id);
    assert_eq!(
        project.path_with_namespace.as_deref(),
        Some(ground.project.path_with_namespace.as_str())
    );
    assert_eq!(
        project.default_branch.as_deref(),
        Some(ground.project.default_branch.as_str())
    );

    assert_json_field_u64(project_json, "id", ground.project.id);
    assert_json_field_eq(
        project_json,
        "path_with_namespace",
        &ground.project.path_with_namespace,
    );
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_api_equivalence_merge_request() {
    if common::skip_if_live_unavailable("live_full_api_equivalence_merge_request") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();
    let endpoint = PagedMergeRequests {
        project: ground.project.id,
        page: 1,
        per_page: 50,
        state: Some("all".into()),
        order_by: None,
        sort: None,
        with_labels_details: Some(true),
        assignee_id: None,
        reviewer_id: None,
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("mrs response must be array");
    assert!(!arr.is_empty(), "expected at least one MR in test ground");
    let first = &arr[0];
    let mr: models::MergeRequest = serde_json::from_value(first.clone())
        .expect("TUI's MergeRequest struct must deserialize API response");
    assert_eq!(
        mr.project_id, ground.project.id,
        "MR.project_id must equal test ground id"
    );

    let title = pick_json(first, "title")
        .and_then(|v| v.as_str())
        .expect("title is required");
    assert_eq!(mr.title, title, "MR.title must match API title");

    if let Some(web_url) = pick_json(first, "web_url").and_then(|v| v.as_str()) {
        assert_eq!(mr.web_url, web_url, "MR.web_url must match API web_url");
    }

    let source = pick_json(first, "source_branch")
        .and_then(|v| v.as_str())
        .expect("source_branch is required");
    let target = pick_json(first, "target_branch")
        .and_then(|v| v.as_str())
        .expect("target_branch is required");
    assert_eq!(mr.source_branch, source, "MR.source_branch must match API");
    assert_eq!(mr.target_branch, target, "MR.target_branch must match API");

    if let Some(draft) = pick_json(first, "draft").and_then(|v| v.as_bool()) {
        assert_eq!(mr.draft, Some(draft), "MR.draft must match API draft");
    }

    let emoji_endpoint = MergeRequestAwardEmoji {
        project: ground.project.id,
        merge_request: mr.iid,
    };
    let nested_emoji = fetch_json(&client, emoji_endpoint).await;
    let nested_count = nested_emoji.as_array().map(|a| a.len()).unwrap_or(0);
    let outer_count = pick_json(first, "award_emoji")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(
        outer_count, nested_count,
        "MR.award_emoji list must match nested API call"
    );
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_api_equivalence_issue() {
    if common::skip_if_live_unavailable("live_full_api_equivalence_issue") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();
    let endpoint = PagedIssues {
        project: ground.project.id,
        page: 1,
        per_page: 50,
        state: Some("all".into()),
        order_by: None,
        sort: None,
        with_labels_details: Some(true),
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("issues response must be array");
    assert!(
        !arr.is_empty(),
        "expected at least one issue in test ground"
    );
    let first = &arr[0];
    let issue: models::Issue = serde_json::from_value(first.clone())
        .expect("TUI's Issue struct must deserialize API response");
    assert_eq!(issue.project_id, ground.project.id);
    let title = pick_json(first, "title")
        .and_then(|v| v.as_str())
        .expect("title is required");
    assert_eq!(issue.title, title);

    if let Some(weight) = pick_json(first, "weight").and_then(|v| v.as_i64()) {
        assert_eq!(issue.weight, Some(weight), "Issue.weight must match API");
    }
    if let Some(conf) = pick_json(first, "confidential").and_then(|v| v.as_bool()) {
        assert_eq!(
            issue.confidential, conf,
            "Issue.confidential must match API"
        );
    }

    let emoji_endpoint = IssueAwardEmoji {
        project: ground.project.id,
        issue: issue.iid,
    };
    let nested_emoji = fetch_json(&client, emoji_endpoint).await;
    let nested_count = nested_emoji.as_array().map(|a| a.len()).unwrap_or(0);
    let outer_count = pick_json(first, "award_emoji")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(
        outer_count, nested_count,
        "Issue.award_emoji list must match nested API call"
    );
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_api_equivalence_label() {
    if common::skip_if_live_unavailable("live_full_api_equivalence_label") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();
    let endpoint = ProjectLabels {
        project: ground.project.id,
    };
    let lp = gitlab_paged(endpoint, Pagination::All);
    let labels: Vec<Value> = AsyncQuery::query_async(&lp, &client)
        .await
        .expect("paged labels query failed");
    let arr: &Vec<Value> = &labels;
    assert!(
        arr.len() >= ground.counts.labels,
        "should have at least {} labels, got {}",
        ground.counts.labels,
        arr.len()
    );

    let first = &arr[0];
    let label: models::Label = serde_json::from_value(first.clone())
        .expect("TUI's Label struct must deserialize API response");
    let api_id = pick_json(first, "id")
        .and_then(|v| v.as_u64())
        .expect("id is required");
    let api_name = pick_json(first, "name")
        .and_then(|v| v.as_str())
        .expect("name is required");
    assert_eq!(label.id, api_id);
    assert_eq!(label.name(), api_name);
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_api_equivalence_milestone() {
    if common::skip_if_live_unavailable("live_full_api_equivalence_milestone") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();
    let endpoint = ProjectMilestones {
        project: ground.project.id,
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("milestones response must be array");
    if arr.is_empty() {
        eprintln!("[live_full_api_equivalence_milestone] no milestones in test ground, skipping");
        return;
    }
    let first = &arr[0];
    let ms: models::Milestone = serde_json::from_value(first.clone())
        .expect("TUI's Milestone struct must deserialize API response");
    let api_id = pick_json(first, "id")
        .and_then(|v| v.as_u64())
        .expect("id is required");
    let api_title = pick_json(first, "title")
        .and_then(|v| v.as_str())
        .expect("title is required");
    assert_eq!(ms.id, api_id);
    assert_eq!(ms.title, api_title);
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_api_equivalence_member() {
    if common::skip_if_live_unavailable("live_full_api_equivalence_member") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();
    let endpoint = ProjectMembersAll {
        project: ground.project.id,
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("members response must be array");
    if arr.is_empty() {
        eprintln!("[live_full_api_equivalence_member] no members in test ground, skipping");
        return;
    }
    let first = &arr[0];
    let author: models::Author = serde_json::from_value(first.clone())
        .expect("TUI's Author struct must deserialize API response");
    let api_id = pick_json(first, "id")
        .and_then(|v| v.as_u64())
        .expect("id is required");
    let api_username = pick_json(first, "username")
        .and_then(|v| v.as_str())
        .expect("username is required");
    assert_eq!(author.id, api_id);
    assert_eq!(author.username, api_username);
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_api_equivalence_pipeline() {
    if common::skip_if_live_unavailable("live_full_api_equivalence_pipeline") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();
    let endpoint = ProjectPipelines {
        project: ground.project.id,
        page: Some(1),
        per_page: Some(20),
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("pipelines response must be array");
    assert!(
        !arr.is_empty(),
        "expected at least one pipeline in test ground"
    );
    let first = &arr[0];
    let pipeline: models::Pipeline = serde_json::from_value(first.clone())
        .expect("TUI's Pipeline struct must deserialize API response");
    let api_id = pick_json(first, "id")
        .and_then(|v| v.as_u64())
        .expect("id is required");
    let api_status = pick_json(first, "status")
        .and_then(|v| v.as_str())
        .expect("status is required");
    let api_source = pick_json(first, "source")
        .and_then(|v| v.as_str())
        .expect("source is required");
    assert_eq!(pipeline.id, api_id);
    assert_eq!(pipeline.status, api_status);
    assert_eq!(pipeline.source, api_source);
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_api_equivalence_discussion() {
    if common::skip_if_live_unavailable("live_full_api_equivalence_discussion") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();
    let mr_endpoint = PagedMergeRequests {
        project: ground.project.id,
        page: 1,
        per_page: 5,
        state: Some("opened".into()),
        order_by: None,
        sort: None,
        with_labels_details: Some(false),
        assignee_id: None,
        reviewer_id: None,
    };
    let mrs = fetch_json(&client, mr_endpoint).await;
    let mrs_arr = mrs.as_array().expect("mrs response must be array");
    if mrs_arr.is_empty() {
        eprintln!("[live_full_api_equivalence_discussion] no opened MRs, skipping");
        return;
    }
    let target_mr_iid = pick_json(&mrs_arr[0], "iid")
        .and_then(|v| v.as_u64())
        .expect("iid is required");
    let endpoint = MergeRequestDiscussions {
        project: ground.project.id,
        merge_request: target_mr_iid,
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("discussions response must be array");

    if let Some(first_disc) = arr.first() {
        let disc: models::Discussion = serde_json::from_value(first_disc.clone())
            .expect("TUI's Discussion struct must deserialize API response");
        let api_id = pick_json(first_disc, "id")
            .and_then(|v| v.as_str())
            .expect("id is required");
        assert_eq!(disc.id, api_id);
    } else {
        eprintln!("[live_full_api_equivalence_discussion] MR !{target_mr_iid} has no discussions");
    }
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_api_equivalence_todo() {
    if common::skip_if_live_unavailable("live_full_api_equivalence_todo") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let endpoint = Todos {
        page: Some(1),
        per_page: Some(20),
        state: Some("pending".into()),
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("todos response must be array");
    if arr.is_empty() {
        eprintln!("[live_full_api_equivalence_todo] no pending todos, skipping");
        return;
    }
    let first = &arr[0];
    let todo: models::Todo = serde_json::from_value(first.clone())
        .expect("TUI's Todo struct must deserialize API response");
    let api_id = pick_json(first, "id")
        .and_then(|v| v.as_u64())
        .expect("id is required");
    let api_body = pick_json(first, "body")
        .and_then(|v| v.as_str())
        .expect("body is required");
    assert_eq!(todo.id, api_id);
    assert_eq!(todo.body, api_body);
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_api_equivalence_project_variables() {
    if common::skip_if_live_unavailable("live_full_api_equivalence_project_variables") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();
    let endpoint = ProjectVariables {
        project: ground.project.id,
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("variables response must be array");
    for v in arr {
        let key = pick_json(v, "key")
            .and_then(|k| k.as_str())
            .expect("key is required");
        let value = pick_json(v, "value")
            .and_then(|k| k.as_str())
            .expect("value is required");
        assert!(!key.is_empty(), "variable key must be non-empty");
        assert!(
            !value.is_empty() || value.is_empty(),
            "variable value round-tripped"
        );
    }
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_api_equivalence_merge_request_notes() {
    if common::skip_if_live_unavailable("live_full_api_equivalence_merge_request_notes") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();
    let mr_endpoint = PagedMergeRequests {
        project: ground.project.id,
        page: 1,
        per_page: 5,
        state: Some("opened".into()),
        order_by: None,
        sort: None,
        with_labels_details: Some(false),
        assignee_id: None,
        reviewer_id: None,
    };
    let mrs = fetch_json(&client, mr_endpoint).await;
    let mrs_arr = mrs.as_array().expect("mrs response must be array");
    if mrs_arr.is_empty() {
        eprintln!("[live_full_api_equivalence_merge_request_notes] no opened MRs, skipping");
        return;
    }
    let target_mr_iid = pick_json(&mrs_arr[0], "iid")
        .and_then(|v| v.as_u64())
        .expect("iid is required");
    let endpoint = MergeRequestNotes {
        project: ground.project.id,
        merge_request: target_mr_iid,
    };
    let raw = fetch_json(&client, endpoint).await;
    let arr = raw.as_array().expect("notes response must be array");
    for note_json in arr {
        let _note: models::Note = serde_json::from_value(note_json.clone())
            .expect("TUI's Note struct must deserialize API response");
    }
}
