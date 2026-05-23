use lazygitlab::models::{App, MergeRequest, Author, Label, DataUpdate, Discussion, Note, Context, MenuSelection, Panel, Issue, Milestone, DiffRefs, MRDiff, Position, DiffDisplayLine, ViewMode, InputTarget};
use gitlab::GitlabBuilder;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use tokio::sync::mpsc;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{path, method};
use std::time::Duration;

async fn setup_app() -> (MockServer, App, mpsc::UnboundedReceiver<DataUpdate>) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    let (tx, rx) = mpsc::unbounded_channel();
    let app = App::new(client, tx);
    (server, app, rx)
}

fn dummy_label(id: u64, name: &str, color: &str) -> Label {
    Label { id, name: name.into(), color: color.into(), text_color: Some("#ffffff".into()) }
}

#[tokio::test]
async fn test_user_story_label_editing() {
    let (server, mut app, mut _rx) = setup_app().await;
    
    // Set context to Project to avoid global refresh clears
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::MergeRequests;
    app.active_panel = Panel::MainDetail;

    let mr = MergeRequest {
        iid: 1, project_id: 1, title: "MR1".into(), description: None, author: Author::new(1, "u".into()),
        web_url: "u".into(), state: "opened".into(), references: None,
        upvotes: Some(0), downvotes: Some(0), labels: vec![dummy_label(0, "bug", "#808080")], 
        milestone: None, assignees: vec![], reviewers: vec![], squash: Some(false), created_at: Some("t".into()), draft: Some(false), 
        diff_refs: None, reactions: vec![],
        merge_status: None, detailed_merge_status: None, has_conflicts: None, head_pipeline: None,
        source_branch: "source".into(), target_branch: "target".into(), participants: vec![]
    };
    app.selected_mr = Some(mr);
    app.detail_section = lazygitlab::models::DetailSection::Metadata;
    app.selected_metadata_field = lazygitlab::models::MetadataField::Labels;

    // 1. Verify Label Fixing via Metadata update
    let full_labels = vec![dummy_label(10, "bug", "#ff0000")];
    app.tx.send(DataUpdate::Metadata { project_id: 1, labels: full_labels, milestones: vec![], members: vec![] }).unwrap();
    
    app.process_updates(&mut _rx).await;

    let fixed_mr = app.selected_mr.as_ref().unwrap();
    assert_eq!(fixed_mr.labels[0].color, "#ff0000", "Label color not fixed from Metadata update");
    assert_eq!(fixed_mr.labels[0].id, 10, "Label ID not fixed from Metadata update");

    // 2. Open Label Popup and Change Labels
    app.enter_detail_action().await;
    assert!(app.show_label_popup);
    assert!(app.selected_label_ids.contains(&10));

    // Deselect label 10
    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)).await;
    assert!(!app.selected_label_ids.contains(&10));

    // Save changes
    Mock::given(method("PUT")).and(path("/api/v4/projects/1/merge_requests/1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"iid":1,"project_id":1,"labels":[]}))).mount(&server).await;
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;
    
    app.process_updates(&mut _rx).await;
    assert!(!app.show_label_popup);
}

#[tokio::test]
async fn test_user_story_discussion_reply() {
    let (server, mut app, mut _rx) = setup_app().await;
    
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::MergeRequests;
    app.active_panel = Panel::MainDetail;

    let mr = MergeRequest {
        iid: 1, project_id: 1, title: "MR1".into(), description: None, author: Author::new(1, "u".into()),
        web_url: "u".into(), state: "opened".into(), references: None,
        upvotes: Some(0), downvotes: Some(0), labels: vec![], 
        milestone: None, assignees: vec![], reviewers: vec![], squash: Some(false), created_at: Some("t".into()), draft: Some(false), 
        diff_refs: None, reactions: vec![],
        merge_status: None, detailed_merge_status: None, has_conflicts: None, head_pipeline: None,
        source_branch: "source".into(), target_branch: "target".into(), participants: vec![]
    };
    app.selected_mr = Some(mr);
    
    let disc = Discussion {
        id: "d1".into(),
        notes: vec![Note { body: "Note 1".into(), author: Author::new(2, "u2".into()), system: false, position: None, context: None }],
        individual_note: false,
        collapsed: false,
        resolved: Some(false),
    };
    app.discussions = vec![disc];
    app.detail_section = lazygitlab::models::DetailSection::Discussions;
    app.discussion_state.select(Some(0));

    // 1. Verify replying to a thread
    app.enter_detail_action().await; // Opens input mode
    assert!(app.input_mode);
    assert_eq!(app.input_target, lazygitlab::models::InputTarget::Reply);

    app.textarea = ratatui_textarea::TextArea::from(vec!["My Reply"]);
    
    Mock::given(method("POST")).and(path("/api/v4/projects/1/merge_requests/1/discussions/d1/notes")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({}))).mount(&server).await;
    // Mock refresh of discussions after post
    Mock::given(method("GET")).and(path("/api/v4/projects/1/merge_requests/1/discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
    Mock::given(method("GET")).and(path("/api/v4/projects/1/merge_requests/1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"iid":1,"project_id":1}))).mount(&server).await;
    Mock::given(method("GET")).and(path("/api/v4/projects/1/merge_requests/1/award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)).await;
    
    // Process updates
    app.process_updates(&mut _rx).await;
    assert!(!app.input_mode);
}

#[tokio::test]
async fn test_user_story_assignee_milestone_editing() {
    let (server, mut app, mut _rx) = setup_app().await;
    
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Issues;
    app.active_panel = Panel::MainDetail;

    let issue = Issue {
        iid: 1, project_id: 1, title: "I1".into(), description: None, author: Author::new(1, "u".into()),
        assignees: vec![], state: "opened".into(), web_url: "u".into(), references: None,
        issue_type: None, labels: vec![], milestone: None, due_date: None, start_date: None, weight: Some(1),
        task_completion_status: None, confidential: false, created_at: Some("t".into()),
        upvotes: Some(0), downvotes: Some(0), reactions: vec![], participants: vec![]
    };
    app.selected_issue = Some(issue);

    // 1. Mock Metadata (Members and Milestones)
    let members = vec![Author::new(2, "dev1".into())];
    let milestones = vec![Milestone { id: 5, title: "Sprint 1".into() }];
    app.tx.send(DataUpdate::Metadata { project_id: 1, labels: vec![], milestones: milestones.clone(), members: members.clone() }).unwrap();
    app.process_updates(&mut _rx).await;

    // 2. Change Assignees
    app.selected_metadata_field = lazygitlab::models::MetadataField::Assignees;
    app.enter_detail_action().await;
    assert!(app.show_assignee_popup);
    
    // Select dev1 (index 0)
    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)).await;
    assert!(app.selected_assignee_ids.contains(&2));

    Mock::given(method("PUT")).and(path("/api/v4/projects/1/issues/1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"iid":1,"project_id":1,"assignee_ids":[2]}))).mount(&server).await;
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;
    app.process_updates(&mut _rx).await;
    assert!(!app.show_assignee_popup);

    // 3. Change Milestone
    app.selected_metadata_field = lazygitlab::models::MetadataField::Milestone;
    app.enter_detail_action().await;
    assert!(app.show_milestone_popup);
    
    // Select Sprint 1 (index 1, as index 0 is None)
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;
    
    app.process_updates(&mut _rx).await;
    assert!(!app.show_milestone_popup);
}

#[tokio::test]
async fn test_user_story_post_comment_and_visibility() {
    let (server, mut app, mut _rx) = setup_app().await;
    
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::MergeRequests;
    app.active_panel = Panel::MainDetail;
    app.view_mode = ViewMode::Diff;

    let mr = MergeRequest {
        iid: 1, project_id: 1, title: "MR1".into(), description: None, author: Author::new(1, "u".into()),
        web_url: "u".into(), state: "opened".into(), references: None,
        upvotes: Some(0), downvotes: Some(0), labels: vec![], 
        milestone: None, assignees: vec![], reviewers: vec![], squash: Some(false), created_at: Some("t".into()), draft: Some(false), 
        diff_refs: Some(DiffRefs { base_sha: "base".into(), head_sha: "head".into(), start_sha: "start".into() }), 
        reactions: vec![],
        merge_status: None, detailed_merge_status: None, has_conflicts: None, head_pipeline: None,
        source_branch: "source".into(), target_branch: "target".into(), participants: vec![]
    };
    app.selected_mr = Some(mr);
    
    let diff = MRDiff {
        old_path: "file.rs".into(),
        new_path: "file.rs".into(),
        diff: "@@ -1,3 +1,3 @@\n line1\n-old\n+new".into(),
        collapsed: false,
        display_lines: Vec::new(),
    };
    app.mr_diffs = vec![diff];
    app.enrich_diffs_with_discussions();
    app.mr_diff_state.select(Some(0));

    // 1. Select the '+' line (visual index 3: header=0, line1=1, old=2, new=3)
    app.diff_line_state.select(Some(3));
    
    // 2. Trigger comment
    app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)).await;
    assert!(app.input_mode);
    assert_eq!(app.input_target, InputTarget::NewComment);

    app.textarea = ratatui_textarea::TextArea::from(vec!["New Review Comment"]);
    
    // 3. Mock the POST and the subsequent GET
    Mock::given(method("POST")).and(path("/api/v4/projects/1/merge_requests/1/discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":"new_disc_id","notes":[{"id":100,"body":"New Review Comment","author":{"id":1,"username":"u"},"system":false,"position":{"new_path":"file.rs","old_path":"file.rs","new_line":2,"old_line":null}}]}))).mount(&server).await;
    
    // Mock the refresh
    let refreshed_discussions = vec![
        Discussion {
            id: "new_disc_id".into(),
            notes: vec![Note { body: "New Review Comment".into(), author: Author::new(1, "u".into()), system: false, position: Some(Position { new_path: Some("file.rs".into()), old_path: Some("file.rs".into()), new_line: Some(2), old_line: None, line_range: None }), context: None }],
            individual_note: false,
            collapsed: false,
            resolved: Some(false),
        }
    ];
    Mock::given(method("GET")).and(path("/api/v4/projects/1/merge_requests/1/discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(refreshed_discussions))).mount(&server).await;
    Mock::given(method("GET")).and(path("/api/v4/projects/1/merge_requests/1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"iid":1,"project_id":1}))).mount(&server).await;
    Mock::given(method("GET")).and(path("/api/v4/projects/1/merge_requests/1/award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;

    // 4. Submit comment
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)).await;
    
    // 5. Process updates (multiple cycles because of background tasks)
    for _ in 0..10 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        app.process_updates(&mut _rx).await;
        let enriched_diff = &app.mr_diffs[0];
        let has_comment = enriched_diff.display_lines.iter().any(|dl| {
            if let DiffDisplayLine::Discussion(d) = dl {
                d.notes[0].body == "New Review Comment"
            } else { false }
        });
        if has_comment {
            break;
        }
    }

    // 6. Verify visibility in diff display lines
    let enriched_diff = &app.mr_diffs[0];
    let has_comment = enriched_diff.display_lines.iter().any(|dl| {
        if let DiffDisplayLine::Discussion(d) = dl {
            d.notes[0].body == "New Review Comment"
        } else { false }
    });
    assert!(has_comment, "Newly posted comment is not visible in the diff view after refresh");
}

#[tokio::test]
async fn test_user_story_multiline_deleted_comment() {
    let (server, mut app, mut _rx) = setup_app().await;
    
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::MergeRequests;
    app.active_panel = Panel::MainDetail;
    app.view_mode = ViewMode::Diff;

    let mr = MergeRequest {
        iid: 1, project_id: 1, title: "MR1".into(), description: None, author: Author::new(1, "u".into()),
        web_url: "u".into(), state: "opened".into(), references: None,
        upvotes: Some(0), downvotes: Some(0), labels: vec![], 
        milestone: None, assignees: vec![], reviewers: vec![], squash: Some(false), created_at: Some("t".into()), draft: Some(false), 
        diff_refs: Some(DiffRefs { base_sha: "base".into(), head_sha: "head".into(), start_sha: "start".into() }), 
        reactions: vec![],
        merge_status: None, detailed_merge_status: None, has_conflicts: None, head_pipeline: None,
        source_branch: "source".into(), target_branch: "target".into(), participants: vec![]
    };
    app.selected_mr = Some(mr);
    
    let diff = MRDiff {
        old_path: "file.rs".into(),
        new_path: "file.rs".into(),
        diff: "@@ -1,4 +1,4 @@\n line1\n-old1\n-old2\n+new".into(),
        collapsed: false,
        display_lines: Vec::new(),
    };
    app.mr_diffs = vec![diff];
    app.enrich_diffs_with_discussions();
    app.mr_diff_state.select(Some(0));

    // In this diff, let's look at the display_lines. We have:
    // Header (index 0)
    // line1 (index 1: old=1, new=1)
    // -old1 (index 2: old=2, new=None)
    // -old2 (index 3: old=3, new=None)
    // +new (index 4: old=None, new=2)
    
    // Select range from index 2 to index 3 (both are deleted lines, starting/ending on None new_line)
    app.diff_line_state.select(Some(3));
    app.diff_selection_start = Some(2);
    
    // Trigger comment
    app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)).await;
    assert!(app.input_mode);
    app.textarea = ratatui_textarea::TextArea::from(vec!["Multiline Deleted Comment"]);
    
    // Mock the POST request. Wiremock matches the path.
    Mock::given(method("POST"))
        .and(path("/api/v4/projects/1/merge_requests/1/discussions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "deleted_range_disc",
            "notes": [{
                "id": 101,
                "body": "Multiline Deleted Comment",
                "author": {"id": 1, "username": "u"},
                "system": false,
                "position": {
                    "new_path": "file.rs",
                    "old_path": "file.rs",
                    "new_line": null,
                    "old_line": 3
                }
            }]
        })))
        .mount(&server)
        .await;

    // Refresh mocks
    Mock::given(method("GET")).and(path("/api/v4/projects/1/merge_requests/1/discussions")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;
    Mock::given(method("GET")).and(path("/api/v4/projects/1/merge_requests/1")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"iid":1,"project_id":1}))).mount(&server).await;
    Mock::given(method("GET")).and(path("/api/v4/projects/1/merge_requests/1/award_emoji")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([]))).mount(&server).await;

    // Submit comment (Ctrl+Enter)
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)).await;
    app.process_updates(&mut _rx).await;

    assert!(!app.input_mode);
}

#[tokio::test]
async fn test_user_story_comment_validation_and_errors() {
    let (_server, mut app, mut _rx) = setup_app().await;
    
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::MergeRequests;
    app.active_panel = Panel::MainDetail;
    app.view_mode = ViewMode::Diff;

    let mr = MergeRequest {
        iid: 1, project_id: 1, title: "MR1".into(), description: None, author: Author::new(1, "u".into()),
        web_url: "u".into(), state: "opened".into(), references: None,
        upvotes: Some(0), downvotes: Some(0), labels: vec![], 
        milestone: None, assignees: vec![], reviewers: vec![], squash: Some(false), created_at: Some("t".into()), draft: Some(false), 
        diff_refs: Some(DiffRefs { base_sha: "base".into(), head_sha: "head".into(), start_sha: "start".into() }), 
        reactions: vec![],
        merge_status: None, detailed_merge_status: None, has_conflicts: None, head_pipeline: None,
        source_branch: "source".into(), target_branch: "target".into(), participants: vec![]
    };
    app.selected_mr = Some(mr);
    
    let diff = MRDiff {
        old_path: "file.rs".into(),
        new_path: "file.rs".into(),
        diff: "@@ -1,3 +1,3 @@\n line1\n-old\n+new".into(),
        collapsed: false,
        display_lines: vec![
            DiffDisplayLine::Code("@@ -1,3 +1,3 @@".into(), None, None), // index 0 (hunk header)
            DiffDisplayLine::Code(" line1".into(), Some(1), Some(1)),       // index 1 (valid unchanged)
            DiffDisplayLine::Code("invalid line".into(), Some(0), Some(0)),  // index 2 (invalid metadata / binary line)
        ],
    };
    app.mr_diffs = vec![diff];
    app.mr_diff_state.select(Some(0));

    // Try commenting on hunk header (index 0)
    app.diff_line_state.select(Some(0));
    app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)).await;
    assert!(!app.input_mode);
    assert_eq!(app.last_error.as_deref(), Some("Cannot comment on this line (no valid line number)"));

    // Clear error
    app.last_error = None;

    // Try commenting on invalid line (index 2)
    app.diff_line_state.select(Some(2));
    app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)).await;
    assert!(!app.input_mode);
    assert_eq!(app.last_error.as_deref(), Some("Cannot comment on this line (no valid line number)"));

    // Clear error
    app.last_error = None;

    // Commenting on valid line (index 1) should work
    app.diff_line_state.select(Some(1));
    app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)).await;
    assert!(app.input_mode);
    assert_eq!(app.input_target, InputTarget::NewComment);
}

#[tokio::test]
async fn test_user_story_kanban_filtering_coherency() {
    let (server, mut app, mut _rx) = setup_app().await;
    
    app.active_context = Context::Project;
    app.selected_project = Some(lazygitlab::models::Project {
        id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()), default_branch: None, namespace: None
    });
    app.active_selection = MenuSelection::Boards;
    app.active_panel = Panel::Main;
    app.view_mode = ViewMode::Board;

    let list = lazygitlab::models::BoardList {
        id: 10,
        label: Some(dummy_label(100, "todo", "#ff0000")),
        position: Some(1),
        list_type: "label".into(),
    };
    app.board_lists = vec![list.clone()];
    app.board_lists_state.select(Some(0));

    let issue1 = Issue {
        iid: 1, project_id: 1, title: "Apple".into(), description: None, author: Author::new(1, "u".into()),
        assignees: vec![], state: "opened".into(), web_url: "u1".into(), references: None,
        issue_type: None, labels: vec![], milestone: None, due_date: None, start_date: None, weight: Some(1),
        task_completion_status: None, confidential: false, created_at: Some("t".into()),
        upvotes: Some(0), downvotes: Some(0), reactions: vec![], participants: vec![]
    };
    let issue2 = Issue {
        iid: 2, project_id: 1, title: "Banana".into(), description: None, author: Author::new(1, "u".into()),
        assignees: vec![], state: "opened".into(), web_url: "u2".into(), references: None,
        issue_type: None, labels: vec![], milestone: None, due_date: None, start_date: None, weight: Some(1),
        task_completion_status: None, confidential: false, created_at: Some("t".into()),
        upvotes: Some(0), downvotes: Some(0), reactions: vec![], participants: vec![]
    };
    let issue3 = Issue {
        iid: 3, project_id: 1, title: "Cherry".into(), description: None, author: Author::new(1, "u".into()),
        assignees: vec![], state: "opened".into(), web_url: "u3".into(), references: None,
        issue_type: None, labels: vec![], milestone: None, due_date: None, start_date: None, weight: Some(1),
        task_completion_status: None, confidential: false, created_at: Some("t".into()),
        upvotes: Some(0), downvotes: Some(0), reactions: vec![], participants: vec![]
    };

    app.board_issues.insert(10, vec![issue1, issue2, issue3]);
    
    // Apply filter "cherry"
    app.filter_input = "cherry".to_string();
    app.filter_lists();

    // Verify filtered issues length is 1 ("Cherry")
    let filtered = app.filtered_board_issues.get(&10).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "Cherry");

    // Initially selection on list 10 is Some(0) (Cherry)
    let state = app.board_issue_states.get(&10).unwrap();
    assert_eq!(state.selected(), Some(0));

    // Try scrolling Down (KeyCode::Down / 'j'). It should clamp to index 0 since filtered len is 1.
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    let state = app.board_issue_states.get(&10).unwrap();
    assert_eq!(state.selected(), Some(0));

    // Verify get_current_board_item_url gets the url for "Cherry" (u3) rather than Apple (u1)
    let url = app.get_current_board_item_url();
    assert_eq!(url, Some("u3".into()));

    // Press Enter to open. Verify app.selected_issue is Cherry.
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;
    assert_eq!(app.view_mode, ViewMode::Detail);
    assert_eq!(app.selected_issue.as_ref().unwrap().title, "Cherry");

    // Revert view mode and select list 10 again
    app.view_mode = ViewMode::Board;

    // Test moving: move_board_issue should target Cherry (iid: 3)
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/issues/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    // Trigger moving the board issue. (List 10 is the only list, but we can call move_board_issue(1))
    // Note: since lists.len() is 1, let's add a second list to allow moving.
    let list2 = lazygitlab::models::BoardList {
        id: 20,
        label: Some(dummy_label(200, "done", "#00ff00")),
        position: Some(2),
        list_type: "label".into(),
    };
    app.board_lists = vec![list.clone(), list2.clone()];
    app.board_lists_state.select(Some(0));
    app.clamp_selections();

    let res = app.move_board_issue(1).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_user_story_tree_scroll_and_clamp() {
    let (server, mut app, mut _rx) = setup_app().await;
    
    app.active_context = Context::Project;
    app.selected_project = Some(lazygitlab::models::Project {
        id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()), default_branch: None, namespace: None
    });
    app.active_selection = MenuSelection::Repository;
    app.active_panel = Panel::Main;

    // Simulate selecting a blob and opening it
    let item = lazygitlab::models::RepositoryTreeItem {
        id: "1".into(), name: "file.txt".into(), r#type: "blob".into(), path: "file.txt".into(), mode: "100644".into()
    };
    app.repository_tree = vec![item];
    app.repository_tree_state.select(vec!["file.txt".into()]);

    // Set scroll to non-zero before entering tree view
    app.job_log_scroll = 50;

    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/repository/files/file.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "file_name": "file.txt",
            "file_path": "file.txt",
            "size": 10,
            "encoding": "base64",
            "content_sha256": "abc",
            "ref": "master",
            "blob_id": "blob1",
            "commit_id": "c1",
            "last_commit_id": "c1",
            "content": "bGluZTEKbGluZTIKbGluZTMKbGluZTQKbGluZTU=" // "line1\nline2\nline3\nline4\nline5"
        })))
        .mount(&server)
        .await;

    // Press Enter to open blob
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;
    for _ in 0..10 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        app.process_updates(&mut _rx).await;
        if app.selected_file_content.is_some() {
            break;
        }
    }

    // Verify view mode is Tree, and scroll is reset to 0
    assert_eq!(app.view_mode, ViewMode::Tree);
    assert_eq!(app.job_log_scroll, 0);

    // Set viewport height to 2. Total lines is 5.
    app.job_log_viewport_height = 2;

    // Press Down (j). Scroll should increment.
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    assert_eq!(app.job_log_scroll, 1);

    // Scroll more. Max scroll should be saturating_sub(5, 2) = 3.
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    assert_eq!(app.job_log_scroll, 3);

    // Extra scroll. Scroll should still be clamped at 3.
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    assert_eq!(app.job_log_scroll, 3);

    // Scroll up (k).
    app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)).await;
    assert_eq!(app.job_log_scroll, 2);
}

#[tokio::test]
async fn test_user_story_diff_navigation_and_reviewers() {
    let (server, mut app, mut _rx) = setup_app().await;

    // 1. Test Diff Navigation Skipping Hunk Headers
    app.active_context = Context::Project;
    app.active_selection = MenuSelection::MergeRequests;
    app.active_panel = Panel::MainDetail;
    app.view_mode = ViewMode::Diff;

    let mr = MergeRequest {
        iid: 1, project_id: 1, title: "MR1".into(), description: None, author: Author::new(1, "u".into()),
        web_url: "u".into(), state: "opened".into(), references: None,
        upvotes: Some(0), downvotes: Some(0), labels: vec![],
        milestone: None, assignees: vec![], reviewers: vec![], squash: Some(false), created_at: Some("t".into()), draft: Some(false),
        diff_refs: Some(DiffRefs { base_sha: "base".into(), head_sha: "head".into(), start_sha: "start".into() }),
        reactions: vec![],
        merge_status: None, detailed_merge_status: None, has_conflicts: None, head_pipeline: None,
        source_branch: "source".into(), target_branch: "target".into(), participants: vec![]
    };
    app.selected_mr = Some(mr);

    let diff = MRDiff {
        old_path: "file.rs".into(),
        new_path: "file.rs".into(),
        diff: "@@ -1,3 +1,3 @@\n line1\n-old\n+new".into(),
        collapsed: false,
        display_lines: Vec::new(),
    };
    app.mr_diffs = vec![diff];
    app.enrich_diffs_with_discussions();
    app.mr_diff_state.select(Some(0));

    // Verify display lines:
    // Index 0: @@ -1,3 +1,3 @@ (header) -> not selectable (old=None, new=None)
    // Index 1:  line1 -> selectable (old=Some(1), new=Some(1))
    // Index 2: -old -> selectable (old=Some(2), new=None)
    // Index 3: +new -> selectable (old=None, new=Some(2))
    
    // Press Right (l/Enter) to enter diff lines. It should skip the header and select index 1.
    app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)).await;
    assert_eq!(app.diff_line_state.selected(), Some(1));

    // Press Down (j). It should move to index 2.
    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)).await;
    assert_eq!(app.diff_line_state.selected(), Some(2));

    // Press Down (j). It should move to index 3.
    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)).await;
    assert_eq!(app.diff_line_state.selected(), Some(3));

    // Press Down (j). It should wrap and skip index 0 (header), selecting index 1.
    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)).await;
    assert_eq!(app.diff_line_state.selected(), Some(1));

    // Press Up (k). It should wrap and skip index 0, selecting index 3.
    app.handle_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)).await;
    assert_eq!(app.diff_line_state.selected(), Some(3));

    // 2. Test Reviewers popup and updates
    app.active_panel = Panel::MainDetail;
    app.detail_section = lazygitlab::models::DetailSection::Metadata;
    app.selected_metadata_field = lazygitlab::models::MetadataField::Reviewers;

    // Mock Members list in Metadata
    let members = vec![Author::new(2, "dev1".into())];
    app.tx.send(DataUpdate::Metadata { project_id: 1, labels: vec![], milestones: vec![], members: members.clone() }).unwrap();
    app.process_updates(&mut _rx).await;

    // Press Enter to open reviewers popup
    app.enter_detail_action().await;
    assert!(app.show_reviewer_popup);

    // Toggle dev1 (index 0)
    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)).await;
    assert!(app.selected_reviewer_ids.contains(&2));

    // Save changes
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/merge_requests/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"iid":1,"project_id":1,"reviewer_ids":[2]})))
        .mount(&server)
        .await;

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;
    app.process_updates(&mut _rx).await;
    assert!(!app.show_reviewer_popup);
}

#[tokio::test]
async fn test_user_story_target_branch_editing() {
    let (server, mut app, mut _rx) = setup_app().await;

    app.active_context = Context::Project;
    app.active_selection = MenuSelection::MergeRequests;
    app.active_panel = Panel::MainDetail;
    app.detail_section = lazygitlab::models::DetailSection::Metadata;
    app.selected_metadata_field = lazygitlab::models::MetadataField::TargetBranch;

    let mr = MergeRequest {
        iid: 1, project_id: 1, title: "MR1".into(), description: None, author: Author::new(1, "u".into()),
        web_url: "u".into(), state: "opened".into(), references: None,
        upvotes: Some(0), downvotes: Some(0), labels: vec![], 
        milestone: None, assignees: vec![], reviewers: vec![], squash: Some(false), created_at: Some("t".into()), draft: Some(false), 
        diff_refs: None, reactions: vec![],
        merge_status: None, detailed_merge_status: None, has_conflicts: None, head_pipeline: None,
        source_branch: "source".into(), target_branch: "master".into(), participants: vec![]
    };
    app.selected_mr = Some(mr);

    // Mock branches endpoint
    Mock::given(method("GET"))
        .and(path("/api/v4/projects/1/repository/branches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"name": "master"},
            {"name": "feature"}
        ])))
        .mount(&server)
        .await;

    // Press Enter to trigger selection popup, which fetches branches
    app.enter_detail_action().await;
    
    let mut fetched = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        app.process_updates(&mut _rx).await;
        if app.show_target_branch_popup && app.project_branches.len() == 2 {
            fetched = true;
            break;
        }
    }
    assert!(fetched);
    assert_eq!(app.project_branches[0].name, "master");
    assert_eq!(app.project_branches[1].name, "feature");
    assert_eq!(app.target_branch_state.selected(), Some(0)); // Current target branch "master" was selected

    // Press Down to select "feature" (index 1)
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)).await;
    assert_eq!(app.target_branch_state.selected(), Some(1));

    // Mock PUT to edit target branch
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/merge_requests/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "iid": 1, "project_id": 1, "title": "MR1", "author": {"id": 1, "username": "u"},
            "web_url": "u", "state": "opened", "source_branch": "source", "target_branch": "feature"
        })))
        .mount(&server)
        .await;

    // Press Enter to save
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;
    
    let mut updated = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        app.process_updates(&mut _rx).await;
        if app.selected_mr.as_ref().map(|m| m.target_branch.as_str()) == Some("feature") {
            updated = true;
            break;
        }
    }
    assert!(updated);
    assert!(!app.show_target_branch_popup);
}

struct ConfidentialMatcher;
impl wiremock::Match for ConfidentialMatcher {
    fn matches(&self, request: &wiremock::Request) -> bool {
        let body_str = String::from_utf8_lossy(&request.body);
        body_str.contains("confidential=")
    }
}

struct StartDateMatcher;
impl wiremock::Match for StartDateMatcher {
    fn matches(&self, request: &wiremock::Request) -> bool {
        let body_str = String::from_utf8_lossy(&request.body);
        body_str.contains("start_date=")
    }
}

struct DueDateMatcher;
impl wiremock::Match for DueDateMatcher {
    fn matches(&self, request: &wiremock::Request) -> bool {
        let body_str = String::from_utf8_lossy(&request.body);
        body_str.contains("due_date=")
    }
}

struct LabelsMatcher;
impl wiremock::Match for LabelsMatcher {
    fn matches(&self, request: &wiremock::Request) -> bool {
        let body_str = String::from_utf8_lossy(&request.body);
        body_str.contains("labels=")
    }
}

#[tokio::test]
async fn test_user_story_issue_metadata_editing() {
    let (server, mut app, mut _rx) = setup_app().await;

    app.active_context = Context::Project;
    app.active_selection = MenuSelection::Issues;
    app.active_panel = Panel::MainDetail;
    app.detail_section = lazygitlab::models::DetailSection::Metadata;

    let issue = Issue {
        iid: 1,
        project_id: 1,
        title: "I1".into(),
        description: None,
        author: Author::new(1, "u".into()),
        assignees: vec![],
        state: "opened".into(),
        web_url: "u".into(),
        references: None,
        issue_type: None,
        labels: vec![],
        milestone: None,
        due_date: None,
        start_date: None,
        weight: Some(1),
        task_completion_status: None,
        confidential: false,
        created_at: Some("t".into()),
        upvotes: Some(0),
        downvotes: Some(0),
        reactions: vec![],
        participants: vec![],
    };
    app.selected_issue = Some(issue);

    // Mount PUT mock for toggling confidentiality
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/issues/1"))
        .and(ConfidentialMatcher)
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "iid": 1, "project_id": 1, "title": "I1", "author": {"id": 1, "username": "u"},
            "web_url": "u", "state": "opened", "confidential": true, "weight": 1, "labels": []
        })))
        .mount(&server)
        .await;

    // Mount PUT mock for saving start_date
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/issues/1"))
        .and(StartDateMatcher)
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "iid": 1, "project_id": 1, "title": "I1", "author": {"id": 1, "username": "u"},
            "web_url": "u", "state": "opened", "confidential": true, "start_date": "2026-05-23", "labels": []
        })))
        .mount(&server)
        .await;

    // Mount PUT mock for saving due_date
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/issues/1"))
        .and(DueDateMatcher)
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "iid": 1, "project_id": 1, "title": "I1", "author": {"id": 1, "username": "u"},
            "web_url": "u", "state": "opened", "confidential": true, "start_date": "2026-05-23", "due_date": "2026-05-25", "labels": []
        })))
        .mount(&server)
        .await;

    // Mount PUT mock for saving labels
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/issues/1"))
        .and(LabelsMatcher)
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "iid": 1, "project_id": 1, "title": "I1", "author": {"id": 1, "username": "u"},
            "web_url": "u", "state": "opened", "confidential": true, "start_date": "2026-05-23", "due_date": "2026-05-25",
            "labels": [{"id": 10, "name": "bug", "color": "#ff0000", "text_color": "#ffffff"}]
        })))
        .mount(&server)
        .await;

    // 1. Test Toggle Confidentiality
    app.selected_metadata_field = lazygitlab::models::MetadataField::Confidential;

    // Press Enter to trigger toggling confidentiality
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;

    let mut confidential_updated = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        app.process_updates(&mut _rx).await;
        if app.selected_issue.as_ref().map(|i| i.confidential) == Some(true) {
            confidential_updated = true;
            break;
        }
    }
    assert!(confidential_updated);

    // 2. Test StartDate Editing
    app.selected_metadata_field = lazygitlab::models::MetadataField::StartDate;
    app.enter_detail_action().await;
    assert!(app.show_calendar_popup);

    app.calendar_date = chrono::NaiveDate::from_ymd_opt(2026, 5, 23).unwrap();
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;

    let mut start_date_updated = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        app.process_updates(&mut _rx).await;
        if app.selected_issue.as_ref().and_then(|i| i.start_date.as_deref()) == Some("2026-05-23") {
            start_date_updated = true;
            break;
        }
    }
    assert!(start_date_updated);
    assert!(!app.show_calendar_popup);

    // 3. Test DueDate Editing
    app.selected_metadata_field = lazygitlab::models::MetadataField::DueDate;
    app.enter_detail_action().await;
    assert!(app.show_calendar_popup);

    app.calendar_date = chrono::NaiveDate::from_ymd_opt(2026, 5, 25).unwrap();
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;

    let mut due_date_updated = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        app.process_updates(&mut _rx).await;
        if app.selected_issue.as_ref().and_then(|i| i.due_date.as_deref()) == Some("2026-05-25") {
            due_date_updated = true;
            break;
        }
    }
    assert!(due_date_updated);
    assert!(!app.show_calendar_popup);

    // 3. Test Labels Editing
    let full_labels = vec![dummy_label(10, "bug", "#ff0000")];
    app.tx.send(DataUpdate::Metadata { project_id: 1, labels: full_labels, milestones: vec![], members: vec![] }).unwrap();
    app.process_updates(&mut _rx).await;

    app.selected_metadata_field = lazygitlab::models::MetadataField::Labels;
    app.enter_detail_action().await;
    assert!(app.show_label_popup);

    // Select label at index 0 (ID 10)
    app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)).await;
    assert!(app.selected_label_ids.contains(&10));

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)).await;

    let mut labels_updated = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        app.process_updates(&mut _rx).await;
        if let Some(iss) = &app.selected_issue {
            if iss.labels.iter().any(|l| l.id == 10) {
                labels_updated = true;
                break;
            }
        }
    }
    assert!(labels_updated);
    assert!(!app.show_label_popup);
}

