//! Command Handlers - Process commands and generate events

use crate::application::{CommandEvent, EventNotification};
use crate::domain::commands::{Command, EdgeCommand, GraphCommand, NodeCommand};
use crate::domain::events::{DomainEvent, EdgeEvent, GraphEvent, NodeEvent};
use crate::domain::value_objects::*;
use bevy::prelude::*;
use serde_json;

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
        }
    }
}

/// Handle graph commands and generate events
fn handle_graph_command(command: &GraphCommand) -> Option<DomainEvent> {
    match command {
        GraphCommand::CreateGraph { id, name } => {
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
            // Convert to metadata update events
            let mut events = Vec::new();

            // Update label
            events.push(DomainEvent::Node(NodeEvent::NodeMetadataUpdated {
                graph_id: *graph_id,
                node_id: *node_id,
                key: "label".to_string(),
                old_value: None, // TODO: Get from aggregate
                new_value: Some(serde_json::Value::String(content.label.clone())),
            }));

            // Update node_type
            events.push(DomainEvent::Node(NodeEvent::NodeMetadataUpdated {
                graph_id: *graph_id,
                node_id: *node_id,
                key: "node_type".to_string(),
                old_value: None, // TODO: Get from aggregate
                new_value: Some(serde_json::to_value(&content.node_type).unwrap()),
            }));

            // Update other properties
            for (key, value) in &content.properties {
                events.push(DomainEvent::Node(NodeEvent::NodeMetadataUpdated {
                    graph_id: *graph_id,
                    node_id: *node_id,
                    key: key.clone(),
                    old_value: None, // TODO: Get from aggregate
                    new_value: Some(value.clone()),
                }));
            }

            // For now, just return the first event (we'll need to handle multiple events later)
            events.into_iter().next()
        }
        NodeCommand::MoveNode {
            graph_id,
            node_id,
            position,
        } => {
            Some(DomainEvent::Node(NodeEvent::NodeMoved {
                graph_id: *graph_id,
                node_id: *node_id,
                old_position: *position, // TODO: Get from aggregate
                new_position: *position,
            }))
        }
        NodeCommand::SelectNode { graph_id, node_id } => {
            Some(DomainEvent::Node(NodeEvent::NodeSelected {
                graph_id: *graph_id,
                node_id: *node_id,
            }))
        }
        NodeCommand::DeselectNode { graph_id, node_id } => {
            Some(DomainEvent::Node(NodeEvent::NodeDeselected {
                graph_id: *graph_id,
                node_id: *node_id,
            }))
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
            relationship: relationship.clone(),
        })),
        EdgeCommand::DisconnectEdge { graph_id, edge_id } => {
            Some(DomainEvent::Edge(EdgeEvent::EdgeDisconnected {
                graph_id: *graph_id,
                edge_id: *edge_id,
                source: NodeId::default(), // TODO: Get from aggregate
                target: NodeId::default(), // TODO: Get from aggregate
            }))
        }
        EdgeCommand::UpdateEdge {
            graph_id,
            edge_id,
            relationship,
        } => {
            Some(DomainEvent::Edge(EdgeEvent::EdgeUpdated {
                graph_id: *graph_id,
                edge_id: *edge_id,
                old_relationship: relationship.clone(), // TODO: Get from aggregate
                new_relationship: relationship.clone(),
            }))
        }
        EdgeCommand::SelectEdge { graph_id, edge_id } => {
            Some(DomainEvent::Edge(EdgeEvent::EdgeSelected {
                graph_id: *graph_id,
                edge_id: *edge_id,
            }))
        }
        EdgeCommand::DeselectEdge { graph_id, edge_id } => {
            Some(DomainEvent::Edge(EdgeEvent::EdgeDeselected {
                graph_id: *graph_id,
                edge_id: *edge_id,
            }))
        }
    }
}
