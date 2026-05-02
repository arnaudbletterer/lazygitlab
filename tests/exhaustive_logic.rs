use lazygitlab::models::{App, DataUpdate, MenuSelection, ViewMode, Project, MergeRequest, Issue, Pipeline, Job, Discussion, MRDiff};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::path;
use gitlab::GitlabBuilder;
use tokio::sync::mpsc;
use std::time::Duration;

async fn setup_app() -> (MockServer, App, mpsc::UnboundedReceiver<DataUpdate>) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    let (tx, rx) = mpsc::unbounded_channel();
    let mut app = App::new(client);
    app.tx = tx;
    (server, app, rx)
}

#[tokio::test]
async fn test_process_updates_exhaustive() {
    let (_server, mut app, mut rx) = setup_app().await;

    let proj = Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()) };
    let mr = MergeRequest {
        iid: 1, project_id: 1, title: "m".into(), description: None,
        author: lazygitlab::models::Author { id: 1, username: "u".into() },
        web_url: "u".into(), state: "o".into(), references: lazygitlab::models::References { full: "r".into() },
        upvotes: 0, downvotes: 0, labels: vec![], milestone: None, assignees: vec![], created_at: "t".into(), reactions: vec![],
    };
    let issue = Issue {
        iid: 1, project_id: 1, title: "i".into(), description: None,
        author: lazygitlab::models::Author { id: 1, username: "u".into() },
        assignees: vec![], state: "o".into(), web_url: "u".into(),
        references: lazygitlab::models::References { full: "r".into() },
        issue_type: None, labels: vec![], milestone: None, due_date: None, weight: None, task_completion_status: None,
        confidential: false, created_at: "t".into(), upvotes: 0, downvotes: 0, reactions: vec![],
    };
    let todo = lazygitlab::models::Todo { id: 1, project: proj.clone(), action_name: "a".into(), target_type: "Issue".into(), target_url: "u".into(), body: "b".into(), state: "p".into(), created_at: "t".into() };
    let group = lazygitlab::models::Group { id: 1, name: "g".into(), path: "g".into(), full_path: "g".into() };
    let pipe = Pipeline {
        id: 1, project_id: 1, status: "s".into(), source: "s".into(), ref_name: "r".into(),
        web_url: "u".into(), created_at: "t".into(), started_at: None, duration: None,
    };

    let updates = vec![
        DataUpdate::Todos { data: vec![todo.clone()], page: 1, has_more: false },
        DataUpdate::AppendTodos(vec![todo.clone()]),
        DataUpdate::Projects { data: vec![proj.clone()], page: 1, has_more: false },
        DataUpdate::AppendProjects(vec![proj.clone()]),
        DataUpdate::Groups { data: vec![group.clone()], page: 1, has_more: false },
        DataUpdate::AppendGroups(vec![group.clone()]),
        DataUpdate::GlobalMRs { data: vec![mr.clone()], page: 1, has_more: false },
        DataUpdate::ClearGlobalMRs,
        DataUpdate::AppendGlobalMRs(vec![mr.clone()]),
        DataUpdate::GlobalIssues { data: vec![issue.clone()], page: 1, has_more: false },
        DataUpdate::ClearGlobalIssues,
        DataUpdate::AppendGlobalIssues(vec![issue.clone()]),
        DataUpdate::ProjectMRs { data: vec![mr.clone()], page: 1, has_more: false },
        DataUpdate::ProjectIssues { data: vec![issue.clone()], page: 1, has_more: false },
        DataUpdate::ProjectPipelines { data: vec![pipe.clone()], page: 1, has_more: false },
        DataUpdate::Discussions(vec![]),
        DataUpdate::IssueDiscussions(vec![]),
        DataUpdate::Jobs(vec![]),
        DataUpdate::Log("line\n".into()),
        DataUpdate::Diffs(vec![]),
        DataUpdate::Metadata { labels: vec![], milestones: vec![], members: vec![] },
        DataUpdate::Loading(true),
        DataUpdate::Loading(false),
        DataUpdate::Error("test error".into()),
    ];

    for u in updates {
        app.tx.send(u).unwrap();
    }
    app.process_updates(&mut rx);
    
    assert_eq!(app.last_error, Some("test error".into()));
    assert!(!app.is_loading);
}

#[tokio::test]
async fn test_keyboard_exhaustive() {
    let (_server, mut app, _rx) = setup_app().await;
    use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind, KeyEventState};
    fn key(code: KeyCode) -> KeyEvent { KeyEvent { code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE } }
    fn key_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent { KeyEvent { code, modifiers, kind: KeyEventKind::Press, state: KeyEventState::NONE } }

    let popups = [
        (|a: &mut App| a.show_project_popup = true),
        (|a: &mut App| a.show_calendar_popup = true),
        (|a: &mut App| a.show_label_popup = true),
        (|a: &mut App| a.show_assignee_popup = true),
        (|a: &mut App| a.show_milestone_popup = true),
        (|a: &mut App| a.show_weight_popup = true),
        (|a: &mut App| { a.show_pipeline_popup = true; a.input_target = lazygitlab::models::InputTarget::PipelineVariableList; }),
        (|a: &mut App| { a.show_pipeline_popup = true; a.show_branches_dropdown = true; }),
        (|a: &mut App| a.show_help_popup = true),
        (|a: &mut App| { a.show_help_popup = true; a.show_help_filter = true; }),
        (|a: &mut App| a.show_create_issue_popup = true),
        (|a: &mut App| a.show_create_mr_popup = true),
    ];

    let keys = [KeyCode::Char('q'), KeyCode::Char('?'), KeyCode::Char('d'), KeyCode::Char('a'), KeyCode::Char('m'), KeyCode::Char('/'), KeyCode::Char('p'), KeyCode::Char(' '), KeyCode::Enter, KeyCode::Esc, KeyCode::Tab, KeyCode::Char('h'), KeyCode::Char('l'), KeyCode::Backspace, KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down, KeyCode::PageUp, KeyCode::PageDown, KeyCode::Home, KeyCode::End, KeyCode::Delete];

    for setup in popups {
        app.show_project_popup = false; app.show_calendar_popup = false; app.show_label_popup = false;
        app.show_assignee_popup = false; app.show_milestone_popup = false; app.show_weight_popup = false; app.show_pipeline_popup = false;
        app.show_help_popup = false; app.show_help_filter = false; app.show_create_issue_popup = false; app.show_create_mr_popup = false;
        app.input_mode = false;
        setup(&mut app);
        for k in &keys { app.handle_key(key(k.clone())).await; }
    }

    app.input_mode = true;
    let targets = [
        lazygitlab::models::InputTarget::Reply, lazygitlab::models::InputTarget::NewComment, lazygitlab::models::InputTarget::EditTitle, lazygitlab::models::InputTarget::EditDescription,
        lazygitlab::models::InputTarget::PipelineRef, lazygitlab::models::InputTarget::PipelineVariableKey,
        lazygitlab::models::InputTarget::PipelineVariableValue, lazygitlab::models::InputTarget::Filter,
        lazygitlab::models::InputTarget::CreateIssueTitle, lazygitlab::models::InputTarget::CreateMRTitle,
        lazygitlab::models::InputTarget::PipelineRefSearch, lazygitlab::models::InputTarget::CreateIssueDescription,
        lazygitlab::models::InputTarget::CreateMRDescription, lazygitlab::models::InputTarget::CreateMRSourceBranch,
        lazygitlab::models::InputTarget::CreateMRTargetBranch, lazygitlab::models::InputTarget::PipelineVariableList,
    ];
    for target in targets {
        app.input_target = target;
        app.cursor_index = 0;
        for k in &keys { app.handle_key(key(k.clone())).await; }
        app.handle_key(key_mod(KeyCode::Enter, KeyModifiers::SHIFT)).await;
    }
    app.input_mode = false;
}

#[tokio::test]
async fn test_back_navigation_exhaustive() {
    let (_server, mut app, _rx) = setup_app().await;

    // Detail -> List
    app.view_mode = ViewMode::Detail;
    app.back().await;
    assert_eq!(app.view_mode, ViewMode::List);

    // JobLog -> Detail
    app.view_mode = ViewMode::JobLog;
    app.back().await;
    assert_eq!(app.view_mode, ViewMode::Detail);

    // Diff -> Detail
    app.view_mode = ViewMode::Diff;
    app.back().await;
    assert_eq!(app.view_mode, ViewMode::Detail);

    // List -> List (No change)
    app.view_mode = ViewMode::List;
    app.back().await;
    assert_eq!(app.view_mode, ViewMode::List);
}

#[tokio::test]
async fn test_exhaustive_app_logic() {
    let (server, mut app, mut rx) = setup_app().await;

    // Test Refresh in all modes
    app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()) });
    
    // List MRs
    app.active_selection = MenuSelection::MergeRequests;
    app.view_mode = ViewMode::List;
    app.selected_mr = None;
    app.selected_issue = None;
    app.selected_pipeline = None;
    Mock::given(path("/api/v4/projects/1/merge_requests")).respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({"iid":1,"project_id":1,"title":"m","author":{"id":1,"username":"u"},"web_url":"u","state":"o","references":{"full":"r"},"upvotes":0,"downvotes":0,"labels":[],"milestone":null,"assignees":[],"created_at":"t"})])).mount(&server).await;
    app.refresh().await;
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while app.is_loading { app.process_updates(&mut rx); tokio::time::sleep(Duration::from_millis(10)).await; }
    }).await;
    assert!(!app.merge_requests.is_empty());

    // Detail MR Discussions
    app.view_mode = ViewMode::Detail;
    app.selected_mr = Some(app.merge_requests[0].clone());
    app.selected_issue = None;
    app.selected_pipeline = None;
    Mock::given(path("/api/v4/projects/1/merge_requests/1/discussions")).respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({"id":"1","individual_note":false,"notes":[{"id":1,"body":"b","author":{"id":1,"username":"u"},"created_at":"t","system":false,"resolved":false}]})])).mount(&server).await;
    app.refresh().await;
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while app.is_loading { app.process_updates(&mut rx); tokio::time::sleep(Duration::from_millis(10)).await; }
    }).await;
    assert!(!app.discussions.is_empty());

    // Diff MR
    app.view_mode = ViewMode::Diff;
    // selected_mr is still needed for Diff
    Mock::given(path("/api/v4/projects/1/merge_requests/1/diffs")).respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({"old_path":"a","new_path":"b","diff":"+line","new_file":false,"renamed_file":false,"deleted_file":false})])).mount(&server).await;
    app.refresh().await;
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while app.is_loading { app.process_updates(&mut rx); tokio::time::sleep(Duration::from_millis(10)).await; }
    }).await;
    assert!(!app.mr_diffs.is_empty());

    // Pipelines
    app.active_selection = MenuSelection::Pipelines;
    app.view_mode = ViewMode::List;
    app.selected_mr = None;
    app.selected_issue = None;
    app.selected_pipeline = None;
    Mock::given(path("/api/v4/projects/1/pipelines")).respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({"id":1,"project_id":1,"status":"success","source":"push","ref":"master","web_url":"u","created_at":"2023-01-01T00:00:00Z"})])).mount(&server).await;
    app.refresh().await;
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while app.is_loading { app.process_updates(&mut rx); tokio::time::sleep(Duration::from_millis(10)).await; }
    }).await;
    assert!(!app.pipelines.is_empty());

    // Pipeline Jobs
    app.view_mode = ViewMode::Detail;
    app.selected_pipeline = Some(app.pipelines[0].clone());
    app.selected_mr = None;
    app.selected_issue = None;
    Mock::given(path("/api/v4/projects/1/pipelines/1/jobs")).respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({
        "id":1,"name":"j","status":"success","stage":"build","web_url":"u","created_at":"2023-01-01T00:00:00Z",
        "started_at":null,"finished_at":null,"duration":null,"user":{"id":1,"username":"u"},"allow_failure":false,
        "tag_list":[], "runner": null
    })])).mount(&server).await;
    app.refresh().await;
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while app.is_loading { app.process_updates(&mut rx); tokio::time::sleep(Duration::from_millis(10)).await; }
    }).await;
    assert!(!app.jobs.is_empty());

    // Job Log
    app.view_mode = ViewMode::JobLog;
    app.selected_job = Some(app.jobs[0].clone());
    Mock::given(path("/api/v4/projects/1/jobs/1/trace")).respond_with(ResponseTemplate::new(200).set_body_string("logs")).mount(&server).await;
    app.refresh().await;
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while app.is_loading { app.process_updates(&mut rx); tokio::time::sleep(Duration::from_millis(10)).await; }
    }).await;
    assert_eq!(app.job_log, "logs");
}

#[tokio::test]
async fn test_help_actions_and_focus_logic() {
    let (_server, mut app, _rx) = setup_app().await;

    // Toggle Help
    assert!(!app.show_help_popup);
    app.toggle_help();
    assert!(app.show_help_popup);
    assert_eq!(app.help_state.selected(), Some(0));
    app.toggle_help();
    assert!(!app.show_help_popup);

    // Help Filter
    app.show_help_popup = true;
    app.help_filter_input = "mr".into();
    app.filter_help();
    assert!(app.filtered_help.iter().all(|h| h.shortcut.to_lowercase().contains("mr") || h.description.to_lowercase().contains("mr")));

    // Focus transitions
    app.input_target = lazygitlab::models::InputTarget::Filter;
    app.input_mode = true;
    app.back().await;
    assert!(!app.input_mode);
}

#[tokio::test]
async fn test_error_handling_exhaustive() {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
    let client = gitlab::GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    Mock::given(wiremock::matchers::any()).respond_with(ResponseTemplate::new(500)).mount(&server).await;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(client);
    app.tx = tx;

    app.selected_project = Some(Project { id: 1, name: "p".into(), path_with_namespace: Some("n/p".into()), web_url: Some("u".into()) });
    app.selected_mr = Some(MergeRequest { iid: 1, project_id: 1, title: "m".into(), description: None, author: lazygitlab::models::Author { id: 1, username: "u".into() }, web_url: "u".into(), state: "o".into(), references: lazygitlab::models::References { full: "r".into() }, upvotes: 0, downvotes: 0, labels: vec![], milestone: None, assignees: vec![], created_at: "t".into(), reactions: vec![] });
    app.selected_issue = Some(Issue { iid: 1, project_id: 1, title: "i".into(), description: None, author: lazygitlab::models::Author { id: 1, username: "u".into() }, assignees: vec![], state: "o".into(), web_url: "u".into(), references: lazygitlab::models::References { full: "r".into() }, issue_type: None, labels: vec![], milestone: None, due_date: None, weight: None, task_completion_status: None, confidential: false, created_at: "t".into(), upvotes: 0, downvotes: 0, reactions: vec![] });
    app.selected_pipeline = Some(Pipeline { id: 1, project_id: 1, status: "s".into(), source: "s".into(), ref_name: "r".into(), web_url: "u".into(), created_at: "t".into(), started_at: None, duration: None });
    app.selected_job = Some(Job { id: 1, name: "j".into(), status: "s".into(), stage: "s".into(), web_url: "u".into(), created_at: "t".into(), started_at: None, finished_at: None, duration: None, user: None, runner: None, allow_failure: false, tag_list: vec![] });

    let mut error_count = 0;

    macro_rules! run_fetch {
        ($call:expr) => {
            let _ = $call.await;
            let _ = tokio::time::timeout(std::time::Duration::from_secs(1), async {
                while app.is_loading { app.process_updates(&mut rx); tokio::time::sleep(std::time::Duration::from_millis(10)).await; }
            }).await;
            app.process_updates(&mut rx);
            if app.last_error.is_some() { error_count += 1; }
            app.last_error = None;
        };
    }

    run_fetch!(app.fetch_todos());
    run_fetch!(app.fetch_projects());
    run_fetch!(app.fetch_groups());
    run_fetch!(app.fetch_global_merge_requests());
    run_fetch!(app.fetch_global_issues());
    run_fetch!(app.fetch_merge_requests());
    run_fetch!(app.fetch_issues());
    run_fetch!(app.fetch_pipelines());
    run_fetch!(app.fetch_discussions(1, 1));
    run_fetch!(app.fetch_issue_discussions(1, 1));
    run_fetch!(app.fetch_jobs(1, 1));
    run_fetch!(app.fetch_job_log(1, 1));
    run_fetch!(app.fetch_mr_diffs(1, 1));
    run_fetch!(app.fetch_project_metadata());

    // We expect many errors
    assert!(error_count > 0);

    // Test action methods with 500 errors
    let _ = app.post_comment().await;
    let _ = app.post_reply().await;
    let _ = app.resolve_discussion().await;
    let _ = app.approve_mr().await;
    let _ = app.unapprove_mr().await;
    let _ = app.merge_mr().await;
    let _ = app.toggle_issue_state().await;
    let _ = app.toggle_mr_state().await;
    let _ = app.update_issue_weight().await;
    let _ = app.update_issue_due_date().await;
    let _ = app.clear_due_date().await;
    let _ = app.cancel_pipeline(1, 1).await;
    let _ = app.retry_pipeline(1, 1).await;
    let _ = app.play_job(1, 1).await;
    let _ = app.retry_job(1, 1).await;
    let _ = app.cancel_job(1, 1).await;
}
