use gitlab::GitlabBuilder;
use lazygitlab::models::*;
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

#[tokio::test]
async fn too_small_terminal_shows_message() {
    let (_server, mut app) = setup_app().await;
    let mut terminal = Terminal::new(TestBackend::new(30, 8)).unwrap();
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    let text: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();
    assert!(text.contains("Terminal"), "{text}");
}

#[tokio::test]
async fn tiny_terminals_never_panic() {
    let (_server, mut app) = setup_app().await;
    app.last_error = Some("x".repeat(500));
    let popups: Vec<fn(&mut App, bool)> = vec![
        |a, on| a.show_project_popup = on,
        |a, on| a.show_schedules_popup = on,
        |a, on| a.show_ci_vars_popup = on,
        |a, on| a.show_label_popup = on,
        |a, on| a.show_assignee_popup = on,
        |a, on| a.show_reviewer_popup = on,
        |a, on| a.show_milestone_popup = on,
        |a, on| a.show_weight_popup = on,
        |a, on| a.show_calendar_popup = on,
        |a, on| a.show_pipeline_popup = on,
        |a, on| a.show_target_branch_popup = on,
        |a, on| a.show_create_branch_popup = on,
        |a, on| a.show_help_popup = on,
        |a, on| a.show_create_issue_popup = on,
        |a, on| a.show_confirm_merge_popup = on,
        |a, on| a.show_confirm_rebase_popup = on,
        |a, on| a.show_confirm_delete_branch_popup = on,
        |a, on| a.show_create_mr_popup = on,
        |a, on| a.show_snooze_popup = on,
        |a, on| a.show_tree_branch_popup = on,
    ];
    let modes = [
        ViewMode::List,
        ViewMode::Detail,
        ViewMode::JobLog,
        ViewMode::Diff,
        ViewMode::Tree,
        ViewMode::Board,
        ViewMode::Epics,
    ];
    for (w, h) in [
        (0, 0),
        (1, 1),
        (3, 2),
        (10, 3),
        (20, 5),
        (30, 8),
        (40, 10),
        (49, 40),
        (100, 13),
        (50, 14),
        (60, 16),
        (80, 24),
    ] {
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        for mode in modes {
            app.view_mode = mode;
            terminal.draw(|f| ui(f, &mut app)).unwrap();
            for (i, set) in popups.iter().enumerate() {
                eprintln!("size {w}x{h} mode {mode:?} popup {i}");
                set(&mut app, true);
                terminal.draw(|f| ui(f, &mut app)).unwrap();
                set(&mut app, false);
            }
        }
    }
}
