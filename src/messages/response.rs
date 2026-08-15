//! Response types from the Messages API.
//!
//! This module provides types for parsing API responses:
//!
//! - [`Response`] - Complete API response with content and metadata
//! - [`StopReason`] - Reason why the model stopped generating
//! - [`StopDetails`] - Structured detail attached to a refusal
//! - [`Container`] - Code execution container handle (for reuse)
//!
//! # Accessing Response Content
//!
//! ```rust,no_run
//! use anthropic_tools::prelude::*;
//!
//! # async fn example() -> Result<()> {
//! # let mut client = Messages::new();
//! # client.model(Model::Opus5).max_tokens(1024).user("Hi");
//! let response = client.post().await?;
//!
//! // Always check for a refusal before reading content — a refused request
//! // returns HTTP 200 with an empty or partial content array.
//! if response.was_refused() {
//!     eprintln!("refused: {:?}", response.refusal_category());
//!     return Ok(());
//! }
//!
//! let text = response.get_text();
//!
//! if response.stopped_naturally() {
//!     println!("Completed normally");
//! } else if response.hit_max_tokens() {
//!     println!("Hit token limit");
//! }
//!
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
//! - `PauseTurn` - A server-side tool loop paused and can be resumed
//! - `Refusal` - Safety classifiers declined the request
//! - `ModelContextWindowExceeded` - The context window, not `max_tokens`, was exhausted

use crate::common::Usage;
use crate::common::errors::{AnthropicToolError, Result};
use crate::messages::request::content::ContentBlock;
use crate::messages::request::model::Model;
use crate::messages::request::role::Role;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use strum::EnumString;

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
    pub model: Model,

    /// Reason the model stopped generating
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,

    /// Stop sequence that caused the model to stop (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,

    /// Structured detail about why generation stopped
    ///
    /// Populated only when `stop_reason` is [`StopReason::Refusal`]; `None`
    /// for every other stop reason. Always guard before reading it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_details: Option<StopDetails>,

    /// Code execution container handle, for reuse across requests
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<Container>,

    /// Token usage information
    pub usage: Usage,
}

/// Result of a token-counting request
///
/// Returned by [`Messages::count_tokens`](crate::messages::request::Messages::count_tokens).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TokenCount {
    /// Number of input tokens the request would consume
    pub input_tokens: usize,
}

/// Code execution container handle
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Container {
    /// Container identifier, reusable on a subsequent request
    pub id: String,

    /// When the container expires
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Structured detail attached to a refusal
///
/// Only present when [`Response::stop_reason`] is [`StopReason::Refusal`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StopDetails {
    /// Detail type (currently always `"refusal"`)
    #[serde(rename = "type")]
    pub type_name: String,

    /// Policy category, e.g. `"cyber"`, `"bio"`, `"reasoning_extraction"`
    ///
    /// This is an open set and may be `null`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Human-readable explanation, when available
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,

    /// A model worth retrying directly, when the API can recommend one
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_model: Option<Model>,
}

/// Reason the model stopped generating
///
/// Unrecognized values deserialize to [`StopReason::Other`] rather than
/// failing, so a new server-side stop reason cannot break an existing client.
///
/// # Note on `Display`
///
/// [`Display`](std::fmt::Display) renders the Rust variant name (`"EndTurn"`),
/// matching this type's historical behavior. Use [`StopReason::wire_str`] for
/// the value the API actually sends (`"end_turn"`).
#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[non_exhaustive]
pub enum StopReason {
    /// Natural end of the response
    #[strum(serialize = "EndTurn", serialize = "end_turn")]
    EndTurn,

    /// Max tokens limit reached
    #[strum(serialize = "MaxTokens", serialize = "max_tokens")]
    MaxTokens,

    /// Stop sequence encountered
    #[strum(serialize = "StopSequence", serialize = "stop_sequence")]
    StopSequence,

    /// Model decided to use a tool
    #[strum(serialize = "ToolUse", serialize = "tool_use")]
    ToolUse,

    /// A server-side tool loop paused and can be resumed
    ///
    /// Re-send the conversation with the assistant turn appended; the server
    /// resumes automatically. Do not add a "continue" user message.
    #[strum(serialize = "PauseTurn", serialize = "pause_turn")]
    PauseTurn,

    /// Safety classifiers declined the request
    ///
    /// The response is HTTP 200 with an empty or partial content array. Check
    /// [`Response::stop_details`] for the policy category.
    #[strum(serialize = "Refusal", serialize = "refusal")]
    Refusal,

    /// The context window was exhausted (distinct from `MaxTokens`)
    #[strum(
        serialize = "ModelContextWindowExceeded",
        serialize = "model_context_window_exceeded"
    )]
    ModelContextWindowExceeded,

    /// A stop reason this version of the library does not model
    #[strum(default)]
    Other(String),
}

impl StopReason {
    /// The wire value the API uses for this stop reason
    pub fn wire_str(&self) -> &str {
        match self {
            StopReason::EndTurn => "end_turn",
            StopReason::MaxTokens => "max_tokens",
            StopReason::StopSequence => "stop_sequence",
            StopReason::ToolUse => "tool_use",
            StopReason::PauseTurn => "pause_turn",
            StopReason::Refusal => "refusal",
            StopReason::ModelContextWindowExceeded => "model_context_window_exceeded",
            StopReason::Other(s) => s.as_str(),
        }
    }
}

impl fmt::Display for StopReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            StopReason::EndTurn => "EndTurn",
            StopReason::MaxTokens => "MaxTokens",
            StopReason::StopSequence => "StopSequence",
            StopReason::ToolUse => "ToolUse",
            StopReason::PauseTurn => "PauseTurn",
            StopReason::Refusal => "Refusal",
            StopReason::ModelContextWindowExceeded => "ModelContextWindowExceeded",
            StopReason::Other(s) => s.as_str(),
        };
        write!(f, "{}", name)
    }
}

impl Serialize for StopReason {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.wire_str())
    }
}

impl<'de> Deserialize<'de> for StopReason {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "end_turn" => StopReason::EndTurn,
            "max_tokens" => StopReason::MaxTokens,
            "stop_sequence" => StopReason::StopSequence,
            "tool_use" => StopReason::ToolUse,
            "pause_turn" => StopReason::PauseTurn,
            "refusal" => StopReason::Refusal,
            "model_context_window_exceeded" => StopReason::ModelContextWindowExceeded,
            _ => StopReason::Other(s),
        })
    }
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

    /// Parse the text content as JSON into `T`
    ///
    /// Intended for responses constrained with
    /// [`OutputFormat::json_schema`](crate::messages::request::body::OutputFormat::json_schema),
    /// where the text content is guaranteed to be valid JSON matching the schema.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use anthropic_tools::prelude::*;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Contact { name: String, email: String }
    ///
    /// # async fn example() -> Result<()> {
    /// # let mut client = Messages::new();
    /// # client.model(Model::Opus5).max_tokens(1024).user("Extract...");
    /// let response = client.post().await?;
    /// let contact: Contact = response.json()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        let text = self.get_text();
        serde_json::from_str(&text).map_err(|e| {
            AnthropicToolError::InvalidParameter(format!(
                "failed to parse response content as JSON: {}",
                e
            ))
        })
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
            ContentBlock::ToolUse { id: tool_id, .. } => tool_id == id,
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
    ///
    /// On current models this is empty unless the request asked for
    /// [`ThinkingDisplay::Summarized`](crate::messages::request::body::ThinkingDisplay::Summarized).
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

    /// Check if the request was refused by safety classifiers
    ///
    /// A refusal is a successful HTTP 200 response, so this must be checked
    /// before reading [`Response::content`].
    pub fn was_refused(&self) -> bool {
        self.stop_reason == Some(StopReason::Refusal)
    }

    /// Get the refusal policy category, if the request was refused
    pub fn refusal_category(&self) -> Option<&str> {
        self.stop_details
            .as_ref()
            .and_then(|d| d.category.as_deref())
    }

    /// Get the refusal explanation, if the request was refused
    pub fn refusal_explanation(&self) -> Option<&str> {
        self.stop_details
            .as_ref()
            .and_then(|d| d.explanation.as_deref())
    }

    /// Check if a server-side tool loop paused and the turn can be resumed
    ///
    /// Re-send the conversation with this response's content appended as an
    /// assistant turn; the server resumes automatically.
    pub fn is_paused(&self) -> bool {
        self.stop_reason == Some(StopReason::PauseTurn)
    }

    /// Check if the context window (not `max_tokens`) was exhausted
    pub fn exceeded_context_window(&self) -> bool {
        self.stop_reason == Some(StopReason::ModelContextWindowExceeded)
    }

    /// Get the refusal-fallback switch points in this response, if any
    ///
    /// Each entry is `(declining model, continuing model)`.
    pub fn fallback_switches(&self) -> Vec<(&Model, &Model)> {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Fallback { from, to } => Some((&from.model, &to.model)),
                _ => None,
            })
            .collect()
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
            model: Model::Opus5,
            stop_reason: Some(StopReason::EndTurn),
            stop_sequence: None,
            stop_details: None,
            container: None,
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
        assert!(!response.was_refused());
        assert!(!response.is_paused());
    }

    #[test]
    fn test_response_with_tool_use() {
        let response = Response {
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
            stop_reason: Some(StopReason::ToolUse),
            usage: Usage::new(20, 15),
            ..sample_response()
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
            "content": [{"type": "text", "text": "Hello!"}],
            "model": "claude-opus-5",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let response: Response = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "msg_01XYZ");
        assert_eq!(response.get_text(), "Hello!");
        assert_eq!(response.model, Model::Opus5);
        assert_eq!(response.stop_reason, Some(StopReason::EndTurn));
    }

    #[test]
    fn test_deserialize_response_unknown_model() {
        let json = r#"{
            "id": "msg_01XYZ",
            "type": "message",
            "role": "assistant",
            "content": [],
            "model": "claude-future-model-2027",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let response: Response = serde_json::from_str(json).unwrap();
        assert_eq!(
            response.model,
            Model::Other("claude-future-model-2027".to_string())
        );
    }

    #[test]
    fn test_deserialize_pause_turn() {
        let json = r#"{
            "id": "msg_01XYZ",
            "type": "message",
            "role": "assistant",
            "content": [],
            "model": "claude-opus-5",
            "stop_reason": "pause_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let response: Response = serde_json::from_str(json).unwrap();
        assert_eq!(response.stop_reason, Some(StopReason::PauseTurn));
        assert!(response.is_paused());
    }

    #[test]
    fn test_deserialize_unknown_stop_reason() {
        let json = r#"{
            "id": "msg_01XYZ",
            "type": "message",
            "role": "assistant",
            "content": [],
            "model": "claude-opus-5",
            "stop_reason": "some_future_reason",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let response: Response = serde_json::from_str(json).unwrap();
        assert_eq!(
            response.stop_reason,
            Some(StopReason::Other("some_future_reason".to_string()))
        );
    }

    #[test]
    fn test_deserialize_refusal_with_stop_details() {
        let json = r#"{
            "id": "msg_01XYZ",
            "type": "message",
            "role": "assistant",
            "content": [],
            "model": "claude-opus-5",
            "stop_reason": "refusal",
            "stop_details": {
                "type": "refusal",
                "category": "cyber",
                "explanation": "Declined by policy."
            },
            "usage": {"input_tokens": 10, "output_tokens": 0}
        }"#;

        let response: Response = serde_json::from_str(json).unwrap();
        assert!(response.was_refused());
        assert_eq!(response.refusal_category(), Some("cyber"));
        assert_eq!(response.refusal_explanation(), Some("Declined by policy."));
    }

    #[test]
    fn test_deserialize_unknown_content_block() {
        // An unmodeled block type must not fail the whole response
        let json = r#"{
            "id": "msg_01XYZ",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "text", "text": "Hi"},
                {"type": "some_future_block", "whatever": 1}
            ],
            "model": "claude-opus-5",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let response: Response = serde_json::from_str(json).unwrap();
        assert_eq!(response.content.len(), 2);
        assert!(matches!(response.content[1], ContentBlock::Unknown));
        assert_eq!(response.get_text(), "Hi");
    }

    #[test]
    fn test_deserialize_redacted_thinking() {
        let json = r#"{
            "id": "msg_01XYZ",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "redacted_thinking", "data": "abc123"}],
            "model": "claude-opus-5",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let response: Response = serde_json::from_str(json).unwrap();
        assert!(matches!(
            response.content[0],
            ContentBlock::RedactedThinking { .. }
        ));
    }

    #[test]
    fn test_fallback_switches() {
        let json = r#"{
            "id": "msg_01XYZ",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "fallback",
                 "from": {"model": "claude-fable-5"},
                 "to": {"model": "claude-opus-4-8"}},
                {"type": "text", "text": "Answer"}
            ],
            "model": "claude-opus-4-8",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let response: Response = serde_json::from_str(json).unwrap();
        let switches = response.fallback_switches();
        assert_eq!(switches.len(), 1);
        assert_eq!(switches[0].0, &Model::Fable5);
        assert_eq!(switches[0].1, &Model::Opus48);
    }

    #[test]
    fn test_response_json_parse() {
        let response = Response {
            content: vec![ContentBlock::Text {
                text: r#"{"name":"Alice","age":30}"#.to_string(),
                cache_control: None,
            }],
            ..sample_response()
        };

        #[derive(serde::Deserialize)]
        struct Person {
            name: String,
            age: u32,
        }

        let person: Person = response.json().unwrap();
        assert_eq!(person.name, "Alice");
        assert_eq!(person.age, 30);
    }

    #[test]
    fn test_serialize_stop_reason() {
        assert_eq!(
            serde_json::to_string(&StopReason::ToolUse).unwrap(),
            "\"tool_use\""
        );
        assert_eq!(
            serde_json::to_string(&StopReason::EndTurn).unwrap(),
            "\"end_turn\""
        );
        assert_eq!(
            serde_json::to_string(&StopReason::PauseTurn).unwrap(),
            "\"pause_turn\""
        );
        assert_eq!(
            serde_json::to_string(&StopReason::Other("x".into())).unwrap(),
            "\"x\""
        );
    }

    #[test]
    fn test_stop_reason_display_and_wire_str() {
        // Display keeps the historical variant-name rendering
        assert_eq!(StopReason::EndTurn.to_string(), "EndTurn");
        assert_eq!(StopReason::EndTurn.wire_str(), "end_turn");
    }

    #[test]
    fn test_stop_reason_from_str() {
        use std::str::FromStr;

        assert_eq!(
            StopReason::from_str("EndTurn").unwrap(),
            StopReason::EndTurn
        );
        assert_eq!(
            StopReason::from_str("end_turn").unwrap(),
            StopReason::EndTurn
        );
        assert_eq!(
            StopReason::from_str("whatever").unwrap(),
            StopReason::Other("whatever".to_string())
        );
    }
}
