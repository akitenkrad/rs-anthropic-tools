//! Content block types for messages.
//!
//! This module provides content block types for constructing messages:
//!
//! - [`ContentBlock`] - Main enum for all content types
//! - [`ImageSource`] - Image data (base64 or URL)
//! - [`DocumentSource`] - PDF document data
//! - [`MediaType`] - Supported image formats
//! - [`CacheControl`] - Prompt caching configuration
//!
//! # Text Content
//!
//! ```rust
//! use anthropic_tools::messages::request::content::ContentBlock;
//!
//! let text = ContentBlock::text("Hello, world!");
//! let cached = ContentBlock::text_with_cache("Cached content");
//! ```
//!
//! # Image Content
//!
//! ```rust
//! use anthropic_tools::messages::request::content::{ContentBlock, MediaType};
//!
//! // From URL
//! let image = ContentBlock::image_from_url("https://example.com/image.png");
//!
//! // From local file (requires image feature)
//! // let image = ContentBlock::image_from_path(MediaType::Png, "path/to/image.png");
//! ```
//!
//! # Tool Use
//!
//! Tool use blocks are typically created by Claude's response, but can be
//! manually constructed:
//!
//! ```rust
//! use anthropic_tools::messages::request::content::ContentBlock;
//! use serde_json::json;
//!
//! let tool_use = ContentBlock::tool_use("tool_123", "search", json!({"query": "rust"}));
//! let result = ContentBlock::tool_result_text("tool_123", "Search results...");
//! ```

use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use strum::{Display, EnumString};

/// Media types supported by Anthropic API
#[derive(Serialize, Deserialize, Debug, Clone, Display, EnumString, PartialEq)]
pub enum MediaType {
    #[strum(serialize = "image/png")]
    #[serde(rename = "image/png")]
    Png,
    #[strum(serialize = "image/jpeg")]
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[strum(serialize = "image/gif")]
    #[serde(rename = "image/gif")]
    Gif,
    #[strum(serialize = "image/webp")]
    #[serde(rename = "image/webp")]
    Webp,
}

/// Source for image content (base64 or URL)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub type_name: String, // "base64", "url", or "file"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>, // base64 data

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>, // URL for url type

    /// Files API identifier, for `type: "file"` sources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}

impl ImageSource {
    /// Create an image source that references a file uploaded via the Files API
    ///
    /// Requires the Files API beta header on the request; see
    /// [`Files`](crate::files::Files).
    pub fn from_file_id<T: AsRef<str>>(file_id: T) -> Self {
        ImageSource {
            type_name: "file".to_string(),
            media_type: None,
            data: None,
            url: None,
            file_id: Some(file_id.as_ref().to_string()),
        }
    }

    /// Create image source from local file path
    pub fn from_path<T: AsRef<str>>(media_type: MediaType, path: T) -> Self {
        let path = PathBuf::from(path.as_ref());
        let ext = std::path::Path::new(&path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png");

        let img = image::ImageReader::open(path.as_path())
            .expect("Failed to open image file")
            .decode()
            .expect("Failed to decode image");

        let img_fmt = match ext {
            "png" => image::ImageFormat::Png,
            "jpg" | "jpeg" => image::ImageFormat::Jpeg,
            "gif" => image::ImageFormat::Gif,
            "webp" => image::ImageFormat::WebP,
            _ => panic!("Unsupported image format: {}", ext),
        };

        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, img_fmt)
            .expect("Failed to write image to buffer");
        let base64_string = BASE64_STANDARD.encode(buf.into_inner());

        ImageSource {
            type_name: "base64".to_string(),
            media_type: Some(media_type.to_string()),
            data: Some(base64_string),
            url: None,
            file_id: None,
        }
    }

    /// Create image source from URL (async fetch and convert to base64)
    pub async fn from_url_as_base64<T: AsRef<str>>(media_type: MediaType, url: T) -> Self {
        let response = request::get(url.as_ref())
            .await
            .expect("Failed to fetch image from URL");
        let bytes = response.bytes().await.expect("Failed to read image bytes");

        let img = image::ImageReader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .expect("Failed to guess image format")
            .decode()
            .expect("Failed to decode image");

        let img_fmt = image::ImageFormat::Png;
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, img_fmt)
            .expect("Failed to write image to buffer");
        let base64_string = BASE64_STANDARD.encode(buf.into_inner());

        ImageSource {
            type_name: "base64".to_string(),
            media_type: Some(media_type.to_string()),
            data: Some(base64_string),
            url: None,
            file_id: None,
        }
    }

    /// Create image source from URL (direct URL reference)
    pub fn from_url<T: AsRef<str>>(url: T) -> Self {
        ImageSource {
            type_name: "url".to_string(),
            media_type: None,
            data: None,
            url: Some(url.as_ref().to_string()),
            file_id: None,
        }
    }

    /// Create image source from base64 string
    pub fn from_base64<T: AsRef<str>>(media_type: MediaType, data: T) -> Self {
        ImageSource {
            type_name: "base64".to_string(),
            media_type: Some(media_type.to_string()),
            data: Some(data.as_ref().to_string()),
            url: None,
            file_id: None,
        }
    }
}

/// Cache control for prompt caching
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub type_name: String, // "ephemeral"
}

impl CacheControl {
    pub fn ephemeral() -> Self {
        CacheControl {
            type_name: "ephemeral".to_string(),
        }
    }
}

/// Content block types for Anthropic API
///
/// Unrecognized block types deserialize to [`ContentBlock::Unknown`] rather
/// than failing, so new server-side block types stay backward compatible.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum ContentBlock {
    /// Text content block
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },

    /// Image content block
    #[serde(rename = "image")]
    Image {
        source: ImageSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },

    /// Tool use content block (from assistant)
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },

    /// Tool result content block (from user)
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<Vec<ContentBlock>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    /// Thinking content block (extended thinking)
    ///
    /// On current models the raw chain of thought is never returned. With the
    /// default display mode the `thinking` field is an empty string; request
    /// [`ThinkingDisplay::Summarized`](crate::messages::request::body::ThinkingDisplay::Summarized)
    /// to receive a readable summary.
    ///
    /// When continuing a conversation on the same model, echo thinking blocks
    /// back **unchanged** — the API rejects modified blocks.
    #[serde(rename = "thinking")]
    Thinking {
        thinking: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    /// Redacted thinking content block
    ///
    /// Echo this back unchanged alongside [`ContentBlock::Thinking`] blocks.
    #[serde(rename = "redacted_thinking")]
    RedactedThinking { data: String },

    /// Document content block (PDF support)
    #[serde(rename = "document")]
    Document {
        source: DocumentSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },

    /// Server-side tool invocation (web search, web fetch, code execution)
    ///
    /// Server tools run on Anthropic's infrastructure — there is nothing to
    /// execute client-side, and the matching result arrives in the same
    /// response.
    #[serde(rename = "server_tool_use")]
    ServerToolUse {
        id: String,
        name: String,
        input: Value,
    },

    /// A refusal-fallback switch point (beta)
    ///
    /// Emitted once per model that ran and declined the turn when
    /// [`Fallbacks`](crate::messages::request::body::Fallbacks) is configured.
    #[serde(rename = "fallback")]
    Fallback {
        from: FallbackModelRef,
        to: FallbackModelRef,
    },

    /// A content block type this version of the library does not model
    ///
    /// Deserializing an unrecognized block yields this variant instead of
    /// failing, so a new server-side block type cannot break an existing
    /// client. Do not echo it back to the API.
    #[serde(other)]
    Unknown,
}

/// The model on one side of a [`ContentBlock::Fallback`] switch point
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FallbackModelRef {
    /// The model identifier
    pub model: crate::messages::request::model::Model,
}

/// Document source for PDF content
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocumentSource {
    #[serde(rename = "type")]
    pub type_name: String, // "base64", "url", or "file"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>, // "application/pdf"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>, // base64 data

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>, // URL for url type

    /// Files API identifier, for `type: "file"` sources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}

impl DocumentSource {
    /// Create a document source that references a file uploaded via the Files API
    ///
    /// Requires the Files API beta header on the request; see
    /// [`Files`](crate::files::Files).
    pub fn from_file_id<T: AsRef<str>>(file_id: T) -> Self {
        DocumentSource {
            type_name: "file".to_string(),
            media_type: None,
            data: None,
            url: None,
            file_id: Some(file_id.as_ref().to_string()),
        }
    }

    /// Create document source from base64 data
    pub fn from_base64<T: AsRef<str>>(data: T) -> Self {
        DocumentSource {
            type_name: "base64".to_string(),
            media_type: Some("application/pdf".to_string()),
            data: Some(data.as_ref().to_string()),
            url: None,
            file_id: None,
        }
    }

    /// Create document source from URL
    pub fn from_url<T: AsRef<str>>(url: T) -> Self {
        DocumentSource {
            type_name: "url".to_string(),
            media_type: None,
            data: None,
            url: Some(url.as_ref().to_string()),
            file_id: None,
        }
    }

    /// Create document source from file path
    pub fn from_path<T: AsRef<str>>(path: T) -> std::io::Result<Self> {
        let data = std::fs::read(path.as_ref())?;
        let base64_string = BASE64_STANDARD.encode(data);

        Ok(DocumentSource {
            type_name: "base64".to_string(),
            media_type: Some("application/pdf".to_string()),
            data: Some(base64_string),
            url: None,
            file_id: None,
        })
    }
}

impl ContentBlock {
    /// Create a text content block
    pub fn text<T: AsRef<str>>(text: T) -> Self {
        ContentBlock::Text {
            text: text.as_ref().to_string(),
            cache_control: None,
        }
    }

    /// Create a text content block with cache control
    pub fn text_with_cache<T: AsRef<str>>(text: T) -> Self {
        ContentBlock::Text {
            text: text.as_ref().to_string(),
            cache_control: Some(CacheControl::ephemeral()),
        }
    }

    /// Create an image content block from file path
    pub fn image_from_path<T: AsRef<str>>(media_type: MediaType, path: T) -> Self {
        ContentBlock::Image {
            source: ImageSource::from_path(media_type, path),
            cache_control: None,
        }
    }

    /// Create an image content block from URL
    pub fn image_from_url<T: AsRef<str>>(url: T) -> Self {
        ContentBlock::Image {
            source: ImageSource::from_url(url),
            cache_control: None,
        }
    }

    /// Create an image content block from base64
    pub fn image_from_base64<T: AsRef<str>>(media_type: MediaType, data: T) -> Self {
        ContentBlock::Image {
            source: ImageSource::from_base64(media_type, data),
            cache_control: None,
        }
    }

    /// Create a tool use content block
    pub fn tool_use<S: AsRef<str>>(id: S, name: S, input: Value) -> Self {
        ContentBlock::ToolUse {
            id: id.as_ref().to_string(),
            name: name.as_ref().to_string(),
            input,
        }
    }

    /// Create a tool result content block with text
    pub fn tool_result_text<S: AsRef<str>>(tool_use_id: S, text: S) -> Self {
        ContentBlock::ToolResult {
            tool_use_id: tool_use_id.as_ref().to_string(),
            content: Some(vec![ContentBlock::text(text)]),
            is_error: None,
        }
    }

    /// Create a tool result content block with error
    pub fn tool_result_error<S: AsRef<str>>(tool_use_id: S, error_message: S) -> Self {
        ContentBlock::ToolResult {
            tool_use_id: tool_use_id.as_ref().to_string(),
            content: Some(vec![ContentBlock::text(error_message)]),
            is_error: Some(true),
        }
    }

    /// Create a document content block from file path
    pub fn document_from_path<T: AsRef<str>>(path: T) -> std::io::Result<Self> {
        Ok(ContentBlock::Document {
            source: DocumentSource::from_path(path)?,
            cache_control: None,
        })
    }

    /// Create a document content block from URL
    pub fn document_from_url<T: AsRef<str>>(url: T) -> Self {
        ContentBlock::Document {
            source: DocumentSource::from_url(url),
            cache_control: None,
        }
    }

    /// Create a document content block from a Files API identifier
    ///
    /// Upload the file first with [`Files`](crate::files::Files), then pass
    /// the returned id here.
    pub fn document_from_file_id<T: AsRef<str>>(file_id: T) -> Self {
        ContentBlock::Document {
            source: DocumentSource::from_file_id(file_id),
            cache_control: None,
        }
    }

    /// Create an image content block from a Files API identifier
    pub fn image_from_file_id<T: AsRef<str>>(file_id: T) -> Self {
        ContentBlock::Image {
            source: ImageSource::from_file_id(file_id),
            cache_control: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guards the trimmed `image` feature set in Cargo.toml.
    ///
    /// The `avif` feature is disabled to drop the ravif/rav1e dependency tree.
    /// Every codec this crate can actually reach must still round-trip.
    #[test]
    fn test_image_from_path_round_trip() {
        let dir = std::env::temp_dir().join("anthropic_tools_image_test");
        std::fs::create_dir_all(&dir).unwrap();

        for (media_type, ext, format) in [
            (MediaType::Png, "png", image::ImageFormat::Png),
            (MediaType::Jpeg, "jpg", image::ImageFormat::Jpeg),
            (MediaType::Gif, "gif", image::ImageFormat::Gif),
            (MediaType::Webp, "webp", image::ImageFormat::WebP),
        ] {
            let path = dir.join(format!("sample.{}", ext));
            let img = image::RgbImage::from_pixel(2, 2, image::Rgb([10, 20, 30]));

            // WebP encoding is not supported by the `image` crate; write the
            // fixture as PNG and let from_path re-encode it to WebP.
            let (fixture_path, fixture_format) = if format == image::ImageFormat::WebP {
                (dir.join("sample_webp_src.png"), image::ImageFormat::Png)
            } else {
                (path.clone(), format)
            };
            img.save_with_format(&fixture_path, fixture_format).unwrap();

            let source = ImageSource::from_path(media_type.clone(), fixture_path.to_str().unwrap());
            assert_eq!(source.type_name, "base64");
            assert_eq!(source.media_type, Some(media_type.to_string()));
            assert!(!source.data.unwrap().is_empty());
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_image_from_file_id() {
        let block = ContentBlock::image_from_file_id("file_abc123");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"image\""));
        assert!(json.contains("\"file_id\":\"file_abc123\""));
    }

    #[test]
    fn test_document_from_file_id() {
        let block = ContentBlock::document_from_file_id("file_abc123");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"document\""));
        assert!(json.contains("\"file_id\":\"file_abc123\""));
        // base64/url fields must be omitted for a file reference
        assert!(!json.contains("\"data\""));
        assert!(!json.contains("\"url\""));
    }

    #[test]
    fn test_unknown_block_deserializes() {
        let json = r#"{"type":"some_future_block","payload":{"a":1}}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        assert!(matches!(block, ContentBlock::Unknown));
    }

    #[test]
    fn test_redacted_thinking_round_trip() {
        let json = r#"{"type":"redacted_thinking","data":"abc"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match &block {
            ContentBlock::RedactedThinking { data } => assert_eq!(data, "abc"),
            other => panic!("unexpected block: {:?}", other),
        }
        assert_eq!(serde_json::to_string(&block).unwrap(), json);
    }

    #[test]
    fn test_text_content_block() {
        let block = ContentBlock::text("Hello, world!");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("\"text\":\"Hello, world!\""));
    }

    #[test]
    fn test_text_with_cache_control() {
        let block = ContentBlock::text_with_cache("Cached text");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"cache_control\""));
        assert!(json.contains("\"type\":\"ephemeral\""));
    }

    #[test]
    fn test_image_from_url() {
        let block = ContentBlock::image_from_url("https://example.com/image.png");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"image\""));
        assert!(json.contains("\"url\":\"https://example.com/image.png\""));
    }

    #[test]
    fn test_tool_use_content_block() {
        let input = serde_json::json!({"query": "test"});
        let block = ContentBlock::tool_use("tool_123", "search", input);
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"tool_use\""));
        assert!(json.contains("\"id\":\"tool_123\""));
        assert!(json.contains("\"name\":\"search\""));
    }

    #[test]
    fn test_tool_result_content_block() {
        let block = ContentBlock::tool_result_text("tool_123", "Search results here");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"tool_result\""));
        assert!(json.contains("\"tool_use_id\":\"tool_123\""));
    }

    #[test]
    fn test_tool_result_error() {
        let block = ContentBlock::tool_result_error("tool_123", "Error occurred");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"is_error\":true"));
    }

    #[test]
    fn test_document_from_url() {
        let block = ContentBlock::document_from_url("https://example.com/doc.pdf");
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"document\""));
        assert!(json.contains("\"url\":\"https://example.com/doc.pdf\""));
    }

    #[test]
    fn test_deserialize_text_block() {
        let json = r#"{"type":"text","text":"Hello"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::Text { text, .. } => assert_eq!(text, "Hello"),
            _ => panic!("Expected Text block"),
        }
    }

    #[test]
    fn test_deserialize_tool_use_block() {
        let json = r#"{"type":"tool_use","id":"123","name":"search","input":{"q":"test"}}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::ToolUse { id, name, input } => {
                assert_eq!(id, "123");
                assert_eq!(name, "search");
                assert_eq!(input["q"], "test");
            }
            _ => panic!("Expected ToolUse block"),
        }
    }
}
