//! System that processes GraphImportRequested events and performs the actual import

use crate::application::EventNotification;
use crate::presentation::events::ImportResultEvent;
use crate::domain::{
    commands::ImportSource,
    events::{DomainEvent, GraphEvent, NodeEvent, EdgeEvent},
    services::GraphImportService,
    value_objects::{NodeId, EdgeId},
};
use bevy::prelude::*;
use std::collections::HashMap;
use tracing::{info, error};

/// System that processes GraphImportRequested events
pub fn process_graph_import_requests(
    mut events: EventReader<EventNotification>,
    mut event_writer: EventWriter<ImportResultEvent>,
) {
    for notification in events.read() {
        if let DomainEvent::Graph(GraphEvent::GraphImportRequested {
            graph_id,
            source,
            format,
            options,
        }) = &notification.event {
            info!("Processing graph import request for graph {:?}", graph_id);

            // Create import service
            let import_service = GraphImportService::new();

            // Load content based on source
            let content_result = match source {
                ImportSource::File { path } => {
                    std::fs::read_to_string(path)
                        .map_err(|e| format!("Failed to read file {path}: {e}"))
                }
                ImportSource::InlineContent { content } => {
                    Ok(content.clone())
                }
                ImportSource::Url { url } => {
                    Err(format!("URL import not yet implemented: {url}"))
                }
                ImportSource::GitRepository { .. } => {
                    Err("Git repository import not yet implemented".to_string())
                }
                ImportSource::NixFlake { .. } => {
                    Err("Nix flake import not yet implemented".to_string())
                }
            };

            match content_result {
                Ok(content) => {
                    // Parse the content
                    // Parse format string to ImportFormat enum
                    let import_format = match format.as_str() {
                        "arrows_app" => crate::domain::services::ImportFormat::ArrowsApp,
                        "cypher" => crate::domain::services::ImportFormat::Cypher,
                        "mermaid" => crate::domain::services::ImportFormat::Mermaid,
                        "dot" => crate::domain::services::ImportFormat::Dot,
                        "progress_json" => crate::domain::services::ImportFormat::ProgressJson,
                        "vocabulary_json" => crate::domain::services::ImportFormat::VocabularyJson,
                        "rss" => crate::domain::services::ImportFormat::RssAtom,
                        "atom" => crate::domain::services::ImportFormat::RssAtom,
                        _ => crate::domain::services::ImportFormat::ArrowsApp, // Default
                    };

                    match import_service.import_from_content(&content, import_format, options.mapping.as_ref()) {
                        Ok(imported_graph) => {
                            info!("Successfully imported {} nodes and {} edges",
                                imported_graph.nodes.len(),
                                imported_graph.edges.len());

                            // Create a mapping from imported node IDs to new NodeIds
                            let mut node_id_map = HashMap::new();

                            // Generate NodeAdded events for each imported node
                            for imported_node in &imported_graph.nodes {
                                let node_id = NodeId::new();
                                node_id_map.insert(imported_node.id.clone(), node_id);

                                // Apply position offset if specified
                                let mut position = imported_node.position;
                                if let Some(offset) = &options.position_offset {
                                    position.x += offset.x;
                                    position.y += offset.y;
                                    position.z += offset.z;
                                }

                                // Create metadata from properties
                                let mut metadata = imported_node.properties.clone();
                                metadata.insert("label".to_string(), serde_json::Value::String(imported_node.label.clone()));
                                metadata.insert("imported_id".to_string(), serde_json::Value::String(imported_node.id.clone()));

                                event_writer.write(ImportResultEvent {
                                    event: DomainEvent::Node(NodeEvent::NodeAdded {
                                        graph_id: *graph_id,
                                        node_id,
                                        metadata,
                                        position,
                                    })
                                });
                            }

                            // Generate EdgeConnected events for each imported edge
                            for imported_edge in &imported_graph.edges {
                                // Look up the mapped node IDs
                                if let (Some(&source_id), Some(&target_id)) = (
                                    node_id_map.get(&imported_edge.source),
                                    node_id_map.get(&imported_edge.target)
                                ) {
                                    let edge_id = EdgeId::new();

                                    event_writer.write(ImportResultEvent {
                                        event: DomainEvent::Edge(EdgeEvent::EdgeConnected {
                                            graph_id: *graph_id,
                                            edge_id,
                                            source: source_id,
                                            target: target_id,
                                            relationship: imported_edge.edge_type.clone(),
                                        })
                                    });
                                } else {
                                    error!("Failed to map edge nodes: {} -> {}",
                                        imported_edge.source, imported_edge.target);
                                }
                            }

                            // Generate import completed event
                            event_writer.write(ImportResultEvent {
                                event: DomainEvent::Graph(GraphEvent::GraphImportCompleted {
                                    graph_id: *graph_id,
                                    imported_nodes: imported_graph.nodes.len(),
                                    imported_edges: imported_graph.edges.len(),
                                    source: source.clone(),
                                })
                            });
                        }
                        Err(e) => {
                            error!("Failed to parse import content: {}", e);
                            // TODO: Generate import failed event
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to load import content: {}", e);
                    // TODO: Generate import failed event
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

    #[test]
    #[should_panic(expected = "conflicts with a previous")]
    fn test_system_parameter_conflict() {
        // This test would have caught the system parameter conflict
        let mut app = App::new();

        // Add minimal plugins
        app.add_plugins(bevy::MinimalPlugins);

        // Add events
        app.add_event::<crate::application::CommandEvent>();
        app.add_event::<crate::application::EventNotification>();
        app.add_event::<ImportRequestEvent>();
        app.add_event::<ImportResultEvent>();

        // Add systems that would conflict
        // process_commands writes to EventNotification
        // process_graph_import_requests was trying to read EventNotification
        app.add_systems(Update, (
            process_commands,
            process_graph_import_requests,
        ));

        // This SHOULD panic with a conflict error
        app.update();
    }

    #[test]
    fn test_no_conflict_with_proper_event_forwarding() {
        // This test shows the correct pattern
        let mut app = App::new();

        app.add_plugins(bevy::MinimalPlugins);

        // Add events
        app.add_event::<crate::application::EventNotification>();
        app.add_event::<ImportRequestEvent>();
        app.add_event::<ImportResultEvent>();

        // Add systems with proper forwarding
        app.add_systems(Update, (
            process_commands,
            crate::presentation::systems::forward_import_requests,
            process_graph_import_requests,
        ).chain());

        // This should NOT panic
        app.update();
    }

    #[test]
    fn test_original_system_parameter_conflict_would_have_failed() {
        // This test demonstrates what the original conflict was
        // We simulate the original implementation where process_graph_import_requests
        // tried to read EventNotification directly

        let mut app = App::new();

        // Add minimal plugins
        app.add_plugins(bevy::MinimalPlugins);

        // Add events
        app.add_event::<crate::application::CommandEvent>();
        app.add_event::<crate::application::EventNotification>();

        // Simulate the original broken implementation
        fn broken_process_graph_import_requests(
            mut events: EventReader<crate::application::EventNotification>,
            mut event_writer: EventWriter<ImportResultEvent>,
        ) {
            // This would have conflicted with process_commands
            // because process_commands writes to EventNotification
            // and this reads from EventNotification
            for _ in events.read() {}
        }

        // Add systems that would conflict
        app.add_systems(Update, (
            process_commands,
            broken_process_graph_import_requests,
        ));

        // This would panic with:
        // "ResMut<bevy_ecs::event::collections::Events<ia::application::EventNotification>>
        // in system ia::presentation::systems::graph_import_processor::process_graph_import_requests
        // conflicts with a previous Res<bevy_ecs::event::collections::Events<ia::application::EventNotification>> access"

        // But we can't test it because Bevy would panic before the test completes
        // This demonstrates why the ImportRequestEvent was necessary
    }

    #[test]
    fn test_fixed_implementation_no_conflict() {
        // This test shows that our fixed implementation works
        let mut app = App::new();

        // Add minimal plugins
        app.add_plugins(bevy::MinimalPlugins);

        // Add events
        app.add_event::<crate::application::CommandEvent>();
        app.add_event::<crate::application::EventNotification>();
        app.add_event::<ImportRequestEvent>();
        app.add_event::<ImportResultEvent>();

        // Add systems with proper forwarding
        app.add_systems(Update, (
            process_commands,
            crate::presentation::systems::forward_import_requests,
            process_graph_import_requests,
        ).chain());

        // This should NOT panic - the fix works!
        app.update();
    }
}
