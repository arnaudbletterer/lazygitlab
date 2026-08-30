use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use gitlab::GitlabBuilder;
use lazygitlab::models::*;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

async fn setup_app() -> App {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::path("/api/v4/user"))
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
    App::new(client, tx, None)
}

fn pipe(id: u64, status: &str) -> Pipeline {
    Pipeline {
        id,
        project_id: 1,
        status: status.into(),
        source: "push".into(),
        ref_name: "main".into(),
        web_url: "u".into(),
        created_at: "t".into(),
        started_at: None,
        duration: None,
    }
}

#[tokio::test]
async fn test_pipeline_status_segments() {
    let mut app = setup_app().await;
    app.active_context = Context::Project;
    app.pipelines = vec![
        pipe(1, "success"),
        pipe(2, "failed"),
        pipe(3, "running"),
        pipe(4, "failed"),
    ];

    app.pipeline_filter = PipelineFilter::Failed;
    app.filter_lists();
    assert_eq!(
        app.filtered_pipelines
            .iter()
            .map(|p| p.id)
            .collect::<Vec<_>>(),
        vec![2, 4],
        "failed segment"
    );

    app.pipeline_filter = PipelineFilter::Running;
    app.filter_lists();
    assert_eq!(
        app.filtered_pipelines
            .iter()
            .map(|p| p.id)
            .collect::<Vec<_>>(),
        vec![3],
        "running segment includes pending-like states"
    );

    app.pipeline_filter = PipelineFilter::Passed;
    app.filter_lists();
    assert_eq!(
        app.filtered_pipelines
            .iter()
            .map(|p| p.id)
            .collect::<Vec<_>>(),
        vec![1],
        "passed segment"
    );

    app.pipeline_filter = PipelineFilter::All;
    app.filter_lists();
    assert_eq!(app.filtered_pipelines.len(), 4);
}

#[tokio::test]
async fn test_bracket_cycles_pipeline_filter_without_refetch() {
    let mut app = setup_app().await;
    app.active_context = Context::Project;
    app.active_panel = Panel::Main;
    app.active_selection = MenuSelection::Pipelines;
    app.pipeline_filter = PipelineFilter::All;

    app.handle_key(key(KeyCode::Char(']'))).await;
    assert_eq!(app.pipeline_filter, PipelineFilter::Running);

    app.handle_key(key(KeyCode::Char('['))).await;
    assert_eq!(app.pipeline_filter, PipelineFilter::All);
}
