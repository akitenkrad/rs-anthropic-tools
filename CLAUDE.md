# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust library (`anthropic-tools`) for interacting with the Anthropic API. Provides a builder-pattern API client for Claude models with support for tool calling, vision/multimodal, prompt caching, adaptive thinking and effort control, structured outputs, the Files API, token counting, and SSE streaming.

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

- **`files/`** - Files API implementation
  - `mod.rs` - `Files` client (upload/list/metadata/download/delete), `FileMetadata`, `FILES_API_BETA`

- **`messages/`** - Messages API implementation
  - `request/mod.rs` - `Messages` client with builder pattern, beta headers, `count_tokens()`
  - `request/body.rs` - Request body, `ThinkingConfig`, `OutputConfig`, `Effort`, `TaskBudget`, `Fallbacks`, and validation
  - `request/content.rs` - `ContentBlock` enum (text, image, tool_use, tool_result, thinking, redacted_thinking, document, server_tool_use, fallback, unknown)
  - `request/message.rs` - `Message` and `SystemPrompt` types
  - `request/model.rs` - `Model` enum and per-model capability queries
  - `response.rs` - `Response`, `StopReason`, `StopDetails`, `TokenCount`, `Container`
  - `streaming.rs` - SSE event types and `StreamAccumulator`

### Key Patterns

1. **Builder Pattern**: The `Messages` client uses method chaining:
   ```rust
   let mut client = Messages::new();
   // Using Model enum (recommended)
   client.model(Model::Opus5).max_tokens(1024).user("Hello");
   // Or using string (backward compatible)
   client.model("claude-sonnet-5").max_tokens(1024).user("Hello");
   ```

   With adaptive thinking and effort:
   ```rust
   client
       .model(Model::Opus5)
       .max_tokens(64000)
       .thinking_summarized()
       .effort(Effort::XHigh)
       .user("Complex problem");
   ```

2. **Model Enum**: Type-safe model selection with `Model` enum:
   - `Model::Fable5`, `Model::Mythos5`, `Model::Opus5`, `Model::Sonnet5` - Claude 5 family
   - `Model::Opus48`, `Model::Opus47`, `Model::Opus46`, `Model::Sonnet46`, `Model::Haiku45` - Claude 4.x family
   - `Model::Opus45`, `Model::Sonnet45` - legacy, still served
   - `Model::Opus4`, `Model::Sonnet4`, `Model::Opus3`, `Model::Sonnet3`, `Model::Haiku3` - **retired**, kept only for backward compatibility (return 404)
   - `Model::Other(String)` - Custom/future models
   - Default: `Model::Opus5`

3. **Model-aware Validation**: `Model` exposes the per-model API constraints
   (`supports_adaptive_thinking()`, `supports_budget_tokens()`,
   `supports_sampling_params()`, `supports_effort_level()`, `supports_prefill()`,
   `is_retired()`, …), and `Body::validate()` enforces them so that requests
   which would return a `400` fail locally instead. When adding a model variant,
   update every capability method in `model.rs` — they are exhaustive matches or
   explicit variant lists, not defaults.

4. **Tagged Union Serialization**: `ContentBlock` uses `#[serde(tag = "type")]` for Anthropic API compliance, plus `#[serde(other)]` on `Unknown` so unrecognized block types do not fail deserialization. `StopReason` and `Model` hand-roll `Serialize`/`Deserialize` with an `Other(String)` fallback for the same reason.

5. **Non-exhaustive Enums**: `Model`, `ContentBlock`, `ThinkingConfig`, `StopReason`, `TaskBudget`, `OutputFormat`, and `Fallbacks` are `#[non_exhaustive]` so new API values can be added without a breaking release.

6. **Prelude Module**: Import `anthropic_tools::prelude::*` for all commonly used types.

## Environment

- Requires `ANTHROPIC_API_KEY` for API calls
- Supports both environment variable and `.env` file:
  ```bash
  # Option 1: Environment variable
  export ANTHROPIC_API_KEY="sk-ant-..."

  # Option 2: .env file in project root
  echo 'ANTHROPIC_API_KEY=sk-ant-...' > .env
  ```
- Priority: Environment variable > `.env` file > `with_api_key()`
