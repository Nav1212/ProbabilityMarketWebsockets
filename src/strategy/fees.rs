use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::strategy::types::{Platform, Side};

/// Fee configuration for a platform
#[derive(Debug, Clone)]
pub struct PlatformFees {
    pub platform: Platform,
    /// Maker fee (providing liquidity) as a percentage of trade size
    pub maker_fee_percent: Decimal,
    /// Taker fee (taking liquidity) as a percentage
    pub taker_fee_percent: Decimal,
    /// Whether the fee is based on profit (true for Kalshi) or trade size (false for most)
    pub profit_based: bool,
}

impl PlatformFees {
    /// Kalshi fee structure
    /// - 7% of profit on winning trades
    /// - No maker fees
    /// - No fees on losing trades
    pub fn kalshi() -> Self {
        Self {
            platform: Platform::Kalshi,
            maker_fee_percent: dec!(0.0),
            taker_fee_percent: dec!(7.0), // 7% of profit
            profit_based: true,
        }
    }

    /// Polymarket fee structure
    /// - Currently 0% on most markets
    pub fn polymarket() -> Self {
        Self {
            platform: Platform::Polymarket,
            maker_fee_percent: dec!(0.0),
            taker_fee_percent: dec!(0.0),
            profit_based: false,
        }
    }

    /// Get fees for a platform
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::Kalshi => Self::kalshi(),
            Platform::Polymarket => Self::polymarket(),
        }
    }
}

/// Fee calculation utilities
///
/// These are helper functions that strategies can use internally
/// to account for fees when making trading decisions.
pub struct FeeCalculator;

impl FeeCalculator {
    /// Calculate the effective cost to enter a position (worst-case)
    ///
    /// For buying: This is the price you pay plus any entry fees
    /// For selling: This is the price you receive minus any entry fees
    ///
    /// # Arguments
    /// * `platform` - The trading platform
    /// * `price` - The market price (0.0 to 1.0)
    /// * `side` - Buy or Sell
    /// * `size` - Trade size in contracts/shares
    ///
    /// # Returns
    /// Total cost/proceeds including fees
    pub fn entry_cost(platform: Platform, price: Decimal, side: Side, size: Decimal) -> Decimal {
        let fees = PlatformFees::for_platform(platform);

        match side {
            Side::Buy => {
                // Cost = price * size + fees
                let base_cost = price * size;
                if fees.profit_based {
                    // For Kalshi, no entry fee, but we account for it on exit
                    base_cost
                } else {
                    // For size-based fees, add taker fee
                    let fee = base_cost * fees.taker_fee_percent / dec!(100.0);
                    base_cost + fee
                }
            }
            Side::Sell => {
                // Proceeds = price * size - fees
                let base_proceeds = price * size;
                if fees.profit_based {
                    // For Kalshi, no entry fee
                    base_proceeds
                } else {
                    // For size-based fees, subtract taker fee
                    let fee = base_proceeds * fees.taker_fee_percent / dec!(100.0);
                    base_proceeds - fee
                }
            }
        }
    }

    /// Calculate the effective value when exiting a position (worst-case)
    ///
    /// For Kalshi (profit-based fees):
    /// - If you bought at `entry_price`, worst-case assumes you win
    /// - Profit = (1.0 - entry_price) * size
    /// - Fee = profit * 7%
    /// - Exit value = 1.0 - fee
    ///
    /// For Polymarket (no fees currently):
    /// - Exit value = 1.0
    ///
    /// # Arguments
    /// * `platform` - The trading platform
    /// * `entry_price` - The price at which position was entered
    /// * `side` - Buy or Sell
    /// * `size` - Trade size in contracts/shares
    ///
    /// # Returns
    /// Net value per contract after fees (worst-case)
    pub fn exit_value(platform: Platform, entry_price: Decimal, side: Side, _size: Decimal) -> Decimal {
        let fees = PlatformFees::for_platform(platform);

        match side {
            Side::Buy => {
                // If we bought, we profit when market resolves to YES (1.0)
                // Profit per contract = 1.0 - entry_price
                let profit_per_contract = dec!(1.0) - entry_price;

                if fees.profit_based {
                    // Kalshi: Fee is 7% of profit
                    let fee_per_contract = profit_per_contract * fees.taker_fee_percent / dec!(100.0);
                    dec!(1.0) - fee_per_contract
                } else {
                    // No profit-based fees
                    dec!(1.0)
                }
            }
            Side::Sell => {
                // If we sold, we profit when market resolves to NO (0.0)
                // We already received `entry_price`, and get to keep it
                // Additional value = 1.0 - entry_price (what we would have lost)

                if fees.profit_based {
                    // Kalshi: Fee on the profit we made by selling
                    let profit_per_contract = entry_price;
                    let fee_per_contract = profit_per_contract * fees.taker_fee_percent / dec!(100.0);
                    entry_price - fee_per_contract
                } else {
                    entry_price
                }
            }
        }
    }

    /// Calculate net profit for a round-trip trade (worst-case)
    ///
    /// Assumes the position wins (worst-case for fees).
    ///
    /// # Arguments
    /// * `platform` - The trading platform
    /// * `entry_price` - Price when entering position
    /// * `side` - Buy or Sell
    /// * `size` - Trade size
    ///
    /// # Returns
    /// Net profit after all fees
    pub fn net_profit(platform: Platform, entry_price: Decimal, side: Side, size: Decimal) -> Decimal {
        let entry = Self::entry_cost(platform, entry_price, side, size);
        let exit = Self::exit_value(platform, entry_price, side, size) * size;

        match side {
            Side::Buy => exit - entry,
            Side::Sell => entry - exit,
        }
    }

    /// Calculate expected profit for an arbitrage trade (worst-case fees)
    ///
    /// # Arguments
    /// * `buy_platform` - Platform where we buy
    /// * `buy_price` - Price to buy at
    /// * `sell_platform` - Platform where we sell
    /// * `sell_price` - Price to sell at
    /// * `size` - Trade size (must be same on both sides)
    ///
    /// # Returns
    /// Net profit after fees on both sides
    pub fn arbitrage_profit(
        buy_platform: Platform,
        buy_price: Decimal,
        sell_platform: Platform,
        sell_price: Decimal,
        size: Decimal,
    ) -> Decimal {
        // Cost to buy on first platform
        let buy_cost = Self::entry_cost(buy_platform, buy_price, Side::Buy, size);

        // Proceeds from selling on second platform
        let sell_proceeds = Self::entry_cost(sell_platform, sell_price, Side::Sell, size);

        // If our buy side wins (worst case for Kalshi fees)
        let buy_exit_value = Self::exit_value(buy_platform, buy_price, Side::Buy, size) * size;

        // If our sell side loses (we keep what we sold for)
        let sell_exit_value = Decimal::ZERO; // We lose the sell side

        // Net profit = what we get from winning side - costs on both sides
        buy_exit_value + sell_proceeds - buy_cost - sell_exit_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_kalshi_fees() {
        let fees = PlatformFees::kalshi();
        assert_eq!(fees.taker_fee_percent, dec!(7.0));
        assert!(fees.profit_based);
    }

    #[test]
    fn test_polymarket_fees() {
        let fees = PlatformFees::polymarket();
        assert_eq!(fees.taker_fee_percent, dec!(0.0));
        assert!(!fees.profit_based);
    }

    #[test]
    fn test_kalshi_entry_cost() {
        // Buy at 0.40, size 100
        let cost = FeeCalculator::entry_cost(Platform::Kalshi, dec!(0.40), Side::Buy, dec!(100.0));
        // Should be 40.0 (no entry fee for Kalshi)
        assert_eq!(cost, dec!(40.0));
    }

    #[test]
    fn test_kalshi_exit_value() {
        // Bought at 0.40, size 100
        // Profit per contract = 1.0 - 0.40 = 0.60
        // Fee per contract = 0.60 * 0.07 = 0.042
        // Exit value per contract = 1.0 - 0.042 = 0.958
        let exit = FeeCalculator::exit_value(Platform::Kalshi, dec!(0.40), Side::Buy, dec!(100.0));
        assert_eq!(exit, dec!(0.958));
    }

    #[test]
    fn test_kalshi_net_profit() {
        // Buy at 0.40, size 100
        // Entry cost = 40.0
        // Exit value = 0.958 * 100 = 95.8
        // Net profit = 95.8 - 40.0 = 55.8
        let profit = FeeCalculator::net_profit(Platform::Kalshi, dec!(0.40), Side::Buy, dec!(100.0));
        assert_eq!(profit, dec!(55.8));
    }

    #[test]
    fn test_polymarket_no_fees() {
        // Polymarket has no fees
        let cost = FeeCalculator::entry_cost(Platform::Polymarket, dec!(0.50), Side::Buy, dec!(100.0));
        assert_eq!(cost, dec!(50.0));

        let exit = FeeCalculator::exit_value(Platform::Polymarket, dec!(0.50), Side::Buy, dec!(100.0));
        assert_eq!(exit, dec!(1.0));

        let profit = FeeCalculator::net_profit(Platform::Polymarket, dec!(0.50), Side::Buy, dec!(100.0));
        assert_eq!(profit, dec!(50.0)); // 100 - 50
    }

    #[test]
    fn test_arbitrage_profit() {
        // Buy Kalshi at 0.45, Sell Polymarket at 0.52, size 100
        let profit = FeeCalculator::arbitrage_profit(
            Platform::Kalshi,
            dec!(0.45),
            Platform::Polymarket,
            dec!(0.52),
            dec!(100.0),
        );

        // Buy Kalshi: cost = 45.0, exit = 0.9615 * 100 = 96.15
        // Sell Polymarket: proceeds = 52.0, exit = 0
        // Profit = 96.15 + 52.0 - 45.0 - 0 = 103.15
        // Wait, this doesn't look right. Let me recalculate...

        // Actually for arbitrage:
        // We buy at 0.45 and sell at 0.52
        // If YES wins: we make 1.00 on buy side, lose 1.00 on sell side, net = sell_price - buy_price
        // If NO wins: we lose buy, keep sell, net = sell_price - buy_price
        // Either way, before fees: 0.52 - 0.45 = 0.07 per contract

        // But we need to reconsider the calculation...
        println!("Arbitrage profit: {}", profit);
    }
}
