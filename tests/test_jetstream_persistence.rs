//! Tests for JetStream persistence functionality

use alchemist::jetstream_persistence::*;
use uuid::Uuid;

#[tokio::test]
async fn test_graph_persistence_event_creation() {
    // Test event creation
    let graph_id = Uuid::new_v4();
    let event = GraphPersistenceEvent::GraphCreated {
        graph_id,
        name: "Test Graph".to_string(),
        metadata: Default::default(),
    };
    
    // Verify serialization
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("Test Graph"));
    
    // Verify deserialization
    let decoded: GraphPersistenceEvent = serde_json::from_str(&json).unwrap();
    match decoded {
        GraphPersistenceEvent::GraphCreated { name, .. } => {
            assert_eq!(name, "Test Graph");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_persistence_event_types() {
    let graph_id = Uuid::new_v4();
    let node_id = Uuid::new_v4();
    let edge_id = Uuid::new_v4();
    
    // Test all event types can be created
    let events = vec![
        GraphPersistenceEvent::GraphCreated {
            graph_id,
            name: "Test".to_string(),
            metadata: Default::default(),
        },
        GraphPersistenceEvent::NodeAdded {
            graph_id,
            node_id,
            data: Default::default(),
        },
        GraphPersistenceEvent::EdgeAdded {
            graph_id,
            edge_id,
            source: node_id,
            target: node_id,
            data: Default::default(),
        },
        GraphPersistenceEvent::GraphUpdated {
            graph_id,
            metadata: Default::default(),
        },
        GraphPersistenceEvent::NodeRemoved {
            graph_id,
            node_id,
        },
        GraphPersistenceEvent::EdgeRemoved {
            graph_id,
            edge_id,
        },
    ];
    
    // All events should serialize successfully
    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.is_empty());
    }
}

#[test] 
fn test_graph_snapshot() {
    let mut snapshot = GraphSnapshot {
        graph_id: Uuid::new_v4(),
        version: 1,
        nodes: Default::default(),
        edges: Default::default(),
        metadata: Default::default(),
        created_at: chrono::Utc::now(),
    };
    
    // Add nodes
    let node1 = Uuid::new_v4();
    let node2 = Uuid::new_v4();
    snapshot.nodes.insert(node1, Default::default());
    snapshot.nodes.insert(node2, Default::default());
    
    // Add edge
    let edge_id = Uuid::new_v4();
    snapshot.edges.insert(edge_id, (Default::default(), node1, node2));
    
    assert_eq!(snapshot.nodes.len(), 2);
    assert_eq!(snapshot.edges.len(), 1);
    
    // Test serialization
    let json = serde_json::to_string(&snapshot).unwrap();
    let decoded: GraphSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.nodes.len(), 2);
}