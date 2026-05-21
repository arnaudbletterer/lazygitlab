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
        milestone: None, assignees: vec![], created_at: Some("t".into()), draft: Some(false), 
        diff_refs: None, reactions: vec![]
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
        milestone: None, assignees: vec![], created_at: Some("t".into()), draft: Some(false), 
        diff_refs: None, reactions: vec![]
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
        issue_type: None, labels: vec![], milestone: None, due_date: None, weight: Some(1),
        task_completion_status: None, confidential: false, created_at: Some("t".into()),
        upvotes: Some(0), downvotes: Some(0), reactions: vec![]
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
        milestone: None, assignees: vec![], created_at: Some("t".into()), draft: Some(false), 
        diff_refs: Some(DiffRefs { base_sha: "base".into(), head_sha: "head".into(), start_sha: "start".into() }), 
        reactions: vec![]
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
