//! Message types for conversations.
//!
//! This module provides types for constructing conversation messages:
//!
//! - [`Message`] - A message in the conversation with role and content
//! - [`SystemPrompt`] - System prompt configuration
//! - [`SystemBlock`] - Structured system prompt blocks
//!
//! # Creating Messages
//!
//! ```rust
//! use anthropic_tools::messages::request::message::Message;
//!
//! // Simple text messages
//! let user_msg = Message::user("Hello!");
//! let assistant_msg = Message::assistant("Hi there!");
//!
//! // Tool result
//! let result = Message::tool_result("tool_123", "Result data");
//! ```
//!
//! # With Images
//!
//! ```rust
//! use anthropic_tools::messages::request::message::Message;
//!
//! let msg = Message::user_with_image_url(
//!     "What's in this image?",
//!     "https://example.com/image.png"
//! );
//! ```
//!
//! # System Prompts
//!
//! ```rust
//! use anthropic_tools::messages::request::message::SystemPrompt;
//!
//! // Simple text
//! let system = SystemPrompt::text("You are a helpful assistant.");
//!
//! // With prompt caching
//! let cached = SystemPrompt::with_cache("Long system prompt...");
//! ```

use crate::messages::request::content::{CacheControl, ContentBlock, MediaType};
use crate::messages::request::role::Role;
use serde::{Deserialize, Serialize};

/// Message in a conversation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

impl Message {
    /// Create a new message with role and content blocks
    pub fn new(role: Role, content: Vec<ContentBlock>) -> Self {
        Message { role, content }
    }

    /// Create a user message with text content
    pub fn user<T: AsRef<str>>(text: T) -> Self {
        Message {
            role: Role::User,
            content: vec![ContentBlock::text(text)],
        }
    }

    /// Create an assistant message with text content
    pub fn assistant<T: AsRef<str>>(text: T) -> Self {
        Message {
            role: Role::Assistant,
            content: vec![ContentBlock::text(text)],
        }
    }

    /// Create a user message with an image from file path
    pub fn user_with_image<T: AsRef<str>>(text: T, media_type: MediaType, image_path: T) -> Self {
        Message {
            role: Role::User,
            content: vec![
                ContentBlock::image_from_path(media_type, image_path),
                ContentBlock::text(text),
            ],
        }
    }

    /// Create a user message with an image from URL
    pub fn user_with_image_url<T: AsRef<str>>(text: T, image_url: T) -> Self {
        Message {
            role: Role::User,
            content: vec![
                ContentBlock::image_from_url(image_url),
                ContentBlock::text(text),
            ],
        }
    }

    /// Create a user message with tool result
    pub fn tool_result<S: AsRef<str>>(tool_use_id: S, result_text: S) -> Self {
        Message {
            role: Role::User,
            content: vec![ContentBlock::tool_result_text(tool_use_id, result_text)],
        }
    }

    /// Create a user message with tool error result
    pub fn tool_error<S: AsRef<str>>(tool_use_id: S, error_message: S) -> Self {
        Message {
            role: Role::User,
            content: vec![ContentBlock::tool_result_error(tool_use_id, error_message)],
        }
    }

    /// Add a content block to the message
    pub fn add_content(&mut self, block: ContentBlock) -> &mut Self {
        self.content.push(block);
        self
    }

    /// Add text content to the message
    pub fn add_text<T: AsRef<str>>(&mut self, text: T) -> &mut Self {
        self.content.push(ContentBlock::text(text));
        self
    }

    /// Add image from path to the message
    pub fn add_image_from_path<T: AsRef<str>>(
        &mut self,
        media_type: MediaType,
        path: T,
    ) -> &mut Self {
        self.content
            .push(ContentBlock::image_from_path(media_type, path));
        self
    }

    /// Add image from URL to the message
    pub fn add_image_from_url<T: AsRef<str>>(&mut self, url: T) -> &mut Self {
        self.content.push(ContentBlock::image_from_url(url));
        self
    }
}

/// System prompt for the conversation
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SystemPrompt {
    /// Simple text system prompt
    Text(String),
    /// System prompt with content blocks (for caching)
    Blocks(Vec<SystemBlock>),
}

/// System block for structured system prompts
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemBlock {
    #[serde(rename = "type")]
    pub type_name: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl SystemPrompt {
    /// Create a simple text system prompt
    pub fn text<T: AsRef<str>>(text: T) -> Self {
        SystemPrompt::Text(text.as_ref().to_string())
    }

    /// Create a system prompt with cache control
    pub fn with_cache<T: AsRef<str>>(text: T) -> Self {
        SystemPrompt::Blocks(vec![SystemBlock {
            type_name: "text".to_string(),
            text: text.as_ref().to_string(),
            cache_control: Some(CacheControl::ephemeral()),
        }])
    }

    /// Create a system prompt from multiple blocks
    pub fn blocks(blocks: Vec<SystemBlock>) -> Self {
        SystemPrompt::Blocks(blocks)
    }
}

impl SystemBlock {
    /// Create a text block
    pub fn text<T: AsRef<str>>(text: T) -> Self {
        SystemBlock {
            type_name: "text".to_string(),
            text: text.as_ref().to_string(),
            cache_control: None,
        }
    }

    /// Create a text block with cache control
    pub fn text_with_cache<T: AsRef<str>>(text: T) -> Self {
        SystemBlock {
            type_name: "text".to_string(),
            text: text.as_ref().to_string(),
            cache_control: Some(CacheControl::ephemeral()),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::request::content::ImageSource;

    #[test]
    fn test_user_message() {
        let msg = Message::user("Hello!");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content.len(), 1);

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"text\":\"Hello!\""));
    }

    #[test]
    fn test_assistant_message() {
        let msg = Message::assistant("Hi there!");
        assert_eq!(msg.role, Role::Assistant);

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"assistant\""));
    }

    #[test]
    fn test_tool_result_message() {
        let msg = Message::tool_result("tool_123", "Result data");

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"tool_use_id\":\"tool_123\""));
    }

    #[test]
    fn test_system_prompt_text() {
        let system = SystemPrompt::text("You are a helpful assistant.");
        let json = serde_json::to_string(&system).unwrap();
        assert_eq!(json, "\"You are a helpful assistant.\"");
    }

    #[test]
    fn test_system_prompt_with_cache() {
        let system = SystemPrompt::with_cache("Cached system prompt");
        let json = serde_json::to_string(&system).unwrap();
        assert!(json.contains("\"cache_control\""));
        assert!(json.contains("\"type\":\"ephemeral\""));
    }

    #[test]
    fn test_message_builder() {
        let mut msg = Message::user("Initial text");
        msg.add_text("More text")
            .add_image_from_url("https://example.com/image.png");

        assert_eq!(msg.content.len(), 3);
    }

    #[tokio::test]
    async fn test_image_source_from_url_async() {
        // Test that async URL fetching works
        let source = ImageSource::from_url("https://example.com/image.png");
        assert_eq!(source.type_name, "url");
        assert!(source.url.is_some());
    }
}
