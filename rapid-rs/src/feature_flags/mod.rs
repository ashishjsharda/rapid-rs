//! Feature flags system
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use rapid_rs::feature_flags::{FeatureFlags, FlagConfig};
//!
//! let flags = FeatureFlags::new(FlagConfig::default());
//!
//! if flags.is_enabled("new_feature", None).await? {
//!     // New feature code
//! }
//! ```

pub mod provider;

pub use provider::{FeatureFlags, FlagConfig, FlagContext, FlagProvider, InMemoryFlagProvider};

use serde::Serialize;
use std::collections::HashMap;

/// Feature flag evaluation result
#[derive(Debug, Clone, Serialize)]
pub struct FlagResult {
    pub enabled: bool,
    pub variant: Option<String>,
    pub reason: String,
}

/// User context for feature flag evaluation
#[derive(Debug, Clone, Serialize)]
pub struct UserContext {
    pub user_id: String,
    pub email: Option<String>,
    pub attributes: HashMap<String, String>,
}
