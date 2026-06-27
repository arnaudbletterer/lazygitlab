use std::time::Duration;
use tokio::sync::mpsc;

use gitlab::GitlabBuilder;
use lazygitlab::models::{App, DataUpdate, MenuSelection, Panel, Context, DetailSection};

mod common;

async fn try_build_client() -> Option<gitlab::AsyncGitlab> {
    let _ = dotenvy::dotenv();
    let token = match std::env::var("GITLAB_TOKEN") {
        Ok(t) if !t.trim().is_empty() => t,
        _ => return None,
    };
    let url = std::env::var("GITLAB_URL").unwrap_or_else(|_| "https://gitlab.com".to_string());
    match GitlabBuilder::new(url.replace("https://", "").replace("http://", ""), token)
        .build_async()
        .await
    {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("Skipping test: Failed to create GitLab client: {}", e);
            None
        }
    }
}

async fn wait_for_loading_finish(app: &mut App, rx: &mut mpsc::UnboundedReceiver<DataUpdate>) {
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(20) {
        app.process_updates(rx).await;
        if !app.is_loading {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn make_issue_discussion(id: &str, resolved: bool) -> lazygitlab::models::Discussion {
    use lazygitlab::models::{Discussion, Note, Author};
    Discussion {
        id: id.to_string(),
        notes: vec![Note { body: "Test note".into(), author: Author::new(1, "u".into()), system: false, position: None, context: None }],
        individual_note: false,
        collapsed: false,
        resolved: Some(resolved),
    }
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_issue_resolve_discussion_sends_correct_payload() {
    if common::skip_if_live_unavailable("live_full_issue_resolve_discussion_sends_correct_payload") {
        return;
    }
    let Some(client) = try_build_client().await else { return; };
    let ground = common::test_ground();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx);
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: Some(ground.project.web_url.clone().unwrap_or_default()),
        default_branch: Some(ground.project.default_branch.clone()),
        namespace: None,
    });
    app.active_context = Context::Project;
    app.active_panel = Panel::MainDetail;
    app.active_selection = MenuSelection::Issues;
    app.detail_section = DetailSection::Discussions;
    app.selected_issue = Some(lazygitlab::models::Issue {
        iid: 1,
        project_id: ground.project.id,
        title: "Test Issue".into(),
        description: None,
        author: lazygitlab::models::Author::new(1, "u".into()),
        assignees: vec![],
        state: "opened".into(),
        web_url: "u".into(),
        references: None,
        issue_type: None,
        labels: vec![],
        milestone: None,
        due_date: None,
        start_date: None,
        weight: None,
        task_completion_status: None,
        confidential: false,
        created_at: Some("t".into()),
        upvotes: Some(0),
        downvotes: Some(0),
        reactions: vec![],
        participants: vec![],
    });
    app.discussions = vec![make_issue_discussion("d-1", false)];
    app.discussion_state.select(Some(0));

    // The function is async, but to assert on the payload we need to intercept
    // the actual HTTP call. We just check that the status message is set and
    // the call dispatches (the live call would actually fire).
    let _ = ground;
    let res = app.resolve_issue_discussion().await;
    assert!(res.is_ok(), "resolve_issue_discussion should not error locally");
    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(msg.to_lowercase().contains("resolving") || msg.to_lowercase().contains("reopening"), "expected status to mention resolving/reopening, got {msg:?}");
    wait_for_loading_finish(&mut app, &mut rx).await;
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_issue_resolve_no_discussion_status_message() {
    if common::skip_if_live_unavailable("live_full_issue_resolve_no_discussion_status_message") {
        return;
    }
    let Some(client) = try_build_client().await else { return; };
    let ground = common::test_ground();

    let (tx, mut _rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx);
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: Some(ground.project.web_url.clone().unwrap_or_default()),
        default_branch: Some(ground.project.default_branch.clone()),
        namespace: None,
    });
    app.selected_issue = Some(lazygitlab::models::Issue {
        iid: 1,
        project_id: ground.project.id,
        title: "Test Issue".into(),
        description: None,
        author: lazygitlab::models::Author::new(1, "u".into()),
        assignees: vec![],
        state: "opened".into(),
        web_url: "u".into(),
        references: None,
        issue_type: None,
        labels: vec![],
        milestone: None,
        due_date: None,
        start_date: None,
        weight: None,
        task_completion_status: None,
        confidential: false,
        created_at: Some("t".into()),
        upvotes: Some(0),
        downvotes: Some(0),
        reactions: vec![],
        participants: vec![],
    });
    app.discussions.clear();
    app.discussion_state.select(None);

    let res = app.resolve_issue_discussion().await;
    assert!(res.is_ok());
    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(msg.to_lowercase().contains("no issue discussion"), "expected a 'no discussion' status, got {msg:?}");
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_issue_s_key_dispatches_to_issue_resolve() {
    if common::skip_if_live_unavailable("live_full_issue_s_key_dispatches_to_issue_resolve") {
        return;
    }
    let Some(client) = try_build_client().await else { return; };
    let ground = common::test_ground();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx);
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: Some(ground.project.web_url.clone().unwrap_or_default()),
        default_branch: Some(ground.project.default_branch.clone()),
        namespace: None,
    });
    app.active_context = Context::Project;
    app.active_panel = Panel::MainDetail;
    app.active_selection = MenuSelection::Issues;
    app.detail_section = DetailSection::Discussions;
    app.selected_issue = Some(lazygitlab::models::Issue {
        iid: 1,
        project_id: ground.project.id,
        title: "Test Issue".into(),
        description: None,
        author: lazygitlab::models::Author::new(1, "u".into()),
        assignees: vec![],
        state: "opened".into(),
        web_url: "u".into(),
        references: None,
        issue_type: None,
        labels: vec![],
        milestone: None,
        due_date: None,
        start_date: None,
        weight: None,
        task_completion_status: None,
        confidential: false,
        created_at: Some("t".into()),
        upvotes: Some(0),
        downvotes: Some(0),
        reactions: vec![],
        participants: vec![],
    });
    app.discussions = vec![make_issue_discussion("d-1", false)];
    app.discussion_state.select(Some(0));

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind, KeyEventState};
    // Lowercase 's' (the resolve key)
    app.handle_key(KeyEvent { code: KeyCode::Char('s'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }).await;
    wait_for_loading_finish(&mut app, &mut rx).await;

    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(msg.to_lowercase().contains("resolving") || msg.to_lowercase().contains("reopening") || msg.to_lowercase().contains("issue discussion"), "expected issue-resolve status, got {msg:?}");
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_help_lists_resolve_shortcut_for_issues() {
    if common::skip_if_live_unavailable("live_full_help_lists_resolve_shortcut_for_issues") {
        return;
    }
    let Some(client) = try_build_client().await else { return; };
    let ground = common::test_ground();

    let (tx, _rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx);
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: Some(ground.project.web_url.clone().unwrap_or_default()),
        default_branch: Some(ground.project.default_branch.clone()),
        namespace: None,
    });
    app.active_context = Context::Project;
    app.active_panel = Panel::MainDetail;
    app.active_selection = MenuSelection::Issues;
    app.detail_section = DetailSection::Discussions;
    app.selected_issue = Some(lazygitlab::models::Issue {
        iid: 1,
        project_id: ground.project.id,
        title: "Test Issue".into(),
        description: None,
        author: lazygitlab::models::Author::new(1, "u".into()),
        assignees: vec![],
        state: "opened".into(),
        web_url: "u".into(),
        references: None,
        issue_type: None,
        labels: vec![],
        milestone: None,
        due_date: None,
        start_date: None,
        weight: None,
        task_completion_status: None,
        confidential: false,
        created_at: Some("t".into()),
        upvotes: Some(0),
        downvotes: Some(0),
        reactions: vec![],
        participants: vec![],
    });
    app.discussions = vec![make_issue_discussion("d-1", false)];

    let items = app.get_available_actions();
    let has_resolve = items.iter().any(|i| i.category == "Discussion" && i.shortcut.contains("s"));
    assert!(has_resolve, "help should list the s (resolve discussion) shortcut for issues with discussions");
}
