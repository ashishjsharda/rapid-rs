//! Axum handlers for file uploads

use axum::{
    extract::{Multipart, State},
    routing::post,
    Json, Router,
};
use std::sync::Arc;
use crate::error::ApiError;
use super::{FileUploadService, UploadedFile};

/// Upload multiple files via multipart form
pub async fn upload_handler(
    State(service): State<Arc<FileUploadService>>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadedFile>>, ApiError> {
    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart.next_field().await
        .map_err(|e| ApiError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let filename = field.file_name()
            .unwrap_or("unnamed")
            .to_string();

        let content_type = field.content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        if uploaded_files.len() >= service.config.max_files {
            return Err(ApiError::BadRequest(format!(
                "Too many files. Maximum allowed: {}",
                service.config.max_files
            )));
        }

        let data = field.bytes().await
            .map_err(|e| ApiError::BadRequest(format!("Failed to read field: {}", e)))?;

        let uploaded = service.save(&filename, &content_type, &data).await?;
        uploaded_files.push(uploaded);
    }

    if uploaded_files.is_empty() {
        return Err(ApiError::BadRequest("No files provided".to_string()));
    }

    Ok(Json(uploaded_files))
}

/// Create upload routes
///
/// Mounts:
/// - POST /upload - Upload one or more files
pub fn upload_routes(service: Arc<FileUploadService>) -> Router {
    Router::new()
        .route("/upload", post(upload_handler))
        .with_state(service)
}
