//! Common types and utilities for the Anthropic API client.
//!
//! This module contains shared types used across the library:
//!
//! - [`errors`] - Error types and result alias
//! - [`tool`] - Tool definitions for function calling
//! - [`usage`] - Token usage information
//!
//! # Example
//!
//! ```rust
//! use anthropic_tools::common::{Tool, Usage, AnthropicToolError};
//!
//! // Create a tool definition
//! let mut tool = Tool::new("search");
//! tool.description("Search the web")
//!     .add_string_property("query", Some("Search query"), true);
//!
//! // Usage information from API response
//! let usage = Usage::new(100, 50);
//! assert_eq!(usage.total_tokens(), 150);
//! ```

pub mod errors;
pub mod tool;
pub mod usage;

pub use errors::{AnthropicToolError, ErrorDetail, ErrorResponse, Result};
pub use tool::{CacheControl, JsonSchema, PropertyDef, Tool};
pub use usage::Usage;
