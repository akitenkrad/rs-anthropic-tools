# anthropic-tools

A Rust library for interacting with the Anthropic API.

<img src="./LOGO.png" alt="LOGO" width="150" height="150">

[![Crates.io](https://img.shields.io/crates/v/anthropic-tools.svg)](https://crates.io/crates/anthropic-tools)
[![Documentation](https://docs.rs/anthropic-tools/badge.svg)](https://docs.rs/anthropic-tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Messages API** - Builder pattern for creating and sending messages
- **Type-safe Model Selection** - `Model` enum covering the Claude 5 and 4.x families
- **Adaptive Thinking & Effort** - Current-generation reasoning control
- **Structured Outputs** - Constrain responses to a JSON Schema
- **Tool/Function Calling** - Define and use tools with JSON Schema
- **Vision/Multimodal** - Support for images and documents
- **Prompt Caching** - Cache control for system prompts and tools
- **Files API** - Upload once, reference by `file_id` across requests
- **Token Counting** - Estimate input tokens before sending
- **Refusal Fallbacks & Task Budgets** (beta)
- **Streaming** - Server-Sent Events (SSE) streaming support
- **Model-aware Validation** - Invalid parameter/model combinations fail before the HTTP request
- **Environment Configuration** - Load API key from env var or `.env` file

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
anthropic-tools = "1.1"
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
        .model(Model::Opus5)  // Type-safe model selection
        .max_tokens(1024)
        .system("You are a helpful assistant.")
        .user("Hello, how are you?");

    let response = client.post().await?;

    // A refusal is a successful HTTP 200 — check before reading content
    if response.was_refused() {
        eprintln!("refused: {:?}", response.refusal_category());
        return Ok(());
    }

    println!("{}", response.get_text());
    Ok(())
}
```

### Adaptive Thinking

Current models decide when and how much to reason. Depth is controlled with
`Effort`, not a fixed token budget.

```rust
use anthropic_tools::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Messages::new();
    client
        .model(Model::Opus5)
        .max_tokens(64000)
        .thinking_summarized()   // adaptive thinking + readable summary
        .effort(Effort::XHigh)   // low | medium | high | xhigh | max
        .user("Solve this complex problem step by step...");

    let response = client.post().await?;

    if response.has_thinking() {
        println!("Thinking: {}", response.get_thinking().unwrap_or_default());
    }
    println!("Response: {}", response.get_text());
    Ok(())
}
```

Without `thinking_summarized()`, thinking blocks arrive with empty text — the
raw chain of thought is never returned by the API.

### Structured Outputs

```rust
use anthropic_tools::prelude::*;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Contact {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Messages::new();
    client
        .model(Model::Opus5)
        .max_tokens(1024)
        .json_schema(json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "email": {"type": "string"}
            },
            "required": ["name", "email"],
            "additionalProperties": false
        }))
        .user("Extract the contact: Jane Doe can be reached at jane@example.com.");

    let response = client.post().await?;
    let contact: Contact = response.json()?;

    println!("{} <{}>", contact.name, contact.email);
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
        .model(Model::Opus5)
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
        .model(Model::Opus5)
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

### Files API

Upload a document once and reference it across many requests.

```rust
use anthropic_tools::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let files = Files::new();
    let uploaded = files.upload_path("report.pdf").await?;

    let mut client = Messages::new();
    client
        .model(Model::Opus5)
        .max_tokens(4096)
        .beta(FILES_API_BETA)
        .add_message(Message::new(
            Role::User,
            vec![
                ContentBlock::document_from_file_id(&uploaded.id),
                ContentBlock::text("Summarize the key findings."),
            ],
        ));

    let response = client.post().await?;
    println!("{}", response.get_text());

    files.delete(&uploaded.id).await?;
    Ok(())
}
```

### Token Counting

```rust
use anthropic_tools::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Messages::new();
    client
        .model(Model::Opus5)
        .max_tokens(1024)
        .user("Hello!");

    let count = client.count_tokens().await?;
    println!("{} input tokens", count.input_tokens);
    Ok(())
}
```

Token counts are model-specific — count against the model you will actually call.

### Refusal Fallbacks (beta)

```rust
use anthropic_tools::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Messages::new();
    client
        .model(Model::Opus5)
        .max_tokens(1024)
        .fallbacks(Fallbacks::auto())  // beta header added automatically
        .user("Hello");

    let response = client.post().await?;

    for (from, to) in response.fallback_switches() {
        println!("{} declined; {} continued", from, to);
    }
    Ok(())
}
```

## Available Models

```rust
use anthropic_tools::prelude::Model;

// Claude 5 Family
Model::Fable5   // claude-fable-5   (thinking always on)
Model::Mythos5  // claude-mythos-5  (Project Glasswing only)
Model::Opus5    // claude-opus-5    (default)
Model::Sonnet5  // claude-sonnet-5

// Claude 4.x Family
Model::Opus48   // claude-opus-4-8
Model::Opus47   // claude-opus-4-7
Model::Opus46   // claude-opus-4-6
Model::Sonnet46 // claude-sonnet-4-6
Model::Haiku45  // claude-haiku-4-5

// Legacy (still served)
Model::Opus45   // claude-opus-4-5
Model::Sonnet45 // claude-sonnet-4-5

// Retired — kept for backward compatibility, these return 404
Model::Opus4    // claude-opus-4-20250514      (retired 2026-06-15)
Model::Sonnet4  // claude-sonnet-4-20250514    (retired 2026-06-15)
Model::Haiku3   // claude-3-haiku-20240307     (retired 2026-04-20)
Model::Opus3    // claude-3-opus-20240229      (retired 2026-01-05)
Model::Sonnet3  // claude-3-sonnet-20240229    (retired 2025-07-21)

// Custom/Future models
Model::Other("custom-model".to_string())
```

Use `Model::is_retired()` to detect retired variants and `Model::replacement()`
to get the recommended successor.

### Model Capabilities

The `Model` enum exposes the per-model API constraints, and `Body::validate()`
enforces them before the request is sent:

| Method | Meaning |
|--------|---------|
| `supports_adaptive_thinking()` | Accepts `thinking: {"type": "adaptive"}` |
| `supports_budget_tokens()` | Accepts the legacy fixed thinking budget |
| `supports_disabled_thinking()` | Accepts `thinking: {"type": "disabled"}` |
| `allows_disabled_thinking_at(effort)` | Disabled thinking is allowed at that effort |
| `thinking_always_on()` | Thinking cannot be turned off |
| `supports_sampling_params()` | Accepts `temperature` / `top_p` / `top_k` |
| `supports_effort()` / `supports_effort_level(e)` | Effort support, per level |
| `supports_prefill()` | Accepts an assistant-turn prefill |
| `context_window()` / `max_output_tokens()` | Token limits |
| `is_retired()` / `replacement()` | Retirement status and successor |

Requests that would return a `400` — `budget_tokens` on Claude Opus 5, a
`temperature` on Claude Sonnet 5, an assistant prefill on Opus 4.6+ — fail
locally with an `InvalidParameter` error instead of a round trip.

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
├── files/
│   └── mod.rs      - Files API client (upload, list, download, delete)
└── messages/
    ├── request/
    │   ├── mod.rs      - Messages client, beta headers, count_tokens
    │   ├── body.rs     - Request body, ThinkingConfig, OutputConfig, Effort, Fallbacks
    │   ├── content.rs  - Content blocks (text, image, tool_use, thinking, etc.)
    │   ├── message.rs  - Message and SystemPrompt types
    │   ├── model.rs    - Model enum and capability queries
    │   ├── role.rs     - Role enum (User, Assistant)
    │   └── mcp.rs      - MCP server configuration
    ├── response.rs     - API response types, StopReason, StopDetails
    └── streaming.rs    - SSE streaming types
```

## Upgrading

See [CHANGELOG.md](./CHANGELOG.md). The 1.0 → 1.1 upgrade is source-compatible
for typical usage, but note that **the default model changed from
`Model::Sonnet4` (retired) to `Model::Opus5`**.

## License

MIT
