//! Convenient re-exports for common types
//!
//! Use `use rapid_rs::prelude::*;` to get everything you need

pub use crate::{
    app::App,
    error::{ApiError, ApiResult},
    extractors::ValidatedJson,
};

// Re-export commonly used types from dependencies
pub use axum::{
    extract::{Extension, Path, Query, State},
    response::Json,
    routing::{delete, get, patch, post, put},
    Router,
};

pub use serde::{Deserialize, Serialize};
pub use validator::Validate;

pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};

pub use utoipa::ToSchema;
