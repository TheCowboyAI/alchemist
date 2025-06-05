//! Content storage service with deduplication and caching

use super::{NatsObjectStore, Result};
use cim_ipld::{TypedContent, Cid};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Cache entry for stored content
#[derive(Clone)]
struct CacheEntry<T> {
    content: T,
    accessed_at: Instant,
    size: usize,
}

/// Content storage service with caching and deduplication
pub struct ContentStorageService {
    /// Underlying object store
    store: Arc<NatsObjectStore>,
    /// In-memory cache
    cache: Arc<RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>>,
    /// Cache metadata
    cache_meta: Arc<RwLock<HashMap<String, (Instant, usize)>>>,
    /// Maximum cache size in bytes
    max_cache_size: usize,
    /// Current cache size
    current_cache_size: Arc<RwLock<usize>>,
    /// Cache TTL
    cache_ttl: Duration,
}

impl ContentStorageService {
    /// Create new content storage service
    pub fn new(store: Arc<NatsObjectStore>) -> Self {
        Self {
            store,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_meta: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size: 100 * 1024 * 1024, // 100MB default
            current_cache_size: Arc::new(RwLock::new(0)),
            cache_ttl: Duration::from_secs(3600), // 1 hour default
        }
    }

    /// Configure cache settings
    pub fn with_cache_config(mut self, max_size: usize, ttl: Duration) -> Self {
        self.max_cache_size = max_size;
        self.cache_ttl = ttl;
        self
    }

    /// Store content with deduplication
    pub async fn store<T>(&self, content: T) -> Result<Cid>
    where
        T: TypedContent + Clone + Send + Sync + 'static,
    {
        let cid = (&content).to_cid()
            .map_err(|e| super::ObjectStoreError::Cid(e.to_string()))?;
        let cid_str = cid.to_string();

        // Check cache first
        {
            let cache = self.cache.read().await;
            if cache.contains_key(&cid_str) {
                debug!("Content found in cache: {}", cid);
                self.update_cache_access(&cid_str).await;
                return Ok(cid);
            }
        }

        // Store in NATS
        let stored_cid = self.store.store(&content).await?;

        // Add to cache
        let size = std::mem::size_of_val(&content);
        self.add_to_cache(cid_str, content, size).await?;

        Ok(stored_cid)
    }

    /// Retrieve content with caching
    pub async fn get<T>(&self, cid: &Cid) -> Result<T>
    where
        T: TypedContent + Clone + Send + Sync + 'static,
    {
        let cid_str = cid.to_string();

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&cid_str) {
                if let Some(content) = entry.downcast_ref::<T>() {
                    debug!("Content retrieved from cache: {}", cid);
                    self.update_cache_access(&cid_str).await;
                    return Ok(content.clone());
                }
            }
        }

        // Retrieve from NATS
        let content: T = self.store.get(cid).await?;

        // Add to cache
        let size = std::mem::size_of_val(&content);
        self.add_to_cache(cid_str, content.clone(), size).await?;

        Ok(content)
    }

    /// Store multiple contents in batch
    pub async fn store_batch<T>(&self, contents: Vec<T>) -> Result<Vec<Cid>>
    where
        T: TypedContent + Clone + Send + Sync + 'static,
    {
        let mut cids = Vec::with_capacity(contents.len());

        for content in contents {
            let cid = self.store(content).await?;
            cids.push(cid);
        }

        Ok(cids)
    }

    /// Retrieve multiple contents in batch
    pub async fn get_batch<T>(&self, cids: &[Cid]) -> Result<Vec<T>>
    where
        T: TypedContent + Clone + Send + Sync + 'static,
    {
        let mut contents = Vec::with_capacity(cids.len());

        for cid in cids {
            let content = self.get(cid).await?;
            contents.push(content);
        }

        Ok(contents)
    }

    /// Check if content exists
    pub async fn exists(&self, cid: &Cid) -> Result<bool> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if cache.contains_key(&cid.to_string()) {
                return Ok(true);
            }
        }

        // Check store
        self.store.exists(cid).await
    }

    /// Delete content
    pub async fn delete(&self, cid: &Cid) -> Result<()> {
        let cid_str = cid.to_string();

        // Remove from cache
        {
            let mut cache = self.cache.write().await;
            if cache.remove(&cid_str).is_some() {
                let mut meta = self.cache_meta.write().await;
                if let Some((_, size)) = meta.remove(&cid_str) {
                    let mut current_size = self.current_cache_size.write().await;
                    *current_size = current_size.saturating_sub(size);
                }
            }
        }

        // Delete from store
        self.store.delete(cid).await
    }

    /// Clear expired cache entries
    pub async fn evict_expired(&self) -> usize {
        let now = Instant::now();
        let mut expired = Vec::new();

        // Find expired entries
        {
            let meta = self.cache_meta.read().await;
            for (cid, (accessed_at, _)) in meta.iter() {
                if now.duration_since(*accessed_at) > self.cache_ttl {
                    expired.push(cid.clone());
                }
            }
        }

        // Remove expired entries
        let count = expired.len();
        for cid in expired {
            self.remove_from_cache(&cid).await;
        }

        if count > 0 {
            info!("Evicted {} expired cache entries", count);
        }

        count
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let current_size = *self.current_cache_size.read().await;

        CacheStats {
            entries: cache.len(),
            size_bytes: current_size,
            max_size_bytes: self.max_cache_size,
            hit_rate: 0.0, // Would need to track hits/misses for this
        }
    }

    /// Add content to cache
    async fn add_to_cache<T>(&self, cid: String, content: T, size: usize) -> Result<()>
    where
        T: Send + Sync + 'static,
    {
        // Check if we need to evict entries
        let mut current_size = self.current_cache_size.write().await;

        // Evict entries if needed
        while *current_size + size > self.max_cache_size {
            // Find oldest entry
            let oldest = {
                let meta = self.cache_meta.read().await;
                meta.iter()
                    .min_by_key(|(_, (accessed_at, _))| accessed_at)
                    .map(|(cid, _)| cid.clone())
            };

            if let Some(cid) = oldest {
                self.remove_from_cache(&cid).await;
                *current_size = *self.current_cache_size.read().await;
            } else {
                break;
            }
        }

        // Add to cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(cid.clone(), Box::new(content));
        }

        // Update metadata
        {
            let mut meta = self.cache_meta.write().await;
            meta.insert(cid, (Instant::now(), size));
        }

        *current_size += size;

        Ok(())
    }

    /// Remove content from cache
    async fn remove_from_cache(&self, cid: &str) {
        let mut cache = self.cache.write().await;
        if cache.remove(cid).is_some() {
            let mut meta = self.cache_meta.write().await;
            if let Some((_, size)) = meta.remove(cid) {
                let mut current_size = self.current_cache_size.write().await;
                *current_size = current_size.saturating_sub(size);
            }
        }
    }

    /// Update cache access time
    async fn update_cache_access(&self, cid: &str) {
        let mut meta = self.cache_meta.write().await;
        if let Some((accessed_at, _size)) = meta.get_mut(cid) {
            *accessed_at = Instant::now();
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub size_bytes: usize,
    pub max_size_bytes: usize,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::content_types::{GraphContent, NodeIPLDContent};
    use async_nats::jetstream;

    async fn create_test_service() -> ContentStorageService {
        // This would need a real NATS connection for integration tests
        // For unit tests, we'd need to mock the NatsObjectStore
        todo!("Implement test setup with mock store")
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        // Test that cache evicts oldest entries when full
        // This would require a mock implementation
    }

    #[tokio::test]
    async fn test_deduplication() {
        // Test that storing the same content twice returns the same CID
        // without storing it twice
    }
}
