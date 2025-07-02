# Semantic Search Guide

## Overview

The CIM Agent domain includes a comprehensive semantic search implementation that enables:
- Vector-based similarity search across different content types
- Embedding generation and storage
- Metadata-based filtering
- Cross-domain search capabilities

## Architecture

### Core Components

1. **Vector Store** (`InMemoryVectorStore`)
   - Stores embeddings with metadata
   - Supports similarity search with cosine distance
   - Provides filtering by source type and metadata

2. **Embedding Service** (`EmbeddingService` trait)
   - Generates vector embeddings from text
   - Supports caching to avoid redundant API calls
   - Pluggable architecture for different providers

3. **Search Engine** (`SemanticSearchEngine`)
   - Combines vector store and embedding service
   - Handles indexing and search operations
   - Provides a unified API for semantic search

## Usage

### Basic Setup

```rust
use cim_domain_agent::semantic_search::{
    search_engine::DefaultSemanticSearchEngine,
    embedding_service::MockEmbeddingService,
    vector_store::InMemoryVectorStore,
    SemanticSearchConfig,
};
use std::sync::Arc;

// Create components
let embedding_service = Arc::new(MockEmbeddingService::new(384));
let vector_store = Arc::new(InMemoryVectorStore::new());
let config = SemanticSearchConfig::default();

// Create search engine
let engine = Arc::new(DefaultSemanticSearchEngine::new(
    embedding_service,
    vector_store,
    config,
));
```

### Indexing Content

```rust
use cim_domain_agent::semantic_search::EmbeddingRequest;
use std::collections::HashMap;
use serde_json::json;

// Index a document
let request = EmbeddingRequest {
    text: "Graph-based workflow optimization".to_string(),
    source_id: "doc1".to_string(),
    source_type: "document".to_string(),
    metadata: HashMap::from([
        ("category".to_string(), json!("workflow")),
    ]),
    model: None,
};

let embedding_id = engine.index(request).await?;
```

### Searching

```rust
use cim_domain_agent::semantic_search::{
    SearchQuery,
    vector_store::SearchFilter,
};

// Basic search
let query = SearchQuery::new("workflow optimization")
    .with_limit(5)
    .with_min_similarity(0.7);

let results = engine.search(query).await?;

// Filtered search
let filter = SearchFilter {
    source_types: Some(vec!["document".to_string()]),
    metadata_filters: HashMap::from([
        ("category".to_string(), json!("workflow")),
    ]),
    created_after: None,
    created_before: None,
};

let filtered_query = SearchQuery::new("optimization")
    .with_filter(filter);

let filtered_results = engine.search(filtered_query).await?;
```

## Integration with AI Providers

### Using Real Embeddings

For production use, integrate with real embedding providers:

```rust
use cim_domain_agent::{
    semantic_search::embedding_service::AIProviderEmbeddingService,
    ai_providers::{ProviderConfig, ProviderType},
};

// Create embedding service with OpenAI
let config = SemanticSearchConfig {
    default_model: "text-embedding-3-small".to_string(),
    dimensions: 1536,
    ..Default::default()
};

let embedding_service = AIProviderEmbeddingService::from_provider_config(
    ProviderConfig::OpenAI {
        api_key: std::env::var("OPENAI_API_KEY")?,
        model: "text-embedding-3-small".to_string(),
    },
    config,
).await?;
```

## Advanced Features

### Caching

The embedding service supports caching to avoid redundant API calls:

```rust
let config = SemanticSearchConfig {
    enable_cache: true,
    cache_ttl_seconds: 3600, // 1 hour
    ..Default::default()
};
```

### Cross-Domain Search

Search across different content types simultaneously:

```rust
// Index different types
let doc_request = EmbeddingRequest {
    source_type: "document".to_string(),
    // ...
};

let graph_request = EmbeddingRequest {
    source_type: "graph_node".to_string(),
    // ...
};

let concept_request = EmbeddingRequest {
    source_type: "concept".to_string(),
    // ...
};

// Search across all types
let cross_domain_query = SearchQuery::new("optimization")
    .with_limit(10);

let results = engine.search(cross_domain_query).await?;

// Group by type
let mut by_type: HashMap<String, Vec<_>> = HashMap::new();
for result in results {
    by_type.entry(result.source_type.clone())
        .or_insert_with(Vec::new)
        .push(result);
}
```

## Performance Considerations

### Vector Store Optimization

The current implementation uses an in-memory vector store with linear search. For production use with large datasets, consider:

1. **Approximate Nearest Neighbor (ANN) algorithms**
   - HNSW (Hierarchical Navigable Small World)
   - LSH (Locality Sensitive Hashing)
   - IVF (Inverted File Index)

2. **External Vector Databases**
   - Qdrant
   - Weaviate
   - Pinecone
   - Milvus

### Batch Operations

Use batch indexing for better performance:

```rust
let requests = vec![
    EmbeddingRequest { /* ... */ },
    EmbeddingRequest { /* ... */ },
    // ...
];

let ids = engine.index_batch(requests).await?;
```

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_semantic_search() {
    let engine = create_test_engine();
    
    // Index test data
    let id = engine.index(test_request()).await.unwrap();
    
    // Search
    let results = engine.search(
        SearchQuery::new("test query")
    ).await.unwrap();
    
    assert!(!results.is_empty());
}
```

### Integration Tests

See `cim-domain-agent/tests/semantic_search_tests.rs` for comprehensive integration tests covering:
- Embedding generation
- Similarity calculations
- Cross-domain search
- Filter functionality

## Future Enhancements

1. **Real Embedding Providers**
   - Implement native embedding APIs for OpenAI, Anthropic, etc.
   - Support for specialized embedding models (code, images, etc.)

2. **Advanced Search Features**
   - Hybrid search (combining vector and keyword search)
   - Faceted search with aggregations
   - Relevance tuning and re-ranking

3. **Scalability**
   - Distributed vector storage
   - Sharding and replication
   - GPU acceleration for similarity calculations

4. **Integration**
   - NATS-based distributed indexing
   - Event-driven index updates
   - Cross-domain entity resolution

## Example: Building a Knowledge Base

```rust
// Create a knowledge base with semantic search
struct KnowledgeBase {
    search_engine: Arc<dyn SemanticSearchEngine>,
}

impl KnowledgeBase {
    async fn add_article(&self, title: &str, content: &str, tags: Vec<String>) -> Result<()> {
        let request = EmbeddingRequest {
            text: format!("{} {}", title, content),
            source_id: Uuid::new_v4().to_string(),
            source_type: "article".to_string(),
            metadata: HashMap::from([
                ("title".to_string(), json!(title)),
                ("tags".to_string(), json!(tags)),
            ]),
            model: None,
        };
        
        self.search_engine.index(request).await?;
        Ok(())
    }
    
    async fn find_similar_articles(&self, query: &str, limit: usize) -> Result<Vec<Article>> {
        let results = self.search_engine.search(
            SearchQuery::new(query)
                .with_limit(limit)
                .with_min_similarity(0.7)
        ).await?;
        
        // Convert results to articles
        Ok(results.into_iter()
            .map(|r| Article::from_search_result(r))
            .collect())
    }
}
```

## Conclusion

The semantic search implementation in CIM provides a solid foundation for building intelligent search capabilities across different domain objects. By leveraging vector embeddings and similarity search, it enables semantic understanding beyond simple keyword matching, making it ideal for knowledge management, content discovery, and AI-powered applications. 