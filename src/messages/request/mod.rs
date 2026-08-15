//! Request types for the Messages API.
//!
//! This module provides the [`Messages`] client and related request types:
//!
//! - [`Messages`] - Main API client with builder pattern
//! - [`body`] - Request body structure and validation
//! - [`content`] - Content block types (text, image, tool use, etc.)
//! - [`message`] - Message and system prompt types
//! - [`model`] - Type-safe model selection
//! - [`role`] - User and assistant roles
//! - [`mcp`] - MCP server configuration (beta)
//!
//! # Builder Pattern
//!
//! The [`Messages`] client uses a builder pattern for constructing requests:
//!
//! ```rust
//! use anthropic_tools::prelude::*;
//!
//! let mut client = Messages::new();
//! client
//!     .model(Model::Opus5)
//!     .max_tokens(16000)
//!     .effort(Effort::High)
//!     .system("You are a helpful assistant.")
//!     .user("Hello!");
//! ```
//!
//! # Thinking
//!
//! Current models use *adaptive* thinking, where Claude decides when and how
//! much to reason. Depth is controlled with [`Effort`] rather
//! than a fixed token budget:
//!
//! ```rust
//! use anthropic_tools::prelude::*;
//!
//! let mut client = Messages::new();
//! client
//!     .model(Model::Opus5)
//!     .max_tokens(64000)
//!     .thinking_adaptive()
//!     .effort(Effort::XHigh)
//!     .user("Solve this complex problem...");
//! ```
//!
//! # Multi-turn Conversations
//!
//! ```rust
//! use anthropic_tools::prelude::*;
//!
//! let mut client = Messages::new();
//! client
//!     .model(Model::Opus5)
//!     .max_tokens(1024)
//!     .user("What is 2+2?")
//!     .assistant("2+2 equals 4.")
//!     .user("And 3+3?");
//! ```

pub mod body;
pub mod content;
pub mod mcp;
pub mod message;
pub mod model;
pub mod role;

use crate::common::errors::{AnthropicToolError, Result};
use crate::messages::response::{Response, TokenCount};
use std::env;

// Re-export for internal use
use body::{
    Body, Effort, Fallbacks, Metadata, OutputConfig, OutputFormat, TaskBudget, ThinkingConfig,
    ThinkingDisplay, ToolChoice,
};
use content::MediaType;
use mcp::McpServer;
use message::{Message, SystemPrompt};
use model::Model;

/// API endpoint for Anthropic Messages API
const MESSAGES_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// API endpoint for token counting
const COUNT_TOKENS_API_URL: &str = "https://api.anthropic.com/v1/messages/count_tokens";

/// Current Anthropic API version
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Beta header required by task budgets
const TASK_BUDGETS_BETA: &str = "task-budgets-2026-03-13";

/// Fields that the token-counting endpoint does not accept
const COUNT_TOKENS_EXCLUDED_FIELDS: &[&str] = &[
    "max_tokens",
    "stream",
    "metadata",
    "container",
    "fallbacks",
    "stop_sequences",
];

/// Messages API client with builder pattern
#[derive(Debug, Clone)]
pub struct Messages {
    api_key: String,
    request_body: Body,
    betas: Vec<String>,
}

impl Default for Messages {
    fn default() -> Self {
        Self::new()
    }
}

impl Messages {
    /// Create a new Messages client
    ///
    /// Loads API key from ANTHROPIC_API_KEY environment variable.
    /// Also loads from `.env` file if present (does not override existing env vars).
    ///
    /// # Priority
    ///
    /// 1. Existing environment variable (highest priority)
    /// 2. `.env` file (if env var is not set)
    /// 3. [`with_api_key()`](Self::with_api_key) for explicit override
    pub fn new() -> Self {
        // Load .env file (ignore errors if file doesn't exist)
        let _ = dotenvy::dotenv();

        let api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        Messages {
            api_key,
            request_body: Body::default(),
            betas: Vec::new(),
        }
    }

    /// Create a new Messages client with explicit API key
    pub fn with_api_key<T: AsRef<str>>(api_key: T) -> Self {
        Messages {
            api_key: api_key.as_ref().to_string(),
            request_body: Body::default(),
            betas: Vec::new(),
        }
    }

    /// Set the model to use
    ///
    /// Accepts both [`Model`] enum variants and string types for backward compatibility.
    ///
    /// # Example
    ///
    /// ```rust
    /// use anthropic_tools::prelude::*;
    ///
    /// let mut client = Messages::new();
    ///
    /// // Using enum (recommended)
    /// client.model(Model::Opus5);
    ///
    /// // Using string (backward compatible)
    /// client.model("claude-sonnet-5");
    /// ```
    pub fn model<T: Into<Model>>(&mut self, model: T) -> &mut Self {
        self.request_body.model = model.into();
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
    ///
    /// Note that a *trailing* assistant message is an assistant-turn prefill,
    /// which current models reject. See
    /// [`Model::supports_prefill`](model::Model::supports_prefill).
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
    ///
    /// Rejected by Opus 4.7 and later, Claude Opus 5, Claude Sonnet 5, and
    /// Claude Fable 5 — [`post`](Self::post) returns an error before sending.
    /// Steer those models via prompting instead.
    pub fn temperature(&mut self, temperature: f32) -> &mut Self {
        self.request_body.temperature = Some(temperature);
        self
    }

    /// Set top_p sampling parameter
    ///
    /// Subject to the same model restrictions as [`temperature`](Self::temperature).
    pub fn top_p(&mut self, top_p: f32) -> &mut Self {
        self.request_body.top_p = Some(top_p);
        self
    }

    /// Set top_k sampling parameter
    ///
    /// Subject to the same model restrictions as [`temperature`](Self::temperature).
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

    /// Set MCP servers (beta)
    pub fn mcp_servers(&mut self, servers: Vec<McpServer>) -> &mut Self {
        self.request_body.mcp_servers = Some(servers);
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

    // --- thinking ---------------------------------------------------------

    /// Enable legacy extended thinking with a fixed token budget
    ///
    /// Prefer [`thinking_adaptive`](Self::thinking_adaptive). This form is
    /// rejected by Opus 4.7 and later, Claude Opus 5, Claude Sonnet 5, and
    /// Claude Fable 5, and [`post`](Self::post) returns an error before
    /// sending on those models.
    ///
    /// # Arguments
    ///
    /// * `budget_tokens` - Token budget for thinking (must be >= 1024 and < max_tokens)
    ///
    /// # Example
    ///
    /// ```rust
    /// use anthropic_tools::prelude::*;
    ///
    /// let mut client = Messages::new();
    /// client
    ///     .model(Model::Haiku45)
    ///     .max_tokens(16000)
    ///     .thinking(10000)
    ///     .user("Solve this complex problem...");
    /// ```
    pub fn thinking(&mut self, budget_tokens: usize) -> &mut Self {
        self.request_body.thinking = Some(ThinkingConfig::enabled(budget_tokens));
        self
    }

    /// Enable adaptive thinking
    ///
    /// Claude decides when and how much to think. Combine with
    /// [`effort`](Self::effort) to control depth.
    pub fn thinking_adaptive(&mut self) -> &mut Self {
        self.request_body.thinking = Some(ThinkingConfig::adaptive());
        self
    }

    /// Enable adaptive thinking and request a readable summary of the reasoning
    ///
    /// Without this, thinking blocks arrive with empty text, which looks like a
    /// long pause when streaming reasoning to users.
    pub fn thinking_summarized(&mut self) -> &mut Self {
        self.request_body.thinking = Some(ThinkingConfig::adaptive_with_display(
            ThinkingDisplay::Summarized,
        ));
        self
    }

    /// Turn thinking off
    ///
    /// Rejected on Claude Fable 5 and Claude Mythos 5 (which always think), and
    /// on Claude Opus 5 at effort `xhigh` or `max`.
    pub fn thinking_disabled(&mut self) -> &mut Self {
        self.request_body.thinking = Some(ThinkingConfig::disabled());
        self
    }

    /// Set an explicit thinking configuration
    pub fn thinking_config(&mut self, config: ThinkingConfig) -> &mut Self {
        self.request_body.thinking = Some(config);
        self
    }

    // --- output config ----------------------------------------------------

    fn output_config_mut(&mut self) -> &mut OutputConfig {
        self.request_body
            .output_config
            .get_or_insert_with(OutputConfig::default)
    }

    /// Set the effort level (reasoning depth and token spend)
    ///
    /// The API default is [`Effort::High`]. Use [`Effort::XHigh`] for coding
    /// and agentic work, and [`Effort::Low`] or [`Effort::Medium`] for routine
    /// or latency-sensitive tasks.
    pub fn effort(&mut self, effort: Effort) -> &mut Self {
        self.output_config_mut().effort = Some(effort);
        self
    }

    /// Constrain the response to a JSON Schema (structured outputs)
    ///
    /// # Example
    ///
    /// ```rust
    /// use anthropic_tools::prelude::*;
    /// use serde_json::json;
    ///
    /// let mut client = Messages::new();
    /// client
    ///     .model(Model::Opus5)
    ///     .max_tokens(1024)
    ///     .json_schema(json!({
    ///         "type": "object",
    ///         "properties": {"name": {"type": "string"}},
    ///         "required": ["name"],
    ///         "additionalProperties": false
    ///     }))
    ///     .user("Extract the name from: Jane Doe <jane@example.com>");
    /// ```
    pub fn json_schema(&mut self, schema: serde_json::Value) -> &mut Self {
        self.output_config_mut().format = Some(OutputFormat::json_schema(schema));
        self
    }

    /// Set the response output format
    pub fn output_format(&mut self, format: OutputFormat) -> &mut Self {
        self.output_config_mut().format = Some(format);
        self
    }

    /// Set a task budget for an agentic loop (beta)
    ///
    /// Automatically adds the required beta header. The minimum total is 20,000
    /// tokens. Unlike `max_tokens`, the model is aware of this budget and paces
    /// itself against it.
    pub fn task_budget(&mut self, budget: TaskBudget) -> &mut Self {
        self.output_config_mut().task_budget = Some(budget);
        self.beta(TASK_BUDGETS_BETA)
    }

    /// Set the complete output configuration, replacing any existing one
    pub fn output_config(&mut self, config: OutputConfig) -> &mut Self {
        self.request_body.output_config = Some(config);
        self
    }

    // --- fallbacks and beta headers ---------------------------------------

    /// Configure server-side refusal fallbacks (beta)
    ///
    /// Automatically adds the beta header the chosen form requires. When safety
    /// classifiers decline a request, the API re-runs it on a fallback model
    /// inside the same call instead of returning the refusal.
    ///
    /// # Example
    ///
    /// ```rust
    /// use anthropic_tools::prelude::*;
    ///
    /// let mut client = Messages::new();
    /// client
    ///     .model(Model::Opus5)
    ///     .max_tokens(1024)
    ///     .fallbacks(Fallbacks::auto())
    ///     .user("Hello");
    /// ```
    pub fn fallbacks(&mut self, fallbacks: Fallbacks) -> &mut Self {
        let header = fallbacks.beta_header();
        self.request_body.fallbacks = Some(fallbacks);
        self.beta(header)
    }

    /// Add an `anthropic-beta` feature flag
    ///
    /// Duplicate flags are ignored.
    pub fn beta<T: AsRef<str>>(&mut self, beta: T) -> &mut Self {
        let beta = beta.as_ref().to_string();
        if !self.betas.contains(&beta) {
            self.betas.push(beta);
        }
        self
    }

    /// Get the currently configured beta feature flags
    pub fn betas(&self) -> &[String] {
        &self.betas
    }

    // --- transport --------------------------------------------------------

    /// Build HTTP headers for the request
    fn build_headers(&self) -> Result<request::header::HeaderMap> {
        use request::header::{HeaderMap, HeaderValue};

        let invalid = |field: &str| {
            AnthropicToolError::InvalidParameter(format!(
                "{} contains invalid header characters",
                field
            ))
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key).map_err(|_| invalid("API key"))?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(ANTHROPIC_VERSION),
        );
        headers.insert("content-type", HeaderValue::from_static("application/json"));

        if !self.betas.is_empty() {
            let joined = self.betas.join(",");
            headers.insert(
                "anthropic-beta",
                HeaderValue::from_str(&joined).map_err(|_| invalid("beta header"))?,
            );
        }

        Ok(headers)
    }

    /// Send the request and get a response
    ///
    /// Note that a refusal is a *successful* HTTP 200 response — check
    /// [`Response::was_refused`](crate::messages::response::Response::was_refused)
    /// before reading the content.
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
            .headers(self.build_headers()?)
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

    /// Count the input tokens this request would consume, without running it
    ///
    /// Token counts are model-specific — the same text tokenizes differently
    /// across model generations, so count against the model you will actually
    /// call.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use anthropic_tools::prelude::*;
    ///
    /// # async fn example() -> Result<()> {
    /// let mut client = Messages::new();
    /// client.model(Model::Opus5).max_tokens(1024).user("Hello!");
    ///
    /// let count = client.count_tokens().await?;
    /// println!("{} input tokens", count.input_tokens);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn count_tokens(&self) -> Result<TokenCount> {
        if self.api_key.is_empty() {
            return Err(AnthropicToolError::ApiKeyNotSet);
        }

        self.request_body.validate()?;

        // The count_tokens endpoint rejects generation-only parameters
        let mut payload = serde_json::to_value(&self.request_body)?;
        if let Some(object) = payload.as_object_mut() {
            for field in COUNT_TOKENS_EXCLUDED_FIELDS {
                object.remove(*field);
            }
        }

        let client = request::Client::new();
        let response = client
            .post(COUNT_TOKENS_API_URL)
            .headers(self.build_headers()?)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effort_sets_output_config() {
        let mut client = Messages::with_api_key("test");
        client.model(Model::Opus5).effort(Effort::XHigh);

        assert_eq!(client.body().effort(), Some(Effort::XHigh));
    }

    #[test]
    fn test_effort_and_json_schema_share_output_config() {
        let mut client = Messages::with_api_key("test");
        client
            .model(Model::Opus5)
            .effort(Effort::High)
            .json_schema(serde_json::json!({"type": "object"}));

        let config = client.body().output_config.as_ref().unwrap();
        assert_eq!(config.effort, Some(Effort::High));
        assert!(config.format.is_some());
    }

    #[test]
    fn test_thinking_adaptive_builder() {
        let mut client = Messages::with_api_key("test");
        client.model(Model::Opus5).thinking_adaptive();

        assert!(client.body().thinking.as_ref().unwrap().is_adaptive());
    }

    #[test]
    fn test_thinking_summarized_builder() {
        let mut client = Messages::with_api_key("test");
        client.model(Model::Opus5).thinking_summarized();

        let json = serde_json::to_string(client.body().thinking.as_ref().unwrap()).unwrap();
        assert!(json.contains("\"display\":\"summarized\""));
    }

    #[test]
    fn test_task_budget_adds_beta_header() {
        let mut client = Messages::with_api_key("test");
        client
            .model(Model::Opus5)
            .task_budget(TaskBudget::tokens(64_000));

        assert!(client.betas().contains(&TASK_BUDGETS_BETA.to_string()));
    }

    #[test]
    fn test_fallbacks_adds_beta_header() {
        let mut client = Messages::with_api_key("test");
        client.model(Model::Opus5).fallbacks(Fallbacks::auto());

        assert!(
            client
                .betas()
                .contains(&"server-side-fallback-2026-07-01".to_string())
        );

        let mut client = Messages::with_api_key("test");
        client
            .model(Model::Opus5)
            .fallbacks(Fallbacks::list(vec![body::FallbackEntry::new(
                Model::Opus48,
            )]));

        assert!(
            client
                .betas()
                .contains(&"server-side-fallback-2026-06-01".to_string())
        );
    }

    #[test]
    fn test_beta_deduplicates() {
        let mut client = Messages::with_api_key("test");
        client.beta("feature-a").beta("feature-a").beta("feature-b");

        assert_eq!(client.betas(), &["feature-a", "feature-b"]);
    }

    #[test]
    fn test_build_headers_includes_betas() {
        let mut client = Messages::with_api_key("test");
        client.beta("feature-a").beta("feature-b");

        let headers = client.build_headers().unwrap();
        assert_eq!(
            headers.get("anthropic-beta").unwrap(),
            "feature-a,feature-b"
        );
        assert_eq!(headers.get("anthropic-version").unwrap(), ANTHROPIC_VERSION);
    }

    #[test]
    fn test_build_headers_omits_beta_when_empty() {
        let client = Messages::with_api_key("test");
        let headers = client.build_headers().unwrap();
        assert!(headers.get("anthropic-beta").is_none());
    }

    #[test]
    fn test_build_headers_rejects_invalid_api_key() {
        let client = Messages::with_api_key("bad\nkey");
        assert!(client.build_headers().is_err());
    }

    #[test]
    fn test_count_tokens_payload_excludes_generation_fields() {
        let mut client = Messages::with_api_key("test");
        client
            .model(Model::Opus5)
            .max_tokens(1024)
            .stream(true)
            .user("Hello");

        let mut payload = serde_json::to_value(client.body()).unwrap();
        let object = payload.as_object_mut().unwrap();
        for field in COUNT_TOKENS_EXCLUDED_FIELDS {
            object.remove(*field);
        }

        assert!(object.contains_key("model"));
        assert!(object.contains_key("messages"));
        assert!(!object.contains_key("max_tokens"));
        assert!(!object.contains_key("stream"));
    }
}
