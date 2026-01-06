//! Authentication utilities for Polymarket API

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::common::errors::{ClientError, Result};

type HmacSha256 = Hmac<Sha256>;

/// Generate HMAC-SHA256 signature for API requests
///
/// # Arguments
/// * `secret` - API secret key (base64 encoded)
/// * `timestamp` - Unix timestamp in seconds
/// * `method` - HTTP method (GET, POST, etc.)
/// * `request_path` - API endpoint path
/// * `body` - Request body (empty string for GET requests)
pub fn sign_request(
    secret: &str,
    timestamp: i64,
    method: &str,
    request_path: &str,
    body: &str,
) -> Result<String> {
    // Decode the base64 secret
    let secret_bytes = BASE64
        .decode(secret)
        .map_err(|e| ClientError::Authentication(format!("Failed to decode secret: {}", e)))?;

    // Create the message to sign: timestamp + method + path + body
    let message = format!("{}{}{}{}", timestamp, method.to_uppercase(), request_path, body);

    // Create HMAC and compute signature
    let mut mac = HmacSha256::new_from_slice(&secret_bytes)
        .map_err(|e| ClientError::Authentication(format!("Failed to create HMAC: {}", e)))?;
    mac.update(message.as_bytes());
    let result = mac.finalize();

    // Encode signature as base64
    Ok(BASE64.encode(result.into_bytes()))
}

/// Generate authentication headers for API requests
///
/// Returns a tuple of (timestamp, signature) to be used in headers
pub fn generate_auth_headers(
    api_key: &str,
    api_secret: &str,
    passphrase: &str,
    method: &str,
    request_path: &str,
    body: &str,
) -> Result<AuthHeaders> {
    let timestamp = chrono::Utc::now().timestamp();
    let signature = sign_request(api_secret, timestamp, method, request_path, body)?;

    Ok(AuthHeaders {
        api_key: api_key.to_string(),
        signature,
        timestamp,
        passphrase: passphrase.to_string(),
    })
}

/// Authentication headers for API requests
#[derive(Debug, Clone)]
pub struct AuthHeaders {
    pub api_key: String,
    pub signature: String,
    pub timestamp: i64,
    pub passphrase: String,
}

impl AuthHeaders {
    /// Add authentication headers to a reqwest RequestBuilder
    pub fn apply_to_request(
        &self,
        request: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        request
            .header("POLY_API_KEY", &self.api_key)
            .header("POLY_SIGNATURE", &self.signature)
            .header("POLY_TIMESTAMP", self.timestamp.to_string())
            .header("POLY_PASSPHRASE", &self.passphrase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request_format() {
        // This tests that the signing function produces a valid base64 output
        // Note: The actual signature depends on the secret and message
        let secret = BASE64.encode(b"test_secret_key_12345");
        let result = sign_request(&secret, 1234567890, "GET", "/test/path", "");
        
        assert!(result.is_ok());
        let signature = result.unwrap();
        
        // Verify it's valid base64
        assert!(BASE64.decode(&signature).is_ok());
    }

    #[test]
    fn test_generate_auth_headers() {
        let secret = BASE64.encode(b"test_secret_key_12345");
        let result = generate_auth_headers(
            "test_api_key",
            &secret,
            "test_passphrase",
            "GET",
            "/test",
            "",
        );

        assert!(result.is_ok());
        let headers = result.unwrap();
        assert_eq!(headers.api_key, "test_api_key");
        assert_eq!(headers.passphrase, "test_passphrase");
        assert!(!headers.signature.is_empty());
    }
}
