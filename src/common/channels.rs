//! Channel type definitions for inter-task communication

use tokio::sync::mpsc;

use super::types::MarketEvent;

/// Default channel buffer size
pub const DEFAULT_CHANNEL_SIZE: usize = 1000;

/// Create a new market event channel with the default buffer size
pub fn create_event_channel() -> (mpsc::Sender<MarketEvent>, mpsc::Receiver<MarketEvent>) {
    mpsc::channel(DEFAULT_CHANNEL_SIZE)
}

/// Create a new market event channel with a custom buffer size
pub fn create_event_channel_with_size(
    size: usize,
) -> (mpsc::Sender<MarketEvent>, mpsc::Receiver<MarketEvent>) {
    mpsc::channel(size)
}
