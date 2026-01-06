//! Unified types used across all platform clients

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Source platform identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Polymarket,
    Kalshi,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Polymarket => write!(f, "polymarket"),
            Platform::Kalshi => write!(f, "kalshi"),
        }
    }
}

/// Order side (buy or sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

/// A single price level in an order book
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Price at this level (0.00 to 1.00 for prediction markets)
    pub price: Decimal,
    /// Total size/quantity at this price level
    pub size: Decimal,
}

impl PriceLevel {
    /// Create a new price level
    pub fn new(price: Decimal, size: Decimal) -> Self {
        Self { price, size }
    }
}

/// Full order book for a market
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBook {
    /// Platform this order book is from
    pub platform: Platform,
    /// Market/token identifier
    pub market_id: String,
    /// Asset/token ID (specific to the outcome)
    pub asset_id: String,
    /// Bid (buy) orders sorted by price descending
    pub bids: Vec<PriceLevel>,
    /// Ask (sell) orders sorted by price ascending
    pub asks: Vec<PriceLevel>,
    /// Timestamp of this snapshot
    pub timestamp: DateTime<Utc>,
    /// Sequence number for ordering updates
    #[serde(default)]
    pub sequence: u64,
}

impl OrderBook {
    /// Get the best bid price (highest buy order)
    pub fn best_bid(&self) -> Option<&PriceLevel> {
        self.bids.first()
    }

    /// Get the best ask price (lowest sell order)
    pub fn best_ask(&self) -> Option<&PriceLevel> {
        self.asks.first()
    }

    /// Calculate the midpoint price
    pub fn midpoint(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid.price + ask.price) / Decimal::from(2)),
            _ => None,
        }
    }

    /// Calculate the spread
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask.price - bid.price),
            _ => None,
        }
    }
}

/// Order book update (delta or snapshot)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBookUpdate {
    /// Platform this update is from
    pub platform: Platform,
    /// Market/token identifier
    pub market_id: String,
    /// Asset/token ID
    pub asset_id: String,
    /// Updated bid levels (price of 0 size means removal)
    pub bids: Vec<PriceLevel>,
    /// Updated ask levels (price of 0 size means removal)
    pub asks: Vec<PriceLevel>,
    /// Timestamp of this update
    pub timestamp: DateTime<Utc>,
    /// Whether this is a full snapshot or a delta update
    pub is_snapshot: bool,
    /// Sequence number for ordering
    #[serde(default)]
    pub sequence: u64,
}

/// A single trade execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trade {
    /// Platform this trade is from
    pub platform: Platform,
    /// Market/token identifier
    pub market_id: String,
    /// Asset/token ID
    pub asset_id: String,
    /// Trade ID (unique identifier)
    pub trade_id: String,
    /// Execution price
    pub price: Decimal,
    /// Trade size/quantity
    pub size: Decimal,
    /// Side of the taker order
    pub side: Side,
    /// Timestamp of the trade
    pub timestamp: DateTime<Utc>,
}

/// Market metadata and status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketInfo {
    /// Platform this market is on
    pub platform: Platform,
    /// Market/condition identifier
    pub market_id: String,
    /// Human-readable market question/title
    pub title: String,
    /// Market description
    #[serde(default)]
    pub description: String,
    /// Token/asset IDs for outcomes (YES/NO tokens)
    pub token_ids: Vec<String>,
    /// Whether the market is currently active/tradeable
    pub is_active: bool,
    /// Market end/resolution date
    pub end_date: Option<DateTime<Utc>>,
    /// Minimum tick size for prices
    pub tick_size: Option<Decimal>,
    /// Whether this is a negative risk market
    #[serde(default)]
    pub neg_risk: bool,
}

/// Connection status for a client
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Successfully connected
    Connected,
    /// Disconnected (with optional reason)
    Disconnected(Option<String>),
    /// Attempting to reconnect
    Reconnecting { attempt: u32 },
    /// Connection error
    Error(String),
}

/// Unified market event from any platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEvent {
    /// Full order book snapshot
    OrderBook(OrderBook),
    /// Incremental order book update
    OrderBookUpdate(OrderBookUpdate),
    /// Trade execution
    Trade(Trade),
    /// Market info/metadata update
    MarketInfo(MarketInfo),
    /// Connection status change
    ConnectionStatus {
        platform: Platform,
        status: ConnectionStatus,
    },
    /// Heartbeat/ping response
    Heartbeat { platform: Platform },
    /// Raw/unknown message (for debugging)
    Raw {
        platform: Platform,
        message: String,
    },
}

impl MarketEvent {
    /// Get the platform this event is from
    pub fn platform(&self) -> Platform {
        match self {
            MarketEvent::OrderBook(ob) => ob.platform,
            MarketEvent::OrderBookUpdate(update) => update.platform,
            MarketEvent::Trade(trade) => trade.platform,
            MarketEvent::MarketInfo(info) => info.platform,
            MarketEvent::ConnectionStatus { platform, .. } => *platform,
            MarketEvent::Heartbeat { platform } => *platform,
            MarketEvent::Raw { platform, .. } => *platform,
        }
    }
}

/// Price data returned from the CLOB API
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceData {
    /// The price (0.00 to 1.00)
    pub price: Decimal,
    /// Side (buy or sell)
    pub side: Side,
}

/// Midpoint price data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MidpointData {
    /// The midpoint price
    pub mid: Decimal,
}

/// Spread data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpreadData {
    /// The spread (ask - bid)
    pub spread: Decimal,
}

/// Server time response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerTime {
    /// Unix timestamp in seconds
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_order_book_midpoint() {
        let order_book = OrderBook {
            platform: Platform::Polymarket,
            market_id: "test".to_string(),
            asset_id: "token123".to_string(),
            bids: vec![PriceLevel::new(dec!(0.45), dec!(100))],
            asks: vec![PriceLevel::new(dec!(0.55), dec!(100))],
            timestamp: Utc::now(),
            sequence: 1,
        };

        assert_eq!(order_book.midpoint(), Some(dec!(0.50)));
        assert_eq!(order_book.spread(), Some(dec!(0.10)));
    }

    #[test]
    fn test_empty_order_book() {
        let order_book = OrderBook {
            platform: Platform::Polymarket,
            market_id: "test".to_string(),
            asset_id: "token123".to_string(),
            bids: vec![],
            asks: vec![],
            timestamp: Utc::now(),
            sequence: 0,
        };

        assert!(order_book.midpoint().is_none());
        assert!(order_book.spread().is_none());
        assert!(order_book.best_bid().is_none());
        assert!(order_book.best_ask().is_none());
    }
}
