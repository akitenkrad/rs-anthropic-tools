//! Model identifiers for the Anthropic API.
//!
//! This module provides the [`Model`] enum for type-safe model selection:
//!
//! # Supported Models
//!
//! ## Claude 4.5 Family
//! - [`Model::Opus45`] - `claude-opus-4-5-20251101`
//!
//! ## Claude 4 Family
//! - [`Model::Opus4`] - `claude-opus-4-20250514`
//! - [`Model::Sonnet4`] - `claude-sonnet-4-20250514`
//!
//! ## Claude 3.5 Family
//! - [`Model::Haiku35`] - `claude-3-5-haiku-20241022`
//!
//! ## Claude 3 Family
//! - [`Model::Opus3`] - `claude-3-opus-20240229`
//! - [`Model::Sonnet3`] - `claude-3-sonnet-20240229`
//! - [`Model::Haiku3`] - `claude-3-haiku-20240307`
//!
//! # Example
//!
//! ```rust
//! use anthropic_tools::messages::request::model::Model;
//!
//! // Using enum variants (recommended)
//! let model = Model::Sonnet4;
//! assert_eq!(model.as_str(), "claude-sonnet-4-20250514");
//!
//! // From string (backward compatibility)
//! let model: Model = "claude-opus-4-20250514".into();
//! assert_eq!(model, Model::Opus4);
//!
//! // Custom/future models
//! let model: Model = "custom-model-v1".into();
//! assert!(matches!(model, Model::Other(_)));
//! ```

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Anthropic model identifiers
///
/// Provides type-safe model selection with backward compatibility for string-based usage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Model {
    // Claude 4.5 Family
    /// claude-opus-4-5-20251101
    Opus45,

    // Claude 4 Family
    /// claude-opus-4-20250514
    Opus4,
    /// claude-sonnet-4-20250514 (default)
    #[default]
    Sonnet4,

    // Claude 3.5 Family
    /// claude-3-5-haiku-20241022
    Haiku35,

    // Claude 3 Family
    /// claude-3-opus-20240229
    Opus3,
    /// claude-3-sonnet-20240229
    Sonnet3,
    /// claude-3-haiku-20240307
    Haiku3,

    // Forward compatibility
    /// Custom or future model
    Other(String),
}

impl Model {
    /// Get the model identifier string
    pub fn as_str(&self) -> &str {
        match self {
            Model::Opus45 => "claude-opus-4-5-20251101",
            Model::Opus4 => "claude-opus-4-20250514",
            Model::Sonnet4 => "claude-sonnet-4-20250514",
            Model::Haiku35 => "claude-3-5-haiku-20241022",
            Model::Opus3 => "claude-3-opus-20240229",
            Model::Sonnet3 => "claude-3-sonnet-20240229",
            Model::Haiku3 => "claude-3-haiku-20240307",
            Model::Other(s) => s.as_str(),
        }
    }

    /// Check if this model supports extended thinking
    pub fn supports_thinking(&self) -> bool {
        matches!(self, Model::Opus45 | Model::Opus4 | Model::Sonnet4)
    }

    /// Check if this is a known model (not Other)
    pub fn is_known(&self) -> bool {
        !matches!(self, Model::Other(_))
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
            "claude-opus-4-5-20251101" => Model::Opus45,
            "claude-opus-4-20250514" => Model::Opus4,
            "claude-sonnet-4-20250514" => Model::Sonnet4,
            "claude-3-5-haiku-20241022" => Model::Haiku35,
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

    #[test]
    fn test_model_as_str() {
        assert_eq!(Model::Opus45.as_str(), "claude-opus-4-5-20251101");
        assert_eq!(Model::Opus4.as_str(), "claude-opus-4-20250514");
        assert_eq!(Model::Sonnet4.as_str(), "claude-sonnet-4-20250514");
        assert_eq!(Model::Haiku35.as_str(), "claude-3-5-haiku-20241022");
        assert_eq!(Model::Opus3.as_str(), "claude-3-opus-20240229");
        assert_eq!(Model::Sonnet3.as_str(), "claude-3-sonnet-20240229");
        assert_eq!(Model::Haiku3.as_str(), "claude-3-haiku-20240307");
    }

    #[test]
    fn test_model_serialize() {
        let model = Model::Sonnet4;
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, "\"claude-sonnet-4-20250514\"");
    }

    #[test]
    fn test_model_serialize_other() {
        let model = Model::Other("custom-model-v1".to_string());
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, "\"custom-model-v1\"");
    }

    #[test]
    fn test_model_deserialize() {
        let json = "\"claude-sonnet-4-20250514\"";
        let model: Model = serde_json::from_str(json).unwrap();
        assert_eq!(model, Model::Sonnet4);
    }

    #[test]
    fn test_model_deserialize_unknown() {
        let json = "\"future-model-2025\"";
        let model: Model = serde_json::from_str(json).unwrap();
        assert_eq!(model, Model::Other("future-model-2025".to_string()));
    }

    #[test]
    fn test_model_from_str() {
        assert_eq!(Model::from("claude-opus-4-20250514"), Model::Opus4);
        assert_eq!(Model::from("claude-sonnet-4-20250514"), Model::Sonnet4);
        assert_eq!(
            Model::from("unknown-model"),
            Model::Other("unknown-model".to_string())
        );
    }

    #[test]
    fn test_model_from_string() {
        let s = String::from("claude-sonnet-4-20250514");
        let model: Model = s.into();
        assert_eq!(model, Model::Sonnet4);
    }

    #[test]
    fn test_model_display() {
        assert_eq!(format!("{}", Model::Sonnet4), "claude-sonnet-4-20250514");
        assert_eq!(format!("{}", Model::Other("custom".to_string())), "custom");
    }

    #[test]
    fn test_model_supports_thinking() {
        // Models that support thinking
        assert!(Model::Opus45.supports_thinking());
        assert!(Model::Opus4.supports_thinking());
        assert!(Model::Sonnet4.supports_thinking());

        // Models that don't support thinking
        assert!(!Model::Haiku35.supports_thinking());
        assert!(!Model::Opus3.supports_thinking());
        assert!(!Model::Sonnet3.supports_thinking());
        assert!(!Model::Haiku3.supports_thinking());
        assert!(!Model::Other("custom".to_string()).supports_thinking());
    }

    #[test]
    fn test_model_is_known() {
        assert!(Model::Sonnet4.is_known());
        assert!(Model::Opus4.is_known());
        assert!(!Model::Other("custom".to_string()).is_known());
    }

    #[test]
    fn test_model_default() {
        assert_eq!(Model::default(), Model::Sonnet4);
    }

    #[test]
    fn test_model_eq_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Model::Sonnet4);
        set.insert(Model::Opus4);
        set.insert(Model::Sonnet4); // duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&Model::Sonnet4));
        assert!(set.contains(&Model::Opus4));
    }
}
