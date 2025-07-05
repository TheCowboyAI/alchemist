//! Minimal integration test to verify basic functionality

use cim_domain::{DomainResult, GraphId, NodeId};

#[tokio::test]
async fn test_basic_domain_functionality() -> DomainResult<()> {
    // Test that basic domain types work
    let graph_id = GraphId::new();
    let node_id = NodeId::new();
    
    println!("Created graph_id: {}", graph_id);
    println!("Created node_id: {}", node_id);
    
    assert_ne!(graph_id.as_uuid(), node_id.as_uuid());
    
    Ok(())
}

#[tokio::test]
async fn test_event_store_in_memory() -> DomainResult<()> {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    // Simple in-memory event store
    let events: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    
    // Add some events
    {
        let mut events_guard = events.lock().await;
        events_guard.push("Event1".to_string());
        events_guard.push("Event2".to_string());
    }
    
    // Verify events
    let events_guard = events.lock().await;
    assert_eq!(events_guard.len(), 2);
    assert_eq!(events_guard[0], "Event1");
    assert_eq!(events_guard[1], "Event2");
    
    Ok(())
}

#[tokio::test] 
async fn test_graph_components() -> DomainResult<()> {
    use cim_domain_graph::{GraphType, components::GraphEntity, components::GraphMetadata};
    
    // Create graph components
    let graph_id = GraphId::new();
    let graph_entity = GraphEntity {
        graph_id,
        graph_type: GraphType::Workflow,
    };
    
    // Create metadata
    let metadata = GraphMetadata {
        name: "Integration Test Graph".to_string(),
        description: "A test graph for integration testing".to_string(),
        tags: vec![],
        properties: std::collections::HashMap::new(),
        created_at: std::time::SystemTime::now(),
        updated_at: std::time::SystemTime::now(),
    };
    
    // Verify
    assert_eq!(graph_entity.graph_id, graph_id);
    assert_eq!(graph_entity.graph_type, GraphType::Workflow);
    assert_eq!(metadata.name, "Integration Test Graph");
    
    println!("Created graph {} with type {:?}", graph_id, graph_entity.graph_type);
    
    Ok(())
}

#[tokio::test]
async fn test_graph_events() -> DomainResult<()> {
    use cim_domain_graph::{GraphCreated, NodeAdded, Position3D};
    use std::collections::HashMap;
    
    // Create a graph created event
    let graph_id = GraphId::new();
    let event = GraphCreated {
        graph_id,
        name: "Test Graph".to_string(),
        description: "Integration test graph".to_string(),
        graph_type: Some(cim_domain_graph::GraphType::General),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
    };
    
    // Create a node added event
    let node_event = NodeAdded {
        graph_id,
        node_id: NodeId::new(),
        position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
        node_type: "test_node".to_string(),
        metadata: HashMap::new(),
    };
    
    // Verify events
    assert_eq!(event.graph_id, graph_id);
    assert_eq!(event.name, "Test Graph");
    assert_eq!(node_event.graph_id, graph_id);
    
    println!("Created events for graph {}", graph_id);
    
    Ok(())
}

#[test]
fn test_sync_functionality() {
    // Test that sync tests work too
    let id1 = NodeId::new();
    let id2 = NodeId::new();
    
    assert_ne!(id1, id2);
    println!("Sync test passed with ids: {} and {}", id1, id2);
} 