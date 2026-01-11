# TODO - Arbitrage System Implementation

## Architecture Decisions

### 1. Dual WebSocket Listener
- **Goal**: Listen to Kalshi and Polymarket simultaneously, compare prices, execute arbitrage trades
- **Rust chosen for**: Consistent latency (no GC), async performance with Tokio, memory safety
- **Network latency is the bottleneck**, not processing - but consistent low-latency processing still matters for arbitrage windows

### 2. Market Matching Strategy
- **Approach**: Hybrid LLM + Cache system
- **Offline (periodic)**:
  - Fetch markets from both platforms
  - Run LLM batch matching for unmatched pairs
  - Store results with confidence scores in database
- **Runtime**:
  - Only use cached mappings (instant HashMap lookup)
  - No LLM calls in hot path
- **Human approval required**: When LLM confidence > 95%
- **Rationale**: LLM provides semantic understanding ("Trump wins" vs "Trump 2024 Victory") without runtime latency cost

### 3. Database Choice: SQLite
- **Why SQLite over JSON**:
  - Query flexibility (filter by status, platform, confidence)
  - Safe concurrent access (multiple readers/writers)
  - Atomic transactions (no read-modify-write races)
  - Low complexity (rusqlite crate, single file, no server)
- **Scale**: ~500-1000 Kalshi markets, ~200-500 Polymarket markets, ~50-200 matched pairs
- **File**: `markets.db`

### 4. Database Schema
```sql
-- Individual markets from each platform
CREATE TABLE markets (
    id UUID PRIMARY KEY,
    platform TEXT NOT NULL,           -- 'kalshi' | 'polymarket'
    platform_market_id TEXT NOT NULL, -- their native ID
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,             -- 'active' | 'closed' | 'resolved'
    fetched_at TIMESTAMP,
    UNIQUE(platform, platform_market_id)
);

-- LLM-generated matches
CREATE TABLE market_matches (
    id UUID PRIMARY KEY,
    kalshi_market_id UUID REFERENCES markets(id),
    polymarket_market_id UUID REFERENCES markets(id),
    confidence DECIMAL,               -- LLM confidence score (0.0 - 1.0)
    match_reason TEXT,                -- LLM explanation for review
    status TEXT NOT NULL,             -- 'pending_review' | 'approved' | 'rejected'
    created_at TIMESTAMP,
    reviewed_at TIMESTAMP
);
```

### 5. Price Normalization
- Both platforms treated as Decimal in range 0.0 to 1.0
- Using `rust_decimal` crate for financial precision

### 6. Fee Handling
- **Approach**: Worst-case fee calculation
- **Implementation**: Fee logic lives inside strategy module as utility functions
- **Kalshi fees**: 7% of profit (profit-based), only on winning trades, no maker fees
- **Polymarket fees**: 0% on most markets (hardcoded for now)
- **Module created**: `src/strategy/fees.rs` with `FeeCalculator` utilities
- Strategies use `FeeCalculator::arbitrage_profit()` and related functions internally
- Fees are hardcoded per platform (can be made configurable later if needed)

### 7. Strategy System (Dependency Injection)
- **Pattern**: Trader class receives Strategy implementations via dependency injection
- **Strategy interface**: Returns Go/NoGo decisions with TradeIntent
- **TradeIntent**: Contains one or more TradeLeg (for single trades or arbitrage pairs)
- **SizeCalculator**: Runs asynchronously, pre-computes optimal sizes
- **Trader**: Grabs pre-computed sizes and executes on Go signal
- **Module created**: `src/strategy/` with traits, types, and fee utilities
- Strategies manage their own state, receive StrategyContext for position/balance info

### 8. Order Execution
- Separate functions for each platform
- No coupling between Kalshi and Polymarket order logic
- Arbitrage executor calls both independently
- Execution strategy (simultaneous vs sequential) TBD

---

## Implementation Tasks

### Phase 1: Infrastructure
- [ ] Create SQLite database module
  - [ ] Schema creation/migration
  - [ ] Market CRUD operations
  - [ ] Match CRUD operations
  - [ ] Query helpers (active markets, pending reviews, approved matches)
- [ ] Create in-memory cache layer
  - [ ] Load approved matches on startup
  - [ ] HashMap for O(1) lookups by market ID
  - [ ] Cache invalidation when markets close

### Phase 2: Kalshi Integration
- [ ] Kalshi REST client (mirror Polymarket structure)
- [ ] Kalshi WebSocket client
- [ ] Kalshi authentication
- [ ] Kalshi message types

### Phase 3: Market Matching
- [ ] Market fetcher (periodic job)
- [ ] LLM integration for batch matching
- [ ] Confidence scoring
- [ ] Review workflow (export pending, import approved)

### Phase 4: Fee Calculation
- [x] Fee calculator utilities (FeeCalculator struct)
- [x] Polymarket fee implementation (0%)
- [x] Kalshi fee implementation (7% profit-based)
- [x] Effective price calculation (entry cost, exit value, net profit)
- [x] Arbitrage profit calculation (worst-case fees)

### Phase 5: Arbitrage Detection
- [ ] Event aggregator (maintain latest prices)
- [ ] Price comparison logic
- [ ] Profit calculation (accounting for fees)
- [ ] Opportunity threshold configuration

### Phase 6: Order Execution
- [ ] Polymarket order placement
- [ ] Kalshi order placement
- [ ] Order result handling
- [ ] Risk management / position limits

### Phase 7: Main Orchestration
- [ ] Wire up WebSocket clients
- [ ] Event routing to aggregator
- [ ] Arbitrage detector integration
- [ ] Graceful shutdown
- [ ] Reconnection logic

---

## Open Questions
- [x] ~~Specific Kalshi fee structure~~ - 7% of profit, profit-based
- [x] ~~Specific Polymarket fee structure~~ - 0% for most markets (hardcoded)
- [x] ~~Fee calculation approach~~ - Worst-case
- [x] ~~Fee logic location~~ - Inside strategy module as utilities
- [ ] LLM provider choice for market matching
- [ ] Confidence threshold tuning
- [ ] Execution strategy (simultaneous vs sequential orders)
- [ ] Position/risk limits

## Completed Modules
- ✅ `src/strategy/types.rs` - Core types (Decision, TradeIntent, TradeLeg, etc.)
- ✅ `src/strategy/traits.rs` - Strategy trait definition
- ✅ `src/strategy/size_calculator.rs` - SizeCalculator trait and implementation
- ✅ `src/strategy/fees.rs` - Fee calculation utilities
