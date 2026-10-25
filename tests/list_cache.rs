use lazygitlab::cache::{
    CacheEntry, CachedIssue, CachedItem, CachedMr, CachedPipeline, CachedTodo, ListCache,
    MAX_AGE_SECS, MAX_FILE_BYTES, age_label, host_hash, query_key,
};
use lazygitlab::models::{Issue, MergeRequest, Pipeline, Todo};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const HOST: &str = "https://gitlab.secret-corp.example";
const TOKEN: &str = "glpat-SAMPLE-TOKEN-123";

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lgl-cache-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d.join("cache").join("v1")
}

fn uid(n: u64) -> Option<NonZeroU64> {
    NonZeroU64::new(n)
}

fn load(dir: &Path) -> ListCache {
    ListCache::load(dir, HOST, uid(7))
}

fn item(n: u64) -> CachedItem {
    CachedItem::Todo(CachedTodo {
        id: n,
        project: None,
        action_name: "assigned".into(),
        target_type: "MergeRequest".into(),
        target_title: "x".repeat(100),
        target_path: "/g/p/-/merge_requests/1".into(),
        state: "pending".into(),
        created_at: "2025-01-01T00:00:00Z".into(),
    })
}

fn entry(n: u64) -> CacheEntry {
    CacheEntry::new(
        vec![item(n)],
        Some("etag".into()),
        Some("2025-01-01T00:00:00Z".into()),
    )
}

fn cache_file(dir: &Path) -> PathBuf {
    load(dir).path().unwrap()
}

#[test]
fn roundtrip() {
    let dir = tmp("rt");
    let c = load(&dir);
    let key = query_key("mr", "", "opened", "all");
    let e = entry(1);
    assert!(c.put(&key, e.clone()));
    c.flush().unwrap();
    let c2 = load(&dir);
    assert_eq!(c2.get(&key), Some(e));
    assert_eq!(c2.last_user_id(), uid(7));
    assert!(c2.get("other").is_none());
}

#[test]
fn last_user_id_mismatch_in_file_invalidates() {
    let dir = tmp("lastuser");
    let path = cache_file(&dir);
    let c = load(&dir);
    c.put("k", entry(1));
    c.flush().unwrap();
    let body = std::fs::read_to_string(&path)
        .unwrap()
        .replace("\"last_user_id\":7", "\"last_user_id\":9");
    std::fs::write(&path, body).unwrap();
    let c2 = load(&dir);
    assert!(c2.get("k").is_none());
    assert!(!path.exists());
}

#[test]
fn unknown_user_never_caches() {
    let dir = tmp("nouser");
    let c = ListCache::load(&dir, HOST, None);
    assert!(!c.put("k", entry(1)));
    assert!(c.get("k").is_none());
    assert!(c.path().is_none());
    c.flush().unwrap();
    assert!(!dir.exists() || std::fs::read_dir(&dir).unwrap().next().is_none());
}

#[cfg(unix)]
#[test]
fn modes_are_private() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tmp("mode");
    let c = load(&dir);
    c.put("k", entry(1));
    c.flush().unwrap();
    let f = std::fs::metadata(c.path().unwrap())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    let d = std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
    assert_eq!(f, 0o600);
    assert_eq!(d, 0o700);
}

#[test]
fn failure_between_write_and_rename_keeps_old_file() {
    let dir = tmp("atomic");
    let c = load(&dir);
    let path = c.path().unwrap();
    c.put("k", entry(1));
    c.flush().unwrap();
    let before = std::fs::read(&path).unwrap();
    c.put("k2", entry(2));
    let seen_old = std::cell::Cell::new(false);
    let res = c.flush_with_hook(&|| {
        seen_old.set(std::fs::read(&path).unwrap() == before);
        Err(std::io::Error::other("injected"))
    });
    assert!(res.is_err());
    assert!(seen_old.get(), "final path was touched before rename");
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let leftovers: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains(".tmp-"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
    c.flush().unwrap();
    assert!(load(&dir).get("k2").is_some());
}

#[test]
fn failed_rename_leaves_no_partial_file() {
    let dir = tmp("rename");
    let c = load(&dir);
    std::fs::create_dir_all(c.path().unwrap()).unwrap();
    c.put("k", entry(1));
    assert!(c.flush().is_err());
    let leftovers = std::fs::read_dir(&dir)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".tmp-")
        })
        .count();
    assert_eq!(leftovers, 0);
}

#[test]
fn schema_mismatch_resets() {
    let dir = tmp("schema");
    let path = cache_file(&dir);
    std::fs::write(&path, r#"{"schema":99,"last_user_id":7,"entries":{}}"#).unwrap();
    let c = load(&dir);
    assert!(c.get("k").is_none());
    assert!(!path.exists());
}

#[test]
fn corrupt_json_resets() {
    let dir = tmp("corrupt");
    let path = cache_file(&dir);
    std::fs::write(&path, "{not json").unwrap();
    let c = load(&dir);
    assert!(c.get("k").is_none());
    assert!(!path.exists());
}

#[test]
fn oversized_file_is_removed() {
    let dir = tmp("bigfile");
    let path = cache_file(&dir);
    std::fs::write(&path, vec![b' '; MAX_FILE_BYTES as usize + 1]).unwrap();
    let c = load(&dir);
    assert!(c.get("k").is_none());
    assert!(!path.exists());
}

#[test]
fn ttl_expiry_drops_old_entries() {
    let dir = tmp("ttl");
    let c = load(&dir);
    let mut old = entry(1);
    old.fetched_at -= MAX_AGE_SECS + 10;
    c.put("old", old);
    c.put("new", entry(2));
    c.flush().unwrap();
    let c2 = load(&dir);
    assert!(c2.get("old").is_none());
    assert!(c2.get("new").is_some());
}

#[test]
fn future_timestamp_is_clamped() {
    let dir = tmp("future");
    let c = load(&dir);
    let mut e = entry(1);
    e.fetched_at += 10_000_000;
    c.put("k", e);
    let got = c.get("k").unwrap();
    assert!(got.fetched_at <= entry(1).fetched_at + 5);
    c.flush().unwrap();
    let path = c.path().unwrap();
    let body = std::fs::read_to_string(&path)
        .unwrap()
        .replace(&got.fetched_at.to_string(), "99999999999");
    std::fs::write(&path, body).unwrap();
    let c2 = load(&dir);
    let loaded = c2.get("k").unwrap();
    assert!(loaded.fetched_at <= entry(1).fetched_at + 5);
}

#[test]
fn lru_eviction_keeps_newest_within_cap() {
    let dir = tmp("lru");
    let c = ListCache::load_with_max(&dir, HOST, uid(7), 1000);
    let base = entry(0).fetched_at;
    for i in 0..10u64 {
        let mut e = entry(i);
        e.fetched_at = base - 100 + i;
        c.put(&format!("k{i}"), e);
    }
    c.flush().unwrap();
    assert!(std::fs::metadata(c.path().unwrap()).unwrap().len() < 1500);
    let c2 = ListCache::load_with_max(&dir, HOST, uid(7), 1000);
    assert!(c2.get("k9").is_some());
    assert!(c2.get("k0").is_none());
}

#[test]
fn get_refreshes_recency() {
    let dir = tmp("touch");
    let one = serde_json::to_vec(&entry(0)).unwrap().len();
    let c = ListCache::load_with_max(&dir, HOST, uid(7), one * 2 + one / 2);
    c.put("a", entry(1));
    c.put("b", entry(2));
    assert!(c.get("a").is_some());
    c.put("c", entry(3));
    assert!(c.get("a").is_some());
    assert!(c.get("b").is_none());
}

#[test]
fn oversized_single_entry_is_rejected_and_keeps_others() {
    let dir = tmp("bigentry");
    let c = ListCache::load_with_max(&dir, HOST, uid(7), 1000);
    assert!(c.put("small", entry(1)));
    let big = CacheEntry::new((0..50).map(item).collect(), None, None);
    assert!(!c.put("big", big));
    assert!(c.get("big").is_none());
    assert!(c.get("small").is_some());
}

#[test]
fn user_change_invalidates() {
    let dir = tmp("user");
    let c = load(&dir);
    c.put("k", entry(1));
    c.flush().unwrap();
    let old = c.path().unwrap();
    c.invalidate_user(uid(8).unwrap());
    assert!(c.get("k").is_none());
    assert!(!old.exists());
    assert_eq!(c.last_user_id(), uid(8));
    assert!(load(&dir).get("k").is_none());
}

#[test]
fn user_becoming_known_enables_caching() {
    let dir = tmp("becomes");
    let c = ListCache::load(&dir, HOST, None);
    c.invalidate_user(uid(5).unwrap());
    assert!(c.put("k", entry(1)));
    c.flush().unwrap();
    assert!(ListCache::load(&dir, HOST, uid(5)).get("k").is_some());
}

const MR_JSON: &str = r##"{
 "id": 111, "iid": 12, "project_id": 3, "title": "Fix login", "description": "see https://gitlab.distinct-host.example/x",
 "state": "opened", "draft": false,
 "created_at": "2025-01-01T00:00:00Z", "updated_at": "2025-01-02T00:00:00Z",
 "web_url": "https://gitlab.distinct-host.example/grp/proj/-/merge_requests/12",
 "author": {"id": 1, "username": "jdoe", "name": "Jane Doe <jane.doe@corp-mail.example>", "state": "active",
   "avatar_url": "https://gitlab.distinct-host.example/uploads/-/system/user/avatar/1/a.png",
   "web_url": "https://gitlab.distinct-host.example/jdoe", "email": "jane.doe@corp-mail.example"},
 "assignees": [{"id": 2, "username": "bob", "name": "Bob", "avatar_url": "https://secure.gravatar.com/avatar/abc", "web_url": "https://gitlab.distinct-host.example/bob"}],
 "reviewers": [{"id": 4, "username": "eve", "name": "Eve", "avatar_url": "https://gitlab.distinct-host.example/eve.png", "web_url": "https://gitlab.distinct-host.example/eve"}],
 "labels": [{"id": 9, "name": "bug", "color": "#ff0000", "text_color": "#ffffff", "description": "d"}],
 "milestone": {"id": 5, "title": "v1", "web_url": "https://gitlab.distinct-host.example/m"},
 "references": {"short": "!12", "relative": "!12", "full": "grp/proj!12"},
 "source_branch": "fix", "target_branch": "main",
 "head_pipeline": {"id": 77, "status": "success", "web_url": "https://gitlab.distinct-host.example/p/77"},
 "diff_refs": {"base_sha": "a", "head_sha": "b", "start_sha": "c"}
}"##;

const ISSUE_JSON: &str = r##"{
 "iid": 4, "project_id": 3, "title": "Crash", "description": "d", "state": "opened",
 "web_url": "https://gitlab.distinct-host.example/grp/proj/-/issues/4",
 "author": {"id": 1, "username": "jdoe", "name": "Jane <jane.doe@corp-mail.example>", "avatar_url": "https://gitlab.distinct-host.example/a.png", "web_url": "https://gitlab.distinct-host.example/jdoe"},
 "assignees": [{"id": 2, "username": "bob", "avatar_url": "https://gitlab.distinct-host.example/b.png", "web_url": "https://gitlab.distinct-host.example/bob"}],
 "labels": ["bug"], "references": {"full": "grp/proj#4"}, "due_date": "2025-03-01",
 "created_at": "2025-01-01T00:00:00Z", "confidential": false
}"##;

const TODO_JSON: &str = r##"{
 "id": 8, "action_name": "mentioned", "target_type": "MergeRequest", "state": "pending",
 "target_url": "https://gitlab.distinct-host.example/grp/proj/-/merge_requests/12#note_5",
 "body": "Fix login", "created_at": "2025-01-01T00:00:00Z",
 "project": {"id": 3, "name": "proj", "path_with_namespace": "grp/proj", "web_url": "https://gitlab.distinct-host.example/grp/proj"},
 "author": {"id": 1, "username": "jdoe", "avatar_url": "https://gitlab.distinct-host.example/a.png"}
}"##;

const PIPELINE_JSON: &str = r##"{
 "id": 77, "project_id": 3, "status": "success", "source": "push", "ref": "main",
 "web_url": "https://gitlab.distinct-host.example/grp/proj/-/pipelines/77",
 "created_at": "2025-01-01T00:00:00Z", "started_at": null, "duration": 12.5
}"##;

#[test]
fn realistic_payloads_leak_no_host_or_personal_data() {
    let mr: MergeRequest = serde_json::from_str(MR_JSON).unwrap();
    let issue: Issue = serde_json::from_str(ISSUE_JSON).unwrap();
    let todo: Todo = serde_json::from_str(TODO_JSON).unwrap();
    let pipeline: Pipeline = serde_json::from_str(PIPELINE_JSON).unwrap();
    let host = "https://gitlab.distinct-host.example";

    let dir = tmp("payload");
    let c = load(&dir);
    c.put(
        "all",
        CacheEntry::new(
            vec![
                CachedItem::Mr(CachedMr::from(&mr)),
                CachedItem::Issue(CachedIssue::from(&issue)),
                CachedItem::Todo(CachedTodo::from(&todo)),
                CachedItem::Pipeline(CachedPipeline::from(&pipeline)),
            ],
            None,
            None,
        ),
    );
    c.flush().unwrap();
    let body = std::fs::read_to_string(c.path().unwrap()).unwrap();
    for needle in [
        "distinct-host",
        "avatar",
        "gravatar",
        "corp-mail",
        "jane.doe",
        "Jane",
        "web_url",
        "http://",
        "https://",
        "sha",
        "description",
    ] {
        assert!(!body.contains(needle), "file leaks {needle}: {body}");
    }

    let rows = load(&dir).get("all").unwrap().items;
    match (&rows[0], &rows[1], &rows[2], &rows[3]) {
        (CachedItem::Mr(m), CachedItem::Issue(i), CachedItem::Todo(t), CachedItem::Pipeline(p)) => {
            let m = m.to_model(host);
            assert_eq!(m.web_url, mr.web_url);
            assert_eq!(m.references, mr.references);
            assert_eq!(m.author.username, "jdoe");
            assert_eq!(m.labels[0].name, "bug");
            assert_eq!(m.head_pipeline, mr.head_pipeline);
            assert_eq!(i.to_model(host).web_url, issue.web_url);
            assert_eq!(i.to_model(host).due_date, issue.due_date);
            let t = t.to_model(host);
            assert_eq!(t.target_url, todo.target_url);
            assert_eq!(t.body, todo.body);
            assert_eq!(
                t.project.unwrap().web_url.as_deref(),
                Some("https://gitlab.distinct-host.example/grp/proj")
            );
            assert_eq!(p.to_model(host), pipeline);
        }
        other => panic!("unexpected rows {other:?}"),
    }
}

#[test]
fn file_has_no_token_or_host() {
    let dir = tmp("secret");
    let c = load(&dir);
    c.put("k", entry(1));
    c.flush().unwrap();
    let path = c.path().unwrap();
    let body = std::fs::read_to_string(&path).unwrap();
    assert!(!body.contains(TOKEN));
    assert!(!body.contains("secret-corp"));
    let name = path.file_name().unwrap().to_string_lossy().into_owned();
    assert!(!name.contains("secret-corp"));
    assert!(name.starts_with(&host_hash(HOST)));
    assert_eq!(host_hash(HOST).len(), 16);
}

#[cfg(unix)]
#[test]
fn symlink_at_cache_path_is_refused() {
    let dir = tmp("symlink");
    let path = cache_file(&dir);
    let target = dir.join("target.json");
    std::fs::write(&target, "precious").unwrap();
    std::os::unix::fs::symlink(&target, &path).unwrap();
    let c = load(&dir);
    assert!(!c.put("k", entry(1)));
    let _ = c.flush();
    assert!(
        std::fs::symlink_metadata(&path)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "precious");
}

#[test]
fn concurrent_puts_do_not_corrupt() {
    let dir = tmp("conc");
    let c = Arc::new(load(&dir));
    let handles: Vec<_> = (0..8)
        .map(|t| {
            let c = c.clone();
            std::thread::spawn(move || {
                for i in 0..20 {
                    c.put(&format!("k{t}-{i}"), entry(i));
                    c.flush().unwrap();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    c.flush().unwrap();
    let c2 = load(&dir);
    for t in 0..8 {
        for i in 0..20 {
            assert!(c2.get(&format!("k{t}-{i}")).is_some());
        }
    }
}

#[test]
fn child_writer() {
    let Ok(dir) = std::env::var("LGL_CHILD_DIR") else {
        return;
    };
    let tag = std::env::var("LGL_CHILD_TAG").unwrap();
    let c = load(Path::new(&dir));
    for i in 0..60 {
        c.put(&format!("{tag}-{i}"), entry(i));
        c.flush().unwrap();
    }
}

#[test]
fn two_processes_never_leave_an_unparseable_file() {
    let dir = tmp("procs");
    let path = cache_file(&dir);
    let exe = std::env::current_exe().unwrap();
    let spawn = |tag: &str| {
        std::process::Command::new(&exe)
            .args(["--exact", "child_writer", "--nocapture"])
            .env("LGL_CHILD_DIR", &dir)
            .env("LGL_CHILD_TAG", tag)
            .spawn()
            .unwrap()
    };
    let mut a = spawn("a");
    let mut b = spawn("b");
    let mut reads = 0;
    loop {
        if let Ok(bytes) = std::fs::read(&path) {
            serde_json::from_slice::<serde_json::Value>(&bytes).expect("torn file");
            reads += 1;
        }
        let done = a.try_wait().unwrap().is_some() && b.try_wait().unwrap().is_some();
        if done {
            break;
        }
    }
    assert!(a.wait().unwrap().success() && b.wait().unwrap().success());
    assert!(reads > 0);
    assert!(load(&dir).get("a-59").is_some() || load(&dir).get("b-59").is_some());
}

#[test]
fn age_labels() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert_eq!(age_label(now - 180), "cached 3 min ago");
    assert_eq!(age_label(now), "cached just now");
}
