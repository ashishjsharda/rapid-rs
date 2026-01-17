//! Caching layer with multiple backends

pub mod memory;

#[cfg(feature = "cache-redis")]
pub mod redis;

use async_trait::async_trait;
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

/// Cache trait for different backends
#[async_trait]
pub trait CacheBackend: Send + Sync {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, ApiError>;
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), ApiError>;
    async fn delete(&self, key: &str) -> Result<(), ApiError>;
    async fn exists(&self, key: &str) -> Result<bool, ApiError>;
    async fn clear(&self) -> Result<(), ApiError>;
    async fn stats(&self) -> Result<CacheStats, ApiError>;
}

/// Cache statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: u64,
    pub hit_rate: f64,
}

/// Main cache interface
pub struct Cache {
    backend: Box<dyn CacheBackend>,
}

impl Cache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            backend: Box::new(MemoryCache::new(config)),
        }
    }
    
    pub fn with_backend(backend: impl CacheBackend + 'static) -> Self {
        Self {
            backend: Box::new(backend),
        }
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
}