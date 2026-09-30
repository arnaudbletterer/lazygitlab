use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

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

fn make_log_line(content: &str) -> LogLine {
    LogLine {
        timestamp: "t".into(),
        content: content.into(),
        section: None,
        is_section_start: false,
        is_section_end: false,
    }
}

#[tokio::test]
async fn test_file_scroll_and_job_log_scroll_are_independent() {
    let (_server, mut app) = setup_app().await;
    app.active_context = Context::Project;
    app.selected_project = Some(Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    });
    app.selected_pipeline = Some(Pipeline {
        id: 100,
        project_id: 1,
        status: "running".into(),
        source: "trigger".into(),
        ref_name: "main".into(),
        web_url: "u".into(),
        created_at: "t".into(),
        started_at: None,
        duration: None,
    });
    app.selected_job = Some(Job {
        id: 1,
        name: "build".into(),
        stage: "build".into(),
        status: "running".into(),
        created_at: "t".into(),
        started_at: None,
        finished_at: None,
        duration: None,
        user: None,
        runner: None,
        allow_failure: false,
        web_url: "u".into(),
        tag_list: vec![],
    });
    app.job_log_lines = vec![make_log_line("a"), make_log_line("b"), make_log_line("c")];
    app.view_mode = ViewMode::JobLog;

    // Scroll the log
    app.job_log_scroll = 5;
    app.job_log_viewport_height = 10;

    // Now switch to the file viewer (Tree) mode
    app.view_mode = ViewMode::Tree;
    app.selected_file_content = Some(RepositoryFile {
        file_name: "test.txt".into(),
        file_path: "test.txt".into(),
        size: 100,
        encoding: "base64".into(),
        content_sha256: "abc".into(),
        ref_name: "main".into(),
        blob_id: "blob1".into(),
        commit_id: "c1".into(),
        last_commit_id: "c1".into(),
        content: "aGVsbG8K".into(),
    });
    app.file_scroll = 0;
    app.file_viewport_height = 0;

    // The file viewer should have its own scroll, not the log's scroll
    assert_eq!(
        app.file_scroll, 0,
        "file_scroll should start at 0, not inherit job_log_scroll"
    );
    assert_eq!(
        app.job_log_scroll, 5,
        "job_log_scroll should be preserved when switching views"
    );
}

#[tokio::test]
async fn test_file_viewer_scroll_handler_uses_file_scroll() {
    let (_server, mut app) = setup_app().await;
    app.view_mode = ViewMode::Tree;
    app.selected_file_content = Some(RepositoryFile {
        file_name: "f.txt".into(),
        file_path: "f.txt".into(),
        size: 1000,
        encoding: "base64".into(),
        content_sha256: "abc".into(),
        ref_name: "main".into(),
        blob_id: "b".into(),
        commit_id: "c".into(),
        last_commit_id: "c".into(),
        content: "bGluZSAwCmxpbmUgMQpsaW5lIDIKbGluZSAzCmxpbmUgNApsaW5lIDUKbGluZSA2Cg==".into(),
    });
    app.file_scroll = 0;
    app.file_viewport_height = 5;

    // Press j to scroll down
    app.handle_key(key(KeyCode::Char('j'))).await;
    assert_eq!(app.file_scroll, 1, "j should scroll file by 1");
    assert_eq!(
        app.job_log_scroll, 0,
        "j in Tree should NOT touch job_log_scroll"
    );

    // Press k to scroll up
    app.handle_key(key(KeyCode::Char('k'))).await;
    assert_eq!(app.file_scroll, 0, "k should scroll file back to 0");
    assert_eq!(
        app.job_log_scroll, 0,
        "k in Tree should NOT touch job_log_scroll"
    );
}

#[tokio::test]
async fn test_file_viewer_scroll_does_not_leak_to_job_log() {
    let (_server, mut app) = setup_app().await;
    app.active_context = Context::Project;
    app.selected_project = Some(Project {
        id: 1,
        name: "p".into(),
        web_url: Some("u".into()),
        path_with_namespace: Some("p".into()),
        default_branch: Some("main".into()),
        namespace: None,
    });
    app.selected_pipeline = Some(Pipeline {
        id: 100,
        project_id: 1,
        status: "running".into(),
        source: "trigger".into(),
        ref_name: "main".into(),
        web_url: "u".into(),
        created_at: "t".into(),
        started_at: None,
        duration: None,
    });
    app.selected_job = Some(Job {
        id: 1,
        name: "build".into(),
        stage: "build".into(),
        status: "running".into(),
        created_at: "t".into(),
        started_at: None,
        finished_at: None,
        duration: None,
        user: None,
        runner: None,
        allow_failure: false,
        web_url: "u".into(),
        tag_list: vec![],
    });
    app.job_log_lines = vec![make_log_line("a"), make_log_line("b"), make_log_line("c")];
    app.job_log_viewport_height = 10;
    app.view_mode = ViewMode::JobLog;

    // Scroll the log to a non-zero value
    app.handle_key(key(KeyCode::Char('j'))).await;
    app.handle_key(key(KeyCode::Char('j'))).await;
    let log_scroll_after = app.job_log_scroll;
    assert_eq!(
        log_scroll_after, 0,
        "j in JobLog should not scroll the log (it uses cursor, not scroll)"
    );

    // Press j in the file viewer
    app.view_mode = ViewMode::Tree;
    app.selected_file_content = Some(RepositoryFile {
        file_name: "f.txt".into(),
        file_path: "f.txt".into(),
        size: 1000,
        encoding: "base64".into(),
        content_sha256: "abc".into(),
        ref_name: "main".into(),
        blob_id: "b".into(),
        commit_id: "c".into(),
        last_commit_id: "c".into(),
        content: "bGluZSAwCmxpbmUgMQpsaW5lIDIKbGluZSAzCmxpbmUgNApsaW5lIDUKbGluZSA2Cg==".into(),
    });
    app.file_scroll = 0;
    app.file_viewport_height = 3;

    // Press j 5 times in the file viewer (file_viewport_height=3, 7 lines, so max_scroll=4)
    for _ in 0..5 {
        app.handle_key(key(KeyCode::Char('j'))).await;
    }

    assert_eq!(
        app.file_scroll, 4,
        "file_scroll should clamp to max_scroll (line_count - viewport_height = 7 - 3 = 4)"
    );
    assert_eq!(
        app.job_log_scroll, log_scroll_after,
        "file scroll should not affect log scroll"
    );
}
