use lazygitlab::models::*;
use lazygitlab::ui::ui;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use gitlab::GitlabBuilder;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::path;

async fn setup_app() -> (MockServer, App) {
    let server = MockServer::start().await;
    Mock::given(path("/api/v4/user")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":1,"username":"u"}))).mount(&server).await;
    let client = GitlabBuilder::new(server.uri().replace("http://",""), "t").insecure().build_async().await.unwrap();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let app = App::new(client, tx);
    (server, app)
}

#[tokio::test]
async fn test_ui_all_popups_exhaustive() {
    let (_server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    // 1. Label Popup
    app.show_label_popup = true;
    app.project_labels = vec![Label { id: 1, name: "bug".into(), color: "#ff0000".into(), text_color: None }];
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.show_label_popup = false;

    // 2. Assignee Popup
    app.show_assignee_popup = true;
    app.project_members = vec![Author::new(1, "u".into())];
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.show_assignee_popup = false;

    // 3. Milestone Popup
    app.show_milestone_popup = true;
    app.project_milestones = vec![Milestone { id: 1, title: "v1".into() }];
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.show_milestone_popup = false;

    // 4. Weight Popup
    app.show_weight_popup = true;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.show_weight_popup = false;

    // 5. Calendar Popup
    app.show_calendar_popup = true;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.show_calendar_popup = false;

    // 6. Pipeline Popup
    app.show_pipeline_popup = true;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.show_pipeline_popup = false;

    // 7. Create Issue Popup
    app.show_create_issue_popup = true;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.show_create_issue_popup = false;

    // 8. Create MR Popup
    app.show_create_mr_popup = true;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.show_create_mr_popup = false;

    // 9. Help Popup
    app.show_help_popup = true;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.show_help_popup = false;

    // 10. Filter Input
    app.show_filter_input = true;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.show_filter_input = false;

    // 11. Snooze Popup
    app.show_snooze_popup = true;
    terminal.draw(|f| ui(f, &mut app)).unwrap();
    app.show_snooze_popup = false;
}

#[tokio::test]
async fn test_ui_all_view_modes() {
    let (_server, mut app) = setup_app().await;
    let backend = TestBackend::new(150, 50);
    let mut terminal = Terminal::new(backend).unwrap();

    // List Mode (already tested extensively)
    app.view_mode = ViewMode::List;
    terminal.draw(|f| ui(f, &mut app)).unwrap();

    // Detail Mode
    app.view_mode = ViewMode::Detail;
    app.selected_mr = Some(MergeRequest { iid: 1, project_id: 1, title: "T".into(), description: Some("D".into()), author: Author::new(1, "u".into()), web_url: "u".into(), state: "opened".into(), references: None, upvotes: None, downvotes: None, labels: vec![], milestone: None, assignees: vec![], reviewers: vec![], squash: None, created_at: None, draft: None, diff_refs: None, reactions: vec![], merge_status: None, detailed_merge_status: None, has_conflicts: None, head_pipeline: None, source_branch: "".into(), target_branch: "".into(), participants: vec![] });
    terminal.draw(|f| ui(f, &mut app)).unwrap();

    // JobLog Mode
    app.view_mode = ViewMode::JobLog;
    app.selected_job = Some(Job { id: 1, name: "j".into(), stage: "s".into(), status: "s".into(), created_at: "t".into(), started_at: None, finished_at: None, duration: None, user: None, runner: None, allow_failure: false, web_url: "u".into(), tag_list: vec![] });
    app.job_log = "log".into();
    terminal.draw(|f| ui(f, &mut app)).unwrap();

    // Diff Mode
    app.view_mode = ViewMode::Diff;
    app.selected_mr = Some(MergeRequest { iid: 1, project_id: 1, title: "T".into(), description: Some("D".into()), author: Author::new(1, "u".into()), web_url: "u".into(), state: "opened".into(), references: None, upvotes: None, downvotes: None, labels: vec![], milestone: None, assignees: vec![], reviewers: vec![], squash: None, created_at: None, draft: None, diff_refs: None, reactions: vec![], merge_status: None, detailed_merge_status: None, has_conflicts: None, head_pipeline: None, source_branch: "".into(), target_branch: "".into(), participants: vec![] });
    app.mr_diffs = vec![MRDiff { old_path: "a".into(), new_path: "b".into(), diff: "d".into(), collapsed: false, display_lines: vec![DiffDisplayLine::Code("d".into(), Some(1), Some(1))] }];
    terminal.draw(|f| ui(f, &mut app)).unwrap();
}
