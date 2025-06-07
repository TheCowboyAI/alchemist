//! Command Handlers - Process commands and generate events

use crate::application::{CommandEvent, EventNotification};
use crate::domain::commands::{Command, EdgeCommand, GraphCommand, NodeCommand};
use crate::domain::events::{DomainEvent, EdgeEvent, GraphEvent, NodeEvent};
use crate::domain::value_objects::*;
use bevy::prelude::*;
use serde_json;

// Async command handlers for integration with event store
pub mod graph_command_handler;
pub mod workflow_command_handler;
pub mod graph_import_handler;

pub use graph_command_handler::{GraphCommandHandler, CommandHandler};
pub use workflow_command_handler::WorkflowCommandHandler;
pub use graph_import_handler::GraphImportHandler;

/// System that processes commands and generates events
pub fn process_commands(
    mut commands: EventReader<CommandEvent>,
    mut events: EventWriter<EventNotification>,
) {
    for command_event in commands.read() {
        match &command_event.command {
            Command::Graph(graph_cmd) => {
                // Process graph commands
                if let Some(event) = handle_graph_command(graph_cmd) {
                    events.write(EventNotification { event });
                }
            }
            Command::Node(node_cmd) => {
                // Process node commands
                if let Some(event) = handle_node_command(node_cmd) {
                    events.write(EventNotification { event });
                }
            }
            Command::Edge(edge_cmd) => {
                // Process edge commands
                if let Some(event) = handle_edge_command(edge_cmd) {
                    events.write(EventNotification { event });
                }
            }
            Command::Workflow(_) => {
                // Handle workflow commands
                tracing::info!("Processing workflow command");
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
        GraphCommand::ImportGraph { graph_id, source, format: _, options } => {
            // For now, emit the GraphImportRequested event
            // The process_graph_import_requests system will handle the actual import
            Some(DomainEvent::Graph(GraphEvent::GraphImportRequested {
                graph_id: *graph_id,
                source: source.clone(),
                format: "arrows_app".to_string(), // TODO: Use the actual format
                options: options.clone(),
            }))
        }
        GraphCommand::AddNode { .. } |
        GraphCommand::UpdateNode { .. } |
        GraphCommand::RemoveNode { .. } |
        GraphCommand::ConnectNodes { .. } |
        GraphCommand::DisconnectNodes { .. } |
        GraphCommand::UpdateEdge { .. } |
        GraphCommand::ImportFromFile { .. } |
        GraphCommand::ImportFromUrl { .. } => {
            // These are handled by the aggregate
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
