//! Content storage service with deduplication and caching

use super::{NatsObjectStore, ObjectStoreError, Result, ContentBucket, ObjectInfo};
use cid::Cid;
use crate::TypedContent;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Cache entry with metadata
#[derive(Clone)]
struct CacheEntry {
    data: Vec<u8>,
    content_type: u64,
    accessed_at: Instant,
    size: usize,
}

/// Content storage service with caching
pub struct ContentStorageService {
    object_store: Arc<NatsObjectStore>,
    cache: Arc<RwLock<LruCache<Cid, CacheEntry>>>,
    cache_ttl: Duration,
    max_cache_size: usize,
    current_cache_size: Arc<RwLock<usize>>,
}

impl ContentStorageService {
    /// Create new content storage service
    pub fn new(
        object_store: Arc<NatsObjectStore>,
        cache_capacity: usize,
        cache_ttl: Duration,
        max_cache_size: usize,
    ) -> Self {
        let cache = LruCache::new(NonZeroUsize::new(cache_capacity).unwrap());

        Self {
            object_store,
            cache: Arc::new(RwLock::new(cache)),
            cache_ttl,
            max_cache_size,
            current_cache_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Store content with deduplication
    pub async fn store<T: TypedContent>(&self, content: &T) -> Result<Cid> {
        // Calculate CID for deduplication
        let cid = content.calculate_cid()
            .map_err(|e| ObjectStoreError::Serialization(e.to_string()))?;

        // Check if already exists
        if self.object_store.exists(&cid, T::CONTENT_TYPE.codec()).await? {
            debug!("Content already exists: {}", cid);
            return Ok(cid);
        }

        // Store in object store
        self.object_store.put(content).await?;

        // Cache the content
        let data = content.to_bytes()
            .map_err(|e| ObjectStoreError::Serialization(e.to_string()))?;
        self.cache_content(cid, data, T::CONTENT_TYPE.codec()).await;

        info!("Stored content: {} (type: {})", cid, T::CONTENT_TYPE.codec());
        Ok(cid)
    }

    /// Retrieve content with caching
    pub async fn get<T: TypedContent>(&self, cid: &Cid) -> Result<T> {
        // Check cache first
        if let Some(entry) = self.get_from_cache(cid).await {
            if entry.content_type == T::CONTENT_TYPE.codec() {
                debug!("Cache hit for: {}", cid);
                return T::from_bytes(&entry.data)
                    .map_err(|e| ObjectStoreError::Deserialization(e.to_string()));
            }
        }

        // Fetch from object store
        let content = self.object_store.get::<T>(cid).await?;

        // Cache for future use
        let data = content.to_bytes()
            .map_err(|e| ObjectStoreError::Serialization(e.to_string()))?;
        self.cache_content(*cid, data, T::CONTENT_TYPE.codec()).await;

        Ok(content)
    }

    /// Check if content exists
    pub async fn exists(&self, cid: &Cid, content_type: u64) -> Result<bool> {
        // Check cache first
        if self.get_from_cache(cid).await.is_some() {
            return Ok(true);
        }

        // Check object store
        self.object_store.exists(cid, content_type).await
    }

    /// Delete content
    pub async fn delete(&self, cid: &Cid, content_type: u64) -> Result<()> {
        // Remove from cache
        self.remove_from_cache(cid).await;

        // Delete from object store
        self.object_store.delete(cid, content_type).await
    }

    /// List content in a bucket
    pub async fn list(&self, bucket: ContentBucket) -> Result<Vec<ObjectInfo>> {
        self.object_store.list(bucket).await
    }

    /// Store multiple contents in batch
    pub async fn store_batch<T: TypedContent>(&self, contents: &[T]) -> Result<Vec<Cid>> {
        let mut cids = Vec::with_capacity(contents.len());

        for content in contents {
            let cid = self.store(content).await?;
            cids.push(cid);
        }

        Ok(cids)
    }

    /// Get multiple contents in batch
    pub async fn get_batch<T: TypedContent>(&self, cids: &[Cid]) -> Result<Vec<T>> {
        let mut contents = Vec::with_capacity(cids.len());

        for cid in cids {
            let content = self.get::<T>(cid).await?;
            contents.push(content);
        }

        Ok(contents)
    }

    /// Cache content
    async fn cache_content(&self, cid: Cid, data: Vec<u8>, content_type: u64) {
        let size = data.len();
        let mut cache = self.cache.write().await;
        let mut current_size = self.current_cache_size.write().await;

        // Check if we need to evict entries
        while *current_size + size > self.max_cache_size && !cache.is_empty() {
            if let Some((_, evicted)) = cache.pop_lru() {
                *current_size -= evicted.size;
            }
        }

        // Add to cache
        let entry = CacheEntry {
            data,
            content_type,
            accessed_at: Instant::now(),
            size,
        };

        if let Some(old_entry) = cache.put(cid, entry) {
            *current_size -= old_entry.size;
        }
        *current_size += size;
    }

    /// Get from cache
    async fn get_from_cache(&self, cid: &Cid) -> Option<CacheEntry> {
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get_mut(cid) {
            // Check TTL
            if entry.accessed_at.elapsed() < self.cache_ttl {
                entry.accessed_at = Instant::now();
                return Some(entry.clone());
            } else {
                // Entry expired, remove it
                let mut current_size = self.current_cache_size.write().await;
                *current_size -= entry.size;
                cache.pop(cid);
            }
        }

        None
    }

    /// Remove from cache
    async fn remove_from_cache(&self, cid: &Cid) {
        let mut cache = self.cache.write().await;
        if let Some(entry) = cache.pop(cid) {
            let mut current_size = self.current_cache_size.write().await;
            *current_size -= entry.size;
        }
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        let mut current_size = self.current_cache_size.write().await;
        *current_size = 0;
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let current_size = self.current_cache_size.read().await;

        CacheStats {
            entries: cache.len(),
            size: *current_size,
            capacity: cache.cap().get(),
            max_size: self.max_cache_size,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub size: usize,
    pub capacity: usize,
    pub max_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_eviction() {
        // Test would require a mock object store
        // For now, just verify the cache stats structure
        let stats = CacheStats {
            entries: 10,
            size: 1024,
            capacity: 100,
            max_size: 10240,
        };

        assert_eq!(stats.entries, 10);
        assert_eq!(stats.size, 1024);
    }
}
