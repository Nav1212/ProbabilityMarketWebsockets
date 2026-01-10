use crate::common::types::MarketEvent;
use crate::strategy::types::{Decision, MarketSubscription, StrategyContext};

/// Core strategy trait
///
/// Strategies receive market events and emit Go/NoGo decisions.
/// They manage their own internal state and use the provided context
/// for position/balance awareness.
///
/// # Implementation Notes
///
/// - `on_market_event` should be fast - no blocking I/O
/// - Internal state (price history, indicators) is owned by the strategy
/// - Position/balance info comes from StrategyContext (read-only)
/// - Size calculation is handled separately by SizeCalculator
///
/// # Example
///
/// ```ignore
/// struct MomentumStrategy {
///     price_history: VecDeque<Decimal>,
///     threshold: Decimal,
/// }
///
/// impl Strategy for MomentumStrategy {
///     fn name(&self) -> &str { "momentum" }
///
///     fn on_market_event(&mut self, event: &MarketEvent, ctx: &StrategyContext) -> Decision {
///         // Update internal state, check for signals
///         Decision::NoGo
///     }
///
///     fn subscribed_markets(&self) -> Vec<MarketSubscription> {
///         vec![MarketSubscription::AllMatchedPairs]
///     }
/// }
/// ```
pub trait Strategy: Send + Sync {
    /// Unique identifier for this strategy
    fn name(&self) -> &str;

    /// Called when new market data arrives
    ///
    /// # Arguments
    /// * `event` - The market event (price update, trade, etc.)
    /// * `ctx` - Read-only context with positions and balances
    ///
    /// # Returns
    /// * `Decision::NoGo` - No action
    /// * `Decision::Go(intent)` - Execute the trade intent
    fn on_market_event(&mut self, event: &MarketEvent, ctx: &StrategyContext) -> Decision;

    /// Called periodically for time-based logic
    ///
    /// Useful for:
    /// - Checking timeouts
    /// - Periodic rebalancing
    /// - Momentum calculations on fixed intervals
    ///
    /// Default implementation returns NoGo.
    fn on_tick(&mut self, _ctx: &StrategyContext) -> Decision {
        Decision::NoGo
    }

    /// Declare which markets this strategy cares about
    ///
    /// The Trader uses this to filter events before calling on_market_event.
    /// This avoids unnecessary processing for irrelevant events.
    fn subscribed_markets(&self) -> Vec<MarketSubscription>;

    /// Called once when strategy is registered with Trader
    ///
    /// Use for any initialization that requires async or context.
    /// Default implementation does nothing.
    fn on_register(&mut self, _ctx: &StrategyContext) {}

    /// Called when strategy is being removed or system is shutting down
    ///
    /// Use for cleanup. Default implementation does nothing.
    fn on_shutdown(&mut self) {}
}

/// Boxed strategy for dynamic dispatch
pub type BoxedStrategy = Box<dyn Strategy>;
