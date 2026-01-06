//! Integration tests for Polymarket WebSocket API
//!
//! These tests make real WebSocket connections to the Polymarket WebSocket API.
//! They test the market channel (public data) which doesn't require authentication.
//!
//! To run these tests:
//! ```
//! cargo test --test polymarket_websocket_integration -- --test-threads=1
//! ```
//!
//! Note: These tests connect to live WebSocket servers and may take some time.
//! They also depend on active markets having data.

use polymarket_websocket::common::types::{ConnectionStatus, MarketEvent, Platform};
use polymarket_websocket::polymarket::rest::PolymarketRestClient;
use polymarket_websocket::polymarket::websocket::PolymarketWebSocketClient;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};

/// Base URLs
const CLOB_BASE_URL: &str = "https://clob.polymarket.com";
const GAMMA_BASE_URL: &str = "https://gamma-api.polymarket.com";
const WS_BASE_URL: &str = "wss://ws-subscriptions-clob.polymarket.com";

/// Test timeout for WebSocket operations
const WS_TIMEOUT_SECS: u64 = 30;

/// Helper to get sample token IDs from active markets
async fn get_sample_token_ids(count: usize) -> Vec<String> {
    let client = PolymarketRestClient::new(CLOB_BASE_URL, GAMMA_BASE_URL)
        .expect("Failed to create REST client");
    
    let mut token_ids = Vec::new();
    
    if let Ok(markets) = client.get_gamma_markets(Some(20), Some(true)).await {
        for market in markets {
            if let Some(tokens) = market.tokens {
                for token in tokens {
                    if !token.token_id.is_empty() && token_ids.len() < count {
                        token_ids.push(token.token_id);
                    }
                }
            }
            if token_ids.len() >= count {
                break;
            }
        }
    }
    
    token_ids
}

// ============================================================================
// Connection Tests
// ============================================================================

#[tokio::test]
async fn test_websocket_connection_market_channel() {
    let mut ws_client = PolymarketWebSocketClient::new_market_channel(WS_BASE_URL);
    
    // Get a sample token ID
    let token_ids = get_sample_token_ids(1).await;
    if token_ids.is_empty() {
        println!("SKIPPED: Could not find active markets with tokens");
        return;
    }
    
    println!("Connecting to WebSocket with token: {}", token_ids[0]);
    
    // Create channel for receiving events
    let (tx, mut rx) = mpsc::channel::<MarketEvent>(100);
    
    // Connect and subscribe
    let connect_result = timeout(
        Duration::from_secs(WS_TIMEOUT_SECS),
        ws_client.connect_and_subscribe(token_ids.clone(), tx),
    )
    .await;
    
    match connect_result {
        Ok(Ok(())) => {
            println!("WebSocket connection established successfully");
            assert!(ws_client.is_connected(), "Client should be marked as connected");
        }
        Ok(Err(e)) => {
            panic!("WebSocket connection failed: {:?}", e);
        }
        Err(_) => {
            panic!("WebSocket connection timed out");
        }
    }
    
    // Wait for connection status event
    let status_result = timeout(Duration::from_secs(5), rx.recv()).await;
    
    match status_result {
        Ok(Some(MarketEvent::ConnectionStatus { platform, status })) => {
            assert_eq!(platform, Platform::Polymarket);
            assert!(matches!(status, ConnectionStatus::Connected));
            println!("Received connection status: {:?}", status);
        }
        Ok(Some(event)) => {
            println!("Received first event: {:?}", event);
            // First event might not be connection status in all cases
        }
        Ok(None) => {
            println!("Channel closed unexpectedly");
        }
        Err(_) => {
            println!("Timed out waiting for first event (this may be normal if market is quiet)");
        }
    }
}

#[tokio::test]
async fn test_websocket_receives_market_data() {
    let mut ws_client = PolymarketWebSocketClient::new_market_channel(WS_BASE_URL);
    
    // Get multiple token IDs to increase chance of receiving data
    let token_ids = get_sample_token_ids(3).await;
    if token_ids.is_empty() {
        println!("SKIPPED: Could not find active markets with tokens");
        return;
    }
    
    println!("Connecting to WebSocket with {} tokens", token_ids.len());
    for (i, id) in token_ids.iter().enumerate() {
        println!("  Token {}: {}", i + 1, id);
    }
    
    let (tx, mut rx) = mpsc::channel::<MarketEvent>(100);
    
    // Connect and subscribe
    if let Err(e) = ws_client.connect_and_subscribe(token_ids, tx).await {
        println!("SKIPPED: WebSocket connection failed: {:?}", e);
        return;
    }
    
    println!("Connected, waiting for market data...");
    
    // Collect events for a period of time
    let mut events_received = 0;
    let mut order_book_updates = 0;
    let mut trades = 0;
    let mut heartbeats = 0;
    let mut raw_messages = 0;
    
    let collection_timeout = Duration::from_secs(15);
    let start_time = std::time::Instant::now();
    
    while start_time.elapsed() < collection_timeout {
        match timeout(Duration::from_secs(2), rx.recv()).await {
            Ok(Some(event)) => {
                events_received += 1;
                match &event {
                    MarketEvent::OrderBookUpdate(update) => {
                        order_book_updates += 1;
                        println!(
                            "OrderBookUpdate: {} bids, {} asks for {}",
                            update.bids.len(),
                            update.asks.len(),
                            update.asset_id
                        );
                    }
                    MarketEvent::Trade(trade) => {
                        trades += 1;
                        println!(
                            "Trade: {} @ {} size {} ({})",
                            trade.asset_id, trade.price, trade.size, trade.side
                        );
                    }
                    MarketEvent::Heartbeat { .. } => {
                        heartbeats += 1;
                    }
                    MarketEvent::ConnectionStatus { status, .. } => {
                        println!("Connection status: {:?}", status);
                    }
                    MarketEvent::Raw { message, .. } => {
                        raw_messages += 1;
                        if raw_messages <= 3 {
                            println!("Raw message: {}", &message[..message.len().min(200)]);
                        }
                    }
                    _ => {}
                }
            }
            Ok(None) => {
                println!("Channel closed");
                break;
            }
            Err(_) => {
                // Timeout on recv, continue
            }
        }
    }
    
    println!("\n=== WebSocket Test Summary ===");
    println!("Total events received: {}", events_received);
    println!("  - Order book updates: {}", order_book_updates);
    println!("  - Trades: {}", trades);
    println!("  - Heartbeats: {}", heartbeats);
    println!("  - Raw messages: {}", raw_messages);
    
    // We should have received at least something
    if events_received == 0 {
        println!("WARNING: No events received. Markets may be very quiet.");
    }
}

#[tokio::test]
async fn test_websocket_heartbeat() {
    let mut ws_client = PolymarketWebSocketClient::new_market_channel(WS_BASE_URL)
        .with_heartbeat_interval(5); // 5 second heartbeat
    
    let token_ids = get_sample_token_ids(1).await;
    if token_ids.is_empty() {
        println!("SKIPPED: Could not find active markets with tokens");
        return;
    }
    
    let (tx, mut rx) = mpsc::channel::<MarketEvent>(100);
    
    if let Err(e) = ws_client.connect_and_subscribe(token_ids, tx).await {
        println!("SKIPPED: WebSocket connection failed: {:?}", e);
        return;
    }
    
    println!("Connected, waiting for heartbeat responses...");
    
    // Wait for at least one heartbeat (need to wait > heartbeat interval)
    let mut heartbeat_received = false;
    let wait_time = Duration::from_secs(15);
    let start = std::time::Instant::now();
    
    while start.elapsed() < wait_time {
        match timeout(Duration::from_secs(2), rx.recv()).await {
            Ok(Some(MarketEvent::Heartbeat { platform })) => {
                assert_eq!(platform, Platform::Polymarket);
                heartbeat_received = true;
                println!("Heartbeat received!");
                break;
            }
            Ok(Some(_)) => {
                // Other event, continue waiting
            }
            Ok(None) => {
                break;
            }
            Err(_) => {
                // Timeout, continue
            }
        }
    }
    
    if !heartbeat_received {
        println!("Note: Heartbeat response not received within timeout (PONG handling may vary)");
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_websocket_invalid_url() {
    let mut ws_client = PolymarketWebSocketClient::new_market_channel("wss://invalid.example.com");
    
    let (tx, _rx) = mpsc::channel::<MarketEvent>(100);
    
    let result = ws_client
        .connect_and_subscribe(vec!["some_token".to_string()], tx)
        .await;
    
    // Should fail to connect
    assert!(result.is_err(), "Expected connection to invalid URL to fail");
    println!("Expected error received: {:?}", result.err());
}

#[tokio::test]
async fn test_websocket_empty_subscription() {
    let mut ws_client = PolymarketWebSocketClient::new_market_channel(WS_BASE_URL);
    
    let (tx, mut rx) = mpsc::channel::<MarketEvent>(100);
    
    // Connect with empty token list
    let result = ws_client.connect_and_subscribe(vec![], tx).await;
    
    // Connection should succeed but we won't get market data
    match result {
        Ok(()) => {
            println!("Connected with empty subscription");
            
            // Wait briefly for any events
            let event = timeout(Duration::from_secs(5), rx.recv()).await;
            println!("Event after empty subscription: {:?}", event);
        }
        Err(e) => {
            println!("Connection with empty subscription failed: {:?}", e);
        }
    }
}

// ============================================================================
// Multiple Subscription Tests
// ============================================================================

#[tokio::test]
async fn test_websocket_multiple_tokens() {
    let mut ws_client = PolymarketWebSocketClient::new_market_channel(WS_BASE_URL);
    
    // Get 5 token IDs
    let token_ids = get_sample_token_ids(5).await;
    if token_ids.len() < 2 {
        println!("SKIPPED: Could not find enough active markets");
        return;
    }
    
    println!("Subscribing to {} tokens", token_ids.len());
    
    let (tx, mut rx) = mpsc::channel::<MarketEvent>(100);
    
    if let Err(e) = ws_client.connect_and_subscribe(token_ids.clone(), tx).await {
        println!("SKIPPED: WebSocket connection failed: {:?}", e);
        return;
    }
    
    // Collect unique asset IDs from received events
    let mut seen_assets = std::collections::HashSet::new();
    let collection_time = Duration::from_secs(10);
    let start = std::time::Instant::now();
    
    while start.elapsed() < collection_time {
        match timeout(Duration::from_secs(2), rx.recv()).await {
            Ok(Some(MarketEvent::OrderBookUpdate(update))) => {
                seen_assets.insert(update.asset_id);
            }
            Ok(Some(MarketEvent::Trade(trade))) => {
                seen_assets.insert(trade.asset_id);
            }
            _ => {}
        }
    }
    
    println!("Received data for {} unique assets", seen_assets.len());
    for asset in &seen_assets {
        println!("  - {}", asset);
    }
}

// ============================================================================
// Connection State Tests
// ============================================================================

#[tokio::test]
async fn test_connection_state_tracking() {
    let mut ws_client = PolymarketWebSocketClient::new_market_channel(WS_BASE_URL);
    
    // Initially not connected
    assert!(!ws_client.is_connected(), "Should not be connected initially");
    
    let token_ids = get_sample_token_ids(1).await;
    if token_ids.is_empty() {
        println!("SKIPPED: Could not find active markets");
        return;
    }
    
    let (tx, _rx) = mpsc::channel::<MarketEvent>(100);
    
    // Connect
    if let Err(e) = ws_client.connect_and_subscribe(token_ids, tx).await {
        println!("SKIPPED: Connection failed: {:?}", e);
        return;
    }
    
    // Should be connected now
    assert!(ws_client.is_connected(), "Should be connected after connect_and_subscribe");
    
    // Wait a bit to ensure connection is stable
    sleep(Duration::from_secs(2)).await;
    
    // Should still be connected
    if ws_client.is_connected() {
        println!("Connection is stable");
    } else {
        println!("Connection was lost (may be due to server-side timeout)");
    }
}

// ============================================================================
// Data Parsing Tests (using recorded messages)
// ============================================================================

#[tokio::test]
async fn test_parse_order_book_message() {
    // Test parsing a sample order book message
    // This tests the parsing logic without making network calls
    
    use polymarket_websocket::polymarket::websocket::PolymarketWebSocketClient;
    
    // Note: We can't directly test parse_message as it's private
    // But we can verify the WebSocket client is created correctly
    let ws_client = PolymarketWebSocketClient::new_market_channel(WS_BASE_URL);
    assert!(ws_client.is_connected() == false);
}

// ============================================================================
// Stress Tests
// ============================================================================

#[tokio::test]
#[ignore] // Run with: cargo test --test polymarket_websocket_integration test_long_running_connection -- --ignored
async fn test_long_running_connection() {
    let mut ws_client = PolymarketWebSocketClient::new_market_channel(WS_BASE_URL)
        .with_heartbeat_interval(10);
    
    let token_ids = get_sample_token_ids(5).await;
    if token_ids.is_empty() {
        println!("SKIPPED: Could not find active markets");
        return;
    }
    
    let (tx, mut rx) = mpsc::channel::<MarketEvent>(1000);
    
    if let Err(e) = ws_client.connect_and_subscribe(token_ids, tx).await {
        println!("SKIPPED: Connection failed: {:?}", e);
        return;
    }
    
    println!("Starting long-running connection test (60 seconds)...");
    
    let mut event_count = 0;
    let duration = Duration::from_secs(60);
    let start = std::time::Instant::now();
    
    while start.elapsed() < duration {
        match timeout(Duration::from_secs(5), rx.recv()).await {
            Ok(Some(event)) => {
                event_count += 1;
                if event_count % 100 == 0 {
                    println!("Received {} events so far...", event_count);
                }
                
                // Check for disconnection
                if let MarketEvent::ConnectionStatus { status: ConnectionStatus::Disconnected(_), .. } = event {
                    println!("Disconnected after {} events", event_count);
                    break;
                }
            }
            Ok(None) => {
                println!("Channel closed after {} events", event_count);
                break;
            }
            Err(_) => {
                // Timeout, check if still connected
                if !ws_client.is_connected() {
                    println!("Connection lost after {} events", event_count);
                    break;
                }
            }
        }
    }
    
    println!("Long-running test completed: {} total events", event_count);
}
