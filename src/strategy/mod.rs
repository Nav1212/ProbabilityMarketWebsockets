//! Strategy module for trade decision making
//!
//! This module provides the core abstractions for implementing trading strategies.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    ASYNC (background)                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │  SizeCalculator                                             │
//! │    - Watches positions, balances, market liquidity          │
//! │    - Pre-computes optimal size for each potential trade     │
//! │    - Updates continuously                                   │
//! └─────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    HOT PATH (sync)                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Event arrives                                              │
//! │       │                                                     │
//! │       ▼                                                     │
//! │  Strategy.on_market_event() → Go/NoGo                       │
//! │       │                                                     │
//! │       ▼ (if Go)                                             │
//! │  Trader                                                     │
//! │    - Grabs pre-computed size from SizeCalculator            │
//! │    - Executes immediately                                   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Components
//!
//! - [`Strategy`]: Trait for implementing trading strategies
//! - [`Decision`]: Go/NoGo enum returned by strategies
//! - [`TradeIntent`]: Contains one or more [`TradeLeg`]s to execute
//! - [`SizeCalculator`]: Pre-computes trade sizes asynchronously
//! - [`StrategyContext`]: Read-only state provided to strategies
//!
//! # Example
//!
//! ```ignore
//! use strategy::{Strategy, Decision, TradeLeg, Platform, Side};
//!
//! struct SimpleArbitrageStrategy {
//!     threshold: Decimal,
//! }
//!
//! impl Strategy for SimpleArbitrageStrategy {
//!     fn name(&self) -> &str { "simple_arbitrage" }
//!
//!     fn on_market_event(&mut self, event: &MarketEvent, ctx: &StrategyContext) -> Decision {
//!         // Check for arbitrage opportunity
//!         if spread > self.threshold {
//!             Decision::go_arbitrage(
//!                 vec![
//!                     TradeLeg::new(Platform::Kalshi, "market_a", Side::Buy),
//!                     TradeLeg::new(Platform::Polymarket, "market_b", Side::Sell),
//!                 ],
//!                 "Cross-platform spread detected",
//!             )
//!         } else {
//!             Decision::no_go()
//!         }
//!     }
//!
//!     fn subscribed_markets(&self) -> Vec<MarketSubscription> {
//!         vec![MarketSubscription::AllMatchedPairs]
//!     }
//! }
//! ```

mod types;
mod traits;
mod size_calculator;
mod fees;

pub use types::{
    Decision,
    MarketSubscription,
    Platform,
    Position,
    Side,
    StrategyContext,
    TradeIntent,
    TradeLeg,
};

pub use traits::{BoxedStrategy, Strategy};

pub use size_calculator::{
    BoxedSizeCalculator,
    ComputedSize,
    InMemorySizeCalculator,
    SizeCalculator,
    SizeKey,
    SizedIntent,
    SizedLeg,
};

pub use fees::{FeeCalculator, PlatformFees};
