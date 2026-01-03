//! # Anthropic Tools
//!
//! A Rust library for interacting with the Anthropic API.
//!
//! ## Features
//!
//! - Messages API with builder pattern
//! - Tool/Function calling support
//! - Vision/Multimodal support
//! - Prompt caching support
//! - Streaming support (planned)
//!
//! ## Example
//!
//! ```rust,no_run
//! use anthropic_tools::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut client = Messages::new();
//!     client
//!         .model("claude-sonnet-4-20250514")
//!         .max_tokens(1024)
//!         .system("You are a helpful assistant.")
//!         .user("Hello, how are you?");
//!
//!     let response = client.post().await?;
//!     println!("{}", response.get_text());
//!     Ok(())
//! }
//! ```

pub mod common;
pub mod messages;

/// Commonly used types and traits
pub mod prelude {
    // Error types
    pub use crate::common::errors::{AnthropicToolError, Result};

    // Usage
    pub use crate::common::usage::Usage;

    // Tool definitions
    pub use crate::common::tool::{CacheControl, JsonSchema, PropertyDef, Tool};

    // Messages API
    pub use crate::messages::request::{
        body::{Body, Metadata, ToolChoice},
        content::{ContentBlock, DocumentSource, ImageSource, MediaType},
        message::{Message, SystemBlock, SystemPrompt},
        role::Role,
        Messages,
    };

    // Response types
    pub use crate::messages::response::{Response, StopReason};

    // Streaming types
    pub use crate::messages::streaming::{
        Delta, MessageDelta, StreamAccumulator, StreamEvent,
    };
}

// Re-export main types at crate level
pub use common::{AnthropicToolError, Result, Tool, Usage};
pub use messages::request::Messages;
pub use messages::response::Response;

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_messages_builder() {
        let mut client = Messages::with_api_key("test_key");
        client
            .model("claude-sonnet-4-20250514")
            .max_tokens(1024)
            .system("You are a helpful assistant.")
            .user("Hello!");

        let body = client.body();
        assert_eq!(body.model, "claude-sonnet-4-20250514");
        assert_eq!(body.max_tokens, 1024);
        assert_eq!(body.messages.len(), 1);
    }

    #[test]
    fn test_tool_builder() {
        let mut tool = Tool::new("search");
        tool.description("Search for information")
            .add_string_property("query", Some("Search query"), true);

        assert_eq!(tool.name, "search");
        assert!(tool.input_schema.properties.is_some());
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello!");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content.len(), 1);
    }

    #[test]
    fn test_content_block() {
        let block = ContentBlock::text("Test text");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("\"text\":\"Test text\""));
    }
}
