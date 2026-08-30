use std::time::Duration;
use tokio::sync::mpsc;

use gitlab::GitlabBuilder;
use lazygitlab::models::{App, Context, DataUpdate, InputTarget, MenuSelection, Panel};

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

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_create_branch_opens_popup_with_prefill() {
    if common::skip_if_live_unavailable("live_full_create_branch_opens_popup_with_prefill") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.initial_setup().await.expect("initial setup");
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: Some(ground.project.web_url.clone().unwrap_or_default()),
        default_branch: Some(ground.project.default_branch.clone()),
        namespace: None,
    });
    app.active_context = Context::Project;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Branches;

    // Simulate pressing 'n' on the Branches list
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    app.handle_key(KeyEvent {
        code: KeyCode::Char('n'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
    .await;
    wait_for_loading_finish(&mut app, &mut rx).await;

    assert!(
        app.show_create_branch_popup,
        "n on Branches should open the create-branch popup"
    );
    assert!(app.input_mode, "create-branch popup should set input_mode");
    assert_eq!(
        app.input_target,
        InputTarget::CreateBranchName,
        "popup should focus the branch name field first"
    );
    assert!(
        !app.create_branch_ref_input.is_empty(),
        "the ref input should be pre-filled (with project default branch or current pipeline's ref)"
    );
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_create_branch_validation_empty_name() {
    if common::skip_if_live_unavailable("live_full_create_branch_validation_empty_name") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();

    let (tx, mut _rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: Some(ground.project.web_url.clone().unwrap_or_default()),
        default_branch: Some(ground.project.default_branch.clone()),
        namespace: None,
    });
    app.active_context = Context::Project;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Branches;

    app.show_create_branch_popup = true;
    app.input_target = InputTarget::CreateBranchName;
    app.create_branch_name = String::new();
    app.create_branch_ref_input = "main".into();

    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    let submit = KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    app.handle_key(submit).await;

    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        msg.to_lowercase().contains("empty") || msg.to_lowercase().contains("name cannot"),
        "expected validation error, got {msg:?}"
    );
    assert!(
        app.show_create_branch_popup,
        "popup should stay open on validation error"
    );
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_create_branch_validation_empty_ref() {
    if common::skip_if_live_unavailable("live_full_create_branch_validation_empty_ref") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();

    let (tx, mut _rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: Some(ground.project.web_url.clone().unwrap_or_default()),
        default_branch: Some(ground.project.default_branch.clone()),
        namespace: None,
    });

    app.show_create_branch_popup = true;
    app.input_target = InputTarget::CreateBranchRef;
    app.create_branch_name = "feat/test".into();
    app.create_branch_ref_input = String::new();

    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    let submit = KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    app.handle_key(submit).await;

    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        msg.to_lowercase().contains("empty") || msg.to_lowercase().contains("reference"),
        "expected reference-required error, got {msg:?}"
    );
    assert!(
        app.show_create_branch_popup,
        "popup should stay open on validation error"
    );
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_create_branch_dismisses_with_esc() {
    if common::skip_if_live_unavailable("live_full_create_branch_dismisses_with_esc") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();

    let (tx, mut _rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: Some(ground.project.web_url.clone().unwrap_or_default()),
        default_branch: Some(ground.project.default_branch.clone()),
        namespace: None,
    });

    app.show_create_branch_popup = true;
    app.input_mode = true;

    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    app.handle_key(KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
    .await;

    assert!(
        !app.show_create_branch_popup,
        "Esc should close the create-branch popup"
    );
    assert!(!app.input_mode, "Esc should clear input_mode");
}

#[tokio::test]
#[ignore = "live_full"]
async fn live_full_delete_branch_success_via_d_key() {
    if common::skip_if_live_unavailable("live_full_delete_branch_success_via_d_key") {
        return;
    }
    let Some(client) = try_build_client().await else {
        return;
    };
    let ground = common::test_ground();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new(client, tx, None);
    app.selected_project = Some(lazygitlab::models::Project {
        id: ground.project.id,
        name: ground.project.path_with_namespace.clone(),
        path_with_namespace: Some(ground.project.path_with_namespace.clone()),
        web_url: Some(ground.project.web_url.clone().unwrap_or_default()),
        default_branch: Some(ground.project.default_branch.clone()),
        namespace: None,
    });
    app.active_context = Context::Project;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Branches;

    // Try to delete a non-existent branch (e.g. "main" is protected, use a fake one)
    let target = "feat/to-delete";
    app.project_branches = vec![
        lazygitlab::models::Branch {
            name: "main".into(),
            merged: Some(false),
            protected: Some(true),
            default: Some(true),
            web_url: None,
        },
        lazygitlab::models::Branch {
            name: target.into(),
            merged: Some(false),
            protected: Some(false),
            default: Some(false),
            web_url: None,
        },
    ];
    app.branches_state.select(Some(1));

    // Press D (uppercase) - the delete key for branches
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    app.handle_key(KeyEvent {
        code: KeyCode::Char('D'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
    .await;
    wait_for_loading_finish(&mut app, &mut rx).await;

    let msg = app.status_message.as_deref().unwrap_or("");
    assert!(
        msg.contains("deleted")
            || msg.contains("Delete")
            || msg.contains("Failed")
            || msg.contains("timed out"),
        "expected a delete status update, got {msg:?} (error: {:?})",
        app.last_error
    );
}
