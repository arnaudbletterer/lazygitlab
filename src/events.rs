use std::future::Future;
use std::io;
use std::time::Duration;

use crossterm::event::Event;

pub trait EventSource {
    fn next_event(
        &self,
        timeout: Duration,
    ) -> impl Future<Output = io::Result<Option<Event>>> + Send;
}
