//! Configuration types

use serde::{Deserialize, Serialize};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Polymarket-specific configuration
    pub polymarket: PolymarketConfig,
    /// Kalshi-specific configuration
    #[serde(default)]
    pub kalshi: Option<KalshiConfig>,
    /// Database configuration (optional)
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    /// General application settings
    #[serde(default)]
    pub settings: AppSettings,
}

/// Polymarket platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolymarketConfig {
    /// API key for authenticated requests
    #[serde(default)]
    pub api_key: Option<String>,
    /// API secret for signing requests
    #[serde(default)]
    pub api_secret: Option<String>,
    /// API passphrase
    #[serde(default)]
    pub api_passphrase: Option<String>,
    /// Base URL for the CLOB REST API
    #[serde(default = "default_polymarket_rest_url")]
    pub rest_url: String,
    /// WebSocket URL for real-time data
    #[serde(default = "default_polymarket_ws_url")]
    pub websocket_url: String,
    /// Gamma API URL for market discovery
    #[serde(default = "default_polymarket_gamma_url")]
    pub gamma_url: String,
    /// List of market/token IDs to subscribe to
    #[serde(default)]
    pub markets: Vec<String>,
}

impl Default for PolymarketConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_secret: None,
            api_passphrase: None,
            rest_url: default_polymarket_rest_url(),
            websocket_url: default_polymarket_ws_url(),
            gamma_url: default_polymarket_gamma_url(),
            markets: Vec::new(),
        }
    }
}

fn default_polymarket_rest_url() -> String {
    "https://clob.polymarket.com".to_string()
}

fn default_polymarket_ws_url() -> String {
    "wss://ws-subscriptions-clob.polymarket.com".to_string()
}

fn default_polymarket_gamma_url() -> String {
    "https://gamma-api.polymarket.com".to_string()
}

/// Kalshi platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KalshiConfig {
    /// API key for authenticated requests
    #[serde(default)]
    pub api_key: Option<String>,
    /// API secret for signing requests
    #[serde(default)]
    pub api_secret: Option<String>,
    /// Base URL for the REST API
    #[serde(default = "default_kalshi_rest_url")]
    pub rest_url: String,
    /// WebSocket URL for real-time data
    #[serde(default = "default_kalshi_ws_url")]
    pub websocket_url: String,
    /// List of tickers to subscribe to
    #[serde(default)]
    pub markets: Vec<String>,
}

fn default_kalshi_rest_url() -> String {
    "https://trading-api.kalshi.com/trade-api/v2".to_string()
}

fn default_kalshi_ws_url() -> String {
    "wss://trading-api.kalshi.com/trade-api/ws/v2".to_string()
}

/// Database configuration for the decision engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_seconds: u64,
}

fn default_max_connections() -> u32 {
    5
}

fn default_connection_timeout() -> u64 {
    30
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Delay between reconnection attempts in milliseconds
    #[serde(default = "default_reconnect_delay")]
    pub reconnect_delay_ms: u64,
    /// Maximum reconnection attempts (0 = infinite)
    #[serde(default)]
    pub max_reconnect_attempts: u32,
    /// Heartbeat/ping interval in seconds
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_seconds: u64,
    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout_seconds: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            reconnect_delay_ms: default_reconnect_delay(),
            max_reconnect_attempts: 0,
            heartbeat_interval_seconds: default_heartbeat_interval(),
            request_timeout_seconds: default_request_timeout(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_reconnect_delay() -> u64 {
    5000
}

fn default_heartbeat_interval() -> u64 {
    10
}

fn default_request_timeout() -> u64 {
    30
}

/// API credentials for authenticated requests
#[derive(Debug, Clone)]
pub struct ApiCredentials {
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: String,
}

impl ApiCredentials {
    pub fn new(api_key: String, api_secret: String, passphrase: String) -> Self {
        Self {
            api_key,
            api_secret,
            passphrase,
        }
    }
}
