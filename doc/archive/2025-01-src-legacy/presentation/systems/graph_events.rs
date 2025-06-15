use crate::domain::events::{DomainEvent, EdgeEvent, NodeEvent};
use crate::domain::value_objects::{GraphId, NodeId, Position3D, SubgraphId};
use crate::presentation::components::{GraphEdge, GraphNode, SubgraphMember};
use bevy::prelude::*;
use std::collections::HashMap;
use uuid;

/// Handle NodeAdded events from the domain
pub fn handle_node_added(
    mut commands: Commands,
    mut events: EventReader<DomainEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        if let DomainEvent::Node(NodeEvent::NodeAdded {
            graph_id,
            node_id,
            position,
            metadata,
            ..
        }) = event
        {
            spawn_node(
                &mut commands,
                &mut meshes,
                &mut materials,
                *graph_id,
                *node_id,
                position.clone(),
                metadata.clone(),
            );
        }
    }
}

/// Handle NodeRemoved events from the domain
pub fn handle_node_removed(
    mut commands: Commands,
    mut events: EventReader<DomainEvent>,
    nodes: Query<(Entity, &GraphNode)>,
) {
    for event in events.read() {
        if let DomainEvent::Node(NodeEvent::NodeRemoved { node_id, .. }) = event {
            // Find and despawn the node entity
            for (entity, node) in nodes.iter() {
                if node.node_id == *node_id {
                    commands.entity(entity).despawn_recursive();
                    break;
                }
            }
        }
    }
}

/// Handle EdgeAdded events from the domain
pub fn handle_edge_added(
    mut commands: Commands,
    mut events: EventReader<DomainEvent>,
    nodes: Query<(Entity, &GraphNode)>,
) {
    for event in events.read() {
        if let DomainEvent::Edge(EdgeEvent::EdgeConnected {
            graph_id,
            edge_id,
            source,
            target,
            ..
        }) = event
        {
            // Find source and target entities
            let mut source_entity = None;
            let mut target_entity = None;

            for (entity, node) in nodes.iter() {
                if node.node_id == *source {
                    source_entity = Some(entity);
                }
                if node.node_id == *target {
                    target_entity = Some(entity);
                }
                if source_entity.is_some() && target_entity.is_some() {
                    break;
                }
            }

            // Create edge if both nodes exist
            if let (Some(source_e), Some(target_e)) = (source_entity, target_entity) {
                commands.spawn((
                    GraphEdge {
                        edge_id: *edge_id,
                        graph_id: *graph_id,
                        source: source_e,
                        target: target_e,
                    },
                    Name::new(format!("Edge_{}", edge_id)),
                ));
            }
        }
    }
}

/// Handle EdgeRemoved events from the domain
pub fn handle_edge_removed(
    mut commands: Commands,
    mut events: EventReader<DomainEvent>,
    edges: Query<(Entity, &GraphEdge)>,
) {
    for event in events.read() {
        if let DomainEvent::Edge(EdgeEvent::EdgeRemoved { edge_id, .. }) = event {
            // Find and despawn the edge entity
            for (entity, edge) in edges.iter() {
                if edge.edge_id == *edge_id {
                    commands.entity(entity).despawn_recursive();
                    break;
                }
            }
        }
    }
}

fn spawn_node(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    graph_id: GraphId,
    node_id: NodeId,
    position: Position3D,
    metadata: HashMap<String, serde_json::Value>,
) {
    // Default node type and color
    let color = Color::srgb(0.2, 0.6, 0.8);

    let mut entity_commands = commands.spawn((
        GraphNode { node_id, graph_id },
        crate::presentation::systems::subgraph_drag_drop::Draggable::default(),
        Mesh3d(meshes.add(Sphere::new(5.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            ..default()
        })),
        Transform::from_translation(Vec3::new(position.x, position.y, position.z)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Name::new(format!("Node_{}", node_id)),
    ));

    // Check if this node belongs to a subgraph
    if let Some(subgraph_id_value) = metadata.get("subgraph_id") {
        if let Some(subgraph_id_str) = subgraph_id_value.as_str() {
            if let Ok(uuid) = uuid::Uuid::parse_str(subgraph_id_str) {
                let subgraph_id = SubgraphId::from_uuid(uuid);

                // Get the subgraph origin
                let subgraph_origin = metadata
                    .get("subgraph_origin")
                    .and_then(|origin_value| {
                        if let Some(origin_str) = origin_value.as_str() {
                            let parts: Vec<&str> = origin_str.split(',').collect();
                            if parts.len() == 3 {
                                Some(Position3D {
                                    x: parts[0].parse().unwrap_or(0.0),
                                    y: parts[1].parse().unwrap_or(0.0),
                                    z: parts[2].parse().unwrap_or(0.0),
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Position3D {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    });

                // Calculate relative position
                let relative_position = Position3D {
                    x: position.x - subgraph_origin.x,
                    y: position.y - subgraph_origin.y,
                    z: position.z - subgraph_origin.z,
                };

                entity_commands.insert(SubgraphMember {
                    subgraph_ids: {
                        let mut set = std::collections::HashSet::new();
                        set.insert(subgraph_id);
                        set
                    },
                    relative_position,
                });
            }
        }
    }
}
