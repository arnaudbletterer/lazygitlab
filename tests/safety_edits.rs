use gitlab::GitlabBuilder;
use lazygitlab::models::{App, Author, DataUpdate, Label, MergeRequest};
use std::collections::BTreeSet;
use tokio::sync::mpsc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Rx = mpsc::UnboundedReceiver<DataUpdate>;

async fn setup_app() -> (MockServer, App, Rx) {
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
    let (tx, rx) = mpsc::unbounded_channel();
    let app = App::new(client, tx, None);
    (server, app, rx)
}

fn label(id: u64, name: &str) -> Label {
    Label {
        id,
        name: name.into(),
        color: "#112233".into(),
        text_color: None,
    }
}

fn dummy_mr(iid: u64, title: &str) -> MergeRequest {
    MergeRequest {
        updated_at: None,
        iid,
        project_id: 1,
        title: title.into(),
        description: None,
        author: Author::new(1, "u".into()),
        web_url: "u".into(),
        state: "opened".into(),
        references: None,
        upvotes: Some(0),
        downvotes: Some(0),
        labels: vec![],
        milestone: None,
        assignees: vec![],
        reviewers: vec![],
        squash: Some(false),
        created_at: Some("t".into()),
        draft: Some(false),
        diff_refs: None,
        reactions: vec![],
        merge_status: None,
        detailed_merge_status: None,
        has_conflicts: None,
        head_pipeline: None,
        source_branch: "source".into(),
        target_branch: "target".into(),
        participants: vec![],
    }
}

fn split(v: &str) -> Vec<String> {
    v.split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

fn resulting_labels(body: &[u8], current: &[&str]) -> BTreeSet<String> {
    let pairs: Vec<(String, String)> = url::form_urlencoded::parse(body).into_owned().collect();
    let get = |k: &str| {
        pairs
            .iter()
            .find(|(key, _)| key == k)
            .map(|(_, v)| v.clone())
    };
    let mut set: BTreeSet<String> = match get("labels") {
        Some(l) => split(&l).into_iter().collect(),
        None => current.iter().map(|s| s.to_string()).collect(),
    };
    if let Some(a) = get("add_labels") {
        set.extend(split(&a));
    }
    if let Some(r) = get("remove_labels") {
        for l in split(&r) {
            set.remove(&l);
        }
    }
    set
}

async fn last_put_body(server: &MockServer) -> Vec<u8> {
    server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .rev()
        .find(|r| r.method.as_str() == "PUT")
        .expect("a PUT request")
        .body
}

#[tokio::test]
async fn label_toggle_keeps_labels_missing_from_popup_list() {
    let (server, mut app, _rx) = setup_app().await;
    Mock::given(method("PUT"))
        .and(path("/api/v4/projects/1/merge_requests/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    let mut mr = dummy_mr(1, "t");
    mr.labels = vec![label(0, "bug"), label(0, "ui")];
    app.merge_requests = vec![mr.clone()];
    app.selected_mr = Some(mr);
    app.project_label_cache
        .insert(1, vec![label(7, "bug"), label(8, "feature")]);
    app.selected_label_ids = vec![0, 0, 8];
    app.update_mr_labels().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let body = last_put_body(&server).await;
    let want: BTreeSet<String> = ["bug", "ui", "feature"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(resulting_labels(&body, &["bug", "ui"]), want);
}
