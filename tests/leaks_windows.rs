use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use serde_json::{Value, json};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Rx = UnboundedReceiver<DataUpdate>;

async fn setup() -> (MockServer, App, Rx) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":1,"username":"u"})))
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

fn project(id: u64) -> Project {
    serde_json::from_value(json!({
        "id": id, "name": format!("p{id}"), "path_with_namespace": format!("g/p{id}"),
        "namespace": {"id": 77, "name": "g", "path": "g", "kind": "group", "full_path": "g"}
    }))
    .unwrap()
}

fn with_project(app: &mut App) {
    app.selected_project = Some(project(1));
    app.active_context = Context::Project;
}

/// Processes updates until `done` holds. Every update is awaited from the
/// channel, so no sleeping is involved.
async fn drain_until(app: &mut App, rx: &mut Rx, done: impl Fn(&App) -> bool) {
    loop {
        let update = tokio::time::timeout(Duration::from_secs(10), rx.recv())
            .await
            .expect("timed out waiting for an update")
            .expect("channel closed");
        app.process_update_single(update).await;
        if done(app) {
            break;
        }
    }
}

async fn settle(app: &mut App, rx: &mut Rx) {
    drain_until(app, rx, |a| {
        a.loading_tasks == 0 && a.page_in_flight.is_empty()
    })
    .await;
    while let Ok(update) = rx.try_recv() {
        app.process_update_single(update).await;
    }
}

fn issue(iid: u64) -> Value {
    json!({"iid": iid, "project_id": 1, "title": format!("i{iid}"),
        "author": {"id": 1, "username": "u"}, "state": "opened",
        "updated_at": format!("2026-01-01T00:00:{:02}Z", 59 - (iid % 50))})
}

fn commit(n: u64) -> Value {
    json!({"id": format!("c{n}"), "short_id": format!("c{n}"), "title": "t",
        "author_name": "a", "created_at": "2026-01-01T00:00:00Z", "message": "m"})
}

fn group(id: u64) -> Value {
    json!({"id": id, "name": format!("g{id:03}"), "path": "g", "full_path": "g"})
}

fn todo(id: u64) -> Value {
    json!({"id": id, "action_name": "mentioned", "target_type": "Issue",
        "target_url": "http://x", "body": "b", "state": "pending",
        "created_at": "2026-01-01T00:00:00Z",
        "project": {"id": 1, "name": "p", "path_with_namespace": "g/p"}})
}

async fn mount_page(server: &MockServer, p: &str, page: u32, body: Vec<Value>) {
    Mock::given(method("GET"))
        .and(path(p))
        .and(query_param("page", page.to_string()))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(server)
        .await;
}

#[tokio::test]
async fn next_page_releases_the_loading_counter_for_every_list() {
    let (server, mut app, mut rx) = setup().await;
    with_project(&mut app);
    mount_page(
        &server,
        "/api/v4/projects/1/issues",
        2,
        vec![issue(1), issue(2)],
    )
    .await;
    mount_page(&server, "/api/v4/projects/1/issues", 3, vec![issue(3)]).await;
    mount_page(
        &server,
        "/api/v4/projects",
        2,
        vec![json!({"id": 5, "name": "p5"})],
    )
    .await;
    mount_page(
        &server,
        "/api/v4/projects",
        3,
        vec![json!({"id": 6, "name": "p6"})],
    )
    .await;
    mount_page(&server, "/api/v4/groups", 2, vec![group(1)]).await;
    mount_page(&server, "/api/v4/groups", 3, vec![group(2)]).await;
    mount_page(&server, "/api/v4/todos", 2, vec![todo(1)]).await;
    mount_page(&server, "/api/v4/todos", 3, vec![todo(2)]).await;
    mount_page(
        &server,
        "/api/v4/projects/1/repository/commits",
        2,
        vec![commit(1)],
    )
    .await;
    mount_page(
        &server,
        "/api/v4/projects/1/repository/commits",
        3,
        vec![commit(2)],
    )
    .await;

    app.issues_page = 1;
    app.projects_page = 1;
    app.groups_page = 1;
    app.todos_page = 1;
    app.commits_page = 1;
    for round in 1..=2usize {
        app.issues_has_more = true;
        app.projects_has_more = true;
        app.groups_has_more = true;
        app.todos_has_more = true;
        app.commits_has_more = true;

        app.fetch_next_issues().await.unwrap();
        settle(&mut app, &mut rx).await;
        assert_eq!(app.loading_tasks, 0, "issues round {round}");
        assert_eq!(app.issues.len(), if round == 1 { 2 } else { 3 });

        app.fetch_next_projects().await.unwrap();
        settle(&mut app, &mut rx).await;
        assert_eq!(app.loading_tasks, 0, "projects round {round}");
        assert_eq!(app.projects.len(), round);

        app.fetch_next_groups().await.unwrap();
        settle(&mut app, &mut rx).await;
        assert_eq!(app.loading_tasks, 0, "groups round {round}");
        assert_eq!(app.groups.len(), round);

        app.fetch_next_todos().await.unwrap();
        settle(&mut app, &mut rx).await;
        assert_eq!(app.loading_tasks, 0, "todos round {round}");
        assert_eq!(app.todos.len(), round);

        app.fetch_next_commits().await.unwrap();
        settle(&mut app, &mut rx).await;
        assert_eq!(app.loading_tasks, 0, "commits round {round}");
        assert_eq!(app.commits.len(), round);
    }
    assert!(!app.is_loading);
}

#[tokio::test]
async fn next_page_is_gated_per_list_not_by_the_global_spinner() {
    let (server, mut app, mut rx) = setup().await;
    with_project(&mut app);
    mount_page(&server, "/api/v4/groups", 2, vec![group(1)]).await;
    app.groups_has_more = true;
    app.loading_tasks = 2;
    app.is_loading = true;
    app.fetch_next_groups().await.unwrap();
    app.fetch_next_groups().await.unwrap();
    drain_until(&mut app, &mut rx, |a| a.page_in_flight.is_empty()).await;
    assert_eq!(
        app.groups.len(),
        1,
        "one request in flight at a time, not zero"
    );
    let pages = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path() == "/api/v4/groups")
        .count();
    assert_eq!(pages, 1);
}

fn projects(range: std::ops::RangeInclusive<u64>) -> Vec<Project> {
    range.map(project).collect()
}

#[tokio::test]
async fn refresh_keeps_projects_from_later_pages() {
    let (_server, mut app, _rx) = setup().await;
    app.process_update_single(DataUpdate::Projects {
        data: projects(1..=100),
        page: 1,
        has_more: true,
    })
    .await;
    app.process_update_single(DataUpdate::AppendProjects {
        page: 2,
        data: projects(101..=110),
    })
    .await;
    assert_eq!(app.projects.len(), 110);
    assert_eq!(app.projects_page, 2);

    app.process_update_single(DataUpdate::Projects {
        data: projects(1..=100),
        page: 1,
        has_more: true,
    })
    .await;
    assert_eq!(
        app.projects.len(),
        110,
        "page 1 refresh must not unload page 2"
    );
    assert_eq!(app.projects_page, 2);

    let mut shifted = projects(1..=99);
    shifted.insert(0, project(500));
    app.process_update_single(DataUpdate::Projects {
        data: shifted,
        page: 1,
        has_more: true,
    })
    .await;
    let ids: Vec<u64> = app.projects.iter().map(|p| p.id).collect();
    assert_eq!(ids[0], 500);
    assert!(ids.contains(&100), "row pushed out of page 1 stays loaded");
    assert!(ids.contains(&110));
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), ids.len(), "no duplicates");
}

#[tokio::test]
async fn refresh_keeps_groups_from_later_pages() {
    let (_server, mut app, _rx) = setup().await;
    let groups = |r: std::ops::RangeInclusive<u64>| -> Vec<Group> {
        r.map(|i| serde_json::from_value(group(i)).unwrap())
            .collect()
    };
    app.process_update_single(DataUpdate::Groups {
        data: groups(1..=50),
        page: 1,
        has_more: true,
    })
    .await;
    app.process_update_single(DataUpdate::AppendGroups {
        page: 2,
        data: groups(51..=60),
    })
    .await;
    assert_eq!(app.groups.len(), 60);
    app.process_update_single(DataUpdate::Groups {
        data: groups(1..=50),
        page: 1,
        has_more: true,
    })
    .await;
    assert_eq!(app.groups.len(), 60);
    assert_eq!(app.groups_page, 2);
}

#[tokio::test]
async fn refresh_keeps_commits_from_later_pages_and_next_page_has_no_duplicates() {
    let (server, mut app, mut rx) = setup().await;
    with_project(&mut app);
    let page1: Vec<Commit> = (1..=50)
        .map(|n| serde_json::from_value(commit(n)).unwrap())
        .collect();
    mount_page(
        &server,
        "/api/v4/projects/1/repository/commits",
        2,
        (51..=60).map(commit).collect(),
    )
    .await;
    app.process_update_single(DataUpdate::Commits {
        project_id: 1,
        data: page1.clone(),
        page: 1,
        has_more: true,
    })
    .await;
    app.fetch_next_commits().await.unwrap();
    settle(&mut app, &mut rx).await;
    assert_eq!(app.commits.len(), 60);
    let ids: std::collections::HashSet<_> = app.commits.iter().map(|c| c.id.clone()).collect();
    assert_eq!(ids.len(), 60, "page 2 request must carry page=2");

    app.process_update_single(DataUpdate::Commits {
        project_id: 1,
        data: page1,
        page: 1,
        has_more: true,
    })
    .await;
    assert_eq!(app.commits.len(), 60, "refresh keeps appended commits");
    assert_eq!(app.commits_page, 2);
}

fn board(id: u64) -> Board {
    serde_json::from_value(json!({"id": id, "name": format!("b{id}")})).unwrap()
}

#[tokio::test]
async fn board_refetch_keeps_selected_board_and_cursor() {
    let (_server, mut app, _rx) = setup().await;
    with_project(&mut app);
    app.process_update_single(DataUpdate::Boards {
        project_id: 1,
        data: vec![board(1), board(2), board(3)],
    })
    .await;
    app.project_boards_state.select(Some(1));
    app.selected_board = Some(board(2));

    app.process_update_single(DataUpdate::Boards {
        project_id: 1,
        data: vec![board(1), board(2), board(3)],
    })
    .await;
    assert_eq!(app.project_boards_state.selected(), Some(1));

    app.process_update_single(DataUpdate::Boards {
        project_id: 1,
        data: vec![board(1), board(3), board(2)],
    })
    .await;
    assert_eq!(
        app.project_boards_state.selected(),
        Some(2),
        "cursor follows the board id"
    );
    app.update_preview().await;
    assert_eq!(app.selected_board.as_ref().map(|b| b.id), Some(2));
}

#[tokio::test]
async fn epic_refetch_keeps_selection_and_checks_group() {
    let (_server, mut app, _rx) = setup().await;
    with_project(&mut app);
    let epic = |id: u64| -> Epic {
        serde_json::from_value(json!({"id": id, "iid": id, "group_id": 77, "title": "e",
            "state": "opened", "web_url": "u"}))
        .unwrap()
    };
    app.process_update_single(DataUpdate::Epics {
        group_id: 77,
        data: vec![epic(1), epic(2)],
    })
    .await;
    app.epics_state.select(Some(1));
    app.process_update_single(DataUpdate::Epics {
        group_id: 77,
        data: vec![epic(2), epic(1)],
    })
    .await;
    assert_eq!(app.epics_state.selected(), Some(0));
    app.process_update_single(DataUpdate::Epics {
        group_id: 999,
        data: vec![],
    })
    .await;
    assert_eq!(app.epics.len(), 2, "epics of another group are ignored");
}

#[tokio::test]
async fn board_issues_of_another_board_are_dropped() {
    let (_server, mut app, _rx) = setup().await;
    with_project(&mut app);
    app.selected_board = Some(board(1));
    app.process_update_single(DataUpdate::BoardIssues {
        project_id: 1,
        board_id: 2,
        list_id: 9,
        data: vec![serde_json::from_value(issue(1)).unwrap()],
    })
    .await;
    assert!(app.board_issues.is_empty());
}

#[tokio::test]
async fn failing_board_lists_keeps_previous_lists_and_reports() {
    let (server, mut app, mut rx) = setup().await;
    with_project(&mut app);
    app.selected_board = Some(board(1));
    let list: BoardList = serde_json::from_value(json!({"id": 5, "list_type": "label"})).unwrap();
    app.board_lists = vec![list.clone()];
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/boards/1/lists"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    app.fetch_board_lists(1).await.unwrap();
    settle(&mut app, &mut rx).await;
    assert_eq!(app.board_lists, vec![list]);
    assert!(app.last_error.is_some(), "an error status is shown");
}

#[tokio::test]
async fn failing_board_list_issues_keeps_previous_issues_and_reports() {
    let (server, mut app, mut rx) = setup().await;
    with_project(&mut app);
    app.selected_board = Some(board(1));
    let kept: Issue = serde_json::from_value(issue(1)).unwrap();
    app.board_issues.insert(5, vec![kept.clone()]);
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/boards/1/lists/5/issues"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    app.fetch_board_list_issues(1, 5).await.unwrap();
    settle(&mut app, &mut rx).await;
    assert_eq!(app.board_issues.get(&5), Some(&vec![kept]));
    assert!(app.last_error.is_some());
}

fn label(id: u64) -> Label {
    serde_json::from_value(json!({"id": id, "name": format!("l{id}"), "color": "#ff0000"})).unwrap()
}

async fn wait_metadata_fetch(app: &mut App, rx: &mut Rx) {
    drain_until(app, rx, |a| a.last_error.is_some()).await;
    while !app
        .project_metadata_fetches_in_progress
        .lock()
        .unwrap()
        .is_empty()
    {
        tokio::task::yield_now().await;
    }
    while let Ok(update) = rx.try_recv() {
        app.process_update_single(update).await;
    }
}

async fn mount_metadata(server: &MockServer, labels_ok: bool) {
    let labels = Mock::given(method("GET")).and(path("/api/v4/projects/1/labels"));
    if labels_ok {
        labels
            .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<Value>::new()))
            .mount(server)
            .await;
    } else {
        labels
            .respond_with(ResponseTemplate::new(500))
            .mount(server)
            .await;
    }
    for p in ["milestones", "members/all"] {
        Mock::given(method("GET"))
            .and(path(format!("/api/v4/projects/1/{p}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<Value>::new()))
            .mount(server)
            .await;
    }
}

#[tokio::test]
async fn failing_labels_call_keeps_the_cached_labels() {
    let (server, mut app, mut rx) = setup().await;
    mount_metadata(&server, false).await;
    app.project_label_cache.insert(1, vec![label(1)]);
    app.project_milestone_cache.insert(1, vec![]);
    app.project_member_cache.insert(1, vec![]);
    app.trigger_project_metadata_fetch_force(1);
    wait_metadata_fetch(&mut app, &mut rx).await;
    assert_eq!(app.project_label_cache.get(&1), Some(&vec![label(1)]));
    assert!(app.last_error.is_some());
}

#[tokio::test]
async fn failed_metadata_call_is_not_marked_as_loaded() {
    let (server, mut app, mut rx) = setup().await;
    mount_metadata(&server, false).await;
    app.trigger_project_metadata_fetch_force(1);
    wait_metadata_fetch(&mut app, &mut rx).await;
    assert!(!app.project_label_cache.contains_key(&1));
    assert!(!app.project_milestone_cache.contains_key(&1));
}

#[tokio::test]
async fn failing_participants_call_keeps_previous_participants() {
    let (server, mut app, mut rx) = setup().await;
    let mr = json!({"iid": 5, "project_id": 1, "title": "t", "author": {"id": 1, "username": "u"}});
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/merge_requests/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mr.clone()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/merge_requests/5/participants"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/merge_requests/5/award_emoji"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let mut selected: MergeRequest = serde_json::from_value(mr).unwrap();
    selected.participants = vec![Author::new(9, "kept".into())];
    selected.reactions = vec![AwardEmoji {
        name: "thumbsup".into(),
    }];
    app.selected_mr = Some(selected);
    app.fetch_discussions(1, 5).await.unwrap();
    drain_until(&mut app, &mut rx, |a| {
        a.loading_tasks == 0 && a.last_error.is_some()
    })
    .await;
    let mr = app.selected_mr.as_ref().unwrap();
    assert_eq!(mr.participants.len(), 1);
    assert_eq!(mr.participants[0].username, "kept");
    assert_eq!(mr.reactions.len(), 1);
}

#[tokio::test]
async fn late_todos_result_for_another_filter_is_dropped() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/todos"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![todo(1)]))
        .mount(&server)
        .await;
    app.todo_filter = TodoFilter::Pending;
    app.fetch_todos().await.unwrap();
    app.todo_filter = TodoFilter::Done;
    settle(&mut app, &mut rx).await;
    assert!(app.todos.is_empty(), "pending result must not land in Done");
}

#[tokio::test]
async fn late_todos_page_for_another_filter_is_dropped() {
    let (server, mut app, mut rx) = setup().await;
    mount_page(&server, "/api/v4/todos", 2, vec![todo(1)]).await;
    app.todos_has_more = true;
    app.todo_filter = TodoFilter::Pending;
    app.fetch_next_todos().await.unwrap();
    app.todo_filter = TodoFilter::Done;
    settle(&mut app, &mut rx).await;
    assert!(app.todos.is_empty());
}

#[tokio::test]
async fn cleared_todos_poll_does_not_reuse_the_etag() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/todos"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(vec![todo(1)])
                .append_header("ETag", "\"w-1\""),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v4/todos"))
        .and(wiremock::matchers::header("if-none-match", "\"w-1\""))
        .respond_with(ResponseTemplate::new(304))
        .with_priority(1)
        .mount(&server)
        .await;
    app.todos_etag = Some("\"w-1\"".into());
    app.todos_etag_state = Some("pending");
    assert!(app.todos.is_empty());
    app.spawn_todos_poll();
    drain_until(&mut app, &mut rx, |a| !a.todos_poll_in_flight).await;
    assert_eq!(app.todos.len(), 1, "a 304 must not leave the list empty");
}

#[tokio::test]
async fn next_projects_page_skips_rows_that_moved_up() {
    let (_server, mut app, _rx) = setup().await;
    app.process_update_single(DataUpdate::Projects {
        data: projects(1..=100),
        page: 1,
        has_more: true,
    })
    .await;
    app.process_update_single(DataUpdate::AppendProjects {
        page: 2,
        data: projects(101..=200),
    })
    .await;
    app.process_update_single(DataUpdate::AppendProjects {
        page: 3,
        data: projects(201..=210)
            .into_iter()
            .chain(projects(150..=160))
            .collect(),
    })
    .await;
    let mut ids: Vec<u64> = app.projects.iter().map(|p| p.id).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(
        ids.len(),
        total,
        "a row that moved up is not appended twice"
    );
}

#[tokio::test]
async fn next_groups_page_skips_rows_that_moved_up() {
    let (_server, mut app, _rx) = setup().await;
    let groups = |r: std::ops::RangeInclusive<u64>| -> Vec<Group> {
        r.map(|i| serde_json::from_value(group(i)).unwrap())
            .collect()
    };
    app.process_update_single(DataUpdate::Groups {
        data: groups(1..=50),
        page: 1,
        has_more: true,
    })
    .await;
    app.process_update_single(DataUpdate::AppendGroups {
        page: 2,
        data: groups(40..=60),
    })
    .await;
    assert_eq!(app.groups.len(), 60);
}

#[tokio::test]
async fn stale_next_page_after_a_page_one_reset_is_dropped() {
    let (_server, mut app, _rx) = setup().await;
    app.process_update_single(DataUpdate::Projects {
        data: projects(1..=100),
        page: 1,
        has_more: true,
    })
    .await;
    app.process_update_single(DataUpdate::AppendProjects {
        page: 3,
        data: projects(201..=210),
    })
    .await;
    assert_eq!(app.projects.len(), 100);
    assert_eq!(app.projects_page, 1);
}

#[tokio::test]
async fn epic_issue_refetch_keeps_selection_by_id() {
    let (_server, mut app, _rx) = setup().await;
    with_project(&mut app);
    let epic: Epic = serde_json::from_value(json!({"id": 1, "iid": 4, "group_id": 77,
        "title": "e", "state": "opened", "web_url": "u"}))
    .unwrap();
    app.selected_epic = Some(epic);
    let issues = |iids: &[u64]| -> Vec<Issue> {
        iids.iter()
            .map(|i| serde_json::from_value(issue(*i)).unwrap())
            .collect()
    };
    app.process_update_single(DataUpdate::EpicIssues {
        group_id: 77,
        epic_iid: 4,
        data: issues(&[1, 2, 3]),
    })
    .await;
    app.epic_issues_state.select(Some(1));
    app.process_update_single(DataUpdate::EpicIssues {
        group_id: 77,
        epic_iid: 4,
        data: issues(&[3, 2, 1]),
    })
    .await;
    let sel = app.epic_issues_state.selected().unwrap();
    assert_eq!(app.filtered_epic_issues[sel].iid, 2);
}

#[tokio::test]
async fn todos_poll_keeps_etag_for_a_user_with_no_todos() {
    let (server, mut app, mut rx) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v4/todos"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(Vec::<Value>::new())
                .append_header("ETag", "\"w-e\""),
        )
        .mount(&server)
        .await;
    for _ in 0..2 {
        app.spawn_todos_poll();
        drain_until(&mut app, &mut rx, |a| !a.todos_poll_in_flight).await;
    }
    let conditional = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.headers.get("if-none-match").is_some())
        .count();
    assert_eq!(conditional, 1);
}
