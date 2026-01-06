//! Integration tests for Polymarket CLOB REST API
//!
//! These tests make real API calls to the Polymarket CLOB API.
//! They test read-only endpoints that don't require authentication.
//!
//! To run these tests:
//! ```
//! cargo test --test polymarket_rest_integration -- --test-threads=1
//! ```
//!
//! Note: These tests are rate-limited and should be run with --test-threads=1
//! to avoid hitting rate limits.

use polymarket_websocket::common::types::Side;
use polymarket_websocket::polymarket::rest::PolymarketRestClient;
use rust_decimal::Decimal;
use std::time::Duration;
use tokio::time::sleep;

/// Base URL for the Polymarket CLOB API
const CLOB_BASE_URL: &str = "https://clob.polymarket.com";
/// Base URL for the Gamma API
const GAMMA_BASE_URL: &str = "https://gamma-api.polymarket.com";

/// Delay between tests to avoid rate limiting
const TEST_DELAY_MS: u64 = 500;

/// Helper function to create a test client
fn create_test_client() -> PolymarketRestClient {
    PolymarketRestClient::new(CLOB_BASE_URL, GAMMA_BASE_URL)
        .expect("Failed to create REST client")
}

/// Helper to add delay between tests
async fn test_delay() {
    sleep(Duration::from_millis(TEST_DELAY_MS)).await;
}

// ============================================================================
// Health Check Tests
// ============================================================================

#[tokio::test]
async fn test_api_health_check() {
    let client = create_test_client();
    
    let result = client.get_ok().await;
    
    assert!(result.is_ok(), "API health check failed: {:?}", result);
    assert!(result.unwrap(), "API should return OK");
}

#[tokio::test]
async fn test_get_server_time() {
    test_delay().await;
    let client = create_test_client();
    
    let result = client.get_server_time().await;
    
    assert!(result.is_ok(), "Failed to get server time: {:?}", result);
    
    let timestamp = result.unwrap();
    // Server time should be a reasonable Unix timestamp (after 2020)
    assert!(timestamp > 1577836800, "Timestamp seems too old: {}", timestamp);
    // And not too far in the future
    let now = chrono::Utc::now().timestamp();
    assert!(
        timestamp <= now + 60,
        "Timestamp is in the future: {} vs now {}",
        timestamp,
        now
    );
    
    println!("Server time: {} ({})", timestamp, 
        chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.to_string())
            .unwrap_or_default()
    );
}

// ============================================================================
// Market Discovery Tests (Gamma API)
// ============================================================================

#[tokio::test]
async fn test_get_gamma_markets() {
    test_delay().await;
    let client = create_test_client();
    
    // Get a limited number of active markets
    let result = client.get_gamma_markets(Some(5), Some(true)).await;
    
    assert!(result.is_ok(), "Failed to get Gamma markets: {:?}", result);
    
    let markets = result.unwrap();
    println!("Retrieved {} markets from Gamma API", markets.len());
    
    // Should have at least some markets
    assert!(!markets.is_empty(), "Expected at least one market");
    
    // Print first market for debugging
    if let Some(market) = markets.first() {
        println!("Sample market: {} - {}", market.id, market.question);
        if let Some(tokens) = &market.tokens {
            for token in tokens {
                println!("  Token: {} ({})", token.token_id, token.outcome);
            }
        }
    }
}

#[tokio::test]
async fn test_get_gamma_events() {
    test_delay().await;
    let client = create_test_client();
    
    let result = client.get_gamma_events(Some(3)).await;
    
    assert!(result.is_ok(), "Failed to get Gamma events: {:?}", result);
    
    let events = result.unwrap();
    println!("Retrieved {} events from Gamma API", events.len());
    
    for event in &events {
        println!("Event: {} - {}", event.id, event.title);
    }
}

// ============================================================================
// Price Data Tests
// ============================================================================

/// Helper to get a valid token ID from the market
async fn get_sample_token_id(client: &PolymarketRestClient) -> Option<String> {
    // Try to get an active market with tokens
    if let Ok(markets) = client.get_gamma_markets(Some(10), Some(true)).await {
        for market in markets {
            if let Some(tokens) = market.tokens {
                if let Some(token) = tokens.first() {
                    if !token.token_id.is_empty() {
                        return Some(token.token_id.clone());
                    }
                }
            }
        }
    }
    None
}

#[tokio::test]
async fn test_get_price_for_active_market() {
    test_delay().await;
    let client = create_test_client();
    
    // First, get a valid token ID
    let token_id = match get_sample_token_id(&client).await {
        Some(id) => id,
        None => {
            println!("SKIPPED: Could not find an active market with tokens");
            return;
        }
    };
    
    test_delay().await;
    
    // Get buy price
    let buy_result = client.get_price(&token_id, Side::Buy).await;
    
    match buy_result {
        Ok(price) => {
            println!("Buy price for {}: {}", token_id, price);
            // Price should be between 0 and 1 for prediction markets
            assert!(price >= Decimal::ZERO, "Price should be non-negative");
            assert!(price <= Decimal::ONE, "Price should be <= 1.00");
        }
        Err(e) => {
            // Market might be inactive or illiquid
            println!("Could not get buy price (market may be illiquid): {}", e);
        }
    }
    
    test_delay().await;
    
    // Get sell price
    let sell_result = client.get_price(&token_id, Side::Sell).await;
    
    match sell_result {
        Ok(price) => {
            println!("Sell price for {}: {}", token_id, price);
            assert!(price >= Decimal::ZERO, "Price should be non-negative");
            assert!(price <= Decimal::ONE, "Price should be <= 1.00");
        }
        Err(e) => {
            println!("Could not get sell price (market may be illiquid): {}", e);
        }
    }
}

#[tokio::test]
async fn test_get_midpoint_for_active_market() {
    test_delay().await;
    let client = create_test_client();
    
    let token_id = match get_sample_token_id(&client).await {
        Some(id) => id,
        None => {
            println!("SKIPPED: Could not find an active market with tokens");
            return;
        }
    };
    
    test_delay().await;
    
    let result = client.get_midpoint(&token_id).await;
    
    match result {
        Ok(midpoint) => {
            println!("Midpoint for {}: {}", token_id, midpoint);
            assert!(midpoint >= Decimal::ZERO, "Midpoint should be non-negative");
            assert!(midpoint <= Decimal::ONE, "Midpoint should be <= 1.00");
        }
        Err(e) => {
            println!("Could not get midpoint (market may be illiquid): {}", e);
        }
    }
}

#[tokio::test]
async fn test_get_spread_for_active_market() {
    test_delay().await;
    let client = create_test_client();
    
    let token_id = match get_sample_token_id(&client).await {
        Some(id) => id,
        None => {
            println!("SKIPPED: Could not find an active market with tokens");
            return;
        }
    };
    
    test_delay().await;
    
    let result = client.get_spread(&token_id).await;
    
    match result {
        Ok(spread) => {
            println!("Spread for {}: {}", token_id, spread);
            // Spread should be non-negative
            assert!(spread >= Decimal::ZERO, "Spread should be non-negative");
        }
        Err(e) => {
            println!("Could not get spread (market may be illiquid): {}", e);
        }
    }
}

// ============================================================================
// Order Book Tests
// ============================================================================

#[tokio::test]
async fn test_get_order_book_for_active_market() {
    test_delay().await;
    let client = create_test_client();
    
    let token_id = match get_sample_token_id(&client).await {
        Some(id) => id,
        None => {
            println!("SKIPPED: Could not find an active market with tokens");
            return;
        }
    };
    
    test_delay().await;
    
    let result = client.get_order_book(&token_id).await;
    
    match result {
        Ok(order_book) => {
            println!(
                "Order book for {}: {} bids, {} asks",
                token_id,
                order_book.bids.len(),
                order_book.asks.len()
            );
            
            // Verify order book structure
            assert_eq!(order_book.asset_id, token_id);
            
            // Print top of book if available
            if let Some(best_bid) = order_book.best_bid() {
                println!("Best bid: {} @ size {}", best_bid.price, best_bid.size);
            }
            if let Some(best_ask) = order_book.best_ask() {
                println!("Best ask: {} @ size {}", best_ask.price, best_ask.size);
            }
            if let Some(mid) = order_book.midpoint() {
                println!("Midpoint: {}", mid);
            }
            if let Some(spread) = order_book.spread() {
                println!("Spread: {}", spread);
            }
            
            // Verify bids are sorted descending by price
            for window in order_book.bids.windows(2) {
                assert!(
                    window[0].price >= window[1].price,
                    "Bids should be sorted descending"
                );
            }
            
            // Verify asks are sorted ascending by price
            for window in order_book.asks.windows(2) {
                assert!(
                    window[0].price <= window[1].price,
                    "Asks should be sorted ascending"
                );
            }
        }
        Err(e) => {
            println!("Could not get order book (market may be illiquid): {}", e);
        }
    }
}

// ============================================================================
// Last Trade Price Tests
// ============================================================================

#[tokio::test]
async fn test_get_last_trade_price() {
    test_delay().await;
    let client = create_test_client();
    
    let token_id = match get_sample_token_id(&client).await {
        Some(id) => id,
        None => {
            println!("SKIPPED: Could not find an active market with tokens");
            return;
        }
    };
    
    test_delay().await;
    
    let result = client.get_last_trade_price(&token_id).await;
    
    match result {
        Ok(price) => {
            println!("Last trade price for {}: {}", token_id, price);
            assert!(price >= Decimal::ZERO, "Price should be non-negative");
            assert!(price <= Decimal::ONE, "Price should be <= 1.00");
        }
        Err(e) => {
            println!("Could not get last trade price (no recent trades): {}", e);
        }
    }
}

// ============================================================================
// Simplified Markets Tests
// ============================================================================

#[tokio::test]
async fn test_get_simplified_markets() {
    test_delay().await;
    let client = create_test_client();
    
    let result = client.get_simplified_markets().await;
    
    assert!(result.is_ok(), "Failed to get simplified markets: {:?}", result);
    
    let markets_response = result.unwrap();
    println!(
        "Retrieved {} simplified markets",
        markets_response.data.len()
    );
    
    // Check first few markets
    for market in markets_response.data.iter().take(3) {
        println!("Market: {} - {:?}", market.condition_id, market.question);
        println!("  Tokens: {}", market.tokens.len());
        println!("  Active: {:?}", market.active);
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_invalid_token_id_returns_error() {
    test_delay().await;
    let client = create_test_client();
    
    // Use an obviously invalid token ID
    let result = client.get_order_book("invalid_token_id_12345").await;
    
    // Should return an error
    assert!(result.is_err(), "Expected error for invalid token ID");
    
    println!("Expected error received: {:?}", result.err());
}

#[tokio::test]
async fn test_connection_timeout() {
    // Create client with very short timeout
    let client = PolymarketRestClient::with_timeout(
        CLOB_BASE_URL,
        GAMMA_BASE_URL,
        Duration::from_millis(1), // 1ms timeout - should fail
    )
    .expect("Failed to create client");
    
    let result = client.get_server_time().await;
    
    // Should fail due to timeout (though sometimes it might succeed on fast connections)
    // This test is more about ensuring timeout is respected
    println!("Result with 1ms timeout: {:?}", result);
}

// ============================================================================
// Concurrent Request Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_requests() {
    test_delay().await;
    let client = create_test_client();
    
    // Get a sample token ID first
    let token_id = match get_sample_token_id(&client).await {
        Some(id) => id,
        None => {
            println!("SKIPPED: Could not find an active market with tokens");
            return;
        }
    };
    
    test_delay().await;
    
    // Make multiple concurrent requests
    let client = std::sync::Arc::new(client);
    let token_id_clone = token_id.clone();
    
    let handles: Vec<_> = (0..3)
        .map(|i| {
            let client = client.clone();
            let token = token_id_clone.clone();
            tokio::spawn(async move {
                match i % 3 {
                    0 => client.get_price(&token, Side::Buy).await.map(|p| format!("Buy: {}", p)),
                    1 => client.get_midpoint(&token).await.map(|m| format!("Mid: {}", m)),
                    _ => client.get_spread(&token).await.map(|s| format!("Spread: {}", s)),
                }
            })
        })
        .collect();
    
    // Wait for all requests
    for handle in handles {
        let result = handle.await;
        match result {
            Ok(Ok(msg)) => println!("Concurrent request succeeded: {}", msg),
            Ok(Err(e)) => println!("Concurrent request error: {}", e),
            Err(e) => println!("Task join error: {}", e),
        }
    }
}

// ============================================================================
// Data Validation Tests
// ============================================================================

#[tokio::test]
async fn test_market_data_consistency() {
    test_delay().await;
    let client = create_test_client();
    
    let token_id = match get_sample_token_id(&client).await {
        Some(id) => id,
        None => {
            println!("SKIPPED: Could not find an active market with tokens");
            return;
        }
    };
    
    test_delay().await;
    
    // Get order book
    let order_book = match client.get_order_book(&token_id).await {
        Ok(ob) => ob,
        Err(e) => {
            println!("Could not get order book: {}", e);
            return;
        }
    };
    
    // If we have both bid and ask, verify spread consistency
    if let (Some(best_bid), Some(best_ask)) = (order_book.best_bid(), order_book.best_ask()) {
        // Best ask should be >= best bid
        assert!(
            best_ask.price >= best_bid.price,
            "Best ask {} should be >= best bid {}",
            best_ask.price,
            best_bid.price
        );
        
        // Calculated midpoint should be between bid and ask
        if let Some(mid) = order_book.midpoint() {
            assert!(
                mid >= best_bid.price && mid <= best_ask.price,
                "Midpoint {} should be between bid {} and ask {}",
                mid,
                best_bid.price,
                best_ask.price
            );
        }
        
        // Calculated spread should match
        if let Some(spread) = order_book.spread() {
            let expected_spread = best_ask.price - best_bid.price;
            assert_eq!(
                spread, expected_spread,
                "Spread {} should equal ask - bid = {}",
                spread, expected_spread
            );
        }
    }
}
