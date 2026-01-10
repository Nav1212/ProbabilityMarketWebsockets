use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Platform identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    Kalshi,
    Polymarket,
}

/// Trade side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

/// A single leg of a trade
///
/// Represents one atomic action: buy or sell on a specific platform/market.
/// Multiple legs can be combined in a TradeIntent for arbitrage or complex strategies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeLeg {
    pub platform: Platform,
    pub market_id: String,
    pub side: Side,
    /// Optional price suggestion from strategy
    /// SizeCalculator or Trader may override based on current book
    pub suggested_price: Option<Decimal>,
}

impl TradeLeg {
    pub fn new(platform: Platform, market_id: impl Into<String>, side: Side) -> Self {
        Self {
            platform,
            market_id: market_id.into(),
            side,
            suggested_price: None,
        }
    }

    pub fn with_price(mut self, price: Decimal) -> Self {
        self.suggested_price = Some(price);
        self
    }
}

/// A trade intent containing one or more legs
///
/// Single leg: Simple directional trade (momentum, mean reversion)
/// Multiple legs: Arbitrage or complex multi-platform trades
///
/// All legs in a single intent are treated as atomic - execute all or none.
#[derive(Debug, Clone)]
pub struct TradeIntent {
    pub legs: Vec<TradeLeg>,
    pub reason: String,
}

impl TradeIntent {
    /// Create a single-leg trade intent
    pub fn single(leg: TradeLeg, reason: impl Into<String>) -> Self {
        Self {
            legs: vec![leg],
            reason: reason.into(),
        }
    }

    /// Create a multi-leg trade intent (e.g., arbitrage)
    pub fn multi(legs: Vec<TradeLeg>, reason: impl Into<String>) -> Self {
        Self {
            legs,
            reason: reason.into(),
        }
    }

    /// Returns true if this is an arbitrage (multi-leg) intent
    pub fn is_arbitrage(&self) -> bool {
        self.legs.len() > 1
    }

    /// Returns the number of legs
    pub fn leg_count(&self) -> usize {
        self.legs.len()
    }
}

/// Strategy decision output
#[derive(Debug, Clone)]
pub enum Decision {
    /// No action should be taken
    NoGo,
    /// Execute the trade intent (one or more legs)
    Go(TradeIntent),
}

impl Decision {
    /// Create a NoGo decision
    pub fn no_go() -> Self {
        Self::NoGo
    }

    /// Create a Go decision with a single leg
    pub fn go_single(leg: TradeLeg, reason: impl Into<String>) -> Self {
        Self::Go(TradeIntent::single(leg, reason))
    }

    /// Create a Go decision with multiple legs (arbitrage)
    pub fn go_arbitrage(legs: Vec<TradeLeg>, reason: impl Into<String>) -> Self {
        Self::Go(TradeIntent::multi(legs, reason))
    }

    /// Returns true if this is a Go decision
    pub fn is_go(&self) -> bool {
        matches!(self, Self::Go(_))
    }
}

/// Current position in a market
#[derive(Debug, Clone)]
pub struct Position {
    pub platform: Platform,
    pub market_id: String,
    /// Positive = long (bought YES/contracts), Negative = short (sold/bought NO)
    pub size: Decimal,
    /// Average entry price
    pub avg_entry_price: Decimal,
}

impl Position {
    pub fn new(platform: Platform, market_id: impl Into<String>) -> Self {
        Self {
            platform,
            market_id: market_id.into(),
            size: Decimal::ZERO,
            avg_entry_price: Decimal::ZERO,
        }
    }
}

/// Context provided to strategies by the Trader
///
/// Contains read-only information about current state.
/// Strategies use this to make informed decisions without owning the state.
#[derive(Debug, Clone, Default)]
pub struct StrategyContext {
    /// Current positions by (platform, market_id)
    pub positions: std::collections::HashMap<(Platform, String), Position>,
    /// Available balance per platform
    pub balances: std::collections::HashMap<Platform, Decimal>,
}

impl StrategyContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get position for a specific market
    pub fn get_position(&self, platform: Platform, market_id: &str) -> Option<&Position> {
        self.positions.get(&(platform, market_id.to_string()))
    }

    /// Get balance for a platform
    pub fn get_balance(&self, platform: Platform) -> Decimal {
        self.balances.get(&platform).copied().unwrap_or_default()
    }

    /// Check if we have any position in a market
    pub fn has_position(&self, platform: Platform, market_id: &str) -> bool {
        self.get_position(platform, market_id)
            .map(|p| p.size != Decimal::ZERO)
            .unwrap_or(false)
    }
}

/// Subscription specifying which markets a strategy cares about
#[derive(Debug, Clone)]
pub enum MarketSubscription {
    /// Subscribe to a specific market on a platform
    Specific { platform: Platform, market_id: String },
    /// Subscribe to all markets on a platform
    AllOnPlatform(Platform),
    /// Subscribe to a matched market pair (for arbitrage)
    MatchedPair { kalshi_market_id: String, polymarket_market_id: String },
    /// Subscribe to all matched pairs
    AllMatchedPairs,
}
