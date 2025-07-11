//! Caching layer for Alchemist
//! 
//! Provides Redis-based caching with fallback to in-memory cache

use anyhow::{Result, Context};
use async_trait::async_trait;
use redis::{Client as RedisClient, AsyncCommands, RedisError};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, warn, error};

/// Cache trait for different cache implementations
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get a value from cache
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    
    /// Set a value in cache with TTL
    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<()>;
    
    /// Delete a value from cache
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// Clear all cache entries
    async fn clear(&self) -> Result<()>;
    
    /// Check if key exists
    async fn exists(&self, key: &str) -> bool;
    
    /// Delete entries matching pattern (default: not implemented)
    async fn delete_pattern(&self, _pattern: &str) -> Result<()> {
        Err(anyhow::anyhow!("Pattern deletion not supported"))
    }
    
    /// Get as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Redis cache implementation
pub struct RedisCache {
    client: RedisClient,
    key_prefix: String,
}

impl RedisCache {
    pub fn new(redis_url: &str, key_prefix: &str) -> Result<Self> {
        let client = RedisClient::open(redis_url)
            .context("Failed to create Redis client")?;
        
        Ok(Self {
            client,
            key_prefix: key_prefix.to_string(),
        })
    }
    
    fn make_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut conn = match self.client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                warn!("Redis connection error: {}", e);
                return None;
            }
        };
        
        let full_key = self.make_key(key);
        let data: Vec<u8> = match conn.get(&full_key).await {
            Ok(data) => data,
            Err(e) => {
                if !matches!(e.kind(), redis::ErrorKind::TypeError) {
                    debug!("Redis get error for key {}: {}", full_key, e);
                }
                return None;
            }
        };
        
        match bincode::deserialize(&data) {
            Ok(value) => {
                debug!("Cache hit for key: {}", full_key);
                Some(value)
            }
            Err(e) => {
                error!("Deserialization error for key {}: {}", full_key, e);
                None
            }
        }
    }
    
    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<()> {
        let mut conn = self.client.get_async_connection().await
            .context("Failed to get Redis connection")?;
        
        let full_key = self.make_key(key);
        let data = bincode::serialize(value)
            .context("Failed to serialize value")?;
        
        conn.set_ex(&full_key, data, ttl.as_secs() as usize).await
            .context("Failed to set value in Redis")?;
        
        debug!("Cache set for key: {} (TTL: {:?})", full_key, ttl);
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let full_key = self.make_key(key);
        
        conn.del(&full_key).await?;
        debug!("Cache delete for key: {}", full_key);
        Ok(())
    }
    
    async fn clear(&self) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let pattern = format!("{}:*", self.key_prefix);
        
        let keys: Vec<String> = conn.keys(&pattern).await?;
        if !keys.is_empty() {
            conn.del(keys).await?;
            debug!("Cleared {} cache entries", keys.len());
        }
        
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> bool {
        if let Ok(mut conn) = self.client.get_async_connection().await {
            let full_key = self.make_key(key);
            conn.exists(&full_key).await.unwrap_or(false)
        } else {
            false
        }
    }
    
    async fn delete_pattern(&self, pattern: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let full_pattern = self.make_key(pattern);
        
        // Use SCAN to find matching keys (safer than KEYS for production)
        let mut cursor = 0u64;
        let mut deleted_count = 0;
        
        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&full_pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await?;
            
            if !keys.is_empty() {
                conn.del(&keys).await?;
                deleted_count += keys.len();
            }
            
            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }
        
        debug!("Deleted {} cache entries matching pattern: {}", deleted_count, pattern);
        Ok(())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// In-memory cache entry
#[derive(Clone)]
struct MemoryCacheEntry {
    data: Vec<u8>,
    expires_at: Instant,
}

/// In-memory cache implementation (fallback)
pub struct MemoryCache {
    entries: Arc<RwLock<HashMap<String, MemoryCacheEntry>>>,
    max_size: usize,
}

impl MemoryCache {
    pub fn new(max_size: usize) -> Self {
        let cache = Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        };
        
        // Start cleanup task
        let entries = cache.entries.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                
                let now = Instant::now();
                let mut cache = entries.write().await;
                cache.retain(|_, entry| entry.expires_at > now);
            }
        });
        
        cache
    }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let cache = self.entries.read().await;
        
        if let Some(entry) = cache.get(key) {
            if entry.expires_at > Instant::now() {
                match bincode::deserialize(&entry.data) {
                    Ok(value) => {
                        debug!("Memory cache hit for key: {}", key);
                        return Some(value);
                    }
                    Err(e) => {
                        error!("Deserialization error for key {}: {}", key, e);
                    }
                }
            }
        }
        
        None
    }
    
    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<()> {
        let data = bincode::serialize(value)?;
        let entry = MemoryCacheEntry {
            data,
            expires_at: Instant::now() + ttl,
        };
        
        let mut cache = self.entries.write().await;
        
        // Evict old entries if at capacity
        if cache.len() >= self.max_size {
            // Simple LRU: remove oldest expired entry
            let now = Instant::now();
            cache.retain(|_, entry| entry.expires_at > now);
            
            // If still full, remove oldest entry
            if cache.len() >= self.max_size {
                if let Some(oldest_key) = cache.iter()
                    .min_by_key(|(_, entry)| entry.expires_at)
                    .map(|(k, _)| k.clone()) {
                    cache.remove(&oldest_key);
                }
            }
        }
        
        cache.insert(key.to_string(), entry);
        debug!("Memory cache set for key: {} (TTL: {:?})", key, ttl);
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        let mut cache = self.entries.write().await;
        cache.remove(key);
        Ok(())
    }
    
    async fn clear(&self) -> Result<()> {
        let mut cache = self.entries.write().await;
        let count = cache.len();
        cache.clear();
        debug!("Cleared {} memory cache entries", count);
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> bool {
        let cache = self.entries.read().await;
        cache.get(key)
            .map(|entry| entry.expires_at > Instant::now())
            .unwrap_or(false)
    }
    
    async fn delete_pattern(&self, pattern: &str) -> Result<()> {
        let mut cache = self.entries.write().await;
        let now = Instant::now();
        
        // Convert pattern to regex-like matching
        let regex_pattern = pattern
            .replace("*", ".*")
            .replace("?", ".");
        
        let mut deleted_count = 0;
        
        // Simple pattern matching
        if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            cache.retain(|key, entry| {
                if entry.expires_at <= now || re.is_match(key) {
                    deleted_count += 1;
                    false
                } else {
                    true
                }
            });
        } else {
            // If regex fails, do simple prefix matching
            cache.retain(|key, entry| {
                if entry.expires_at <= now || key.starts_with(pattern.trim_end_matches('*')) {
                    deleted_count += 1;
                    false
                } else {
                    true
                }
            });
        }
        
        debug!("Deleted {} memory cache entries matching pattern: {}", deleted_count, pattern);
        Ok(())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Layered cache with Redis primary and memory fallback
pub struct LayeredCache {
    primary: Option<Arc<dyn Cache>>,
    fallback: Arc<dyn Cache>,
}

impl LayeredCache {
    pub fn new(redis_url: Option<&str>, key_prefix: &str) -> Self {
        let primary = redis_url.and_then(|url| {
            match RedisCache::new(url, key_prefix) {
                Ok(cache) => Some(Arc::new(cache) as Arc<dyn Cache>),
                Err(e) => {
                    warn!("Failed to create Redis cache, using memory only: {}", e);
                    None
                }
            }
        });
        
        let fallback = Arc::new(MemoryCache::new(10000)) as Arc<dyn Cache>;
        
        Self { primary, fallback }
    }
}

#[async_trait]
impl Cache for LayeredCache {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        // Try primary cache first
        if let Some(cache) = &self.primary {
            if let Some(value) = cache.get(key).await {
                return Some(value);
            }
        }
        
        // Fall back to memory cache
        self.fallback.get(key).await
    }
    
    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<()> {
        // Set in both caches
        if let Some(cache) = &self.primary {
            if let Err(e) = cache.set(key, value, ttl).await {
                warn!("Failed to set in primary cache: {}", e);
            }
        }
        
        self.fallback.set(key, value, ttl).await
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        // Delete from both caches
        if let Some(cache) = &self.primary {
            let _ = cache.delete(key).await;
        }
        
        self.fallback.delete(key).await
    }
    
    async fn clear(&self) -> Result<()> {
        // Clear both caches
        if let Some(cache) = &self.primary {
            let _ = cache.clear().await;
        }
        
        self.fallback.clear().await
    }
    
    async fn exists(&self, key: &str) -> bool {
        if let Some(cache) = &self.primary {
            if cache.exists(key).await {
                return true;
            }
        }
        
        self.fallback.exists(key).await
    }
    
    async fn delete_pattern(&self, pattern: &str) -> Result<()> {
        // Delete from both caches
        let mut errors = Vec::new();
        
        if let Some(cache) = &self.primary {
            if let Err(e) = cache.delete_pattern(pattern).await {
                errors.push(format!("Primary cache: {}", e));
            }
        }
        
        if let Err(e) = self.fallback.delete_pattern(pattern).await {
            errors.push(format!("Fallback cache: {}", e));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Pattern deletion errors: {}", errors.join("; ")))
        }
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Cache manager with typed operations
pub struct CacheManager {
    cache: Arc<dyn Cache>,
}

impl CacheManager {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }
    
    /// Get or compute a value
    pub async fn get_or_compute<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        compute: F,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Clone,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Try cache first
        if let Some(value) = self.cache.get(key).await {
            return Ok(value);
        }
        
        // Compute value
        let value = compute().await?;
        
        // Store in cache
        if let Err(e) = self.cache.set(key, &value, ttl).await {
            warn!("Failed to cache computed value: {}", e);
        }
        
        Ok(value)
    }
    
    /// Invalidate cache entries by pattern
    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<()> {
        // Pattern matching for cache keys
        // Supports wildcards: * (any characters) and ? (single character)
        
        self.cache.delete_pattern(pattern).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_cache() {
        let cache = MemoryCache::new(100);
        
        // Test set and get
        cache.set("test_key", &"test_value", Duration::from_secs(60)).await.unwrap();
        let value: String = cache.get("test_key").await.unwrap();
        assert_eq!(value, "test_value");
        
        // Test expiration
        cache.set("expire_key", &"expire_value", Duration::from_millis(100)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        let value: Option<String> = cache.get("expire_key").await;
        assert!(value.is_none());
        
        // Test delete
        cache.set("delete_key", &"delete_value", Duration::from_secs(60)).await.unwrap();
        cache.delete("delete_key").await.unwrap();
        let value: Option<String> = cache.get("delete_key").await;
        assert!(value.is_none());
    }
    
    #[tokio::test]
    async fn test_cache_manager() {
        let cache = Arc::new(MemoryCache::new(100));
        let manager = CacheManager::new(cache);
        
        // Test get_or_compute
        let value = manager.get_or_compute(
            "computed_key",
            Duration::from_secs(60),
            || async { Ok("computed_value".to_string()) }
        ).await.unwrap();
        
        assert_eq!(value, "computed_value");
        
        // Second call should use cache
        let value2 = manager.get_or_compute(
            "computed_key",
            Duration::from_secs(60),
            || async { Ok("should_not_compute".to_string()) }
        ).await.unwrap();
        
        assert_eq!(value2, "computed_value");
    }
}