//! Feature flags provider

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::ApiError;

/// Feature flag configuration
#[derive(Debug, Clone)]
pub struct FlagConfig {
    /// Environment (development, staging, production)
    pub environment: String,
}

impl Default for FlagConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
        }
    }
}

/// Context for feature flag evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagContext {
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub attributes: HashMap<String, String>,
}

impl FlagContext {
    pub fn new() -> Self {
        Self {
            user_id: None,
            email: None,
            attributes: HashMap::new(),
        }
    }
    
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }
    
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }
}

impl Default for FlagContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Feature flag result
#[derive(Debug, Clone, Serialize)]
pub struct FlagResult {
    pub enabled: bool,
    pub variant: Option<String>,
    pub reason: String,
}

/// Trait for feature flag providers
#[async_trait]
pub trait FlagProvider: Send + Sync {
    /// Check if a feature is enabled
    async fn is_enabled(
        &self,
        flag_key: &str,
        context: Option<&FlagContext>,
    ) -> Result<bool, ApiError>;
    
    /// Get flag value with variant
    async fn get_variant(
        &self,
        flag_key: &str,
        context: Option<&FlagContext>,
    ) -> Result<FlagResult, ApiError>;
    
    /// Get all flags for context
    async fn get_all_flags(
        &self,
        context: Option<&FlagContext>,
    ) -> Result<HashMap<String, bool>, ApiError>;
}

/// In-memory feature flags (for development)
pub struct InMemoryFlagProvider {
    flags: Arc<RwLock<HashMap<String, FlagDefinition>>>,
}

#[derive(Debug, Clone)]
struct FlagDefinition {
    enabled: bool,
    variant: Option<String>,
    targeting: Option<FlagTargeting>,
}

#[derive(Debug, Clone)]
struct FlagTargeting {
    user_ids: Vec<String>,
    attributes: HashMap<String, Vec<String>>,
}

impl InMemoryFlagProvider {
    pub fn new() -> Self {
        Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Set a flag
    pub async fn set_flag(&self, key: String, enabled: bool) {
        let mut flags = self.flags.write().await;
        flags.insert(
            key,
            FlagDefinition {
                enabled,
                variant: None,
                targeting: None,
            },
        );
    }
    
    /// Set a flag with variant
    pub async fn set_flag_with_variant(&self, key: String, enabled: bool, variant: String) {
        let mut flags = self.flags.write().await;
        flags.insert(
            key,
            FlagDefinition {
                enabled,
                variant: Some(variant),
                targeting: None,
            },
        );
    }
    
    /// Set flag targeting
    pub async fn set_targeting(
        &self,
        key: String,
        user_ids: Vec<String>,
        attributes: HashMap<String, Vec<String>>,
    ) {
        let mut flags = self.flags.write().await;
        if let Some(flag) = flags.get_mut(&key) {
            flag.targeting = Some(FlagTargeting {
                user_ids,
                attributes,
            });
        }
    }
    
    /// Remove a flag
    pub async fn remove_flag(&self, key: &str) {
        let mut flags = self.flags.write().await;
        flags.remove(key);
    }
    
    /// Clear all flags
    pub async fn clear_all(&self) {
        let mut flags = self.flags.write().await;
        flags.clear();
    }
}

impl Default for InMemoryFlagProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FlagProvider for InMemoryFlagProvider {
    async fn is_enabled(
        &self,
        flag_key: &str,
        context: Option<&FlagContext>,
    ) -> Result<bool, ApiError> {
        let flags = self.flags.read().await;
        
        if let Some(flag) = flags.get(flag_key) {
            // Check targeting rules if present
            if let Some(targeting) = &flag.targeting {
                if let Some(ctx) = context {
                    // Check user ID targeting
                    if let Some(user_id) = &ctx.user_id {
                        if targeting.user_ids.contains(user_id) {
                            return Ok(true);
                        }
                    }
                    
                    // Check attribute targeting
                    for (key, values) in &targeting.attributes {
                        if let Some(user_value) = ctx.attributes.get(key) {
                            if values.contains(user_value) {
                                return Ok(true);
                            }
                        }
                    }
                    
                    // If targeting is set but didn't match, flag is disabled for this user
                    return Ok(false);
                }
            }
            
            Ok(flag.enabled)
        } else {
            // Flag not found, default to disabled
            Ok(false)
        }
    }
    
    async fn get_variant(
        &self,
        flag_key: &str,
        context: Option<&FlagContext>,
    ) -> Result<FlagResult, ApiError> {
        let enabled = self.is_enabled(flag_key, context).await?;
        
        let flags = self.flags.read().await;
        let variant = flags
            .get(flag_key)
            .and_then(|f| f.variant.clone());
        
        Ok(FlagResult {
            enabled,
            variant,
            reason: if enabled {
                "Flag is enabled".to_string()
            } else {
                "Flag is disabled".to_string()
            },
        })
    }
    
    async fn get_all_flags(
        &self,
        context: Option<&FlagContext>,
    ) -> Result<HashMap<String, bool>, ApiError> {
        let flags = self.flags.read().await;
        let mut result = HashMap::new();
        
        for (key, _) in flags.iter() {
            let enabled = self.is_enabled(key, context).await?;
            result.insert(key.clone(), enabled);
        }
        
        Ok(result)
    }
}

/// Main feature flags interface
pub struct FeatureFlags {
    provider: Box<dyn FlagProvider>,
    config: FlagConfig,
}

impl FeatureFlags {
    /// Create with in-memory provider
    pub fn new(config: FlagConfig) -> Self {
        Self {
            provider: Box::new(InMemoryFlagProvider::new()),
            config,
        }
    }
    
    /// Create with custom provider
    pub fn with_provider(provider: impl FlagProvider + 'static, config: FlagConfig) -> Self {
        Self {
            provider: Box::new(provider),
            config,
        }
    }
    
    /// Check if feature is enabled
    pub async fn is_enabled(
        &self,
        flag_key: &str,
        context: Option<&FlagContext>,
    ) -> Result<bool, ApiError> {
        self.provider.is_enabled(flag_key, context).await
    }
    
    /// Get flag with variant
    pub async fn get_variant(
        &self,
        flag_key: &str,
        context: Option<&FlagContext>,
    ) -> Result<FlagResult, ApiError> {
        self.provider.get_variant(flag_key, context).await
    }
    
    /// Get all flags
    pub async fn get_all_flags(
        &self,
        context: Option<&FlagContext>,
    ) -> Result<HashMap<String, bool>, ApiError> {
        self.provider.get_all_flags(context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_feature_flags() {
        let provider = InMemoryFlagProvider::new();
        provider.set_flag("new_ui".to_string(), true).await;
        provider.set_flag("beta_feature".to_string(), false).await;
        
        let flags = FeatureFlags::with_provider(provider, FlagConfig::default());
        
        assert!(flags.is_enabled("new_ui", None).await.unwrap());
        assert!(!flags.is_enabled("beta_feature", None).await.unwrap());
        assert!(!flags.is_enabled("unknown_flag", None).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_flag_targeting() {
        let provider = InMemoryFlagProvider::new();
        provider.set_flag("premium_feature".to_string(), false).await;
        
        provider
            .set_targeting(
                "premium_feature".to_string(),
                vec!["user-123".to_string()],
                HashMap::new(),
            )
            .await;
        
        let flags = FeatureFlags::with_provider(provider, FlagConfig::default());
        
        let context = FlagContext::new().with_user("user-123".to_string());
        assert!(flags.is_enabled("premium_feature", Some(&context)).await.unwrap());
        
        let other_context = FlagContext::new().with_user("user-456".to_string());
        assert!(!flags.is_enabled("premium_feature", Some(&other_context)).await.unwrap());
    }
}
