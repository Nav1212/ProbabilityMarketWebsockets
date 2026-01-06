//! REST API client for Polymarket CLOB

use reqwest::Client;
use rust_decimal::Decimal;
use std::time::Duration;
use tracing::{debug, instrument};

use super::auth::{generate_auth_headers, AuthHeaders};
use super::messages::*;
use crate::common::errors::{ClientError, Result};
use crate::common::types::{OrderBook, Platform, PriceLevel, Side};
use crate::config::types::ApiCredentials;

/// REST API client for Polymarket CLOB
#[derive(Debug, Clone)]
pub struct PolymarketRestClient {
    /// HTTP client
    client: Client,
    /// Base URL for the CLOB API
    base_url: String,
    /// Base URL for the Gamma API
    gamma_url: String,
    /// Optional API credentials for authenticated endpoints
    credentials: Option<ApiCredentials>,
}

impl PolymarketRestClient {
    /// Create a new REST client (unauthenticated)
    pub fn new(base_url: &str, gamma_url: &str) -> Result<Self> {
        Self::with_timeout(base_url, gamma_url, Duration::from_secs(30))
    }

    /// Create a new REST client with custom timeout
    pub fn with_timeout(base_url: &str, gamma_url: &str, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| ClientError::Internal(e.to_string()))?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            gamma_url: gamma_url.trim_end_matches('/').to_string(),
            credentials: None,
        })
    }

    /// Set API credentials for authenticated requests
    pub fn with_credentials(mut self, credentials: ApiCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Generate authentication headers if credentials are set
    fn auth_headers(&self, method: &str, path: &str, body: &str) -> Result<Option<AuthHeaders>> {
        match &self.credentials {
            Some(creds) => {
                let headers = generate_auth_headers(
                    &creds.api_key,
                    &creds.api_secret,
                    &creds.passphrase,
                    method,
                    path,
                    body,
                )?;
                Ok(Some(headers))
            }
            None => Ok(None),
        }
    }

    // ========================================================================
    // Public Endpoints (No Authentication Required)
    // ========================================================================

    /// Check if the API is healthy
    #[instrument(skip(self))]
    pub async fn get_ok(&self) -> Result<bool> {
        let url = format!("{}/", self.base_url);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get server time
    #[instrument(skip(self))]
    pub async fn get_server_time(&self) -> Result<i64> {
        let url = format!("{}/time", self.base_url);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(ClientError::InvalidResponse(format!(
                "Server returned status: {}",
                response.status()
            )));
        }

        let time_response: TimeResponse = response.json().await?;
        time_response
            .timestamp
            .parse()
            .map_err(|e| ClientError::InvalidResponse(format!("Invalid timestamp: {}", e)))
    }

    /// Get price for a token
    ///
    /// # Arguments
    /// * `token_id` - The token ID to get price for
    /// * `side` - BUY or SELL side
    #[instrument(skip(self))]
    pub async fn get_price(&self, token_id: &str, side: Side) -> Result<Decimal> {
        let side_str = match side {
            Side::Buy => "buy",
            Side::Sell => "sell",
        };
        let url = format!(
            "{}/price?token_id={}&side={}",
            self.base_url, token_id, side_str
        );
        debug!("Fetching price from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::InvalidResponse(format!(
                "Server returned status {}: {}",
                status, body
            )));
        }

        let price_response: PriceResponse = response.json().await?;
        price_response
            .price
            .parse()
            .map_err(|e| ClientError::InvalidResponse(format!("Invalid price: {}", e)))
    }

    /// Get midpoint price for a token
    #[instrument(skip(self))]
    pub async fn get_midpoint(&self, token_id: &str) -> Result<Decimal> {
        let url = format!("{}/midpoint?token_id={}", self.base_url, token_id);
        debug!("Fetching midpoint from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::InvalidResponse(format!(
                "Server returned status {}: {}",
                status, body
            )));
        }

        let midpoint_response: MidpointResponse = response.json().await?;
        midpoint_response
            .mid
            .parse()
            .map_err(|e| ClientError::InvalidResponse(format!("Invalid midpoint: {}", e)))
    }

    /// Get spread for a token
    #[instrument(skip(self))]
    pub async fn get_spread(&self, token_id: &str) -> Result<Decimal> {
        let url = format!("{}/spread?token_id={}", self.base_url, token_id);
        debug!("Fetching spread from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::InvalidResponse(format!(
                "Server returned status {}: {}",
                status, body
            )));
        }

        let spread_response: SpreadResponse = response.json().await?;
        spread_response
            .spread
            .parse()
            .map_err(|e| ClientError::InvalidResponse(format!("Invalid spread: {}", e)))
    }

    /// Get order book for a token
    #[instrument(skip(self))]
    pub async fn get_order_book(&self, token_id: &str) -> Result<OrderBook> {
        let url = format!("{}/book?token_id={}", self.base_url, token_id);
        debug!("Fetching order book from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::InvalidResponse(format!(
                "Server returned status {}: {}",
                status, body
            )));
        }

        let book_response: OrderBookResponse = response.json().await?;
        self.convert_order_book_response(book_response)
    }

    /// Get last trade price for a token
    #[instrument(skip(self))]
    pub async fn get_last_trade_price(&self, token_id: &str) -> Result<Decimal> {
        let url = format!("{}/last-trade-price?token_id={}", self.base_url, token_id);
        debug!("Fetching last trade price from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::InvalidResponse(format!(
                "Server returned status {}: {}",
                status, body
            )));
        }

        let ltp_response: LastTradePriceResponse = response.json().await?;
        ltp_response
            .price
            .parse()
            .map_err(|e| ClientError::InvalidResponse(format!("Invalid price: {}", e)))
    }

    /// Get simplified markets list
    #[instrument(skip(self))]
    pub async fn get_simplified_markets(&self) -> Result<MarketsResponse> {
        let url = format!("{}/simplified-markets", self.base_url);
        debug!("Fetching simplified markets from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::InvalidResponse(format!(
                "Server returned status {}: {}",
                status, body
            )));
        }

        let markets: MarketsResponse = response.json().await?;
        Ok(markets)
    }

    /// Get market by condition ID
    #[instrument(skip(self))]
    pub async fn get_market(&self, condition_id: &str) -> Result<MarketResponse> {
        let url = format!("{}/markets/{}", self.base_url, condition_id);
        debug!("Fetching market from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(ClientError::MarketNotFound(condition_id.to_string()));
            }
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::InvalidResponse(format!(
                "Server returned status {}: {}",
                status, body
            )));
        }

        let market: MarketResponse = response.json().await?;
        Ok(market)
    }

    // ========================================================================
    // Gamma API Endpoints (Market Discovery)
    // ========================================================================

    /// Get events from Gamma API
    #[instrument(skip(self))]
    pub async fn get_gamma_events(&self, limit: Option<u32>) -> Result<Vec<GammaEvent>> {
        let mut url = format!("{}/events", self.gamma_url);
        if let Some(l) = limit {
            url = format!("{}?limit={}", url, l);
        }
        debug!("Fetching events from Gamma API: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::InvalidResponse(format!(
                "Gamma API returned status {}: {}",
                status, body
            )));
        }

        let events_response: GammaEventsResponse = response.json().await?;
        Ok(events_response.data.or(events_response.events).unwrap_or_default())
    }

    /// Get markets from Gamma API
    #[instrument(skip(self))]
    pub async fn get_gamma_markets(&self, limit: Option<u32>, active: Option<bool>) -> Result<Vec<GammaMarket>> {
        let mut url = format!("{}/markets", self.gamma_url);
        let mut params = vec![];
        
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(a) = active {
            params.push(format!("active={}", a));
        }
        
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }
        debug!("Fetching markets from Gamma API: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::InvalidResponse(format!(
                "Gamma API returned status {}: {}",
                status, body
            )));
        }

        let markets_response: GammaMarketsResponse = response.json().await?;
        Ok(markets_response.data.or(markets_response.markets).unwrap_or_default())
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Convert API order book response to unified OrderBook type
    fn convert_order_book_response(&self, response: OrderBookResponse) -> Result<OrderBook> {
        let bids: Result<Vec<PriceLevel>> = response
            .bids
            .into_iter()
            .map(|level| {
                Ok(PriceLevel {
                    price: level
                        .price
                        .parse()
                        .map_err(|e| ClientError::InvalidResponse(format!("Invalid bid price: {}", e)))?,
                    size: level
                        .size
                        .parse()
                        .map_err(|e| ClientError::InvalidResponse(format!("Invalid bid size: {}", e)))?,
                })
            })
            .collect();

        let asks: Result<Vec<PriceLevel>> = response
            .asks
            .into_iter()
            .map(|level| {
                Ok(PriceLevel {
                    price: level
                        .price
                        .parse()
                        .map_err(|e| ClientError::InvalidResponse(format!("Invalid ask price: {}", e)))?,
                    size: level
                        .size
                        .parse()
                        .map_err(|e| ClientError::InvalidResponse(format!("Invalid ask size: {}", e)))?,
                })
            })
            .collect();

        Ok(OrderBook {
            platform: Platform::Polymarket,
            market_id: response.market,
            asset_id: response.asset_id,
            bids: bids?,
            asks: asks?,
            timestamp: chrono::Utc::now(),
            sequence: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = PolymarketRestClient::new(
            "https://clob.polymarket.com",
            "https://gamma-api.polymarket.com",
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_url_normalization() {
        let client = PolymarketRestClient::new(
            "https://clob.polymarket.com/",
            "https://gamma-api.polymarket.com/",
        )
        .unwrap();
        assert!(!client.base_url.ends_with('/'));
        assert!(!client.gamma_url.ends_with('/'));
    }
}
