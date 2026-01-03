//! Request body structure for the Messages API.
//!
//! This module provides the [`Body`] struct that represents the JSON payload
//! sent to the Anthropic API:
//!
//! - [`Body`] - Complete request body with all parameters
//! - [`ToolChoice`] - Configuration for tool selection behavior
//! - [`Metadata`] - Optional request metadata
//!
//! # Request Parameters
//!
//! Required:
//! - `model` - Model identifier (e.g., "claude-sonnet-4-20250514")
//! - `messages` - Conversation messages
//! - `max_tokens` - Maximum tokens to generate
//!
//! Optional:
//! - `system` - System prompt
//! - `temperature` - Sampling temperature (0.0-1.0)
//! - `top_p`, `top_k` - Sampling parameters
//! - `stop_sequences` - Custom stop sequences
//! - `tools` - Available tools for function calling
//! - `stream` - Enable streaming responses
//!
//! # Example
//!
//! ```rust
//! use anthropic_tools::messages::request::body::Body;
//!
//! let body = Body::new("claude-sonnet-4-20250514", 1024);
//! assert_eq!(body.model, "claude-sonnet-4-20250514");
//! assert_eq!(body.max_tokens, 1024);
//! ```

use crate::common::errors::{AnthropicToolError, Result};
use crate::messages::request::{mcp::McpServer, message::Message, message::SystemPrompt};
use serde::{Deserialize, Serialize};

/// Request body for the Messages API
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body {
    /// The model to use (e.g., "claude-sonnet-4-20250514")
    pub model: String,

    /// Input messages for the conversation
    pub messages: Vec<Message>,

    /// Maximum number of tokens to generate (required)
    pub max_tokens: usize,

    /// System prompt (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,

    /// Sampling temperature (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top-p sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Top-k sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Custom stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Tools available to the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,

    /// Tool choice configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Request metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,

    /// Container for code execution (beta)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,

    /// MCP servers configuration (beta)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Vec<McpServer>>,
}

/// Tool choice configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ToolChoice {
    /// Let the model decide whether to use tools
    #[serde(rename = "auto")]
    Auto,

    /// Force the model to use a specific tool
    #[serde(rename = "tool")]
    Tool { name: String },

    /// Force the model to use any tool
    #[serde(rename = "any")]
    Any,

    /// Disable tool use
    #[serde(rename = "none")]
    None,
}

/// Request metadata
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Metadata {
    /// User ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

impl Default for Body {
    fn default() -> Self {
        Body {
            model: String::new(),
            messages: Vec::new(),
            max_tokens: 1024,
            system: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: None,
            tools: None,
            tool_choice: None,
            metadata: None,
            container: None,
            mcp_servers: None,
        }
    }
}

impl Body {
    /// Create a new request body with model and max_tokens
    pub fn new<T: AsRef<str>>(model: T, max_tokens: usize) -> Self {
        Body {
            model: model.as_ref().to_string(),
            max_tokens,
            ..Default::default()
        }
    }

    /// Validate the request body
    pub fn validate(&self) -> Result<()> {
        if self.model.is_empty() {
            return Err(AnthropicToolError::MissingRequiredField(
                "model".to_string(),
            ));
        }

        if self.messages.is_empty() {
            return Err(AnthropicToolError::MissingRequiredField(
                "messages".to_string(),
            ));
        }

        if self.max_tokens == 0 {
            return Err(AnthropicToolError::InvalidParameter(
                "max_tokens must be greater than 0".to_string(),
            ));
        }

        // Validate temperature if set
        if let Some(temp) = self.temperature {
            if !(0.0..=1.0).contains(&temp) {
                return Err(AnthropicToolError::InvalidParameter(
                    "temperature must be between 0.0 and 1.0".to_string(),
                ));
            }
        }

        // Validate top_p if set
        if let Some(top_p) = self.top_p {
            if !(0.0..=1.0).contains(&top_p) {
                return Err(AnthropicToolError::InvalidParameter(
                    "top_p must be between 0.0 and 1.0".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_new() {
        let body = Body::new("claude-sonnet-4-20250514", 1024);
        assert_eq!(body.model, "claude-sonnet-4-20250514");
        assert_eq!(body.max_tokens, 1024);
    }

    #[test]
    fn test_body_validate_missing_model() {
        let body = Body::default();
        let result = body.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_body_validate_missing_messages() {
        let body = Body::new("claude-sonnet-4-20250514", 1024);
        let result = body.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_choice_serialize() {
        let auto = ToolChoice::Auto;
        let json = serde_json::to_string(&auto).unwrap();
        assert!(json.contains("\"type\":\"auto\""));

        let tool = ToolChoice::Tool {
            name: "search".to_string(),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("\"type\":\"tool\""));
        assert!(json.contains("\"name\":\"search\""));
    }

    #[test]
    fn test_body_serialize() {
        let body = Body::new("claude-sonnet-4-20250514", 1024);
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"model\":\"claude-sonnet-4-20250514\""));
        assert!(json.contains("\"max_tokens\":1024"));
        // Optional fields should not be present
        assert!(!json.contains("\"temperature\""));
        assert!(!json.contains("\"system\""));
    }
}
