//! Model identifiers for the Anthropic API.
//!
//! This module provides the [`Model`] enum for type-safe model selection:
//!
//! # Supported Models
//!
//! ## Claude 5.x Family (current)
//! - [`Model::Fable51`] - `claude-fable-5-1` - most capable model
//! - [`Model::Mythos51`] - `claude-mythos-5-1` - Project Glasswing participants only
//! - [`Model::Opus55`] - `claude-opus-5-5` - **default**
//! - [`Model::Sonnet55`] - `claude-sonnet-5-5`
//! - [`Model::Haiku55`] - `claude-haiku-5-5`
//! - [`Model::Fable5`] - `claude-fable-5` - still served
//! - [`Model::Mythos5`] - `claude-mythos-5` - still served, Project Glasswing only
//! - [`Model::Opus5`] - `claude-opus-5` - still served
//! - [`Model::Sonnet5`] - `claude-sonnet-5` - still served
//!
//! ## Claude 4.x Family (current)
//! - [`Model::Opus48`] - `claude-opus-4-8`
//! - [`Model::Opus47`] - `claude-opus-4-7`
//! - [`Model::Opus46`] - `claude-opus-4-6`
//! - [`Model::Sonnet46`] - `claude-sonnet-4-6`
//! - [`Model::Haiku45`] - `claude-haiku-4-5`
//!
//! ## Legacy (still served)
//! - [`Model::Opus45`] - `claude-opus-4-5`
//! - [`Model::Sonnet45`] - `claude-sonnet-4-5` - deprecated, retires 2026-11-30
//!
//! ## Retired (kept for backward compatibility only)
//!
//! These variants no longer resolve to a served model and will return
//! `404 not_found_error`. They remain in the enum so that existing code keeps
//! compiling; use [`Model::is_retired`] to detect them.
//!
//! - [`Model::Opus4`], [`Model::Sonnet4`] - retired 2026-06-15
//! - [`Model::Haiku3`] - retired 2026-04-20
//! - [`Model::Opus3`] - retired 2026-01-05
//! - [`Model::Sonnet3`] - retired 2025-07-21
//!
//! # Example
//!
//! ```rust
//! use anthropic_tools::messages::request::model::Model;
//!
//! // Using enum variants (recommended)
//! let model = Model::default();
//! assert_eq!(model, Model::Opus55);
//! assert_eq!(model.as_str(), "claude-opus-5-5");
//!
//! // From string (backward compatibility)
//! let model: Model = "claude-sonnet-5-5".into();
//! assert_eq!(model, Model::Sonnet55);
//!
//! // Custom/future models
//! let model: Model = "custom-model-v1".into();
//! assert!(matches!(model, Model::Other(_)));
//! ```

use crate::messages::request::body::Effort;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Anthropic model identifiers
///
/// Provides type-safe model selection with backward compatibility for string-based usage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Model {
    // Claude 5.x Family
    /// claude-fable-5-1 - most capable model
    ///
    /// Thinking is always on and cannot be disabled.
    Fable51,

    /// claude-mythos-5-1 - same capabilities as Fable 5.1, Project Glasswing only
    Mythos51,

    /// claude-opus-5-5 (default)
    ///
    /// Thinking is always on. The API default effort is `medium` (unlike
    /// Claude Opus 5, whose default is `high`).
    #[default]
    Opus55,

    /// claude-sonnet-5-5
    ///
    /// Use `thinking: {"type":"between_tools"}` to turn thinking off between
    /// tool calls.
    Sonnet55,

    /// claude-haiku-5-5
    ///
    /// The API default effort is `medium`.
    Haiku55,

    // Claude 5 Family (still served)
    /// claude-fable-5 - most capable widely released model
    ///
    /// Thinking is always on and cannot be disabled. Requires 30-day data
    /// retention (not available under zero data retention).
    Fable5,

    /// claude-mythos-5 - same capabilities as Fable 5, Project Glasswing only
    Mythos5,

    /// claude-opus-5
    Opus5,

    /// claude-sonnet-5
    Sonnet5,

    // Claude 4.x Family
    /// claude-opus-4-8
    Opus48,
    /// claude-opus-4-7
    Opus47,
    /// claude-opus-4-6
    Opus46,
    /// claude-sonnet-4-6
    Sonnet46,
    /// claude-haiku-4-5
    Haiku45,

    // Legacy (still served)
    /// claude-opus-4-5
    Opus45,
    /// claude-sonnet-4-5 - **deprecated, retires 2026-11-30**
    Sonnet45,

    // Retired - kept for backward compatibility, will return 404
    /// claude-opus-4-20250514 - **retired 2026-06-15**
    Opus4,
    /// claude-sonnet-4-20250514 - **retired 2026-06-15**
    Sonnet4,
    /// claude-3-opus-20240229 - **retired 2026-01-05**
    Opus3,
    /// claude-3-sonnet-20240229 - **retired 2025-07-21**
    Sonnet3,
    /// claude-3-haiku-20240307 - **retired 2026-04-20**
    Haiku3,

    // Forward compatibility
    /// Custom or future model
    Other(String),
}

impl Model {
    /// Get the model identifier string
    pub fn as_str(&self) -> &str {
        match self {
            Model::Fable51 => "claude-fable-5-1",
            Model::Mythos51 => "claude-mythos-5-1",
            Model::Opus55 => "claude-opus-5-5",
            Model::Sonnet55 => "claude-sonnet-5-5",
            Model::Haiku55 => "claude-haiku-5-5",
            Model::Fable5 => "claude-fable-5",
            Model::Mythos5 => "claude-mythos-5",
            Model::Opus5 => "claude-opus-5",
            Model::Sonnet5 => "claude-sonnet-5",
            Model::Opus48 => "claude-opus-4-8",
            Model::Opus47 => "claude-opus-4-7",
            Model::Opus46 => "claude-opus-4-6",
            Model::Sonnet46 => "claude-sonnet-4-6",
            Model::Haiku45 => "claude-haiku-4-5",
            Model::Opus45 => "claude-opus-4-5",
            Model::Sonnet45 => "claude-sonnet-4-5",
            Model::Opus4 => "claude-opus-4-20250514",
            Model::Sonnet4 => "claude-sonnet-4-20250514",
            Model::Opus3 => "claude-3-opus-20240229",
            Model::Sonnet3 => "claude-3-sonnet-20240229",
            Model::Haiku3 => "claude-3-haiku-20240307",
            Model::Other(s) => s.as_str(),
        }
    }

    /// Check if this model supports extended thinking in any form
    ///
    /// Note that *how* thinking is configured differs by model — see
    /// [`Model::supports_adaptive_thinking`],
    /// [`Model::supports_between_tools_thinking`], and
    /// [`Model::supports_budget_tokens`].
    pub fn supports_thinking(&self) -> bool {
        matches!(
            self,
            Model::Fable51
                | Model::Mythos51
                | Model::Opus55
                | Model::Sonnet55
                | Model::Haiku55
                | Model::Fable5
                | Model::Mythos5
                | Model::Opus5
                | Model::Sonnet5
                | Model::Opus48
                | Model::Opus47
                | Model::Opus46
                | Model::Sonnet46
                | Model::Haiku45
                | Model::Opus45
                | Model::Sonnet45
                | Model::Opus4
                | Model::Sonnet4
        )
    }

    /// Check if this model supports adaptive thinking (`{"type": "adaptive"}`)
    ///
    /// Adaptive thinking replaces the deprecated fixed token budget and lets
    /// Claude decide when and how much to think.
    pub fn supports_adaptive_thinking(&self) -> bool {
        matches!(
            self,
            Model::Fable51
                | Model::Mythos51
                | Model::Opus55
                | Model::Sonnet55
                | Model::Haiku55
                | Model::Fable5
                | Model::Mythos5
                | Model::Opus5
                | Model::Sonnet5
                | Model::Opus48
                | Model::Opus47
                | Model::Opus46
                | Model::Sonnet46
        )
    }

    /// Check if this model accepts `thinking: {"type": "enabled", "budget_tokens": N}`
    ///
    /// Current Claude 5.x models reject `budget_tokens` with a `400`; use
    /// adaptive thinking with [`Effort`] instead.
    pub fn supports_budget_tokens(&self) -> bool {
        matches!(
            self,
            Model::Opus46
                | Model::Sonnet46
                | Model::Haiku45
                | Model::Opus45
                | Model::Sonnet45
                | Model::Opus4
                | Model::Sonnet4
        )
    }

    /// Check if this model accepts `thinking: {"type": "disabled"}`
    ///
    /// Claude Fable 5/5.1, Mythos 5/5.1, and Opus 5.5 always think; an explicit
    /// `disabled` configuration returns a `400`. Sonnet 5.5 uses
    /// `between_tools` instead.
    ///
    /// On Claude Opus 5 and Haiku 5.5, `disabled` is additionally capped to
    /// effort `high` or lower — see [`Model::allows_disabled_thinking_at`].
    pub fn supports_disabled_thinking(&self) -> bool {
        !matches!(
            self,
            Model::Fable51
                | Model::Mythos51
                | Model::Opus55
                | Model::Sonnet55
                | Model::Fable5
                | Model::Mythos5
        )
    }

    /// Check if disabled thinking is allowed at the given effort level
    ///
    /// Claude Opus 5 and Haiku 5.5 reject `thinking: {"type": "disabled"}`
    /// when effort is `xhigh` or `max`.
    pub fn allows_disabled_thinking_at(&self, effort: Option<Effort>) -> bool {
        if !self.supports_disabled_thinking() {
            return false;
        }
        !matches!(
            (self, effort),
            (
                Model::Opus5 | Model::Haiku55,
                Some(Effort::XHigh) | Some(Effort::Max)
            )
        )
    }

    /// Check if thinking is always on for this model (cannot be turned off)
    pub fn thinking_always_on(&self) -> bool {
        matches!(
            self,
            Model::Fable51 | Model::Mythos51 | Model::Opus55 | Model::Fable5 | Model::Mythos5
        )
    }

    /// Check if this model accepts `thinking: {"type": "between_tools"}`
    pub fn supports_between_tools_thinking(&self) -> bool {
        matches!(self, Model::Sonnet55)
    }

    /// Check if this model accepts forced `tool_choice` values (`any` or `tool`)
    pub fn supports_forced_tool_choice(&self) -> bool {
        !matches!(
            self,
            Model::Fable51 | Model::Mythos51 | Model::Opus55 | Model::Sonnet55
        )
    }

    /// Check if this model accepts `temperature`, `top_p`, and `top_k`
    ///
    /// Returns `false` for Opus 4.7 and later and current Claude 5.x models —
    /// those models reject sampling parameters with a `400`. Steer behavior
    /// via prompting instead.
    pub fn supports_sampling_params(&self) -> bool {
        !matches!(
            self,
            Model::Fable51
                | Model::Mythos51
                | Model::Opus55
                | Model::Sonnet55
                | Model::Haiku55
                | Model::Fable5
                | Model::Mythos5
                | Model::Opus5
                | Model::Sonnet5
                | Model::Opus48
                | Model::Opus47
        )
    }

    /// Check if this model supports `output_config.effort`
    pub fn supports_effort(&self) -> bool {
        matches!(
            self,
            Model::Fable51
                | Model::Mythos51
                | Model::Opus55
                | Model::Sonnet55
                | Model::Haiku55
                | Model::Fable5
                | Model::Mythos5
                | Model::Opus5
                | Model::Sonnet5
                | Model::Opus48
                | Model::Opus47
                | Model::Opus46
                | Model::Sonnet46
                | Model::Opus45
        )
    }

    /// Check if this model supports a specific effort level
    ///
    /// Opus 4.5 supports only `low`, `medium`, and `high`; `xhigh` arrived with
    /// Opus 4.7 and `max` with Opus 4.6.
    pub fn supports_effort_level(&self, effort: Effort) -> bool {
        if !self.supports_effort() {
            return false;
        }
        match effort {
            Effort::Low | Effort::Medium | Effort::High => true,
            Effort::Max => !matches!(self, Model::Opus45),
            Effort::XHigh => matches!(
                self,
                Model::Fable51
                    | Model::Mythos51
                    | Model::Opus55
                    | Model::Sonnet55
                    | Model::Haiku55
                    | Model::Fable5
                    | Model::Mythos5
                    | Model::Opus5
                    | Model::Sonnet5
                    | Model::Opus48
                    | Model::Opus47
            ),
        }
    }

    /// Check if this model supports assistant-turn prefilling
    ///
    /// Returns `false` for the 4.6 family and later — a trailing `assistant`
    /// message returns a `400`. Use structured outputs
    /// (`output_config.format`) or a system prompt instruction instead.
    pub fn supports_prefill(&self) -> bool {
        !matches!(
            self,
            Model::Fable51
                | Model::Mythos51
                | Model::Opus55
                | Model::Sonnet55
                | Model::Haiku55
                | Model::Fable5
                | Model::Mythos5
                | Model::Opus5
                | Model::Sonnet5
                | Model::Opus48
                | Model::Opus47
                | Model::Opus46
                | Model::Sonnet46
        )
    }

    /// Get the context window (maximum input tokens) for this model
    ///
    /// Returns `None` for [`Model::Other`].
    pub fn context_window(&self) -> Option<usize> {
        match self {
            Model::Fable51
            | Model::Mythos51
            | Model::Opus55
            | Model::Sonnet55
            | Model::Haiku55
            | Model::Fable5
            | Model::Mythos5
            | Model::Opus5
            | Model::Sonnet5
            | Model::Opus48
            | Model::Opus47
            | Model::Opus46
            | Model::Sonnet46 => Some(1_000_000),
            Model::Haiku45
            | Model::Opus45
            | Model::Sonnet45
            | Model::Opus4
            | Model::Sonnet4
            | Model::Opus3
            | Model::Sonnet3
            | Model::Haiku3 => Some(200_000),
            Model::Other(_) => None,
        }
    }

    /// Get the maximum output tokens for this model
    ///
    /// Returns `None` for [`Model::Other`]. Values above roughly 16000 require
    /// streaming to avoid HTTP timeouts.
    pub fn max_output_tokens(&self) -> Option<usize> {
        match self {
            Model::Fable51
            | Model::Mythos51
            | Model::Opus55
            | Model::Sonnet55
            | Model::Haiku55
            | Model::Fable5
            | Model::Mythos5
            | Model::Opus5
            | Model::Sonnet5
            | Model::Opus48
            | Model::Opus47
            | Model::Opus46
            | Model::Sonnet46 => Some(128_000),
            Model::Haiku45 => Some(64_000),
            Model::Opus45 | Model::Sonnet45 => Some(64_000),
            Model::Opus4 | Model::Sonnet4 => Some(64_000),
            Model::Opus3 | Model::Sonnet3 | Model::Haiku3 => Some(4_096),
            Model::Other(_) => None,
        }
    }

    /// Check if this is a known model (not [`Model::Other`])
    pub fn is_known(&self) -> bool {
        !matches!(self, Model::Other(_))
    }

    /// Check if this model has been retired and will return `404`
    pub fn is_retired(&self) -> bool {
        matches!(
            self,
            Model::Opus4 | Model::Sonnet4 | Model::Opus3 | Model::Sonnet3 | Model::Haiku3
        )
    }

    /// Get the recommended replacement for a retired model
    ///
    /// Returns `None` for models that are still served.
    pub fn replacement(&self) -> Option<Model> {
        match self {
            Model::Opus4 | Model::Opus3 => Some(Model::Opus55),
            Model::Sonnet4 | Model::Sonnet3 => Some(Model::Sonnet55),
            Model::Haiku3 => Some(Model::Haiku55),
            _ => None,
        }
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// From implementations for backward compatibility
impl From<&str> for Model {
    fn from(s: &str) -> Self {
        match s {
            "claude-fable-5-1" => Model::Fable51,
            "claude-mythos-5-1" => Model::Mythos51,
            "claude-opus-5-5" => Model::Opus55,
            "claude-sonnet-5-5" => Model::Sonnet55,
            "claude-haiku-5-5" => Model::Haiku55,
            "claude-fable-5" => Model::Fable5,
            "claude-mythos-5" => Model::Mythos5,
            "claude-opus-5" => Model::Opus5,
            "claude-sonnet-5" => Model::Sonnet5,
            "claude-opus-4-8" => Model::Opus48,
            "claude-opus-4-7" => Model::Opus47,
            "claude-opus-4-6" => Model::Opus46,
            "claude-sonnet-4-6" => Model::Sonnet46,
            "claude-haiku-4-5" | "claude-haiku-4-5-20251001" => Model::Haiku45,
            "claude-opus-4-5" | "claude-opus-4-5-20251101" => Model::Opus45,
            "claude-sonnet-4-5" | "claude-sonnet-4-5-20250929" => Model::Sonnet45,
            "claude-opus-4-0" | "claude-opus-4-20250514" => Model::Opus4,
            "claude-sonnet-4-0" | "claude-sonnet-4-20250514" => Model::Sonnet4,
            "claude-3-opus-20240229" => Model::Opus3,
            "claude-3-sonnet-20240229" => Model::Sonnet3,
            "claude-3-haiku-20240307" => Model::Haiku3,
            other => Model::Other(other.to_string()),
        }
    }
}

impl From<String> for Model {
    fn from(s: String) -> Self {
        Model::from(s.as_str())
    }
}

// Custom serde (plain string, not tagged)
impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Model::from(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_models() -> [Model; 5] {
        [
            Model::Fable51,
            Model::Mythos51,
            Model::Opus55,
            Model::Sonnet55,
            Model::Haiku55,
        ]
    }

    #[test]
    fn test_model_as_str() {
        assert_eq!(Model::Fable51.as_str(), "claude-fable-5-1");
        assert_eq!(Model::Mythos51.as_str(), "claude-mythos-5-1");
        assert_eq!(Model::Opus55.as_str(), "claude-opus-5-5");
        assert_eq!(Model::Sonnet55.as_str(), "claude-sonnet-5-5");
        assert_eq!(Model::Haiku55.as_str(), "claude-haiku-5-5");
        assert_eq!(Model::Fable5.as_str(), "claude-fable-5");
        assert_eq!(Model::Opus5.as_str(), "claude-opus-5");
        assert_eq!(Model::Sonnet5.as_str(), "claude-sonnet-5");
        assert_eq!(Model::Opus48.as_str(), "claude-opus-4-8");
        assert_eq!(Model::Opus47.as_str(), "claude-opus-4-7");
        assert_eq!(Model::Opus46.as_str(), "claude-opus-4-6");
        assert_eq!(Model::Sonnet46.as_str(), "claude-sonnet-4-6");
        assert_eq!(Model::Haiku45.as_str(), "claude-haiku-4-5");
        assert_eq!(Model::Opus45.as_str(), "claude-opus-4-5");
        assert_eq!(Model::Sonnet45.as_str(), "claude-sonnet-4-5");
        // Retired models keep their original identifiers
        assert_eq!(Model::Opus4.as_str(), "claude-opus-4-20250514");
        assert_eq!(Model::Sonnet4.as_str(), "claude-sonnet-4-20250514");
        assert_eq!(Model::Haiku3.as_str(), "claude-3-haiku-20240307");
    }

    #[test]
    fn test_model_serialize() {
        let model = Model::Opus55;
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, "\"claude-opus-5-5\"");
    }

    #[test]
    fn test_model_serialize_other() {
        let model = Model::Other("custom-model-v1".to_string());
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, "\"custom-model-v1\"");
    }

    #[test]
    fn test_model_deserialize() {
        for (id, expected) in [
            ("claude-fable-5-1", Model::Fable51),
            ("claude-mythos-5-1", Model::Mythos51),
            ("claude-opus-5-5", Model::Opus55),
            ("claude-sonnet-5-5", Model::Sonnet55),
            ("claude-haiku-5-5", Model::Haiku55),
        ] {
            let model: Model = serde_json::from_str(&format!(r#""{id}""#)).unwrap();
            assert_eq!(model, expected);
        }
    }

    #[test]
    fn test_model_deserialize_unknown() {
        let json = "\"future-model-2027\"";
        let model: Model = serde_json::from_str(json).unwrap();
        assert_eq!(model, Model::Other("future-model-2027".to_string()));
    }

    #[test]
    fn test_model_from_str() {
        assert_eq!(Model::from("claude-fable-5-1"), Model::Fable51);
        assert_eq!(Model::from("claude-mythos-5-1"), Model::Mythos51);
        assert_eq!(Model::from("claude-opus-5-5"), Model::Opus55);
        assert_eq!(Model::from("claude-sonnet-5-5"), Model::Sonnet55);
        assert_eq!(Model::from("claude-haiku-5-5"), Model::Haiku55);
        assert_eq!(Model::from("claude-opus-5"), Model::Opus5);
        assert_eq!(Model::from("claude-opus-4-8"), Model::Opus48);
        assert_eq!(
            Model::from("unknown-model"),
            Model::Other("unknown-model".to_string())
        );
    }

    #[test]
    fn test_model_from_dated_alias() {
        // Dated full IDs resolve to their alias variant
        assert_eq!(Model::from("claude-haiku-4-5-20251001"), Model::Haiku45);
        assert_eq!(Model::from("claude-opus-4-5-20251101"), Model::Opus45);
        assert_eq!(Model::from("claude-sonnet-4-5-20250929"), Model::Sonnet45);
    }

    #[test]
    fn test_model_from_string() {
        let s = String::from("claude-opus-5");
        let model: Model = s.into();
        assert_eq!(model, Model::Opus5);
    }

    #[test]
    fn test_model_display() {
        assert_eq!(format!("{}", Model::Opus5), "claude-opus-5");
        assert_eq!(format!("{}", Model::Other("custom".to_string())), "custom");
    }

    #[test]
    fn test_model_supports_thinking() {
        for model in new_models() {
            assert!(model.supports_thinking(), "{model}");
        }
        assert!(Model::Opus5.supports_thinking());
        assert!(Model::Sonnet5.supports_thinking());
        assert!(Model::Haiku45.supports_thinking());

        assert!(!Model::Opus3.supports_thinking());
        assert!(!Model::Haiku3.supports_thinking());
        assert!(!Model::Other("custom".to_string()).supports_thinking());
    }

    #[test]
    fn test_model_supports_adaptive_thinking() {
        for model in new_models() {
            assert!(model.supports_adaptive_thinking(), "{model}");
        }
        assert!(Model::Opus5.supports_adaptive_thinking());
        assert!(Model::Sonnet46.supports_adaptive_thinking());
        assert!(Model::Fable5.supports_adaptive_thinking());

        // Haiku 4.5 and older use the fixed token budget instead
        assert!(!Model::Haiku45.supports_adaptive_thinking());
        assert!(!Model::Sonnet4.supports_adaptive_thinking());
    }

    #[test]
    fn test_model_supports_budget_tokens() {
        for model in new_models() {
            assert!(!model.supports_budget_tokens(), "{model}");
        }
        assert!(Model::Haiku45.supports_budget_tokens());
        assert!(Model::Opus46.supports_budget_tokens());
        assert!(Model::Sonnet4.supports_budget_tokens());

        // Removed on Opus 4.7 and later
        assert!(!Model::Opus47.supports_budget_tokens());
        assert!(!Model::Opus48.supports_budget_tokens());
        assert!(!Model::Opus5.supports_budget_tokens());
        assert!(!Model::Sonnet5.supports_budget_tokens());
        assert!(!Model::Fable5.supports_budget_tokens());
    }

    #[test]
    fn test_model_disabled_thinking() {
        assert!(!Model::Fable51.supports_disabled_thinking());
        assert!(!Model::Mythos51.supports_disabled_thinking());
        assert!(!Model::Opus55.supports_disabled_thinking());
        assert!(!Model::Sonnet55.supports_disabled_thinking());
        assert!(Model::Haiku55.supports_disabled_thinking());

        assert!(Model::Fable51.thinking_always_on());
        assert!(Model::Mythos51.thinking_always_on());
        assert!(Model::Opus55.thinking_always_on());
        assert!(!Model::Sonnet55.thinking_always_on());
        assert!(!Model::Haiku55.thinking_always_on());

        assert!(!Model::Opus55.allows_disabled_thinking_at(Some(Effort::Low)));
        assert!(Model::Haiku55.allows_disabled_thinking_at(Some(Effort::High)));
        assert!(!Model::Haiku55.allows_disabled_thinking_at(Some(Effort::XHigh)));
        assert!(!Model::Haiku55.allows_disabled_thinking_at(Some(Effort::Max)));

        assert!(Model::Opus5.supports_disabled_thinking());
        assert!(!Model::Fable5.supports_disabled_thinking());
        assert!(!Model::Mythos5.supports_disabled_thinking());

        assert!(Model::Fable5.thinking_always_on());
        assert!(!Model::Opus5.thinking_always_on());

        // Opus 5 caps disabled thinking at effort "high"
        assert!(Model::Opus5.allows_disabled_thinking_at(Some(Effort::High)));
        assert!(!Model::Opus5.allows_disabled_thinking_at(Some(Effort::XHigh)));
        assert!(!Model::Opus5.allows_disabled_thinking_at(Some(Effort::Max)));
        assert!(Model::Opus48.allows_disabled_thinking_at(Some(Effort::Max)));
    }

    #[test]
    fn test_model_supports_sampling_params() {
        for model in new_models() {
            assert!(!model.supports_sampling_params(), "{model}");
        }
        assert!(Model::Haiku45.supports_sampling_params());
        assert!(Model::Opus46.supports_sampling_params());
        assert!(Model::Sonnet46.supports_sampling_params());

        assert!(!Model::Opus47.supports_sampling_params());
        assert!(!Model::Opus5.supports_sampling_params());
        assert!(!Model::Sonnet5.supports_sampling_params());
        assert!(!Model::Fable5.supports_sampling_params());
    }

    #[test]
    fn test_model_supports_effort() {
        for model in new_models() {
            assert!(model.supports_effort(), "{model}");
            for effort in [
                Effort::Low,
                Effort::Medium,
                Effort::High,
                Effort::XHigh,
                Effort::Max,
            ] {
                assert!(model.supports_effort_level(effort), "{model}: {effort:?}");
            }
        }
        assert!(Model::Opus5.supports_effort());
        assert!(Model::Opus45.supports_effort());
        assert!(!Model::Haiku45.supports_effort());
        assert!(!Model::Sonnet45.supports_effort());

        // xhigh arrived with Opus 4.7
        assert!(Model::Opus5.supports_effort_level(Effort::XHigh));
        assert!(!Model::Opus46.supports_effort_level(Effort::XHigh));
        // max is unavailable on Opus 4.5
        assert!(!Model::Opus45.supports_effort_level(Effort::Max));
        assert!(Model::Opus45.supports_effort_level(Effort::High));
    }

    #[test]
    fn test_model_supports_prefill() {
        for model in new_models() {
            assert!(!model.supports_prefill(), "{model}");
        }
        assert!(Model::Haiku45.supports_prefill());
        assert!(Model::Opus45.supports_prefill());
        assert!(!Model::Opus46.supports_prefill());
        assert!(!Model::Opus5.supports_prefill());
        assert!(!Model::Fable5.supports_prefill());
    }

    #[test]
    fn test_model_limits() {
        for model in new_models() {
            assert_eq!(model.context_window(), Some(1_000_000), "{model}");
            assert_eq!(model.max_output_tokens(), Some(128_000), "{model}");
        }
        assert_eq!(Model::Opus5.context_window(), Some(1_000_000));
        assert_eq!(Model::Haiku45.context_window(), Some(200_000));
        assert_eq!(Model::Other("x".into()).context_window(), None);

        assert_eq!(Model::Opus5.max_output_tokens(), Some(128_000));
        assert_eq!(Model::Haiku45.max_output_tokens(), Some(64_000));
        assert_eq!(Model::Other("x".into()).max_output_tokens(), None);
    }

    #[test]
    fn test_model_is_known() {
        assert!(Model::Opus5.is_known());
        assert!(Model::Sonnet46.is_known());
        assert!(!Model::Other("custom".to_string()).is_known());
    }

    #[test]
    fn test_model_is_retired() {
        assert!(Model::Sonnet4.is_retired());
        assert!(Model::Opus4.is_retired());
        assert!(Model::Opus3.is_retired());
        assert!(Model::Haiku3.is_retired());

        assert!(!Model::Opus5.is_retired());
        assert!(!Model::Opus45.is_retired());
        assert!(!Model::Other("custom".to_string()).is_retired());
    }

    #[test]
    fn test_model_replacement() {
        assert_eq!(Model::Sonnet4.replacement(), Some(Model::Sonnet55));
        assert_eq!(Model::Sonnet3.replacement(), Some(Model::Sonnet55));
        assert_eq!(Model::Opus4.replacement(), Some(Model::Opus55));
        assert_eq!(Model::Opus3.replacement(), Some(Model::Opus55));
        assert_eq!(Model::Haiku3.replacement(), Some(Model::Haiku55));
        assert_eq!(Model::Opus5.replacement(), None);
    }

    #[test]
    fn test_model_default() {
        assert_eq!(Model::default(), Model::Opus55);
    }

    #[test]
    fn test_model_supports_between_tools_thinking() {
        assert!(Model::Sonnet55.supports_between_tools_thinking());
        assert!(!Model::Fable51.supports_between_tools_thinking());
        assert!(!Model::Mythos51.supports_between_tools_thinking());
        assert!(!Model::Opus55.supports_between_tools_thinking());
        assert!(!Model::Haiku55.supports_between_tools_thinking());
        assert!(!Model::Sonnet5.supports_between_tools_thinking());
    }

    #[test]
    fn test_model_supports_forced_tool_choice() {
        assert!(!Model::Fable51.supports_forced_tool_choice());
        assert!(!Model::Mythos51.supports_forced_tool_choice());
        assert!(!Model::Opus55.supports_forced_tool_choice());
        assert!(!Model::Sonnet55.supports_forced_tool_choice());
        assert!(Model::Haiku55.supports_forced_tool_choice());
        assert!(Model::Opus5.supports_forced_tool_choice());
    }

    #[test]
    fn test_model_eq_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Model::Opus5);
        set.insert(Model::Sonnet5);
        set.insert(Model::Opus5); // duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&Model::Opus5));
        assert!(set.contains(&Model::Sonnet5));
    }
}
