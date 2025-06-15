//! Comprehensive tests for Graph Aggregate

use ia::domain::{
    aggregates::graph::*,
    commands::{EdgeCommand, GraphCommand, NodeCommand},
    events::{DomainEvent, EdgeEvent, GraphEvent, NodeEvent},
    value_objects::*,
};
use std::collections::HashMap;

#[test]
fn test_graph_creation() {
    // Given
    let id = GraphId::new();
    let name = "Test Graph".to_string();
    let description = "Test Description".to_string();

    // When
    let graph = Graph::new(id, name.clone(), Some(description.clone()));

    // Then
    assert_eq!(graph.id, id);
    assert_eq!(graph.metadata.name, name);
    assert_eq!(graph.version, 0);
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);

    // Verify creation event was emitted
    let events = graph.get_uncommitted_events();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Graph(GraphEvent::GraphCreated {
            id: event_id,
            metadata,
        }) => {
            assert_eq!(*event_id, id);
            assert_eq!(metadata.name, name);
            assert!(metadata.tags.contains(&description));
        }
        _ => panic!("Expected GraphCreated event"),
    }
}

#[test]
fn test_handle_rename_graph_command() {
    // Given
    let id = GraphId::new();
    let mut graph = Graph::new(id, "Original".to_string(), None);
    graph.mark_events_as_committed();

    // When
    let command = Command::Graph(GraphCommand::RenameGraph {
        id,
        new_name: "Renamed".to_string(),
    });
    let result = graph.handle_command(command);

    // Then
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Graph(GraphEvent::GraphRenamed {
            id: _,
            old_name,
            new_name,
        }) => {
            assert_eq!(old_name, "Original");
            assert_eq!(new_name, "Renamed");
        }
        _ => panic!("Expected GraphRenamed event"),
    }
}

#[test]
fn test_handle_tag_graph_command() {
    // Given
    let id = GraphId::new();
    let mut graph = Graph::new(id, "Graph".to_string(), None);
    graph.mark_events_as_committed();

    // When
    let command = Command::Graph(GraphCommand::TagGraph {
        id,
        tag: "Important".to_string(),
    });
    let result = graph.handle_command(command);

    // Then
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Graph(GraphEvent::GraphTagged { id: _, tag }) => {
            assert_eq!(tag, "Important");
        }
        _ => panic!("Expected GraphTagged event"),
    }
}

#[test]
fn test_handle_untag_graph_command() {
    // Given
    let id = GraphId::new();
    let mut graph = Graph::new(id, "Graph".to_string(), Some("ToRemove".to_string()));
    graph.mark_events_as_committed();

    // When
    let command = Command::Graph(GraphCommand::UntagGraph {
        id,
        tag: "ToRemove".to_string(),
    });
    let result = graph.handle_command(command);

    // Then
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Graph(GraphEvent::GraphUntagged { id: _, tag }) => {
            assert_eq!(tag, "ToRemove");
        }
        _ => panic!("Expected GraphUntagged event"),
    }
}

#[test]
fn test_handle_untag_nonexistent_tag() {
    // Given
    let id = GraphId::new();
    let mut graph = Graph::new(id, "Graph".to_string(), None);
    graph.mark_events_as_committed();

    // When
    let command = Command::Graph(GraphCommand::UntagGraph {
        id,
        tag: "NonExistent".to_string(),
    });
    let result = graph.handle_command(command);

    // Then
    assert!(result.is_err());
    match result.unwrap_err() {
        GraphError::InvalidOperation(msg) => {
            assert!(msg.contains("Tag 'NonExistent' not found"));
        }
        _ => panic!("Expected InvalidOperation error"),
    }
}

#[test]
fn test_handle_add_node_command() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);
    graph.mark_events_as_committed();

    let node_id = NodeId::new();
    let content = NodeContent {
        label: "Test Node".to_string(),
        node_type: NodeType::Entity,
        properties: HashMap::new(),
    };
    let position = Position3D::new(1.0, 2.0, 3.0).unwrap();

    // When
    let command = Command::Node(NodeCommand::AddNode {
        graph_id,
        node_id,
        content: content.clone(),
        position,
    });
    let result = graph.handle_command(command);

    // Then
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Node(NodeEvent::NodeAdded {
            graph_id: _,
            node_id: event_node_id,
            metadata,
            position: event_pos,
        }) => {
            assert_eq!(*event_node_id, node_id);
            assert_eq!(*event_pos, position);
            assert_eq!(
                metadata.get("label").unwrap().as_str().unwrap(),
                "Test Node"
            );
        }
        _ => panic!("Expected NodeAdded event"),
    }
}

#[test]
fn test_handle_add_duplicate_node() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);
    let node_id = NodeId::new();

    // Add first node
    let command = Command::Node(NodeCommand::AddNode {
        graph_id,
        node_id,
        content: NodeContent {
            label: "Node".to_string(),
            node_type: NodeType::Entity,
            properties: HashMap::new(),
        },
        position: Position3D::default(),
    });
    graph.handle_command(command).unwrap();
    graph.mark_events_as_committed();

    // When - try to add same node again
    let command = Command::Node(NodeCommand::AddNode {
        graph_id,
        node_id,
        content: NodeContent {
            label: "Node".to_string(),
            node_type: NodeType::Entity,
            properties: HashMap::new(),
        },
        position: Position3D::default(),
    });
    let result = graph.handle_command(command);

    // Then
    assert!(result.is_err());
    match result.unwrap_err() {
        GraphError::NodeAlreadyExists(id) => assert_eq!(id, node_id),
        _ => panic!("Expected NodeAlreadyExists error"),
    }
}

#[test]
fn test_handle_add_node_with_invalid_position() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);

    // When - try to add node with NaN position
    let command = Command::Node(NodeCommand::AddNode {
        graph_id,
        node_id: NodeId::new(),
        content: NodeContent {
            label: "Node".to_string(),
            node_type: NodeType::Entity,
            properties: HashMap::new(),
        },
        position: Position3D::new(f32::NAN, 0.0, 0.0),
    });
    let result = graph.handle_command(command);

    // Then
    assert!(result.is_err());
    match result.unwrap_err() {
        GraphError::InvalidNodePosition => {}
        _ => panic!("Expected InvalidNodePosition error"),
    }
}

#[test]
fn test_handle_remove_node_command() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);
    let node_id = NodeId::new();

    // Add node first
    let add_command = Command::Node(NodeCommand::AddNode {
        graph_id,
        node_id,
        content: NodeContent {
            label: "Node".to_string(),
            node_type: NodeType::Entity,
            properties: HashMap::new(),
        },
        position: Position3D::default(),
    });
    graph.handle_command(add_command).unwrap();
    graph.mark_events_as_committed();

    // When
    let remove_command = Command::Node(NodeCommand::RemoveNode { graph_id, node_id });
    let result = graph.handle_command(remove_command);

    // Then
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Node(NodeEvent::NodeRemoved {
            graph_id: _,
            node_id: event_node_id,
        }) => {
            assert_eq!(*event_node_id, node_id);
        }
        _ => panic!("Expected NodeRemoved event"),
    }
}

#[test]
fn test_handle_remove_node_with_edges_cascade_delete() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);
    let node1 = NodeId::new();
    let node2 = NodeId::new();
    let edge_id = EdgeId::new();

    // Add two nodes
    for (id, label) in [(node1, "Node1"), (node2, "Node2")] {
        let command = Command::Node(NodeCommand::AddNode {
            graph_id,
            node_id: id,
            content: NodeContent {
                label: label.to_string(),
                node_type: NodeType::Entity,
                properties: HashMap::new(),
            },
            position: Position3D::default(),
        });
        graph.handle_command(command).unwrap();
    }

    // Connect them with an edge
    let edge_command = Command::Edge(EdgeCommand::ConnectEdge {
        graph_id,
        edge_id,
        source: node1,
        target: node2,
        relationship: EdgeRelationship {
            relationship_type: RelationshipType::DependsOn,
            properties: HashMap::new(),
            bidirectional: false,
        },
    });
    graph.handle_command(edge_command).unwrap();
    graph.mark_events_as_committed();

    // When - remove node1
    let remove_command = Command::Node(NodeCommand::RemoveNode {
        graph_id,
        node_id: node1,
    });
    let result = graph.handle_command(remove_command);

    // Then
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 2); // EdgeRemoved + NodeRemoved

    // First event should be edge removal
    match &events[0] {
        DomainEvent::Edge(EdgeEvent::EdgeRemoved {
            edge_id: removed_edge_id,
            ..
        }) => {
            assert_eq!(*removed_edge_id, edge_id);
        }
        _ => panic!("Expected EdgeRemoved event first"),
    }

    // Second event should be node removal
    match &events[1] {
        DomainEvent::Node(NodeEvent::NodeRemoved {
            node_id: removed_node_id,
            ..
        }) => {
            assert_eq!(*removed_node_id, node1);
        }
        _ => panic!("Expected NodeRemoved event second"),
    }
}

#[test]
fn test_handle_update_node_command() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);
    let node_id = NodeId::new();

    // Add node first
    let add_command = Command::Node(NodeCommand::AddNode {
        graph_id,
        node_id,
        content: NodeContent {
            label: "Original".to_string(),
            node_type: NodeType::Entity,
            properties: HashMap::new(),
        },
        position: Position3D::default(),
    });
    graph.handle_command(add_command).unwrap();
    graph.mark_events_as_committed();

    // When
    let update_command = Command::Node(NodeCommand::UpdateNode {
        graph_id,
        node_id,
        content: NodeContent {
            label: "Updated".to_string(),
            node_type: NodeType::ValueObject,
            properties: HashMap::new(),
        },
    });
    let result = graph.handle_command(update_command);

    // Then
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 2); // Remove + Add

    // Should follow DDD pattern: remove then add
    match &events[0] {
        DomainEvent::Node(NodeEvent::NodeRemoved { .. }) => {}
        _ => panic!("Expected NodeRemoved event first"),
    }

    match &events[1] {
        DomainEvent::Node(NodeEvent::NodeAdded { metadata, .. }) => {
            assert_eq!(metadata.get("label").unwrap().as_str().unwrap(), "Updated");
        }
        _ => panic!("Expected NodeAdded event second"),
    }
}

#[test]
fn test_handle_move_node_command() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);
    let node_id = NodeId::new();

    // Add node first
    let add_command = Command::Node(NodeCommand::AddNode {
        graph_id,
        node_id,
        content: NodeContent {
            label: "Node".to_string(),
            node_type: NodeType::Entity,
            properties: HashMap::new(),
        },
        position: Position3D::new(0.0, 0.0, 0.0),
    });
    graph.handle_command(add_command).unwrap();
    graph.mark_events_as_committed();

    // When
    let new_position = Position3D::new(10.0, 20.0, 30.0);
    let move_command = Command::Node(NodeCommand::MoveNode {
        graph_id,
        node_id,
        position: new_position,
    });
    let result = graph.handle_command(move_command);

    // Then
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 2); // Remove + Add

    // Should follow DDD pattern: remove then add
    match &events[0] {
        DomainEvent::Node(NodeEvent::NodeRemoved { .. }) => {}
        _ => panic!("Expected NodeRemoved event first"),
    }

    match &events[1] {
        DomainEvent::Node(NodeEvent::NodeAdded { position, .. }) => {
            assert_eq!(*position, new_position);
        }
        _ => panic!("Expected NodeAdded event second"),
    }
}

#[test]
fn test_handle_connect_edge_command() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);
    let node1 = NodeId::new();
    let node2 = NodeId::new();

    // Add two nodes
    for (id, label) in [(node1, "Node1"), (node2, "Node2")] {
        let command = Command::Node(NodeCommand::AddNode {
            graph_id,
            node_id: id,
            content: NodeContent {
                label: label.to_string(),
                node_type: NodeType::Entity,
                properties: HashMap::new(),
            },
            position: Position3D::default(),
        });
        graph.handle_command(command).unwrap();
    }
    graph.mark_events_as_committed();

    // When
    let edge_id = EdgeId::new();
    let relationship = EdgeRelationship {
        relationship_type: RelationshipType::DependsOn,
        properties: HashMap::new(),
        bidirectional: false,
    };
    let command = Command::Edge(EdgeCommand::ConnectEdge {
        graph_id,
        edge_id,
        source: node1,
        target: node2,
        relationship: relationship.clone(),
    });
    let result = graph.handle_command(command);

    // Then
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Edge(EdgeEvent::EdgeConnected {
            graph_id: _,
            edge_id: event_edge_id,
            source,
            target,
            relationship: rel,
        }) => {
            assert_eq!(*event_edge_id, edge_id);
            assert_eq!(*source, node1);
            assert_eq!(*target, node2);
            assert_eq!(rel.relationship_type, relationship.relationship_type);
        }
        _ => panic!("Expected EdgeConnected event"),
    }
}

#[test]
fn test_handle_connect_edge_self_loop_error() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);
    let node_id = NodeId::new();

    // Add node
    let command = Command::Node(NodeCommand::AddNode {
        graph_id,
        node_id,
        content: NodeContent {
            label: "Node".to_string(),
            node_type: NodeType::Entity,
            properties: HashMap::new(),
        },
        position: Position3D::default(),
    });
    graph.handle_command(command).unwrap();
    graph.mark_events_as_committed();

    // When - try to connect node to itself
    let command = Command::Edge(EdgeCommand::ConnectEdge {
        graph_id,
        edge_id: EdgeId::new(),
        source: node_id,
        target: node_id,
        relationship: EdgeRelationship {
            relationship_type: RelationshipType::DependsOn,
            properties: HashMap::new(),
            bidirectional: false,
        },
    });
    let result = graph.handle_command(command);

    // Then
    assert!(result.is_err());
    match result.unwrap_err() {
        GraphError::SelfLoopNotAllowed(id) => assert_eq!(id, node_id),
        _ => panic!("Expected SelfLoopNotAllowed error"),
    }
}

#[test]
fn test_handle_connect_duplicate_edge_error() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);
    let node1 = NodeId::new();
    let node2 = NodeId::new();

    // Add two nodes
    for (id, label) in [(node1, "Node1"), (node2, "Node2")] {
        let command = Command::Node(NodeCommand::AddNode {
            graph_id,
            node_id: id,
            content: NodeContent {
                label: label.to_string(),
                node_type: NodeType::Entity,
                properties: HashMap::new(),
            },
            position: Position3D::default(),
        });
        graph.handle_command(command).unwrap();
    }

    // Add first edge
    let command = Command::Edge(EdgeCommand::ConnectEdge {
        graph_id,
        edge_id: EdgeId::new(),
        source: node1,
        target: node2,
        relationship: EdgeRelationship {
            relationship_type: RelationshipType::DependsOn,
            properties: HashMap::new(),
            bidirectional: false,
        },
    });
    graph.handle_command(command).unwrap();
    graph.mark_events_as_committed();

    // When - try to add duplicate edge
    let command = Command::Edge(EdgeCommand::ConnectEdge {
        graph_id,
        edge_id: EdgeId::new(),
        source: node1,
        target: node2,
        relationship: EdgeRelationship {
            relationship_type: RelationshipType::Contains,
            properties: HashMap::new(),
            bidirectional: false,
        },
    });
    let result = graph.handle_command(command);

    // Then
    assert!(result.is_err());
    match result.unwrap_err() {
        GraphError::DuplicateEdge(s, t) => {
            assert_eq!(s, node1);
            assert_eq!(t, node2);
        }
        _ => panic!("Expected DuplicateEdge error"),
    }
}

#[test]
fn test_handle_disconnect_edge_command() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);
    let node1 = NodeId::new();
    let node2 = NodeId::new();
    let edge_id = EdgeId::new();

    // Add nodes and edge
    for (id, label) in [(node1, "Node1"), (node2, "Node2")] {
        let command = Command::Node(NodeCommand::AddNode {
            graph_id,
            node_id: id,
            content: NodeContent {
                label: label.to_string(),
                node_type: NodeType::Entity,
                properties: HashMap::new(),
            },
            position: Position3D::default(),
        });
        graph.handle_command(command).unwrap();
    }

    let command = Command::Edge(EdgeCommand::ConnectEdge {
        graph_id,
        edge_id,
        source: node1,
        target: node2,
        relationship: EdgeRelationship {
            relationship_type: RelationshipType::DependsOn,
            properties: HashMap::new(),
            bidirectional: false,
        },
    });
    graph.handle_command(command).unwrap();
    graph.mark_events_as_committed();

    // When
    let command = Command::Edge(EdgeCommand::DisconnectEdge { graph_id, edge_id });
    let result = graph.handle_command(command);

    // Then
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Edge(EdgeEvent::EdgeRemoved {
            edge_id: removed_id,
            ..
        }) => {
            assert_eq!(*removed_id, edge_id);
        }
        _ => panic!("Expected EdgeRemoved event"),
    }
}

#[test]
fn test_handle_select_deselect_node_commands() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);
    let node_id = NodeId::new();

    // Add node
    let command = Command::Node(NodeCommand::AddNode {
        graph_id,
        node_id,
        content: NodeContent {
            label: "Node".to_string(),
            node_type: NodeType::Entity,
            properties: HashMap::new(),
        },
        position: Position3D::default(),
    });
    graph.handle_command(command).unwrap();
    graph.mark_events_as_committed();

    // When - select node
    let select_command = Command::Node(NodeCommand::SelectNode { graph_id, node_id });
    let result = graph.handle_command(select_command);

    // Then
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);
    match &events[0] {
        DomainEvent::Node(NodeEvent::NodeSelected { .. }) => {}
        _ => panic!("Expected NodeSelected event"),
    }

    graph.mark_events_as_committed();

    // When - deselect node
    let deselect_command = Command::Node(NodeCommand::DeselectNode { graph_id, node_id });
    let result = graph.handle_command(deselect_command);

    // Then
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);
    match &events[0] {
        DomainEvent::Node(NodeEvent::NodeDeselected { .. }) => {}
        _ => panic!("Expected NodeDeselected event"),
    }
}

#[test]
fn test_event_sourcing_reconstruction() {
    // Given
    let graph_id = GraphId::new();
    let node_id = NodeId::new();
    let edge_id = EdgeId::new();
    let node2_id = NodeId::new();

    // Create a series of events
    let events = vec![
        DomainEvent::Graph(GraphEvent::GraphCreated {
            id: graph_id,
            metadata: GraphMetadata::new("Test Graph".to_string()),
        }),
        DomainEvent::Node(NodeEvent::NodeAdded {
            graph_id,
            node_id,
            position: Position3D::new(1.0, 2.0, 3.0),
            metadata: {
                let mut m = HashMap::new();
                m.insert("label".to_string(), serde_json::json!("Node1"));
                m.insert("node_type".to_string(), serde_json::json!("Entity"));
                m
            },
        }),
        DomainEvent::Node(NodeEvent::NodeAdded {
            graph_id,
            node_id: node2_id,
            position: Position3D::new(4.0, 5.0, 6.0),
            metadata: {
                let mut m = HashMap::new();
                m.insert("label".to_string(), serde_json::json!("Node2"));
                m.insert("node_type".to_string(), serde_json::json!("ValueObject"));
                m
            },
        }),
        DomainEvent::Edge(EdgeEvent::EdgeConnected {
            graph_id,
            edge_id,
            source: node_id,
            target: node2_id,
            relationship: EdgeRelationship {
                relationship_type: RelationshipType::DependsOn,
                properties: HashMap::new(),
                bidirectional: false,
            },
        }),
    ];

    // When
    let graph = Graph::from_events(graph_id, events);

    // Then
    assert_eq!(graph.id, graph_id);
    assert_eq!(graph.metadata.name, "Test Graph");
    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);
    assert_eq!(graph.version, 0); // Version is incremented only for new events
    assert_eq!(graph.get_uncommitted_events().len(), 0); // No new events
}

#[test]
fn test_version_tracking() {
    // Given
    let graph_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);
    assert_eq!(graph.version, 0);
    graph.mark_events_as_committed();

    // When - add multiple commands
    for i in 0..3 {
        let command = Command::Node(NodeCommand::AddNode {
            graph_id,
            node_id: NodeId::new(),
            content: NodeContent {
                label: format!("Node{}", i),
                node_type: NodeType::Entity,
                properties: HashMap::new(),
            },
            position: Position3D::default(),
        });
        graph.handle_command(command).unwrap();
    }

    // Then
    assert_eq!(graph.version, 3);
    assert_eq!(graph.get_uncommitted_events().len(), 3);
}

#[test]
fn test_wrong_graph_id_error() {
    // Given
    let graph_id = GraphId::new();
    let wrong_id = GraphId::new();
    let mut graph = Graph::new(graph_id, "Graph".to_string(), None);

    // When - try command with wrong graph ID
    let command = Command::Node(NodeCommand::AddNode {
        graph_id: wrong_id,
        node_id: NodeId::new(),
        content: NodeContent {
            label: "Node".to_string(),
            node_type: NodeType::Entity,
            properties: HashMap::new(),
        },
        position: Position3D::default(),
    });
    let result = graph.handle_command(command);

    // Then
    assert!(result.is_err());
    match result.unwrap_err() {
        GraphError::GraphNotFound(id) => assert_eq!(id, wrong_id),
        _ => panic!("Expected GraphNotFound error"),
    }
}
