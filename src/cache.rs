pub use crate::cache_rows::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Bump SCHEMA whenever a cached row gains, loses or changes a field.
pub const SCHEMA: u32 = 3;
pub const MAX_TOTAL_BYTES: usize = 18 * 1024 * 1024;
pub const MAX_FILE_BYTES: u64 = 20 * 1024 * 1024;
pub const MAX_AGE_SECS: u64 = 7 * 24 * 3600;

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheEntry {
    pub fetched_at: u64,
    pub etag: Option<String>,
    pub cursor_updated_at: Option<String>,
    pub items: Vec<CachedItem>,
}

impl CacheEntry {
    pub fn new(
        items: Vec<CachedItem>,
        etag: Option<String>,
        cursor_updated_at: Option<String>,
    ) -> Self {
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

struct Slot {
    entry: CacheEntry,
    size: usize,
    tick: u64,
}

struct State {
    user_id: Option<NonZeroU64>,
    entries: BTreeMap<String, Slot>,
    tick: u64,
    generation: u64,
    flushed_generation: u64,
}

pub struct ListCache {
    dir: PathBuf,
    host_hash: String,
    max_bytes: usize,
    refused: bool,
    flush_guard: Mutex<()>,
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

pub fn default_cache_dir() -> Option<PathBuf> {
    let state = crate::logger::state_dir();
    if state.as_os_str().is_empty() || state.starts_with(std::env::temp_dir()) {
        tracing::info!("list cache disabled: state dir is under temp");
        return None;
    }
    Some(state.join("cache").join("v1"))
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
    (buf.len() as u64 <= MAX_FILE_BYTES).then_some(buf)
}

fn evict_to_fit(state: &mut State, max_bytes: usize) {
    let mut total: usize = state.entries.values().map(|s| s.size).sum();
    while total > max_bytes {
        let oldest = state
            .entries
            .iter()
            .min_by_key(|(_, s)| s.tick)
            .map(|(k, _)| k.clone());
        match oldest.and_then(|k| state.entries.remove(&k)) {
            Some(s) => total -= s.size,
            None => break,
        }
    }
}

impl ListCache {
    pub fn load(dir: &Path, host: &str, user_id: Option<NonZeroU64>) -> Self {
        Self::load_with_max(dir, host, user_id, MAX_TOTAL_BYTES)
    }

    pub fn load_with_max(
        dir: &Path,
        host: &str,
        user_id: Option<NonZeroU64>,
        max_bytes: usize,
    ) -> Self {
        let mut cache = Self {
            dir: dir.to_path_buf(),
            host_hash: host_hash(host),
            max_bytes,
            refused: false,
            flush_guard: Mutex::new(()),
            state: Mutex::new(State {
                user_id,
                entries: BTreeMap::new(),
                tick: 0,
                generation: 0,
                flushed_generation: 0,
            }),
        };
        let Some(id) = user_id else {
            return cache;
        };
        if crate::logger::ensure_private_dir(dir).is_err() {
            cache.refused = true;
            return cache;
        }
        let path = cache.path_for(id);
        if is_symlink(&path) {
            cache.refused = true;
            return cache;
        }
        if std::fs::symlink_metadata(&path).is_err() {
            return cache;
        }
        let now = now_secs();
        let parsed = read_file(&path)
            .and_then(|b| serde_json::from_slice::<FileData>(&b).ok())
            .filter(|d| d.schema == SCHEMA && d.last_user_id == id.get());
        match parsed {
            Some(data) => {
                let state = cache.state.get_mut().unwrap_or_else(|e| e.into_inner());
                let mut fresh: Vec<(String, CacheEntry)> = data
                    .entries
                    .into_iter()
                    .map(|(k, mut e)| {
                        e.fetched_at = e.fetched_at.min(now);
                        (k, e)
                    })
                    .filter(|(_, e)| now - e.fetched_at <= MAX_AGE_SECS)
                    .collect();
                fresh.sort_by_key(|(_, e)| e.fetched_at);
                for (key, entry) in fresh {
                    let size = entry_size(&entry);
                    state.tick += 1;
                    let tick = state.tick;
                    state.entries.insert(key, Slot { entry, size, tick });
                }
                evict_to_fit(state, max_bytes);
            }
            None => {
                let _ = std::fs::remove_file(&path);
            }
        }
        cache
    }

    pub fn path(&self) -> Option<PathBuf> {
        let id = self.lock().user_id?;
        Some(self.path_for(id))
    }

    fn path_for(&self, user_id: NonZeroU64) -> PathBuf {
        self.dir.join(format!("{}-{user_id}.json", self.host_hash))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        let mut state = self.lock();
        state.tick += 1;
        let tick = state.tick;
        let slot = state.entries.get_mut(key)?;
        if now_secs().saturating_sub(slot.entry.fetched_at) > MAX_AGE_SECS {
            return None;
        }
        slot.tick = tick;
        Some(slot.entry.clone())
    }

    pub fn put(&self, key: &str, mut entry: CacheEntry) -> bool {
        entry.fetched_at = entry.fetched_at.min(now_secs());
        let size = entry_size(&entry);
        let mut state = self.lock();
        if self.refused || state.user_id.is_none() || size > self.max_bytes {
            return false;
        }
        state.tick += 1;
        let tick = state.tick;
        state
            .entries
            .insert(key.to_string(), Slot { entry, size, tick });
        evict_to_fit(&mut state, self.max_bytes);
        state.generation += 1;
        true
    }

    pub fn last_user_id(&self) -> Option<NonZeroU64> {
        self.lock().user_id
    }

    pub fn invalidate_user(&self, new_user_id: NonZeroU64) {
        let _flush = self.flush_guard.lock().unwrap_or_else(|e| e.into_inner());
        let old_id = {
            let mut state = self.lock();
            if state.user_id == Some(new_user_id) {
                return;
            }
            let old_id = state.user_id;
            state.user_id = Some(new_user_id);
            state.entries.clear();
            state.flushed_generation = state.generation;
            old_id
        };
        if let Some(old_id) = old_id {
            let old = self.path_for(old_id);
            if !self.refused && !is_symlink(&old) {
                let _ = std::fs::remove_file(&old);
            }
        }
    }

    pub fn flush(&self) -> io::Result<()> {
        self.flush_with_hook(&|| Ok(()))
    }

    #[doc(hidden)]
    pub fn flush_with_hook(&self, before_rename: &dyn Fn() -> io::Result<()>) -> io::Result<()> {
        let _flush = self.flush_guard.lock().unwrap_or_else(|e| e.into_inner());
        let (id, dirty) = {
            let state = self.lock();
            (state.user_id, state.generation != state.flushed_generation)
        };
        let Some(id) = id else {
            return Ok(());
        };
        if !dirty {
            return Ok(());
        }
        let path = self.path_for(id);
        if self.refused || is_symlink(&path) {
            return Err(io::Error::other("cache path is refused"));
        }
        crate::logger::ensure_private_dir(&self.dir)?;
        let (body, generation) = {
            let state = self.lock();
            let data = FileData {
                schema: SCHEMA,
                last_user_id: id.get(),
                entries: state
                    .entries
                    .iter()
                    .map(|(k, s)| (k.clone(), s.entry.clone()))
                    .collect(),
            };
            let body = serde_json::to_vec(&data).map_err(io::Error::other)?;
            (body, state.generation)
        };
        write_private(&path, &body, before_rename)?;
        let mut state = self.lock();
        state.flushed_generation = state.flushed_generation.max(generation);
        Ok(())
    }
}

fn write_private(
    path: &Path,
    body: &[u8],
    before_rename: &dyn Fn() -> io::Result<()>,
) -> io::Result<()> {
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
        before_rename()?;
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
        return result;
    }
    #[cfg(unix)]
    if let Some(parent) = path.parent() {
        let _ = std::fs::File::open(parent).and_then(|d| d.sync_all());
    }
    result
}
