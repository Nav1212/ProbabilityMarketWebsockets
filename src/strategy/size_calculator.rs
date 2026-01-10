use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::strategy::types::{Platform, Side, TradeLeg, TradeIntent};

/// Pre-computed size for a potential trade
#[derive(Debug, Clone)]
pub struct ComputedSize {
    pub platform: Platform,
    pub market_id: String,
    pub side: Side,
    pub size: Decimal,
    pub price: Decimal,
    /// Timestamp when this was computed
    pub computed_at: chrono::DateTime<chrono::Utc>,
}

/// Key for looking up pre-computed sizes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SizeKey {
    pub platform: Platform,
    pub market_id: String,
    pub side: Side,
}

impl SizeKey {
    pub fn new(platform: Platform, market_id: impl Into<String>, side: Side) -> Self {
        Self {
            platform,
            market_id: market_id.into(),
            side,
        }
    }

    pub fn from_leg(leg: &TradeLeg) -> Self {
        Self {
            platform: leg.platform,
            market_id: leg.market_id.clone(),
            side: leg.side,
        }
    }
}

/// Sized trade leg ready for execution
#[derive(Debug, Clone)]
pub struct SizedLeg {
    pub platform: Platform,
    pub market_id: String,
    pub side: Side,
    pub size: Decimal,
    pub price: Decimal,
}

/// Sized trade intent ready for execution
#[derive(Debug, Clone)]
pub struct SizedIntent {
    pub legs: Vec<SizedLeg>,
    pub reason: String,
}

impl SizedIntent {
    pub fn is_valid(&self) -> bool {
        !self.legs.is_empty() && self.legs.iter().all(|leg| leg.size > Decimal::ZERO)
    }
}

/// SizeCalculator trait
///
/// Runs asynchronously in the background, pre-computing optimal trade sizes.
/// The Trader queries this synchronously in the hot path to get sizes instantly.
///
/// # Responsibilities
///
/// - Monitor balances, positions, and market liquidity
/// - Pre-compute optimal sizes for potential trades
/// - Account for fees, slippage, and risk limits
/// - Update continuously as market conditions change
///
/// # Design
///
/// The calculator maintains a cache of ComputedSize entries.
/// When the Trader receives a Go decision, it calls `get_sized_intent()`
/// which looks up pre-computed sizes for each leg.
pub trait SizeCalculator: Send + Sync {
    /// Get pre-computed size for a single leg
    fn get_size(&self, key: &SizeKey) -> Option<ComputedSize>;

    /// Convert a TradeIntent into a SizedIntent
    ///
    /// Looks up pre-computed sizes for all legs.
    /// Returns None if any leg doesn't have a computed size.
    fn get_sized_intent(&self, intent: &TradeIntent) -> Option<SizedIntent> {
        let mut sized_legs = Vec::with_capacity(intent.legs.len());

        for leg in &intent.legs {
            let key = SizeKey::from_leg(leg);
            let computed = self.get_size(&key)?;

            sized_legs.push(SizedLeg {
                platform: leg.platform,
                market_id: leg.market_id.clone(),
                side: leg.side,
                size: computed.size,
                price: leg.suggested_price.unwrap_or(computed.price),
            });
        }

        Some(SizedIntent {
            legs: sized_legs,
            reason: intent.reason.clone(),
        })
    }

    /// Check if sizes are available for all legs of an intent
    fn can_size(&self, intent: &TradeIntent) -> bool {
        intent.legs.iter().all(|leg| {
            let key = SizeKey::from_leg(leg);
            self.get_size(&key).is_some()
        })
    }

    /// Get the age of the oldest size computation for an intent
    /// Useful for checking staleness
    fn oldest_computation_age(&self, intent: &TradeIntent) -> Option<chrono::Duration> {
        let now = chrono::Utc::now();
        intent
            .legs
            .iter()
            .filter_map(|leg| {
                let key = SizeKey::from_leg(leg);
                self.get_size(&key).map(|c| now - c.computed_at)
            })
            .max()
    }
}

/// Simple in-memory size calculator implementation
///
/// Stores pre-computed sizes in a HashMap.
/// In production, this would be updated by a background task.
pub struct InMemorySizeCalculator {
    sizes: HashMap<SizeKey, ComputedSize>,
}

impl InMemorySizeCalculator {
    pub fn new() -> Self {
        Self {
            sizes: HashMap::new(),
        }
    }

    /// Update or insert a computed size
    pub fn set_size(&mut self, size: ComputedSize) {
        let key = SizeKey::new(size.platform, &size.market_id, size.side);
        self.sizes.insert(key, size);
    }

    /// Remove a computed size
    pub fn remove_size(&mut self, key: &SizeKey) {
        self.sizes.remove(key);
    }

    /// Clear all computed sizes
    pub fn clear(&mut self) {
        self.sizes.clear();
    }

    /// Get number of cached sizes
    pub fn len(&self) -> usize {
        self.sizes.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.sizes.is_empty()
    }
}

impl Default for InMemorySizeCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl SizeCalculator for InMemorySizeCalculator {
    fn get_size(&self, key: &SizeKey) -> Option<ComputedSize> {
        self.sizes.get(key).cloned()
    }
}

/// Boxed size calculator for dynamic dispatch
pub type BoxedSizeCalculator = Box<dyn SizeCalculator>;
