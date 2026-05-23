use lazygitlab::models::{App, Context, MenuSelection, Panel};
use gitlab::GitlabBuilder;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::path;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};

async fn setup_app() -> (MockServer, App) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    (server, App::new(client, tx))
}

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

#[tokio::test]
async fn test_g_prefix_shortcuts() {
    let (_server, mut app) = setup_app().await;
    
    // Initial state
    assert_eq!(app.active_context, Context::Dashboard);
    assert_eq!(app.active_selection, MenuSelection::Todos);
    assert!(!app.pending_g_prefix);

    // Press 'g'
    app.handle_key(key_event(KeyCode::Char('g'))).await;
    assert!(app.pending_g_prefix);

    // Press 'i' -> Should go to Issues
    app.handle_key(key_event(KeyCode::Char('i'))).await;
    assert!(!app.pending_g_prefix);
    assert_eq!(app.active_selection, MenuSelection::Issues);
    assert_eq!(app.active_panel, Panel::Main);

    // Press 'g' then 't' -> Should go to Todos
    app.handle_key(key_event(KeyCode::Char('g'))).await;
    app.handle_key(key_event(KeyCode::Char('t'))).await;
    assert_eq!(app.active_selection, MenuSelection::Todos);
}

#[tokio::test]
async fn test_ux_interaction_refinements() {
    let (_server, mut app) = setup_app().await;

    // 1. Swap keyboard shortcuts for List Filtering ('/') and Global Search ('f')
    assert!(!app.input_mode);
    assert!(!app.show_filter_input);
    assert!(!app.show_search);

    app.active_panel = Panel::Main;

    // Press '/' -> Should open filter input
    app.handle_key(key_event(KeyCode::Char('/'))).await;
    assert!(app.input_mode);
    assert!(app.show_filter_input);
    assert_eq!(app.input_target, lazygitlab::models::InputTarget::Filter);

    // Type a character 'a'
    app.handle_key(key_event(KeyCode::Char('a'))).await;
    assert_eq!(app.filter_input, "a");

    // Type '/' character inside filter input -> Should be allowed and appended
    app.handle_key(key_event(KeyCode::Char('/'))).await;
    assert_eq!(app.filter_input, "a/");

    // Press 'Enter' -> Should commit filter (keeps filter text, exits input mode)
    app.handle_key(key_event(KeyCode::Enter)).await;
    assert!(!app.input_mode);
    assert!(!app.show_filter_input);
    assert_eq!(app.filter_input, "a/");

    // Press '/' to go back to filter input, then Esc -> Should clear filter
    app.handle_key(key_event(KeyCode::Char('/'))).await;
    assert!(app.show_filter_input);
    app.handle_key(key_event(KeyCode::Esc)).await;
    assert!(!app.input_mode);
    assert!(!app.show_filter_input);
    assert!(app.filter_input.is_empty());

    // Press 'f' -> Should open Global Search popup
    app.handle_key(key_event(KeyCode::Char('f'))).await;
    assert!(app.input_mode);
    assert!(app.show_search);
    assert_eq!(app.input_target, lazygitlab::models::InputTarget::Search);

    // Exit search
    app.handle_key(key_event(KeyCode::Esc)).await;
    assert!(!app.show_search);

    // 2. Tab cycling using '[' and ']'
    let mr = lazygitlab::models::MergeRequest {
        iid: 1,
        project_id: 1,
        title: "MR".to_string(),
        description: Some("MR Description".to_string()),
        state: "opened".to_string(),
        web_url: "http://gitlab.com/mr/1".to_string(),
        source_branch: "source".to_string(),
        target_branch: "target".to_string(),
        author: lazygitlab::models::Author {
            id: 1,
            username: "u".to_string(),
            name: Some("U".to_string()),
            state: None,
            avatar_url: None,
            web_url: None,
        },
        assignees: vec![],
        reviewers: vec![],
        draft: Some(false),
        has_conflicts: Some(false),
        detailed_merge_status: Some("mergeable".to_string()),
        squash: Some(false),
        references: None,
        upvotes: None,
        downvotes: None,
        labels: vec![],
        milestone: None,
        created_at: None,
        diff_refs: None,
        reactions: vec![],
        merge_status: None,
        head_pipeline: None,
        participants: vec![],
    };
    app.filtered_mrs = vec![lazygitlab::models::GroupedItem::Item(mr.clone())];
    app.mrs_state.select(Some(0));
    app.selected_mr = Some(mr);
    app.mr_diffs = vec![
        lazygitlab::models::MRDiff {
            old_path: "a".to_string(),
            new_path: "b".to_string(),
            diff: "diff".to_string(),
            collapsed: false,
            display_lines: vec![],
        }
    ];

    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Todos;
    app.todo_filter = lazygitlab::models::TodoFilter::Pending;
    app.handle_key(key_event(KeyCode::Char(']'))).await;
    assert_eq!(app.todo_filter, lazygitlab::models::TodoFilter::Snoozed);

    app.active_panel = Panel::MainDetail;
    app.active_selection = MenuSelection::MergeRequests;
    app.detail_section = lazygitlab::models::DetailSection::Description;
    app.handle_key(key_event(KeyCode::Char(']'))).await;
    assert_eq!(app.detail_section, lazygitlab::models::DetailSection::Discussions);

    app.handle_key(key_event(KeyCode::Char('['))).await;
    assert_eq!(app.detail_section, lazygitlab::models::DetailSection::Metadata);

    // 3. Mouse Click Selection: Detail tabs & Changes panel
    app.set_rect("detail_tabs", ratatui::layout::Rect::new(10, 5, 50, 1));
    app.set_rect("changes_panel", ratatui::layout::Rect::new(10, 6, 50, 10));

    // Handle mouse click on " Discussions " tab
    // Click at x = 10 + 13 + 3 + 2 = 28, y = 5
    app.handle_mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: 28,
        row: 5,
        modifiers: crossterm::event::KeyModifiers::empty(),
    }).await;
    assert_eq!(app.active_panel, Panel::MainDetail);
    assert_eq!(app.detail_section, lazygitlab::models::DetailSection::Discussions);

    // Handle mouse click on changes panel (Tab 2)
    app.detail_section = lazygitlab::models::DetailSection::Changes;
    app.mr_diffs = vec![
        lazygitlab::models::MRDiff {
            old_path: "a".to_string(),
            new_path: "b".to_string(),
            diff: "diff".to_string(),
            collapsed: false,
            display_lines: vec![],
        }
    ];
    app.handle_mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: 15,
        row: 6,
        modifiers: crossterm::event::KeyModifiers::empty(),
    }).await;
    assert_eq!(app.detail_section, lazygitlab::models::DetailSection::Changes);
    assert_eq!(app.mr_diff_state.selected(), Some(0));
}

