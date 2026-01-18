//! Redis cache backend implementation

#[cfg(feature = "cache-redis")]
use redis::AsyncCommands;
#[cfg(feature = "cache-redis")]
use serde::{de::DeserializeOwned, Serialize};
#[cfg(feature = "cache-redis")]
use std::sync::Arc;
#[cfg(feature = "cache-redis")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "cache-redis")]
use std::time::Duration;

#[cfg(feature = "cache-redis")]
use super::{CacheConfig, CacheStats};
#[cfg(feature = "cache-redis")]
use crate::error::ApiError;

/// Redis cache backend
#[cfg(feature = "cache-redis")]
pub struct RedisCache {
    client: redis::Client,
    connection_manager: Arc<tokio::sync::Mutex<redis::aio::ConnectionManager>>,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

#[cfg(feature = "cache-redis")]
impl RedisCache {
    pub async fn new(redis_url: &str, _config: CacheConfig) -> Result<Self, ApiError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to create Redis client: {}", e)))?;
        
        let connection_manager = redis::aio::ConnectionManager::new(client.clone())
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Failed to connect to Redis: {}", e)))?;
        
        Ok(Self {
            client,
            connection_manager: Arc::new(tokio::sync::Mutex::new(connection_manager)),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        })
    }
    
    async fn get_connection(&self) -> redis::aio::ConnectionManager {
        self.connection_manager.lock().await.clone()
    }
    
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, ApiError> {
        let mut conn = self.get_connection().await;
        
        match conn.get::<_, Option<Vec<u8>>>(key).await {
            Ok(Some(bytes)) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                
                let value = serde_json::from_slice(&bytes)
                    .map_err(|e| ApiError::InternalServerError(
                        format!("Cache deserialization error: {}", e)
                    ))?;
                
                Ok(Some(value))
            }
            Ok(None) => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Ok(None)
            }
            Err(e) => Err(ApiError::InternalServerError(
                format!("Redis get error: {}", e)
            )),
        }
    }
    
    pub async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), ApiError> {
        let bytes = serde_json::to_vec(value)
            .map_err(|e| ApiError::InternalServerError(
                format!("Cache serialization error: {}", e)
            ))?;
        
        let mut conn = self.get_connection().await;
        
        // Fix: u64 not usize, and add type annotation
        conn.set_ex::<_, _, ()>(key, bytes, ttl.as_secs())
            .await
            .map_err(|e| ApiError::InternalServerError(
                format!("Redis set error: {}", e)
            ))?;
        
        Ok(())
    }
    
    pub async fn delete(&self, key: &str) -> Result<(), ApiError> {
        let mut conn = self.get_connection().await;
        
        conn.del::<_, ()>(key)
            .await
            .map_err(|e| ApiError::InternalServerError(
                format!("Redis delete error: {}", e)
            ))?;
        
        Ok(())
    }
    
    pub async fn exists(&self, key: &str) -> Result<bool, ApiError> {
        let mut conn = self.get_connection().await;
        
        conn.exists(key)
            .await
            .map_err(|e| ApiError::InternalServerError(
                format!("Redis exists error: {}", e)
            ))
    }
    
    pub async fn clear(&self) -> Result<(), ApiError> {
        let mut conn = self.get_connection().await;
        
        redis::cmd("FLUSHDB")
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| ApiError::InternalServerError(
                format!("Redis clear error: {}", e)
            ))?;
        
        Ok(())
    }
    
    pub async fn stats(&self) -> Result<CacheStats, ApiError> {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };
        
        let mut conn = self.get_connection().await;
        
        let entries: u64 = redis::cmd("DBSIZE")
            .query_async(&mut conn)
            .await
            .unwrap_or(0);
        
        Ok(CacheStats {
            hits,
            misses,
            entries,
            hit_rate,
        })
    }
}

#[cfg(test)]
#[cfg(feature = "cache-redis")]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore]
    async fn test_redis_cache() {
        let cache = RedisCache::new("redis://127.0.0.1/", CacheConfig::default())
            .await
            .unwrap();
        
        cache.set("test_key", &"test_value", Duration::from_secs(60))
            .await
            .unwrap();
        
        let value: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));
        
        cache.delete("test_key").await.unwrap();
        
        let value: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(value, None);
    }
}