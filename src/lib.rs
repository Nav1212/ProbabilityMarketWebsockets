//! PolymarketWebsocket Library
//!
//! A Rust library for connecting to Polymarket and Kalshi websockets
//! for real-time market data consumption.

pub mod common;
pub mod config;
pub mod polymarket;
pub mod strategy;

// Re-export commonly used types
pub use common::errors::{ClientError, Result};
pub use common::speedtest::{BenchmarkStats, SpeedTest, SpeedTestGuard, SpeedTestResult};
pub use common::types::{MarketEvent, OrderBook, OrderBookUpdate, Platform, PriceLevel, Side, Trade};
pub use config::types::AppConfig;
pub use polymarket::client::PolymarketClient;
pub use polymarket::rest::PolymarketRestClient;
pub use polymarket::websocket::PolymarketWebSocketClient;

// Strategy types
pub use strategy::{
    BoxedSizeCalculator, BoxedStrategy, ComputedSize, Decision, InMemorySizeCalculator,
    MarketSubscription, Position, SizeCalculator, SizeKey, SizedIntent, SizedLeg, Strategy,
    StrategyContext, TradeIntent, TradeLeg,
};
pub use strategy::{Platform as StrategyPlatform, Side as StrategySide};
