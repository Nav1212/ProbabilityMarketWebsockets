//! Configuration loader

use config::{Config, Environment, File};
use std::path::Path;

use super::types::AppConfig;
use crate::common::errors::{ClientError, Result};

/// Load configuration from file and environment variables
///
/// Priority (highest to lowest):
/// 1. Environment variables (prefixed with APP_)
/// 2. Configuration file (TOML format)
/// 3. Default values
pub fn load_config(config_path: Option<&str>) -> Result<AppConfig> {
    let mut builder = Config::builder();

    // Add default config file if it exists
    if let Some(path) = config_path {
        if Path::new(path).exists() {
            builder = builder.add_source(File::with_name(path).required(false));
        }
    }

    // Add environment variables with APP_ prefix
    builder = builder.add_source(
        Environment::with_prefix("APP")
            .separator("__")
            .try_parsing(true),
    );

    // Also check for specific Polymarket env vars
    builder = builder.add_source(
        Environment::default()
            .prefix("POLYMARKET")
            .separator("_")
            .try_parsing(true),
    );

    let config = builder
        .build()
        .map_err(|e| ClientError::Configuration(e.to_string()))?;

    config
        .try_deserialize()
        .map_err(|e| ClientError::Configuration(e.to_string()))
}

/// Load configuration from environment variables only
pub fn load_from_env() -> Result<AppConfig> {
    // Try to load from .env file
    dotenvy::dotenv().ok();

    let polymarket_config = super::types::PolymarketConfig {
        api_key: std::env::var("POLYMARKET_API_KEY").ok(),
        api_secret: std::env::var("POLYMARKET_API_SECRET").ok(),
        api_passphrase: std::env::var("POLYMARKET_API_PASSPHRASE").ok(),
        rest_url: std::env::var("POLYMARKET_REST_URL")
            .unwrap_or_else(|_| "https://clob.polymarket.com".to_string()),
        websocket_url: std::env::var("POLYMARKET_WS_URL")
            .unwrap_or_else(|_| "wss://ws-subscriptions-clob.polymarket.com".to_string()),
        gamma_url: std::env::var("POLYMARKET_GAMMA_URL")
            .unwrap_or_else(|_| "https://gamma-api.polymarket.com".to_string()),
        markets: std::env::var("POLYMARKET_MARKETS")
            .map(|s| s.split(',').map(|m| m.trim().to_string()).collect())
            .unwrap_or_default(),
    };

    Ok(AppConfig {
        polymarket: polymarket_config,
        kalshi: None,
        database: None,
        settings: super::types::AppSettings::default(),
    })
}
