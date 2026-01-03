//! Messages API client for Claude.
//!
//! This module provides the main interface for interacting with the Anthropic Messages API:
//!
//! - [`request`] - Request types and the [`Messages`](request::Messages) client
//! - [`response`] - Response types including [`Response`](response::Response)
//! - [`streaming`] - SSE streaming support
//!
//! # Basic Usage
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
//!         .user("Hello!");
//!
//!     let response = client.post().await?;
//!     println!("{}", response.get_text());
//!     Ok(())
//! }
//! ```
//!
//! # With Tools
//!
//! ```rust,no_run
//! use anthropic_tools::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut tool = Tool::new("search");
//!     tool.description("Search the web")
//!         .add_string_property("query", Some("Search query"), true);
//!
//!     let mut client = Messages::new();
//!     client
//!         .model("claude-sonnet-4-20250514")
//!         .max_tokens(1024)
//!         .tools(vec![tool.to_value()])
//!         .user("Search for Rust programming");
//!
//!     let response = client.post().await?;
//!     if response.has_tool_use() {
//!         // Handle tool use
//!     }
//!     Ok(())
//! }
//! ```

pub mod request;
pub mod response;
pub mod streaming;
