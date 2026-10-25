use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_app() -> (
    MockServer,
    App,
    tokio::sync::mpsc::UnboundedReceiver<DataUpdate>,
) {
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
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    (server, App::new(client, tx, None), rx)
}

fn mr(project: u64, iid: u64) -> serde_json::Value {
    serde_json::json!({"iid": iid, "project_id": project, "title": format!("MR {}", iid), "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "source_branch": "a", "target_branch": "b", "draft": false, "created_at": format!("2026-01-{:02}T00:00:00Z", iid.min(28))})
}

fn issue(project: u64, iid: u64) -> serde_json::Value {
    serde_json::json!({"iid": iid, "project_id": project, "title": format!("Issue {}", iid), "author": {"id": 1, "username": "u"}, "web_url": "u", "state": "opened", "created_at": format!("2026-01-{:02}T00:00:00Z", iid.min(28))})
}

fn mrs(project: u64, iids: &[u64]) -> Vec<MergeRequest> {
    iids.iter()
        .map(|i| serde_json::from_value(mr(project, *i)).unwrap())
        .collect()
}

fn issues(project: u64, iids: &[u64]) -> Vec<Issue> {
    iids.iter()
        .map(|i| serde_json::from_value(issue(project, *i)).unwrap())
        .collect()
}

async fn settle(app: &mut App, rx: &mut tokio::sync::mpsc::UnboundedReceiver<DataUpdate>) {
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let start = std::time::Instant::now();
    loop {
        app.process_updates(rx).await;
        if !app.is_loading || start.elapsed() > std::time::Duration::from_secs(10) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    app.process_updates(rx).await;
}

fn dashboard(app: &mut App, sel: MenuSelection) {
    app.active_context = Context::Dashboard;
    app.active_panel = Panel::Main;
    app.active_selection = sel;
    app.member_project_ids = [1u64, 2].into_iter().collect();
    app.membership_sync_attempted = true;
}

fn project_ctx(app: &mut App, sel: MenuSelection) {
    app.active_context = Context::Project;
    app.active_panel = Panel::Main;
    app.active_selection = sel;
    app.selected_project =
        Some(serde_json::from_value(serde_json::json!({"id": 1, "name": "p"})).unwrap());
}

fn mr_ids(app: &App) -> Vec<u64> {
    let mut v: Vec<u64> = app
        .filtered_mrs
        .iter()
        .filter_map(|g| match g {
            GroupedItem::Item(m) => Some(m.iid),
            _ => None,
        })
        .collect();
    v.sort();
    v
}

fn issue_ids(app: &App) -> Vec<u64> {
    let mut v: Vec<u64> = app
        .filtered_issues
        .iter()
        .filter_map(|g| match g {
            GroupedItem::Item(i) => Some(i.iid),
            _ => None,
        })
        .collect();
    v.sort();
    v
}

fn assert_failed_status(app: &App) {
    let msg = app.status_message.clone().unwrap_or_default();
    assert!(
        msg.starts_with("refresh failed, showing data from"),
        "status was {:?}",
        msg
    );
}

#[tokio::test]
async fn global_mrs_partial_failure_keeps_rows_of_failed_project() {
    let (server, mut app, mut rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::MergeRequests);
    app.process_update_single(DataUpdate::GlobalMRs {
        data: [mrs(1, &[1, 2]), mrs(2, &[3, 4, 5])].concat(),
        page: 1,
        has_more: false,
    })
    .await;
    assert_eq!(mr_ids(&app), vec![1, 2, 3, 4, 5]);

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/merge_requests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![mr(1, 1), mr(1, 2)]))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/2/merge_requests"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(mr_ids(&app), vec![1, 2, 3, 4, 5]);
    assert_failed_status(&app);
}

#[tokio::test]
async fn global_mrs_total_failure_keeps_list() {
    let (server, mut app, mut rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::MergeRequests);
    app.process_update_single(DataUpdate::GlobalMRs {
        data: mrs(1, &[1, 2, 3]),
        page: 1,
        has_more: false,
    })
    .await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(mr_ids(&app), vec![1, 2, 3]);
    let msg = app.status_message.clone().unwrap_or_default();
    assert!(msg.starts_with("Rate limited by GitLab"), "status was {msg:?}");
    assert!(app.mrs_stale);
}

#[tokio::test]
async fn global_issues_total_failure_keeps_list() {
    let (server, mut app, mut rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::Issues);
    app.process_update_single(DataUpdate::GlobalIssues {
        data: issues(1, &[1, 2, 3]),
        page: 1,
        has_more: false,
    })
    .await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(502))
        .mount(&server)
        .await;
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(issue_ids(&app), vec![1, 2, 3]);
    assert_failed_status(&app);
}

#[tokio::test]
async fn global_issues_partial_failure_keeps_rows_of_failed_project() {
    let (server, mut app, mut rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::Issues);
    app.process_update_single(DataUpdate::GlobalIssues {
        data: [issues(1, &[1]), issues(2, &[2, 3])].concat(),
        page: 1,
        has_more: false,
    })
    .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/issues"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![issue(1, 1), issue(1, 4)]))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/2/issues"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(issue_ids(&app), vec![1, 2, 3, 4]);
    assert_failed_status(&app);
}

#[tokio::test]
async fn failed_membership_sync_does_not_empty_member_set_or_lists() {
    let (_server, mut app, _rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::MergeRequests);
    app.process_update_single(DataUpdate::GlobalMRs {
        data: mrs(1, &[1, 2]),
        page: 1,
        has_more: false,
    })
    .await;
    app.process_update_single(DataUpdate::MemberSyncFailed)
        .await;

    assert!(app.member_project_ids.contains(&1));
    assert_eq!(mr_ids(&app), vec![1, 2]);
}

#[tokio::test]
async fn global_refresh_with_empty_membership_does_not_clear_list() {
    let (_server, mut app, mut rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::MergeRequests);
    app.process_update_single(DataUpdate::GlobalMRs {
        data: mrs(1, &[1, 2]),
        page: 1,
        has_more: false,
    })
    .await;
    app.member_project_ids.clear();
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(mr_ids(&app), vec![1, 2]);
}

#[tokio::test]
async fn project_mrs_server_error_keeps_list_and_reports() {
    let (server, mut app, mut rx) = setup_app().await;
    project_ctx(&mut app, MenuSelection::MergeRequests);
    app.process_update_single(DataUpdate::ProjectMRs {
        project_id: 1,
        data: mrs(1, &(1..=20).collect::<Vec<_>>()),
        page: 1,
        has_more: false,
    })
    .await;
    assert_eq!(mr_ids(&app).len(), 20, "precondition");
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(mr_ids(&app).len(), 20);
    assert_failed_status(&app);
}

#[tokio::test]
async fn project_issues_server_error_keeps_list_and_reports() {
    let (server, mut app, mut rx) = setup_app().await;
    project_ctx(&mut app, MenuSelection::Issues);
    app.process_update_single(DataUpdate::ProjectIssues {
        project_id: 1,
        data: issues(1, &[1, 2, 3]),
        page: 1,
        has_more: false,
    })
    .await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(issue_ids(&app), vec![1, 2, 3]);
    assert_failed_status(&app);
}

#[tokio::test]
async fn successful_empty_refresh_still_empties_list() {
    let (_server, mut app, _rx) = setup_app().await;
    project_ctx(&mut app, MenuSelection::MergeRequests);
    app.process_update_single(DataUpdate::ProjectMRs {
        project_id: 1,
        data: mrs(1, &[1, 2]),
        page: 1,
        has_more: false,
    })
    .await;
    app.process_update_single(DataUpdate::ProjectMRs {
        project_id: 1,
        data: vec![],
        page: 1,
        has_more: false,
    })
    .await;
    assert!(mr_ids(&app).is_empty());
}

#[tokio::test]
async fn selection_follows_item_id_when_list_shifts() {
    let (_server, mut app, _rx) = setup_app().await;
    project_ctx(&mut app, MenuSelection::MergeRequests);
    app.process_update_single(DataUpdate::ProjectMRs {
        project_id: 1,
        data: mrs(1, &[3, 2, 1]),
        page: 1,
        has_more: false,
    })
    .await;
    app.mrs_state.select(Some(1));
    app.process_update_single(DataUpdate::ProjectMRs {
        project_id: 1,
        data: mrs(1, &[5, 4, 3, 2, 1]),
        page: 1,
        has_more: false,
    })
    .await;
    let idx = app.mrs_state.selected().unwrap();
    match &app.filtered_mrs[idx] {
        GroupedItem::Item(m) => assert_eq!(m.iid, 2),
        _ => panic!("header selected"),
    }
}

#[tokio::test]
async fn selection_follows_issue_id_when_list_shrinks() {
    let (_server, mut app, _rx) = setup_app().await;
    project_ctx(&mut app, MenuSelection::Issues);
    app.process_update_single(DataUpdate::ProjectIssues {
        project_id: 1,
        data: issues(1, &[4, 3, 2, 1]),
        page: 1,
        has_more: false,
    })
    .await;
    app.issues_state.select(Some(2));
    app.process_update_single(DataUpdate::ProjectIssues {
        project_id: 1,
        data: issues(1, &[3, 2]),
        page: 1,
        has_more: false,
    })
    .await;
    let idx = app.issues_state.selected().unwrap();
    match &app.filtered_issues[idx] {
        GroupedItem::Item(i) => assert_eq!(i.iid, 2),
        _ => panic!("header selected"),
    }
}

fn todo(id: u64) -> Todo {
    serde_json::from_value(serde_json::json!({"id": id, "action_name": "mentioned", "target_type": "Issue", "target_url": "http://x/issues/1", "body": "b", "state": "pending", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z", "project": {"id": 1, "name": "p", "path_with_namespace": "g/p"}, "author": {"id": 2, "username": "a"}})).unwrap()
}

fn pipeline_json(project: u64, id: u64) -> serde_json::Value {
    serde_json::json!({"id": id, "project_id": project, "status": "success", "source": "push", "ref": "main", "web_url": "u", "created_at": format!("2026-01-{:02}T00:00:00Z", id.min(28))})
}

fn pipeline(project: u64, id: u64) -> Pipeline {
    serde_json::from_value(pipeline_json(project, id)).unwrap()
}

#[tokio::test]
async fn cross_pipelines_partial_result_keeps_rows_of_unanswered_project() {
    let (server, mut app, mut rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::Pipelines);
    app.process_update_single(DataUpdate::GlobalPipelines {
        data: vec![pipeline(1, 3), pipeline(2, 2), pipeline(2, 1)],
        page: 1,
        has_more: false,
    })
    .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/pipelines"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![pipeline_json(1, 3)]))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/2/pipelines"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(app.cross_pipelines.len(), 3);
    assert_failed_status(&app);
}

#[tokio::test]
async fn error_update_does_not_cancel_other_loading_tasks() {
    let (_server, mut app, _rx) = setup_app().await;
    app.process_update_single(DataUpdate::Loading(true)).await;
    app.process_update_single(DataUpdate::Loading(true)).await;
    app.process_update_single(DataUpdate::Error("x fetch failed".into()))
        .await;
    assert!(app.is_loading, "one task is still running");
    app.process_update_single(DataUpdate::Loading(false)).await;
    app.process_update_single(DataUpdate::Loading(false)).await;
    assert!(!app.is_loading);
}

#[tokio::test]
async fn global_next_page_is_not_truncated_to_fifty_rows() {
    let (server, mut app, mut rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::MergeRequests);
    app.mrs_has_more = true;
    app.mrs_page = 1;
    for pid in [1u64, 2] {
        let body: Vec<_> = (1..=40).map(|i| mr(pid, i)).collect();
        Mock::given(method("GET"))
            .and(path(format!("/api/v4/projects/{}/merge_requests", pid)))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;
    }
    app.fetch_next_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    assert_eq!(app.merge_requests.len(), 80);
}

#[tokio::test]
async fn result_for_a_filter_the_user_left_is_dropped() {
    let (_server, mut app, _rx) = setup_app().await;
    project_ctx(&mut app, MenuSelection::MergeRequests);
    app.mr_filter = MergeRequestFilter::Merged;
    app.process_update_single(DataUpdate::ScopedMr(
        MergeRequestFilter::Opened,
        Box::new(DataUpdate::ProjectMRs {
            project_id: 1,
            data: mrs(1, &[1, 2]),
            page: 1,
            has_more: false,
        }),
    ))
    .await;
    assert!(mr_ids(&app).is_empty());
    app.process_update_single(DataUpdate::ScopedMr(
        MergeRequestFilter::Merged,
        Box::new(DataUpdate::ProjectMRs {
            project_id: 1,
            data: mrs(1, &[1, 2]),
            page: 1,
            has_more: false,
        }),
    ))
    .await;
    assert_eq!(mr_ids(&app), vec![1, 2]);
}

#[tokio::test]
async fn project_that_fails_every_pass_does_not_freeze_other_projects() {
    let (server, mut app, mut rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::MergeRequests);
    app.process_update_single(DataUpdate::GlobalMRs {
        data: [mrs(1, &[1, 2]), mrs(2, &[3])].concat(),
        page: 1,
        has_more: false,
    })
    .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/merge_requests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![mr(1, 1)]))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/2/merge_requests"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;
    app.refresh_background().await;
    settle(&mut app, &mut rx).await;

    assert_eq!(mr_ids(&app), vec![1, 3]);
    assert_failed_status(&app);
}

#[tokio::test]
async fn cross_pipelines_partial_merge_is_capped_at_fifty() {
    let (_server, mut app, _rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::Pipelines);
    app.process_update_single(DataUpdate::MergeGlobalPipelines(
        (1..=60).map(|i| pipeline(1, i)).collect(),
    ))
    .await;
    assert_eq!(app.cross_pipelines.len(), 50);
}

#[tokio::test]
async fn successful_empty_membership_clears_the_set() {
    let (_server, mut app, _rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::MergeRequests);
    app.process_update_single(DataUpdate::MemberProjectIds(Default::default()))
        .await;
    assert!(app.member_project_ids.is_empty());
}

#[tokio::test]
async fn refresh_failure_for_a_filter_the_user_left_is_not_shown() {
    let (_server, mut app, _rx) = setup_app().await;
    app.mr_filter = MergeRequestFilter::Merged;
    app.process_update_single(DataUpdate::ScopedMr(
        MergeRequestFilter::Opened,
        Box::new(DataUpdate::RefreshFailed),
    ))
    .await;
    assert!(app.status_message.is_none());
}

fn mr_at(project: u64, iid: u64, created_day: u64, updated_day: u64) -> MergeRequest {
    let mut v = mr(project, iid);
    v["created_at"] = format!("2026-01-{:02}T00:00:00Z", created_day).into();
    v["updated_at"] = format!("2026-02-{:02}T00:00:00Z", updated_day).into();
    serde_json::from_value(v).unwrap()
}

fn visible_mr_order(app: &App) -> Vec<u64> {
    app.filtered_mrs
        .iter()
        .filter_map(|g| match g {
            GroupedItem::Item(m) => Some(m.iid),
            _ => None,
        })
        .collect()
}

#[tokio::test]
async fn recently_updated_rows_stay_on_top_after_loading_page_two() {
    let (_server, mut app, _rx) = setup_app().await;
    project_ctx(&mut app, MenuSelection::MergeRequests);
    app.process_update_single(DataUpdate::ProjectMRs {
        project_id: 1,
        data: vec![mr_at(1, 1, 1, 20), mr_at(1, 2, 2, 19)],
        page: 1,
        has_more: true,
    })
    .await;
    app.process_update_single(DataUpdate::AppendProjectMRs {
        project_id: 1,
        data: vec![mr_at(1, 3, 25, 10)],
    })
    .await;
    assert_eq!(visible_mr_order(&app), vec![1, 2, 3]);
}

#[tokio::test]
async fn failed_global_next_page_is_retried_with_the_same_page() {
    let (server, mut app, mut rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::MergeRequests);
    app.mrs_has_more = true;
    app.process_update_single(DataUpdate::GlobalMRs {
        data: mrs(1, &[1]),
        page: 1,
        has_more: true,
    })
    .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/merge_requests"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/merge_requests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![mr(1, 2)]))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/2/merge_requests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<serde_json::Value>::new()))
        .mount(&server)
        .await;

    app.fetch_next_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;
    assert_failed_status(&app);
    app.fetch_next_global_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;

    assert_eq!(mr_ids(&app), vec![1, 2]);
    let pages: Vec<String> = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path() == "/api/v4/projects/1/merge_requests")
        .filter_map(|r| {
            r.url
                .query_pairs()
                .find(|(k, _)| k == "page")
                .map(|(_, v)| v.to_string())
        })
        .collect();
    assert_eq!(pages, vec!["2", "2"], "the failed page is requested again");
}

#[tokio::test]
async fn switching_filter_restarts_paging_at_page_two() {
    let (server, mut app, mut rx) = setup_app().await;
    project_ctx(&mut app, MenuSelection::MergeRequests);
    app.mrs_page = 3;
    app.mrs_has_more = false;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![mr(1, 1)]))
        .mount(&server)
        .await;
    app.handle_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char(']'),
        crossterm::event::KeyModifiers::NONE,
    ))
    .await;
    app.fetch_next_merge_requests().await.unwrap();
    settle(&mut app, &mut rx).await;

    let pages: Vec<String> = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter_map(|r| {
            r.url
                .query_pairs()
                .find(|(k, _)| k == "page")
                .map(|(_, v)| v.to_string())
        })
        .collect();
    assert!(pages.contains(&"2".to_string()), "pages: {:?}", pages);
    assert!(!pages.contains(&"4".to_string()), "pages: {:?}", pages);
}

fn ts(day: &str, n: u64) -> String {
    format!("{}T{:02}:{:02}:00Z", day, n / 60, n % 60)
}

fn mr_n(project: u64, iid: u64) -> MergeRequest {
    let mut v = mr(project, iid);
    v["created_at"] = ts("2026-01-01", 1439 - iid).into();
    v["updated_at"] = ts("2026-02-01", iid).into();
    serde_json::from_value(v).unwrap()
}

fn issue_n(project: u64, iid: u64) -> Issue {
    let mut v = issue(project, iid);
    v["created_at"] = ts("2026-01-01", 1439 - iid).into();
    v["updated_at"] = ts("2026-02-01", iid).into();
    serde_json::from_value(v).unwrap()
}

fn range(hi: u64, lo: u64) -> impl Iterator<Item = u64> {
    (lo..=hi).rev()
}

#[tokio::test]
async fn scenario_120_mrs_paged_then_one_updated_and_one_merged() {
    let (_server, mut app, _rx) = setup_app().await;
    project_ctx(&mut app, MenuSelection::MergeRequests);
    let rows = |hi, lo| range(hi, lo).map(|i| mr_n(1, i)).collect::<Vec<_>>();
    app.process_update_single(DataUpdate::ProjectMRs {
        project_id: 1,
        data: rows(120, 71),
        page: 1,
        has_more: true,
    })
    .await;
    app.process_update_single(DataUpdate::AppendProjectMRs {
        project_id: 1,
        data: rows(70, 21),
    })
    .await;
    assert_eq!(visible_mr_order(&app).len(), 100);

    let mut updated = mr(1, 100);
    updated["created_at"] = ts("2026-01-01", 1439 - 100).into();
    updated["updated_at"] = ts("2026-02-02", 0).into();
    let mut fresh: Vec<MergeRequest> = vec![serde_json::from_value(updated).unwrap()];
    fresh.extend(
        range(120, 71)
            .filter(|i| *i != 100 && *i != 80)
            .map(|i| mr_n(1, i)),
    );
    fresh.push(mr_n(1, 70));
    assert_eq!(fresh.len(), 50);
    app.process_update_single(DataUpdate::ProjectMRs {
        project_id: 1,
        data: fresh,
        page: 1,
        has_more: true,
    })
    .await;

    let order = visible_mr_order(&app);
    assert_eq!(order[0], 100, "updated row jumps to the top");
    assert_eq!(
        order.iter().filter(|i| **i == 100).count(),
        1,
        "no duplicate"
    );
    assert!(!order.contains(&80), "merged row leaves");
    assert_eq!(order.len(), 99);
    for i in 21..=69 {
        assert!(order.contains(&i), "row {} from page two is kept", i);
    }
}

#[tokio::test]
async fn short_first_page_is_the_whole_list_so_the_tail_goes() {
    let (_server, mut app, _rx) = setup_app().await;
    project_ctx(&mut app, MenuSelection::MergeRequests);
    app.process_update_single(DataUpdate::ProjectMRs {
        project_id: 1,
        data: vec![mr_n(1, 10), mr_n(1, 9), mr_n(1, 8)],
        page: 1,
        has_more: true,
    })
    .await;
    app.process_update_single(DataUpdate::AppendProjectMRs {
        project_id: 1,
        data: vec![mr_n(1, 7), mr_n(1, 6)],
    })
    .await;
    app.process_update_single(DataUpdate::ProjectMRs {
        project_id: 1,
        data: vec![mr_n(1, 10), mr_n(1, 8), mr_n(1, 7)],
        page: 1,
        has_more: false,
    })
    .await;
    assert_eq!(visible_mr_order(&app), vec![10, 8, 7]);
}

#[tokio::test]
async fn issues_paged_then_one_closed_keeps_older_pages() {
    let (_server, mut app, _rx) = setup_app().await;
    project_ctx(&mut app, MenuSelection::Issues);
    let rows = |hi, lo| range(hi, lo).map(|i| issue_n(1, i)).collect::<Vec<_>>();
    app.process_update_single(DataUpdate::ProjectIssues {
        project_id: 1,
        data: rows(100, 51),
        page: 1,
        has_more: true,
    })
    .await;
    app.process_update_single(DataUpdate::AppendProjectIssues {
        project_id: 1,
        data: rows(50, 1),
    })
    .await;
    let mut fresh: Vec<Issue> = range(100, 50)
        .filter(|i| *i != 90)
        .map(|i| issue_n(1, i))
        .collect();
    assert_eq!(fresh.len(), 50);
    app.process_update_single(DataUpdate::ProjectIssues {
        project_id: 1,
        data: std::mem::take(&mut fresh),
        page: 1,
        has_more: true,
    })
    .await;
    let ids = issue_ids(&app);
    assert_eq!(ids.len(), 99);
    assert!(!ids.contains(&90));
}

#[tokio::test]
async fn global_refresh_keeps_busy_project_page_two_despite_quiet_project() {
    let (_server, mut app, _rx) = setup_app().await;
    dashboard(&mut app, MenuSelection::MergeRequests);
    let first = || {
        let mut v: Vec<MergeRequest> = range(100, 51).map(|i| mr_n(1, i)).collect();
        v.push(mr_n(2, 5));
        v
    };
    app.process_update_single(DataUpdate::GlobalMRs {
        data: first(),
        page: 1,
        has_more: true,
    })
    .await;
    app.process_update_single(DataUpdate::AppendGlobalMRs {
        data: range(50, 40).map(|i| mr_n(1, i)).collect(),
        has_more: true,
        page: 2,
        complete: true,
    })
    .await;
    app.process_update_single(DataUpdate::GlobalMRs {
        data: first(),
        page: 1,
        has_more: true,
    })
    .await;
    assert_eq!(mr_ids(&app).len(), 62);
}

#[tokio::test]
async fn todos_paged_refresh_keeps_older_pages() {
    let (_server, mut app, _rx) = setup_app().await;
    let page = |hi, lo| range(hi, lo).map(todo).collect::<Vec<_>>();
    app.process_update_single(DataUpdate::Todos {
        data: page(100, 51),
        page: 1,
        has_more: true,
    })
    .await;
    app.process_update_single(DataUpdate::AppendTodos(page(50, 1)))
        .await;
    app.process_update_single(DataUpdate::Todos {
        data: page(100, 51),
        page: 1,
        has_more: true,
    })
    .await;
    assert_eq!(app.todos.len(), 100);
}

#[tokio::test]
async fn project_pipelines_paged_refresh_keeps_older_pages() {
    let (_server, mut app, _rx) = setup_app().await;
    project_ctx(&mut app, MenuSelection::Pipelines);
    let page = |hi, lo| range(hi, lo).map(|i| pipeline(1, i)).collect::<Vec<_>>();
    app.process_update_single(DataUpdate::ProjectPipelines {
        project_id: 1,
        data: page(100, 51),
        page: 1,
        has_more: true,
    })
    .await;
    app.process_update_single(DataUpdate::AppendProjectPipelines {
        project_id: 1,
        data: page(50, 1),
    })
    .await;
    app.process_update_single(DataUpdate::ProjectPipelines {
        project_id: 1,
        data: page(100, 51),
        page: 1,
        has_more: true,
    })
    .await;
    assert_eq!(app.pipelines.len(), 100);
}
