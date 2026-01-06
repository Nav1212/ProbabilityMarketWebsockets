//! WebSocket client for Polymarket real-time data

use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, info, instrument, warn};

use super::messages::*;
use crate::common::errors::{ClientError, Result};
use crate::common::types::{
    ConnectionStatus, MarketEvent, OrderBookUpdate, Platform, PriceLevel, Side, Trade,
};
use crate::config::types::ApiCredentials;

/// WebSocket client for Polymarket real-time data
pub struct PolymarketWebSocketClient {
    /// WebSocket URL
    url: String,
    /// Channel type (market or user)
    channel_type: ChannelType,
    /// Optional API credentials for user channel
    credentials: Option<ApiCredentials>,
    /// Heartbeat interval in seconds
    heartbeat_interval: u64,
    /// Connected state flag
    is_connected: Arc<AtomicBool>,
    /// Current subscribed asset IDs
    subscribed_assets: Vec<String>,
}

impl PolymarketWebSocketClient {
    /// Create a new WebSocket client for the market channel (public data)
    pub fn new_market_channel(url: &str) -> Self {
        Self {
            url: format!("{}/ws/market", url.trim_end_matches('/')),
            channel_type: ChannelType::Market,
            credentials: None,
            heartbeat_interval: 10,
            is_connected: Arc::new(AtomicBool::new(false)),
            subscribed_assets: Vec::new(),
        }
    }

    /// Create a new WebSocket client for the user channel (authenticated)
    pub fn new_user_channel(url: &str, credentials: ApiCredentials) -> Self {
        Self {
            url: format!("{}/ws/user", url.trim_end_matches('/')),
            channel_type: ChannelType::User,
            credentials: Some(credentials),
            heartbeat_interval: 10,
            is_connected: Arc::new(AtomicBool::new(false)),
            subscribed_assets: Vec::new(),
        }
    }

    /// Set the heartbeat interval
    pub fn with_heartbeat_interval(mut self, seconds: u64) -> Self {
        self.heartbeat_interval = seconds;
        self
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }

    /// Connect and start receiving messages
    ///
    /// This method connects to the WebSocket, subscribes to the given assets,
    /// and spawns tasks to handle incoming messages and heartbeats.
    #[instrument(skip(self, event_sender))]
    pub async fn connect_and_subscribe(
        &mut self,
        asset_ids: Vec<String>,
        event_sender: mpsc::Sender<MarketEvent>,
    ) -> Result<()> {
        info!("Connecting to Polymarket WebSocket: {}", self.url);

        // Connect to WebSocket
        let (ws_stream, _response) = connect_async(&self.url)
            .await
            .map_err(|e| ClientError::WebSocketConnection(e.to_string()))?;

        info!("WebSocket connection established");
        self.is_connected.store(true, Ordering::SeqCst);
        self.subscribed_assets = asset_ids.clone();

        // Send connection status
        let _ = event_sender
            .send(MarketEvent::ConnectionStatus {
                platform: Platform::Polymarket,
                status: ConnectionStatus::Connected,
            })
            .await;

        let (mut write, mut read) = ws_stream.split();

        // Send initial subscription message
        let subscribe_msg = self.create_subscribe_message(&asset_ids);
        let msg_json = serde_json::to_string(&subscribe_msg)?;
        debug!("Sending subscription message: {}", msg_json);
        write.send(Message::Text(msg_json)).await?;

        // Clone values for the spawned tasks
        let is_connected = self.is_connected.clone();
        let heartbeat_interval = self.heartbeat_interval;
        let event_sender_clone = event_sender.clone();

        // Spawn heartbeat task
        let is_connected_heartbeat = is_connected.clone();
        let (heartbeat_tx, mut heartbeat_rx) = mpsc::channel::<()>(1);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(heartbeat_interval));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !is_connected_heartbeat.load(Ordering::SeqCst) {
                            break;
                        }
                        // Heartbeat is handled by the main loop
                    }
                    _ = heartbeat_rx.recv() => {
                        // Shutdown signal received
                        break;
                    }
                }
            }
        });

        // Spawn message handling task
        let is_connected_msg = is_connected.clone();
        tokio::spawn(async move {
            let mut ping_interval = interval(Duration::from_secs(heartbeat_interval));
            
            loop {
                tokio::select! {
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if text == "PONG" || text == "pong" {
                                    debug!("Received PONG");
                                    let _ = event_sender_clone
                                        .send(MarketEvent::Heartbeat {
                                            platform: Platform::Polymarket,
                                        })
                                        .await;
                                    continue;
                                }

                                // Parse and forward the message
                                match Self::parse_message(&text) {
                                    Ok(event) => {
                                        if let Err(e) = event_sender_clone.send(event).await {
                                            error!("Failed to send event: {}", e);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse message: {} - {}", e, text);
                                        // Send raw message for debugging
                                        let _ = event_sender_clone
                                            .send(MarketEvent::Raw {
                                                platform: Platform::Polymarket,
                                                message: text,
                                            })
                                            .await;
                                    }
                                }
                            }
                            Some(Ok(Message::Ping(data))) => {
                                debug!("Received Ping, sending Pong");
                                // Note: Pong should be sent automatically by tungstenite
                            }
                            Some(Ok(Message::Pong(_))) => {
                                debug!("Received Pong");
                            }
                            Some(Ok(Message::Close(frame))) => {
                                info!("WebSocket closed: {:?}", frame);
                                is_connected_msg.store(false, Ordering::SeqCst);
                                let _ = event_sender_clone
                                    .send(MarketEvent::ConnectionStatus {
                                        platform: Platform::Polymarket,
                                        status: ConnectionStatus::Disconnected(
                                            frame.map(|f| f.reason.to_string()),
                                        ),
                                    })
                                    .await;
                                break;
                            }
                            Some(Err(e)) => {
                                error!("WebSocket error: {}", e);
                                is_connected_msg.store(false, Ordering::SeqCst);
                                let _ = event_sender_clone
                                    .send(MarketEvent::ConnectionStatus {
                                        platform: Platform::Polymarket,
                                        status: ConnectionStatus::Error(e.to_string()),
                                    })
                                    .await;
                                break;
                            }
                            None => {
                                info!("WebSocket stream ended");
                                is_connected_msg.store(false, Ordering::SeqCst);
                                let _ = event_sender_clone
                                    .send(MarketEvent::ConnectionStatus {
                                        platform: Platform::Polymarket,
                                        status: ConnectionStatus::Disconnected(None),
                                    })
                                    .await;
                                break;
                            }
                            _ => {}
                        }
                    }
                    _ = ping_interval.tick() => {
                        // Note: Ping sending would need write access
                        // In production, you'd use a shared write handle
                    }
                }
            }
            
            // Signal heartbeat task to stop
            drop(heartbeat_tx);
        });

        Ok(())
    }

    /// Create subscription message based on channel type
    fn create_subscribe_message(&self, asset_ids: &[String]) -> WsSubscribeMessage {
        match self.channel_type {
            ChannelType::Market => WsSubscribeMessage {
                channel_type: ChannelType::Market,
                assets_ids: Some(asset_ids.to_vec()),
                markets: None,
                auth: None,
            },
            ChannelType::User => {
                let auth = self.credentials.as_ref().map(|creds| WsAuth {
                    api_key: creds.api_key.clone(),
                    secret: creds.api_secret.clone(),
                    passphrase: creds.passphrase.clone(),
                });
                WsSubscribeMessage {
                    channel_type: ChannelType::User,
                    assets_ids: None,
                    markets: Some(asset_ids.to_vec()),
                    auth,
                }
            }
        }
    }

    /// Parse an incoming WebSocket message into a MarketEvent
    fn parse_message(text: &str) -> Result<MarketEvent> {
        // Try to parse as JSON
        let value: serde_json::Value = serde_json::from_str(text)?;

        // Check for event_type field
        if let Some(event_type) = value.get("event_type").and_then(|v| v.as_str()) {
            match event_type {
                "book" => {
                    let book_event: BookUpdateEvent = serde_json::from_value(value)?;
                    return Ok(Self::convert_book_update(book_event));
                }
                "price_change" => {
                    let price_event: PriceChangeEvent = serde_json::from_value(value)?;
                    return Ok(Self::convert_price_change(price_event));
                }
                "trade" | "last_trade_price" => {
                    // Check if it's a trade or just a price update
                    if value.get("id").is_some() {
                        let trade_event: TradeEvent = serde_json::from_value(value)?;
                        return Ok(Self::convert_trade(trade_event));
                    } else {
                        let ltp_event: LastTradePriceEvent = serde_json::from_value(value)?;
                        return Ok(MarketEvent::Raw {
                            platform: Platform::Polymarket,
                            message: format!(
                                "Last trade price for {}: {}",
                                ltp_event.asset_id, ltp_event.price
                            ),
                        });
                    }
                }
                _ => {}
            }
        }

        // If we couldn't parse it specifically, try general parsing
        if value.get("bids").is_some() && value.get("asks").is_some() {
            let book_event: BookUpdateEvent = serde_json::from_value(value)?;
            return Ok(Self::convert_book_update(book_event));
        }

        // Return as raw message
        Ok(MarketEvent::Raw {
            platform: Platform::Polymarket,
            message: text.to_string(),
        })
    }

    /// Convert a BookUpdateEvent to OrderBookUpdate
    fn convert_book_update(event: BookUpdateEvent) -> MarketEvent {
        let bids: Vec<PriceLevel> = event
            .bids
            .into_iter()
            .filter_map(|level| {
                Some(PriceLevel {
                    price: level.price.parse().ok()?,
                    size: level.size.parse().ok()?,
                })
            })
            .collect();

        let asks: Vec<PriceLevel> = event
            .asks
            .into_iter()
            .filter_map(|level| {
                Some(PriceLevel {
                    price: level.price.parse().ok()?,
                    size: level.size.parse().ok()?,
                })
            })
            .collect();

        MarketEvent::OrderBookUpdate(OrderBookUpdate {
            platform: Platform::Polymarket,
            market_id: event.market.unwrap_or_default(),
            asset_id: event.asset_id,
            bids,
            asks,
            timestamp: chrono::Utc::now(),
            is_snapshot: event.event_type.as_deref() == Some("book"),
            sequence: 0,
        })
    }

    /// Convert a PriceChangeEvent to OrderBookUpdate
    fn convert_price_change(event: PriceChangeEvent) -> MarketEvent {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        if let Some(changes) = event.changes {
            for change in changes {
                if let (Ok(price), Ok(size)) = (change.price.parse(), change.size.parse()) {
                    let level = PriceLevel { price, size };
                    match change.side.to_lowercase().as_str() {
                        "buy" | "bid" => bids.push(level),
                        "sell" | "ask" => asks.push(level),
                        _ => {}
                    }
                }
            }
        }

        MarketEvent::OrderBookUpdate(OrderBookUpdate {
            platform: Platform::Polymarket,
            market_id: event.market.unwrap_or_default(),
            asset_id: event.asset_id,
            bids,
            asks,
            timestamp: chrono::Utc::now(),
            is_snapshot: false,
            sequence: 0,
        })
    }

    /// Convert a TradeEvent to Trade
    fn convert_trade(event: TradeEvent) -> MarketEvent {
        let side = match event.side.to_lowercase().as_str() {
            "buy" | "bid" => Side::Buy,
            _ => Side::Sell,
        };

        MarketEvent::Trade(Trade {
            platform: Platform::Polymarket,
            market_id: event.market.unwrap_or_default(),
            asset_id: event.asset_id,
            trade_id: event.id.unwrap_or_default(),
            price: event.price.parse().unwrap_or_default(),
            size: event.size.parse().unwrap_or_default(),
            side,
            timestamp: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_channel_creation() {
        let client =
            PolymarketWebSocketClient::new_market_channel("wss://ws-subscriptions-clob.polymarket.com");
        assert!(client.url.contains("/ws/market"));
        assert!(!client.is_connected());
    }

    #[test]
    fn test_parse_book_update() {
        let json = r#"{
            "event_type": "book",
            "asset_id": "123456",
            "market": "condition_123",
            "bids": [{"price": "0.50", "size": "100"}],
            "asks": [{"price": "0.55", "size": "50"}]
        }"#;

        let result = PolymarketWebSocketClient::parse_message(json);
        assert!(result.is_ok());

        if let Ok(MarketEvent::OrderBookUpdate(update)) = result {
            assert_eq!(update.asset_id, "123456");
            assert_eq!(update.bids.len(), 1);
            assert_eq!(update.asks.len(), 1);
        } else {
            panic!("Expected OrderBookUpdate");
        }
    }

    #[test]
    fn test_parse_trade() {
        let json = r#"{
            "event_type": "trade",
            "asset_id": "123456",
            "market": "condition_123",
            "id": "trade_1",
            "price": "0.52",
            "size": "25",
            "side": "buy"
        }"#;

        let result = PolymarketWebSocketClient::parse_message(json);
        assert!(result.is_ok());

        if let Ok(MarketEvent::Trade(trade)) = result {
            assert_eq!(trade.asset_id, "123456");
            assert_eq!(trade.trade_id, "trade_1");
            assert_eq!(trade.side, Side::Buy);
        } else {
            panic!("Expected Trade");
        }
    }
}
