use tracing_subscriber::{self, fmt, prelude::*, Registry};
use tracing_appender;

pub fn init() -> tracing_appender::non_blocking::WorkerGuard {
    let file_appender = tracing_appender::rolling::never(".", "lazygitlab.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let format = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_thread_ids(true)
        .with_target(false);

    let subscriber = Registry::default().with(format);

    tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");

    tracing::info!("lazygitlab logging initialized");
    guard
}
