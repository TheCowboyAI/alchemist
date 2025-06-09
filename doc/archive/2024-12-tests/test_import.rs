//! Test import functionality without Bevy

use ia::domain::{
    commands::{Command, GraphCommand, ImportSource, ImportOptions},
    events::{DomainEvent, GraphEvent, NodeEvent, EdgeEvent},
    services::{GraphImportService, ImportFormat, graph_import::ImportedGraph},
    value_objects::{GraphId, NodeId, EdgeId},
};
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn main() {
    // Set up logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting import test application");

    // Test 1: Direct domain service test
    test_domain_import();

    // Test 2: Command handler test
    test_command_import();

    info!("All tests complete!");
}

fn test_domain_import() {
    info!("=== Testing Domain Import Service ===");

    let import_service = GraphImportService::new();

    // Read the sample file
    let content = std::fs::read_to_string("examples/data/sample_graph.json")
        .expect("Failed to read sample file");

    info!("Read {} bytes from sample file", content.len());

    // Import using the service directly
    match import_service.import_from_content(&content, ImportFormat::ArrowsApp, None) {
        Ok(imported_graph) => {
            info!("Successfully imported graph:");
            info!("  - {} nodes", imported_graph.nodes.len());
            info!("  - {} edges", imported_graph.edges.len());

            // Log node details
            for node in &imported_graph.nodes {
                info!("  Node: {} at ({}, {}, {})",
                    node.id, node.position.x, node.position.y, node.position.z);
                if let Some(caption) = node.properties.get("caption")
                    .and_then(|v| v.as_str()) {
                    info!("    Caption: {}", caption);
                }
                info!("    Type: {}", node.node_type);
                info!("    Label: {}", node.label);
            }

            // Log edge details
            for edge in &imported_graph.edges {
                info!("  Edge: {} -> {} ({})",
                    edge.source, edge.target, edge.edge_type);
            }
        }
        Err(e) => {
            eprintln!("Import failed: {}", e);
        }
    }
}

fn test_command_import() {
    info!("\n=== Testing Command Import ===");

    // Create a mock event collector
    let mut events = Vec::new();

    // Create the import command
    let graph_id = GraphId::new();
    let command = Command::Graph(GraphCommand::ImportGraph {
        graph_id,
        source: ImportSource::File {
            path: "examples/data/sample_graph.json".to_string(),
        },
        format: "arrows_app".to_string(),
        options: ImportOptions::default(),
    });

    info!("Created import command for graph: {:?}", graph_id);

    // Process the command (simulating what the handler would do)
    match command {
        Command::Graph(GraphCommand::ImportGraph { graph_id, source, format, options }) => {
            // First emit the import requested event
            events.push(DomainEvent::Graph(GraphEvent::GraphImportRequested {
                graph_id,
                source: source.clone(),
                format: format.clone(),
                options: options.clone(),
            }));

            // Load the content
            let content = match &source {
                ImportSource::File { path } => {
                    std::fs::read_to_string(path).ok()
                }
                ImportSource::InlineContent { content } => Some(content.clone()),
                _ => None,
            };

            if let Some(content) = content {
                let import_service = GraphImportService::new();

                let format = match format.as_str() {
                    "arrows_app" => ImportFormat::ArrowsApp,
                    "mermaid" => ImportFormat::Mermaid,
                    "cypher" => ImportFormat::Cypher,
                    "dot" => ImportFormat::Dot,
                    "progress_json" => ImportFormat::ProgressJson,
                    "vocabulary_json" => ImportFormat::VocabularyJson,
                    "rss_atom" => ImportFormat::RssAtom,
                    _ => ImportFormat::ArrowsApp,
                };

                match import_service.import_from_content(&content, format, options.mapping.as_ref()) {
                    Ok(imported_graph) => {
                        info!("Import successful, generating events...");

                        // Keep track of node ID mappings
                        let mut node_id_map = HashMap::new();

                        // Generate node events
                        for node in imported_graph.nodes {
                            let node_id = NodeId::new();
                            node_id_map.insert(node.id.clone(), node_id);

                            let mut metadata = node.properties;
                            if let Some(caption) = metadata.get("caption") {
                                metadata.insert("label".to_string(), caption.clone());
                            }

                            events.push(DomainEvent::Node(NodeEvent::NodeAdded {
                                graph_id,
                                node_id,
                                metadata,
                                position: node.position,
                            }));
                        }

                        // Generate edge events
                        for edge in imported_graph.edges {
                            // Look up the actual NodeIds from our mapping
                            if let (Some(&source_id), Some(&target_id)) =
                                (node_id_map.get(&edge.source), node_id_map.get(&edge.target)) {

                                events.push(DomainEvent::Edge(EdgeEvent::EdgeConnected {
                                    graph_id,
                                    edge_id: EdgeId::new(),
                                    source: source_id,
                                    target: target_id,
                                    relationship: edge.edge_type,
                                }));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Import failed: {}", e);
                    }
                }
            }
        }
        _ => {}
    }

    // Report the events that would be generated
    info!("\nGenerated {} events:", events.len());
    let mut node_count = 0;
    let mut edge_count = 0;

    for event in &events {
        match event {
            DomainEvent::Graph(GraphEvent::GraphImportRequested { .. }) => {
                info!("  - GraphImportRequested");
            }
            DomainEvent::Node(NodeEvent::NodeAdded { node_id, position, .. }) => {
                node_count += 1;
                info!("  - NodeAdded: {} at ({}, {}, {})",
                    node_id, position.x, position.y, position.z);
            }
            DomainEvent::Edge(EdgeEvent::EdgeConnected { source, target, relationship, .. }) => {
                edge_count += 1;
                info!("  - EdgeConnected: {} -> {} ({})",
                    source, target, relationship);
            }
            _ => {}
        }
    }

    info!("\nSummary: {} nodes, {} edges imported", node_count, edge_count);
}
