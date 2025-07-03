//! Semantic Search Demo
//! 
//! This example demonstrates the semantic search capabilities of CIM,
//! including document indexing, embedding generation, and similarity search.

use bevy::prelude::*;
use ia::prelude::*;
use ia::semantic_search::{
    DocumentId, DocumentMetadata, EmbeddingConfig, EmbeddingService,
    IndexingConfig, IndexingPipeline, SearchQuery, SearchableDocument,
    SemanticSearchEngine, VectorStoreConfig, create_vector_store,
};
use std::sync::Arc;
use uuid::Uuid;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ia::plugins::CimPlugins)
        .add_systems(Startup, setup_semantic_search)
        .add_systems(Update, (
            index_graph_events,
            perform_searches.run_if(run_once()),
        ))
        .run();
}

/// Setup semantic search components
fn setup_semantic_search(
    mut commands: Commands,
) {
    info!("Setting up semantic search demo...");
    
    // Create camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Add light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Create demo graph
    create_demo_graph(&mut commands);
}

/// Create a demo graph with various nodes
fn create_demo_graph(commands: &mut Commands) {
    use ia::events::{NodeAdded, EdgeAdded};
    use ia::value_objects::{GraphId, NodeId, NodeType, EdgeId, EdgeRelationship, Position3D};
    
    let graph_id = GraphId(Uuid::new_v4());
    
    // Create nodes with different content
    let nodes = vec![
        ("Rust Programming", "A systems programming language focused on safety, speed, and concurrency"),
        ("Bevy Engine", "A data-driven game engine built in Rust using ECS architecture"),
        ("Entity Component System", "An architectural pattern for game development separating data from behavior"),
        ("Graph Database", "A database that uses graph structures with nodes, edges, and properties"),
        ("Event Sourcing", "A pattern where state changes are logged as a sequence of events"),
        ("Domain-Driven Design", "An approach to software development focusing on the core domain"),
        ("Semantic Search", "Search technology that understands the contextual meaning of search queries"),
        ("Vector Embeddings", "Numerical representations of text that capture semantic meaning"),
    ];
    
    let mut node_ids = Vec::new();
    
    for (i, (title, description)) in nodes.iter().enumerate() {
        let node_id = NodeId(Uuid::new_v4());
        node_ids.push(node_id);
        
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("title".to_string(), title.to_string());
        metadata.insert("description".to_string(), description.to_string());
        metadata.insert("tags".to_string(), "technology,programming".to_string());
        
        commands.spawn(NodeAdded {
            node_id,
            graph_id,
            node_type: NodeType::Concept,
            position: Position3D {
                x: (i as f32 * 5.0) - 17.5,
                y: 0.0,
                z: 0.0,
            },
            metadata,
        });
    }
    
    // Create some edges
    commands.spawn(EdgeAdded {
        edge_id: EdgeId(Uuid::new_v4()),
        graph_id,
        source: node_ids[0], // Rust -> Bevy
        target: node_ids[1],
        relationship: EdgeRelationship::DependsOn,
    });
    
    commands.spawn(EdgeAdded {
        edge_id: EdgeId(Uuid::new_v4()),
        graph_id,
        source: node_ids[1], // Bevy -> ECS
        target: node_ids[2],
        relationship: EdgeRelationship::Uses,
    });
    
    commands.spawn(EdgeAdded {
        edge_id: EdgeId(Uuid::new_v4()),
        graph_id,
        source: node_ids[6], // Semantic Search -> Vector Embeddings
        target: node_ids[7],
        relationship: EdgeRelationship::Uses,
    });
}

/// Index graph events into semantic search
fn index_graph_events(
    mut commands: Commands,
    mut node_events: EventReader<ia::events::NodeAdded>,
    mut indexed: Local<bool>,
) {
    if *indexed {
        return;
    }
    
    let mut documents = Vec::new();
    
    for event in node_events.read() {
        let title = event.metadata.get("title")
            .cloned()
            .unwrap_or_else(|| format!("Node {:?}", event.node_type));
        
        let description = event.metadata.get("description")
            .cloned()
            .unwrap_or_default();
        
        let tags: Vec<String> = event.metadata.get("tags")
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        
        let doc = SearchableDocument {
            metadata: DocumentMetadata {
                id: DocumentId(Uuid::new_v4()),
                title: Some(title.clone()),
                source: format!("graph/{}", event.graph_id.0),
                timestamp: chrono::Utc::now(),
                tags,
                domain: "graph".to_string(),
                entity_type: "node".to_string(),
                custom_fields: std::collections::HashMap::new(),
            },
            content: format!("{}\n\n{}", title, description),
            embedding: None,
        };
        
        documents.push(doc);
    }
    
    if !documents.is_empty() {
        *indexed = true;
        commands.insert_resource(IndexedDocuments(documents));
        info!("Indexed {} documents for semantic search", documents.len());
    }
}

#[derive(Resource)]
struct IndexedDocuments(Vec<SearchableDocument>);

/// Perform semantic searches
fn perform_searches(
    documents: Option<Res<IndexedDocuments>>,
) {
    let Some(documents) = documents else {
        return;
    };
    
    // Create runtime for async operations
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    runtime.block_on(async {
        // Setup semantic search engine
        let embedding_service = Arc::new(
            EmbeddingService::new(EmbeddingConfig::default()).unwrap()
        );
        
        let vector_store = create_vector_store(VectorStoreConfig {
            dimension: 384,
            ..Default::default()
        }).await.unwrap();
        
        let search_engine = SemanticSearchEngine::new(
            embedding_service,
            vector_store,
        );
        
        // Index documents
        info!("Indexing documents...");
        search_engine.index_documents(documents.0.clone()).await.unwrap();
        
        // Perform searches
        let queries = vec![
            "game development frameworks",
            "database technologies",
            "software architecture patterns",
            "programming languages for systems",
            "search and information retrieval",
        ];
        
        for query_text in queries {
            info!("\n🔍 Searching for: '{}'", query_text);
            
            let query = SearchQuery::new(query_text)
                .with_k(3);
            
            let response = search_engine.search(query).await.unwrap();
            
            info!("Found {} results in {}ms:", response.results.len(), response.search_time_ms);
            
            for (i, result) in response.results.iter().enumerate() {
                info!(
                    "  {}. {} (score: {:.3})",
                    i + 1,
                    result.metadata.title.as_deref().unwrap_or("Untitled"),
                    result.score.0
                );
            }
        }
        
        // Get statistics
        let stats = search_engine.get_stats().await.unwrap();
        info!("\n📊 Search Engine Statistics:");
        info!("  Documents indexed: {}", stats.document_count);
        info!("  Cache size: {}/{}", stats.cache_stats.size, stats.cache_stats.capacity);
        info!("  Cache hit rate: {:.1}%", stats.cache_stats.hit_rate * 100.0);
    });
} 