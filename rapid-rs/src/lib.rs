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
//!
//! ## With Authentication
//!
//! ```rust,ignore
//! use rapid_rs::prelude::*;
//! use rapid_rs::auth::{AuthConfig, auth_routes};
//!
//! #[tokio::main]
//! async fn main() {
//!     let auth_config = AuthConfig::from_env();
//!     
//!     App::new()
//!         .auto_configure()
//!         .mount(auth_routes(auth_config))
//!         .run()
//!         .await
//!         .unwrap();
//! }
//! ```
//!
//! ## With Database Migrations
//!
//! ```rust,ignore
//! use rapid_rs::prelude::*;
//! use rapid_rs::database::{connect_and_migrate, MigrationConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let pool = connect_and_migrate(
//!         "postgres://localhost/myapp",
//!         MigrationConfig::default()
//!     ).await.unwrap();
//!     
//!     App::new()
//!         .auto_configure()
//!         .run()
//!         .await
//!         .unwrap();
//! }
//! ```

pub mod app;
pub mod config;
pub mod database;
pub mod error;
pub mod extractors;
pub mod prelude;

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "testing")]
pub mod testing;

pub use app::App;
pub use error::{ApiError, ApiResult};
pub use extractors::ValidatedJson;
