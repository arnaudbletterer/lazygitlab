use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tracing_subscriber::{self, Registry, fmt, prelude::*};

pub const LOG_FILE: &str = "lazygitlab.log";

pub fn resolve_log_path(
    explicit: Option<&str>,
    xdg_state: Option<&str>,
    home: Option<&str>,
    local_app_data: Option<&str>,
) -> PathBuf {
    fn non_empty(v: Option<&str>) -> Option<&str> {
        v.map(str::trim).filter(|s| !s.is_empty())
    }
    if let Some(p) = non_empty(explicit) {
        return PathBuf::from(p);
    }
    let dir = if let Some(x) = non_empty(xdg_state) {
        PathBuf::from(x).join("lazygitlab")
    } else if let Some(h) = non_empty(home) {
        PathBuf::from(h)
            .join(".local")
            .join("state")
            .join("lazygitlab")
    } else if let Some(l) = non_empty(local_app_data) {
        PathBuf::from(l).join("lazygitlab")
    } else {
        std::env::temp_dir().join("lazygitlab")
    };
    dir.join(LOG_FILE)
}

pub fn default_log_path() -> PathBuf {
    let get = |k: &str| std::env::var(k).ok();
    resolve_log_path(
        get("LAZYGITLAB_LOG").as_deref(),
        get("XDG_STATE_HOME").as_deref(),
        get("HOME").or_else(|| get("USERPROFILE")).as_deref(),
        get("LOCALAPPDATA").as_deref(),
    )
}

pub fn open_log_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::other("log path is not a regular file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(file)
}

fn create_private_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(dir)
    }
}

fn log_writer() -> Box<dyn Write + Send> {
    let path = default_log_path();
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        let _ = create_private_dir(parent);
    }
    match open_log_file(&path) {
        Ok(file) => Box::new(file),
        Err(_) => Box::new(io::sink()),
    }
}

pub fn init() -> tracing_appender::non_blocking::WorkerGuard {
    let (non_blocking, guard) = tracing_appender::non_blocking(log_writer());

    let format = fmt::layer()
        .with_writer(crate::redaction::redacting(non_blocking))
        .with_ansi(false)
        .with_thread_ids(true)
        .with_target(false);

    let subscriber = Registry::default().with(format);

    let _ = tracing::subscriber::set_global_default(subscriber);

    tracing::info!("lazygitlab logging initialized");
    guard
}
