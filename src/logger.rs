use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use tracing_subscriber::{self, Registry, fmt, prelude::*};

pub const LOG_FILE: &str = "lazygitlab.log";

pub fn open_log_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(file)
}

fn log_writer() -> Box<dyn Write + Send> {
    match open_log_file(Path::new(LOG_FILE)) {
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
