//! File upload support
//!
//! Provides multipart file upload handling with local and S3 storage backends.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use rapid_rs::uploads::{FileUploadService, UploadConfig, upload_routes};
//! use std::sync::Arc;
//!
//! let config = UploadConfig::new()
//!     .with_max_size(10 * 1024 * 1024)  // 10 MB
//!     .with_upload_dir("./uploads");
//!
//! let service = Arc::new(FileUploadService::new(config));
//!
//! App::new()
//!     .auto_configure()
//!     .mount(upload_routes(service))
//!     .run()
//!     .await
//!     .unwrap();
//! ```

pub mod handler;
pub mod storage;

pub use handler::upload_routes;
pub use storage::{LocalStorage, StorageBackend, UploadStorage};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Upload configuration
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// Maximum file size in bytes (default: 10 MB)
    pub max_file_size: usize,
    /// Allowed MIME types (empty = allow all)
    pub allowed_types: Vec<String>,
    /// Upload directory for local storage
    pub upload_dir: String,
    /// Maximum number of files per request
    pub max_files: usize,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10 MB
            allowed_types: Vec::new(),
            upload_dir: "./uploads".to_string(),
            max_files: 10,
        }
    }
}

impl UploadConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_size(mut self, bytes: usize) -> Self {
        self.max_file_size = bytes;
        self
    }

    pub fn with_allowed_types(mut self, types: Vec<impl Into<String>>) -> Self {
        self.allowed_types = types.into_iter().map(|t| t.into()).collect();
        self
    }

    pub fn with_upload_dir(mut self, dir: impl Into<String>) -> Self {
        self.upload_dir = dir.into();
        self
    }

    pub fn with_max_files(mut self, max: usize) -> Self {
        self.max_files = max;
        self
    }

    /// Check if a MIME type is allowed
    pub fn is_allowed(&self, content_type: &str) -> bool {
        if self.allowed_types.is_empty() {
            return true;
        }
        self.allowed_types.iter().any(|t| t == content_type || t == "*/*")
    }
}

/// Metadata about an uploaded file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    /// Unique file identifier
    pub id: Uuid,
    /// Original filename from the upload
    pub original_name: String,
    /// Stored filename (may differ from original)
    pub stored_name: String,
    /// MIME type
    pub content_type: String,
    /// File size in bytes
    pub size: usize,
    /// Storage URL or path
    pub url: String,
    /// Upload timestamp
    pub uploaded_at: DateTime<Utc>,
}

impl UploadedFile {
    pub fn new(
        original_name: String,
        stored_name: String,
        content_type: String,
        size: usize,
        url: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            original_name,
            stored_name,
            content_type,
            size,
            url,
            uploaded_at: Utc::now(),
        }
    }
}

/// File upload service
pub struct FileUploadService {
    pub config: UploadConfig,
    pub storage: StorageBackend,
}

impl FileUploadService {
    /// Create a new file upload service with local storage
    pub fn new(config: UploadConfig) -> Self {
        let storage = StorageBackend::Local(LocalStorage::new(&config.upload_dir));
        Self { config, storage }
    }

    /// Create a new file upload service with a custom storage backend
    pub fn with_storage(config: UploadConfig, storage: StorageBackend) -> Self {
        Self { config, storage }
    }

    /// Save raw bytes as a file
    pub async fn save(
        &self,
        filename: &str,
        content_type: &str,
        data: &[u8],
    ) -> Result<UploadedFile, crate::error::ApiError> {
        if data.len() > self.config.max_file_size {
            return Err(crate::error::ApiError::BadRequest(format!(
                "File size {} exceeds maximum allowed size {}",
                data.len(),
                self.config.max_file_size
            )));
        }

        if !self.config.is_allowed(content_type) {
            return Err(crate::error::ApiError::BadRequest(format!(
                "Content type '{}' is not allowed",
                content_type
            )));
        }

        self.storage.save(filename, content_type, data).await
    }

    /// Delete a file by stored name
    pub async fn delete(&self, stored_name: &str) -> Result<(), crate::error::ApiError> {
        self.storage.delete(stored_name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_config_defaults() {
        let config = UploadConfig::default();
        assert_eq!(config.max_file_size, 10 * 1024 * 1024);
        assert!(config.allowed_types.is_empty());
        assert_eq!(config.max_files, 10);
    }

    #[test]
    fn test_upload_config_builder() {
        let config = UploadConfig::new()
            .with_max_size(5 * 1024 * 1024)
            .with_allowed_types(vec!["image/jpeg", "image/png"])
            .with_upload_dir("/tmp/uploads");

        assert_eq!(config.max_file_size, 5 * 1024 * 1024);
        assert_eq!(config.allowed_types.len(), 2);
        assert_eq!(config.upload_dir, "/tmp/uploads");
    }

    #[test]
    fn test_is_allowed() {
        let config = UploadConfig::new()
            .with_allowed_types(vec!["image/jpeg", "image/png"]);

        assert!(config.is_allowed("image/jpeg"));
        assert!(config.is_allowed("image/png"));
        assert!(!config.is_allowed("text/plain"));
    }

    #[test]
    fn test_is_allowed_empty_means_all() {
        let config = UploadConfig::new();
        assert!(config.is_allowed("image/jpeg"));
        assert!(config.is_allowed("application/pdf"));
        assert!(config.is_allowed("text/plain"));
    }
}
