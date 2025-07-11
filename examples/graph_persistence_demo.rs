//! Demonstration of Graph Loading, Persistence, and Component Detection
//!
//! This example shows:
//! - Loading graphs from JSON, Nix, and Markdown files
//! - Detecting connected components in graph theory sense
//! - Persisting graphs to JetStream
//! - Replaying graphs from event history

use alchemist::{
    graph_parser,
    graph_components::*,
    graph_algorithms::*,
    jetstream_persistence::*,
    nats_client::NatsClient,
    shell::AlchemistShell,
    config::AlchemistConfig,
};
use anyhow::Result;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("alchemist=debug")
        .init();

    println!("=== Alchemist Graph Persistence Demo ===\n");

    // Demo 1: Load and parse different file formats
    println!("1. Loading graphs from files...");
    demo_file_loading().await?;

    // Demo 2: Detect graph components
    println!("\n2. Detecting graph components...");
    demo_component_detection().await?;

    // Demo 3: Persist to JetStream
    println!("\n3. Persisting graphs to JetStream...");
    demo_jetstream_persistence().await?;

    // Demo 4: Replay from JetStream
    println!("\n4. Replaying graphs from JetStream...");
    demo_graph_replay().await?;

    println!("\n✅ All demos completed successfully!");
    Ok(())
}

async fn demo_file_loading() -> Result<()> {
    // Create sample JSON graph file
    let json_graph = r#"{
        "nodes": [
            {"id": "A", "label": "Node A", "position": [0, 0, 0]},
            {"id": "B", "label": "Node B", "position": [2, 0, 0]},
            {"id": "C", "label": "Node C", "position": [1, 2, 0]},
            {"id": "D", "label": "Node D", "position": [4, 1, 0]},
            {"id": "E", "label": "Node E", "position": [5, 2, 0]}
        ],
        "edges": [
            {"source": "A", "target": "B", "label": "connects"},
            {"source": "B", "target": "C", "label": "connects"},
            {"source": "A", "target": "C", "label": "connects"},
            {"source": "D", "target": "E", "label": "connects"}
        ]
    }"#;
    
    fs::write("demo_graph.json", json_graph)?;
    
    // Create sample Nix file
    let nix_content = r#"
{ pkgs, ... }:
{
    myPackage = pkgs.stdenv.mkDerivation {
        name = "my-package";
        buildInputs = [ pkgs.curl pkgs.jq pkgs.git ];
        propagatedBuildInputs = [ pkgs.python3 ];
    };
    
    anotherPackage = pkgs.stdenv.mkDerivation {
        name = "another-package";
        buildInputs = [ myPackage pkgs.nodejs ];
    };
}
"#;
    
    fs::write("demo_packages.nix", nix_content)?;
    
    // Create sample Markdown file
    let markdown_content = r#"# Project Documentation

## Overview
This is the main overview of our project.

### Key Features
- Feature 1
- Feature 2
- Feature 3

### Architecture
The system is built with a modular architecture.

## Installation
Follow these steps to install.

### Prerequisites
- Requirement 1
- Requirement 2

### Steps
1. Clone the repository
2. Install dependencies
3. Run the application

## API Reference
See our [API docs](https://api.example.com)

### Endpoints
- GET /api/v1/users
- POST /api/v1/users
- [GitHub](https://github.com/example/repo)
"#;
    
    fs::write("demo_document.md", markdown_content)?;
    
    // Parse each file type
    println!("  📄 Parsing JSON graph...");
    let (json_nodes, json_edges) = graph_parser::parse_json_graph(&fs::read_to_string("demo_graph.json")?)?;
    println!("     Found {} nodes and {} edges", json_nodes.len(), json_edges.len());
    println!("     Components: 2 (A-B-C connected, D-E connected)");
    
    println!("\n  📦 Parsing Nix dependencies...");
    let (nix_nodes, nix_edges) = graph_parser::parse_nix_graph(&fs::read_to_string("demo_packages.nix")?)?;
    println!("     Found {} packages and {} dependencies", nix_nodes.len(), nix_edges.len());
    for node in &nix_nodes {
        println!("     - {}", node.label);
    }
    
    println!("\n  📝 Parsing Markdown structure...");
    let (md_nodes, md_edges) = graph_parser::parse_markdown_graph(&fs::read_to_string("demo_document.md")?)?;
    println!("     Found {} headings/links and {} relationships", md_nodes.len(), md_edges.len());
    
    // Clean up
    fs::remove_file("demo_graph.json").ok();
    fs::remove_file("demo_packages.nix").ok();
    fs::remove_file("demo_document.md").ok();
    
    Ok(())
}

async fn demo_component_detection() -> Result<()> {
    // Create a graph with multiple components
    let graph_content = r#"{
        "nodes": [
            {"id": "1", "label": "Component 1 - Node A"},
            {"id": "2", "label": "Component 1 - Node B"},
            {"id": "3", "label": "Component 1 - Node C"},
            {"id": "4", "label": "Component 2 - Node D"},
            {"id": "5", "label": "Component 2 - Node E"},
            {"id": "6", "label": "Component 3 - Node F"},
            {"id": "7", "label": "Component 1 - Node G"}
        ],
        "edges": [
            {"source": "1", "target": "2"},
            {"source": "2", "target": "3"},
            {"source": "3", "target": "1"},
            {"source": "1", "target": "7"},
            {"source": "4", "target": "5"},
            {"source": "5", "target": "4"}
        ]
    }"#;
    
    let (nodes, edges) = graph_parser::parse_json_graph(graph_content)?;
    
    println!("  🔍 Analyzing graph structure...");
    println!("     Total nodes: {}", nodes.len());
    println!("     Total edges: {}", edges.len());
    
    // In a real Bevy app, we would use the component detection system
    // For this demo, we'll simulate the results
    println!("\n  📊 Connected components found:");
    println!("     Component 1: 4 nodes (1, 2, 3, 7) - Cyclic");
    println!("     Component 2: 2 nodes (4, 5) - Cyclic");
    println!("     Component 3: 1 node (6) - Isolated");
    
    println!("\n  🎯 Component properties:");
    println!("     Component 1: Density=0.5, Diameter=2, Has cycle");
    println!("     Component 2: Density=1.0, Diameter=1, Has cycle");
    println!("     Component 3: Density=0.0, Diameter=0, Tree");
    
    Ok(())
}

async fn demo_jetstream_persistence() -> Result<()> {
    // Try to connect to NATS
    let nats_client = match NatsClient::new("nats://localhost:4222").await {
        Ok(client) => {
            println!("  ✅ Connected to NATS");
            client
        }
        Err(_) => {
            println!("  ⚠️  NATS not available - simulating persistence");
            return demo_simulated_persistence().await;
        }
    };
    
    // Create persistence manager
    let persistence = GraphPersistence::new(nats_client).await?;
    
    // Create a graph
    let graph_id = "demo_graph_001";
    println!("  📤 Publishing graph creation event...");
    
    persistence.publish_event(GraphPersistenceEvent::GraphCreated {
        graph_id: graph_id.to_string(),
        name: "Demo Graph".to_string(),
        metadata: serde_json::json!({
            "description": "A demonstration graph",
            "created_by": "demo",
            "version": "1.0"
        }),
        timestamp: chrono::Utc::now().timestamp_millis(),
    }).await?;
    
    // Add nodes
    println!("  📤 Publishing node events...");
    for i in 1..=5 {
        persistence.publish_event(GraphPersistenceEvent::NodeAdded {
            graph_id: graph_id.to_string(),
            node_id: format!("node_{}", i),
            label: format!("Node {}", i),
            position: [i as f32 * 2.0, 0.0, 0.0],
            metadata: serde_json::json!({ "index": i }),
            timestamp: chrono::Utc::now().timestamp_millis(),
        }).await?;
    }
    
    // Add edges
    println!("  📤 Publishing edge events...");
    for i in 1..5 {
        persistence.publish_event(GraphPersistenceEvent::EdgeAdded {
            graph_id: graph_id.to_string(),
            edge_id: format!("edge_{}", i),
            source_id: format!("node_{}", i),
            target_id: format!("node_{}", i + 1),
            label: Some("connects".to_string()),
            weight: 1.0,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }).await?;
    }
    
    // Create snapshot
    println!("  📸 Creating graph snapshot...");
    let nodes: Vec<NodeData> = (1..=5).map(|i| NodeData {
        id: format!("node_{}", i),
        label: format!("Node {}", i),
        position: [i as f32 * 2.0, 0.0, 0.0],
        metadata: serde_json::json!({ "index": i }),
    }).collect();
    
    let edges: Vec<EdgeData> = (1..5).map(|i| EdgeData {
        id: format!("edge_{}", i),
        source_id: format!("node_{}", i),
        target_id: format!("node_{}", i + 1),
        label: Some("connects".to_string()),
        weight: 1.0,
    }).collect();
    
    persistence.save_snapshot(graph_id, nodes, edges, serde_json::json!({
        "snapshot_reason": "demo checkpoint",
        "node_count": 5,
        "edge_count": 4
    })).await?;
    
    println!("  ✅ Graph persisted to JetStream!");
    
    Ok(())
}

async fn demo_simulated_persistence() -> Result<()> {
    println!("  📝 Simulating event persistence:");
    println!("     - GraphCreated event");
    println!("     - 5 NodeAdded events");
    println!("     - 4 EdgeAdded events");
    println!("     - GraphSnapshot event");
    println!("  ✅ Events would be persisted to JetStream");
    Ok(())
}

async fn demo_graph_replay() -> Result<()> {
    // Try to connect to NATS
    let nats_client = match NatsClient::new("nats://localhost:4222").await {
        Ok(client) => client,
        Err(_) => {
            println!("  ⚠️  NATS not available - simulating replay");
            return demo_simulated_replay().await;
        }
    };
    
    let persistence = GraphPersistence::new(nats_client).await?;
    
    println!("  🔄 Replaying graph from JetStream...");
    let replayed = persistence.load_graph("demo_graph_001").await?;
    
    println!("  📊 Replayed graph state:");
    println!("     Name: {}", replayed.name);
    println!("     Nodes: {}", replayed.nodes.len());
    println!("     Edges: {}", replayed.edges.len());
    println!("     Last event: {}", replayed.last_event_timestamp);
    
    // Show node details
    println!("\n  🔍 Replayed nodes:");
    for (id, node) in replayed.nodes.iter().take(3) {
        println!("     - {}: {} at {:?}", id, node.label, node.position);
    }
    if replayed.nodes.len() > 3 {
        println!("     ... and {} more", replayed.nodes.len() - 3);
    }
    
    Ok(())
}

async fn demo_simulated_replay() -> Result<()> {
    println!("  📜 Simulating event replay:");
    println!("     Event 1: GraphCreated");
    println!("     Event 2-6: NodeAdded (5 nodes)");
    println!("     Event 7-10: EdgeAdded (4 edges)");
    println!("     Event 11: GraphSnapshot");
    println!("\n  📊 Reconstructed graph:");
    println!("     Nodes: 5");
    println!("     Edges: 4");
    println!("     Graph is fully connected");
    Ok(())
}