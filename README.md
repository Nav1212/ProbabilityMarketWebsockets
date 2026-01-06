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
├── config/           # Configuration management
├── polymarket/       # Polymarket websocket client
├── kalshi/           # Kalshi websocket client
├── common/           # Shared types and traits
├── decision/         # Decision-making engine
└── db/               # External database integration
```

## License

[Add your license here]
