//! # rapid-rs
//!
//! Zero-config, batteries-included web framework for Rust.
//! FastAPI meets Spring Boot, powered by Axum.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rapid_rs::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     App::new()
//!         .auto_configure()
//!         .run()
//!         .await
//!         .unwrap();
//! }
//! ```

pub mod app;
pub mod config;
pub mod error;
pub mod extractors;
pub mod prelude;

pub use app::App;
pub use error::{ApiError, ApiResult};
pub use extractors::ValidatedJson;
