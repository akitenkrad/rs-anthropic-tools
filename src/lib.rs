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
//! - Adaptive thinking and effort control
//! - Structured outputs (JSON Schema)
//! - Refusal fallbacks and task budgets (beta)
//! - Token counting
//! - Files API
//! - SSE streaming
//!
//! ## Configuration
//!
//! Set the API key via environment variable or `.env` file:
//!
//! ```bash
//! # Environment variable
//! export ANTHROPIC_API_KEY="sk-ant-..."
//!
//! # Or create .env file in project root
//! echo 'ANTHROPIC_API_KEY=sk-ant-...' > .env
//! ```
//!
//! Priority: Environment variable > `.env` file > [`Messages::with_api_key()`]
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
//!         .model(Model::Opus5)  // Type-safe model selection
//!         .max_tokens(1024)
//!         .system("You are a helpful assistant.")
//!         .user("Hello, how are you?");
//!
//!     let response = client.post().await?;
//!
//!     // A refusal is a successful HTTP 200 — check before reading content
//!     if response.was_refused() {
//!         eprintln!("refused: {:?}", response.refusal_category());
//!         return Ok(());
//!     }
//!
//!     println!("{}", response.get_text());
//!     Ok(())
//! }
//! ```

pub mod common;
pub mod files;
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
        Messages,
        body::{
            Body, Effort, FallbackEntry, FallbackMode, Fallbacks, Metadata, OutputConfig,
            OutputFormat, TaskBudget, ThinkingConfig, ThinkingDisplay, ToolChoice,
        },
        content::{ContentBlock, DocumentSource, FallbackModelRef, ImageSource, MediaType},
        mcp::McpServer,
        message::{Message, SystemBlock, SystemPrompt},
        model::Model,
        role::Role,
    };

    // Response types
    pub use crate::messages::response::{Container, Response, StopDetails, StopReason, TokenCount};

    // Files API
    pub use crate::files::{
        FILES_API_BETA, FileDeleted, FileList, FileMetadata, Files, ListOptions,
    };

    // Streaming types
    pub use crate::messages::streaming::{Delta, MessageDelta, StreamAccumulator, StreamEvent};
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
            .model(Model::Opus5)
            .max_tokens(1024)
            .system("You are a helpful assistant.")
            .user("Hello!");

        let body = client.body();
        assert_eq!(body.model, Model::Opus5);
        assert_eq!(body.max_tokens, 1024);
        assert_eq!(body.messages.len(), 1);
    }

    #[test]
    fn test_messages_builder_with_string_model() {
        let mut client = Messages::with_api_key("test_key");
        client
            .model("claude-sonnet-5") // string still works
            .max_tokens(2048)
            .user("Test");

        let body = client.body();
        assert_eq!(body.model, Model::Sonnet5);
    }

    #[test]
    fn test_retired_model_string_still_parses() {
        // Backward compatibility: retired identifiers still map to their variant
        let mut client = Messages::with_api_key("test_key");
        client.model("claude-sonnet-4-20250514");

        let model = &client.body().model;
        assert_eq!(model, &Model::Sonnet4);
        assert!(model.is_retired());
        assert_eq!(model.replacement(), Some(Model::Sonnet5));
    }

    #[test]
    fn test_current_api_builder() {
        let mut client = Messages::with_api_key("test_key");
        client
            .model(Model::Opus5)
            .max_tokens(64000)
            .thinking_summarized()
            .effort(Effort::XHigh)
            .user("Solve this.");

        let body = client.body();
        assert_eq!(body.effort(), Some(Effort::XHigh));
        assert!(body.thinking.as_ref().unwrap().is_adaptive());
        assert!(body.validate().is_ok());
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
