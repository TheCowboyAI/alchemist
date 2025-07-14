//! Integration test for graph functionality

use alchemist::{
    graph_parser,
    nats_client::NatsClient,
};

#[cfg(feature = "bevy")]
use alchemist::{
    graph_components::*,
    graph_algorithms::*,
    jetstream_persistence::*,
};
use anyhow::Result;

#[tokio::test]
async fn test_json_graph_parsing() -> Result<()> {
    let json_content = r#"{
        "nodes": [
            {"id": "a", "label": "Node A"},
            {"id": "b", "label": "Node B"},
            {"id": "c", "label": "Node C"}
        ],
        "edges": [
            {"source": "a", "target": "b", "label": "connects"},
            {"source": "b", "target": "c", "label": "connects"}
        ]
    }"#;
    
    let (nodes, edges) = graph_parser::parse_json_graph(json_content)?;
    
    assert_eq!(nodes.len(), 3);
    assert_eq!(edges.len(), 2);
    assert_eq!(nodes[0].id, "a");
    assert_eq!(nodes[0].label, "Node A");
    
    Ok(())
}

#[tokio::test]
async fn test_nix_graph_parsing() -> Result<()> {
    let nix_content = r#"{ pkgs, ... }: {
        myPackage = pkgs.stdenv.mkDerivation {
            name = "test-package";
            buildInputs = [ pkgs.curl pkgs.git ];
        };
    }"#;
    
    let (nodes, edges) = graph_parser::parse_nix_graph(nix_content)?;
    
    // Should have nodes for myPackage, curl, git
    assert!(nodes.len() >= 3);
    // Should have edges from myPackage to its dependencies
    assert!(edges.len() >= 2);
    
    Ok(())
}

#[tokio::test]
async fn test_markdown_graph_parsing() -> Result<()> {
    let markdown_content = r#"# Main Title

## Section 1
Some content with a [link](https://example.com).

## Section 2
Another section with [internal link](#section-1).
"#;
    
    let (nodes, edges) = graph_parser::parse_markdown_graph(markdown_content)?;
    
    // Should have nodes for headings and links
    assert!(nodes.len() >= 4); // Main Title, Section 1, Section 2, external link
    assert!(edges.len() >= 3); // Hierarchy + links
    
    Ok(())
}

#[tokio::test]
async fn test_cytoscape_format_parsing() -> Result<()> {
    let cytoscape_content = r#"{
        "elements": [
            {"data": {"id": "n1", "label": "Node 1"}},
            {"data": {"id": "n2", "label": "Node 2"}},
            {"data": {"id": "e1", "source": "n1", "target": "n2"}}
        ]
    }"#;
    
    let (nodes, edges) = graph_parser::parse_json_graph(cytoscape_content)?;
    
    assert_eq!(nodes.len(), 2);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].source_id, "n1");
    assert_eq!(edges[0].target_id, "n2");
    
    Ok(())
}

#[cfg(feature = "bevy")]
#[tokio::test]
async fn test_graph_persistence_events() -> Result<()> {
    use chrono::Utc;
    
    // Test event serialization
    let event = GraphPersistenceEvent::NodeAdded {
        graph_id: "test_graph".to_string(),
        node_id: "node1".to_string(),
        label: "Test Node".to_string(),
        position: [1.0, 2.0, 3.0],
        metadata: serde_json::json!({"type": "test"}),
        timestamp: Utc::now().timestamp_millis(),
    };
    
    let serialized = serde_json::to_string(&event)?;
    let deserialized: GraphPersistenceEvent = serde_json::from_str(&serialized)?;
    
    match deserialized {
        GraphPersistenceEvent::NodeAdded { node_id, label, .. } => {
            assert_eq!(node_id, "node1");
            assert_eq!(label, "Test Node");
        }
        _ => panic!("Wrong event type"),
    }
    
    Ok(())
}

#[cfg(feature = "bevy")]
#[tokio::test]
async fn test_replayed_graph_construction() -> Result<()> {
    use chrono::Utc;
    
    let mut graph = ReplayedGraph::new("test".to_string());
    
    // Apply creation event
    graph.apply_event(GraphPersistenceEvent::GraphCreated {
        graph_id: "test".to_string(),
        name: "Test Graph".to_string(),
        metadata: serde_json::json!({}),
        timestamp: Utc::now().timestamp_millis(),
    });
    
    assert_eq!(graph.name, "Test Graph");
    
    // Apply node addition
    graph.apply_event(GraphPersistenceEvent::NodeAdded {
        graph_id: "test".to_string(),
        node_id: "n1".to_string(),
        label: "Node 1".to_string(),
        position: [0.0, 0.0, 0.0],
        metadata: serde_json::json!({}),
        timestamp: Utc::now().timestamp_millis(),
    });
    
    assert_eq!(graph.nodes.len(), 1);
    assert!(graph.nodes.contains_key("n1"));
    
    // Apply edge addition
    graph.apply_event(GraphPersistenceEvent::NodeAdded {
        graph_id: "test".to_string(),
        node_id: "n2".to_string(),
        label: "Node 2".to_string(),
        position: [1.0, 0.0, 0.0],
        metadata: serde_json::json!({}),
        timestamp: Utc::now().timestamp_millis(),
    });
    
    graph.apply_event(GraphPersistenceEvent::EdgeAdded {
        graph_id: "test".to_string(),
        edge_id: "e1".to_string(),
        source_id: "n1".to_string(),
        target_id: "n2".to_string(),
        label: Some("connects".to_string()),
        weight: 1.0,
        timestamp: Utc::now().timestamp_millis(),
    });
    
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    
    Ok(())
}

#[tokio::test]
async fn test_graph_structure_analysis() -> Result<()> {
    // Create a graph with two components
    let json_content = r#"{
        "nodes": [
            {"id": "1", "label": "Component 1 - A"},
            {"id": "2", "label": "Component 1 - B"},
            {"id": "3", "label": "Component 2 - A"},
            {"id": "4", "label": "Component 2 - B"},
            {"id": "5", "label": "Isolated"}
        ],
        "edges": [
            {"source": "1", "target": "2"},
            {"source": "3", "target": "4"}
        ]
    }"#;
    
    let (nodes, edges) = graph_parser::parse_json_graph(json_content)?;
    
    // Build adjacency for simple analysis
    let mut adjacency = std::collections::HashMap::new();
    for node in &nodes {
        adjacency.insert(node.id.clone(), Vec::new());
    }
    for edge in &edges {
        adjacency.get_mut(&edge.source_id).unwrap().push(edge.target_id.clone());
        adjacency.get_mut(&edge.target_id).unwrap().push(edge.source_id.clone());
    }
    
    // Count components
    let mut visited = std::collections::HashSet::new();
    let mut component_count = 0;
    
    for node in &nodes {
        if !visited.contains(&node.id) {
            component_count += 1;
            // DFS to mark component
            let mut stack = vec![node.id.clone()];
            while let Some(current) = stack.pop() {
                if !visited.contains(&current) {
                    visited.insert(current.clone());
                    if let Some(neighbors) = adjacency.get(&current) {
                        stack.extend(neighbors.clone());
                    }
                }
            }
        }
    }
    
    assert_eq!(component_count, 3); // Two connected components + one isolated
    
    Ok(())
}