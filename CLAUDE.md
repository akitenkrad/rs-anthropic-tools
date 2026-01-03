# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust library (`anthropic-tools`) for interacting with the Anthropic API. Provides a builder-pattern API client for Claude models with support for tool calling, vision/multimodal, prompt caching, and SSE streaming.

## Build & Test Commands

```bash
# Build
cargo build
cargo build --release

# Run unit tests with nextest
cargo nextest run
cargo make nextest

# Run API integration tests (requires ANTHROPIC_API_KEY)
cargo make test-api

# Run all tests (unit + API integration + doc tests)
cargo make test-all

# Run a single test
cargo nextest run <test_name>

# Run doc tests only
cargo test --doc

# Format and lint
cargo make format-all

# Generate documentation
cargo doc --open
```

## Architecture

### Module Structure

- **`common/`** - Shared types
  - `errors.rs` - `AnthropicToolError` enum and `Result` type alias
  - `tool.rs` - `Tool`, `JsonSchema`, `PropertyDef` for function calling
  - `usage.rs` - Token usage tracking

- **`messages/`** - Messages API implementation
  - `request/mod.rs` - `Messages` client with builder pattern
  - `request/body.rs` - Request body structure and validation
  - `request/content.rs` - `ContentBlock` enum (text, image, tool_use, tool_result, thinking, document)
  - `request/message.rs` - `Message` and `SystemPrompt` types
  - `response.rs` - `Response` struct with helper methods
  - `streaming.rs` - SSE event types and `StreamAccumulator`

### Key Patterns

1. **Builder Pattern**: The `Messages` client uses method chaining:
   ```rust
   let mut client = Messages::new();
   client.model("claude-sonnet-4-20250514").max_tokens(1024).user("Hello");
   ```

2. **Tagged Union Serialization**: `ContentBlock` uses `#[serde(tag = "type")]` for Anthropic API compliance.

3. **Prelude Module**: Import `anthropic_tools::prelude::*` for all commonly used types.

## Environment

- Requires `ANTHROPIC_API_KEY` environment variable for API calls
- Uses `.env` file (loaded by cargo-make tasks)
