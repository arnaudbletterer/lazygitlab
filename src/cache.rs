use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const SCHEMA: u32 = 1;
pub const MAX_TOTAL_BYTES: usize = 20 * 1024 * 1024;
pub const MAX_FILE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_AGE_SECS: u64 = 7 * 24 * 3600;

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheEntry {
    pub fetched_at: u64,
    pub etag: Option<String>,
    pub cursor_updated_at: Option<String>,
    pub items: Vec<Value>,
}

impl CacheEntry {
    pub fn new(items: Vec<Value>, etag: Option<String>, cursor_updated_at: Option<String>) -> Self {
        Self {
            fetched_at: now_secs(),
            etag,
            cursor_updated_at,
            items,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct FileData {
    schema: u32,
    last_user_id: u64,
    entries: BTreeMap<String, CacheEntry>,
}

struct State {
    user_id: u64,
    entries: BTreeMap<String, (CacheEntry, usize)>,
    dirty: bool,
}

pub struct ListCache {
    dir: PathBuf,
    host_hash: String,
    max_bytes: usize,
    refused: bool,
    state: Mutex<State>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn host_hash(host: &str) -> String {
    let norm = host.trim().trim_end_matches('/').to_lowercase();
    let digest = Sha256::digest(norm.as_bytes());
    digest.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

pub fn default_cache_dir() -> PathBuf {
    crate::logger::state_dir().join("cache").join("v1")
}

pub fn query_key(kind: &str, filter: &str, state: &str, scope: &str) -> String {
    format!("kind={kind}&filter={filter}&state={state}&scope={scope}")
}

pub fn age_label(fetched_at: u64) -> String {
    let secs = now_secs().saturating_sub(fetched_at);
    match secs {
        0..=59 => "cached just now".to_string(),
        60..=3599 => format!("cached {} min ago", secs / 60),
        3600..=86399 => format!("cached {} h ago", secs / 3600),
        _ => format!("cached {} d ago", secs / 86400),
    }
}

fn entry_size(entry: &CacheEntry) -> usize {
    serde_json::to_vec(entry).map(|v| v.len()).unwrap_or(0)
}

fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

fn read_file(path: &Path) -> Option<Vec<u8>> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path).ok()?;
    let meta = file.metadata().ok()?;
    if !meta.is_file() || meta.len() > MAX_FILE_BYTES {
        return None;
    }
    let mut buf = Vec::with_capacity(meta.len() as usize);
    file.take(MAX_FILE_BYTES + 1).read_to_end(&mut buf).ok()?;
    Some(buf)
}

fn parse(bytes: &[u8]) -> Option<FileData> {
    let value: Value = serde_json::from_slice(bytes).ok()?;
    if value.get("schema").and_then(Value::as_u64) != Some(u64::from(SCHEMA)) {
        return None;
    }
    serde_json::from_value(value).ok()
}

impl ListCache {
    pub fn load(dir: &Path, host: &str, user_id: u64) -> Self {
        let mut cache = Self {
            dir: dir.to_path_buf(),
            host_hash: host_hash(host),
            max_bytes: MAX_TOTAL_BYTES,
            refused: false,
            state: Mutex::new(State {
                user_id,
                entries: BTreeMap::new(),
                dirty: false,
            }),
        };
        cache.refused = crate::logger::ensure_private_dir(dir).is_err();
        let path = cache.path_for(user_id);
        if cache.refused || is_symlink(&path) {
            cache.refused = true;
            return cache;
        }
        if std::fs::symlink_metadata(&path).is_err() {
            return cache;
        }
        let now = now_secs();
        match read_file(&path).as_deref().and_then(parse) {
            Some(data) => {
                let state = cache.state.get_mut().unwrap_or_else(|e| e.into_inner());
                for (key, entry) in data.entries {
                    if now.saturating_sub(entry.fetched_at) <= MAX_AGE_SECS {
                        let size = entry_size(&entry);
                        state.entries.insert(key, (entry, size));
                    }
                }
            }
            None => {
                let _ = std::fs::remove_file(&path);
            }
        }
        cache
    }

    pub fn with_max_bytes(mut self, max_bytes: usize) -> Self {
        self.max_bytes = max_bytes;
        self
    }

    pub fn path(&self) -> PathBuf {
        let id = self.lock().user_id;
        self.path_for(id)
    }

    fn path_for(&self, user_id: u64) -> PathBuf {
        self.dir.join(format!("{}-{user_id}.json", self.host_hash))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        let state = self.lock();
        let (entry, _) = state.entries.get(key)?;
        (now_secs().saturating_sub(entry.fetched_at) <= MAX_AGE_SECS).then(|| entry.clone())
    }

    pub fn put(&self, key: &str, entry: CacheEntry) {
        let size = entry_size(&entry);
        let mut state = self.lock();
        state.entries.insert(key.to_string(), (entry, size));
        let mut total: usize = state.entries.values().map(|(_, s)| *s).sum();
        while total > self.max_bytes {
            let oldest = state
                .entries
                .iter()
                .min_by_key(|(_, (e, _))| e.fetched_at)
                .map(|(k, _)| k.clone());
            match oldest.and_then(|k| state.entries.remove(&k)) {
                Some((_, s)) => total -= s,
                None => break,
            }
        }
        state.dirty = true;
    }

    pub fn last_user_id(&self) -> u64 {
        self.lock().user_id
    }

    pub fn invalidate_user(&self, new_user_id: u64) {
        let mut state = self.lock();
        if state.user_id == new_user_id {
            return;
        }
        let old = self.path_for(state.user_id);
        if !self.refused && !is_symlink(&old) {
            let _ = std::fs::remove_file(&old);
        }
        state.user_id = new_user_id;
        state.entries.clear();
        state.dirty = false;
    }

    pub fn flush(&self) -> io::Result<()> {
        let mut state = self.lock();
        if !state.dirty {
            return Ok(());
        }
        let path = self.path_for(state.user_id);
        if self.refused || is_symlink(&path) {
            return Err(io::Error::other("cache path is refused"));
        }
        let data = FileData {
            schema: SCHEMA,
            last_user_id: state.user_id,
            entries: state
                .entries
                .iter()
                .map(|(k, (e, _))| (k.clone(), e.clone()))
                .collect(),
        };
        let body = serde_json::to_vec(&data).map_err(io::Error::other)?;
        write_private(&path, &body)?;
        state.dirty = false;
        Ok(())
    }
}

fn write_private(path: &Path, body: &[u8]) -> io::Result<()> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "cache.json".into());
    let n = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = path.with_file_name(format!(".{name}.tmp-{}-{n}", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| {
        let mut file = options.open(&tmp)?;
        file.write_all(body)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}
