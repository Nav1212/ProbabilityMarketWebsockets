# PolymarketWebsocket

A Rust application that connects to Polymarket and Kalshi websockets for real-time market data consumption, enabling live trading and research/analysis workflows.

## Features

- **Dual-Platform Support**: Simultaneous websocket connections to both Polymarket and Kalshi
- **Real-Time Data Processing**: Stream all available market data including order books, trades, and market updates
- **Decision Engine Integration**: Make trading and research decisions based on live data and external database references
- **Unified Data Model**: Common abstractions across platforms for consistent data handling
- **Automatic Reconnection**: Resilient connection management with automatic reconnection on failures

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Polymarket API credentials
- Kalshi API credentials
- Access to your external decision-making database

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd PolymarketWebsocket

# Build the project
cargo build --release

# Run the application
cargo run --release
```

## Configuration

Create a `.env` file or set the following environment variables:

```env
# Polymarket Configuration
POLYMARKET_API_KEY=your_polymarket_api_key
POLYMARKET_API_SECRET=your_polymarket_api_secret

# Kalshi Configuration
KALSHI_API_KEY=your_kalshi_api_key
KALSHI_API_SECRET=your_kalshi_api_secret

# External Database (for decision-making reference)
DATABASE_URL=postgresql://user:password@host:port/database

# Optional Settings
LOG_LEVEL=info
RECONNECT_DELAY_MS=5000
```

Alternatively, create a `config.toml` file:

```toml
[polymarket]
api_key = "your_polymarket_api_key"
api_secret = "your_polymarket_api_secret"
websocket_url = "wss://ws-subscriptions-clob.polymarket.com/ws/market"

[kalshi]
api_key = "your_kalshi_api_key"
api_secret = "your_kalshi_api_secret"
websocket_url = "wss://trading-api.kalshi.com/trade-api/ws/v2"

[database]
url = "postgresql://user:password@host:port/database"

[settings]
log_level = "info"
reconnect_delay_ms = 5000
```

## Usage

### Basic Usage

```bash
# Run with default configuration
cargo run --release

# Run with custom config file
cargo run --release -- --config path/to/config.toml

# Run with specific markets
cargo run --release -- --polymarket-markets "market_id_1,market_id_2" --kalshi-markets "ticker_1,ticker_2"
```

### Command Line Options

```
OPTIONS:
    -c, --config <FILE>              Path to configuration file
    --polymarket-markets <IDS>       Comma-separated Polymarket market IDs to subscribe
    --kalshi-markets <TICKERS>       Comma-separated Kalshi tickers to subscribe
    --log-level <LEVEL>              Log level (trace, debug, info, warn, error)
    -h, --help                       Print help information
    -V, --version                    Print version information
```

## Supported Data Types

### Polymarket

| Data Type | Description |
|-----------|-------------|
| Order Book | Real-time bid/ask depth and price levels |
| Trades | Executed trades with price, size, and timestamp |
| Market Updates | Market status, outcome prices, and metadata changes |
| Price Snapshots | Current best bid/ask prices |

### Kalshi

| Data Type | Description |
|-----------|-------------|
| Order Book | Real-time bid/ask depth across all price levels |
| Trades | Trade executions with price and quantity |
| Market Updates | Market lifecycle events, settlement status |
| Ticker Updates | Real-time ticker price and volume changes |

## Project Structure

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed technical documentation.

```
src/
├── main.rs           # Entry point and orchestration
├── lib.rs            # Library exports
├── config/           # Configuration management
├── polymarket/       # Polymarket websocket client
├── kalshi/           # Kalshi websocket client
├── common/           # Shared types and traits
├── decision/         # Decision-making engine
└── db/               # External database integration

tests/
├── polymarket_rest_integration.rs      # REST API integration tests
├── polymarket_websocket_integration.rs # WebSocket integration tests
└── common/                             # Test utilities
```

## Testing

The project includes comprehensive integration tests for the Polymarket CLOB REST API and WebSocket.

### Running Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run only unit tests
cargo test --lib

# Run REST API integration tests
cargo test --test polymarket_rest_integration -- --test-threads=1

# Run WebSocket integration tests
cargo test --test polymarket_websocket_integration -- --test-threads=1

# Run with verbose output
cargo test -- --nocapture

# Run long-running stress tests (ignored by default)
cargo test --test polymarket_websocket_integration test_long_running_connection -- --ignored
```

### Integration Test Coverage

#### REST API Tests
- API health check
- Server time retrieval
- Market discovery (Gamma API)
- Price data (buy/sell/midpoint/spread)
- Order book retrieval
- Last trade price
- Error handling for invalid inputs
- Concurrent request handling
- Data consistency validation

#### WebSocket Tests
- Connection to market channel
- Receiving market data (order book updates, trades)
- Heartbeat/ping-pong handling
- Multiple token subscriptions
- Connection state tracking
- Error handling for invalid URLs
- Long-running connection stability (stress test)

### Test Notes

1. **Rate Limiting**: Use `--test-threads=1` to avoid hitting API rate limits
2. **Network Dependency**: Integration tests require internet access to Polymarket APIs
3. **Market Activity**: Some tests may show fewer events if markets are quiet
4. **Authentication**: Integration tests use only public endpoints (no API keys required)

## License

[Add your license here]
