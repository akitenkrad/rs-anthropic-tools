//! Request types for the Messages API.
//!
//! This module provides the [`Messages`] client and related request types:
//!
//! - [`Messages`] - Main API client with builder pattern
//! - [`body`] - Request body structure and validation
//! - [`content`] - Content block types (text, image, tool use, etc.)
//! - [`message`] - Message and system prompt types
//! - [`role`] - User and assistant roles
//! - [`mcp`] - MCP server configuration (beta)
//!
//! # Builder Pattern
//!
//! The [`Messages`] client uses a builder pattern for constructing requests:
//!
//! ```rust
//! use anthropic_tools::messages::request::Messages;
//!
//! let mut client = Messages::new();
//! client
//!     .model("claude-sonnet-4-20250514")
//!     .max_tokens(1024)
//!     .temperature(0.7)
//!     .system("You are a helpful assistant.")
//!     .user("Hello!");
//! ```
//!
//! # Multi-turn Conversations
//!
//! ```rust
//! use anthropic_tools::messages::request::{Messages, message::Message};
//!
//! let mut client = Messages::new();
//! client
//!     .model("claude-sonnet-4-20250514")
//!     .max_tokens(1024)
//!     .user("What is 2+2?")
//!     .assistant("2+2 equals 4.")
//!     .user("And 3+3?");
//! ```

pub mod body;
pub mod content;
pub mod mcp;
pub mod message;
pub mod role;

use crate::common::errors::{AnthropicToolError, Result};
use crate::messages::response::Response;
use std::env;

// Re-export for internal use
use body::{Body, Metadata, ToolChoice};
use content::MediaType;
use message::{Message, SystemPrompt};

/// API endpoint for Anthropic Messages API
const MESSAGES_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Current Anthropic API version
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Messages API client with builder pattern
#[derive(Debug, Clone)]
pub struct Messages {
    api_key: String,
    request_body: Body,
}

impl Default for Messages {
    fn default() -> Self {
        Self::new()
    }
}

impl Messages {
    /// Create a new Messages client
    ///
    /// Loads API key from ANTHROPIC_API_KEY environment variable
    pub fn new() -> Self {
        let api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        Messages {
            api_key,
            request_body: Body::default(),
        }
    }

    /// Create a new Messages client with explicit API key
    pub fn with_api_key<T: AsRef<str>>(api_key: T) -> Self {
        Messages {
            api_key: api_key.as_ref().to_string(),
            request_body: Body::default(),
        }
    }

    /// Set the model to use
    pub fn model<T: AsRef<str>>(&mut self, model: T) -> &mut Self {
        self.request_body.model = model.as_ref().to_string();
        self
    }

    /// Set the maximum number of tokens to generate
    pub fn max_tokens(&mut self, max_tokens: usize) -> &mut Self {
        self.request_body.max_tokens = max_tokens;
        self
    }

    /// Set the system prompt
    pub fn system<T: AsRef<str>>(&mut self, system: T) -> &mut Self {
        self.request_body.system = Some(SystemPrompt::text(system));
        self
    }

    /// Set the system prompt with cache control
    pub fn system_with_cache<T: AsRef<str>>(&mut self, system: T) -> &mut Self {
        self.request_body.system = Some(SystemPrompt::with_cache(system));
        self
    }

    /// Set the messages
    pub fn messages(&mut self, messages: Vec<Message>) -> &mut Self {
        self.request_body.messages = messages;
        self
    }

    /// Add a message
    pub fn add_message(&mut self, message: Message) -> &mut Self {
        self.request_body.messages.push(message);
        self
    }

    /// Add a user text message
    pub fn user<T: AsRef<str>>(&mut self, text: T) -> &mut Self {
        self.request_body.messages.push(Message::user(text));
        self
    }

    /// Add an assistant text message
    pub fn assistant<T: AsRef<str>>(&mut self, text: T) -> &mut Self {
        self.request_body.messages.push(Message::assistant(text));
        self
    }

    /// Add a user message with image from path
    pub fn user_with_image<T: AsRef<str>>(
        &mut self,
        text: T,
        media_type: MediaType,
        image_path: T,
    ) -> &mut Self {
        self.request_body
            .messages
            .push(Message::user_with_image(text, media_type, image_path));
        self
    }

    /// Add a user message with image from URL
    pub fn user_with_image_url<T: AsRef<str>>(&mut self, text: T, image_url: T) -> &mut Self {
        self.request_body
            .messages
            .push(Message::user_with_image_url(text, image_url));
        self
    }

    /// Add a tool result message
    pub fn tool_result<S: AsRef<str>>(&mut self, tool_use_id: S, result_text: S) -> &mut Self {
        self.request_body
            .messages
            .push(Message::tool_result(tool_use_id, result_text));
        self
    }

    /// Add a tool error result message
    pub fn tool_error<S: AsRef<str>>(&mut self, tool_use_id: S, error_message: S) -> &mut Self {
        self.request_body
            .messages
            .push(Message::tool_error(tool_use_id, error_message));
        self
    }

    /// Set the sampling temperature (0.0 to 1.0)
    pub fn temperature(&mut self, temperature: f32) -> &mut Self {
        self.request_body.temperature = Some(temperature);
        self
    }

    /// Set top_p sampling parameter
    pub fn top_p(&mut self, top_p: f32) -> &mut Self {
        self.request_body.top_p = Some(top_p);
        self
    }

    /// Set top_k sampling parameter
    pub fn top_k(&mut self, top_k: u32) -> &mut Self {
        self.request_body.top_k = Some(top_k);
        self
    }

    /// Set stop sequences
    pub fn stop_sequences(&mut self, sequences: Vec<String>) -> &mut Self {
        self.request_body.stop_sequences = Some(sequences);
        self
    }

    /// Set tools available to the model
    pub fn tools(&mut self, tools: Vec<serde_json::Value>) -> &mut Self {
        self.request_body.tools = Some(tools);
        self
    }

    /// Set tool choice
    pub fn tool_choice(&mut self, choice: ToolChoice) -> &mut Self {
        self.request_body.tool_choice = Some(choice);
        self
    }

    /// Set user ID for metadata
    pub fn user_id<T: AsRef<str>>(&mut self, user_id: T) -> &mut Self {
        self.request_body.metadata = Some(Metadata {
            user_id: Some(user_id.as_ref().to_string()),
        });
        self
    }

    /// Enable streaming
    pub fn stream(&mut self, enabled: bool) -> &mut Self {
        self.request_body.stream = Some(enabled);
        self
    }

    /// Set container for code execution (beta)
    pub fn container<T: AsRef<str>>(&mut self, container: T) -> &mut Self {
        self.request_body.container = Some(container.as_ref().to_string());
        self
    }

    /// Build HTTP headers for the request
    fn build_headers(&self) -> request::header::HeaderMap {
        let mut headers = request::header::HeaderMap::new();
        headers.insert("x-api-key", self.api_key.parse().unwrap());
        headers.insert("anthropic-version", ANTHROPIC_VERSION.parse().unwrap());
        headers.insert("content-type", "application/json".parse().unwrap());
        headers
    }

    /// Send the request and get a response
    pub async fn post(&self) -> Result<Response> {
        // Validate API key
        if self.api_key.is_empty() {
            return Err(AnthropicToolError::ApiKeyNotSet);
        }

        // Validate request body
        self.request_body.validate()?;

        // Build and send request
        let client = request::Client::new();
        let response = client
            .post(MESSAGES_API_URL)
            .headers(self.build_headers())
            .json(&self.request_body)
            .send()
            .await?;

        // Handle response
        if response.status().is_success() {
            let response_body: Response = response.json().await?;
            Ok(response_body)
        } else {
            let error_response: crate::common::errors::ErrorResponse = response.json().await?;
            Err(error_response.into_error())
        }
    }

    /// Get a reference to the request body (for debugging)
    pub fn body(&self) -> &Body {
        &self.request_body
    }
}

