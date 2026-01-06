//! PolymarketWebsocket Library
//!
//! A Rust library for connecting to Polymarket and Kalshi websockets
//! for real-time market data consumption.

pub mod common;
pub mod config;
pub mod polymarket;

// Re-export commonly used types
pub use common::errors::{ClientError, Result};
pub use common::speedtest::{BenchmarkStats, SpeedTest, SpeedTestGuard, SpeedTestResult};
pub use common::types::{MarketEvent, OrderBook, OrderBookUpdate, Platform, PriceLevel, Side, Trade};
pub use config::types::AppConfig;
pub use polymarket::client::PolymarketClient;
pub use polymarket::rest::PolymarketRestClient;
pub use polymarket::websocket::PolymarketWebSocketClient;
