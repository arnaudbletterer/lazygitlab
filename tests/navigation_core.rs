use std::collections::VecDeque;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use gitlab::GitlabBuilder;
use lazygitlab::events::EventSource;
use lazygitlab::models::*;
use lazygitlab::ui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup() -> (MockServer, App) {
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
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.membership_sync_attempted = true;
    app.member_project_ids.insert(1);
    (server, app)
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

fn mr(iid: u64) -> MergeRequest {
    serde_json::from_value(serde_json::json!({
        "iid": iid, "project_id": 1, "title": format!("mr {iid}"),
        "author": {"id": 1, "username": "u"}, "web_url": "http://x", "state": "opened",
        "created_at": "2026-01-01T00:00:00Z", "reactions": [],
        "source_branch": "a", "target_branch": "b"
    }))
    .unwrap()
}

fn issue(iid: u64) -> Issue {
    serde_json::from_value(serde_json::json!({
        "iid": iid, "project_id": 1, "title": format!("issue {iid}"),
        "author": {"id": 1, "username": "u"}, "state": "opened"
    }))
    .unwrap()
}

fn todo(id: u64) -> Todo {
    serde_json::from_value(serde_json::json!({
        "id": id, "project": null, "action_name": "mentioned", "target_type": "Issue",
        "target_url": "http://x/issues/1", "body": format!("todo {id}"), "state": "pending",
        "created_at": "2026-01-01T00:00:00Z", "group": null
    }))
    .unwrap()
}

fn pipeline(id: u64, status: &str) -> Pipeline {
    serde_json::from_value(serde_json::json!({
        "id": id, "project_id": 1, "status": status, "source": "push", "ref": "main",
        "web_url": "http://x", "created_at": "2026-01-01T00:00:00Z"
    }))
    .unwrap()
}

fn project(id: u64, name: &str) -> Project {
    serde_json::from_value(serde_json::json!({
        "id": id, "name": name, "path_with_namespace": format!("g/{name}")
    }))
    .unwrap()
}

fn seed_lists(app: &mut App) {
    app.merge_requests = (1..=5).map(mr).collect();
    app.issues = (1..=4).map(issue).collect();
    app.todos = (1..=3).map(todo).collect();
    app.filter_lists();
}

fn screen_text(terminal: &Terminal<TestBackend>) -> String {
    let buffer = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..buffer.area().height {
        for x in 0..buffer.area().width {
            out.push_str(buffer[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

fn draw(app: &mut App) -> String {
    let mut terminal = Terminal::new(TestBackend::new(160, 40)).unwrap();
    terminal.draw(|f| ui(f, app)).unwrap();
    screen_text(&terminal)
}

#[tokio::test]
async fn sidebar_round_trip_keeps_rows() {
    let (_server, mut app) = setup().await;
    seed_lists(&mut app);
    app.active_panel = Panel::Dashboard;
    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::MergeRequests;
    app.dashboard_menu_state.select(Some(2));

    app.handle_key(key(KeyCode::Char('j'))).await;
    assert_eq!(app.active_selection, MenuSelection::Issues);
    assert_eq!(app.merge_requests.len(), 5, "MRs shrank on MR -> Issues");
    assert_eq!(app.issues.len(), 4);
    assert_eq!(app.todos.len(), 3);

    app.handle_key(key(KeyCode::Char('k'))).await;
    assert_eq!(app.active_selection, MenuSelection::MergeRequests);
    assert_eq!(app.merge_requests.len(), 5, "MRs shrank on Issues -> MR");
    assert_eq!(app.issues.len(), 4, "issues shrank on Issues -> MR");
    assert!(!app.filtered_mrs.is_empty());
}

#[tokio::test]
async fn g_jumps_keep_rows_and_reset_view_state() {
    let (_server, mut app) = setup().await;
    seed_lists(&mut app);
    app.active_panel = Panel::Main;
    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::MergeRequests;
    app.detail_section = DetailSection::Discussions;
    app.selected_mr = Some(mr(1));

    app.handle_key(key(KeyCode::Char('g'))).await;
    app.handle_key(key(KeyCode::Char('i'))).await;
    assert_eq!(app.active_selection, MenuSelection::Issues);
    assert_eq!(app.detail_section, DetailSection::Metadata);
    assert!(app.selected_mr.is_none());
    assert_eq!(app.merge_requests.len(), 5);
    assert_eq!(app.issues.len(), 4);
}

#[tokio::test]
async fn filter_cycling_clears_only_its_own_list() {
    let (_server, mut app) = setup().await;
    seed_lists(&mut app);
    app.active_panel = Panel::Main;
    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::MergeRequests;

    app.handle_key(key(KeyCode::Char(']'))).await;
    assert_eq!(app.todos.len(), 3, "MR filter wiped todos");
    assert_eq!(app.issues.len(), 4, "MR filter wiped issues");
    assert!(app.merge_requests.is_empty());

    app.active_selection = MenuSelection::Issues;
    app.merge_requests = (1..=5).map(mr).collect();
    app.handle_key(key(KeyCode::Char(']'))).await;
    assert_eq!(app.merge_requests.len(), 5, "issue filter wiped MRs");
    assert_eq!(app.todos.len(), 3);
    assert!(app.issues.is_empty());

    app.active_selection = MenuSelection::Todos;
    app.todos_etag = Some("\"etag\"".into());
    app.handle_key(key(KeyCode::Char(']'))).await;
    assert!(app.todos.is_empty());
    assert!(app.todos_etag.is_none(), "etag must reset with the list");
    assert_eq!(app.merge_requests.len(), 5);
}

#[tokio::test]
async fn detail_responses_never_create_a_selection() {
    let (_server, mut app) = setup().await;
    seed_lists(&mut app);
    app.active_panel = Panel::Dashboard;
    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::Issues;
    app.dashboard_menu_state.select(Some(3));

    app.process_update_single(DataUpdate::MRDetail(mr(1))).await;
    assert!(app.selected_mr.is_none());
    app.process_update_single(DataUpdate::IssueDetail(issue(1)))
        .await;
    assert!(app.selected_issue.is_none());
}

async fn all_puts(server: &MockServer) -> Vec<String> {
    tokio::time::sleep(Duration::from_millis(400)).await;
    server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.method.as_str() == "PUT")
        .map(|r| r.url.path().to_string())
        .collect()
}

async fn edit_title(app: &mut App) {
    app.input_mode = true;
    app.input_target = InputTarget::EditTitle;
    app.textarea = ratatui_textarea::TextArea::from(["renamed"]);
    app.enter().await;
    app.input_mode = false;
}

async fn jumps_then_edits(first: char, second: char) -> Vec<String> {
    let (server, mut app) = setup().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    seed_lists(&mut app);
    app.active_panel = Panel::Main;
    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::Home;

    app.handle_key(key(KeyCode::Char('g'))).await;
    app.handle_key(key(KeyCode::Char(first))).await;
    app.handle_key(key(KeyCode::Char('j'))).await;
    app.handle_key(key(KeyCode::Char('g'))).await;
    app.handle_key(key(KeyCode::Char(second))).await;
    // Right after the jump, before any row of the new screen is selected.
    app.process_update_single(DataUpdate::MRDetail(mr(1))).await;
    app.process_update_single(DataUpdate::IssueDetail(issue(1)))
        .await;
    edit_title(&mut app).await;
    app.handle_key(key(KeyCode::Char('j'))).await;
    edit_title(&mut app).await;
    all_puts(&server).await
}

#[tokio::test]
async fn edit_after_g_m_then_g_i_targets_the_issue_only() {
    let puts = jumps_then_edits('m', 'i').await;
    assert!(!puts.is_empty());
    assert!(
        puts.iter().all(|p| p == "/api/v4/projects/1/issues/1"),
        "{puts:?}"
    );
}

#[tokio::test]
async fn edit_after_g_i_then_g_m_targets_the_mr_only() {
    let puts = jumps_then_edits('i', 'm').await;
    assert!(!puts.is_empty());
    assert!(
        puts.iter()
            .all(|p| p == "/api/v4/projects/1/merge_requests/1"),
        "{puts:?}"
    );
}

#[tokio::test]
async fn dashboard_pipelines_are_navigable_and_target_the_filtered_row() {
    let (server, mut app) = setup().await;
    Mock::given(method("POST"))
        .and(path_regex(r"/pipelines/\d+/cancel$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 12, "project_id": 1, "status": "canceled", "source": "push", "ref": "main",
            "web_url": "http://x", "created_at": "2026-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;
    app.cross_pipelines = vec![
        pipeline(10, "success"),
        pipeline(11, "failed"),
        pipeline(12, "failed"),
    ];
    app.pipeline_filter = PipelineFilter::Failed;
    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::Pipelines;
    app.active_panel = Panel::Main;
    app.filter_lists();
    assert_eq!(app.filtered_pipelines.len(), 2);
    assert_eq!(app.pipelines_state.selected(), Some(0));

    app.handle_key(key(KeyCode::Char('j'))).await;
    assert_eq!(app.pipelines_state.selected(), Some(1));
    assert_eq!(app.selected_pipeline.as_ref().map(|p| p.id), Some(12));

    app.handle_key(key(KeyCode::Char('C'))).await;
    let posts: Vec<String> = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.method.as_str() == "POST")
        .map(|r| r.url.path().to_string())
        .collect();
    assert!(
        posts.iter().any(|p| p.ends_with("/pipelines/12/cancel")),
        "cancel went to {posts:?}"
    );
    assert!(!posts.iter().any(|p| p.ends_with("/pipelines/11/cancel")));
}

#[tokio::test]
async fn enter_on_filtered_projects_opens_the_right_project_and_never_panics() {
    let (_server, mut app) = setup().await;
    app.projects = vec![project(1, "alpha"), project(2, "bravo"), project(3, "zulu")];
    app.filter_input = "zul".into();
    app.active_context = Context::Explore;
    app.active_selection = MenuSelection::Projects;
    app.active_panel = Panel::Main;
    app.filter_lists();
    assert_eq!(app.filtered_projects.len(), 1);
    app.projects_state.select(Some(2));
    app.handle_key(key(KeyCode::Enter)).await;
    assert!(
        app.selected_project.is_none(),
        "stale index opened a project"
    );

    app.projects_state.select(Some(0));
    app.handle_key(key(KeyCode::Enter)).await;
    assert_eq!(app.selected_project.as_ref().map(|p| p.id), Some(3));
    assert_eq!(app.active_context, Context::Project);
}

#[tokio::test]
async fn empty_state_tells_loading_failed_and_empty_apart() {
    let (_server, mut app) = setup().await;
    app.active_panel = Panel::Main;
    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::MergeRequests;

    app.is_loading = true;
    let text = draw(&mut app);
    assert!(text.contains("Loading..."), "{text}");
    assert!(!text.contains("match this view"));
    assert!(text.contains("LOADING"));

    app.is_loading = false;
    app.list_error = Some("MRs fetch failed: boom".into());
    let text = draw(&mut app);
    assert!(
        text.contains("Couldn't load: MRs fetch failed: boom (r to retry)"),
        "{text}"
    );
    assert!(text.contains("STALE"));
    assert!(!text.contains("LIVE"));

    app.list_error = None;
    let text = draw(&mut app);
    assert!(text.contains("No merge requests match this view"), "{text}");
    assert!(text.contains("LIVE"));
}

#[tokio::test]
async fn detail_column_is_reserved_so_layout_does_not_flip() {
    let (_server, mut app) = setup().await;
    seed_lists(&mut app);
    app.active_panel = Panel::Main;
    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::MergeRequests;
    let _ = draw(&mut app);
    let without = app.get_rect("main_list").unwrap();
    app.selected_mr = Some(mr(1));
    let _ = draw(&mut app);
    let with = app.get_rect("main_list").unwrap();
    assert_eq!(without.width, with.width);
}

fn click(x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::empty(),
    }
}

#[tokio::test]
async fn mouse_hits_every_dashboard_row_and_the_right_tab() {
    let (_server, mut app) = setup().await;
    seed_lists(&mut app);
    app.active_panel = Panel::Main;
    app.active_context = Context::Dashboard;
    app.active_selection = MenuSelection::MergeRequests;
    let _ = draw(&mut app);

    let nav = app.get_rect("nav_dashboard").unwrap();
    app.handle_mouse(click(nav.x + 3, nav.y + 1 + 4)).await;
    assert_eq!(app.active_selection, MenuSelection::Pipelines);

    app.todos = (1..=3).map(todo).collect();
    app.merge_requests = (1..=5).map(mr).collect();
    let _ = draw(&mut app);
    let list = app.get_rect("main_list").unwrap();
    app.handle_mouse(click(list.x + 1 + 3, list.y + 1)).await;
    assert_eq!(app.todos.len(), 3, "Pipelines tab click wiped todos");
    assert_eq!(app.merge_requests.len(), 5, "Pipelines tab click wiped MRs");

    app.active_selection = MenuSelection::MergeRequests;
    let _ = draw(&mut app);
    let list = app.get_rect("main_list").unwrap();
    // Tab cells: Opened 0..8, Review 11..19, Assigned 22..32, Merged 35..43, Closed 46..54.
    app.handle_mouse(click(list.x + 1 + 50, list.y + 1)).await;
    assert_eq!(app.mr_filter, MergeRequestFilter::Closed);
}

struct Script {
    events: Mutex<VecDeque<Event>>,
    frames: Arc<AtomicUsize>,
    seen: Mutex<Vec<usize>>,
}

impl EventSource for Script {
    async fn next_event(&self, _timeout: Duration) -> io::Result<Option<Event>> {
        self.seen
            .lock()
            .unwrap()
            .push(self.frames.load(Ordering::SeqCst));
        let next = self.events.lock().unwrap().pop_front();
        if next.is_none() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        Ok(next)
    }
}

#[tokio::test]
async fn resize_event_redraws() {
    let (_server, mut app) = setup().await;
    let frames = Arc::new(AtomicUsize::new(0));
    let source = Script {
        events: Mutex::new(VecDeque::from([
            Event::Resize(100, 30),
            Event::Key(key(KeyCode::Char('q'))),
        ])),
        frames: frames.clone(),
        seen: Mutex::new(Vec::new()),
    };
    let counter = frames.clone();
    let mut capture: Option<lazygitlab::runner::FrameCaptureFn<TestBackend>> =
        Some(Box::new(move |_t: &Terminal<TestBackend>| {
            counter.fetch_add(1, Ordering::SeqCst);
        }));
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let (_tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    lazygitlab::runner::run_app(&mut terminal, &mut app, &source, &mut rx, &mut capture)
        .await
        .unwrap();
    let seen = source.seen.lock().unwrap().clone();
    assert!(seen.len() >= 2, "{seen:?}");
    assert!(
        seen[1] > seen[0],
        "no frame was drawn between the resize and the next event: {seen:?}"
    );
}
