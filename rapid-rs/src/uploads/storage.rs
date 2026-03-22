//! File storage backends

use std::path::Path;
use tokio::fs;
use uuid::Uuid;
use crate::error::ApiError;
use super::UploadedFile;

/// Storage trait for file backends
#[async_trait::async_trait]
pub trait UploadStorage: Send + Sync {
    async fn save(&self, filename: &str, content_type: &str, data: &[u8]) -> Result<UploadedFile, ApiError>;
    async fn delete(&self, stored_name: &str) -> Result<(), ApiError>;
    async fn url(&self, stored_name: &str) -> String;
}

/// Storage backend enum
pub enum StorageBackend {
    Local(LocalStorage),
}

impl StorageBackend {
    pub async fn save(&self, filename: &str, content_type: &str, data: &[u8]) -> Result<UploadedFile, ApiError> {
        match self {
            StorageBackend::Local(s) => s.save(filename, content_type, data).await,
        }
    }

    pub async fn delete(&self, stored_name: &str) -> Result<(), ApiError> {
        match self {
            StorageBackend::Local(s) => s.delete(stored_name).await,
        }
    }
}

/// Local filesystem storage
pub struct LocalStorage {
    base_dir: String,
    base_url: String,
}

impl LocalStorage {
    pub fn new(base_dir: impl Into<String>) -> Self {
        let base_dir = base_dir.into();
        Self {
            base_url: "/uploads".to_string(),
            base_dir,
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    fn extension_from_mime(content_type: &str) -> &str {
        match content_type {
            "image/jpeg" => ".jpg",
            "image/png" => ".png",
            "image/gif" => ".gif",
            "image/webp" => ".webp",
            "application/pdf" => ".pdf",
            "text/plain" => ".txt",
            "text/csv" => ".csv",
            "application/json" => ".json",
            "application/zip" => ".zip",
            "video/mp4" => ".mp4",
            _ => "",
        }
    }
}

#[async_trait::async_trait]
impl UploadStorage for LocalStorage {
    async fn save(&self, filename: &str, content_type: &str, data: &[u8]) -> Result<UploadedFile, ApiError> {
        // Create upload dir if it doesn't exist
        fs::create_dir_all(&self.base_dir).await
            .map_err(|e| ApiError::InternalServerError(format!("Failed to create upload dir: {}", e)))?;

        // Generate unique stored filename
        let ext = Self::extension_from_mime(content_type);
        let stored_name = format!("{}{}", Uuid::new_v4(), ext);
        let file_path = Path::new(&self.base_dir).join(&stored_name);

        // Write file
        fs::write(&file_path, data).await
            .map_err(|e| ApiError::InternalServerError(format!("Failed to write file: {}", e)))?;

        let url = format!("{}/{}", self.base_url, stored_name);

        Ok(UploadedFile::new(
            filename.to_string(),
            stored_name,
            content_type.to_string(),
            data.len(),
            url,
        ))
    }

    async fn delete(&self, stored_name: &str) -> Result<(), ApiError> {
        let file_path = Path::new(&self.base_dir).join(stored_name);
        if file_path.exists() {
            fs::remove_file(&file_path).await
                .map_err(|e| ApiError::InternalServerError(format!("Failed to delete file: {}", e)))?;
        }
        Ok(())
    }

    async fn url(&self, stored_name: &str) -> String {
        format!("{}/{}", self.base_url, stored_name)
    }
}
