//! Token usage information from API responses.
//!
//! This module provides the [`Usage`] struct for tracking token consumption:
//!
//! - Input tokens consumed
//! - Output tokens generated
//! - Cache creation and read tokens (for prompt caching)
//!
//! # Example
//!
//! ```rust
//! use anthropic_tools::common::usage::Usage;
//!
//! let usage = Usage::new(100, 50);
//! assert_eq!(usage.input_tokens, 100);
//! assert_eq!(usage.output_tokens, 50);
//! assert_eq!(usage.total_tokens(), 150);
//! ```
//!
//! # With Prompt Caching
//!
//! When using prompt caching, the usage will include cache-related fields:
//!
//! ```rust
//! use anthropic_tools::common::usage::Usage;
//!
//! let mut usage = Usage::new(100, 50);
//! // These would be set by the API response
//! // usage.cache_creation_input_tokens = Some(50);
//! // usage.cache_read_input_tokens = Some(25);
//! assert_eq!(usage.cached_tokens(), 0); // No cache tokens in this example
//! ```

use serde::{Deserialize, Serialize};

/// Token usage information from Anthropic API response
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    /// The number of input tokens used
    pub input_tokens: usize,

    /// The number of output tokens generated
    pub output_tokens: usize,

    /// The number of input tokens used to create the cache entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<usize>,

    /// The number of input tokens read from the cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<usize>,
}

impl Usage {
    /// Create a new Usage instance
    pub fn new(input_tokens: usize, output_tokens: usize) -> Self {
        Self {
            input_tokens,
            output_tokens,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        }
    }

    /// Get total tokens (input + output)
    pub fn total_tokens(&self) -> usize {
        self.input_tokens + self.output_tokens
    }

    /// Get total cached tokens
    pub fn cached_tokens(&self) -> usize {
        self.cache_creation_input_tokens.unwrap_or(0) + self.cache_read_input_tokens.unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_new() {
        let usage = Usage::new(100, 50);
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total_tokens(), 150);
    }

    #[test]
    fn test_usage_deserialize() {
        let json = r#"{
            "input_tokens": 100,
            "output_tokens": 50,
            "cache_creation_input_tokens": 10,
            "cache_read_input_tokens": 20
        }"#;

        let usage: Usage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.cache_creation_input_tokens, Some(10));
        assert_eq!(usage.cache_read_input_tokens, Some(20));
        assert_eq!(usage.cached_tokens(), 30);
    }

    #[test]
    fn test_usage_serialize() {
        let usage = Usage::new(100, 50);
        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("\"input_tokens\":100"));
        assert!(json.contains("\"output_tokens\":50"));
        // cache fields should be omitted when None
        assert!(!json.contains("cache_creation_input_tokens"));
        assert!(!json.contains("cache_read_input_tokens"));
    }
}
