//! Request body structure for the Messages API.
//!
//! This module provides the [`Body`] struct that represents the JSON payload
//! sent to the Anthropic API:
//!
//! - [`Body`] - Complete request body with all parameters
//! - [`ToolChoice`] - Configuration for tool selection behavior
//! - [`ThinkingConfig`] - Thinking configuration (adaptive / disabled / legacy budget)
//! - [`OutputConfig`] - Effort, structured outputs, and task budgets
//! - [`Effort`] - Reasoning-depth and token-spend control
//! - [`Fallbacks`] - Server-side refusal fallbacks (beta)
//! - [`Metadata`] - Optional request metadata
//!
//! # Request Parameters
//!
//! Required:
//! - `model` - Model identifier (e.g., `"claude-opus-5"`)
//! - `messages` - Conversation messages
//! - `max_tokens` - Maximum tokens to generate
//!
//! Optional:
//! - `system` - System prompt
//! - `temperature`, `top_p`, `top_k` - Sampling parameters (**rejected** on
//!   Opus 4.7 and later, Claude Opus 5, Claude Sonnet 5, and Claude Fable 5)
//! - `stop_sequences` - Custom stop sequences
//! - `tools` - Available tools for function calling
//! - `stream` - Enable streaming responses
//! - `thinking` - Thinking configuration
//! - `output_config` - Effort, output format, and task budget
//! - `fallbacks` - Refusal fallback models (beta)
//!
//! # Example
//!
//! ```rust
//! use anthropic_tools::messages::request::{body::Body, model::Model};
//!
//! // Using Model enum (recommended)
//! let body = Body::new(Model::Opus5, 1024);
//! assert_eq!(body.model, Model::Opus5);
//!
//! // Using string (backward compatible)
//! let body = Body::new("claude-sonnet-5", 1024);
//! assert_eq!(body.model, Model::Sonnet5);
//! assert_eq!(body.max_tokens, 1024);
//! ```

use crate::common::errors::{AnthropicToolError, Result};
use crate::messages::request::model::Model;
use crate::messages::request::role::Role;
use crate::messages::request::{mcp::McpServer, message::Message, message::SystemPrompt};
use serde::{Deserialize, Serialize};

/// Request body for the Messages API
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body {
    /// The model to use
    pub model: Model,

    /// Input messages for the conversation
    pub messages: Vec<Message>,

    /// Maximum number of tokens to generate (required)
    pub max_tokens: usize,

    /// System prompt (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,

    /// Sampling temperature (0.0 to 1.0)
    ///
    /// Rejected with a `400` on Opus 4.7 and later, Claude Opus 5,
    /// Claude Sonnet 5, Claude Fable 5, and Claude Mythos 5.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top-p sampling parameter
    ///
    /// Rejected on the same models as [`Body::temperature`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Top-k sampling parameter
    ///
    /// Rejected on the same models as [`Body::temperature`].
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

    /// Thinking configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,

    /// Output configuration: effort, response format, and task budget
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_config: Option<OutputConfig>,

    /// Refusal fallbacks (beta)
    ///
    /// Requires a beta header, which
    /// [`Messages`](crate::messages::request::Messages) adds automatically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<Fallbacks>,
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

/// Visibility of thinking content in the response
///
/// This controls visibility only — thinking happens and is billed the same
/// under every setting, and the raw chain of thought is never returned.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingDisplay {
    /// Thinking blocks are returned with empty text (default on current models)
    #[default]
    Omitted,

    /// A readable summary of the reasoning is returned
    Summarized,
}

/// Thinking configuration
///
/// Controls whether and how the model reasons internally before responding.
///
/// # Choosing a variant
///
/// - [`ThinkingConfig::Adaptive`] — **recommended**. Claude decides when and
///   how much to think. Combine with [`Effort`] to control depth.
/// - [`ThinkingConfig::Disabled`] — turn thinking off. Rejected on Claude
///   Fable 5 and Claude Mythos 5, and on Claude Opus 5 at effort `xhigh`/`max`.
/// - [`ThinkingConfig::Enabled`] — **legacy**. A fixed token budget. Rejected
///   with a `400` on Opus 4.7 and later, Claude Opus 5, Claude Sonnet 5, and
///   Claude Fable 5. Kept for models that still accept it.
///
/// # Example
///
/// ```rust
/// use anthropic_tools::messages::request::body::{ThinkingConfig, ThinkingDisplay};
///
/// // Recommended on current models
/// let config = ThinkingConfig::adaptive();
/// assert!(config.is_adaptive());
///
/// // Surface a readable summary of the reasoning
/// let config = ThinkingConfig::adaptive_with_display(ThinkingDisplay::Summarized);
///
/// // Legacy fixed budget (older models only)
/// let config = ThinkingConfig::enabled(10000);
/// assert_eq!(config.budget_tokens_opt(), Some(10000));
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum ThinkingConfig {
    /// Legacy extended thinking with a fixed token budget
    ///
    /// Rejected on Opus 4.7 and later, Claude Opus 5, Claude Sonnet 5,
    /// Claude Fable 5, and Claude Mythos 5. Use
    /// [`ThinkingConfig::Adaptive`] with [`Effort`] instead.
    #[serde(rename = "enabled")]
    Enabled {
        /// Token budget for thinking (must be >= 1024 and < max_tokens)
        budget_tokens: usize,
    },

    /// Adaptive thinking — Claude decides when and how much to think
    #[serde(rename = "adaptive")]
    Adaptive {
        /// Visibility of thinking content in the response
        #[serde(skip_serializing_if = "Option::is_none")]
        display: Option<ThinkingDisplay>,
    },

    /// Thinking turned off
    #[serde(rename = "disabled")]
    Disabled,
}

impl ThinkingConfig {
    /// Create a legacy fixed-budget thinking configuration
    ///
    /// Prefer [`ThinkingConfig::adaptive`] — this form is rejected on Opus 4.7
    /// and later, Claude Opus 5, Claude Sonnet 5, and Claude Fable 5.
    ///
    /// # Arguments
    ///
    /// * `budget_tokens` - Token budget for thinking (must be >= 1024)
    pub fn enabled(budget_tokens: usize) -> Self {
        ThinkingConfig::Enabled { budget_tokens }
    }

    /// Create an adaptive thinking configuration
    pub fn adaptive() -> Self {
        ThinkingConfig::Adaptive { display: None }
    }

    /// Create an adaptive thinking configuration with an explicit display mode
    pub fn adaptive_with_display(display: ThinkingDisplay) -> Self {
        ThinkingConfig::Adaptive {
            display: Some(display),
        }
    }

    /// Create a disabled thinking configuration
    pub fn disabled() -> Self {
        ThinkingConfig::Disabled
    }

    /// Get the budget tokens, if this is a fixed-budget configuration
    pub fn budget_tokens_opt(&self) -> Option<usize> {
        match self {
            ThinkingConfig::Enabled { budget_tokens } => Some(*budget_tokens),
            _ => None,
        }
    }

    /// Get the budget tokens, returning `0` for non-budget configurations
    ///
    /// Prefer [`ThinkingConfig::budget_tokens_opt`], which distinguishes
    /// "no budget" from "a budget of zero".
    pub fn budget_tokens(&self) -> usize {
        self.budget_tokens_opt().unwrap_or(0)
    }

    /// Check if this is an adaptive configuration
    pub fn is_adaptive(&self) -> bool {
        matches!(self, ThinkingConfig::Adaptive { .. })
    }

    /// Check if this configuration turns thinking off
    pub fn is_disabled(&self) -> bool {
        matches!(self, ThinkingConfig::Disabled)
    }
}

/// Reasoning-depth and token-spend control
///
/// Passed inside [`OutputConfig`], not at the top level of the request. The
/// API default is [`Effort::High`].
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    /// Short, scoped tasks and latency-sensitive workloads
    Low,

    /// Cost-sensitive workloads that trade some intelligence for fewer tokens
    Medium,

    /// The API default; balances token usage and intelligence
    #[default]
    High,

    /// Recommended for coding and agentic work (Opus 4.7 and later)
    XHigh,

    /// Maximum depth, for cases where correctness matters more than cost
    Max,
}

/// Task budget for an agentic loop (beta)
///
/// Unlike `max_tokens`, which is an enforced per-response ceiling the model is
/// unaware of, a task budget is surfaced to the model so it paces itself and
/// finishes gracefully. The minimum `total` is 20,000.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum TaskBudget {
    /// A token-denominated budget
    #[serde(rename = "tokens")]
    Tokens {
        /// Total token budget for the task (minimum 20,000)
        total: usize,

        /// Remaining budget; leave unset to let the server track it
        #[serde(skip_serializing_if = "Option::is_none")]
        remaining: Option<usize>,
    },
}

impl TaskBudget {
    /// Create a token-denominated task budget
    pub fn tokens(total: usize) -> Self {
        TaskBudget::Tokens {
            total,
            remaining: None,
        }
    }

    /// Get the total budget
    pub fn total(&self) -> usize {
        match self {
            TaskBudget::Tokens { total, .. } => *total,
        }
    }
}

/// Response format constraint (structured outputs)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum OutputFormat {
    /// Constrain the response to a JSON Schema
    #[serde(rename = "json_schema")]
    JsonSchema {
        /// The JSON Schema the response must satisfy
        schema: serde_json::Value,
    },
}

impl OutputFormat {
    /// Create a JSON Schema output format
    pub fn json_schema(schema: serde_json::Value) -> Self {
        OutputFormat::JsonSchema { schema }
    }
}

/// Output configuration: effort, response format, and task budget
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct OutputConfig {
    /// Reasoning depth and token spend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<Effort>,

    /// Structured output format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<OutputFormat>,

    /// Task budget for agentic loops (beta)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_budget: Option<TaskBudget>,
}

impl OutputConfig {
    /// Create an empty output configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the effort level
    pub fn with_effort(mut self, effort: Effort) -> Self {
        self.effort = Some(effort);
        self
    }

    /// Set the structured output format
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set the task budget
    pub fn with_task_budget(mut self, budget: TaskBudget) -> Self {
        self.task_budget = Some(budget);
        self
    }

    /// Check whether every field is unset
    pub fn is_empty(&self) -> bool {
        self.effort.is_none() && self.format.is_none() && self.task_budget.is_none()
    }
}

/// A single server-side fallback target
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FallbackEntry {
    /// The model to fall back to
    pub model: Model,

    /// Optional per-hop `max_tokens` override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
}

impl FallbackEntry {
    /// Create a fallback entry for the given model
    pub fn new<T: Into<Model>>(model: T) -> Self {
        FallbackEntry {
            model: model.into(),
            max_tokens: None,
        }
    }

    /// Set a per-hop `max_tokens` override
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}

/// Server-side refusal fallbacks (beta)
///
/// When safety classifiers decline a request, the API re-runs it on a fallback
/// model inside the same call instead of returning the refusal.
///
/// [`Fallbacks::Auto`] is preferred: it routes by refusal category and needs no
/// maintenance when a pinned fallback model is deprecated.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum Fallbacks {
    /// Let the server pick the recommended fallback (`"default"`)
    ///
    /// Serializes as the bare string `"default"`.
    Auto(FallbackMode),

    /// An explicit, ordered list of fallback models
    List(Vec<FallbackEntry>),
}

/// The scalar fallback mode
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum FallbackMode {
    /// Anthropic's recommended fallback, routed by refusal category
    #[default]
    Default,
}

impl Fallbacks {
    /// Use Anthropic's recommended fallback, routed by refusal category
    pub fn auto() -> Self {
        Fallbacks::Auto(FallbackMode::Default)
    }

    /// Use an explicit list of fallback models
    pub fn list(entries: Vec<FallbackEntry>) -> Self {
        Fallbacks::List(entries)
    }

    /// The beta header this configuration requires
    pub fn beta_header(&self) -> &'static str {
        match self {
            Fallbacks::Auto(_) => "server-side-fallback-2026-07-01",
            Fallbacks::List(_) => "server-side-fallback-2026-06-01",
        }
    }
}

impl Default for Body {
    fn default() -> Self {
        Body {
            model: Model::default(),
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
            thinking: None,
            output_config: None,
            fallbacks: None,
        }
    }
}

impl Body {
    /// Create a new request body with model and max_tokens
    ///
    /// Accepts both [`Model`] enum variants and string types.
    pub fn new<T: Into<Model>>(model: T, max_tokens: usize) -> Self {
        Body {
            model: model.into(),
            max_tokens,
            ..Default::default()
        }
    }

    /// The effort level that will be sent, if any
    pub fn effort(&self) -> Option<Effort> {
        self.output_config.as_ref().and_then(|c| c.effort)
    }

    /// Validate the request body
    ///
    /// Beyond the basic required-field checks, this rejects parameter
    /// combinations that the selected model would reject with a `400`, so that
    /// the problem surfaces before the HTTP request is made.
    pub fn validate(&self) -> Result<()> {
        // Check for empty custom model
        if let Model::Other(ref s) = self.model
            && s.is_empty()
        {
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

        self.validate_sampling_params()?;
        self.validate_thinking()?;
        self.validate_output_config()?;
        self.validate_prefill()?;

        Ok(())
    }

    fn validate_sampling_params(&self) -> Result<()> {
        // Validate temperature if set
        if let Some(temp) = self.temperature
            && !(0.0..=1.0).contains(&temp)
        {
            return Err(AnthropicToolError::InvalidParameter(
                "temperature must be between 0.0 and 1.0".to_string(),
            ));
        }

        // Validate top_p if set
        if let Some(top_p) = self.top_p
            && !(0.0..=1.0).contains(&top_p)
        {
            return Err(AnthropicToolError::InvalidParameter(
                "top_p must be between 0.0 and 1.0".to_string(),
            ));
        }

        // Sampling parameters are removed on recent models
        let uses_sampling =
            self.temperature.is_some() || self.top_p.is_some() || self.top_k.is_some();
        if uses_sampling && self.model.is_known() && !self.model.supports_sampling_params() {
            return Err(AnthropicToolError::InvalidParameter(format!(
                "model {} does not accept temperature/top_p/top_k; \
                 remove them and steer behavior via prompting instead",
                self.model
            )));
        }

        Ok(())
    }

    fn validate_thinking(&self) -> Result<()> {
        let Some(ref thinking) = self.thinking else {
            return Ok(());
        };

        match thinking {
            ThinkingConfig::Enabled { budget_tokens } => {
                if self.model.is_known() && !self.model.supports_budget_tokens() {
                    return Err(AnthropicToolError::InvalidParameter(format!(
                        "model {} does not accept thinking.budget_tokens; \
                         use ThinkingConfig::adaptive() with an Effort level instead",
                        self.model
                    )));
                }
                if *budget_tokens < 1024 {
                    return Err(AnthropicToolError::InvalidParameter(
                        "thinking budget_tokens must be at least 1024".to_string(),
                    ));
                }
                if *budget_tokens >= self.max_tokens {
                    return Err(AnthropicToolError::InvalidParameter(format!(
                        "thinking budget_tokens ({}) must be less than max_tokens ({})",
                        budget_tokens, self.max_tokens
                    )));
                }
            }
            ThinkingConfig::Adaptive { .. } => {
                if self.model.is_known() && !self.model.supports_adaptive_thinking() {
                    return Err(AnthropicToolError::InvalidParameter(format!(
                        "model {} does not support adaptive thinking; \
                         use ThinkingConfig::enabled(budget_tokens) instead",
                        self.model
                    )));
                }
            }
            ThinkingConfig::Disabled => {
                if self.model.is_known() && self.model.thinking_always_on() {
                    return Err(AnthropicToolError::InvalidParameter(format!(
                        "model {} always thinks; omit the thinking field \
                         instead of disabling it",
                        self.model
                    )));
                }
                if self.model.is_known() && !self.model.allows_disabled_thinking_at(self.effort()) {
                    return Err(AnthropicToolError::InvalidParameter(format!(
                        "model {} does not allow disabled thinking at effort {:?}; \
                         lower the effort to \"high\" or enable adaptive thinking",
                        self.model,
                        self.effort().unwrap_or_default()
                    )));
                }
            }
        }

        Ok(())
    }

    fn validate_output_config(&self) -> Result<()> {
        let Some(ref config) = self.output_config else {
            return Ok(());
        };

        if let Some(effort) = config.effort
            && self.model.is_known()
            && !self.model.supports_effort_level(effort)
        {
            return Err(AnthropicToolError::InvalidParameter(format!(
                "model {} does not support effort level {:?}",
                self.model, effort
            )));
        }

        if let Some(ref budget) = config.task_budget
            && budget.total() < 20_000
        {
            return Err(AnthropicToolError::InvalidParameter(format!(
                "task_budget total ({}) must be at least 20000",
                budget.total()
            )));
        }

        Ok(())
    }

    fn validate_prefill(&self) -> Result<()> {
        let is_prefill = self
            .messages
            .last()
            .is_some_and(|m| m.role == Role::Assistant);

        if is_prefill && self.model.is_known() && !self.model.supports_prefill() {
            return Err(AnthropicToolError::InvalidParameter(format!(
                "model {} does not support assistant-turn prefilling; \
                 use output_config.format (structured outputs) or a system \
                 prompt instruction instead",
                self.model
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::request::message::Message;

    #[test]
    fn test_body_new_with_model_enum() {
        let body = Body::new(Model::Opus5, 1024);
        assert_eq!(body.model, Model::Opus5);
        assert_eq!(body.max_tokens, 1024);
    }

    #[test]
    fn test_body_new_with_string() {
        let body = Body::new("claude-sonnet-5", 1024);
        assert_eq!(body.model, Model::Sonnet5);
        assert_eq!(body.max_tokens, 1024);
    }

    #[test]
    fn test_body_validate_empty_custom_model() {
        let mut body = Body {
            model: Model::Other(String::new()),
            ..Default::default()
        };
        body.messages.push(Message::user("Test"));
        let result = body.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_body_validate_default_model_ok() {
        let mut body = Body::default();
        body.messages.push(Message::user("Test"));
        assert!(body.validate().is_ok());
    }

    #[test]
    fn test_body_validate_missing_messages() {
        let body = Body::new(Model::Opus5, 1024);
        assert!(body.validate().is_err());
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
        let body = Body::new(Model::Opus5, 1024);
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"model\":\"claude-opus-5\""));
        assert!(json.contains("\"max_tokens\":1024"));
        // Optional fields should not be present
        assert!(!json.contains("\"temperature\""));
        assert!(!json.contains("\"system\""));
        assert!(!json.contains("\"output_config\""));
        assert!(!json.contains("\"fallbacks\""));
    }

    // --- thinking ---------------------------------------------------------

    #[test]
    fn test_thinking_adaptive_serialize() {
        let config = ThinkingConfig::adaptive();
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{"type":"adaptive"}"#);
    }

    #[test]
    fn test_thinking_adaptive_with_display_serialize() {
        let config = ThinkingConfig::adaptive_with_display(ThinkingDisplay::Summarized);
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"adaptive\""));
        assert!(json.contains("\"display\":\"summarized\""));
    }

    #[test]
    fn test_thinking_disabled_serialize() {
        let config = ThinkingConfig::disabled();
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{"type":"disabled"}"#);
    }

    #[test]
    fn test_thinking_config_serialize() {
        let config = ThinkingConfig::enabled(10000);
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"enabled\""));
        assert!(json.contains("\"budget_tokens\":10000"));
    }

    #[test]
    fn test_thinking_config_deserialize() {
        let json = r#"{"type":"enabled","budget_tokens":8000}"#;
        let config: ThinkingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.budget_tokens_opt(), Some(8000));
        // Legacy accessor still works
        assert_eq!(config.budget_tokens(), 8000);

        let json = r#"{"type":"adaptive","display":"summarized"}"#;
        let config: ThinkingConfig = serde_json::from_str(json).unwrap();
        assert!(config.is_adaptive());
        assert_eq!(config.budget_tokens_opt(), None);
        assert_eq!(config.budget_tokens(), 0);
    }

    #[test]
    fn test_body_with_adaptive_thinking() {
        let mut body = Body::new(Model::Opus5, 16000);
        body.thinking = Some(ThinkingConfig::adaptive());
        body.messages.push(Message::user("Test"));

        assert!(body.validate().is_ok());
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains(r#""thinking":{"type":"adaptive"}"#));
    }

    #[test]
    fn test_budget_tokens_rejected_on_new_model() {
        let mut body = Body::new(Model::Opus5, 16000);
        body.thinking = Some(ThinkingConfig::enabled(10000));
        body.messages.push(Message::user("Test"));

        let err = body.validate().unwrap_err();
        assert!(
            err.to_string()
                .contains("does not accept thinking.budget_tokens")
        );
    }

    #[test]
    fn test_adaptive_rejected_on_old_model() {
        let mut body = Body::new(Model::Haiku45, 16000);
        body.thinking = Some(ThinkingConfig::adaptive());
        body.messages.push(Message::user("Test"));

        let err = body.validate().unwrap_err();
        assert!(
            err.to_string()
                .contains("does not support adaptive thinking")
        );
    }

    #[test]
    fn test_disabled_thinking_rejected_on_fable5() {
        let mut body = Body::new(Model::Fable5, 16000);
        body.thinking = Some(ThinkingConfig::disabled());
        body.messages.push(Message::user("Test"));

        let err = body.validate().unwrap_err();
        assert!(err.to_string().contains("always thinks"));
    }

    #[test]
    fn test_disabled_thinking_capped_at_high_on_opus5() {
        let mut body = Body::new(Model::Opus5, 16000);
        body.thinking = Some(ThinkingConfig::disabled());
        body.output_config = Some(OutputConfig::new().with_effort(Effort::XHigh));
        body.messages.push(Message::user("Test"));

        let err = body.validate().unwrap_err();
        assert!(err.to_string().contains("does not allow disabled thinking"));

        // Allowed at "high"
        body.output_config = Some(OutputConfig::new().with_effort(Effort::High));
        assert!(body.validate().is_ok());
    }

    #[test]
    fn test_validate_thinking_budget_too_small() {
        let mut body = Body::new(Model::Haiku45, 16000);
        body.thinking = Some(ThinkingConfig::enabled(500));
        body.messages.push(Message::user("Test"));

        let err = body.validate().unwrap_err();
        assert!(err.to_string().contains("at least 1024"));
    }

    #[test]
    fn test_validate_thinking_budget_exceeds_max_tokens() {
        let mut body = Body::new(Model::Haiku45, 8000);
        body.thinking = Some(ThinkingConfig::enabled(10000));
        body.messages.push(Message::user("Test"));

        let err = body.validate().unwrap_err();
        assert!(err.to_string().contains("must be less than max_tokens"));
    }

    #[test]
    fn test_validate_thinking_budget_valid() {
        let mut body = Body::new(Model::Haiku45, 16000);
        body.thinking = Some(ThinkingConfig::enabled(10000));
        body.messages.push(Message::user("Test"));

        assert!(body.validate().is_ok());
    }

    // --- sampling params --------------------------------------------------

    #[test]
    fn test_sampling_params_rejected_on_new_model() {
        let mut body = Body::new(Model::Opus5, 1024);
        body.temperature = Some(0.7);
        body.messages.push(Message::user("Test"));

        let err = body.validate().unwrap_err();
        assert!(err.to_string().contains("does not accept temperature"));
    }

    #[test]
    fn test_sampling_params_allowed_on_older_model() {
        let mut body = Body::new(Model::Sonnet46, 1024);
        body.temperature = Some(0.7);
        body.messages.push(Message::user("Test"));

        assert!(body.validate().is_ok());
    }

    #[test]
    fn test_temperature_range_still_validated() {
        let mut body = Body::new(Model::Sonnet46, 1024);
        body.temperature = Some(1.5);
        body.messages.push(Message::user("Test"));

        let err = body.validate().unwrap_err();
        assert!(err.to_string().contains("between 0.0 and 1.0"));
    }

    // --- output config ----------------------------------------------------

    #[test]
    fn test_output_config_serialize() {
        let config =
            OutputConfig::new()
                .with_effort(Effort::XHigh)
                .with_format(OutputFormat::json_schema(serde_json::json!({
                    "type": "object",
                    "properties": {"name": {"type": "string"}},
                    "required": ["name"],
                    "additionalProperties": false
                })));

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"effort\":\"xhigh\""));
        assert!(json.contains("\"type\":\"json_schema\""));
        assert!(json.contains("\"schema\""));
        assert!(!json.contains("task_budget"));
    }

    #[test]
    fn test_effort_serialize_all_levels() {
        assert_eq!(serde_json::to_string(&Effort::Low).unwrap(), "\"low\"");
        assert_eq!(
            serde_json::to_string(&Effort::Medium).unwrap(),
            "\"medium\""
        );
        assert_eq!(serde_json::to_string(&Effort::High).unwrap(), "\"high\"");
        assert_eq!(serde_json::to_string(&Effort::XHigh).unwrap(), "\"xhigh\"");
        assert_eq!(serde_json::to_string(&Effort::Max).unwrap(), "\"max\"");
    }

    #[test]
    fn test_unsupported_effort_level_rejected() {
        let mut body = Body::new(Model::Opus46, 1024);
        body.output_config = Some(OutputConfig::new().with_effort(Effort::XHigh));
        body.messages.push(Message::user("Test"));

        let err = body.validate().unwrap_err();
        assert!(err.to_string().contains("does not support effort level"));
    }

    #[test]
    fn test_task_budget_minimum_enforced() {
        let mut body = Body::new(Model::Opus5, 128_000);
        body.output_config = Some(OutputConfig::new().with_task_budget(TaskBudget::tokens(1000)));
        body.messages.push(Message::user("Test"));

        let err = body.validate().unwrap_err();
        assert!(err.to_string().contains("at least 20000"));
    }

    #[test]
    fn test_task_budget_serialize() {
        let budget = TaskBudget::tokens(64_000);
        let json = serde_json::to_string(&budget).unwrap();
        assert!(json.contains("\"type\":\"tokens\""));
        assert!(json.contains("\"total\":64000"));
        assert!(!json.contains("remaining"));
    }

    // --- prefill ----------------------------------------------------------

    #[test]
    fn test_prefill_rejected_on_new_model() {
        let mut body = Body::new(Model::Opus5, 1024);
        body.messages.push(Message::user("Extract the name."));
        body.messages.push(Message::assistant("{\"name\": \""));

        let err = body.validate().unwrap_err();
        assert!(
            err.to_string()
                .contains("does not support assistant-turn prefilling")
        );
    }

    #[test]
    fn test_prefill_allowed_on_older_model() {
        let mut body = Body::new(Model::Haiku45, 1024);
        body.messages.push(Message::user("Extract the name."));
        body.messages.push(Message::assistant("{\"name\": \""));

        assert!(body.validate().is_ok());
    }

    #[test]
    fn test_multiturn_assistant_message_is_not_prefill() {
        // An assistant turn followed by a user turn is ordinary history
        let mut body = Body::new(Model::Opus5, 1024);
        body.messages.push(Message::user("What is 2+2?"));
        body.messages.push(Message::assistant("4."));
        body.messages.push(Message::user("And 3+3?"));

        assert!(body.validate().is_ok());
    }

    // --- fallbacks --------------------------------------------------------

    #[test]
    fn test_fallbacks_auto_serialize() {
        let fallbacks = Fallbacks::auto();
        let json = serde_json::to_string(&fallbacks).unwrap();
        assert_eq!(json, "\"default\"");
        assert_eq!(fallbacks.beta_header(), "server-side-fallback-2026-07-01");
    }

    #[test]
    fn test_fallbacks_list_serialize() {
        let fallbacks = Fallbacks::list(vec![FallbackEntry::new(Model::Opus48)]);
        let json = serde_json::to_string(&fallbacks).unwrap();
        assert_eq!(json, r#"[{"model":"claude-opus-4-8"}]"#);
        assert_eq!(fallbacks.beta_header(), "server-side-fallback-2026-06-01");
    }

    #[test]
    fn test_fallback_entry_with_max_tokens() {
        let entry = FallbackEntry::new(Model::Opus48).with_max_tokens(4096);
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"max_tokens\":4096"));
    }
}
