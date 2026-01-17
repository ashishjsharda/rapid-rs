//! Tenant context management

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{TenantId, TenantConfig};
use crate::error::ApiError;

/// Tenant information in request context
#[derive(Debug, Clone)]
pub struct TenantInfo {
    pub id: TenantId,
    pub name: String,
    pub features: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl From<TenantConfig> for TenantInfo {
    fn from(config: TenantConfig) -> Self {
        Self {
            id: config.id,
            name: config.name,
            features: config.features,
            metadata: config.metadata,
        }
    }
}

/// Tenant context for request processing
#[derive(Debug, Clone)]
pub struct TenantContext {
    info: TenantInfo,
}

impl TenantContext {
    pub fn new(info: TenantInfo) -> Self {
        Self { info }
    }
    
    pub fn tenant_id(&self) -> &TenantId {
        &self.info.id
    }
    
    pub fn tenant_name(&self) -> &str {
        &self.info.name
    }
    
    pub fn has_feature(&self, feature: &str) -> bool {
        self.info.features.iter().any(|f| f == feature)
    }
    
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.info.metadata.get(key)
    }
}

/// Trait for resolving tenant from request
#[async_trait]
pub trait TenantResolver: Send + Sync {
    /// Resolve tenant ID from subdomain
    async fn resolve_from_subdomain(&self, subdomain: &str) -> Result<TenantId, ApiError>;
    
    /// Resolve tenant ID from header
    async fn resolve_from_header(&self, header_value: &str) -> Result<TenantId, ApiError>;
    
    /// Get tenant configuration
    async fn get_tenant_config(&self, tenant_id: &TenantId) -> Result<TenantConfig, ApiError>;
}

/// In-memory tenant resolver (for development)
pub struct InMemoryTenantResolver {
    tenants: Arc<RwLock<HashMap<TenantId, TenantConfig>>>,
    subdomain_map: Arc<RwLock<HashMap<String, TenantId>>>,
}

impl InMemoryTenantResolver {
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
            subdomain_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add a tenant
    pub async fn add_tenant(&self, config: TenantConfig) -> Result<(), ApiError> {
        let mut tenants = self.tenants.write().await;
        
        // Update subdomain mapping if present
        if let Some(ref subdomain) = config.subdomain {
            let mut subdomain_map = self.subdomain_map.write().await;
            subdomain_map.insert(subdomain.clone(), config.id.clone());
        }
        
        tenants.insert(config.id.clone(), config);
        
        Ok(())
    }
    
    /// Remove a tenant
    pub async fn remove_tenant(&self, tenant_id: &TenantId) -> Result<(), ApiError> {
        let mut tenants = self.tenants.write().await;
        
        if let Some(config) = tenants.remove(tenant_id) {
            // Remove subdomain mapping
            if let Some(subdomain) = config.subdomain {
                let mut subdomain_map = self.subdomain_map.write().await;
                subdomain_map.remove(&subdomain);
            }
        }
        
        Ok(())
    }
    
    /// List all tenants
    pub async fn list_tenants(&self) -> Vec<TenantConfig> {
        let tenants = self.tenants.read().await;
        tenants.values().cloned().collect()
    }
}

impl Default for InMemoryTenantResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TenantResolver for InMemoryTenantResolver {
    async fn resolve_from_subdomain(&self, subdomain: &str) -> Result<TenantId, ApiError> {
        let subdomain_map = self.subdomain_map.read().await;
        
        subdomain_map
            .get(subdomain)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Tenant not found for subdomain: {}", subdomain)))
    }
    
    async fn resolve_from_header(&self, header_value: &str) -> Result<TenantId, ApiError> {
        // Assume header value is the tenant ID
        Ok(TenantId::new(header_value))
    }
    
    async fn get_tenant_config(&self, tenant_id: &TenantId) -> Result<TenantConfig, ApiError> {
        let tenants = self.tenants.read().await;
        
        tenants
            .get(tenant_id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Tenant not found: {}", tenant_id)))
    }
}

/// PostgreSQL tenant resolver
#[cfg(feature = "database")]
pub struct PostgresTenantResolver {
    pool: sqlx::PgPool,
}

#[cfg(feature = "database")]
impl PostgresTenantResolver {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
    
    /// Initialize the tenants table
    pub async fn init(&self) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tenants (
                id VARCHAR(255) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                subdomain VARCHAR(255) UNIQUE,
                database_url TEXT,
                features JSONB NOT NULL DEFAULT '[]',
                metadata JSONB NOT NULL DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                is_active BOOLEAN NOT NULL DEFAULT TRUE
            );
            
            CREATE INDEX IF NOT EXISTS idx_tenants_subdomain ON tenants(subdomain);
            CREATE INDEX IF NOT EXISTS idx_tenants_active ON tenants(is_active);
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}

#[cfg(feature = "database")]
#[async_trait]
impl TenantResolver for PostgresTenantResolver {
    async fn resolve_from_subdomain(&self, subdomain: &str) -> Result<TenantId, ApiError> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT id FROM tenants WHERE subdomain = $1 AND is_active = TRUE"
        )
        .bind(subdomain)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Tenant not found for subdomain: {}", subdomain)))?;
        
        Ok(TenantId::new(row.0))
    }
    
    async fn resolve_from_header(&self, header_value: &str) -> Result<TenantId, ApiError> {
        Ok(TenantId::new(header_value))
    }
    
    async fn get_tenant_config(&self, tenant_id: &TenantId) -> Result<TenantConfig, ApiError> {
        let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, serde_json::Value, serde_json::Value, chrono::DateTime<chrono::Utc>, bool)>(
            "SELECT id, name, subdomain, database_url, features, metadata, created_at, is_active FROM tenants WHERE id = $1"
        )
        .bind(tenant_id.as_str())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Tenant not found: {}", tenant_id)))?;
        
        let features: Vec<String> = serde_json::from_value(row.4)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to parse features: {}", e)))?;
        
        let metadata: HashMap<String, String> = serde_json::from_value(row.5)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to parse metadata: {}", e)))?;
        
        Ok(TenantConfig {
            id: TenantId::new(row.0),
            name: row.1,
            subdomain: row.2,
            database_url: row.3,
            features,
            metadata,
            created_at: row.6,
            is_active: row.7,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_in_memory_resolver() {
        let resolver = InMemoryTenantResolver::new();
        
        let config = TenantConfig::new(
            TenantId::new("tenant-1"),
            "Test Tenant".to_string(),
        )
        .with_subdomain("test".to_string());
        
        resolver.add_tenant(config).await.unwrap();
        
        let tenant_id = resolver
            .resolve_from_subdomain("test")
            .await
            .unwrap();
        
        assert_eq!(tenant_id.as_str(), "tenant-1");
    }
}
