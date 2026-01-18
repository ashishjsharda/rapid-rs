//! # rapid-rs v0.4.0 - Phase 3 Complete
//!
//! Zero-config, batteries-included web framework for Rust.
//! FastAPI meets Spring Boot, powered by Axum.

pub mod app;
pub mod config;
pub mod database;
pub mod error;
pub mod extractors;
pub mod prelude;

// Phase 2 features
#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "testing")]
pub mod testing;

// Phase 3 features
#[cfg(feature = "jobs")]
pub mod jobs;

#[cfg(feature = "websocket")]
pub mod websocket;

#[cfg(feature = "cache")]
pub mod cache;

#[cfg(feature = "rate-limit")]
pub mod rate_limit;

#[cfg(feature = "observability")]
pub mod metrics;

#[cfg(feature = "feature-flags")]
pub mod feature_flags;

#[cfg(feature = "multi-tenancy")]
pub mod multi_tenancy;

pub use app::App;
pub use error::{ApiError, ApiResult};
pub use extractors::ValidatedJson;