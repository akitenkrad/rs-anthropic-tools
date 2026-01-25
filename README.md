# anthropic-tools

A Rust library for interacting with the Anthropic API.

<img src="./LOGO.png" alt="LOGO" width="150" height="150">

[![Crates.io](https://img.shields.io/crates/v/anthropic-tools.svg)](https://crates.io/crates/anthropic-tools)
[![Documentation](https://docs.rs/anthropic-tools/badge.svg)](https://docs.rs/anthropic-tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Messages API** - Builder pattern for creating and sending messages
- **Type-safe Model Selection** - `Model` enum with all Claude models
- **Tool/Function Calling** - Define and use tools with JSON Schema
- **Vision/Multimodal** - Support for images and documents
- **Extended Thinking** - Enable Claude's internal reasoning for complex tasks
- **Prompt Caching** - Cache control for system prompts and tools
- **Streaming** - Server-Sent Events (SSE) streaming support
- **Environment Configuration** - Load API key from env var or `.env` file

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
anthropic-tools = "1.0"
```

## Quick Start

### Configuration

Set your API key via environment variable or `.env` file:

```bash
# Environment variable
export ANTHROPIC_API_KEY="sk-ant-..."

# Or create .env file in project root
echo 'ANTHROPIC_API_KEY=sk-ant-...' > .env
```

### Basic Usage

```rust
use anthropic_tools::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Messages::new();
    client
        .model(Model::Sonnet4)  // Type-safe model selection
        .max_tokens(1024)
        .system("You are a helpful assistant.")
        .user("Hello, how are you?");

    let response = client.post().await?;
    println!("{}", response.get_text());
    Ok(())
}
```

### Extended Thinking

```rust
use anthropic_tools::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Messages::new();
    client
        .model(Model::Sonnet4)
        .max_tokens(16000)
        .thinking(10000)  // Enable extended thinking with 10k token budget
        .user("Solve this complex problem step by step...");

    let response = client.post().await?;

    // Access thinking content if available
    if response.has_thinking() {
        println!("Thinking: {}", response.get_thinking().unwrap_or_default());
    }
    println!("Response: {}", response.get_text());
    Ok(())
}
```

### Tool Calling

```rust
use anthropic_tools::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Define a tool
    let mut tool = Tool::new("get_weather");
    tool.description("Get the current weather for a location")
        .add_string_property("location", Some("City name"), true);

    // Create client with tool
    let mut client = Messages::new();
    client
        .model(Model::Sonnet4)
        .max_tokens(1024)
        .tools(vec![tool.to_value()])
        .user("What's the weather in Tokyo?");

    let response = client.post().await?;

    // Check if tool was used
    if response.has_tool_use() {
        for tool_use in response.get_tool_uses() {
            if let ContentBlock::ToolUse { name, input, .. } = tool_use {
                println!("Tool: {}, Input: {}", name, input);
            }
        }
    }
    Ok(())
}
```

### Vision (Image Input)

```rust
use anthropic_tools::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Messages::new();
    client
        .model(Model::Sonnet4)
        .max_tokens(1024)
        .user_with_image_url(
            "Describe this image",
            "https://example.com/image.png",
        );

    let response = client.post().await?;
    println!("{}", response.get_text());
    Ok(())
}
```

## Available Models

```rust
use anthropic_tools::prelude::Model;

// Claude 4.5 Family
Model::Opus45   // claude-opus-4-5-20251101

// Claude 4 Family
Model::Opus4    // claude-opus-4-20250514
Model::Sonnet4  // claude-sonnet-4-20250514 (default)

// Claude 3.5 Family
Model::Haiku35  // claude-3-5-haiku-20241022

// Claude 3 Family
Model::Opus3    // claude-3-opus-20240229
Model::Sonnet3  // claude-3-sonnet-20240229
Model::Haiku3   // claude-3-haiku-20240307

// Custom/Future models
Model::Other("custom-model".to_string())
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ANTHROPIC_API_KEY` | Your Anthropic API key (required) |

Supports loading from `.env` file automatically.

## Module Structure

```
anthropic-tools
├── common/
│   ├── errors.rs   - Error types (AnthropicToolError)
│   ├── tool.rs     - Tool definitions (Tool, JsonSchema)
│   └── usage.rs    - Token usage tracking
└── messages/
    ├── request/
    │   ├── mod.rs      - Messages client
    │   ├── body.rs     - Request body, ThinkingConfig
    │   ├── content.rs  - Content blocks (text, image, tool_use, etc.)
    │   ├── message.rs  - Message and SystemPrompt types
    │   ├── model.rs    - Model enum
    │   ├── role.rs     - Role enum (User, Assistant)
    │   └── mcp.rs      - MCP server configuration
    ├── response.rs     - API response types
    └── streaming.rs    - SSE streaming types
```

## License

MIT
