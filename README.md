# anthropic-tools

A Rust library for interacting with the Anthropic API.

<img src="./LOGO.png" alt="LOGO" width="150" height="150">

## Features

- **Messages API** - Builder pattern for creating and sending messages
- **Tool/Function Calling** - Define and use tools with JSON Schema
- **Vision/Multimodal** - Support for images and documents
- **Prompt Caching** - Cache control for system prompts and tools
- **Streaming** - Server-Sent Events (SSE) streaming support

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
anthropic-tools = { git = "https://github.com/akitenkrad/rs-anthropic-tools" }
```

## Quick Start

### Basic Usage

```rust
use anthropic_tools::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(1024)
        .system("You are a helpful assistant.")
        .user("Hello, how are you?");

    let response = client.post().await?;
    println!("{}", response.get_text());
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
        .model("claude-sonnet-4-20250514")
        .max_tokens(1024)
        .tool(tool)
        .user("What's the weather in Tokyo?");

    let response = client.post().await?;

    // Check if tool was used
    if let Some(tool_use) = response.get_tool_use() {
        println!("Tool: {}, Input: {}", tool_use.name, tool_use.input);
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
        .model("claude-sonnet-4-20250514")
        .max_tokens(1024)
        .user_with_image("Describe this image", "/path/to/image.png")?;

    let response = client.post().await?;
    println!("{}", response.get_text());
    Ok(())
}
```

### Streaming

```rust
use anthropic_tools::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(1024)
        .stream(true)
        .user("Tell me a story");

    let events = client.post_stream().await?;
    let mut accumulator = StreamAccumulator::new();

    for event in events {
        accumulator.process_event(&event);
        if let StreamEvent::ContentBlockDelta { delta, .. } = event {
            if let Delta::TextDelta { text } = delta {
                print!("{}", text);
            }
        }
    }

    let response = accumulator.finalize();
    println!("\n\nTotal tokens: {:?}", response.usage);
    Ok(())
}
```

## Environment Variables

- `ANTHROPIC_API_KEY` - Your Anthropic API key (required)

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
    │   ├── body.rs     - Request body
    │   ├── content.rs  - Content blocks (text, image, tool_use, etc.)
    │   └── message.rs  - Message and SystemPrompt types
    ├── response.rs     - API response types
    └── streaming.rs    - SSE streaming types
```

## License

MIT
