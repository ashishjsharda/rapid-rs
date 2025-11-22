//! Authentication and Authorization module for rapid-rs
//!
//! Provides JWT-based authentication, password hashing, and route protection.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use rapid_rs::prelude::*;
//! use rapid_rs::auth::{AuthConfig, AuthUser, login, register};
//!
//! #[tokio::main]
//! async fn main() {
//!     App::new()
//!         .auto_configure()
//!         .with_auth(AuthConfig::default())
//!         .mount(auth_routes())
//!         .run()
//!         .await
//!         .unwrap();
//! }
//! ```

pub mod config;
pub mod jwt;
pub mod password;
pub mod extractors;
pub mod middleware;
pub mod handlers;
pub mod models;

pub use config::AuthConfig;
pub use jwt::{TokenPair, Claims, create_token_pair, verify_token};
pub use password::{hash_password, verify_password};
pub use extractors::AuthUser;
pub use middleware::RequireAuth;
pub use handlers::{auth_routes, login, register, refresh_token, logout, UserStore, StoredUser, CreateUserData, InMemoryUserStore, auth_routes_with_store, AuthAppState};
pub use models::{LoginRequest, RegisterRequest, AuthResponse, TokenRefreshRequest};
