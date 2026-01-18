//! Caching layer with multiple backends

pub mod memory;

#[cfg(feature = "cache-redis")]
pub mod redis;

use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use crate::error::ApiError;

pub use memory::MemoryCache;

#[cfg(feature = "cache-redis")]
pub use redis::RedisCache;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub default_ttl_seconds: u64,
    pub max_entries: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl_seconds: 300,
            max_entries: 10_000,
        }
    }
}

impl CacheConfig {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_default_ttl(mut self, seconds: u64) -> Self {
        self.default_ttl_seconds = seconds;
        self
    }
    
    pub fn with_max_entries(mut self, max: u64) -> Self {
        self.max_entries = max;
        self
    }
}

/// Cache statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: u64,
    pub hit_rate: f64,
}

impl CacheStats {
    pub fn efficiency_score(&self) -> f64 {
        self.hit_rate * 100.0
    }
    
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }
}

/// Cache backend enum to avoid dyn trait issues
pub enum CacheBackend {
    Memory(MemoryCache),
    #[cfg(feature = "cache-redis")]
    Redis(RedisCache),
}

impl CacheBackend {
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, ApiError> {
        match self {
            CacheBackend::Memory(cache) => cache.get(key).await,
            #[cfg(feature = "cache-redis")]
            CacheBackend::Redis(cache) => cache.get(key).await,
        }
    }
    
    pub async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), ApiError> {
        match self {
            CacheBackend::Memory(cache) => cache.set(key, value, ttl).await,
            #[cfg(feature = "cache-redis")]
            CacheBackend::Redis(cache) => cache.set(key, value, ttl).await,
        }
    }
    
    pub async fn delete(&self, key: &str) -> Result<(), ApiError> {
        match self {
            CacheBackend::Memory(cache) => cache.delete(key).await,
            #[cfg(feature = "cache-redis")]
            CacheBackend::Redis(cache) => cache.delete(key).await,
        }
    }
    
    pub async fn exists(&self, key: &str) -> Result<bool, ApiError> {
        match self {
            CacheBackend::Memory(cache) => cache.exists(key).await,
            #[cfg(feature = "cache-redis")]
            CacheBackend::Redis(cache) => cache.exists(key).await,
        }
    }
    
    pub async fn clear(&self) -> Result<(), ApiError> {
        match self {
            CacheBackend::Memory(cache) => cache.clear().await,
            #[cfg(feature = "cache-redis")]
            CacheBackend::Redis(cache) => cache.clear().await,
        }
    }
    
    pub async fn stats(&self) -> Result<CacheStats, ApiError> {
        match self {
            CacheBackend::Memory(cache) => cache.stats().await,
            #[cfg(feature = "cache-redis")]
            CacheBackend::Redis(cache) => cache.stats().await,
        }
    }
}

/// Main cache interface
pub struct Cache {
    backend: CacheBackend,
}

impl Cache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            backend: CacheBackend::Memory(MemoryCache::new(config)),
        }
    }
    
    pub fn with_memory(config: CacheConfig) -> Self {
        Self {
            backend: CacheBackend::Memory(MemoryCache::new(config)),
        }
    }
    
    #[cfg(feature = "cache-redis")]
    pub async fn with_redis(redis_url: &str, config: CacheConfig) -> Result<Self, ApiError> {
        Ok(Self {
            backend: CacheBackend::Redis(RedisCache::new(redis_url, config).await?),
        })
    }
    
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, ApiError> {
        self.backend.get(key).await
    }
    
    pub async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), ApiError> {
        self.backend.set(key, value, ttl).await
    }
    
    pub async fn delete(&self, key: &str) -> Result<(), ApiError> {
        self.backend.delete(key).await
    }
    
    pub async fn exists(&self, key: &str) -> Result<bool, ApiError> {
        self.backend.exists(key).await
    }
    
    pub async fn clear(&self) -> Result<(), ApiError> {
        self.backend.clear().await
    }
    
    pub async fn stats(&self) -> Result<CacheStats, ApiError> {
        self.backend.stats().await
    }
    
    pub async fn get_or_compute<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        compute: F,
    ) -> Result<T, ApiError>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, ApiError>>,
    {
        if let Some(value) = self.get(key).await? {
            return Ok(value);
        }
        
        let value = compute().await?;
        self.set(key, &value, ttl).await?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cache_config() {
        let config = CacheConfig::new()
            .with_default_ttl(600)
            .with_max_entries(50000);
        
        assert_eq!(config.default_ttl_seconds, 600);
        assert_eq!(config.max_entries, 50000);
    }
    
    #[tokio::test]
    async fn test_cache_operations() {
        let cache = Cache::new(CacheConfig::default());
        
        cache.set("test_key", &"test_value", Duration::from_secs(60))
            .await
            .unwrap();
        
        let value: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));
        
        assert!(cache.exists("test_key").await.unwrap());
        
        cache.delete("test_key").await.unwrap();
        
        let value: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(value, None);
    }
    
    #[tokio::test]
    async fn test_cache_stats() {
        let cache = Cache::new(CacheConfig::default());
        
        cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
        
        let _: Option<String> = cache.get("key1").await.unwrap(); // Hit
        let _: Option<String> = cache.get("key2").await.unwrap(); // Miss
        
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_requests(), 2);
    }
}