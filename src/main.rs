//! PolymarketWebsocket - Main Entry Point
//!
//! A Rust application that connects to Polymarket and Kalshi websockets
//! for real-time market data consumption.

use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// CLI arguments for the application
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Comma-separated list of Polymarket market IDs to subscribe
    #[arg(long)]
    polymarket_markets: Option<String>,

    /// Comma-separated list of Kalshi tickers to subscribe
    #[arg(long)]
    kalshi_markets: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    let level = match args.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting PolymarketWebsocket application");
    info!("Configuration file: {}", args.config);

    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // TODO: Initialize configuration, clients, and decision engine
    // This will be implemented as the project grows

    info!("Application initialized successfully");

    // Keep the application running
    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal, cleaning up...");

    Ok(())
}
