use std::io;
use std::time::Duration;

use crossterm::event::Event;

#[async_trait::async_trait]
pub trait EventSource {
    async fn next_event(&self, timeout: Duration) -> io::Result<Option<Event>>;
}
