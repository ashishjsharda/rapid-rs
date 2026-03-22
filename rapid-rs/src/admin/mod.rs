//! Admin dashboard for rapid-rs
//!
//! Provides an embedded web-based admin interface with system stats,
//! health monitoring, and management capabilities.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use rapid_rs::admin::{AdminConfig, admin_routes};
//!
//! let config = AdminConfig::new()
//!     .with_secret_key("your-admin-secret-key")
//!     .with_base_path("/admin");
//!
//! App::new()
//!     .auto_configure()
//!     .mount(admin_routes(config))
//!     .run()
//!     .await
//!     .unwrap();
//! ```

pub mod handlers;
pub mod ui;

pub use handlers::{admin_routes, AdminStats, SystemInfo};

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

/// Admin dashboard configuration
#[derive(Debug, Clone)]
pub struct AdminConfig {
    /// Secret key required to access admin dashboard
    pub secret_key: Option<String>,
    /// Base path for admin routes (default: /admin)
    pub base_path: String,
    /// Application name shown in dashboard
    pub app_name: String,
    /// Application version shown in dashboard
    pub app_version: String,
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self {
            secret_key: None,
            base_path: "/admin".to_string(),
            app_name: "rapid-rs App".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl AdminConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_secret_key(mut self, key: impl Into<String>) -> Self {
        self.secret_key = Some(key.into());
        self
    }

    pub fn with_base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = path.into();
        self
    }

    pub fn with_app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = name.into();
        self
    }

    pub fn with_app_version(mut self, version: impl Into<String>) -> Self {
        self.app_version = version.into();
        self
    }
}

/// Global request counter for admin stats
static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);
static ERROR_COUNT: AtomicU64 = AtomicU64::new(0);

static START_TIME: OnceLock<Instant> = OnceLock::new();

fn start_time() -> &'static Instant {
    START_TIME.get_or_init(Instant::now)
}

/// Increment the global request counter
pub fn increment_request_count() {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Increment the global error counter
pub fn increment_error_count() {
    ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Get current request count
pub fn get_request_count() -> u64 {
    REQUEST_COUNT.load(Ordering::Relaxed)
}

/// Get current error count
pub fn get_error_count() -> u64 {
    ERROR_COUNT.load(Ordering::Relaxed)
}

/// Get server uptime in seconds
pub fn get_uptime_seconds() -> u64 {
    start_time().elapsed().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_config_defaults() {
        let config = AdminConfig::default();
        assert_eq!(config.base_path, "/admin");
        assert!(config.secret_key.is_none());
    }

    #[test]
    fn test_admin_config_builder() {
        let config = AdminConfig::new()
            .with_secret_key("my-secret")
            .with_base_path("/management")
            .with_app_name("My API");

        assert_eq!(config.secret_key, Some("my-secret".to_string()));
        assert_eq!(config.base_path, "/management");
        assert_eq!(config.app_name, "My API");
    }

    #[test]
    fn test_counters() {
        let initial = get_request_count();
        increment_request_count();
        assert_eq!(get_request_count(), initial + 1);

        let initial_err = get_error_count();
        increment_error_count();
        assert_eq!(get_error_count(), initial_err + 1);
    }
}
