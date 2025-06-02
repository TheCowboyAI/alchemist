//! Verification of storage implementation
//! This is a simple verification module to ensure our storage works

use super::storage::*;
use super::domain::*;
use super::events::*;
use uuid::Uuid;
use std::collections::HashMap;

/// Verify basic storage operations
pub fn verify_storage_operations() {
    println!("=== Verifying Storage Operations ===");

    // Create storage
    let mut storage = GraphStorage::new();
    println!("✓ Created storage");

    // Create a graph
    let graph_id = GraphIdentity(Uuid::new_v4());
    match storage.create_graph(graph_id) {
        Ok(_) => println!("✓ Created graph: {:?}", graph_id),
        Err(e) => println!("✗ Failed to create graph: {:?}", e),
    }

    // Add nodes
    let node1_id = NodeIdentity(Uuid::new_v4());
    let node1_data = NodeData {
        identity: node1_id,
        content: NodeContent {
            label: "Node 1".to_string(),
            category: "test".to_string(),
            properties: HashMap::new(),
        },
        position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
    };

    match storage.add_node(graph_id, node1_data) {
        Ok(idx) => println!("✓ Added node1 at index: {:?}", idx),
        Err(e) => println!("✗ Failed to add node1: {:?}", e),
    }

    let node2_id = NodeIdentity(Uuid::new_v4());
    let node2_data = NodeData {
        identity: node2_id,
        content: NodeContent {
            label: "Node 2".to_string(),
            category: "test".to_string(),
            properties: HashMap::new(),
        },
        position: SpatialPosition::at_3d(1.0, 1.0, 0.0),
    };

    match storage.add_node(graph_id, node2_data) {
        Ok(idx) => println!("✓ Added node2 at index: {:?}", idx),
        Err(e) => println!("✗ Failed to add node2: {:?}", e),
    }

    // Add edge
    let edge_data = EdgeData {
        identity: EdgeIdentity(Uuid::new_v4()),
        relationship: EdgeRelationship {
            source: node1_id,
            target: node2_id,
            category: "connects".to_string(),
            strength: 1.0,
            properties: HashMap::new(),
        },
    };

    match storage.add_edge(graph_id, node1_id, node2_id, edge_data) {
        Ok(idx) => println!("✓ Added edge at index: {:?}", idx),
        Err(e) => println!("✗ Failed to add edge: {:?}", e),
    }

    // Verify storage
    let nodes = storage.get_nodes(graph_id);
    println!("✓ Retrieved {} nodes", nodes.len());

    let edges = storage.get_edges(graph_id);
    println!("✓ Retrieved {} edges", edges.len());

    // Test error cases
    let non_existent_graph = GraphIdentity(Uuid::new_v4());
    let test_node_data = NodeData {
        identity: NodeIdentity(Uuid::new_v4()),
        content: NodeContent {
            label: "Test Node".to_string(),
            category: "test".to_string(),
            properties: HashMap::new(),
        },
        position: SpatialPosition::at_3d(0.0, 0.0, 0.0),
    };

    match storage.add_node(non_existent_graph, test_node_data) {
        Err(StorageError::GraphNotFound(_)) => println!("✓ Correctly errored on non-existent graph"),
        _ => println!("✗ Should have errored on non-existent graph"),
    }

    println!("\n=== Storage Verification Complete ===");
}
