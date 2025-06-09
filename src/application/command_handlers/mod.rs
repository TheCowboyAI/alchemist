//! Command Handlers - Process commands and generate events

use crate::application::{CommandEvent, EventNotification};
use crate::domain::commands::{Command, EdgeCommand, GraphCommand, NodeCommand};
use crate::domain::events::{DomainEvent, EdgeEvent, GraphEvent, NodeEvent};
use crate::domain::value_objects::{GraphId, NodeId, EdgeId, SubgraphId, Position3D, GraphMetadata};
use crate::domain::aggregates::graph::Graph;
use crate::infrastructure::event_bridge::{EventBridge, BridgeCommand};
use bevy::prelude::*;
use serde_json;
use tracing::{error, warn, info};
use std::collections::HashMap;

// Async command handlers for integration with event store
pub mod graph_command_handler;
pub mod workflow_command_handler;
pub mod graph_import_handler;
pub mod metric_context_handler;
pub mod rule_context_handler;

pub use graph_command_handler::{GraphCommandHandler, CommandHandler};
pub use workflow_command_handler::WorkflowCommandHandler;
pub use graph_import_handler::GraphImportHandler;
pub use metric_context_handler::MetricContextHandler;
pub use rule_context_handler::RuleContextHandler;

/// System that processes commands and generates events
pub fn process_commands(
    mut commands: EventReader<CommandEvent>,
    mut events: EventWriter<EventNotification>,
    event_bridge: Res<EventBridge>,
) {
    for command_event in commands.read() {
        tracing::info!("Processing command: {:?}", command_event.command.command_type());

        match &command_event.command {
            Command::Graph(graph_cmd) => {
                tracing::info!("Processing graph command: {:?}", graph_cmd.command_type());

                // Use the handle_graph_command function
                if let Some(event) = handle_graph_command(graph_cmd) {
                    if let Err(e) = event_bridge.send_command(BridgeCommand::PublishEvent(event.clone())) {
                        error!("Failed to send event: {}", e);
                    }

                    events.write(EventNotification { event });
                }
            }
            Command::Node(node_cmd) => {
                // Process node commands
                if let Some(event) = handle_node_command(node_cmd) {
                    if let Err(e) = event_bridge.send_command(BridgeCommand::PublishEvent(event.clone())) {
                        error!("Failed to send event: {}", e);
                    }
                    events.write(EventNotification { event });
                }
            }
            Command::Edge(edge_cmd) => {
                // Process edge commands
                if let Some(event) = handle_edge_command(edge_cmd) {
                    if let Err(e) = event_bridge.send_command(BridgeCommand::PublishEvent(event.clone())) {
                        error!("Failed to send event: {}", e);
                    }
                    events.write(EventNotification { event });
                }
            }
            Command::Subgraph(_) => {
                warn!("Subgraph commands not yet implemented");
                Ok(vec![])
            }
            Command::SubgraphOperation(_) => {
                warn!("SubgraphOperation commands not yet implemented");
                Ok(vec![])
            }
            Command::ContextBridge(_) => {
                warn!("ContextBridge commands should be handled by async handler");
                // ContextBridge commands are handled by the async ContextBridgeHandler
                Ok(vec![])
            }
            Command::MetricContext(_) => {
                warn!("MetricContext commands should be handled by async handler");
                // MetricContext commands are handled by the async MetricContextHandler
            }
            Command::RuleContext(_) => {
                warn!("RuleContext commands should be handled by async handler");
                // RuleContext commands are handled by the async RuleContextHandler
            }
        }
    }
}

/// Handle graph commands and generate events
fn handle_graph_command(command: &GraphCommand) -> Option<DomainEvent> {
    match command {
        GraphCommand::CreateGraph { id, name, metadata: _ } => {
            Some(DomainEvent::Graph(GraphEvent::GraphCreated {
                id: *id,
                metadata: GraphMetadata::new(name.clone()),
            }))
        }
        GraphCommand::RenameGraph { id, new_name } => {
            Some(DomainEvent::Graph(GraphEvent::GraphRenamed {
                id: *id,
                old_name: String::new(), // TODO: Get from aggregate
                new_name: new_name.clone(),
            }))
        }
        GraphCommand::TagGraph { id, tag } => Some(DomainEvent::Graph(GraphEvent::GraphTagged {
            id: *id,
            tag: tag.clone(),
        })),
        GraphCommand::UntagGraph { id, tag } => {
            Some(DomainEvent::Graph(GraphEvent::GraphUntagged {
                id: *id,
                tag: tag.clone(),
            }))
        }
        GraphCommand::DeleteGraph { id } => {
            Some(DomainEvent::Graph(GraphEvent::GraphDeleted { id: *id }))
        }
        GraphCommand::UpdateGraph { id, name, description } => {
            Some(DomainEvent::Graph(GraphEvent::GraphUpdated {
                graph_id: *id,
                name: name.clone(),
                description: description.clone(),
            }))
        }
        GraphCommand::ClearGraph { .. } => {
            // ClearGraph is handled by the aggregate - it generates NodeRemoved and EdgeRemoved events
            None
        }
        GraphCommand::ImportGraph { graph_id, source, format, options } => {
            // For now, emit the GraphImportRequested event
            // The process_graph_import_requests system will handle the actual import
            Some(DomainEvent::Graph(GraphEvent::GraphImportRequested {
                graph_id: *graph_id,
                source: source.clone(),
                format: format.clone(), // Use the actual format from the command
                options: options.clone(),
            }))
        }
        GraphCommand::ImportFromFile { graph_id, file_path, format } => {
            // Convert to ImportGraph with File source
            Some(DomainEvent::Graph(GraphEvent::GraphImportRequested {
                graph_id: *graph_id,
                source: crate::domain::commands::ImportSource::File {
                    path: file_path.clone()
                },
                format: format.clone(),
                options: crate::domain::commands::ImportOptions::default(),
            }))
        }
        GraphCommand::ImportFromUrl { graph_id, url, format } => {
            // Convert to ImportGraph with URL source
            Some(DomainEvent::Graph(GraphEvent::GraphImportRequested {
                graph_id: *graph_id,
                source: crate::domain::commands::ImportSource::Url {
                    url: url.clone()
                },
                format: format.clone(),
                options: crate::domain::commands::ImportOptions::default(),
            }))
        }
        GraphCommand::AddNode { .. } |
        GraphCommand::UpdateNode { .. } |
        GraphCommand::RemoveNode { .. } |
        GraphCommand::ConnectNodes { .. } |
        GraphCommand::DisconnectNodes { .. } |
        GraphCommand::UpdateEdge { .. } => {
            // These are handled by the aggregate
            None
        }
        // Conceptual graph commands - handled by the aggregate
        GraphCommand::CreateConceptualGraph { .. } |
        GraphCommand::AddConceptualNode { .. } |
        GraphCommand::ApplyGraphMorphism { .. } |
        GraphCommand::ComposeConceptualGraphs { .. } => {
            // These are handled by the conceptual graph aggregate
            None
        }
    }
}

/// Handle node commands and generate events
fn handle_node_command(command: &NodeCommand) -> Option<DomainEvent> {
    match command {
        NodeCommand::AddNode {
            graph_id,
            node_id,
            content,
            position,
        } => {
            // Convert NodeContent to metadata HashMap
            let mut metadata = content.properties.clone();
            metadata.insert("label".to_string(), serde_json::Value::String(content.label.clone()));
            metadata.insert("node_type".to_string(), serde_json::to_value(&content.node_type).unwrap());

            Some(DomainEvent::Node(NodeEvent::NodeAdded {
                graph_id: *graph_id,
                node_id: *node_id,
                metadata,
                position: *position,
            }))
        }
        NodeCommand::RemoveNode { graph_id, node_id } => {
            Some(DomainEvent::Node(NodeEvent::NodeRemoved {
                graph_id: *graph_id,
                node_id: *node_id,
            }))
        }
        NodeCommand::UpdateNode {
            graph_id,
            node_id,
            content,
        } => {
            // Convert to content update event
            Some(DomainEvent::Node(NodeEvent::NodeContentChanged {
                graph_id: *graph_id,
                node_id: *node_id,
                old_content: serde_json::Value::Null, // TODO: Get from aggregate
                new_content: serde_json::to_value(content).unwrap(),
            }))
        }
        NodeCommand::MoveNode {
            graph_id,
            node_id,
            position,
        } => {
            // Create a move event
            Some(DomainEvent::Node(NodeEvent::NodeMoved {
                graph_id: *graph_id,
                node_id: *node_id,
                old_position: Position3D::default(), // TODO: Get from aggregate
                new_position: *position,
            }))
        }
        NodeCommand::SelectNode { .. } | NodeCommand::DeselectNode { .. } => {
            // Selection is a presentation concern, not a domain event
            None
        }
    }
}

/// Handle edge commands and generate events
fn handle_edge_command(command: &EdgeCommand) -> Option<DomainEvent> {
    match command {
        EdgeCommand::ConnectEdge {
            graph_id,
            edge_id,
            source,
            target,
            relationship,
        } => Some(DomainEvent::Edge(EdgeEvent::EdgeConnected {
            graph_id: *graph_id,
            edge_id: *edge_id,
            source: *source,
            target: *target,
            relationship: relationship.relationship_type.to_string(),
        })),
        EdgeCommand::DisconnectEdge { graph_id, edge_id } => {
            Some(DomainEvent::Edge(EdgeEvent::EdgeRemoved {
                graph_id: *graph_id,
                edge_id: *edge_id,
            }))
        }
        EdgeCommand::SelectEdge { .. } | EdgeCommand::DeselectEdge { .. } => {
            // Selection is a presentation concern, not a domain event
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::commands::{ImportSource, ImportOptions};
    use crate::domain::commands::graph_commands::MergeBehavior;

    #[test]
    fn test_import_graph_command_generates_event() {
        // Test that ImportGraph commands now generate GraphImportRequested events
        let graph_id = GraphId::new();
        let cmd = GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::InlineContent {
                content: r#"{"nodes": [], "relationships": []}"#.to_string(),
            },
            format: "arrows_app".to_string(),
            options: ImportOptions {
                merge_behavior: MergeBehavior::AlwaysCreate,
                id_prefix: None,
                position_offset: None,
                mapping: None,
                validate: true,
                max_nodes: None,
            },
        };

        let result = handle_graph_command(&cmd);
        assert!(result.is_some(), "ImportGraph should now be handled");

        if let Some(DomainEvent::Graph(GraphEvent::GraphImportRequested { .. })) = result {
            // Success - the command generates the expected event
        } else {
            panic!("Expected GraphImportRequested event");
        }
    }
}
