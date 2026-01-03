//! Response types from the Messages API.
//!
//! This module provides types for parsing API responses:
//!
//! - [`Response`] - Complete API response with content and metadata
//! - [`StopReason`] - Reason why the model stopped generating
//!
//! # Accessing Response Content
//!
//! ```rust,no_run
//! use anthropic_tools::prelude::*;
//!
//! # async fn example() -> Result<()> {
//! # let mut client = Messages::new();
//! # client.model("claude-sonnet-4-20250514").max_tokens(1024).user("Hi");
//! let response = client.post().await?;
//!
//! // Get text content
//! let text = response.get_text();
//!
//! // Check stop reason
//! if response.stopped_naturally() {
//!     println!("Completed normally");
//! } else if response.hit_max_tokens() {
//!     println!("Hit token limit");
//! }
//!
//! // Check for tool use
//! if response.has_tool_use() {
//!     for tool_use in response.get_tool_uses() {
//!         // Handle tool use
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Stop Reasons
//!
//! - `EndTurn` - Natural end of response
//! - `MaxTokens` - Hit the token limit
//! - `StopSequence` - Hit a stop sequence
//! - `ToolUse` - Model wants to use a tool
//! - `Refusal` - Content was refused

use crate::common::Usage;
use crate::messages::request::content::ContentBlock;
use crate::messages::request::role::Role;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Response from the Messages API
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    /// Unique identifier for the response
    pub id: String,

    /// Object type (always "message")
    #[serde(rename = "type")]
    pub type_name: String,

    /// Role of the response (always "assistant")
    pub role: Role,

    /// Content blocks in the response
    pub content: Vec<ContentBlock>,

    /// Model that generated the response
    pub model: String,

    /// Reason the model stopped generating
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,

    /// Stop sequence that caused the model to stop (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,

    /// Token usage information
    pub usage: Usage,
}

/// Reason the model stopped generating
#[derive(Serialize, Deserialize, Debug, Clone, Display, EnumString, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Natural end of the response
    EndTurn,

    /// Max tokens limit reached
    MaxTokens,

    /// Stop sequence encountered
    StopSequence,

    /// Model decided to use a tool
    ToolUse,

    /// Content was refused
    Refusal,
}

impl Response {
    /// Get the text content from the response
    pub fn text(&self) -> Option<String> {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
            .into()
    }

    /// Get all text content as a single string
    pub fn get_text(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Check if the response contains tool use
    pub fn has_tool_use(&self) -> bool {
        self.content
            .iter()
            .any(|block| matches!(block, ContentBlock::ToolUse { .. }))
    }

    /// Get all tool use blocks
    pub fn get_tool_uses(&self) -> Vec<&ContentBlock> {
        self.content
            .iter()
            .filter(|block| matches!(block, ContentBlock::ToolUse { .. }))
            .collect()
    }

    /// Get tool use by ID
    pub fn get_tool_use_by_id(&self, id: &str) -> Option<&ContentBlock> {
        self.content.iter().find(|block| match block {
            ContentBlock::ToolUse {
                id: tool_id,
                ..
            } => tool_id == id,
            _ => false,
        })
    }

    /// Check if the response contains thinking content
    pub fn has_thinking(&self) -> bool {
        self.content
            .iter()
            .any(|block| matches!(block, ContentBlock::Thinking { .. }))
    }

    /// Get thinking content
    pub fn get_thinking(&self) -> Option<String> {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Thinking { thinking, .. } => Some(thinking.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
            .into()
    }

    /// Check if the model stopped due to tool use
    pub fn stopped_for_tool_use(&self) -> bool {
        self.stop_reason == Some(StopReason::ToolUse)
    }

    /// Check if the model stopped naturally
    pub fn stopped_naturally(&self) -> bool {
        self.stop_reason == Some(StopReason::EndTurn)
    }

    /// Check if the model hit the max tokens limit
    pub fn hit_max_tokens(&self) -> bool {
        self.stop_reason == Some(StopReason::MaxTokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_response() -> Response {
        Response {
            id: "msg_123".to_string(),
            type_name: "message".to_string(),
            role: Role::Assistant,
            content: vec![ContentBlock::Text {
                text: "Hello, world!".to_string(),
                cache_control: None,
            }],
            model: "claude-sonnet-4-20250514".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            stop_sequence: None,
            usage: Usage::new(10, 5),
        }
    }

    #[test]
    fn test_response_text() {
        let response = sample_response();
        assert_eq!(response.get_text(), "Hello, world!");
    }

    #[test]
    fn test_response_stop_reason() {
        let response = sample_response();
        assert!(response.stopped_naturally());
        assert!(!response.stopped_for_tool_use());
        assert!(!response.hit_max_tokens());
    }

    #[test]
    fn test_response_with_tool_use() {
        let response = Response {
            id: "msg_123".to_string(),
            type_name: "message".to_string(),
            role: Role::Assistant,
            content: vec![
                ContentBlock::Text {
                    text: "Let me search for that.".to_string(),
                    cache_control: None,
                },
                ContentBlock::ToolUse {
                    id: "tool_123".to_string(),
                    name: "search".to_string(),
                    input: serde_json::json!({"query": "test"}),
                },
            ],
            model: "claude-sonnet-4-20250514".to_string(),
            stop_reason: Some(StopReason::ToolUse),
            stop_sequence: None,
            usage: Usage::new(20, 15),
        };

        assert!(response.has_tool_use());
        assert!(response.stopped_for_tool_use());
        assert_eq!(response.get_tool_uses().len(), 1);
    }

    #[test]
    fn test_deserialize_response() {
        let json = r#"{
            "id": "msg_01XYZ",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "Hello!"
                }
            ],
            "model": "claude-sonnet-4-20250514",
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": 10,
                "output_tokens": 5
            }
        }"#;

        let response: Response = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "msg_01XYZ");
        assert_eq!(response.get_text(), "Hello!");
        assert_eq!(response.stop_reason, Some(StopReason::EndTurn));
    }

    #[test]
    fn test_serialize_stop_reason() {
        let reason = StopReason::ToolUse;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"tool_use\"");

        let reason = StopReason::EndTurn;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"end_turn\"");
    }
}
