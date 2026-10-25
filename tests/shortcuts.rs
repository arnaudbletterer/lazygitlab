use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use gitlab::GitlabBuilder;
use lazygitlab::models::{App, Context, MenuSelection, Panel};
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

#[tokio::test]
async fn test_g_prefix_shortcuts() {
    let (_server, mut app) = setup_app().await;

    // Initial state (starts on Home Launchpad)
    assert_eq!(app.active_context, Context::Dashboard);
    assert_eq!(app.active_selection, MenuSelection::Home);
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

    // Press 'g' then 'h' -> Should return to Home
    app.handle_key(key_event(KeyCode::Char('g'))).await;
    app.handle_key(key_event(KeyCode::Char('h'))).await;
    assert_eq!(app.active_selection, MenuSelection::Home);

    // In project context: test 'g b' (Branches) and 'g r' (Repository)
    app.selected_project = Some(lazygitlab::models::Project {
        id: 1,
        name: "test-proj".into(),
        path_with_namespace: Some("grp/test-proj".into()),
        web_url: None,
        default_branch: Some("main".into()),
        namespace: None,
    });

    app.handle_key(key_event(KeyCode::Char('g'))).await;
    app.handle_key(key_event(KeyCode::Char('b'))).await;
    assert_eq!(app.active_selection, MenuSelection::Branches);
    assert_eq!(app.active_context, Context::Project);

    app.handle_key(key_event(KeyCode::Char('g'))).await;
    app.handle_key(key_event(KeyCode::Char('r'))).await;
    assert_eq!(app.active_selection, MenuSelection::Repository);
    assert_eq!(app.active_context, Context::Project);
}

#[tokio::test]
async fn test_available_actions_catalog() {
    let (_server, app) = setup_app().await;

    let actions = app.get_available_actions();
    assert!(
        actions
            .iter()
            .any(|a| a.shortcut == "p" && a.category == "Global")
    );
    assert!(
        actions
            .iter()
            .any(|a| a.shortcut == "q" && a.category == "Global")
    );
    assert!(
        actions
            .iter()
            .any(|a| a.shortcut == "?" && a.category == "Global")
    );
    assert!(
        actions
            .iter()
            .any(|a| a.shortcut == "r" && a.category == "Global")
    );
    assert!(
        actions
            .iter()
            .any(|a| a.shortcut == "f" && a.category == "Global")
    );
    assert!(
        actions
            .iter()
            .any(|a| a.shortcut == "g b" && a.category == "Jump")
    );
    assert!(
        actions
            .iter()
            .any(|a| a.shortcut == "g r" && a.category == "Jump")
    );
    assert!(
        actions
            .iter()
            .any(|a| a.shortcut == "g t" && a.category == "Jump")
    );
    assert!(
        actions
            .iter()
            .any(|a| a.shortcut == "g m" && a.category == "Jump")
    );
    assert!(
        actions
            .iter()
            .any(|a| a.shortcut == "g i" && a.category == "Jump")
    );
    assert!(
        actions
            .iter()
            .any(|a| a.shortcut == "g p" && a.category == "Jump")
    );
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
        updated_at: None,
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
    app.mr_diffs = vec![lazygitlab::models::MRDiff {
        old_path: "a".to_string(),
        new_path: "b".to_string(),
        diff: "diff".to_string(),
        collapsed: false,
        display_lines: vec![],
    }];

    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Todos;
    app.todo_filter = lazygitlab::models::TodoFilter::Pending;
    app.handle_key(key_event(KeyCode::Char(']'))).await;
    assert_eq!(app.todo_filter, lazygitlab::models::TodoFilter::Snoozed);

    app.active_panel = Panel::MainDetail;
    app.active_selection = MenuSelection::MergeRequests;
    app.detail_section = lazygitlab::models::DetailSection::Description;
    app.handle_key(key_event(KeyCode::Char(']'))).await;
    assert_eq!(
        app.detail_section,
        lazygitlab::models::DetailSection::Discussions
    );

    app.handle_key(key_event(KeyCode::Char('['))).await;
    assert_eq!(
        app.detail_section,
        lazygitlab::models::DetailSection::Metadata
    );

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
    })
    .await;
    assert_eq!(app.active_panel, Panel::MainDetail);
    assert_eq!(
        app.detail_section,
        lazygitlab::models::DetailSection::Discussions
    );

    // Handle mouse click on changes panel (Tab 2)
    app.detail_section = lazygitlab::models::DetailSection::Changes;
    app.mr_diffs = vec![lazygitlab::models::MRDiff {
        old_path: "a".to_string(),
        new_path: "b".to_string(),
        diff: "diff".to_string(),
        collapsed: false,
        display_lines: vec![],
    }];
    app.handle_mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: 15,
        row: 6,
        modifiers: crossterm::event::KeyModifiers::empty(),
    })
    .await;
    assert_eq!(
        app.detail_section,
        lazygitlab::models::DetailSection::Changes
    );
    assert_eq!(app.mr_diff_state.selected(), Some(0));
}

#[tokio::test]
async fn test_work_item_and_mr_detail_navigation_coherence() {
    let (_server, mut app) = setup_app().await;

    app.selected_mr = Some(lazygitlab::models::MergeRequest {
        updated_at: None,
        iid: 1,
        project_id: 1,
        title: "Feature MR".into(),
        description: Some("Long description text\nline 2\nline 3\nline 4".into()),
        author: lazygitlab::models::Author::new(1, "author".into()),
        web_url: "http://example.com/mr/1".into(),
        state: "opened".into(),
        references: None,
        upvotes: Some(0),
        downvotes: Some(0),
        labels: vec![],
        milestone: None,
        assignees: vec![],
        reviewers: vec![],
        squash: Some(false),
        created_at: Some("2026-08-30T10:00:00Z".into()),
        draft: Some(false),
        diff_refs: None,
        reactions: vec![],
        merge_status: None,
        detailed_merge_status: Some("mergeable".into()),
        has_conflicts: Some(false),
        head_pipeline: None,
        source_branch: "feature".into(),
        target_branch: "main".into(),
        participants: vec![],
    });

    app.active_selection = MenuSelection::MergeRequests;
    app.active_panel = Panel::MainDetail;
    app.detail_section = lazygitlab::models::DetailSection::Metadata;
    app.selected_metadata_field = lazygitlab::models::MetadataField::Title;

    // 1. Step down in MR Metadata
    app.handle_key(key_event(KeyCode::Char('j'))).await;
    assert_eq!(
        app.selected_metadata_field,
        lazygitlab::models::MetadataField::TargetBranch
    );

    app.handle_key(key_event(KeyCode::Char('j'))).await;
    assert_eq!(
        app.selected_metadata_field,
        lazygitlab::models::MetadataField::Labels
    );

    app.handle_key(key_event(KeyCode::Char('j'))).await;
    assert_eq!(
        app.selected_metadata_field,
        lazygitlab::models::MetadataField::Assignees
    );

    app.handle_key(key_event(KeyCode::Char('j'))).await;
    assert_eq!(
        app.selected_metadata_field,
        lazygitlab::models::MetadataField::Reviewers
    );

    app.handle_key(key_event(KeyCode::Char('j'))).await;
    assert_eq!(
        app.selected_metadata_field,
        lazygitlab::models::MetadataField::Milestone
    );

    // Pressing 'j' at the bottom of MR metadata MUST clamp at Milestone and NOT jump to Description!
    app.handle_key(key_event(KeyCode::Char('j'))).await;
    assert_eq!(
        app.detail_section,
        lazygitlab::models::DetailSection::Metadata
    );
    assert_eq!(
        app.selected_metadata_field,
        lazygitlab::models::MetadataField::Milestone
    );

    // 2. Direct jump to Description via 'd'
    app.handle_key(key_event(KeyCode::Char('d'))).await;
    assert_eq!(
        app.detail_section,
        lazygitlab::models::DetailSection::Description
    );
    assert_eq!(app.description_scroll, 0);

    // Pressing 'j' in Description scrolls down smoothly
    app.handle_key(key_event(KeyCode::Char('j'))).await;
    assert_eq!(app.description_scroll, 1);
    app.handle_key(key_event(KeyCode::Char('j'))).await;
    assert_eq!(app.description_scroll, 2);
    app.handle_key(key_event(KeyCode::Char('k'))).await;
    assert_eq!(app.description_scroll, 1);

    // 3. Direct jump back to Metadata via 'm'
    app.handle_key(key_event(KeyCode::Char('m'))).await;
    assert_eq!(
        app.detail_section,
        lazygitlab::models::DetailSection::Metadata
    );

    // 4. Pressing 'e' in Description enters editing mode
    app.handle_key(key_event(KeyCode::Char('d'))).await;
    assert_eq!(
        app.detail_section,
        lazygitlab::models::DetailSection::Description
    );
    assert!(!app.input_mode);

    app.handle_key(key_event(KeyCode::Char('e'))).await;
    assert!(app.input_mode);
    assert_eq!(
        app.input_target,
        lazygitlab::models::InputTarget::EditDescription
    );
}
