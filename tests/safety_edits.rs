use gitlab::GitlabBuilder;
use lazygitlab::cache::ListCache;
use lazygitlab::models::{
    App, Author, Context, DataUpdate, GroupedItem, Issue, Label, MenuSelection, MergeRequest,
    Panel, Project, ProjectVariable,
};
use std::collections::BTreeSet;
use tokio::sync::mpsc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Rx = mpsc::UnboundedReceiver<DataUpdate>;

async fn setup_app() -> (MockServer, App, Rx) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"})),
        )
        .mount(&server)
        .await;
    let client = GitlabBuilder::new(server.uri().replace("http://", ""), "t")
        .insecure()
        .build_async()
        .await
        .unwrap();
    let (tx, rx) = mpsc::unbounded_channel();
    let app = App::new(client, tx, None);
    (server, app, rx)
}

fn label(id: u64, name: &str) -> Label {
    Label {
        id,
        name: name.into(),
        color: "#112233".into(),
        text_color: None,
    }
}

fn dummy_mr(iid: u64, title: &str) -> MergeRequest {
    MergeRequest {
        updated_at: None,
        iid,
        project_id: 1,
        title: title.into(),
        description: None,
        author: Author::new(1, "u".into()),
        web_url: "u".into(),
        state: "opened".into(),
        references: None,
        upvotes: Some(0),
        downvotes: Some(0),
        labels: vec![],
        milestone: None,
        assignees: vec![],
        reviewers: vec![],
        squash: Some(false),
        created_at: Some("t".into()),
        draft: Some(false),
        diff_refs: None,
        reactions: vec![],
        merge_status: None,
        detailed_merge_status: None,
        has_conflicts: None,
        head_pipeline: None,
        source_branch: "source".into(),
        target_branch: "target".into(),
        participants: vec![],
    }
}

fn split(v: &str) -> Vec<String> {
    v.split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

fn resulting_labels(body: &[u8], current: &[&str]) -> BTreeSet<String> {
    let pairs: Vec<(String, String)> = url::form_urlencoded::parse(body).into_owned().collect();
    let get = |k: &str| {
        pairs
            .iter()
            .find(|(key, _)| key == k)
            .map(|(_, v)| v.clone())
    };
    let mut set: BTreeSet<String> = match get("labels") {
        Some(l) => split(&l).into_iter().collect(),
        None => current.iter().map(|s| s.to_string()).collect(),
    };
    if let Some(a) = get("add_labels") {
        set.extend(split(&a));
    }
    if let Some(r) = get("remove_labels") {
        for l in split(&r) {
            set.remove(&l);
        }
    }
    set
}

async fn last_put_body(server: &MockServer) -> Vec<u8> {
    server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .rev()
        .find(|r| r.method.as_str() == "PUT")
        .expect("a PUT request")
        .body
}

fn issue(project_id: u64, iid: u64, title: &str) -> Issue {
    serde_json::from_value(serde_json::json!({
        "iid": iid,
        "project_id": project_id,
        "title": title,
        "author": {"id": 1, "username": "u"},
        "state": "opened"
    }))
    .unwrap()
}

fn project(id: u64) -> Project {
    Project {
        id,
        name: format!("p{id}"),
        path_with_namespace: Some(format!("g/p{id}")),
        web_url: None,
        default_branch: Some("main".into()),
        namespace: None,
    }
}

async fn drain(app: &mut App, rx: &mut Rx) {
    while let Ok(update) = rx.try_recv() {
        app.process_update_single(update).await;
    }
}

async fn pump(app: &mut App, rx: &mut Rx) {
    for _ in 0..40 {
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        drain(app, rx).await;
    }
}

async fn mount_put_ok(server: &MockServer, p: &str) {
    Mock::given(method("PUT"))
        .and(path(p))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(server)
        .await;
}

fn form(body: &[u8], key: &str) -> Option<String> {
    url::form_urlencoded::parse(body)
        .into_owned()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
}

fn set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|s| s.to_string()).collect()
}

fn mr_with_labels(app: &mut App, labels: &[(u64, &str)], popup: &[(u64, &str)]) {
    let mut mr = dummy_mr(1, "t");
    mr.labels = labels.iter().map(|(i, n)| label(*i, n)).collect();
    app.merge_requests = vec![mr.clone()];
    app.selected_mr = Some(mr);
    app.project_label_cache
        .insert(1, popup.iter().map(|(i, n)| label(*i, n)).collect());
    app.open_label_popup();
}

#[tokio::test]
async fn label_toggle_keeps_labels_missing_from_popup_list() {
    let (server, mut app, _rx) = setup_app().await;
    mount_put_ok(&server, "/api/v4/projects/1/merge_requests/1").await;
    mr_with_labels(
        &mut app,
        &[(0, "bug"), (0, "ui")],
        &[(7, "bug"), (8, "feature")],
    );
    app.label_state.select(Some(1));
    app.toggle_popup_label();
    app.update_mr_labels().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let body = last_put_body(&server).await;
    assert_eq!(
        resulting_labels(&body, &["bug", "ui"]),
        set(&["bug", "ui", "feature"])
    );
    assert_eq!(form(&body, "add_labels").as_deref(), Some("feature"));
    assert!(form(&body, "labels").is_none());
    assert!(form(&body, "remove_labels").is_none());
}

#[tokio::test]
async fn label_untoggle_removes_only_that_label() {
    let (server, mut app, _rx) = setup_app().await;
    mount_put_ok(&server, "/api/v4/projects/1/merge_requests/1").await;
    mr_with_labels(
        &mut app,
        &[(0, "bug"), (0, "ui")],
        &[(7, "bug"), (8, "feature")],
    );
    app.label_state.select(Some(0));
    app.toggle_popup_label();
    app.update_mr_labels().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let body = last_put_body(&server).await;
    assert_eq!(resulting_labels(&body, &["bug", "ui"]), set(&["ui"]));
    assert_eq!(form(&body, "remove_labels").as_deref(), Some("bug"));
}

#[tokio::test]
async fn id_zero_labels_toggle_independently() {
    let (server, mut app, _rx) = setup_app().await;
    mount_put_ok(&server, "/api/v4/projects/1/merge_requests/1").await;
    mr_with_labels(&mut app, &[(0, "bug")], &[(0, "bug"), (0, "feature")]);
    app.label_state.select(Some(1));
    app.toggle_popup_label();
    assert!(app.label_checked(&label(0, "bug")));
    assert!(app.label_checked(&label(0, "feature")));
    app.update_mr_labels().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let body = last_put_body(&server).await;
    assert_eq!(resulting_labels(&body, &["bug"]), set(&["bug", "feature"]));
}

#[tokio::test]
async fn issue_label_toggle_keeps_unlisted_labels() {
    let (server, mut app, _rx) = setup_app().await;
    mount_put_ok(&server, "/api/v4/projects/1/issues/3").await;
    let mut i = issue(1, 3, "t");
    i.labels = vec![label(0, "bug"), label(0, "ui")];
    app.issues = vec![i.clone()];
    app.selected_issue = Some(i);
    app.project_label_cache
        .insert(1, vec![label(7, "bug"), label(8, "feature")]);
    app.open_label_popup();
    app.label_state.select(Some(1));
    app.toggle_popup_label();
    app.update_issue_labels().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let body = last_put_body(&server).await;
    assert_eq!(
        resulting_labels(&body, &["bug", "ui"]),
        set(&["bug", "ui", "feature"])
    );
}

#[tokio::test]
async fn failed_edit_rolls_back_and_shows_error() {
    let (server, mut app, mut rx) = setup_app().await;
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/merge_requests/1"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let mr = dummy_mr(1, "old title");
    app.merge_requests = vec![mr.clone()];
    app.selected_mr = Some(mr);
    app.input_target = lazygitlab::models::InputTarget::EditTitle;
    app.textarea = ratatui_textarea::TextArea::from(vec!["new title"]);
    app.update_mr().await.unwrap();
    assert_eq!(app.selected_mr.as_ref().unwrap().title, "new title");
    pump(&mut app, &mut rx).await;
    assert_eq!(app.selected_mr.as_ref().unwrap().title, "old title");
    assert_eq!(app.merge_requests[0].title, "old title");
    assert!(app.last_error.is_some());
}

#[tokio::test]
async fn failed_issue_label_edit_rolls_back() {
    let (server, mut app, mut rx) = setup_app().await;
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/issues/3"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let mut i = issue(1, 3, "t");
    i.labels = vec![label(7, "bug")];
    app.issues = vec![i.clone()];
    app.selected_issue = Some(i);
    app.project_label_cache
        .insert(1, vec![label(7, "bug"), label(8, "feature")]);
    app.open_label_popup();
    app.label_state.select(Some(1));
    app.toggle_popup_label();
    app.update_issue_labels().await.unwrap();
    assert_eq!(app.selected_issue.as_ref().unwrap().labels.len(), 2);
    pump(&mut app, &mut rx).await;
    assert_eq!(app.selected_issue.as_ref().unwrap().labels.len(), 1);
    assert_eq!(app.issues[0].labels.len(), 1);
    assert!(app.last_error.is_some());
}

#[tokio::test]
async fn pending_and_unconfirmed_rows_never_reach_the_cache_file() {
    let (_server, mut app, _rx) = setup_app().await;
    let dir = std::env::temp_dir()
        .join(format!("lgl-safety-{}", std::process::id()))
        .join("cache")
        .join("v1");
    let _ = std::fs::remove_dir_all(&dir);
    let cache = std::sync::Arc::new(ListCache::load(
        &dir,
        "https://h.example",
        std::num::NonZeroU64::new(7),
    ));
    app.list_cache = Some(cache.clone());
    let real = dummy_mr(5, "edited optimistically");
    let pending = dummy_mr(1_000_006, "fake");
    app.merge_requests = vec![pending, real];
    let snapshot = dummy_mr(5, "confirmed title");
    app.mr_edit_snapshots.insert((1, 5), snapshot);
    app.write_mr_cache("k", None);
    cache.flush().unwrap();
    let file = cache.path().unwrap();
    let raw = std::fs::read_to_string(file).unwrap();
    assert!(!raw.contains("fake"), "pending row leaked: {raw}");
    assert!(!raw.contains("1000006"));
    assert!(
        !raw.contains("edited optimistically"),
        "unconfirmed edit leaked"
    );
    assert!(raw.contains("confirmed title"));
}

#[tokio::test]
async fn remove_pending_matches_project_and_iid() {
    let (_server, mut app, mut rx) = setup_app().await;
    app.issues = vec![issue(1, 1_000_001, "a"), issue(2, 1_000_001, "b")];
    let mut m1 = dummy_mr(1_000_001, "a");
    m1.project_id = 1;
    let mut m2 = dummy_mr(1_000_001, "b");
    m2.project_id = 2;
    app.merge_requests = vec![m1, m2];
    let _ = app.tx.send(DataUpdate::RemovePendingIssue {
        project_id: 1,
        iid: 1_000_001,
    });
    let _ = app.tx.send(DataUpdate::RemovePendingMR {
        project_id: 2,
        iid: 1_000_001,
    });
    drain(&mut app, &mut rx).await;
    assert_eq!(app.issues.len(), 1);
    assert_eq!(app.issues[0].project_id, 2);
    assert_eq!(app.merge_requests.len(), 1);
    assert_eq!(app.merge_requests[0].project_id, 1);
}

#[tokio::test]
async fn detail_without_participants_keeps_existing_ones() {
    let (_server, mut app, mut rx) = setup_app().await;
    let mut mr = dummy_mr(1, "t");
    mr.participants = vec![Author::new(9, "alice".into())];
    mr.reactions = vec![lazygitlab::models::AwardEmoji {
        name: "thumbsup".into(),
    }];
    mr.updated_at = Some("2025-01-01T00:00:00Z".into());
    app.merge_requests = vec![mr.clone()];
    app.selected_mr = Some(mr);
    let mut incoming = dummy_mr(1, "renamed");
    incoming.updated_at = Some("2025-01-02T00:00:00Z".into());
    let _ = app.tx.send(DataUpdate::MRDetail(incoming));
    drain(&mut app, &mut rx).await;
    let sel = app.selected_mr.as_ref().unwrap();
    assert_eq!(sel.title, "renamed");
    assert_eq!(sel.participants.len(), 1);
    assert_eq!(sel.reactions.len(), 1);
    assert_eq!(app.merge_requests[0].title, "renamed");
    assert_eq!(app.merge_requests[0].participants.len(), 1);
}

#[tokio::test]
async fn stale_list_row_does_not_revert_a_rename_in_preview() {
    let (_server, mut app, _rx) = setup_app().await;
    let mut detail = dummy_mr(1, "renamed");
    detail.updated_at = Some("2025-01-02T00:00:00Z".into());
    let mut row = dummy_mr(1, "old name");
    row.updated_at = Some("2025-01-01T00:00:00Z".into());
    app.active_panel = Panel::Main;
    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::MergeRequests;
    app.dashboard_menu_state.select(Some(2));
    app.merge_requests = vec![row];
    app.filter_lists();
    let idx = app
        .filtered_mrs
        .iter()
        .position(|g| matches!(g, GroupedItem::Item(_)))
        .unwrap();
    app.mrs_state.select(Some(idx));
    app.selected_mr = Some(detail);
    app.update_preview().await;
    assert_eq!(app.selected_mr.as_ref().unwrap().title, "renamed");
}

#[tokio::test]
async fn issue_detail_after_edit_updates_the_issue_row() {
    let (_server, mut app, mut rx) = setup_app().await;
    let i = issue(1, 3, "old");
    app.issues = vec![i.clone()];
    app.selected_issue = Some(i);
    let mut fresh = issue(1, 3, "renamed");
    fresh.updated_at = Some("2025-01-02T00:00:00Z".into());
    let _ = app.tx.send(DataUpdate::IssueDetail(fresh));
    drain(&mut app, &mut rx).await;
    assert_eq!(app.issues[0].title, "renamed");
}

#[tokio::test]
async fn project_switch_resets_project_scope() {
    let (_server, mut app, _rx) = setup_app().await;
    app.projects = vec![project(1), project(2)];
    app.projects_state.select(Some(0));
    app.project_labels = vec![label(1, "only-in-a")];
    app.project_milestones = vec![lazygitlab::models::Milestone {
        id: 1,
        title: "m".into(),
    }];
    app.project_members = vec![Author::new(1, "a".into())];
    app.project_variables = vec![ProjectVariable {
        key: "K".into(),
        value: "v".into(),
        variable_type: "env_var".into(),
        protected: false,
        masked: false,
        environment_scope: "*".into(),
    }];
    app.pipeline_schedules = vec![
        serde_json::from_value(serde_json::json!({"id": 1, "description": "nightly"})).unwrap(),
    ];
    app.mr_meta.insert(
        (1, 1),
        lazygitlab::models::MrMeta {
            pipeline_id: Some(1),
            milestone_id: None,
        },
    );
    app.repository_ref = Some("main".into());
    app.select_project().await;
    assert!(app.project_labels.is_empty());
    assert!(app.project_milestones.is_empty());
    assert!(app.project_members.is_empty());
    assert!(app.project_variables.is_empty());
    assert!(app.pipeline_schedules.is_empty());
    assert!(app.mr_meta.is_empty());
    assert!(app.repository_ref.is_none());
}

#[tokio::test]
async fn stale_variable_update_for_another_project_is_ignored() {
    let (_server, mut app, mut rx) = setup_app().await;
    app.selected_project = Some(project(2));
    let _ = app.tx.send(DataUpdate::VariableSaved {
        project_id: 1,
        key: "LEAK".into(),
        value: "x".into(),
    });
    let _ = app.tx.send(DataUpdate::VariableDeleted {
        project_id: 1,
        key: "OTHER".into(),
    });
    drain(&mut app, &mut rx).await;
    assert!(app.project_variables.is_empty());
    assert!(app.status_message.is_none());
    let _ = app.tx.send(DataUpdate::VariableSaved {
        project_id: 2,
        key: "OK".into(),
        value: "y".into(),
    });
    drain(&mut app, &mut rx).await;
    assert_eq!(app.project_variables.len(), 1);
}

#[tokio::test]
async fn metadata_cache_expires() {
    let (_server, mut app, _rx) = setup_app().await;
    app.metadata_fetched_at.insert(
        1,
        std::time::Instant::now() - std::time::Duration::from_secs(601),
    );
    assert!(!app.metadata_fresh(1));
    app.metadata_fetched_at.insert(1, std::time::Instant::now());
    assert!(app.metadata_fresh(1));
    assert!(!app.metadata_fresh(2));
}
