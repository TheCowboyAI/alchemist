//! Value Object Change Patterns for DDD and Event Sourcing
//!
//! This module demonstrates the proper patterns for handling changes to value objects
//! in a Domain-Driven Design with Event Sourcing architecture.

use crate::domain::commands::{EdgeCommand, NodeCommand};
use crate::domain::events::{DomainEvent, EdgeEvent, NodeEvent};
use crate::domain::value_objects::*;

/// Helper functions for changing value objects according to DDD principles
pub struct ValueObjectChangePatterns;

impl ValueObjectChangePatterns {
    /// Change a node's position (value object) by emitting remove/add events
    ///
    /// # Example
    /// ```ignore
    /// let events = ValueObjectChangePatterns::change_node_position(
    ///     graph_id,
    ///     node_id,
    ///     old_position,
    ///     new_position,
    ///     metadata,
    /// );
    /// ```
    pub fn change_node_position(
        graph_id: GraphId,
        node_id: NodeId,
        _old_position: Position3D, // Not used in events, but kept for clarity
        new_position: Position3D,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Vec<DomainEvent> {
        vec![
            // First, remove the node at its old position
            DomainEvent::Node(NodeEvent::NodeRemoved { graph_id, node_id }),
            // Then, add it back at the new position
            DomainEvent::Node(NodeEvent::NodeAdded {
                graph_id,
                node_id,
                position: new_position,
                metadata,
            }),
        ]
    }

    /// Change an edge's relationship (value object) by emitting disconnect/connect events
    ///
    /// # Example
    /// ```ignore
    /// let events = ValueObjectChangePatterns::change_edge_relationship(
    ///     graph_id,
    ///     old_edge_id,
    ///     source,
    ///     target,
    ///     new_relationship,
    /// );
    /// ```
    pub fn change_edge_relationship(
        graph_id: GraphId,
        old_edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        new_relationship: EdgeRelationship,
    ) -> Vec<DomainEvent> {
        vec![
            // First, disconnect the old edge
            DomainEvent::Edge(EdgeEvent::EdgeDisconnected {
                graph_id,
                edge_id: old_edge_id,
                source,
                target,
            }),
            // Then, connect a new edge with the new relationship
            DomainEvent::Edge(EdgeEvent::EdgeConnected {
                graph_id,
                edge_id: EdgeId::new(), // New edge ID for the new value object
                source,
                target,
                relationship: new_relationship,
            }),
        ]
    }

    /// Convert a "move node" command into proper remove/add commands
    ///
    /// This demonstrates how to handle commands that imply value object changes
    pub fn convert_move_node_command(
        graph_id: GraphId,
        node_id: NodeId,
        new_position: Position3D,
    ) -> Vec<NodeCommand> {
        vec![
            NodeCommand::RemoveNode { graph_id, node_id },
            NodeCommand::AddNode {
                graph_id,
                node_id,
                content: NodeContent {
                    label: String::new(), // Would be retrieved from aggregate
                    node_type: NodeType::Entity, // Would be retrieved from aggregate
                    properties: std::collections::HashMap::new(), // Would be retrieved from aggregate
                },
                position: new_position,
            },
        ]
    }

    /// Convert an "update edge" command into proper disconnect/connect commands
    pub fn convert_update_edge_command(
        graph_id: GraphId,
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        new_relationship: EdgeRelationship,
    ) -> Vec<EdgeCommand> {
        vec![
            EdgeCommand::DisconnectEdge { graph_id, edge_id },
            EdgeCommand::ConnectEdge {
                graph_id,
                edge_id: EdgeId::new(), // New edge for the new relationship
                source,
                target,
                relationship: new_relationship,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_position_change_generates_correct_events() {
        let graph_id = GraphId::new();
        let node_id = NodeId::new();
        let old_position = Position3D::new(0.0, 0.0, 0.0);
        let new_position = Position3D::new(10.0, 20.0, 30.0);
        let metadata = std::collections::HashMap::new();

        let events = ValueObjectChangePatterns::change_node_position(
            graph_id,
            node_id,
            old_position,
            new_position,
            metadata.clone(),
        );

        assert_eq!(events.len(), 2);

        // First event should be removal
        match &events[0] {
            DomainEvent::Node(NodeEvent::NodeRemoved { graph_id: g, node_id: n }) => {
                assert_eq!(*g, graph_id);
                assert_eq!(*n, node_id);
            }
            _ => panic!("Expected NodeRemoved event"),
        }

        // Second event should be addition with new position
        match &events[1] {
            DomainEvent::Node(NodeEvent::NodeAdded {
                graph_id: g,
                node_id: n,
                position: p,
                metadata: m,
            }) => {
                assert_eq!(*g, graph_id);
                assert_eq!(*n, node_id);
                assert_eq!(*p, new_position);
                assert_eq!(*m, metadata);
            }
            _ => panic!("Expected NodeAdded event"),
        }
    }

    #[test]
    fn test_edge_relationship_change_generates_correct_events() {
        let graph_id = GraphId::new();
        let old_edge_id = EdgeId::new();
        let source = NodeId::new();
        let target = NodeId::new();
        let new_relationship = EdgeRelationship {
            relationship_type: RelationshipType::DependsOn,
            properties: std::collections::HashMap::new(),
            bidirectional: false,
        };

        let events = ValueObjectChangePatterns::change_edge_relationship(
            graph_id,
            old_edge_id,
            source,
            target,
            new_relationship.clone(),
        );

        assert_eq!(events.len(), 2);

        // First event should be disconnection
        match &events[0] {
            DomainEvent::Edge(EdgeEvent::EdgeDisconnected {
                graph_id: g,
                edge_id: e,
                source: s,
                target: t,
            }) => {
                assert_eq!(*g, graph_id);
                assert_eq!(*e, old_edge_id);
                assert_eq!(*s, source);
                assert_eq!(*t, target);
            }
            _ => panic!("Expected EdgeDisconnected event"),
        }

        // Second event should be connection with new relationship
        match &events[1] {
            DomainEvent::Edge(EdgeEvent::EdgeConnected {
                graph_id: g,
                edge_id: e,
                source: s,
                target: t,
                relationship: r,
            }) => {
                assert_eq!(*g, graph_id);
                assert_ne!(*e, old_edge_id); // Should be a new edge ID
                assert_eq!(*s, source);
                assert_eq!(*t, target);
                assert_eq!(*r, new_relationship);
            }
            _ => panic!("Expected EdgeConnected event"),
        }
    }
}
