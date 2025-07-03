//! Semantic search capabilities for CIM
//! 
//! This module provides semantic search functionality using vector embeddings
//! and similarity search to enable intelligent content discovery across the
//! CIM knowledge graph.

pub mod embedding_service;
pub mod vector_store;
pub mod search_engine;
pub mod indexing_pipeline;

pub use embedding_service::{EmbeddingService, EmbeddingModel, EmbeddingConfig};
pub use vector_store::{VectorStore, VectorStoreConfig, SearchResult, create_vector_store};
pub use search_engine::{SemanticSearchEngine, SearchQuery, SearchOptions};
pub use indexing_pipeline::{IndexingPipeline, IndexingConfig};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for searchable documents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub Uuid);

/// Metadata associated with searchable content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: DocumentId,
    pub title: Option<String>,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub domain: String,
    pub entity_type: String,
    pub custom_fields: std::collections::HashMap<String, serde_json::Value>,
}

/// A searchable document with content and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchableDocument {
    pub metadata: DocumentMetadata,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
}

/// Search relevance score
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RelevanceScore(pub f32);

impl RelevanceScore {
    /// Create a new relevance score, clamped to [0.0, 1.0]
    pub fn new(score: f32) -> Self {
        Self(score.clamp(0.0, 1.0))
    }
}

/// Error types for semantic search operations
#[derive(Debug, thiserror::Error)]
pub enum SemanticSearchError {
    #[error("Embedding service error: {0}")]
    EmbeddingError(String),
    
    #[error("Vector store error: {0}")]
    VectorStoreError(String),
    
    #[error("Indexing error: {0}")]
    IndexingError(String),
    
    #[error("Search query error: {0}")]
    QueryError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SemanticSearchError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_relevance_score_clamping() {
        assert_eq!(RelevanceScore::new(1.5).0, 1.0);
        assert_eq!(RelevanceScore::new(-0.5).0, 0.0);
        assert_eq!(RelevanceScore::new(0.75).0, 0.75);
    }
    
    #[test]
    fn test_document_id_generation() {
        let id1 = DocumentId(Uuid::new_v4());
        let id2 = DocumentId(Uuid::new_v4());
        assert_ne!(id1, id2);
    }
} 