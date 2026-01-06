//! Main Polymarket client that combines REST and WebSocket functionality

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, instrument};

use super::rest::PolymarketRestClient;
use super::websocket::PolymarketWebSocketClient;
use crate::common::errors::Result;
use crate::common::traits::MarketClient;
use crate::common::types::{MarketEvent, OrderBook};
use crate::config::types::{ApiCredentials, PolymarketConfig};

/// Combined Polymarket client for REST API and WebSocket connections
pub struct PolymarketClient {
    /// REST API client
    rest_client: PolymarketRestClient,
    /// WebSocket client (created on connect)
    ws_client: Option<PolymarketWebSocketClient>,
    /// Configuration
    config: PolymarketConfig,
    /// API credentials (optional)
    credentials: Option<ApiCredentials>,
    /// Current subscribed markets
    subscribed_markets: Arc<RwLock<Vec<String>>>,
    /// Event sender for WebSocket events
    event_sender: Option<mpsc::Sender<MarketEvent>>,
}

impl PolymarketClient {
    /// Create a new Polymarket client from configuration
    pub fn new(config: PolymarketConfig) -> Result<Self> {
        let rest_client = PolymarketRestClient::new(&config.rest_url, &config.gamma_url)?;

        let credentials = match (&config.api_key, &config.api_secret, &config.api_passphrase) {
            (Some(key), Some(secret), Some(passphrase)) => {
                Some(ApiCredentials::new(key.clone(), secret.clone(), passphrase.clone()))
            }
            _ => None,
        };

        // Apply credentials to REST client if available
        let rest_client = if let Some(ref creds) = credentials {
            rest_client.with_credentials(creds.clone())
        } else {
            rest_client
        };

        Ok(Self {
            rest_client,
            ws_client: None,
            config,
            credentials,
            subscribed_markets: Arc::new(RwLock::new(Vec::new())),
            event_sender: None,
        })
    }

    /// Get a reference to the REST client
    pub fn rest(&self) -> &PolymarketRestClient {
        &self.rest_client
    }

    /// Check if the API is healthy
    pub async fn check_health(&self) -> Result<bool> {
        self.rest_client.get_ok().await
    }

    /// Get server time
    pub async fn get_server_time(&self) -> Result<i64> {
        self.rest_client.get_server_time().await
    }

    /// Get order book for a token
    pub async fn get_order_book(&self, token_id: &str) -> Result<OrderBook> {
        self.rest_client.get_order_book(token_id).await
    }
}

#[async_trait]
impl MarketClient for PolymarketClient {
    #[instrument(skip(self))]
    async fn connect(&mut self) -> Result<()> {
        info!("Creating Polymarket WebSocket client");

        let ws_client = PolymarketWebSocketClient::new_market_channel(&self.config.websocket_url);
        self.ws_client = Some(ws_client);

        Ok(())
    }

    #[instrument(skip(self))]
    async fn subscribe(&mut self, asset_ids: &[String]) -> Result<()> {
        let mut markets = self.subscribed_markets.write().await;
        for id in asset_ids {
            if !markets.contains(id) {
                markets.push(id.clone());
            }
        }
        info!("Subscribed to {} markets", markets.len());
        Ok(())
    }

    #[instrument(skip(self))]
    async fn unsubscribe(&mut self, asset_ids: &[String]) -> Result<()> {
        let mut markets = self.subscribed_markets.write().await;
        markets.retain(|m| !asset_ids.contains(m));
        info!("Unsubscribed from markets, {} remaining", markets.len());
        Ok(())
    }

    #[instrument(skip(self, sender))]
    async fn start(&mut self, sender: mpsc::Sender<MarketEvent>) -> Result<()> {
        self.event_sender = Some(sender.clone());

        let markets = self.subscribed_markets.read().await.clone();

        if let Some(ref mut ws_client) = self.ws_client {
            ws_client.connect_and_subscribe(markets, sender).await?;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn disconnect(&mut self) -> Result<()> {
        // WebSocket will be dropped and closed
        self.ws_client = None;
        self.event_sender = None;
        info!("Disconnected from Polymarket");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.ws_client
            .as_ref()
            .map(|ws| ws.is_connected())
            .unwrap_or(false)
    }

    fn platform_name(&self) -> &'static str {
        "Polymarket"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = PolymarketConfig::default();
        let client = PolymarketClient::new(config);
        assert!(client.is_ok());
    }
}
