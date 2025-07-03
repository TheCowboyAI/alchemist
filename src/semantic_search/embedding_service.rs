//! Embedding service for converting text to vector representations
//! 
//! This service provides abstraction over different embedding models
//! and handles text preprocessing, batching, and caching.

use super::{Result, SemanticSearchError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Supported embedding models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmbeddingModel {
    /// OpenAI's text-embedding-ada-002
    OpenAIAda002,
    /// OpenAI's text-embedding-3-small
    OpenAI3Small,
    /// OpenAI's text-embedding-3-large
    OpenAI3Large,
    /// Sentence Transformers all-MiniLM-L6-v2
    AllMiniLMV2,
    /// Sentence Transformers all-mpnet-base-v2
    AllMpnetBaseV2,
    /// Custom model
    Custom(u32), // Model ID for custom models
}

impl EmbeddingModel {
    /// Get the dimension of embeddings produced by this model
    pub fn dimension(&self) -> usize {
        match self {
            Self::OpenAIAda002 => 1536,
            Self::OpenAI3Small => 1536,
            Self::OpenAI3Large => 3072,
            Self::AllMiniLMV2 => 384,
            Self::AllMpnetBaseV2 => 768,
            Self::Custom(_) => 512, // Default for custom models
        }
    }
    
    /// Get the maximum token limit for this model
    pub fn max_tokens(&self) -> usize {
        match self {
            Self::OpenAIAda002 => 8191,
            Self::OpenAI3Small => 8191,
            Self::OpenAI3Large => 8191,
            Self::AllMiniLMV2 => 512,
            Self::AllMpnetBaseV2 => 512,
            Self::Custom(_) => 512,
        }
    }
}

/// Configuration for the embedding service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model: EmbeddingModel,
    pub api_key: Option<String>,
    pub api_url: Option<String>,
    pub batch_size: usize,
    pub cache_size: usize,
    pub timeout_seconds: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model: EmbeddingModel::AllMiniLMV2,
            api_key: None,
            api_url: None,
            batch_size: 100,
            cache_size: 10000,
            timeout_seconds: 30,
        }
    }
}

/// Trait for embedding providers
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embeddings for a batch of texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    
    /// Get the model being used
    fn model(&self) -> EmbeddingModel;
}

/// Mock embedding provider for testing
pub struct MockEmbeddingProvider {
    model: EmbeddingModel,
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let dim = self.model.dimension();
        Ok(texts.iter()
            .map(|text| {
                // Simple mock: use text length as seed for reproducible embeddings
                let seed = text.len() as f32;
                (0..dim)
                    .map(|i| ((seed + i as f32).sin() + 1.0) / 2.0)
                    .collect()
            })
            .collect())
    }
    
    fn model(&self) -> EmbeddingModel {
        self.model
    }
}

/// Cache entry for embeddings
#[derive(Clone)]
struct CacheEntry {
    embedding: Vec<f32>,
    access_count: usize,
    last_accessed: std::time::Instant,
}

/// Main embedding service
pub struct EmbeddingService {
    provider: Arc<dyn EmbeddingProvider>,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    config: EmbeddingConfig,
}

impl EmbeddingService {
    /// Create a new embedding service
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        let provider: Arc<dyn EmbeddingProvider> = match config.model {
            EmbeddingModel::OpenAIAda002 |
            EmbeddingModel::OpenAI3Small |
            EmbeddingModel::OpenAI3Large => {
                // In production, would create OpenAI provider
                Arc::new(MockEmbeddingProvider { model: config.model })
            }
            _ => Arc::new(MockEmbeddingProvider { model: config.model }),
        };
        
        Ok(Self {
            provider,
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }
    
    /// Generate embedding for a single text
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(entry) = cache.get_mut(text) {
                entry.access_count += 1;
                entry.last_accessed = std::time::Instant::now();
                return Ok(entry.embedding.clone());
            }
        }
        
        // Generate embedding
        let embeddings = self.provider.embed_batch(&[text.to_string()]).await?;
        let embedding = embeddings.into_iter().next()
            .ok_or_else(|| SemanticSearchError::EmbeddingError(
                "No embedding returned".to_string()
            ))?;
        
        // Cache the result
        self.cache_embedding(text.to_string(), embedding.clone()).await;
        
        Ok(embedding)
    }
    
    /// Generate embeddings for multiple texts
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        let mut uncached_texts = Vec::new();
        let mut uncached_indices = Vec::new();
        
        // Check cache for each text
        {
            let mut cache = self.cache.write().await;
            for (i, text) in texts.iter().enumerate() {
                if let Some(entry) = cache.get_mut(text) {
                    entry.access_count += 1;
                    entry.last_accessed = std::time::Instant::now();
                    results.push(Some(entry.embedding.clone()));
                } else {
                    results.push(None);
                    uncached_texts.push(text.clone());
                    uncached_indices.push(i);
                }
            }
        }
        
        // Generate embeddings for uncached texts
        if !uncached_texts.is_empty() {
            let new_embeddings = self.provider.embed_batch(&uncached_texts).await?;
            
            // Cache and fill results
            for (idx, (text, embedding)) in uncached_texts.into_iter()
                .zip(new_embeddings.into_iter())
                .enumerate()
            {
                self.cache_embedding(text, embedding.clone()).await;
                results[uncached_indices[idx]] = Some(embedding);
            }
        }
        
        // Unwrap all results
        results.into_iter()
            .map(|opt| opt.ok_or_else(|| {
                SemanticSearchError::EmbeddingError("Missing embedding".to_string())
            }))
            .collect()
    }
    
    /// Cache an embedding
    async fn cache_embedding(&self, text: String, embedding: Vec<f32>) {
        let mut cache = self.cache.write().await;
        
        // Evict old entries if cache is full
        if cache.len() >= self.config.cache_size {
            // Find least recently used entry
            if let Some((key, _)) = cache.iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(k, v)| (k.clone(), v.clone()))
            {
                cache.remove(&key);
            }
        }
        
        cache.insert(text, CacheEntry {
            embedding,
            access_count: 1,
            last_accessed: std::time::Instant::now(),
        });
    }
    
    /// Clear the embedding cache
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }
    
    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let total_accesses: usize = cache.values()
            .map(|entry| entry.access_count)
            .sum();
        
        CacheStats {
            size: cache.len(),
            capacity: self.config.cache_size,
            total_accesses,
            hit_rate: if total_accesses > 0 {
                (total_accesses - cache.len()) as f32 / total_accesses as f32
            } else {
                0.0
            },
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
    pub total_accesses: usize,
    pub hit_rate: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_embedding_service() {
        let config = EmbeddingConfig::default();
        let service = EmbeddingService::new(config).unwrap();
        
        // Test single embedding
        let text = "Hello, world!";
        let embedding = service.embed(text).await.unwrap();
        assert_eq!(embedding.len(), 384); // AllMiniLMV2 dimension
        
        // Test cache hit
        let embedding2 = service.embed(text).await.unwrap();
        assert_eq!(embedding, embedding2);
        
        // Check cache stats
        let stats = service.cache_stats().await;
        assert_eq!(stats.size, 1);
        assert_eq!(stats.total_accesses, 2);
    }
    
    #[tokio::test]
    async fn test_batch_embedding() {
        let config = EmbeddingConfig::default();
        let service = EmbeddingService::new(config).unwrap();
        
        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];
        
        let embeddings = service.embed_batch(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 3);
        assert!(embeddings.iter().all(|e| e.len() == 384));
    }
} 