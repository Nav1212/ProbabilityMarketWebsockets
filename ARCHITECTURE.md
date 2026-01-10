# Architecture

This document describes the technical architecture of the PolymarketWebsocket application.

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PolymarketWebsocket                            │
│                                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   Config     │  │   Logger     │  │   Metrics    │  │    CLI       │    │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Common Layer                                 │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │   │
│  │  │  MarketClient   │  │  Unified Types  │  │   Channels      │      │   │
│  │  │     Trait       │  │  (OrderBook,    │  │  (mpsc, async)  │      │   │
│  │  │                 │  │   Trade, etc.)  │  │                 │      │   │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────┐        ┌─────────────────────────┐            │
│  │   Polymarket Module     │        │     Kalshi Module       │            │
│  │  ┌───────────────────┐  │        │  ┌───────────────────┐  │            │
│  │  │  WebSocket Client │  │        │  │  WebSocket Client │  │            │
│  │  └───────────────────┘  │        │  └───────────────────┘  │            │
│  │  ┌───────────────────┐  │        │  ┌───────────────────┐  │            │
│  │  │  Message Parser   │  │        │  │  Message Parser   │  │            │
│  │  └───────────────────┘  │        │  └───────────────────┘  │            │
│  │  ┌───────────────────┐  │        │  ┌───────────────────┐  │            │
│  │  │  Event Handlers   │  │        │  │  Event Handlers   │  │            │
│  │  └───────────────────┘  │        │  └───────────────────┘  │            │
│  └─────────────────────────┘        └─────────────────────────┘            │
│               │                                  │                          │
│               └──────────────┬───────────────────┘                          │
│                              ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      Decision Engine                                 │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │   │
│  │  │  Event Router   │  │  Strategy Logic │  │  Action Emitter │      │   │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                              │
│                              ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    External Database Layer                           │   │
│  │  ┌─────────────────┐  ┌─────────────────┐                           │   │
│  │  │  DB Connection  │  │  Query Engine   │                           │   │
│  │  │     Pool        │  │                 │                           │   │
│  │  └─────────────────┘  └─────────────────┘                           │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                    │                                    │
                    ▼                                    ▼
         ┌──────────────────┐                 ┌──────────────────┐
         │    Polymarket    │                 │      Kalshi      │
         │    WebSocket     │                 │    WebSocket     │
         │      Server      │                 │      Server      │
         └──────────────────┘                 └──────────────────┘
```

## Module Structure

```
src/
├── main.rs                 # Application entry point
├── lib.rs                  # Library exports
│
├── config/
│   ├── mod.rs              # Configuration module root
│   ├── loader.rs           # Config file and env loading
│   └── types.rs            # Configuration structs
│
├── polymarket/
│   ├── mod.rs              # Polymarket module root
│   ├── client.rs           # WebSocket connection management
│   ├── messages.rs         # Polymarket-specific message types
│   ├── handlers.rs         # Message event handlers
│   └── auth.rs             # Authentication logic
│
├── kalshi/
│   ├── mod.rs              # Kalshi module root
│   ├── client.rs           # WebSocket connection management
│   ├── messages.rs         # Kalshi-specific message types
│   ├── handlers.rs         # Message event handlers
│   └── auth.rs             # Authentication logic
│
├── common/
│   ├── mod.rs              # Common module root
│   ├── traits.rs           # MarketClient trait, EventHandler trait
│   ├── types.rs            # Unified types (OrderBook, Trade, MarketUpdate)
│   ├── errors.rs           # Error types and handling
│   └── channels.rs         # Channel type definitions
│
├── decision/
│   ├── mod.rs              # Decision engine module root
│   ├── engine.rs           # Core decision-making logic
│   ├── router.rs           # Event routing to strategies
│   └── actions.rs          # Action types (signals, alerts, trades)
│
└── db/
    ├── mod.rs              # Database module root
    ├── client.rs           # Database connection and pooling
    └── queries.rs          # Query helpers for decision data

tests/
├── polymarket_rest_integration.rs      # REST API integration tests
├── polymarket_websocket_integration.rs # WebSocket integration tests
└── common/                             # Shared test utilities
    └── mod.rs
```

## Data Flow

```
┌────────────────┐     ┌────────────────┐
│   Polymarket   │     │     Kalshi     │
│   WebSocket    │     │   WebSocket    │
└───────┬────────┘     └───────┬────────┘
        │                      │
        ▼                      ▼
┌───────────────┐      ┌───────────────┐
│ Parse Message │      │ Parse Message │
│ (Platform-    │      │ (Platform-    │
│  specific)    │      │  specific)    │
└───────┬───────┘      └───────┬───────┘
        │                      │
        ▼                      ▼
┌───────────────┐      ┌───────────────┐
│ Convert to    │      │ Convert to    │
│ Unified Type  │      │ Unified Type  │
└───────┬───────┘      └───────┬───────┘
        │                      │
        └──────────┬───────────┘
                   │
                   ▼
         ┌─────────────────┐
         │  Event Channel  │
         │    (mpsc)       │
         └────────┬────────┘
                  │
                  ▼
         ┌─────────────────┐
         │ Decision Engine │◄────────┐
         │                 │         │
         └────────┬────────┘         │
                  │                  │
                  ▼                  │
         ┌─────────────────┐         │
         │ External DB     │─────────┘
         │ (Reference Data)│
         └────────┬────────┘
                  │
                  ▼
         ┌─────────────────┐
         │  Action Output  │
         │ (Signals/Trades)│
         └─────────────────┘
```

## Key Components

### MarketClient Trait

The `MarketClient` trait provides a unified interface for both platform clients:

```rust
#[async_trait]
pub trait MarketClient: Send + Sync {
    /// Connect to the websocket server
    async fn connect(&mut self) -> Result<(), ClientError>;

    /// Subscribe to specific markets
    async fn subscribe(&mut self, markets: &[String]) -> Result<(), ClientError>;

    /// Unsubscribe from markets
    async fn unsubscribe(&mut self, markets: &[String]) -> Result<(), ClientError>;

    /// Start receiving messages (spawns internal task)
    async fn start(&mut self, sender: mpsc::Sender<MarketEvent>) -> Result<(), ClientError>;

    /// Gracefully disconnect
    async fn disconnect(&mut self) -> Result<(), ClientError>;

    /// Check connection health
    fn is_connected(&self) -> bool;
}
```

### Unified Event Types

```rust
/// Unified market event from any platform
pub enum MarketEvent {
    OrderBookUpdate(OrderBookUpdate),
    Trade(Trade),
    MarketUpdate(MarketUpdate),
    ConnectionStatus(ConnectionStatus),
}

/// Source platform identifier
pub enum Platform {
    Polymarket,
    Kalshi,
}

/// Unified order book update
pub struct OrderBookUpdate {
    pub platform: Platform,
    pub market_id: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: DateTime<Utc>,
}

/// Unified trade
pub struct Trade {
    pub platform: Platform,
    pub market_id: String,
    pub price: Decimal,
    pub size: Decimal,
    pub side: Side,
    pub timestamp: DateTime<Utc>,
}
```

## Connection Lifecycle

```
┌─────────┐
│  Init   │
└────┬────┘
     │
     ▼
┌─────────────┐
│ Load Config │
└──────┬──────┘
       │
       ▼
┌──────────────┐
│ Authenticate │
└──────┬───────┘
       │
       ▼
┌─────────────────┐     ┌─────────────────┐
│ Connect WS      │────►│ Connection      │
│ (with retry)    │     │ Failed          │
└────────┬────────┘     └────────┬────────┘
         │                       │
         │                       │ (after delay)
         │◄──────────────────────┘
         │
         ▼
┌─────────────────┐
│ Subscribe to    │
│ Markets         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Receive Loop    │◄─────────┐
│ (process msgs)  │          │
└────────┬────────┘          │
         │                   │
         ├───────────────────┤
         │                   │
         ▼                   │
┌─────────────────┐          │
│ Handle Message  │──────────┘
└────────┬────────┘
         │
         ▼ (on disconnect)
┌─────────────────┐
│ Reconnect Logic │
│ (exp. backoff)  │
└─────────────────┘
```

## Concurrency Model

The application uses an async runtime with the following concurrent tasks:

1. **Main Task**: Orchestrates startup, shutdown, and signal handling
2. **Polymarket Client Task**: Manages Polymarket websocket connection and message parsing
3. **Kalshi Client Task**: Manages Kalshi websocket connection and message parsing
4. **Decision Engine Task**: Consumes events from both clients, queries external DB, emits actions
5. **Heartbeat Tasks**: Per-connection ping/pong for connection health

```
┌─────────────────────────────────────────────────────────────────┐
│                      Async Runtime                              │
│                                                                 │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐                │
│  │ Polymarket │  │   Kalshi   │  │  Decision  │                │
│  │   Task     │  │   Task     │  │   Task     │                │
│  └─────┬──────┘  └─────┬──────┘  └──────┬─────┘                │
│        │               │                │                       │
│        │   ┌───────────┴────────────┐   │                       │
│        └──►│    Event Channel       │◄──┘                       │
│            │      (mpsc)            │                           │
│            └────────────────────────┘                           │
│                                                                 │
│  ┌────────────┐  ┌────────────┐                                │
│  │ Heartbeat  │  │ Heartbeat  │                                │
│  │   (PM)     │  │  (Kalshi)  │                                │
│  └────────────┘  └────────────┘                                │
└─────────────────────────────────────────────────────────────────┘
```

## Error Handling Strategy

### Error Categories

| Category | Examples | Handling |
|----------|----------|----------|
| **Connection Errors** | Network timeout, DNS failure | Retry with exponential backoff |
| **Authentication Errors** | Invalid credentials, expired token | Log and exit (requires user intervention) |
| **Parse Errors** | Malformed JSON, unknown message type | Log warning, skip message, continue |
| **Rate Limit Errors** | Too many requests | Back off, respect retry-after header |
| **Application Errors** | DB query failure, logic error | Log error, attempt recovery or propagate |

### Retry Strategy

```rust
pub struct RetryConfig {
    pub initial_delay_ms: u64,      // 1000
    pub max_delay_ms: u64,          // 60000
    pub multiplier: f64,            // 2.0
    pub max_attempts: Option<u32>,  // None (infinite)
}
```

## External Database Integration

The decision engine queries an external database for reference data. This is a **read-only** integration.

```rust
pub struct DbClient {
    pool: Pool<Postgres>,
}

impl DbClient {
    /// Query historical data for decision making
    pub async fn get_market_history(&self, market_id: &str) -> Result<MarketHistory, DbError>;

    /// Query strategy parameters
    pub async fn get_strategy_params(&self, strategy_id: &str) -> Result<StrategyParams, DbError>;

    /// Query reference data
    pub async fn get_reference_data(&self, key: &str) -> Result<ReferenceData, DbError>;
}
```

The decision engine uses this data alongside real-time events to make trading and research decisions.

## Configuration Schema

```rust
pub struct AppConfig {
    pub polymarket: PolymarketConfig,
    pub kalshi: KalshiConfig,
    pub database: DatabaseConfig,
    pub settings: AppSettings,
}

pub struct PolymarketConfig {
    pub api_key: String,
    pub api_secret: String,
    pub websocket_url: String,
    pub markets: Vec<String>,
}

pub struct KalshiConfig {
    pub api_key: String,
    pub api_secret: String,
    pub websocket_url: String,
    pub markets: Vec<String>,
}

pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

pub struct AppSettings {
    pub log_level: String,
    pub reconnect_delay_ms: u64,
}
```

## Testing Architecture

The project uses a layered testing approach with both unit tests and integration tests.

### Test Structure

```
tests/
├── polymarket_rest_integration.rs      # REST API integration tests
├── polymarket_websocket_integration.rs # WebSocket integration tests
└── common/                             # Shared test utilities
    └── mod.rs
```

### Integration Test Categories

#### REST API Tests (`polymarket_rest_integration.rs`)

Tests for the Polymarket CLOB REST API:

| Test Category | Description |
|---------------|-------------|
| Health Checks | API availability and server time |
| Market Discovery | Fetching markets from Gamma API |
| Price Data | Buy/sell prices, midpoint, spread calculations |
| Order Book | Full order book retrieval and validation |
| Trade Data | Last trade price queries |
| Error Handling | Invalid input handling, edge cases |
| Concurrency | Parallel request handling |
| Data Consistency | Cross-endpoint data validation |

#### WebSocket Tests (`polymarket_websocket_integration.rs`)

Tests for real-time WebSocket connections:

| Test Category | Description |
|---------------|-------------|
| Connection | Establishing market channel connections |
| Data Streaming | Receiving order book updates and trades |
| Heartbeat | Ping/pong handling for connection health |
| Subscriptions | Multi-token subscription management |
| State Tracking | Connection state monitoring |
| Error Handling | Invalid URL and connection failure cases |
| Stress Tests | Long-running connection stability (ignored by default) |

### Test Execution Strategy

```
┌─────────────────────────────────────────────────────────────┐
│                      Test Pyramid                           │
│                                                             │
│                    ┌─────────────┐                          │
│                    │   E2E/      │  Few, slow, high value   │
│                    │   Stress    │                          │
│                    └──────┬──────┘                          │
│               ┌───────────┴───────────┐                     │
│               │    Integration Tests   │  Network-dependent │
│               │    (REST + WebSocket)  │                    │
│               └───────────┬───────────┘                     │
│          ┌────────────────┴────────────────┐                │
│          │          Unit Tests              │  Fast, isolated│
│          │   (Message parsing, logic, etc.) │                │
│          └──────────────────────────────────┘                │
└─────────────────────────────────────────────────────────────┘
```

### Test Configuration

- **Rate Limiting**: Tests run single-threaded (`--test-threads=1`) to respect API limits
- **Network Dependency**: Integration tests require internet access
- **No Authentication**: Public endpoints only (no API keys required for tests)
- **Stress Tests**: Long-running tests are `#[ignore]` by default

## Future Considerations

- **Horizontal Scaling**: Add support for multiple instances with market sharding
- **Metrics Export**: Prometheus/OpenTelemetry integration for monitoring
- **Additional Platforms**: Modular design allows adding new prediction market platforms
- **Replay Mode**: Support for replaying historical data for backtesting
- **Kalshi Integration Tests**: Add corresponding test coverage for Kalshi APIs
