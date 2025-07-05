//! Vector store for efficient similarity search
//! 
//! This module provides abstraction over vector databases for storing
//! and searching embeddings with support for filtering and metadata.

use super::{DocumentId, DocumentMetadata, RelevanceScore, Result, SemanticSearchError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    pub store_type: VectorStoreType,
    pub connection_string: Option<String>,
    pub collection_name: String,
    pub dimension: usize,
    pub metric: DistanceMetric,
    pub index_type: IndexType,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            store_type: VectorStoreType::InMemory,
            connection_string: None,
            collection_name: "cim_embeddings".to_string(),
            dimension: 384,
            metric: DistanceMetric::Cosine,
            index_type: IndexType::Flat,
        }
    }
}

/// Types of vector stores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorStoreType {
    InMemory,
    Qdrant,
    Weaviate,
    Pinecone,
    Milvus,
}

/// Distance metrics for similarity search
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

/// Index types for vector search
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexType {
    Flat,
    IVF,
    HNSW,
    Annoy,
}

/// Search result from vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: DocumentId,
    pub score: RelevanceScore,
    pub metadata: DocumentMetadata,
}

/// Filter for vector search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SearchFilter {
    pub domain: Option<String>,
    pub entity_type: Option<String>,
    pub tags: Option<Vec<String>>,
    pub time_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    pub custom_filters: HashMap<String, serde_json::Value>,
}


/// Trait for vector store implementations
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Initialize the vector store
    async fn initialize(&self) -> Result<()>;
    
    /// Insert a vector with metadata
    async fn insert(
        &self,
        id: DocumentId,
        embedding: Vec<f32>,
        metadata: DocumentMetadata,
    ) -> Result<()>;
    
    /// Insert multiple vectors
    async fn insert_batch(
        &self,
        items: Vec<(DocumentId, Vec<f32>, DocumentMetadata)>,
    ) -> Result<()>;
    
    /// Search for similar vectors
    async fn search(
        &self,
        query_vector: &[f32],
        k: usize,
        filter: Option<SearchFilter>,
    ) -> Result<Vec<SearchResult>>;
    
    /// Delete a vector by ID
    async fn delete(&self, id: DocumentId) -> Result<()>;
    
    /// Update metadata for a vector
    async fn update_metadata(&self, id: DocumentId, metadata: DocumentMetadata) -> Result<()>;
    
    /// Get total count of vectors
    async fn count(&self) -> Result<usize>;
    
    /// Clear all vectors
    async fn clear(&self) -> Result<()>;
}

/// In-memory vector store implementation
pub struct InMemoryVectorStore {
    data: Arc<RwLock<HashMap<DocumentId, (Vec<f32>, DocumentMetadata)>>>,
    config: VectorStoreConfig,
}

impl InMemoryVectorStore {
    #[must_use] pub fn new(config: VectorStoreConfig) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    fn calculate_similarity(&self, v1: &[f32], v2: &[f32]) -> f32 {
        match self.config.metric {
            DistanceMetric::Cosine => {
                let dot_product: f32 = v1.iter().zip(v2.iter())
                    .map(|(a, b)| a * b)
                    .sum();
                let norm1: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm2: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();
                
                if norm1 == 0.0 || norm2 == 0.0 {
                    0.0
                } else {
                    dot_product / (norm1 * norm2)
                }
            }
            DistanceMetric::Euclidean => {
                let sum_sq: f32 = v1.iter().zip(v2.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum();
                1.0 / (1.0 + sum_sq.sqrt())
            }
            DistanceMetric::DotProduct => {
                v1.iter().zip(v2.iter())
                    .map(|(a, b)| a * b)
                    .sum()
            }
        }
    }
    
    fn matches_filter(&self, metadata: &DocumentMetadata, filter: &SearchFilter) -> bool {
        // Check domain filter
        if let Some(ref domain) = filter.domain {
            if metadata.domain != *domain {
                return false;
            }
        }
        
        // Check entity type filter
        if let Some(ref entity_type) = filter.entity_type {
            if metadata.entity_type != *entity_type {
                return false;
            }
        }
        
        // Check tags filter
        if let Some(ref tags) = filter.tags {
            if !tags.iter().any(|tag| metadata.tags.contains(tag)) {
                return false;
            }
        }
        
        // Check time range filter
        if let Some((start, end)) = filter.time_range {
            if metadata.timestamp < start || metadata.timestamp > end {
                return false;
            }
        }
        
        // Check custom filters
        for (key, value) in &filter.custom_filters {
            if let Some(metadata_value) = metadata.custom_fields.get(key) {
                if metadata_value != value {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    
    async fn insert(
        &self,
        id: DocumentId,
        embedding: Vec<f32>,
        metadata: DocumentMetadata,
    ) -> Result<()> {
        if embedding.len() != self.config.dimension {
            return Err(SemanticSearchError::VectorStoreError(
                format!("Expected dimension {}, got {}", self.config.dimension, embedding.len())
            ));
        }
        
        self.data.write().await.insert(id, (embedding, metadata));
        Ok(())
    }
    
    async fn insert_batch(
        &self,
        items: Vec<(DocumentId, Vec<f32>, DocumentMetadata)>,
    ) -> Result<()> {
        let mut data = self.data.write().await;
        for (id, embedding, metadata) in items {
            if embedding.len() != self.config.dimension {
                return Err(SemanticSearchError::VectorStoreError(
                    format!("Expected dimension {}, got {}", self.config.dimension, embedding.len())
                ));
            }
            data.insert(id, (embedding, metadata));
        }
        Ok(())
    }
    
    async fn search(
        &self,
        query_vector: &[f32],
        k: usize,
        filter: Option<SearchFilter>,
    ) -> Result<Vec<SearchResult>> {
        if query_vector.len() != self.config.dimension {
            return Err(SemanticSearchError::VectorStoreError(
                format!("Expected dimension {}, got {}", self.config.dimension, query_vector.len())
            ));
        }
        
        let data = self.data.read().await;
        let mut results: Vec<(DocumentId, f32, DocumentMetadata)> = data.iter()
            .filter(|(_, (_, metadata))| {
                filter.as_ref().is_none_or(|f| self.matches_filter(metadata, f))
            })
            .map(|(id, (embedding, metadata))| {
                let similarity = self.calculate_similarity(query_vector, embedding);
                (*id, similarity, metadata.clone())
            })
            .collect();
        
        // Sort by similarity (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top k results
        results.truncate(k);
        
        Ok(results.into_iter()
            .map(|(id, score, metadata)| SearchResult {
                id,
                score: RelevanceScore::new(score),
                metadata,
            })
            .collect())
    }
    
    async fn delete(&self, id: DocumentId) -> Result<()> {
        self.data.write().await.remove(&id);
        Ok(())
    }
    
    async fn update_metadata(&self, id: DocumentId, metadata: DocumentMetadata) -> Result<()> {
        let mut data = self.data.write().await;
        if let Some((embedding, _)) = data.get(&id) {
            let embedding = embedding.clone();
            data.insert(id, (embedding, metadata));
            Ok(())
        } else {
            Err(SemanticSearchError::VectorStoreError(
                format!("Document {} not found", id.0)
            ))
        }
    }
    
    async fn count(&self) -> Result<usize> {
        Ok(self.data.read().await.len())
    }
    
    async fn clear(&self) -> Result<()> {
        self.data.write().await.clear();
        Ok(())
    }
}

/// Create a vector store based on configuration
pub async fn create_vector_store(config: VectorStoreConfig) -> Result<Arc<dyn VectorStore>> {
    match config.store_type {
        VectorStoreType::InMemory => {
            let store = InMemoryVectorStore::new(config);
            store.initialize().await?;
            Ok(Arc::new(store))
        }
        _ => Err(SemanticSearchError::ConfigError(
            format!("Vector store type {:?} not yet implemented", config.store_type)
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[tokio::test]
    async fn test_in_memory_vector_store() {
        let config = VectorStoreConfig {
            dimension: 3,
            ..Default::default()
        };
        
        let store = InMemoryVectorStore::new(config);
        store.initialize().await.unwrap();
        
        // Insert test vectors
        let id1 = DocumentId(Uuid::new_v4());
        let vec1 = vec![1.0, 0.0, 0.0];
        let metadata1 = DocumentMetadata {
            id: id1,
            title: Some("Test 1".to_string()),
            source: "test".to_string(),
            timestamp: chrono::Utc::now(),
            tags: vec!["test".to_string()],
            domain: "test".to_string(),
            entity_type: "document".to_string(),
            custom_fields: HashMap::new(),
        };
        
        store.insert(id1, vec1.clone(), metadata1.clone()).await.unwrap();
        
        // Search
        let results = store.search(&vec1, 10, None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id1);
        assert_eq!(results[0].score.0, 1.0); // Perfect match
    }
    
    #[tokio::test]
    async fn test_vector_store_filtering() {
        let config = VectorStoreConfig {
            dimension: 3,
            ..Default::default()
        };
        
        let store = InMemoryVectorStore::new(config);
        
        // Insert vectors with different domains
        for i in 0..5 {
            let id = DocumentId(Uuid::new_v4());
            let vec = vec![i as f32, 0.0, 0.0];
            let metadata = DocumentMetadata {
                id,
                title: Some(format!("Test {}", i)),
                source: "test".to_string(),
                timestamp: chrono::Utc::now(),
                tags: vec!["test".to_string()],
                domain: if i % 2 == 0 { "even".to_string() } else { "odd".to_string() },
                entity_type: "document".to_string(),
                custom_fields: HashMap::new(),
            };
            
            store.insert(id, vec, metadata).await.unwrap();
        }
        
        // Search with filter
        let filter = SearchFilter {
            domain: Some("even".to_string()),
            ..Default::default()
        };
        
        let results = store.search(&[1.0, 0.0, 0.0], 10, Some(filter)).await.unwrap();
        assert_eq!(results.len(), 3); // Only even domain results
    }
} 