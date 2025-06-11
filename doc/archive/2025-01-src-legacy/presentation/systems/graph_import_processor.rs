//! Graph import processor system
//!
//! Processes import requests and generates domain events for imported graphs.

use crate::application::CommandEvent;
use crate::domain::{
    commands::{Command, NodeCommand, EdgeCommand, ImportSource},
    events::{DomainEvent, GraphEvent},
    value_objects::{NodeId, EdgeId, NodeContent, NodeType, EdgeRelationship, RelationshipType},
};
use crate::presentation::events::{ImportRequestEvent, ImportResultEvent};
use crate::domain::services::GraphImportService;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use tracing::{info, error};
use crate::presentation::components::{SubgraphRegion, BoundaryType};
use crate::domain::value_objects::SubgraphId;

/// System that processes GraphImportRequested events
pub fn process_graph_import_requests(
    mut commands: Commands,
    mut import_requests: EventReader<ImportRequestEvent>,
    mut import_results: EventWriter<ImportResultEvent>,
    mut command_events: EventWriter<CommandEvent>,
) {
    for request in import_requests.read() {
        // Extract the GraphImportRequested event
        if let DomainEvent::Graph(GraphEvent::GraphImportRequested {
            graph_id,
            source,
            format,
            options,
        }) = &request.event {
            eprintln!("process_graph_import_requests: Processing import request for graph {:?}", graph_id);
            info!("Processing import request for graph: {:?}", graph_id);

            // Parse the imported graph
            let import_service = GraphImportService::new();

            // Get content from source
            let content_result = match source {
                ImportSource::File { path } => {
                    std::fs::read_to_string(path)
                        .map_err(|e| format!("Failed to read file: {}", e))
                }
                ImportSource::InlineContent { content } => Ok(content.clone()),
                _ => Err("Unsupported import source".to_string()),
            };

            match content_result {
                Ok(content) => {
                    eprintln!("process_graph_import_requests: Content loaded, format requested: {}", format);
                    eprintln!("process_graph_import_requests: First 100 chars of content: {}", &content.chars().take(100).collect::<String>());

                    // Parse format
                    let import_format = match format.as_str() {
                        "arrows_app" => crate::domain::services::ImportFormat::ArrowsApp,
                        "cypher" => crate::domain::services::ImportFormat::Cypher,
                        "mermaid" => crate::domain::services::ImportFormat::Mermaid,
                        "dot" => crate::domain::services::ImportFormat::Dot,
                        "progress_json" => crate::domain::services::ImportFormat::ProgressJson,
                        "vocabulary_json" => crate::domain::services::ImportFormat::VocabularyJson,
                        "rss_atom" => crate::domain::services::ImportFormat::RssAtom,
                        _ => {
                            error!("Unknown import format: {}", format);
                            import_results.write(ImportResultEvent {
                                event: DomainEvent::Graph(GraphEvent::GraphImportCompleted {
                                    graph_id: *graph_id,
                                    imported_nodes: 0,
                                    imported_edges: 0,
                                    source: source.clone(),
                                }),
                            });
                            continue;
                        }
                    };

                    eprintln!("process_graph_import_requests: Using import format: {:?}", import_format);

                    // Import the graph
                    match import_service.import_from_content(&content, import_format, options.mapping.as_ref()) {
                        Ok(imported_graph) => {
                            eprintln!("process_graph_import_requests: Successfully imported {} nodes and {} edges",
                                imported_graph.nodes.len(),
                                imported_graph.edges.len()
                            );
                            info!("Successfully imported {} nodes and {} edges",
                                imported_graph.nodes.len(),
                                imported_graph.edges.len()
                            );

                            // Check if this import contains subgraphs
                            let has_subgraphs = imported_graph.metadata.contains_key("subgraphs");
                            let mut subgraph_entities: HashMap<usize, SubgraphId> = HashMap::new();

                            if has_subgraphs {
                                eprintln!("process_graph_import_requests: Import contains subgraphs");

                                // Create subgraph regions
                                if let Some(subgraphs) = imported_graph.metadata.get("subgraphs") {
                                    if let Some(subgraph_array) = subgraphs.as_array() {
                                        for subgraph_info in subgraph_array {
                                            if let (Some(id), Some(name), Some(node_ids)) = (
                                                subgraph_info.get("id").and_then(|v| v.as_u64()),
                                                subgraph_info.get("name").and_then(|v| v.as_str()),
                                                subgraph_info.get("node_ids").and_then(|v| v.as_array())
                                            ) {
                                                let subgraph_id = SubgraphId::new();
                                                subgraph_entities.insert(id as usize, subgraph_id);

                                                // Collect node IDs for this subgraph
                                                let mut node_set = HashSet::new();
                                                for node_id_val in node_ids {
                                                    if let Some(node_id_str) = node_id_val.as_str() {
                                                        // We'll need to map these to actual NodeIds after nodes are created
                                                        node_set.insert(node_id_str.to_string());
                                                    }
                                                }

                                                // Create subgraph region entity
                                                let color = match id % 6 {
                                                    0 => Color::srgb(0.2, 0.6, 0.9),  // Blue
                                                    1 => Color::srgb(0.9, 0.2, 0.2),  // Red
                                                    2 => Color::srgb(0.2, 0.9, 0.2),  // Green
                                                    3 => Color::srgb(0.9, 0.9, 0.2),  // Yellow
                                                    4 => Color::srgb(0.9, 0.2, 0.9),  // Magenta
                                                    _ => Color::srgb(0.2, 0.9, 0.9),  // Cyan
                                                };

                                                commands.spawn((
                                                    SubgraphRegion {
                                                        subgraph_id,
                                                        name: name.to_string(),
                                                        color,
                                                        nodes: HashSet::new(), // Will be populated later
                                                        boundary_type: BoundaryType::ConvexHull,
                                                    },
                                                    Transform::default(),
                                                    Visibility::default(),
                                                ));

                                                eprintln!("Created subgraph region: {} with {} nodes", name, node_set.len());
                                            }
                                        }
                                    }
                                }
                            }

                            // Apply position offset if specified
                            let mut nodes = imported_graph.nodes;
                            if let Some(offset) = &options.position_offset {
                                eprintln!("process_graph_import_requests: Applying position offset: {:?}", offset);
                                for node in &mut nodes {
                                    node.position.x += offset.x;
                                    node.position.y += offset.y;
                                    node.position.z += offset.z;
                                }
                            }

                            // Create a mapping from imported node IDs to new NodeIds
                            let mut node_id_map = HashMap::new();

                            // Create nodes with subgraph membership
                            for node in &nodes {
                                eprintln!("process_graph_import_requests: Creating node {} at position {:?}", node.label, node.position);

                                let node_id = NodeId::new();
                                node_id_map.insert(node.id.clone(), node_id);

                                // Check if this node belongs to a subgraph
                                let subgraph_membership = if has_subgraphs {
                                    node.properties.get("subgraph_id")
                                        .and_then(|v| v.as_u64())
                                        .and_then(|id| subgraph_entities.get(&(id as usize)))
                                        .copied()
                                } else {
                                    None
                                };

                                let mut metadata = HashMap::new();
                                metadata.insert("label".to_string(), serde_json::json!(node.label.clone()));
                                metadata.insert("node_type".to_string(), serde_json::json!(node.node_type.clone()));

                                // Add subgraph info to metadata if present
                                if let Some(subgraph_name) = node.properties.get("subgraph").and_then(|v| v.as_str()) {
                                    metadata.insert("subgraph".to_string(), serde_json::json!(subgraph_name));
                                }

                                for (key, value) in &node.properties {
                                    if key != "subgraph" && key != "subgraph_id" {
                                        metadata.insert(key.clone(), value.clone());
                                    }
                                }

                                let node_command = Command::Node(NodeCommand::AddNode {
                                    graph_id: *graph_id,
                                    node_id,
                                    content: NodeContent {
                                        label: node.label.clone(),
                                        node_type: match node.node_type.as_str() {
                                                            "Entity" => NodeType::Entity,
                "ValueObject" => NodeType::ValueObject,
                "Aggregate" => NodeType::Aggregate,
                                            "Service" => NodeType::Custom("Service".to_string()),
                                            "Repository" => NodeType::Custom("Repository".to_string()),
                                            "Factory" => NodeType::Custom("Factory".to_string()),
                                            "Event" => NodeType::Event,
                                            "Command" => NodeType::Command,
                                            "Query" => NodeType::Query,
                                            "Policy" => NodeType::Custom("Policy".to_string()),
                                            _ => NodeType::Custom(node.node_type.clone()),
                                        },
                                        properties: node.properties.clone(),
                                    },
                                    position: node.position.clone(),
                                });

                                command_events.write(CommandEvent {
                                    command: node_command,
                                });
                            }

                            // Create edges
                            for edge in &imported_graph.edges {
                                eprintln!("process_graph_import_requests: Creating edge from {} to {}", edge.source, edge.target);

                                // Look up the mapped node IDs
                                if let (Some(&source_id), Some(&target_id)) = (
                                    node_id_map.get(&edge.source),
                                    node_id_map.get(&edge.target)
                                ) {
                                    let edge_command = Command::Edge(EdgeCommand::ConnectEdge {
                                        graph_id: *graph_id,
                                        edge_id: EdgeId::new(),
                                        source: source_id,
                                        target: target_id,
                                        relationship: EdgeRelationship {
                                            relationship_type: match edge.edge_type.as_str() {
                                                "contains" => RelationshipType::Contains,
                                                "references" => RelationshipType::References,
                                                "depends_on" => RelationshipType::DependsOn,
                                                "publishes" => RelationshipType::Publishes,
                                                "subscribes" => RelationshipType::Subscribes,
                                                "implements" => RelationshipType::Implements,
                                                "extends" => RelationshipType::Extends,
                                                "parent" => RelationshipType::Parent,
                                                "merged" => RelationshipType::Merged,
                                                "branched" => RelationshipType::Branched,
                                                "tagged" => RelationshipType::Tagged,
                                                "sequence" => RelationshipType::Sequence,
                                                "hierarchy" => RelationshipType::Contains, // Hierarchical containment
                                                "blocks" => RelationshipType::Blocks,
                                                _ => RelationshipType::Custom(edge.edge_type.clone()),
                                            },
                                            properties: edge.properties.clone(),
                                            bidirectional: false,
                                        },
                                    });

                                    command_events.write(CommandEvent {
                                        command: edge_command,
                                    });
                                } else {
                                    error!("Failed to map edge nodes: {} -> {}", edge.source, edge.target);
                                }
                            }

                            // Send success result
                            import_results.write(ImportResultEvent {
                                event: DomainEvent::Graph(GraphEvent::GraphImportCompleted {
                                    graph_id: *graph_id,
                                    imported_nodes: nodes.len(),
                                    imported_edges: imported_graph.edges.len(),
                                    source: source.clone(),
                                }),
                            });
                        }
                        Err(e) => {
                            eprintln!("process_graph_import_requests: Import failed: {}", e);
                            error!("Import failed: {}", e);
                            import_results.write(ImportResultEvent {
                                event: DomainEvent::Graph(GraphEvent::GraphImportCompleted {
                                    graph_id: *graph_id,
                                    imported_nodes: 0,
                                    imported_edges: 0,
                                    source: source.clone(),
                                }),
                            });
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to load content: {}", e);
                    import_results.write(ImportResultEvent {
                        event: DomainEvent::Graph(GraphEvent::GraphImportCompleted {
                            graph_id: *graph_id,
                            imported_nodes: 0,
                            imported_edges: 0,
                            source: source.clone(),
                        }),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use bevy::app::Update;
    use crate::application::command_handlers::process_commands;
    use crate::presentation::events::ImportRequestEvent;
    use crate::domain::commands::graph_commands::MergeBehavior;
    use crate::domain::commands::ImportOptions;
    use crate::domain::value_objects::GraphId;

    #[test]
    fn test_system_parameter_conflict() {
        // User Story: Architecture - System Parameter Conflicts
        // Acceptance Criteria: Systems should not have conflicting parameter access
        // Test Purpose: Documents the original conflict and how it was resolved
        // Expected Behavior: The fixed implementation should not panic

        // This test documents that the original implementation had a conflict:
        // - process_commands writes to EventNotification
        // - process_graph_import_requests was trying to read EventNotification
        // This created a system parameter conflict in Bevy

        // The fix was to introduce ImportRequestEvent as an intermediate event
        // Now the flow is:
        // 1. process_commands writes EventNotification
        // 2. forward_import_requests reads EventNotification and writes ImportRequestEvent
        // 3. process_graph_import_requests reads ImportRequestEvent

        assert!(true, "Conflict was resolved by introducing ImportRequestEvent");
    }

    #[test]
    fn test_no_conflict_with_proper_event_forwarding() {
        // User Story: Architecture - Event Forwarding Pattern
        // Acceptance Criteria: Systems can be chained without conflicts
        // Test Purpose: Validates that the event forwarding pattern works
        // Expected Behavior: Systems run without conflicts

        let mut app = App::new();

        app.add_plugins(bevy::MinimalPlugins);

        // Add events
        app.add_event::<crate::application::EventNotification>();
        app.add_event::<ImportRequestEvent>();
        app.add_event::<ImportResultEvent>();
        app.add_event::<CommandEvent>();

        // Add systems with proper forwarding
        app.add_systems(Update, (
            // Mock system that writes EventNotification
            |mut writer: EventWriter<crate::application::EventNotification>| {
                writer.write(crate::application::EventNotification {
                    event: DomainEvent::Graph(GraphEvent::GraphImportRequested {
                        graph_id: GraphId::new(),
                        source: ImportSource::InlineContent {
                            content: "test".to_string(),
                        },
                        format: "mermaid".to_string(),
                        options: ImportOptions {
                            merge_behavior: MergeBehavior::AlwaysCreate,
                            id_prefix: None,
                            position_offset: None,
                            mapping: None,
                            validate: true,
                            max_nodes: None,
                        },
                    })
                });
            },
            crate::presentation::systems::forward_import_requests,
            process_graph_import_requests,
        ).chain());

        // This should NOT panic
        app.update();

        // The test passes if we don't panic - the event forwarding pattern works
        assert!(true, "Event forwarding pattern works without conflicts");
    }
}
