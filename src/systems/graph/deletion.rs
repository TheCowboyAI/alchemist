//! Systems for deleting graph nodes and edges
//!
//! These systems handle:
//! - Safe node deletion with edge cleanup
//! - Edge deletion with connection updates
//! - Batch deletion operations
//! - Undo tracking for deletions

use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    components::*,
    events::*,
    resources::*,
};

/// System that handles node deletion
///
/// This system:
/// 1. Validates the node exists
/// 2. Finds and deletes all connected edges
/// 3. Removes the node entity
/// 4. Updates UUID mapping
/// 5. Sends modification events for undo
pub fn handle_delete_node(
    mut commands: Commands,
    mut events: EventReader<DeleteNodeEvent>,
    nodes: Query<&NodeId>,
    edges: Query<(Entity, &Edge)>,
    mut uuid_to_entity: ResMut<UuidToEntity>,
    mut delete_edge_events: EventWriter<DeleteEdgeEvent>,
    mut modification_events: EventWriter<GraphModificationEvent>,
    mut deselect_events: EventWriter<DeselectAllEvent>,
) {
    for event in events.read() {
        // Get node ID before deletion
        let node_id = match nodes.get(event.entity) {
            Ok(id) => id.0,
            Err(_) => {
                warn!("Attempted to delete non-existent node entity: {:?}", event.entity);
                continue;
            }
        };

        // Find all connected edges
        let mut edges_to_delete = Vec::new();
        for (edge_entity, edge) in edges.iter() {
            if edge.source == event.entity || edge.target == event.entity {
                edges_to_delete.push((edge_entity, edge.id, edge.source));
            }
        }

        // Delete connected edges first
        for (edge_entity, edge_id, source) in edges_to_delete {
            delete_edge_events.send(DeleteEdgeEvent {
                source,
                edge_id,
            });
        }

        // Remove from UUID mapping
        uuid_to_entity.0.remove(&node_id);

        // Delete the node entity
        commands.entity(event.entity).despawn_recursive();

        // Send modification event for undo
        modification_events.send(GraphModificationEvent::NodeDeleted {
            id: node_id,
        });

        // Deselect if this node was selected
        deselect_events.send(DeselectAllEvent);

        info!("Deleted node with ID: {}", node_id);
    }
}

/// System that handles edge deletion
///
/// This system:
/// 1. Validates the edge exists
/// 2. Removes edge components from connected nodes
/// 3. Deletes the edge entity
/// 4. Sends modification events
pub fn handle_delete_edge(
    mut commands: Commands,
    mut events: EventReader<DeleteEdgeEvent>,
    edges: Query<(Entity, &Edge)>,
    mut nodes_with_outgoing: Query<&mut OutgoingEdge>,
    mut nodes_with_incoming: Query<&mut IncomingEdge>,
    mut modification_events: EventWriter<GraphModificationEvent>,
) {
    for event in events.read() {
        // Find the edge entity
        let edge_data = edges.iter()
            .find(|(_, edge)| edge.id == event.edge_id);

        if let Some((edge_entity, edge)) = edge_data {
            // Remove outgoing edge component from source node
            if let Ok(mut outgoing) = nodes_with_outgoing.get_mut(edge.source) {
                if outgoing.edge_id == event.edge_id {
                    commands.entity(edge.source).remove::<OutgoingEdge>();
                }
            }

            // Remove incoming edge component from target node
            if let Ok(mut incoming) = nodes_with_incoming.get_mut(edge.target) {
                if incoming.edge_id == event.edge_id {
                    commands.entity(edge.target).remove::<IncomingEdge>();
                }
            }

            // Delete the edge entity
            commands.entity(edge_entity).despawn_recursive();

            // Send modification event
            modification_events.send(GraphModificationEvent::EdgeDeleted {
                id: event.edge_id,
            });

            info!("Deleted edge with ID: {}", event.edge_id);
        } else {
            warn!("Attempted to delete non-existent edge: {}", event.edge_id);
        }
    }
}

/// System that handles batch deletion operations
///
/// This system processes multiple deletions efficiently
pub fn handle_batch_deletion(
    mut events: EventReader<BatchOperationEvent>,
    mut delete_node_events: EventWriter<DeleteNodeEvent>,
    mut delete_edge_events: EventWriter<DeleteEdgeEvent>,
) {
    for event in events.read() {
        for operation in &event.operations {
            match operation {
                GraphOperation::DeleteNode(entity) => {
                    delete_node_events.send(DeleteNodeEvent {
                        entity: *entity,
                    });
                }
                GraphOperation::DeleteEdge { source, edge_id } => {
                    delete_edge_events.send(DeleteEdgeEvent {
                        source: *source,
                        edge_id: *edge_id,
                    });
                }
                _ => {} // Other operations handled by different systems
            }
        }
    }
}

/// System that clears the entire graph
///
/// This system deletes all nodes and edges when requested
pub fn handle_clear_graph(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    nodes: Query<Entity, With<NodeId>>,
    edges: Query<Entity, With<Edge>>,
    mut uuid_to_entity: ResMut<UuidToEntity>,
    mut modification_events: EventWriter<GraphModificationEvent>,
    mut notification_events: EventWriter<ShowNotificationEvent>,
) {
    // Check for clear graph shortcut (Ctrl+Shift+Delete)
    if keyboard.pressed(KeyCode::ControlLeft)
        && keyboard.pressed(KeyCode::ShiftLeft)
        && keyboard.just_pressed(KeyCode::Delete)
    {
        let node_count = nodes.iter().count();
        let edge_count = edges.iter().count();

        // Delete all edges
        for entity in edges.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Delete all nodes
        for entity in nodes.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Clear UUID mapping
        uuid_to_entity.0.clear();

        // Send modification event
        modification_events.send(GraphModificationEvent::GraphCleared);

        // Send notification
        notification_events.send(ShowNotificationEvent {
            message: format!("Cleared graph: {} nodes and {} edges deleted", node_count, edge_count),
            notification_type: NotificationType::Info,
            duration_seconds: 3.0,
        });

        info!("Cleared entire graph: {} nodes, {} edges", node_count, edge_count);
    }
}

/// System that handles deletion of selected entities
///
/// This system deletes all currently selected nodes when Delete key is pressed
pub fn handle_delete_selected(
    keyboard: Res<Input<KeyCode>>,
    selected: Query<Entity, With<Selected>>,
    mut delete_events: EventWriter<DeleteNodeEvent>,
) {
    if keyboard.just_pressed(KeyCode::Delete) && !keyboard.pressed(KeyCode::ShiftLeft) {
        let count = selected.iter().count();

        for entity in selected.iter() {
            delete_events.send(DeleteNodeEvent { entity });
        }

        if count > 0 {
            info!("Deleting {} selected nodes", count);
        }
    }
}

/// System that handles cut operations (delete with clipboard)
///
/// This system copies selected nodes to clipboard before deleting them
pub fn handle_cut_operation(
    mut events: EventReader<GraphClipboardEvent>,
    selected: Query<(Entity, &NodeId, &Transform, &DomainNodeType, &NodeProperties), With<Selected>>,
    mut clipboard: ResMut<GraphClipboard>,
    mut delete_events: EventWriter<DeleteNodeEvent>,
) {
    for event in events.read() {
        if let ClipboardOperation::Cut(entities) = &event.operation {
            // Clear existing clipboard
            clipboard.nodes.clear();
            clipboard.edges.clear();

            // Copy node data to clipboard
            for entity in entities {
                if let Ok((_, node_id, transform, domain_type, properties)) = selected.get(*entity) {
                    clipboard.nodes.push(ClipboardNode {
                        id: node_id.0,
                        position: transform.translation,
                        domain_type: domain_type.clone(),
                        properties: properties.clone(),
                    });

                    // Delete the node
                    delete_events.send(DeleteNodeEvent { entity: *entity });
                }
            }

            info!("Cut {} nodes to clipboard", clipboard.nodes.len());
        }
    }
}

/// System that validates deletions before they occur
///
/// This system can prevent deletion of certain nodes based on rules
pub fn validate_deletion(
    mut events: EventReader<DeleteNodeEvent>,
    nodes: Query<(&NodeId, &DomainNodeType, Option<&Protected>)>,
    mut validated_events: EventWriter<DeleteNodeEvent>,
    mut notification_events: EventWriter<ShowNotificationEvent>,
) {
    for event in events.read() {
        if let Ok((node_id, domain_type, protected)) = nodes.get(event.entity) {
            // Check if node is protected
            if protected.is_some() {
                notification_events.send(ShowNotificationEvent {
                    message: "Cannot delete protected node".to_string(),
                    notification_type: NotificationType::Warning,
                    duration_seconds: 2.0,
                });
                continue;
            }

            // Check domain-specific rules
            match domain_type {
                DomainNodeType::Aggregate => {
                    // Could check if aggregate has entities before allowing deletion
                    validated_events.send(*event);
                }
                _ => {
                    // Other nodes can be deleted freely
                    validated_events.send(*event);
                }
            }
        }
    }
}
