use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tracing_subscriber::{self, Registry, fmt, prelude::*};

pub const LOG_FILE: &str = "lazygitlab.log";

fn non_empty(v: Option<&str>) -> Option<&str> {
    v.map(str::trim).filter(|s| !s.is_empty())
}

pub fn resolve_log_path(
    explicit: Option<&str>,
    xdg_state: Option<&str>,
    home: Option<&str>,
    local_app_data: Option<&str>,
    windows: bool,
    user_tag: &str,
) -> PathBuf {
    if let Some(p) = non_empty(explicit) {
        return PathBuf::from(p);
    }
    let home_dir = non_empty(home).map(|h| {
        PathBuf::from(h)
            .join(".local")
            .join("state")
            .join("lazygitlab")
    });
    let app_data_dir = non_empty(local_app_data).map(|l| PathBuf::from(l).join("lazygitlab"));
    let dir = if let Some(x) = non_empty(xdg_state) {
        Some(PathBuf::from(x).join("lazygitlab"))
    } else if windows {
        app_data_dir.or(home_dir)
    } else {
        home_dir.or(app_data_dir)
    }
    .unwrap_or_else(|| std::env::temp_dir().join(format!("lazygitlab-{user_tag}")));
    dir.join(LOG_FILE)
}

#[cfg(unix)]
fn user_tag() -> String {
    unsafe { libc::geteuid() }.to_string()
}

#[cfg(not(unix))]
fn user_tag() -> String {
    std::env::var("USERNAME").unwrap_or_else(|_| "user".to_string())
}

fn env_var(k: &str) -> Option<String> {
    std::env::var(k).ok()
}

pub fn default_log_path() -> PathBuf {
    resolve_log_path(
        env_var("LAZYGITLAB_LOG").as_deref(),
        env_var("XDG_STATE_HOME").as_deref(),
        env_var("HOME")
            .or_else(|| env_var("USERPROFILE"))
            .as_deref(),
        env_var("LOCALAPPDATA").as_deref(),
        cfg!(windows),
        &user_tag(),
    )
}

pub fn state_dir() -> PathBuf {
    let log = resolve_log_path(
        None,
        env_var("XDG_STATE_HOME").as_deref(),
        env_var("HOME")
            .or_else(|| env_var("USERPROFILE"))
            .as_deref(),
        env_var("LOCALAPPDATA").as_deref(),
        cfg!(windows),
        &user_tag(),
    );
    log.parent().map(Path::to_path_buf).unwrap_or_default()
}

pub fn open_log_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
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

pub fn ensure_private_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)?;
        let meta = std::fs::symlink_metadata(dir)?;
        if !meta.file_type().is_dir() {
            return Err(io::Error::other("log directory is not a plain directory"));
        }
        if meta.uid() != unsafe { libc::geteuid() } {
            return Err(io::Error::other("log directory is owned by another user"));
        }
        if meta.mode() & 0o077 != 0 {
            std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(dir)
    }
}

fn describe_failure(path: &Path, err: &io::Error) -> String {
    let is_symlink = std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);
    let why = if is_symlink {
        "it is a symbolic link, which is refused".to_string()
    } else {
        err.to_string()
    };
    format!(
        "lazygitlab: cannot write log file {}: {why}; logging is disabled (set LAZYGITLAB_LOG to another path)",
        path.display()
    )
}

fn log_writer() -> Box<dyn Write + Send> {
    let path = default_log_path();
    let explicit = non_empty(env_var("LAZYGITLAB_LOG").as_deref()).is_some();
    let prepared = match path.parent().filter(|p| !p.as_os_str().is_empty()) {
        Some(parent) if !explicit => ensure_private_dir(parent),
        _ => Ok(()),
    };
    match prepared.and_then(|()| open_log_file(&path)) {
        Ok(file) => Box::new(file),
        Err(e) => {
            eprintln!("{}", describe_failure(&path, &e));
            Box::new(io::sink())
        }
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
