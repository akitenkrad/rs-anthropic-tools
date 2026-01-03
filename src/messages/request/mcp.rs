//! MCP (Model Context Protocol) server configuration (beta).
//!
//! This module provides types for configuring MCP servers that Claude
//! can use during conversations:
//!
//! - [`McpServer`] - MCP server connection configuration
//! - [`ToolConfiguration`] - Tool access configuration for servers
//!
//! # Note
//!
//! MCP support is currently in beta. See the Anthropic documentation
//! for the latest information on MCP capabilities and configuration.
//!
//! # Example
//!
//! ```rust
//! use anthropic_tools::messages::request::mcp::{McpServer, ToolConfiguration};
//!
//! let server = McpServer {
//!     name: "my-server".to_string(),
//!     type_name: "url".to_string(),
//!     url: "https://mcp.example.com".to_string(),
//!     authorization_token: Some("token".to_string()),
//!     tool_configuration: Some(ToolConfiguration {
//!         allowed_tools: vec!["tool1".to_string(), "tool2".to_string()],
//!         enabled: true,
//!     }),
//! };
//! ```

use serde::{Deserialize, Serialize};

/// Tool configuration for MCP servers
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolConfiguration {
    pub allowed_tools: Vec<String>,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpServer {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub url: String,
    pub authorization_token: Option<String>,
    pub tool_configuration: Option<ToolConfiguration>,
}
