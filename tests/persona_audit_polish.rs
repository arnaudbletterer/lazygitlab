use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use gitlab::GitlabBuilder;
use lazygitlab::models::{
    App, Board, BoardList, DataUpdate, DiffDisplayLine, Issue, MRDiff, MergeRequest, Panel,
    Project, ViewMode,
};
use lazygitlab::ui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_app() -> (MockServer, App) {
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
    (server, App::new(client, tx, None))
}

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

fn sample_mr(iid: u64, squash: bool) -> MergeRequest {
    serde_json::from_value(serde_json::json!({
        "id": 100 + iid,
        "iid": iid,
        "project_id": 1,
        "title": format!("MR !{}", iid),
        "description": "Test MR",
        "state": "opened",
        "source_branch": "feature",
        "target_branch": "main",
        "author": { "id": 1, "username": "reviewer", "name": "Reviewer" },
        "assignees": [],
        "reviewers": [],
        "labels": [],
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
        "web_url": "https://gitlab.com/test",
        "draft": false,
        "squash": squash,
        "has_conflicts": false,
        "detailed_merge_status": "mergeable",
    }))
    .unwrap()
}

fn sample_issue(iid: u64, state: &str) -> Issue {
    serde_json::from_value(serde_json::json!({
        "id": 200 + iid,
        "iid": iid,
        "project_id": 1,
        "title": format!("Issue #{}", iid),
        "description": "Issue description",
        "state": state,
        "author": { "id": 2, "username": "triager", "name": "Triager" },
        "assignees": [],
        "labels": [],
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
        "web_url": "https://gitlab.com/issue",
        "confidential": false,
        "upvotes": 3,
        "downvotes": 0,
        "user_notes_count": 1,
    }))
    .unwrap()
}

fn sample_project(id: u64) -> Project {
    Project {
        id,
        name: "test-proj".into(),
        path_with_namespace: Some("group/test-proj".into()),
        web_url: Some("https://gitlab.com/group/test-proj".into()),
        default_branch: None,
        namespace: None,
    }
}

// -------------------------------------------------------------------------
// PERSONA 1: Code Reviewer Tests
// -------------------------------------------------------------------------

#[tokio::test]
async fn test_merge_dialog_interactive_squash_and_delete_branch_toggles() {
    let (_server, mut app) = setup_app().await;
    app.active_panel = Panel::Main;
    app.active_selection = lazygitlab::models::MenuSelection::MergeRequests;
    app.dashboard_menu_state.select(Some(1));
    let mr = sample_mr(42, false);
    app.merge_requests = vec![mr.clone()];
    app.mrs_state.select(Some(1));
    app.view_mode = ViewMode::Detail;
    app.selected_mr = Some(mr);

    // Open merge confirmation popup
    app.open_confirm_merge_popup();
    assert!(app.show_confirm_merge_popup);
    assert!(!app.merge_squash);
    assert!(app.merge_remove_source_branch);

    // Toggle squash using 's'
    app.handle_key(key_event(KeyCode::Char('s'))).await;
    assert!(app.merge_squash);

    // Toggle delete branch using 'd'
    app.handle_key(key_event(KeyCode::Char('d'))).await;
    assert!(!app.merge_remove_source_branch);

    // Render UI and verify visual representation
    let backend = TestBackend::new(100, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui(f, &mut app)).unwrap();

    let mut content = String::new();
    let buffer = terminal.backend().buffer();
    for y in 0..buffer.area().height {
        for x in 0..buffer.area().width {
            content.push(buffer[(x, y)].symbol().chars().next().unwrap());
        }
        content.push('\n');
    }

    assert!(content.contains("[s] Squash commits: [X] Yes"));
    assert!(content.contains("[d] Delete source branch: [ ] No"));
    assert!(content.contains("[s] Toggle Squash"));
    assert!(content.contains("[d] Toggle Delete Branch"));
}

#[tokio::test]
async fn test_diff_view_side_by_side_toggle_and_rendering() {
    let (_server, mut app) = setup_app().await;
    app.selected_mr = Some(sample_mr(10, true));
    app.view_mode = ViewMode::Diff;

    app.mr_diffs = vec![MRDiff {
        old_path: "src/old_file.rs".into(),
        new_path: "src/new_file.rs".into(),
        diff: "@@ -1 +1 @@\n-old code\n+new code\n".into(),
        collapsed: false,
        display_lines: vec![
            DiffDisplayLine::Code("@@ -1,1 +1,1 @@".into(), None, None),
            DiffDisplayLine::Code("-old code".into(), Some(1), None),
            DiffDisplayLine::Code("+new code".into(), None, Some(1)),
        ],
    }];
    app.mr_diff_state.select(Some(0));

    // Initially in unified diff mode
    assert!(!app.diff_side_by_side);

    // Press 'w' to toggle to side-by-side
    app.handle_key(key_event(KeyCode::Char('w'))).await;
    assert!(app.diff_side_by_side);
    assert_eq!(
        app.status_message.as_deref(),
        Some("Diff view: Side-by-Side")
    );

    // Render side-by-side diff
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui(f, &mut app)).unwrap();

    let mut content = String::new();
    let buffer = terminal.backend().buffer();
    for y in 0..buffer.area().height {
        for x in 0..buffer.area().width {
            content.push(buffer[(x, y)].symbol().chars().next().unwrap());
        }
        content.push('\n');
    }

    // Side-by-Side renders Base and Head columns
    assert!(content.contains("Base: src/old_file.rs"));
    assert!(content.contains("Head: src/new_file.rs"));
    assert!(content.contains("[w]"));
    assert!(content.contains("Unified"));

    // Press 'w' again to toggle back to unified
    app.handle_key(key_event(KeyCode::Char('w'))).await;
    assert!(!app.diff_side_by_side);
    assert_eq!(app.status_message.as_deref(), Some("Diff view: Unified"));
}

#[tokio::test]
async fn test_empty_and_binary_diff_handling() {
    let (_server, mut app) = setup_app().await;

    app.mr_diffs = vec![
        MRDiff {
            old_path: "empty.txt".into(),
            new_path: "empty.txt".into(),
            diff: String::new(),
            collapsed: false,
            display_lines: Vec::new(),
        },
        MRDiff {
            old_path: "image.png".into(),
            new_path: "image.png".into(),
            diff: "Binary files a/image.png and b/image.png differ".into(),
            collapsed: false,
            display_lines: Vec::new(),
        },
    ];

    app.enrich_diffs_with_discussions();

    // The empty diff without collapsed API flag must NOT be collapsed
    assert!(!app.mr_diffs[0].collapsed);
    assert_eq!(app.mr_diffs[0].display_lines.len(), 1);
    if let DiffDisplayLine::Code(line, _, _) = &app.mr_diffs[0].display_lines[0] {
        assert!(line.contains("Empty diff or binary file: no changes to display"));
    } else {
        panic!("Expected code line");
    }

    // The binary diff must show the binary notice
    assert_eq!(app.mr_diffs[1].display_lines.len(), 1);
    if let DiffDisplayLine::Code(line, _, _) = &app.mr_diffs[1].display_lines[0] {
        assert!(line.contains("Binary file: image.png"));
    } else {
        panic!("Expected binary line");
    }
}

// -------------------------------------------------------------------------
// PERSONA 2: Issue Triager / Kanban Master Tests
// -------------------------------------------------------------------------

#[tokio::test]
async fn test_kanban_board_lists_cursor_preservation_on_update() {
    let (_server, mut app) = setup_app().await;
    app.selected_project = Some(sample_project(1));
    app.selected_board = Some(Board {
        id: 99,
        name: "Scrum Board".into(),
        lists: None,
    });

    let lists = vec![
        BoardList {
            id: 1,
            label: None,
            position: Some(0),
            list_type: "backlog".into(),
        },
        BoardList {
            id: 2,
            label: None,
            position: Some(1),
            list_type: "label".into(),
        },
        BoardList {
            id: 3,
            label: None,
            position: Some(2),
            list_type: "closed".into(),
        },
    ];

    // Select column 1
    app.board_lists = lists.clone();
    app.board_lists_state.select(Some(1));

    // Incoming DataUpdate::BoardLists must preserve column 1 selection
    app.process_update_single(DataUpdate::BoardLists {
        project_id: 1,
        board_id: 99,
        data: lists,
    })
    .await;

    assert_eq!(app.board_lists_state.selected(), Some(1));
}

#[tokio::test]
async fn test_kanban_board_issues_state_synchronization() {
    let (_server, mut app) = setup_app().await;
    app.selected_project = Some(sample_project(1));
    app.selected_board = Some(Board {
        id: 99,
        name: "b".into(),
        lists: None,
    });
    app.board_lists = vec![BoardList {
        id: 10,
        label: None,
        position: Some(0),
        list_type: "todo".into(),
    }];

    let issues = vec![sample_issue(1, "opened"), sample_issue(2, "opened")];

    // Receiving BoardIssues must immediately filter lists so filtered_board_issues is synced
    app.process_update_single(DataUpdate::BoardIssues {
        project_id: 1,
        board_id: 99,
        list_id: 10,
        data: issues,
    })
    .await;

    assert_eq!(app.board_issues.get(&10).map(|v| v.len()), Some(2));
    assert_eq!(app.filtered_board_issues.get(&10).map(|v| v.len()), Some(2));
}

#[tokio::test]
async fn test_kanban_move_board_issue_optimistic_update() {
    let (_server, mut app) = setup_app().await;
    app.view_mode = ViewMode::Board;
    app.selected_project = Some(sample_project(1));
    app.selected_board = Some(Board {
        id: 99,
        name: "Scrum Board".into(),
        lists: None,
    });

    let list_open = BoardList {
        id: 101,
        label: None,
        position: Some(0),
        list_type: "backlog".into(),
    };
    let list_closed = BoardList {
        id: 102,
        label: None,
        position: Some(1),
        list_type: "closed".into(),
    };
    app.board_lists = vec![list_open.clone(), list_closed.clone()];
    app.board_lists_state.select(Some(0));

    let issue = sample_issue(55, "opened");
    app.board_issues.insert(101, vec![issue.clone()]);
    app.board_issues.insert(102, Vec::new());
    app.filter_lists();
    app.board_issue_states
        .entry(101)
        .or_default()
        .select(Some(0));

    // Move right (direction +1) to closed list
    let _ = app.move_board_issue(1).await;

    // Verify optimistic move
    assert_eq!(app.board_issues.get(&101).map(|v| v.len()), Some(0));
    assert_eq!(app.board_issues.get(&102).map(|v| v.len()), Some(1));
    assert_eq!(
        app.filtered_board_issues.get(&102).map(|v| v.len()),
        Some(1)
    );
    assert_eq!(app.board_issues.get(&102).unwrap()[0].state, "closed");
    assert_eq!(app.board_lists_state.selected(), Some(1));
}

#[tokio::test]
async fn test_kanban_card_c_key_sets_selected_issue() {
    let (_server, mut app) = setup_app().await;
    app.view_mode = ViewMode::Board;
    app.active_panel = Panel::Main;

    let list = BoardList {
        id: 201,
        label: None,
        position: Some(0),
        list_type: "opened".into(),
    };
    app.board_lists = vec![list.clone()];
    app.board_lists_state.select(Some(0));

    let issue = sample_issue(77, "opened");
    app.board_issues.insert(201, vec![issue]);
    app.filter_lists();
    app.board_issue_states
        .entry(201)
        .or_default()
        .select(Some(0));

    // Press 'c' on card
    app.handle_key(key_event(KeyCode::Char('c'))).await;

    // Verifies selected_issue was loaded with card 77 for toggling
    assert!(app.selected_issue.is_some());
    assert_eq!(app.selected_issue.as_ref().unwrap().iid, 77);
}
