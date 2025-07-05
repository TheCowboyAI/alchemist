//! Indexing pipeline for processing documents from various sources
//! 
//! This module provides functionality to ingest documents from different
//! domains and prepare them for semantic search.

use super::{
    DocumentId, DocumentMetadata, Result, SearchableDocument, SemanticSearchEngine,
    SemanticSearchError,
};
use crate::events::{NodeAdded, EdgeAdded};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Wrapper for entity data that can be sent between threads
pub struct EntityData {
    inner: Arc<dyn std::any::Any + Send + Sync>,
}

impl EntityData {
    pub fn new<T: std::any::Any + Send + Sync + 'static>(data: T) -> Self {
        Self {
            inner: Arc::new(data),
        }
    }
    
    #[must_use] pub fn downcast_ref<T: std::any::Any + 'static>(&self) -> Option<&T> {
        self.inner.downcast_ref::<T>()
    }
}

/// Configuration for the indexing pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingConfig {
    pub batch_size: usize,
    pub parallel_workers: usize,
    pub index_on_create: bool,
    pub index_on_update: bool,
    pub index_on_delete: bool,
    pub domains_to_index: Vec<String>,
}

impl Default for IndexingConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            parallel_workers: 4,
            index_on_create: true,
            index_on_update: true,
            index_on_delete: true,
            domains_to_index: vec![
                "graph".to_string(),
                "document".to_string(),
                "agent".to_string(),
                "workflow".to_string(),
            ],
        }
    }
}

/// Trait for document extractors
#[async_trait]
pub trait DocumentExtractor: Send + Sync {
    /// Extract searchable documents from domain entities
    async fn extract(&self, entity: &EntityData) -> Result<Vec<SearchableDocument>>;
    
    /// Get the domain this extractor handles
    fn domain(&self) -> &str;
}

/// Graph domain document extractor
pub struct GraphDocumentExtractor;

#[async_trait]
impl DocumentExtractor for GraphDocumentExtractor {
    async fn extract(&self, entity: &EntityData) -> Result<Vec<SearchableDocument>> {
        let mut documents = Vec::new();
        
        if let Some(node_event) = entity.downcast_ref::<NodeAdded>() {
            let title = node_event.metadata.get("title")
                .cloned()
                .or_else(|| Some(format!("Node {:?}", node_event.node_type)));
            
            let tags: Vec<String> = node_event.metadata.get("tags")
                .and_then(|t| t.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>().into())
                .unwrap_or_default();
            
            let doc = SearchableDocument {
                metadata: DocumentMetadata {
                    id: DocumentId(Uuid::new_v4()),
                    title,
                    source: format!("graph/{}", node_event.graph_id.0),
                    timestamp: chrono::Utc::now(),
                    tags,
                    domain: "graph".to_string(),
                    entity_type: "node".to_string(),
                    custom_fields: serde_json::json!({
                        "node_id": node_event.node_id.0,
                        "graph_id": node_event.graph_id.0,
                        "node_type": format!("{:?}", node_event.node_type),
                        "position": format!("{:?}", node_event.position),
                    }).as_object().unwrap().clone().into_iter().collect(),
                },
                content: format!(
                    "Node Type: {:?}\nPosition: {:?}\nMetadata: {:?}",
                    node_event.node_type,
                    node_event.position,
                    node_event.metadata
                ),
                embedding: None,
            };
            documents.push(doc);
        } else if let Some(edge_event) = entity.downcast_ref::<EdgeAdded>() {
            let doc = SearchableDocument {
                metadata: DocumentMetadata {
                    id: DocumentId(Uuid::new_v4()),
                    title: Some(format!("Edge: {:?}", edge_event.relationship)),
                    source: format!("graph/{}", edge_event.graph_id.0),
                    timestamp: chrono::Utc::now(),
                    tags: vec!["edge".to_string()],
                    domain: "graph".to_string(),
                    entity_type: "edge".to_string(),
                    custom_fields: serde_json::json!({
                        "edge_id": edge_event.edge_id.0,
                        "graph_id": edge_event.graph_id.0,
                        "source": edge_event.source.0,
                        "target": edge_event.target.0,
                        "relationship": format!("{:?}", edge_event.relationship),
                    }).as_object().unwrap().clone().into_iter().collect(),
                },
                content: format!(
                    "Edge connecting nodes with relationship: {:?}",
                    edge_event.relationship
                ),
                embedding: None,
            };
            documents.push(doc);
        }
        
        Ok(documents)
    }
    
    fn domain(&self) -> &'static str {
        "graph"
    }
}

/// Document domain extractor (placeholder)
pub struct DocumentDomainExtractor;

#[async_trait]
impl DocumentExtractor for DocumentDomainExtractor {
    async fn extract(&self, _entity: &EntityData) -> Result<Vec<SearchableDocument>> {
        // TODO: Implement when document events are available
        Ok(Vec::new())
    }
    
    fn domain(&self) -> &'static str {
        "document"
    }
}

/// Agent domain extractor
pub struct AgentDocumentExtractor;

#[async_trait]
impl DocumentExtractor for AgentDocumentExtractor {
    async fn extract(&self, entity: &EntityData) -> Result<Vec<SearchableDocument>> {
        let mut documents = Vec::new();
        
        // Check for AgentDeployed event
        if let Some(event) = entity.downcast_ref::<crate::events::AgentDeployed>() {
            let doc = SearchableDocument {
                metadata: DocumentMetadata {
                    id: DocumentId(event.agent_id.0),
                    title: Some(format!("Agent {:?}", event.agent_type)),
                    source: "agent".to_string(),
                    timestamp: chrono::Utc::now(),
                    tags: vec!["agent".to_string()],
                    domain: "agent".to_string(),
                    entity_type: "agent".to_string(),
                    custom_fields: serde_json::json!({
                        "agent_type": format!("{:?}", event.agent_type),
                        "owner_id": event.owner_id.to_string(),
                    }).as_object().unwrap().clone().into_iter().collect(),
                },
                content: format!(
                    "Agent deployed\nType: {:?}\nOwner: {}",
                    event.agent_type,
                    event.owner_id
                ),
                embedding: None,
            };
            documents.push(doc);
        }
        
        Ok(documents)
    }
    
    fn domain(&self) -> &'static str {
        "agent"
    }
}

/// Indexing pipeline
pub struct IndexingPipeline {
    search_engine: Arc<SemanticSearchEngine>,
    extractors: Vec<Arc<dyn DocumentExtractor>>,
    config: IndexingConfig,
    tx: mpsc::Sender<IndexingTask>,
    rx: Arc<tokio::sync::Mutex<mpsc::Receiver<IndexingTask>>>,
}

/// Task for the indexing pipeline
#[derive(Debug)]
enum IndexingTask {
    Index(Vec<SearchableDocument>),
    Delete(DocumentId),
    Shutdown,
}

impl IndexingPipeline {
    /// Create a new indexing pipeline
    #[must_use] pub fn new(
        search_engine: Arc<SemanticSearchEngine>,
        config: IndexingConfig,
    ) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        
        let extractors: Vec<Arc<dyn DocumentExtractor>> = vec![
            Arc::new(GraphDocumentExtractor),
            Arc::new(DocumentDomainExtractor),
            Arc::new(AgentDocumentExtractor),
        ];
        
        Self {
            search_engine,
            extractors,
            config,
            tx,
            rx: Arc::new(tokio::sync::Mutex::new(rx)),
        }
    }
    
    /// Start the indexing pipeline workers
    pub async fn start(&self) -> Result<()> {
        info!("Starting indexing pipeline with {} workers", self.config.parallel_workers);
        
        for worker_id in 0..self.config.parallel_workers {
            let search_engine = self.search_engine.clone();
            let rx = self.rx.clone();
            let batch_size = self.config.batch_size;
            
            tokio::spawn(async move {
                Self::worker_loop(worker_id, search_engine, rx, batch_size).await;
            });
        }
        
        Ok(())
    }
    
    /// Worker loop for processing indexing tasks
    async fn worker_loop(
        worker_id: usize,
        search_engine: Arc<SemanticSearchEngine>,
        rx: Arc<tokio::sync::Mutex<mpsc::Receiver<IndexingTask>>>,
        batch_size: usize,
    ) {
        let mut batch = Vec::with_capacity(batch_size);
        
        loop {
            let task = {
                let mut rx = rx.lock().await;
                rx.recv().await
            };
            
            match task {
                Some(IndexingTask::Index(docs)) => {
                    batch.extend(docs);
                    
                    if batch.len() >= batch_size {
                        if let Err(e) = search_engine.index_documents(batch.clone()).await {
                            warn!("Worker {} failed to index batch: {}", worker_id, e);
                        } else {
                            debug!("Worker {} indexed {} documents", worker_id, batch.len());
                        }
                        batch.clear();
                    }
                }
                Some(IndexingTask::Delete(id)) => {
                    // Flush batch before deletion
                    if !batch.is_empty() {
                        if let Err(e) = search_engine.index_documents(batch.clone()).await {
                            warn!("Worker {} failed to index batch: {}", worker_id, e);
                        }
                        batch.clear();
                    }
                    
                    if let Err(e) = search_engine.delete_document(id).await {
                        warn!("Worker {} failed to delete document: {}", worker_id, e);
                    }
                }
                Some(IndexingTask::Shutdown) | None => {
                    // Index remaining batch
                    if !batch.is_empty() {
                        if let Err(e) = search_engine.index_documents(batch).await {
                            warn!("Worker {} failed to index final batch: {}", worker_id, e);
                        }
                    }
                    info!("Worker {} shutting down", worker_id);
                    break;
                }
            }
        }
    }
    
    /// Process an entity for indexing
    pub async fn process_entity(&self, entity: &EntityData, domain: &str) -> Result<()> {
        if !self.config.domains_to_index.contains(&domain.to_string()) {
            return Ok(());
        }
        
        for extractor in &self.extractors {
            if extractor.domain() == domain {
                let documents = extractor.extract(entity).await?;
                if !documents.is_empty() {
                    self.tx.send(IndexingTask::Index(documents)).await
                        .map_err(|_| SemanticSearchError::IndexingError(
                            "Failed to send indexing task".to_string()
                        ))?;
                }
                break;
            }
        }
        
        Ok(())
    }
    
    /// Delete a document from the index
    pub async fn delete_document(&self, id: DocumentId) -> Result<()> {
        self.tx.send(IndexingTask::Delete(id)).await
            .map_err(|_| SemanticSearchError::IndexingError(
                "Failed to send delete task".to_string()
            ))
    }
    
    /// Shutdown the indexing pipeline
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down indexing pipeline");
        
        for _ in 0..self.config.parallel_workers {
            self.tx.send(IndexingTask::Shutdown).await
                .map_err(|_| SemanticSearchError::IndexingError(
                    "Failed to send shutdown signal".to_string()
                ))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_search::{
        EmbeddingConfig, EmbeddingService, VectorStoreConfig, create_vector_store,
    };
    use crate::value_objects::{GraphId, NodeId, NodeType};
    
    #[tokio::test]
    async fn test_graph_document_extractor() {
        let extractor = GraphDocumentExtractor;
        
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("title".to_string(), "Test Node".to_string());
        metadata.insert("body".to_string(), "This is a test node".to_string());
        metadata.insert("tags".to_string(), "test".to_string());
        
        let node_event = NodeAdded {
            graph_id: GraphId(Uuid::new_v4()),
            node_id: NodeId(Uuid::new_v4()),
            node_type: NodeType::Concept,
            position: Default::default(),
            metadata,
        };
        
        let entity_data = EntityData::new(node_event);
        let docs = extractor.extract(&entity_data).await.unwrap();
        
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].metadata.domain, "graph");
        assert_eq!(docs[0].metadata.entity_type, "node");
        assert!(docs[0].content.contains("Test Node"));
    }
    
    #[tokio::test]
    async fn test_indexing_pipeline() {
        let embedding_service = Arc::new(
            EmbeddingService::new(EmbeddingConfig::default()).unwrap()
        );
        
        let vector_store = create_vector_store(VectorStoreConfig {
            dimension: 384,
            ..Default::default()
        }).await.unwrap();
        
        let search_engine = Arc::new(SemanticSearchEngine::new(
            embedding_service,
            vector_store,
        ));
        
        let config = IndexingConfig {
            parallel_workers: 2,
            batch_size: 10,
            ..Default::default()
        };
        
        let pipeline = IndexingPipeline::new(search_engine.clone(), config);
        pipeline.start().await.unwrap();
        
        // Process a test entity
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("title".to_string(), "Pipeline Test".to_string());
        metadata.insert("body".to_string(), "Testing the indexing pipeline".to_string());
        metadata.insert("tags".to_string(), "pipeline".to_string());
        
        let node_event = NodeAdded {
            graph_id: GraphId(Uuid::new_v4()),
            node_id: NodeId(Uuid::new_v4()),
            node_type: NodeType::Concept,
            position: Default::default(),
            metadata,
        };
        
        let entity_data = EntityData::new(node_event);
        pipeline.process_entity(&entity_data, "graph").await.unwrap();
        
        // Give workers time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Verify document was indexed
        let stats = search_engine.get_stats().await.unwrap();
        assert!(stats.document_count > 0);
        
        pipeline.shutdown().await.unwrap();
    }
} 