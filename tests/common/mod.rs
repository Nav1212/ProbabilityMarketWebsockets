//! Common test utilities and fixtures

use polymarket_websocket::common::types::{OrderBook, Platform, PriceLevel, Side, Trade};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Create a sample order book for testing
pub fn sample_order_book() -> OrderBook {
    OrderBook {
        platform: Platform::Polymarket,
        market_id: "test_market_123".to_string(),
        asset_id: "test_token_456".to_string(),
        bids: vec![
            PriceLevel::new(dec!(0.50), dec!(100)),
            PriceLevel::new(dec!(0.48), dec!(200)),
            PriceLevel::new(dec!(0.45), dec!(150)),
        ],
        asks: vec![
            PriceLevel::new(dec!(0.55), dec!(80)),
            PriceLevel::new(dec!(0.58), dec!(120)),
            PriceLevel::new(dec!(0.60), dec!(90)),
        ],
        timestamp: chrono::Utc::now(),
        sequence: 1,
    }
}

/// Create a sample trade for testing
pub fn sample_trade() -> Trade {
    Trade {
        platform: Platform::Polymarket,
        market_id: "test_market_123".to_string(),
        asset_id: "test_token_456".to_string(),
        trade_id: "trade_789".to_string(),
        price: dec!(0.52),
        size: dec!(50),
        side: Side::Buy,
        timestamp: chrono::Utc::now(),
    }
}

/// Sample WebSocket messages for testing parsing
pub mod ws_messages {
    /// Sample book update message
    pub const BOOK_UPDATE: &str = r#"{
        "event_type": "book",
        "asset_id": "109681959945973300464568698402968596289258214226684818748321941747028805721376",
        "market": "0x123456",
        "hash": "0xabc123",
        "bids": [
            {"price": "0.50", "size": "100"},
            {"price": "0.48", "size": "200"}
        ],
        "asks": [
            {"price": "0.55", "size": "80"},
            {"price": "0.58", "size": "120"}
        ],
        "timestamp": 1704067200
    }"#;

    /// Sample price change message
    pub const PRICE_CHANGE: &str = r#"{
        "event_type": "price_change",
        "asset_id": "109681959945973300464568698402968596289258214226684818748321941747028805721376",
        "market": "0x123456",
        "changes": [
            {"side": "buy", "price": "0.51", "size": "150"}
        ],
        "timestamp": 1704067201
    }"#;

    /// Sample trade message
    pub const TRADE: &str = r#"{
        "event_type": "trade",
        "asset_id": "109681959945973300464568698402968596289258214226684818748321941747028805721376",
        "market": "0x123456",
        "id": "trade_001",
        "price": "0.52",
        "size": "25",
        "side": "buy",
        "timestamp": 1704067202
    }"#;
}

/// Sample API responses for testing
pub mod api_responses {
    /// Sample order book response
    pub const ORDER_BOOK: &str = r#"{
        "market": "0x123456",
        "asset_id": "109681959945973300464568698402968596289258214226684818748321941747028805721376",
        "hash": "0xabc123",
        "bids": [
            {"price": "0.50", "size": "100"},
            {"price": "0.48", "size": "200"}
        ],
        "asks": [
            {"price": "0.55", "size": "80"},
            {"price": "0.58", "size": "120"}
        ]
    }"#;

    /// Sample markets response
    pub const MARKETS: &str = r#"{
        "data": [
            {
                "condition_id": "0x123456",
                "question": "Will it rain tomorrow?",
                "tokens": [
                    {"token_id": "token_yes", "outcome": "Yes"},
                    {"token_id": "token_no", "outcome": "No"}
                ],
                "active": true
            }
        ]
    }"#;

    /// Sample Gamma market response
    pub const GAMMA_MARKET: &str = r#"{
        "id": "market_001",
        "question": "Will it rain tomorrow?",
        "condition_id": "0x123456",
        "active": true,
        "tokens": [
            {"token_id": "token_yes", "outcome": "Yes", "price": 0.65},
            {"token_id": "token_no", "outcome": "No", "price": 0.35}
        ]
    }"#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_order_book() {
        let ob = sample_order_book();
        assert_eq!(ob.bids.len(), 3);
        assert_eq!(ob.asks.len(), 3);
        assert_eq!(ob.best_bid().unwrap().price, dec!(0.50));
        assert_eq!(ob.best_ask().unwrap().price, dec!(0.55));
    }

    #[test]
    fn test_sample_trade() {
        let trade = sample_trade();
        assert_eq!(trade.side, Side::Buy);
        assert_eq!(trade.price, dec!(0.52));
    }
}
