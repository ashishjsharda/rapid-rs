//! Multi-tenancy support
//!
//! Provides tenant isolation, context management, and middleware for building
//! multi-tenant SaaS applications.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use rapid_rs::multi_tenancy::*;
//! use axum::middleware;
//! use std::sync::Arc;
//!
//! async fn my_handler(TenantExtractor(tenant): TenantExtractor) -> String {
//!     format!("Hello from tenant: {}", tenant.tenant_name())
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     // Setup tenant resolver
//!     let resolver = InMemoryTenantResolver::new();
//!     
//!     // Add a tenant
//!     let tenant = TenantConfig::new(
//!         TenantId::new("acme"),
//!         "Acme Corp".to_string()
//!     ).with_subdomain("acme".to_string());
//!     
//!     resolver.add_tenant(tenant).await.unwrap();
//!     
//!     // Setup middleware
//!     let config = Arc::new(TenantMiddlewareConfig::new(resolver));
//!     
//!     let app = Router::new()
//!         .route("/", get(my_handler))
//!         .layer(middleware::from_fn_with_state(
//!             config,
//!             tenant_middleware
//!         ));
//!     
//!     App::new()
//!         .auto_configure()
//!         .mount(app)
//!         .run()
//!         .await
//!         .unwrap();
//! }
//! ```

pub mod context;
pub mod middleware;

pub use context::{TenantContext, TenantInfo, TenantResolver, InMemoryTenantResolver};
pub use middleware::{tenant_middleware, TenantExtractor, TenantMiddlewareConfig};

#[cfg(feature = "database")]
pub use context::PostgresTenantResolver;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tenant identifier
///
/// Uniquely identifies a tenant in the system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);

impl TenantId {
    /// Create a new tenant ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the tenant ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for TenantId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for TenantId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<Uuid> for TenantId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid.to_string())
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tenant configuration
///
/// Contains all the configuration and metadata for a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    /// Unique tenant identifier
    pub id: TenantId,
    
    /// Display name for the tenant
    pub name: String,
    
    /// Subdomain for this tenant (e.g., "acme" for acme.example.com)
    pub subdomain: Option<String>,
    
    /// Optional dedicated database URL for this tenant
    pub database_url: Option<String>,
    
    /// Enabled features for this tenant
    pub features: Vec<String>,
    
    /// Custom metadata key-value pairs
    pub metadata: std::collections::HashMap<String, String>,
    
    /// When the tenant was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Whether the tenant is currently active
    pub is_active: bool,
}

impl TenantConfig {
    /// Create a new tenant configuration
    pub fn new(id: TenantId, name: String) -> Self {
        Self {
            id,
            name,
            subdomain: None,
            database_url: None,
            features: Vec::new(),
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
            is_active: true,
        }
    }
    
    /// Set the subdomain for this tenant
    pub fn with_subdomain(mut self, subdomain: String) -> Self {
        self.subdomain = Some(subdomain);
        self
    }
    
    /// Set a dedicated database URL for this tenant
    pub fn with_database(mut self, url: String) -> Self {
        self.database_url = Some(url);
        self
    }
    
    /// Set the enabled features for this tenant
    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }
    
    /// Add a metadata key-value pair
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Check if a feature is enabled for this tenant
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }
    
    /// Set whether the tenant is active
    pub fn set_active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }
}

/// Tenant isolation strategy
///
/// Defines how data is isolated between tenants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationStrategy {
    /// Each tenant has its own separate database
    Database,
    
    /// All tenants share a database with tenant_id column for filtering
    Schema,
    
    /// Hybrid approach combining database and schema isolation
    Hybrid,
}

impl Default for IsolationStrategy {
    fn default() -> Self {
        Self::Schema
    }
}

/// Tenant plan/tier for subscription management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantPlan {
    /// Free tier with limited features
    Free,
    
    /// Basic paid plan
    Basic,
    
    /// Professional plan with more features
    Professional,
    
    /// Enterprise plan with all features
    Enterprise,
    
    /// Custom plan
    Custom,
}

impl Default for TenantPlan {
    fn default() -> Self {
        Self::Free
    }
}

/// Tenant limits for resource management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantLimits {
    /// Maximum number of users
    pub max_users: Option<u32>,
    
    /// Maximum API requests per hour
    pub max_api_requests_per_hour: Option<u32>,
    
    /// Maximum storage in bytes
    pub max_storage_bytes: Option<u64>,
    
    /// Maximum number of projects/workspaces
    pub max_projects: Option<u32>,
}

impl Default for TenantLimits {
    fn default() -> Self {
        Self {
            max_users: Some(10),
            max_api_requests_per_hour: Some(1000),
            max_storage_bytes: Some(1_073_741_824), // 1 GB
            max_projects: Some(5),
        }
    }
}

impl TenantLimits {
    /// Create unlimited tenant limits
    pub fn unlimited() -> Self {
        Self {
            max_users: None,
            max_api_requests_per_hour: None,
            max_storage_bytes: None,
            max_projects: None,
        }
    }
    
    /// Create limits for a specific plan
    pub fn for_plan(plan: TenantPlan) -> Self {
        match plan {
            TenantPlan::Free => Self::default(),
            TenantPlan::Basic => Self {
                max_users: Some(25),
                max_api_requests_per_hour: Some(5000),
                max_storage_bytes: Some(5_368_709_120), // 5 GB
                max_projects: Some(10),
            },
            TenantPlan::Professional => Self {
                max_users: Some(100),
                max_api_requests_per_hour: Some(25000),
                max_storage_bytes: Some(53_687_091_200), // 50 GB
                max_projects: Some(50),
            },
            TenantPlan::Enterprise | TenantPlan::Custom => Self::unlimited(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tenant_id_creation() {
        let id = TenantId::new("tenant-123");
        assert_eq!(id.as_str(), "tenant-123");
        assert_eq!(id.to_string(), "tenant-123");
        
        let id_from_string = TenantId::from("tenant-456".to_string());
        assert_eq!(id_from_string.as_str(), "tenant-456");
        
        let id_from_uuid = TenantId::from(Uuid::new_v4());
        assert!(!id_from_uuid.as_str().is_empty());
    }
    
    #[test]
    fn test_tenant_config() {
        let config = TenantConfig::new(
            TenantId::new("tenant-1"),
            "Acme Corp".to_string(),
        )
        .with_subdomain("acme".to_string())
        .with_features(vec!["premium".to_string(), "api".to_string()])
        .with_metadata("industry".to_string(), "technology".to_string());
        
        assert_eq!(config.name, "Acme Corp");
        assert_eq!(config.subdomain, Some("acme".to_string()));
        assert!(config.has_feature("premium"));
        assert!(config.has_feature("api"));
        assert!(!config.has_feature("enterprise"));
        assert_eq!(config.metadata.get("industry"), Some(&"technology".to_string()));
        assert!(config.is_active);
    }
    
    #[test]
    fn test_isolation_strategy() {
        let strategy = IsolationStrategy::default();
        assert_eq!(strategy, IsolationStrategy::Schema);
        
        let db_strategy = IsolationStrategy::Database;
        assert_ne!(db_strategy, IsolationStrategy::Schema);
    }
    
    #[test]
    fn test_tenant_limits() {
        let limits = TenantLimits::default();
        assert_eq!(limits.max_users, Some(10));
        assert_eq!(limits.max_api_requests_per_hour, Some(1000));
        
        let unlimited = TenantLimits::unlimited();
        assert_eq!(unlimited.max_users, None);
        assert_eq!(unlimited.max_api_requests_per_hour, None);
        
        let professional = TenantLimits::for_plan(TenantPlan::Professional);
        assert_eq!(professional.max_users, Some(100));
        assert_eq!(professional.max_api_requests_per_hour, Some(25000));
    }
    
    #[test]
    fn test_tenant_plan() {
        let plan = TenantPlan::default();
        assert_eq!(plan, TenantPlan::Free);
        
        let limits = TenantLimits::for_plan(TenantPlan::Enterprise);
        assert_eq!(limits.max_users, None); // Unlimited
    }
}