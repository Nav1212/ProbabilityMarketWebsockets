//! Error types for the application

use thiserror::Error;

/// Result type alias using our ClientError
pub type Result<T> = std::result::Result<T, ClientError>;

/// Main error type for client operations
#[derive(Error, Debug)]
pub enum ClientError {
    /// WebSocket connection errors
    #[error("WebSocket connection error: {0}")]
    WebSocketConnection(String),

    /// WebSocket send/receive errors
    #[error("WebSocket communication error: {0}")]
    WebSocketCommunication(String),

    /// HTTP request errors
    #[error("HTTP request error: {0}")]
    HttpRequest(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}, retry after {retry_after_seconds:?} seconds")]
    RateLimit {
        message: String,
        retry_after_seconds: Option<u64>,
    },

    /// Invalid API response
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Market not found
    #[error("Market not found: {0}")]
    MarketNotFound(String),

    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Channel send errors
    #[error("Channel send error: {0}")]
    ChannelSend(String),

    /// Channel receive errors
    #[error("Channel receive error: {0}")]
    ChannelReceive(String),

    /// Generic internal errors
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<tokio_tungstenite::tungstenite::Error> for ClientError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        ClientError::WebSocketCommunication(err.to_string())
    }
}
