//! Semantic search engine combining embedding and vector search
//! 
//! This module provides the main search interface that combines
//! text embedding and vector similarity search.

use super::{
    DocumentId, DocumentMetadata, EmbeddingService, Result, 
    SearchResult, SearchableDocument, SemanticSearchError, VectorStore,
};
use super::vector_store::SearchFilter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Search query configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query_text: String,
    pub k: usize,
    pub filter: Option<SearchFilter>,
    pub options: SearchOptions,
}

impl SearchQuery {
    pub fn new(query_text: impl Into<String>) -> Self {
        Self {
            query_text: query_text.into(),
            k: 10,
            filter: None,
            options: SearchOptions::default(),
        }
    }
    
    pub fn with_k(mut self, k: usize) -> Self {
        self.k = k;
        self
    }
    
    pub fn with_filter(mut self, filter: SearchFilter) -> Self {
        self.filter = Some(filter);
        self
    }
    
    pub fn with_options(mut self, options: SearchOptions) -> Self {
        self.options = options;
        self
    }
}

/// Search options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub rerank: bool,
    pub expand_query: bool,
    pub min_score: Option<f32>,
    pub include_embeddings: bool,
    pub timeout_seconds: Option<u64>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            rerank: false,
            expand_query: false,
            min_score: None,
            include_embeddings: false,
            timeout_seconds: Some(30),
        }
    }
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub query_embedding: Option<Vec<f32>>,
    pub total_found: usize,
    pub search_time_ms: u64,
}

/// Semantic search engine
pub struct SemanticSearchEngine {
    embedding_service: Arc<EmbeddingService>,
    vector_store: Arc<dyn VectorStore>,
}

impl SemanticSearchEngine {
    /// Create a new semantic search engine
    pub fn new(
        embedding_service: Arc<EmbeddingService>,
        vector_store: Arc<dyn VectorStore>,
    ) -> Self {
        Self {
            embedding_service,
            vector_store,
        }
    }
    
    /// Search for documents
    pub async fn search(&self, query: SearchQuery) -> Result<SearchResponse> {
        let start_time = std::time::Instant::now();
        
        info!("Processing search query: {}", query.query_text);
        
        // Generate query embedding
        let query_embedding = self.embedding_service
            .embed(&query.query_text)
            .await?;
        
        debug!("Generated query embedding with dimension: {}", query_embedding.len());
        
        // Perform vector search
        let mut results = self.vector_store
            .search(&query_embedding, query.k, query.filter)
            .await?;
        
        debug!("Found {} initial results", results.len());
        
        // Apply minimum score filter if specified
        if let Some(min_score) = query.options.min_score {
            results.retain(|r| r.score.0 >= min_score);
            debug!("After score filtering: {} results", results.len());
        }
        
        // Rerank if requested
        if query.options.rerank {
            results = self.rerank_results(results, &query.query_text).await?;
            debug!("Reranked results");
        }
        
        let total_found = results.len();
        let search_time_ms = start_time.elapsed().as_millis() as u64;
        
        info!(
            "Search completed in {}ms, found {} results",
            search_time_ms, total_found
        );
        
        Ok(SearchResponse {
            results,
            query_embedding: if query.options.include_embeddings {
                Some(query_embedding)
            } else {
                None
            },
            total_found,
            search_time_ms,
        })
    }
    
    /// Index a document
    pub async fn index_document(&self, document: SearchableDocument) -> Result<()> {
        info!("Indexing document: {:?}", document.metadata.id);
        
        // Generate embedding if not provided
        let embedding = if let Some(emb) = document.embedding {
            emb
        } else {
            self.embedding_service
                .embed(&document.content)
                .await?
        };
        
        // Store in vector store
        self.vector_store
            .insert(document.metadata.id, embedding, document.metadata)
            .await?;
        
        debug!("Document indexed successfully");
        Ok(())
    }
    
    /// Index multiple documents
    pub async fn index_documents(&self, documents: Vec<SearchableDocument>) -> Result<()> {
        info!("Indexing {} documents", documents.len());
        
        let mut items = Vec::with_capacity(documents.len());
        
        // Generate embeddings for documents without them
        let mut texts_to_embed = Vec::new();
        let mut embed_indices = Vec::new();
        
        for (i, doc) in documents.iter().enumerate() {
            if doc.embedding.is_none() {
                texts_to_embed.push(doc.content.clone());
                embed_indices.push(i);
            }
        }
        
        let embeddings = if !texts_to_embed.is_empty() {
            self.embedding_service
                .embed_batch(&texts_to_embed)
                .await?
        } else {
            Vec::new()
        };
        
        // Prepare items for batch insert
        let mut embed_iter = embeddings.into_iter();
        for (i, doc) in documents.into_iter().enumerate() {
            let embedding = if embed_indices.contains(&i) {
                embed_iter.next()
                    .ok_or_else(|| SemanticSearchError::IndexingError(
                        "Missing embedding".to_string()
                    ))?
            } else {
                doc.embedding.unwrap()
            };
            
            items.push((doc.metadata.id, embedding, doc.metadata));
        }
        
        // Batch insert
        self.vector_store.insert_batch(items).await?;
        
        info!("Documents indexed successfully");
        Ok(())
    }
    
    /// Delete a document from the index
    pub async fn delete_document(&self, id: DocumentId) -> Result<()> {
        info!("Deleting document: {:?}", id);
        self.vector_store.delete(id).await
    }
    
    /// Update document metadata
    pub async fn update_metadata(&self, id: DocumentId, metadata: DocumentMetadata) -> Result<()> {
        info!("Updating metadata for document: {:?}", id);
        self.vector_store.update_metadata(id, metadata).await
    }
    
    /// Get index statistics
    pub async fn get_stats(&self) -> Result<IndexStats> {
        let document_count = self.vector_store.count().await?;
        let cache_stats = self.embedding_service.cache_stats().await;
        
        Ok(IndexStats {
            document_count,
            cache_stats,
        })
    }
    
    /// Clear the index
    pub async fn clear_index(&self) -> Result<()> {
        warn!("Clearing entire search index");
        self.vector_store.clear().await?;
        self.embedding_service.clear_cache().await;
        Ok(())
    }
    
    /// Rerank results (placeholder for more sophisticated reranking)
    async fn rerank_results(
        &self,
        mut results: Vec<SearchResult>,
        _query: &str,
    ) -> Result<Vec<SearchResult>> {
        // Simple reranking based on metadata freshness
        results.sort_by(|a, b| {
            let score_cmp = b.score.0.partial_cmp(&a.score.0)
                .unwrap_or(std::cmp::Ordering::Equal);
            
            if score_cmp == std::cmp::Ordering::Equal {
                // If scores are equal, prefer more recent documents
                b.metadata.timestamp.cmp(&a.metadata.timestamp)
            } else {
                score_cmp
            }
        });
        
        Ok(results)
    }
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub document_count: usize,
    pub cache_stats: super::embedding_service::CacheStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_search::{
        EmbeddingConfig, EmbeddingService, VectorStoreConfig, create_vector_store,
    };
    use uuid::Uuid;
    
    async fn create_test_engine() -> SemanticSearchEngine {
        let embedding_service = Arc::new(
            EmbeddingService::new(EmbeddingConfig::default()).unwrap()
        );
        
        let vector_store = create_vector_store(VectorStoreConfig {
            dimension: 384,
            ..Default::default()
        }).await.unwrap();
        
        SemanticSearchEngine::new(embedding_service, vector_store)
    }
    
    #[tokio::test]
    async fn test_search_engine() {
        let engine = create_test_engine().await;
        
        // Index test documents
        let docs = vec![
            SearchableDocument {
                metadata: DocumentMetadata {
                    id: DocumentId(Uuid::new_v4()),
                    title: Some("Rust Programming".to_string()),
                    source: "test".to_string(),
                    timestamp: chrono::Utc::now(),
                    tags: vec!["rust".to_string(), "programming".to_string()],
                    domain: "technology".to_string(),
                    entity_type: "article".to_string(),
                    custom_fields: Default::default(),
                },
                content: "Rust is a systems programming language focused on safety".to_string(),
                embedding: None,
            },
            SearchableDocument {
                metadata: DocumentMetadata {
                    id: DocumentId(Uuid::new_v4()),
                    title: Some("Python Tutorial".to_string()),
                    source: "test".to_string(),
                    timestamp: chrono::Utc::now(),
                    tags: vec!["python".to_string(), "programming".to_string()],
                    domain: "technology".to_string(),
                    entity_type: "tutorial".to_string(),
                    custom_fields: Default::default(),
                },
                content: "Python is a high-level programming language".to_string(),
                embedding: None,
            },
        ];
        
        engine.index_documents(docs).await.unwrap();
        
        // Search
        let query = SearchQuery::new("programming languages")
            .with_k(5);
        
        let response = engine.search(query).await.unwrap();
        
        assert_eq!(response.results.len(), 2);
        assert!(response.search_time_ms > 0);
    }
    
    #[tokio::test]
    async fn test_search_with_filter() {
        let engine = create_test_engine().await;
        
        // Index documents with different entity types
        let docs = vec![
            SearchableDocument {
                metadata: DocumentMetadata {
                    id: DocumentId(Uuid::new_v4()),
                    title: Some("Article 1".to_string()),
                    source: "test".to_string(),
                    timestamp: chrono::Utc::now(),
                    tags: vec!["test".to_string()],
                    domain: "test".to_string(),
                    entity_type: "article".to_string(),
                    custom_fields: Default::default(),
                },
                content: "Test content".to_string(),
                embedding: None,
            },
            SearchableDocument {
                metadata: DocumentMetadata {
                    id: DocumentId(Uuid::new_v4()),
                    title: Some("Tutorial 1".to_string()),
                    source: "test".to_string(),
                    timestamp: chrono::Utc::now(),
                    tags: vec!["test".to_string()],
                    domain: "test".to_string(),
                    entity_type: "tutorial".to_string(),
                    custom_fields: Default::default(),
                },
                content: "Test content".to_string(),
                embedding: None,
            },
        ];
        
        engine.index_documents(docs).await.unwrap();
        
        // Search with filter
        let filter = SearchFilter {
            entity_type: Some("article".to_string()),
            ..Default::default()
        };
        
        let query = SearchQuery::new("test")
            .with_filter(filter);
        
        let response = engine.search(query).await.unwrap();
        
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].metadata.entity_type, "article");
    }
} 