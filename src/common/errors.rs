//! Error types for the Anthropic API client.
//!
//! This module provides error handling for the library, including:
//!
//! - [`AnthropicToolError`] - Main error type for all API errors
//! - [`ErrorResponse`] - API error response structure
//! - [`ErrorDetail`] - Detailed error information
//! - [`Result`] - Type alias for `Result<T, AnthropicToolError>`
//!
//! # Error Types
//!
//! The API can return various error types:
//!
//! - `ApiKeyNotSet` - Missing API key
//! - `InvalidRequestError` - Malformed request
//! - `AuthenticationError` - Invalid API key
//! - `RateLimitError` - Too many requests
//! - `OverloadedError` - Server overloaded
//!
//! # Example
//!
//! ```rust
//! use anthropic_tools::common::errors::{AnthropicToolError, Result};
//!
//! fn validate_api_key(key: &str) -> Result<()> {
//!     if key.is_empty() {
//!         return Err(AnthropicToolError::ApiKeyNotSet);
//!     }
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Anthropic API error types
#[derive(Error, Debug)]
pub enum AnthropicToolError {
    #[error("API key is not set. Set ANTHROPIC_API_KEY environment variable.")]
    ApiKeyNotSet,

    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("HTTP request error: {0}")]
    RequestError(#[from] request::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("API error ({error_type}): {message}")]
    ApiError {
        error_type: String,
        message: String,
        request_id: Option<String>,
    },

    #[error("Invalid request error: {0}")]
    InvalidRequestError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Permission error: {0}")]
    PermissionError(String),

    #[error("Not found error: {0}")]
    NotFoundError(String),

    #[error("Rate limit error: {0}")]
    RateLimitError(String),

    #[error("Overloaded error: {0}")]
    OverloadedError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AnthropicToolError>;

/// Error response from Anthropic API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    #[serde(rename = "type")]
    pub type_name: String,
    pub error: ErrorDetail,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Error detail in API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    #[serde(rename = "type")]
    pub type_name: String,
    pub message: String,
}

impl ErrorResponse {
    /// Convert ErrorResponse to AnthropicToolError
    pub fn into_error(self) -> AnthropicToolError {
        let message = self.error.message.clone();
        let request_id = self.request_id.clone();

        match self.error.type_name.as_str() {
            "invalid_request_error" => AnthropicToolError::InvalidRequestError(message),
            "authentication_error" => AnthropicToolError::AuthenticationError(message),
            "permission_error" => AnthropicToolError::PermissionError(message),
            "not_found_error" => AnthropicToolError::NotFoundError(message),
            "rate_limit_error" => AnthropicToolError::RateLimitError(message),
            "overloaded_error" => AnthropicToolError::OverloadedError(message),
            _ => AnthropicToolError::ApiError {
                error_type: self.error.type_name,
                message,
                request_id,
            },
        }
    }
}
