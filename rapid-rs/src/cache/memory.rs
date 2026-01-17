//! In-memory cache implementation using Moka

use async_trait::async_trait;
use moka::future::Cache as MokaCache;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::{CacheBackend, CacheConfig, CacheStats};
use crate::error::ApiError;

pub struct MemoryCache {
    cache: MokaCache<String, Vec<u8>>,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

impl MemoryCache {
    pub fn new(config: CacheConfig) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(config.max_entries)
            .time_to_live(Duration::from_secs(config.default_ttl_seconds))
            .build();
        
        Self {
            cache,
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }
}

#[async_trait]
impl CacheBackend for MemoryCache {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, ApiError> {
        match self.cache.get(key).await {
            Some(bytes) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                let value = serde_json::from_slice(&bytes)
                    .map_err(|e| ApiError::InternalServerError(
                        format!("Cache deserialization error: {}", e)
                    ))?;
                Ok(Some(value))
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Ok(None)
            }
        }
    }
    
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), ApiError> {
        let bytes = serde_json::to_vec(value)
            .map_err(|e| ApiError::InternalServerError(
                format!("Cache serialization error: {}", e)
            ))?;
        
        self.cache.insert(key.to_string(), bytes).await;
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<(), ApiError> {
        self.cache.invalidate(key).await;
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> Result<bool, ApiError> {
        Ok(self.cache.get(key).await.is_some())
    }
    
    async fn clear(&self) -> Result<(), ApiError> {
        self.cache.invalidate_all();
        Ok(())
    }
    
    async fn stats(&self) -> Result<CacheStats, ApiError> {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };
        
        Ok(CacheStats {
            hits,
            misses,
            entries: self.cache.entry_count(),
            hit_rate,
        })
    }
}