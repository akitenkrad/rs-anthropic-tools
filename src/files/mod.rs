//! Files API client.
//!
//! Upload a file once and reference it by `file_id` across many Messages API
//! requests, instead of re-encoding it into every request body.
//!
//! - [`Files`] - API client
//! - [`FileMetadata`] - Metadata for a single uploaded file
//! - [`FileList`] - A page of file metadata
//! - [`FileDeleted`] - Deletion confirmation
//!
//! # Beta
//!
//! The Files API is in beta and requires the `files-api-2025-04-14` header.
//! [`Files`] sends it automatically.
//!
//! Not available on Amazon Bedrock or Google Vertex AI.
//!
//! # Limits
//!
//! - Maximum file size: 500 MB
//! - Total storage: 100 GB per organization
//! - Files persist until deleted
//! - Upload, list, and delete are free; content used in messages is billed as
//!   input tokens
//!
//! # Example
//!
//! ```rust,no_run
//! use anthropic_tools::prelude::*;
//!
//! # async fn example() -> Result<()> {
//! let files = Files::new();
//!
//! // Upload once
//! let uploaded = files.upload_path("report.pdf").await?;
//!
//! // Reference it from as many requests as you like
//! let mut client = Messages::new();
//! client
//!     .model(Model::Opus5)
//!     .max_tokens(4096)
//!     .beta(anthropic_tools::files::FILES_API_BETA)
//!     .add_message(Message::new(
//!         Role::User,
//!         vec![
//!             ContentBlock::document_from_file_id(&uploaded.id),
//!             ContentBlock::text("Summarize the key findings."),
//!         ],
//!     ));
//!
//! let response = client.post().await?;
//! println!("{}", response.get_text());
//!
//! files.delete(&uploaded.id).await?;
//! # Ok(())
//! # }
//! ```

use crate::common::errors::{AnthropicToolError, ErrorResponse, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

/// Base URL for the Files API
const FILES_API_URL: &str = "https://api.anthropic.com/v1/files";

/// Current Anthropic API version
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Beta header required by the Files API
///
/// Pass this to [`Messages::beta`](crate::messages::request::Messages::beta)
/// on any request that references an uploaded file.
pub const FILES_API_BETA: &str = "files-api-2025-04-14";

/// Metadata for an uploaded file
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FileMetadata {
    /// File identifier, e.g. `"file_011CNha8iCJcU1wXNR6q4V8w"`
    pub id: String,

    /// Object type (always `"file"`)
    #[serde(rename = "type", default)]
    pub type_name: String,

    /// Original filename
    #[serde(default)]
    pub filename: String,

    /// MIME type of the stored content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Size of the stored content in bytes
    #[serde(default)]
    pub size_bytes: u64,

    /// Creation timestamp (RFC 3339)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// Whether the file's content can be downloaded
    ///
    /// Only files produced by the code execution tool or skills are
    /// downloadable — user-uploaded files are not.
    #[serde(default)]
    pub downloadable: bool,
}

/// A page of file metadata
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FileList {
    /// The files in this page
    pub data: Vec<FileMetadata>,

    /// Whether more pages are available
    #[serde(default)]
    pub has_more: bool,

    /// Cursor for the first item in this page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,

    /// Cursor for the last item in this page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
}

/// Confirmation that a file was deleted
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FileDeleted {
    /// Identifier of the deleted file
    pub id: String,

    /// Object type (always `"file_deleted"`)
    #[serde(rename = "type", default)]
    pub type_name: String,
}

/// Pagination options for [`Files::list_with`]
#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    /// Maximum number of items to return
    pub limit: Option<usize>,

    /// Return items after this cursor
    pub after_id: Option<String>,

    /// Return items before this cursor
    pub before_id: Option<String>,
}

impl ListOptions {
    /// Create empty pagination options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the page size
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the forward cursor
    pub fn after_id<T: AsRef<str>>(mut self, after_id: T) -> Self {
        self.after_id = Some(after_id.as_ref().to_string());
        self
    }

    /// Set the backward cursor
    pub fn before_id<T: AsRef<str>>(mut self, before_id: T) -> Self {
        self.before_id = Some(before_id.as_ref().to_string());
        self
    }
}

/// Files API client
#[derive(Debug, Clone)]
pub struct Files {
    api_key: String,
    betas: Vec<String>,
}

impl Default for Files {
    fn default() -> Self {
        Self::new()
    }
}

impl Files {
    /// Create a new Files client
    ///
    /// Loads the API key from `ANTHROPIC_API_KEY`, falling back to a `.env`
    /// file, exactly like
    /// [`Messages::new`](crate::messages::request::Messages::new).
    pub fn new() -> Self {
        let _ = dotenvy::dotenv();

        let api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        Files {
            api_key,
            betas: vec![FILES_API_BETA.to_string()],
        }
    }

    /// Create a new Files client with an explicit API key
    pub fn with_api_key<T: AsRef<str>>(api_key: T) -> Self {
        Files {
            api_key: api_key.as_ref().to_string(),
            betas: vec![FILES_API_BETA.to_string()],
        }
    }

    /// Add an additional `anthropic-beta` feature flag
    ///
    /// The Files API beta flag is always sent; this adds others alongside it.
    pub fn beta<T: AsRef<str>>(&mut self, beta: T) -> &mut Self {
        let beta = beta.as_ref().to_string();
        if !self.betas.contains(&beta) {
            self.betas.push(beta);
        }
        self
    }

    fn build_headers(&self) -> Result<request::header::HeaderMap> {
        use request::header::{HeaderMap, HeaderValue};

        if self.api_key.is_empty() {
            return Err(AnthropicToolError::ApiKeyNotSet);
        }

        let invalid = |field: &str| {
            AnthropicToolError::InvalidParameter(format!(
                "{} contains invalid header characters",
                field
            ))
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key).map_err(|_| invalid("API key"))?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(ANTHROPIC_VERSION),
        );
        headers.insert(
            "anthropic-beta",
            HeaderValue::from_str(&self.betas.join(",")).map_err(|_| invalid("beta header"))?,
        );

        Ok(headers)
    }

    /// Upload a file from a local path
    ///
    /// The MIME type is inferred from the file extension; use
    /// [`upload_bytes`](Self::upload_bytes) to set it explicitly.
    pub async fn upload_path<P: AsRef<Path>>(&self, path: P) -> Result<FileMetadata> {
        let path = path.as_ref();
        let bytes = std::fs::read(path)?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload")
            .to_string();
        let mime_type = mime_type_for_path(path);

        self.upload_bytes(filename, mime_type, bytes).await
    }

    /// Upload a file from in-memory bytes
    pub async fn upload_bytes<S: AsRef<str>>(
        &self,
        filename: S,
        mime_type: S,
        bytes: Vec<u8>,
    ) -> Result<FileMetadata> {
        let part = request::multipart::Part::bytes(bytes)
            .file_name(filename.as_ref().to_string())
            .mime_str(mime_type.as_ref())
            .map_err(|e| {
                AnthropicToolError::InvalidParameter(format!("invalid MIME type: {}", e))
            })?;

        let form = request::multipart::Form::new().part("file", part);

        let client = request::Client::new();
        let response = client
            .post(FILES_API_URL)
            .headers(self.build_headers()?)
            .multipart(form)
            .send()
            .await?;

        parse_json(response).await
    }

    /// List uploaded files (first page)
    pub async fn list(&self) -> Result<FileList> {
        self.list_with(ListOptions::new()).await
    }

    /// List uploaded files with pagination options
    pub async fn list_with(&self, options: ListOptions) -> Result<FileList> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = options.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(ref after) = options.after_id {
            query.push(("after_id", after.clone()));
        }
        if let Some(ref before) = options.before_id {
            query.push(("before_id", before.clone()));
        }

        let client = request::Client::new();
        let response = client
            .get(FILES_API_URL)
            .headers(self.build_headers()?)
            .query(&query)
            .send()
            .await?;

        parse_json(response).await
    }

    /// Retrieve metadata for a single file
    pub async fn metadata<T: AsRef<str>>(&self, file_id: T) -> Result<FileMetadata> {
        let client = request::Client::new();
        let response = client
            .get(format!("{}/{}", FILES_API_URL, file_id.as_ref()))
            .headers(self.build_headers()?)
            .send()
            .await?;

        parse_json(response).await
    }

    /// Download a file's content
    ///
    /// Only files created by the code execution tool or skills can be
    /// downloaded; user-uploaded files return an error. Check
    /// [`FileMetadata::downloadable`] first.
    pub async fn download<T: AsRef<str>>(&self, file_id: T) -> Result<Vec<u8>> {
        let client = request::Client::new();
        let response = client
            .get(format!("{}/{}/content", FILES_API_URL, file_id.as_ref()))
            .headers(self.build_headers()?)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.bytes().await?.to_vec())
        } else {
            Err(parse_error(response).await)
        }
    }

    /// Download a file's content and write it to a local path
    pub async fn download_to_path<T: AsRef<str>, P: AsRef<Path>>(
        &self,
        file_id: T,
        path: P,
    ) -> Result<()> {
        let bytes = self.download(file_id).await?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Delete a file
    pub async fn delete<T: AsRef<str>>(&self, file_id: T) -> Result<FileDeleted> {
        let client = request::Client::new();
        let response = client
            .delete(format!("{}/{}", FILES_API_URL, file_id.as_ref()))
            .headers(self.build_headers()?)
            .send()
            .await?;

        parse_json(response).await
    }
}

async fn parse_json<T: serde::de::DeserializeOwned>(response: request::Response) -> Result<T> {
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(parse_error(response).await)
    }
}

async fn parse_error(response: request::Response) -> AnthropicToolError {
    let status = response.status();
    match response.json::<ErrorResponse>().await {
        Ok(error_response) => error_response.into_error(),
        Err(_) => AnthropicToolError::ApiError {
            error_type: status.as_str().to_string(),
            message: format!("Files API request failed with status {}", status),
            request_id: None,
        },
    }
}

/// Best-effort MIME type from a file extension
fn mime_type_for_path(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let mime = match ext.as_str() {
        "pdf" => "application/pdf",
        "txt" | "text" => "text/plain",
        "md" | "markdown" => "text/markdown",
        "csv" => "text/csv",
        "json" => "application/json",
        "xml" => "application/xml",
        "html" | "htm" => "text/html",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xls" => "application/vnd.ms-excel",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "py" => "text/x-python",
        "rs" => "text/x-rust",
        "js" | "mjs" => "text/javascript",
        "ts" => "text/x-typescript",
        _ => "application/octet-stream",
    };

    mime.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_files_client_sends_beta_header() {
        let files = Files::with_api_key("test_key");
        let headers = files.build_headers().unwrap();
        assert_eq!(headers.get("anthropic-beta").unwrap(), FILES_API_BETA);
    }

    #[test]
    fn test_files_client_additional_beta() {
        let mut files = Files::with_api_key("test_key");
        files.beta("managed-agents-2026-04-01");

        let headers = files.build_headers().unwrap();
        assert_eq!(
            headers.get("anthropic-beta").unwrap(),
            "files-api-2025-04-14,managed-agents-2026-04-01"
        );
    }

    #[test]
    fn test_files_client_requires_api_key() {
        let files = Files::with_api_key("");
        assert!(matches!(
            files.build_headers(),
            Err(AnthropicToolError::ApiKeyNotSet)
        ));
    }

    #[test]
    fn test_mime_type_for_path() {
        assert_eq!(mime_type_for_path(Path::new("a.pdf")), "application/pdf");
        assert_eq!(mime_type_for_path(Path::new("a.PNG")), "image/png");
        assert_eq!(mime_type_for_path(Path::new("a.csv")), "text/csv");
        assert_eq!(
            mime_type_for_path(Path::new("noext")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_file_metadata_deserialize() {
        let json = r#"{
            "id": "file_011CNha8iCJcU1wXNR6q4V8w",
            "type": "file",
            "filename": "report.pdf",
            "mime_type": "application/pdf",
            "size_bytes": 12345,
            "created_at": "2026-08-15T00:00:00Z",
            "downloadable": false
        }"#;

        let metadata: FileMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.id, "file_011CNha8iCJcU1wXNR6q4V8w");
        assert_eq!(metadata.filename, "report.pdf");
        assert_eq!(metadata.size_bytes, 12345);
        assert!(!metadata.downloadable);
    }

    #[test]
    fn test_file_list_deserialize() {
        let json = r#"{
            "data": [{"id": "file_1", "type": "file", "filename": "a.txt", "size_bytes": 3}],
            "has_more": true,
            "first_id": "file_1",
            "last_id": "file_1"
        }"#;

        let list: FileList = serde_json::from_str(json).unwrap();
        assert_eq!(list.data.len(), 1);
        assert!(list.has_more);
        assert_eq!(list.last_id.as_deref(), Some("file_1"));
    }

    #[test]
    fn test_list_options_builder() {
        let options = ListOptions::new().limit(10).after_id("file_1");
        assert_eq!(options.limit, Some(10));
        assert_eq!(options.after_id.as_deref(), Some("file_1"));
        assert!(options.before_id.is_none());
    }
}
