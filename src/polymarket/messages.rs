//! Polymarket-specific message types

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// WebSocket channel types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Market,
    User,
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelType::Market => write!(f, "market"),
            ChannelType::User => write!(f, "user"),
        }
    }
}

/// Authentication payload for WebSocket connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsAuth {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

/// Subscribe message for WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSubscribeMessage {
    /// Channel type (market or user)
    #[serde(rename = "type")]
    pub channel_type: ChannelType,
    /// Asset IDs to subscribe to (for market channel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets_ids: Option<Vec<String>>,
    /// Market/condition IDs (for user channel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markets: Option<Vec<String>>,
    /// Authentication (required for user channel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<WsAuth>,
}

/// Subscribe/unsubscribe operation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsOperationMessage {
    /// Operation type
    pub operation: String, // "subscribe" or "unsubscribe"
    /// Asset IDs for the operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets_ids: Option<Vec<String>>,
    /// Market IDs for the operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markets: Option<Vec<String>>,
}

/// Incoming WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WsIncomingMessage {
    /// Price change event
    PriceChange(PriceChangeEvent),
    /// Order book snapshot/update
    BookUpdate(BookUpdateEvent),
    /// Trade event
    Trade(TradeEvent),
    /// Last trade price event
    LastTradePrice(LastTradePriceEvent),
    /// User order update
    OrderUpdate(OrderUpdateEvent),
    /// Generic/unknown message
    Unknown(serde_json::Value),
}

/// Price change event from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceChangeEvent {
    pub event_type: Option<String>,
    pub asset_id: String,
    #[serde(default)]
    pub market: Option<String>,
    pub price: Option<Decimal>,
    #[serde(default)]
    pub changes: Option<Vec<PriceChange>>,
    #[serde(default)]
    pub timestamp: Option<i64>,
}

/// A single price change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceChange {
    pub side: String,
    pub price: String,
    pub size: String,
}

/// Book update event from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookUpdateEvent {
    pub event_type: Option<String>,
    pub asset_id: String,
    #[serde(default)]
    pub market: Option<String>,
    pub hash: Option<String>,
    #[serde(default)]
    pub bids: Vec<BookLevel>,
    #[serde(default)]
    pub asks: Vec<BookLevel>,
    #[serde(default)]
    pub timestamp: Option<i64>,
}

/// A price level in the book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel {
    pub price: String,
    pub size: String,
}

/// Trade event from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEvent {
    pub event_type: Option<String>,
    pub asset_id: String,
    #[serde(default)]
    pub market: Option<String>,
    pub id: Option<String>,
    pub price: String,
    pub size: String,
    pub side: String,
    #[serde(default)]
    pub timestamp: Option<i64>,
}

/// Last trade price event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastTradePriceEvent {
    pub event_type: Option<String>,
    pub asset_id: String,
    pub price: String,
    #[serde(default)]
    pub timestamp: Option<i64>,
}

/// User order update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdateEvent {
    pub event_type: Option<String>,
    pub order_id: String,
    pub market: Option<String>,
    pub asset_id: Option<String>,
    pub side: String,
    pub price: String,
    pub original_size: String,
    pub size_matched: String,
    pub status: String,
    #[serde(default)]
    pub timestamp: Option<i64>,
}

// ============================================================================
// REST API Response Types
// ============================================================================

/// Response from GET /
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkResponse {
    #[serde(default)]
    pub ok: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

/// Response from GET /time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeResponse {
    pub timestamp: String,
}

/// Response from GET /price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceResponse {
    pub price: String,
}

/// Response from GET /midpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidpointResponse {
    pub mid: String,
}

/// Response from GET /spread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadResponse {
    pub spread: String,
}

/// Response from GET /book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookResponse {
    pub market: String,
    pub asset_id: String,
    #[serde(default)]
    pub hash: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    pub bids: Vec<BookLevel>,
    pub asks: Vec<BookLevel>,
}

/// Response from GET /last-trade-price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastTradePriceResponse {
    pub price: String,
}

/// Market from GET /markets or /simplified-markets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketResponse {
    pub condition_id: String,
    pub question_id: Option<String>,
    pub tokens: Vec<TokenInfo>,
    pub rewards: Option<RewardsInfo>,
    #[serde(default)]
    pub minimum_order_size: Option<String>,
    #[serde(default)]
    pub minimum_tick_size: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub end_date_iso: Option<String>,
    #[serde(default)]
    pub game_start_time: Option<String>,
    #[serde(default)]
    pub question: Option<String>,
    #[serde(default)]
    pub market_slug: Option<String>,
    #[serde(default)]
    pub min_incentive_size: Option<String>,
    #[serde(default)]
    pub max_incentive_spread: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub closed: Option<bool>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub accepting_orders: Option<bool>,
    #[serde(default)]
    pub accepting_order_timestamp: Option<String>,
    #[serde(default)]
    pub neg_risk: Option<bool>,
}

/// Token information within a market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub token_id: String,
    pub outcome: String,
    #[serde(default)]
    pub price: Option<String>,
    #[serde(default)]
    pub winner: Option<bool>,
}

/// Rewards information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsInfo {
    #[serde(default)]
    pub rates: Option<Vec<RewardRate>>,
    #[serde(default)]
    pub min_size: Option<String>,
    #[serde(default)]
    pub max_spread: Option<String>,
}

/// Individual reward rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardRate {
    pub asset_address: String,
    pub rewards_daily_rate: String,
}

/// Paginated markets response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsResponse {
    pub data: Vec<MarketResponse>,
    #[serde(default)]
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub count: Option<u32>,
}

/// Trade history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResponse {
    pub id: String,
    #[serde(default)]
    pub taker_order_id: Option<String>,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub size: String,
    pub price: String,
    pub status: String,
    #[serde(default)]
    pub match_time: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub fee_rate_bps: Option<String>,
    #[serde(default)]
    pub maker_address: Option<String>,
    #[serde(default)]
    pub trader_side: Option<String>,
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub bucket_index: Option<u32>,
    #[serde(default)]
    pub outcome: Option<String>,
    #[serde(default)]
    pub type_: Option<String>,
}

/// Paginated trades response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesResponse {
    pub data: Vec<TradeResponse>,
    #[serde(default)]
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub count: Option<u32>,
}

// ============================================================================
// Gamma API Response Types (Market Discovery)
// ============================================================================

/// Event from Gamma API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaEvent {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub markets: Option<Vec<GammaMarket>>,
}

/// Market from Gamma API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaMarket {
    pub id: String,
    pub question: String,
    #[serde(default)]
    pub condition_id: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub volume: Option<String>,
    #[serde(default)]
    pub liquidity: Option<String>,
    #[serde(default)]
    pub outcomes: Option<Vec<String>>,
    #[serde(rename = "outcomePrices", default)]
    pub outcome_prices: Option<Vec<String>>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub closed: Option<bool>,
    #[serde(default)]
    pub neg_risk: Option<bool>,
    #[serde(default)]
    pub tokens: Option<Vec<GammaToken>>,
}

/// Token from Gamma API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaToken {
    pub token_id: String,
    pub outcome: String,
    #[serde(default)]
    pub price: Option<f64>,
}

/// Paginated Gamma events response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaEventsResponse {
    #[serde(default)]
    pub data: Option<Vec<GammaEvent>>,
    #[serde(default)]
    pub events: Option<Vec<GammaEvent>>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// Paginated Gamma markets response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaMarketsResponse {
    #[serde(default)]
    pub data: Option<Vec<GammaMarket>>,
    #[serde(default)]
    pub markets: Option<Vec<GammaMarket>>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}
