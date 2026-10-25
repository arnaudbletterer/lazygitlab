use lazygitlab::cache::{CacheEntry, ListCache, MAX_AGE_SECS, age_label, host_hash, query_key};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

const HOST: &str = "https://gitlab.secret-corp.example";
const TOKEN: &str = "glpat-SAMPLE-TOKEN-123";

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lgl-cache-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d.join("cache").join("v1")
}

fn entry(n: usize) -> CacheEntry {
    CacheEntry::new(
        vec![json!({"iid": n, "title": "x".repeat(100)})],
        Some("etag".into()),
        Some("2025-01-01T00:00:00Z".into()),
    )
}

fn cache_file(dir: &std::path::Path) -> PathBuf {
    ListCache::load(dir, HOST, 7).path()
}

#[test]
fn roundtrip() {
    let dir = tmp("rt");
    let c = ListCache::load(&dir, HOST, 7);
    let key = query_key("mr", "", "opened", "all");
    let e = entry(1);
    c.put(&key, e.clone());
    c.flush().unwrap();
    let c2 = ListCache::load(&dir, HOST, 7);
    assert_eq!(c2.get(&key), Some(e));
    assert_eq!(c2.last_user_id(), 7);
    assert!(c2.get("other").is_none());
}

#[cfg(unix)]
#[test]
fn modes_are_private() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tmp("mode");
    let c = ListCache::load(&dir, HOST, 7);
    c.put("k", entry(1));
    c.flush().unwrap();
    let f = std::fs::metadata(c.path()).unwrap().permissions().mode() & 0o777;
    let d = std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
    assert_eq!(f, 0o600);
    assert_eq!(d, 0o700);
}

#[test]
fn failed_rename_leaves_no_partial_file() {
    let dir = tmp("atomic");
    let c = ListCache::load(&dir, HOST, 7);
    std::fs::create_dir_all(c.path()).unwrap();
    c.put("k", entry(1));
    assert!(c.flush().is_err());
    let leftovers: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains(".tmp-"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[test]
fn schema_mismatch_resets() {
    let dir = tmp("schema");
    let path = cache_file(&dir);
    std::fs::write(&path, r#"{"schema":99,"last_user_id":7,"entries":{}}"#).unwrap();
    let c = ListCache::load(&dir, HOST, 7);
    assert!(c.get("k").is_none());
    assert!(!path.exists());
}

#[test]
fn corrupt_json_resets() {
    let dir = tmp("corrupt");
    let path = cache_file(&dir);
    std::fs::write(&path, "{not json").unwrap();
    let c = ListCache::load(&dir, HOST, 7);
    assert!(c.get("k").is_none());
    assert!(!path.exists());
}

#[test]
fn ttl_expiry_drops_old_entries() {
    let dir = tmp("ttl");
    let c = ListCache::load(&dir, HOST, 7);
    let mut old = entry(1);
    old.fetched_at -= MAX_AGE_SECS + 10;
    c.put("old", old);
    c.put("new", entry(2));
    c.flush().unwrap();
    let c2 = ListCache::load(&dir, HOST, 7);
    assert!(c2.get("old").is_none());
    assert!(c2.get("new").is_some());
}

#[test]
fn lru_eviction_keeps_newest_within_cap() {
    let dir = tmp("lru");
    let c = ListCache::load(&dir, HOST, 7).with_max_bytes(1000);
    let base = entry(0).fetched_at;
    for i in 0..10u64 {
        let mut e = entry(i as usize);
        e.fetched_at = base - 100 + i;
        c.put(&format!("k{i}"), e);
    }
    c.flush().unwrap();
    assert!(std::fs::metadata(c.path()).unwrap().len() < 1500);
    let c2 = ListCache::load(&dir, HOST, 7);
    assert!(c2.get("k9").is_some());
    assert!(c2.get("k0").is_none());
}

#[test]
fn user_change_invalidates() {
    let dir = tmp("user");
    let c = ListCache::load(&dir, HOST, 7);
    c.put("k", entry(1));
    c.flush().unwrap();
    let old = c.path();
    c.invalidate_user(8);
    assert!(c.get("k").is_none());
    assert!(!old.exists());
    assert_eq!(c.last_user_id(), 8);
    assert!(ListCache::load(&dir, HOST, 7).get("k").is_none());
}

#[test]
fn file_has_no_token_or_host() {
    let dir = tmp("secret");
    let c = ListCache::load(&dir, HOST, 7);
    c.put("k", entry(1));
    c.flush().unwrap();
    let body = std::fs::read_to_string(c.path()).unwrap();
    assert!(!body.contains(TOKEN));
    assert!(!body.contains("secret-corp"));
    let name = c.path().file_name().unwrap().to_string_lossy().into_owned();
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
    let c = ListCache::load(&dir, HOST, 7);
    c.put("k", entry(1));
    assert!(c.flush().is_err());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "precious");
}

#[test]
fn concurrent_puts_do_not_corrupt() {
    let dir = tmp("conc");
    let c = Arc::new(ListCache::load(&dir, HOST, 7));
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
    let c2 = ListCache::load(&dir, HOST, 7);
    for t in 0..8 {
        for i in 0..20 {
            assert!(c2.get(&format!("k{t}-{i}")).is_some());
        }
    }
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
