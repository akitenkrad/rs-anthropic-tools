# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-08-15

This release brings the library up to the current Anthropic API. The previous
release targeted models and request shapes that have since been retired or
removed — most importantly, **the old default model (`claude-sonnet-4-20250514`)
was retired on 2026-06-15**, so an unconfigured client returned `404`.

### ⚠️ Behavioral changes

- **The default model changed from `Model::Sonnet4` to `Model::Opus5`.**
  `Model::Sonnet4` resolves to `claude-sonnet-4-20250514`, which is retired. Code
  that relied on the default now targets `claude-opus-5` ($5/$25 per MTok). Set
  the model explicitly if you want a different tier — `Model::Sonnet5` is the
  closest match to the previous default's tier.
- **`Body::validate()` now rejects parameter/model combinations the API would
  reject with a `400`.** These previously reached the API and failed there; they
  now fail locally with `AnthropicToolError::InvalidParameter`:
  - `temperature` / `top_p` / `top_k` on Opus 4.7+, Claude Opus 5, Claude Sonnet 5,
    Claude Fable 5, Claude Mythos 5
  - `thinking(budget_tokens)` on the same models
  - `thinking: adaptive` on models that predate it (e.g. Haiku 4.5)
  - `thinking: disabled` on Claude Fable 5 / Mythos 5, or on Claude Opus 5 at
    effort `xhigh` / `max`
  - An unsupported effort level for the selected model
  - A trailing assistant message (prefill) on Opus 4.6 and later
  - A task budget below the 20,000-token minimum
- **`Model`, `ContentBlock`, `ThinkingConfig`, `StopReason`, `TaskBudget`,
  `OutputFormat`, and `Fallbacks` are now `#[non_exhaustive]`.** Exhaustive
  `match` expressions over these types need a wildcard arm. New variants were
  added to `Model`, `ContentBlock`, and `StopReason` in this release, so such
  matches required updating regardless.
- **`Response` gained the fields `stop_details` and `container`.** Struct-literal
  construction of `Response` (mostly test mocks) needs the new fields.
- `Messages::post()` no longer panics on an API key containing invalid header
  bytes; it returns `AnthropicToolError::InvalidParameter` instead.
- **`Model::Opus45.as_str()` now returns the alias `"claude-opus-4-5"` instead of
  the dated `"claude-opus-4-5-20251101"`**, matching every other variant. Both
  identifiers resolve to the same model at the API, and `Model::from()` still
  accepts the dated form — but code that string-compares against the dated
  identifier needs updating.

### Added

#### Models

- Claude 5 family: `Model::Fable5`, `Model::Mythos5`, `Model::Opus5`, `Model::Sonnet5`
- Claude 4.x family: `Model::Opus48`, `Model::Opus47`, `Model::Opus46`,
  `Model::Sonnet46`, `Model::Haiku45`, `Model::Sonnet45`
- Capability queries on `Model`: `supports_adaptive_thinking()`,
  `supports_budget_tokens()`, `supports_disabled_thinking()`,
  `allows_disabled_thinking_at()`, `thinking_always_on()`,
  `supports_sampling_params()`, `supports_effort()`, `supports_effort_level()`,
  `supports_prefill()`, `context_window()`, `max_output_tokens()`,
  `is_retired()`, `replacement()`
- `Model::from()` now also resolves dated full identifiers
  (e.g. `claude-haiku-4-5-20251001` → `Model::Haiku45`)

#### Thinking and effort

- `ThinkingConfig::Adaptive` and `ThinkingConfig::Disabled` variants, with
  `adaptive()`, `adaptive_with_display()`, `disabled()`, `is_adaptive()`,
  `is_disabled()`, and `budget_tokens_opt()`
- `ThinkingDisplay` (`Omitted` / `Summarized`)
- `Effort` (`Low` / `Medium` / `High` / `XHigh` / `Max`)
- Builder methods `thinking_adaptive()`, `thinking_summarized()`,
  `thinking_disabled()`, `thinking_config()`, `effort()`

#### Output configuration

- `OutputConfig` with `effort`, `format`, and `task_budget`
- `OutputFormat::json_schema()` for structured outputs, plus the
  `Messages::json_schema()` / `output_format()` builder methods
- `Response::json::<T>()` to deserialize a schema-constrained response
- `TaskBudget` (beta) and `Messages::task_budget()`, which adds the required
  beta header automatically

#### Responses

- `StopReason::PauseTurn` and `StopReason::ModelContextWindowExceeded` — a
  `pause_turn` response previously failed to deserialize
- `StopReason::Other(String)` — unrecognized stop reasons no longer fail
  deserialization; `StopReason::wire_str()` returns the API's value
- `StopDetails` and `Response::stop_details`, plus `was_refused()`,
  `refusal_category()`, `refusal_explanation()`
- `Response::is_paused()`, `exceeded_context_window()`, `fallback_switches()`
- `Container` and `Response::container` for code-execution container reuse

#### Content blocks

- `ContentBlock::RedactedThinking`, `ServerToolUse`, and `Fallback`
- `ContentBlock::Unknown` — unrecognized block types deserialize to this variant
  instead of failing the whole response
- `ContentBlock::document_from_file_id()` and `image_from_file_id()`, backed by
  a new `file_id` field on `DocumentSource` and `ImageSource`

#### Files API

- New `files` module: `Files` client with `upload_path()`, `upload_bytes()`,
  `list()`, `list_with()`, `metadata()`, `download()`, `download_to_path()`,
  and `delete()`
- `FileMetadata`, `FileList`, `FileDeleted`, `ListOptions`, `FILES_API_BETA`

#### Client

- `Messages::count_tokens()` — estimate input tokens without running the request
- `Messages::beta()` / `betas()` — `anthropic-beta` feature flags, deduplicated
  and joined into a single header
- `Messages::fallbacks()` with `Fallbacks::auto()` / `Fallbacks::list()` and
  `FallbackEntry`; the matching beta header is added automatically
- `Messages::mcp_servers()` — the `mcp_servers` body field previously had no
  builder method

### Changed

- Dependencies updated: `base64` 0.22 → 0.23, `strum` 0.27 → 0.28,
  `reqwest` 0.13.1 → 0.13.4 (with the `multipart` and `query` features),
  `image` 0.25.9 → 0.25.10, `serde` 1.0.229, `serde_json` 1.0.151,
  `thiserror` 2.0.20, `tokio` 1.53.1, `test-log` 0.2.21
- `image` now builds with `default-features = false` and every default format
  except `avif`. The `avif` feature is encode-only — AVIF *decoding* requires
  `avif-native`, which was never enabled — and this crate only ever encodes
  PNG/JPEG/GIF/WebP, so no reachable functionality is lost. This drops the
  `ravif`/`rav1e` dependency tree, removes the yanked transitive `core2 0.4.0`,
  and cuts the non-dev dependency graph from 196 to 163 crates. A round-trip
  test in `content.rs` guards the reduced codec set.
- `ThinkingConfig::budget_tokens()` returns `0` for non-budget configurations;
  prefer `budget_tokens_opt()`, which distinguishes "no budget" from "zero"

### Deprecated

`Model::Opus4`, `Model::Sonnet4`, `Model::Opus3`, `Model::Sonnet3`, and
`Model::Haiku3` are retired upstream and return `404`. They remain in the enum
so existing code compiles; `is_retired()` detects them and `replacement()`
suggests a successor.

## [1.0.1] - 2026-01-XX

### Removed

- Deprecated Claude 3.5 series models

## [1.0.0]

Initial release: Messages API client with builder pattern, `Model` enum, tool
calling, vision, prompt caching, extended thinking, and SSE streaming.
