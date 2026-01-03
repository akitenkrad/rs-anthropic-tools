//! Tool definitions for function calling with the Anthropic API.
//!
//! This module provides types for defining tools that Claude can use:
//!
//! - [`Tool`] - Main tool definition with name, description, and input schema
//! - [`JsonSchema`] - JSON Schema for tool input parameters
//! - [`PropertyDef`] - Property definitions within a schema
//! - [`CacheControl`] - Cache control for prompt caching
//!
//! # Example
//!
//! ```rust
//! use anthropic_tools::common::tool::{Tool, PropertyDef};
//!
//! // Create a weather tool
//! let mut tool = Tool::new("get_weather");
//! tool.description("Get the current weather for a location")
//!     .add_string_property("location", Some("City and state, e.g., San Francisco, CA"), true)
//!     .add_enum_property("unit", Some("Temperature unit"), vec!["celsius", "fahrenheit"], false);
//!
//! // Convert to JSON for API request
//! let json = tool.to_value();
//! ```
//!
//! # With Prompt Caching
//!
//! ```rust
//! use anthropic_tools::common::tool::Tool;
//!
//! let mut tool = Tool::new("expensive_tool");
//! tool.description("A tool with many parameters")
//!     .with_cache();  // Enable prompt caching
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool definition for the Anthropic API
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tool {
    /// Name of the tool
    pub name: String,

    /// Description of what the tool does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// JSON schema for the tool's input parameters
    pub input_schema: JsonSchema,

    /// Cache control for prompt caching
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Cache control for prompt caching
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub type_name: String,
}

impl CacheControl {
    pub fn ephemeral() -> Self {
        CacheControl {
            type_name: "ephemeral".to_string(),
        }
    }
}

/// JSON Schema for tool input
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonSchema {
    #[serde(rename = "type")]
    pub type_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, PropertyDef>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<bool>,
}

/// Property definition in JSON schema
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PropertyDef {
    #[serde(rename = "type")]
    pub type_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<PropertyDef>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, PropertyDef>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(rename = "default", skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
}

impl Tool {
    /// Create a new tool with name only
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Tool {
            name: name.as_ref().to_string(),
            description: None,
            input_schema: JsonSchema::object(),
            cache_control: None,
        }
    }

    /// Set the tool description
    pub fn description<S: AsRef<str>>(&mut self, desc: S) -> &mut Self {
        self.description = Some(desc.as_ref().to_string());
        self
    }

    /// Add a string property to the input schema
    pub fn add_string_property<S: AsRef<str>>(
        &mut self,
        name: S,
        description: Option<S>,
        required: bool,
    ) -> &mut Self {
        self.add_property(
            name.as_ref(),
            PropertyDef::string(description.map(|s| s.as_ref().to_string())),
            required,
        )
    }

    /// Add a number property to the input schema
    pub fn add_number_property<S: AsRef<str>>(
        &mut self,
        name: S,
        description: Option<S>,
        required: bool,
    ) -> &mut Self {
        self.add_property(
            name.as_ref(),
            PropertyDef::number(description.map(|s| s.as_ref().to_string())),
            required,
        )
    }

    /// Add a boolean property to the input schema
    pub fn add_boolean_property<S: AsRef<str>>(
        &mut self,
        name: S,
        description: Option<S>,
        required: bool,
    ) -> &mut Self {
        self.add_property(
            name.as_ref(),
            PropertyDef::boolean(description.map(|s| s.as_ref().to_string())),
            required,
        )
    }

    /// Add an enum property to the input schema
    pub fn add_enum_property<S: AsRef<str>>(
        &mut self,
        name: S,
        description: Option<S>,
        values: Vec<S>,
        required: bool,
    ) -> &mut Self {
        self.add_property(
            name.as_ref(),
            PropertyDef::enum_type(
                description.map(|s| s.as_ref().to_string()),
                values.into_iter().map(|s| s.as_ref().to_string()).collect(),
            ),
            required,
        )
    }

    /// Add an array property to the input schema
    pub fn add_array_property<S: AsRef<str>>(
        &mut self,
        name: S,
        description: Option<S>,
        items: PropertyDef,
        required: bool,
    ) -> &mut Self {
        self.add_property(
            name.as_ref(),
            PropertyDef::array(description.map(|s| s.as_ref().to_string()), items),
            required,
        )
    }

    /// Add a property with custom PropertyDef
    fn add_property(&mut self, name: &str, prop: PropertyDef, required: bool) -> &mut Self {
        if self.input_schema.properties.is_none() {
            self.input_schema.properties = Some(HashMap::new());
        }

        if let Some(props) = &mut self.input_schema.properties {
            props.insert(name.to_string(), prop);
        }

        if required {
            if self.input_schema.required.is_none() {
                self.input_schema.required = Some(Vec::new());
            }
            if let Some(req) = &mut self.input_schema.required {
                req.push(name.to_string());
            }
        }

        self
    }

    /// Enable cache control for this tool
    pub fn with_cache(&mut self) -> &mut Self {
        self.cache_control = Some(CacheControl::ephemeral());
        self
    }

    /// Build the tool and return ownership
    pub fn build(self) -> Self {
        self
    }

    /// Convert to serde_json::Value
    pub fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

impl JsonSchema {
    /// Create an object schema
    pub fn object() -> Self {
        JsonSchema {
            type_name: "object".to_string(),
            properties: Some(HashMap::new()),
            required: None,
            additional_properties: None,
        }
    }

    /// Create an empty object schema (no properties)
    pub fn empty_object() -> Self {
        JsonSchema {
            type_name: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: None,
        }
    }
}

impl PropertyDef {
    /// Create a string property
    pub fn string(description: Option<String>) -> Self {
        PropertyDef {
            type_name: "string".to_string(),
            description,
            enum_values: None,
            items: None,
            properties: None,
            required: None,
            default_value: None,
        }
    }

    /// Create a number property
    pub fn number(description: Option<String>) -> Self {
        PropertyDef {
            type_name: "number".to_string(),
            description,
            enum_values: None,
            items: None,
            properties: None,
            required: None,
            default_value: None,
        }
    }

    /// Create an integer property
    pub fn integer(description: Option<String>) -> Self {
        PropertyDef {
            type_name: "integer".to_string(),
            description,
            enum_values: None,
            items: None,
            properties: None,
            required: None,
            default_value: None,
        }
    }

    /// Create a boolean property
    pub fn boolean(description: Option<String>) -> Self {
        PropertyDef {
            type_name: "boolean".to_string(),
            description,
            enum_values: None,
            items: None,
            properties: None,
            required: None,
            default_value: None,
        }
    }

    /// Create an enum (string with allowed values) property
    pub fn enum_type(description: Option<String>, values: Vec<String>) -> Self {
        PropertyDef {
            type_name: "string".to_string(),
            description,
            enum_values: Some(values),
            items: None,
            properties: None,
            required: None,
            default_value: None,
        }
    }

    /// Create an array property
    pub fn array(description: Option<String>, items: PropertyDef) -> Self {
        PropertyDef {
            type_name: "array".to_string(),
            description,
            enum_values: None,
            items: Some(Box::new(items)),
            properties: None,
            required: None,
            default_value: None,
        }
    }

    /// Create an object property
    pub fn object(description: Option<String>, properties: HashMap<String, PropertyDef>) -> Self {
        PropertyDef {
            type_name: "object".to_string(),
            description,
            enum_values: None,
            items: None,
            properties: Some(properties),
            required: None,
            default_value: None,
        }
    }

    /// Set a default value
    pub fn with_default(&mut self, value: serde_json::Value) -> &mut Self {
        self.default_value = Some(value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_builder() {
        let mut tool = Tool::new("search");
        tool.description("Search the web for information")
            .add_string_property("query", Some("The search query"), true)
            .add_number_property("limit", Some("Maximum results to return"), false);

        assert_eq!(tool.name, "search");
        assert!(tool.description.is_some());
        assert!(tool.input_schema.properties.is_some());

        let props = tool.input_schema.properties.as_ref().unwrap();
        assert!(props.contains_key("query"));
        assert!(props.contains_key("limit"));

        let required = tool.input_schema.required.as_ref().unwrap();
        assert!(required.contains(&"query".to_string()));
        assert!(!required.contains(&"limit".to_string()));
    }

    #[test]
    fn test_tool_serialize() {
        let mut tool = Tool::new("get_weather");
        tool.description("Get the current weather in a given location")
            .add_string_property("location", Some("The city and state, e.g. San Francisco, CA"), true)
            .add_enum_property(
                "unit",
                Some("Temperature unit"),
                vec!["celsius", "fahrenheit"],
                false,
            );

        let json = serde_json::to_string_pretty(&tool).unwrap();
        assert!(json.contains("\"name\": \"get_weather\""));
        assert!(json.contains("\"type\": \"object\""));
        assert!(json.contains("\"location\""));
        assert!(json.contains("\"required\""));
    }

    #[test]
    fn test_property_def_string() {
        let prop = PropertyDef::string(Some("A test property".to_string()));
        assert_eq!(prop.type_name, "string");
        assert_eq!(prop.description, Some("A test property".to_string()));
    }

    #[test]
    fn test_property_def_enum() {
        let prop = PropertyDef::enum_type(
            Some("Choose a color".to_string()),
            vec!["red".to_string(), "green".to_string(), "blue".to_string()],
        );
        assert_eq!(prop.type_name, "string");
        assert!(prop.enum_values.is_some());
        assert_eq!(prop.enum_values.unwrap().len(), 3);
    }

    #[test]
    fn test_property_def_array() {
        let items = PropertyDef::string(Some("Item in array".to_string()));
        let prop = PropertyDef::array(Some("An array of strings".to_string()), items);
        assert_eq!(prop.type_name, "array");
        assert!(prop.items.is_some());
    }

    #[test]
    fn test_tool_with_cache() {
        let mut tool = Tool::new("cached_tool");
        tool.with_cache();
        assert!(tool.cache_control.is_some());
    }

    #[test]
    fn test_tool_to_value() {
        let mut tool = Tool::new("test");
        tool.description("A test tool");
        let value = tool.to_value();
        assert!(value.is_object());
        assert_eq!(value["name"], "test");
    }
}
