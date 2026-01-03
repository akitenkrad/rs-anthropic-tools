//! SSE streaming support for the Messages API.
//!
//! This module provides types for handling Server-Sent Events (SSE) streaming
//! responses from the Anthropic API:
//!
//! - [`StreamEvent`] - Enum of all possible stream event types
//! - [`Delta`] - Content deltas (text, tool input, thinking)
//! - [`MessageDelta`] - Final message metadata (stop reason, usage)
//! - [`StreamAccumulator`] - Helper for accumulating streamed content
//! - [`parse_sse_line`] - Parse individual SSE lines
//!
//! # Stream Event Types
//!
//! - `MessageStart` - Initial message with metadata
//! - `ContentBlockStart` - New content block beginning
//! - `ContentBlockDelta` - Incremental content update
//! - `ContentBlockStop` - Content block complete
//! - `MessageDelta` - Final usage and stop reason
//! - `MessageStop` - Stream complete
//! - `Ping` - Keep-alive event
//! - `Error` - Error event
//!
//! # Using StreamAccumulator
//!
//! ```rust
//! use anthropic_tools::messages::streaming::{StreamAccumulator, StreamEvent, Delta};
//! use anthropic_tools::messages::request::content::ContentBlock;
//!
//! let mut acc = StreamAccumulator::new();
//!
//! // Process events as they arrive
//! acc.process_event(StreamEvent::ContentBlockStart {
//!     index: 0,
//!     content_block: ContentBlock::text(""),
//! });
//!
//! acc.process_event(StreamEvent::ContentBlockDelta {
//!     index: 0,
//!     delta: Delta::TextDelta { text: "Hello".to_string() },
//! });
//!
//! assert_eq!(acc.get_text(), "Hello");
//! ```

use crate::common::errors::{ErrorDetail, Result};
use crate::common::Usage;
use crate::messages::request::content::ContentBlock;
use crate::messages::response::Response;
use serde::{Deserialize, Serialize};

/// Server-Sent Events stream event types
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum StreamEvent {
    /// Message has started
    #[serde(rename = "message_start")]
    MessageStart { message: Response },

    /// Content block has started
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },

    /// Ping event to keep connection alive
    #[serde(rename = "ping")]
    Ping,

    /// Delta for content block
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: usize, delta: Delta },

    /// Content block has stopped
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },

    /// Message delta (final usage and stop reason)
    #[serde(rename = "message_delta")]
    MessageDelta { delta: MessageDelta, usage: Usage },

    /// Message has stopped
    #[serde(rename = "message_stop")]
    MessageStop,

    /// Error event
    #[serde(rename = "error")]
    Error { error: ErrorDetail },
}

/// Delta types for streaming content
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Delta {
    /// Text delta
    #[serde(rename = "text_delta")]
    TextDelta { text: String },

    /// Input JSON delta (for tool use)
    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },

    /// Thinking delta (for extended thinking)
    #[serde(rename = "thinking_delta")]
    ThinkingDelta { thinking: String },

    /// Signature delta (for thinking)
    #[serde(rename = "signature_delta")]
    SignatureDelta { signature: String },
}

/// Message delta for final message updates
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageDelta {
    /// Stop reason for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,

    /// Stop sequence that caused the stop
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
}

/// SSE data line prefix
const SSE_DATA_PREFIX: &str = "data: ";
const SSE_EVENT_PREFIX: &str = "event: ";

/// Parse an SSE line into a StreamEvent
pub fn parse_sse_line(line: &str) -> Result<Option<StreamEvent>> {
    // Skip empty lines
    if line.trim().is_empty() {
        return Ok(None);
    }

    // Skip event type lines (we get the type from the JSON)
    if line.starts_with(SSE_EVENT_PREFIX) {
        return Ok(None);
    }

    // Parse data lines
    if let Some(data) = line.strip_prefix(SSE_DATA_PREFIX) {
        // Handle [DONE] signal
        if data.trim() == "[DONE]" {
            return Ok(None);
        }

        // Parse JSON
        let event: StreamEvent = serde_json::from_str(data)?;
        return Ok(Some(event));
    }

    Ok(None)
}

/// Stream accumulator for building complete response from streaming events
#[derive(Debug, Default)]
pub struct StreamAccumulator {
    /// Accumulated text content
    pub text: String,

    /// Accumulated tool use inputs (tool_id -> partial JSON)
    pub tool_inputs: std::collections::HashMap<String, String>,

    /// Accumulated thinking content
    pub thinking: String,

    /// Current content blocks
    pub content_blocks: Vec<ContentBlock>,

    /// Final usage
    pub usage: Option<Usage>,

    /// Stop reason
    pub stop_reason: Option<String>,

    /// Model ID
    pub model: Option<String>,

    /// Message ID
    pub id: Option<String>,
}

impl StreamAccumulator {
    /// Create a new accumulator
    pub fn new() -> Self {
        StreamAccumulator::default()
    }

    /// Process a stream event and update the accumulator
    pub fn process_event(&mut self, event: StreamEvent) {
        match event {
            StreamEvent::MessageStart { message } => {
                self.id = Some(message.id);
                self.model = Some(message.model);
            }
            StreamEvent::ContentBlockStart {
                content_block,
                index,
            } => {
                // Ensure we have enough slots
                while self.content_blocks.len() <= index {
                    self.content_blocks.push(ContentBlock::Text {
                        text: String::new(),
                        cache_control: None,
                    });
                }
                self.content_blocks[index] = content_block;
            }
            StreamEvent::ContentBlockDelta { index, delta } => match delta {
                Delta::TextDelta { text } => {
                    self.text.push_str(&text);
                    // Update the content block if it exists
                    if let Some(ContentBlock::Text {
                        text: block_text, ..
                    }) = self.content_blocks.get_mut(index)
                    {
                        block_text.push_str(&text);
                    }
                }
                Delta::InputJsonDelta { partial_json } => {
                    // For tool use, accumulate JSON
                    if let Some(ContentBlock::ToolUse { id, .. }) =
                        self.content_blocks.get(index)
                    {
                        self.tool_inputs
                            .entry(id.clone())
                            .or_default()
                            .push_str(&partial_json);
                    }
                }
                Delta::ThinkingDelta { thinking } => {
                    self.thinking.push_str(&thinking);
                }
                Delta::SignatureDelta { .. } => {
                    // Signatures are typically not accumulated
                }
            },
            StreamEvent::ContentBlockStop { .. } => {
                // Block finished, nothing to do
            }
            StreamEvent::MessageDelta { delta, usage } => {
                self.stop_reason = delta.stop_reason;
                self.usage = Some(usage);
            }
            StreamEvent::MessageStop => {
                // Message complete
            }
            StreamEvent::Ping => {
                // Keep-alive, ignore
            }
            StreamEvent::Error { .. } => {
                // Error handled separately
            }
        }
    }

    /// Get the accumulated text
    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// Check if streaming is complete
    pub fn is_complete(&self) -> bool {
        self.stop_reason.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_delta() {
        let line = r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
        let event = parse_sse_line(line).unwrap().unwrap();

        match event {
            StreamEvent::ContentBlockDelta { index, delta } => {
                assert_eq!(index, 0);
                match delta {
                    Delta::TextDelta { text } => assert_eq!(text, "Hello"),
                    _ => panic!("Expected TextDelta"),
                }
            }
            _ => panic!("Expected ContentBlockDelta"),
        }
    }

    #[test]
    fn test_parse_message_stop() {
        let line = r#"data: {"type":"message_stop"}"#;
        let event = parse_sse_line(line).unwrap().unwrap();

        assert!(matches!(event, StreamEvent::MessageStop));
    }

    #[test]
    fn test_parse_ping() {
        let line = r#"data: {"type":"ping"}"#;
        let event = parse_sse_line(line).unwrap().unwrap();

        assert!(matches!(event, StreamEvent::Ping));
    }

    #[test]
    fn test_parse_done() {
        let line = "data: [DONE]";
        let result = parse_sse_line(line).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_empty_line() {
        let line = "";
        let result = parse_sse_line(line).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_event_line() {
        let line = "event: message_start";
        let result = parse_sse_line(line).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_accumulator_text() {
        let mut acc = StreamAccumulator::new();

        acc.process_event(StreamEvent::ContentBlockStart {
            index: 0,
            content_block: ContentBlock::Text {
                text: String::new(),
                cache_control: None,
            },
        });

        acc.process_event(StreamEvent::ContentBlockDelta {
            index: 0,
            delta: Delta::TextDelta {
                text: "Hello ".to_string(),
            },
        });

        acc.process_event(StreamEvent::ContentBlockDelta {
            index: 0,
            delta: Delta::TextDelta {
                text: "world!".to_string(),
            },
        });

        assert_eq!(acc.get_text(), "Hello world!");
    }

    #[test]
    fn test_accumulator_complete() {
        let mut acc = StreamAccumulator::new();

        assert!(!acc.is_complete());

        acc.process_event(StreamEvent::MessageDelta {
            delta: MessageDelta {
                stop_reason: Some("end_turn".to_string()),
                stop_sequence: None,
            },
            usage: Usage::new(10, 5),
        });

        assert!(acc.is_complete());
        assert!(acc.usage.is_some());
    }
}
