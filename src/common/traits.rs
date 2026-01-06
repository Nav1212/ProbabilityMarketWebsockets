//! Trait definitions for market clients

use async_trait::async_trait;
use tokio::sync::mpsc;

use super::errors::Result;
use super::types::MarketEvent;

/// Trait for market data clients (Polymarket, Kalshi, etc.)
///
/// This trait provides a unified interface for connecting to and
/// receiving data from prediction market platforms.
#[async_trait]
pub trait MarketClient: Send + Sync {
    /// Connect to the platform's API/WebSocket server
    async fn connect(&mut self) -> Result<()>;

    /// Subscribe to specific markets/assets
    ///
    /// # Arguments
    /// * `asset_ids` - List of asset/token IDs to subscribe to
    async fn subscribe(&mut self, asset_ids: &[String]) -> Result<()>;

    /// Unsubscribe from specific markets/assets
    ///
    /// # Arguments
    /// * `asset_ids` - List of asset/token IDs to unsubscribe from
    async fn unsubscribe(&mut self, asset_ids: &[String]) -> Result<()>;

    /// Start receiving messages and sending them to the provided channel
    ///
    /// This spawns an internal task that processes incoming messages
    /// and forwards them as `MarketEvent`s.
    ///
    /// # Arguments
    /// * `sender` - Channel sender for forwarding events
    async fn start(&mut self, sender: mpsc::Sender<MarketEvent>) -> Result<()>;

    /// Gracefully disconnect from the platform
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if the client is currently connected
    fn is_connected(&self) -> bool;

    /// Get the name of the platform
    fn platform_name(&self) -> &'static str;
}

/// Trait for handling market events
pub trait EventHandler: Send + Sync {
    /// Handle an incoming market event
    fn handle_event(&mut self, event: &MarketEvent);

    /// Called when a connection is established
    fn on_connect(&mut self);

    /// Called when a connection is lost
    fn on_disconnect(&mut self, reason: Option<&str>);
}
