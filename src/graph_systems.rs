//! ECS Systems for Graph Operations
//!
//! These systems handle graph creation, modification, layout, persistence,
//! and integration with JetStream for event sourcing.

use bevy::prelude::*;
use crate::graph_components::*;
use crate::graph_parser;
use crate::nats_client::{NatsClient, JetStreamClient};
use async_nats::jetstream;
use serde_json::json;
use std::collections::{HashMap, VecDeque};

/// Plugin that adds all graph systems
pub struct GraphSystemsPlugin;

impl Plugin for GraphSystemsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GraphManager>()
            .init_resource::<GraphConfig>()
            .init_resource::<GraphLayoutEngine>()
            .add_event::<GraphOperationEvent>()
            .add_systems(Update, (
                handle_graph_operations,
                update_node_connections,
                apply_force_directed_layout,
                handle_node_selection,
                handle_node_dragging,
                update_edge_positions,
                animate_node_positions,
                persist_graph_changes,
                sync_with_jetstream,
            ))
            .add_systems(PostUpdate, (
                update_graph_stats,
                cleanup_orphaned_edges,
            ));
    }
}

/// System that handles graph operation events
pub fn handle_graph_operations(
    mut commands: Commands,
    mut events: EventReader<GraphOperationEvent>,
    mut graph_manager: ResMut<GraphManager>,
    mut graphs: Query<&mut Graph>,
    nodes: Query<&GraphNode>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        match &event.operation {
            GraphOperation::CreateNode { id, label, position } => {
                // Create node entity with all components
                let node_entity = commands.spawn(GraphNodeBundle {
                    node: GraphNode {
                        id: id.clone(),
                        graph_id: event.graph_id.clone(),
                        label: label.clone(),
                    },
                    transform: Transform::from_translation(*position),
                    ..Default::default()
                })
                .insert(NodeMetadata::default())
                .insert(NodeStyle::default())
                .insert(Selectable::default())
                .insert(Draggable::default())
                .insert(NodeConnections::default())
                .insert(PhysicsNode::default())
                .insert(NodeLabel {
                    text: label.clone(),
                    ..Default::default()
                })
                .insert(Persistent {
                    collection: "nodes".to_string(),
                    last_saved: None,
                    dirty: true,
                })
                .with_children(|parent| {
                    // Add 3D mesh for visualization
                    parent.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.5).mesh())),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(0.5, 0.5, 0.8),
                            ..default()
                        })),
                    ));
                })
                .id();
                
                // Add to graph
                if let Some(graph_entity) = graph_manager.get_graph(&event.graph_id) {
                    if let Ok(mut graph) = graphs.get_mut(graph_entity) {
                        graph.add_node(node_entity);
                    }
                }
            }
            
            GraphOperation::CreateEdge { source, target, label } => {
                // Create edge entity
                let edge_entity = commands.spawn(GraphEdgeBundle {
                    edge: GraphEdge {
                        id: format!("edge_{}_{}", source.index(), target.index()),
                        graph_id: event.graph_id.clone(),
                        source: *source,
                        target: *target,
                        label: label.clone(),
                        weight: 1.0,
                    },
                    style: EdgeStyle::default(),
                    transform: Transform::default(),
                    global_transform: GlobalTransform::default(),
                })
                .insert(Persistent {
                    collection: "edges".to_string(),
                    last_saved: None,
                    dirty: true,
                })
                .id();
                
                // Add to graph
                if let Some(graph_entity) = graph_manager.get_graph(&event.graph_id) {
                    if let Ok(mut graph) = graphs.get_mut(graph_entity) {
                        graph.add_edge(edge_entity);
                    }
                }
            }
            
            GraphOperation::DeleteNode { entity } => {
                commands.entity(*entity).despawn();
                
                // Remove from graph
                if let Some(graph_entity) = graph_manager.get_graph(&event.graph_id) {
                    if let Ok(mut graph) = graphs.get_mut(graph_entity) {
                        graph.remove_node(*entity);
                    }
                }
            }
            
            GraphOperation::DeleteEdge { entity } => {
                commands.entity(*entity).despawn();
                
                // Remove from graph
                if let Some(graph_entity) = graph_manager.get_graph(&event.graph_id) {
                    if let Ok(mut graph) = graphs.get_mut(graph_entity) {
                        graph.remove_edge(*entity);
                    }
                }
            }
            
            GraphOperation::UpdateNode { entity, label, metadata } => {
                if let Ok(mut entity_commands) = commands.get_entity(*entity) {
                    if let Some(new_label) = label {
                        entity_commands.insert(GraphNode {
                            id: nodes.get(*entity).map(|n| n.id.clone()).unwrap_or_default(),
                            graph_id: nodes.get(*entity).map(|n| n.graph_id.clone()).unwrap_or_default(),
                            label: new_label.clone(),
                        });
                    }
                    
                    if let Some(new_metadata) = metadata {
                        entity_commands.insert(NodeMetadata {
                            properties: new_metadata.clone(),
                            ..default()
                        });
                    }
                    
                    entity_commands.insert(Persistent {
                        collection: "nodes".to_string(),
                        last_saved: None,
                        dirty: true,
                    });
                }
            }
            
            GraphOperation::Clear => {
                if let Some(graph_entity) = graph_manager.get_graph(&event.graph_id) {
                    if let Ok(graph) = graphs.get(graph_entity) {
                        // Delete all nodes and edges
                        for &node in &graph.nodes {
                            commands.entity(node).despawn();
                        }
                        for &edge in &graph.edges {
                            commands.entity(edge).despawn();
                        }
                    }
                }
            }
            
            GraphOperation::ApplyLayout { layout_type } => {
                // This would trigger layout recalculation
                info!("Applying {:?} layout to graph {}", layout_type, event.graph_id);
            }
            
            _ => {} // Handle other operations
        }
    }
}

/// System that updates node connections based on edges
pub fn update_node_connections(
    edges: Query<&GraphEdge, Changed<GraphEdge>>,
    mut nodes: Query<&mut NodeConnections>,
) {
    for edge in edges.iter() {
        // Update source node's outgoing connections
        if let Ok(mut source_connections) = nodes.get_mut(edge.source) {
            source_connections.outgoing.insert(edge.target);
        }
        
        // Update target node's incoming connections
        if let Ok(mut target_connections) = nodes.get_mut(edge.target) {
            target_connections.incoming.insert(edge.source);
        }
    }
}

/// Force-directed layout system
pub fn apply_force_directed_layout(
    time: Res<Time>,
    config: Res<GraphConfig>,
    mut nodes: Query<(&mut Transform, &mut PhysicsNode, &NodeConnections, Entity)>,
    edges: Query<&GraphEdge>,
) {
    if !config.physics_enabled || !config.auto_layout {
        return;
    }
    
    let delta = time.delta_secs();
    
    // Reset accelerations
    for (_, mut physics, _, _) in nodes.iter_mut() {
        physics.acceleration = Vec3::ZERO;
    }
    
    // Calculate repulsive forces between all nodes
    let mut node_positions = HashMap::new();
    for (transform, _, _, entity) in nodes.iter() {
        node_positions.insert(entity, transform.translation);
    }
    
    for (transform, mut physics, _, entity) in nodes.iter_mut() {
        let pos1 = transform.translation;
        
        // Repulsive forces
        let charge = physics.charge;
        let mass = physics.mass;
        
        for (&other_entity, &pos2) in node_positions.iter() {
            if entity != other_entity {
                let diff = pos1 - pos2;
                let distance = diff.length().max(0.1);
                let force_magnitude = charge * charge / (distance * distance);
                let force = diff.normalize() * force_magnitude;
                physics.acceleration += force / mass;
            }
        }
        
        // Centering force
        let center_force = -pos1 * 0.01;
        physics.acceleration += center_force;
    }
    
    // Apply spring forces for edges
    for edge in edges.iter() {
        if let Ok([(t1, mut p1, _, _), (t2, mut p2, _, _)]) = 
            nodes.get_many_mut([edge.source, edge.target]) {
            
            let diff = t2.translation - t1.translation;
            let distance = diff.length().max(0.1);
            let ideal_distance = 3.0;
            let spring_force = (distance - ideal_distance) * 0.1;
            let force = diff.normalize() * spring_force;
            
            let mass1 = p1.mass;
            let mass2 = p2.mass;
            p1.acceleration += force / mass1;
            p2.acceleration -= force / mass2;
        }
    }
    
    // Update velocities and positions
    for (mut transform, mut physics, _, _) in nodes.iter_mut() {
        let damping = physics.damping;
        let acceleration = physics.acceleration;
        physics.velocity += acceleration * delta;
        physics.velocity *= damping;
        transform.translation += physics.velocity * delta;
        
        // Clamp to reasonable bounds
        transform.translation = transform.translation.clamp(
            Vec3::splat(-50.0),
            Vec3::splat(50.0)
        );
    }
}

/// System for handling node selection
fn handle_node_selection(
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut nodes: Query<(Entity, &mut Selectable, &Transform, &NodeStyle)>,
    config: Res<GraphConfig>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    
    let Some((camera, camera_transform)) = camera_query.iter().next() else { return };
    let Some(window) = windows.iter().next() else { return };
    
    if let Some(cursor_position) = window.cursor_position() {
        let ray = camera.viewport_to_world(camera_transform, cursor_position);
        
        if let Ok(ray) = ray {
            // Simple ray-sphere intersection for node selection
            let mut closest_hit: Option<Entity> = None;
            let mut closest_distance = f32::MAX;
            
            for (entity, selectable, transform, style) in nodes.iter() {
                let sphere_center = transform.translation;
                let sphere_radius = style.size;
                
                // Ray-sphere intersection
                let oc = ray.origin - sphere_center;
                let direction_vec = *ray.direction;
                let a = direction_vec.dot(direction_vec);
                let b = 2.0 * oc.dot(direction_vec);
                let c = oc.dot(oc) - sphere_radius * sphere_radius;
                let discriminant: f32 = b * b - 4.0 * a * c;
                
                if discriminant >= 0.0 {
                    let t = (-b - discriminant.sqrt()) / (2.0 * a);
                    if t > 0.0 && t < closest_distance {
                        closest_distance = t;
                        closest_hit = Some(entity);
                    }
                }
            }
            
            // Update selection
            if config.selection_mode == SelectionMode::Single {
                for (_, mut selectable, _, _) in nodes.iter_mut() {
                    selectable.selected = false;
                }
            }
            
            if let Some(hit_entity) = closest_hit {
                if let Ok((_, mut selectable, _, _)) = nodes.get_mut(hit_entity) {
                    selectable.selected = !selectable.selected;
                }
            }
        }
    }
}

/// System for dragging nodes
fn handle_node_dragging(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut nodes: Query<(&mut Transform, &Selectable, &mut Draggable)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
) {
    let Some((camera, camera_transform)) = camera_query.iter().next() else { return };
    let Some(window) = windows.iter().next() else { return };
    
    if mouse_button.pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                for (mut transform, selectable, mut draggable) in nodes.iter_mut() {
                    if selectable.selected {
                        if !draggable.is_dragging {
                            draggable.is_dragging = true;
                            draggable.drag_offset = transform.translation - ray.origin;
                        }
                        
                        // Project cursor to node's plane
                        let plane_normal = (camera_transform.translation() - transform.translation).normalize();
                        let plane_d = -plane_normal.dot(transform.translation);
                        let t = -(ray.origin.dot(plane_normal) + plane_d) / ray.direction.dot(plane_normal);
                        
                        if t > 0.0 {
                            let new_position = ray.origin + ray.direction * t;
                            transform.translation = new_position;
                        }
                    }
                }
            }
        }
    } else {
        // Release all dragging
        for (_, _, mut draggable) in nodes.iter_mut() {
            draggable.is_dragging = false;
        }
    }
}

/// System to update edge positions based on node positions
fn update_edge_positions(
    mut edges: Query<(&GraphEdge, &mut Transform)>,
    nodes: Query<&Transform, (With<GraphNode>, Without<GraphEdge>)>,
) {
    for (edge, mut edge_transform) in edges.iter_mut() {
        if let Ok([source_transform, target_transform]) = 
            nodes.get_many([edge.source, edge.target]) {
            
            // Position edge at midpoint
            edge_transform.translation = (source_transform.translation + target_transform.translation) / 2.0;
            
            // Orient edge to point from source to target
            let direction = (target_transform.translation - source_transform.translation).normalize();
            edge_transform.rotation = Quat::from_rotation_arc(Vec3::Z, direction);
        }
    }
}

/// System for animating node positions
fn animate_node_positions(
    mut commands: Commands,
    time: Res<Time>,
    mut nodes: Query<(Entity, &mut Transform, &NodeAnimation)>,
) {
    let current_time = time.elapsed_secs();
    
    for (entity, mut transform, animation) in nodes.iter_mut() {
        let progress = ((current_time - animation.start_time) / animation.duration).clamp(0.0, 1.0);
        
        let t = match animation.easing {
            EasingFunction::Linear => progress,
            EasingFunction::EaseInOut => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    let val: f32 = -2.0 * progress + 2.0;
                    1.0 - val.powi(2) / 2.0
                }
            }
            _ => progress, // Implement other easing functions
        };
        
        transform.translation = animation.start_position.lerp(animation.target_position, t);
        
        if progress >= 1.0 {
            commands.entity(entity).remove::<NodeAnimation>();
        }
    }
}

/// System to persist graph changes to storage
fn persist_graph_changes(
    mut persistent_nodes: Query<
        (Entity, &GraphNode, &Transform, &NodeMetadata, &mut Persistent),
        Changed<GraphNode>
    >,
    mut persistent_edges: Query<
        (Entity, &GraphEdge, &EdgeStyle, &mut Persistent),
        Changed<GraphEdge>
    >,
    time: Res<Time>,
) {
    let now = time.elapsed();
    
    // Mark changed nodes as dirty
    for (entity, node, transform, metadata, mut persistent) in persistent_nodes.iter_mut() {
        persistent.dirty = true;
        
        // In a real implementation, we would serialize and save here
        let node_data = json!({
            "entity": entity.index(),
            "id": node.id,
            "label": node.label,
            "position": [transform.translation.x, transform.translation.y, transform.translation.z],
            "metadata": metadata.properties,
        });
        
        // This would be sent to storage backend
        debug!("Node {} marked for persistence: {:?}", node.id, node_data);
    }
    
    // Mark changed edges as dirty
    for (entity, edge, style, mut persistent) in persistent_edges.iter_mut() {
        persistent.dirty = true;
        
        let edge_data = json!({
            "entity": entity.index(),
            "source": edge.source.index(),
            "target": edge.target.index(),
            "label": edge.label,
            "weight": edge.weight,
            "style": {
                "color": [style.color.to_srgba().red, style.color.to_srgba().green, style.color.to_srgba().blue, style.color.to_srgba().alpha],
                "width": style.width,
            }
        });
        
        debug!("Edge marked for persistence: {:?}", edge_data);
    }
}

/// System to sync with JetStream
fn sync_with_jetstream(
    persistent_entities: Query<(Entity, &Persistent), With<Persistent>>,
    graph_events: Query<&GraphEvent>,
    nats_client: Option<Res<NatsClient>>,
) {
    let Some(nats) = nats_client else { return };
    
    // Queue events for sending to JetStream
    for event in graph_events.iter() {
        let subject = format!("graph.{}.{:?}", event.graph_id, event.event_type);
        
        // In a real implementation, this would be async
        let event_data = serde_json::to_vec(event).unwrap();
        
        // This would actually publish to NATS
        debug!("Would publish to JetStream: {} -> {:?}", subject, event);
    }
}

/// System to update graph statistics
fn update_graph_stats(
    graphs: Query<(Entity, &Graph)>,
    nodes: Query<&NodeConnections>,
    mut stats: Query<&mut GraphStats>,
) {
    for (graph_entity, graph) in graphs.iter() {
        if let Ok(mut graph_stats) = stats.get_mut(graph_entity) {
            graph_stats.node_count = graph.nodes.len();
            graph_stats.edge_count = graph.edges.len();
            
            // Calculate average degree
            let total_degree: usize = graph.nodes
                .iter()
                .filter_map(|&node| nodes.get(node).ok())
                .map(|conn| conn.degree())
                .sum();
            
            graph_stats.average_degree = if graph.nodes.is_empty() {
                0.0
            } else {
                total_degree as f32 / graph.nodes.len() as f32
            };
            
            // Calculate density
            let max_edges = graph.nodes.len() * (graph.nodes.len() - 1);
            graph_stats.density = if max_edges > 0 {
                graph.edges.len() as f32 / max_edges as f32
            } else {
                0.0
            };
        }
    }
}

/// System to clean up orphaned edges
fn cleanup_orphaned_edges(
    mut commands: Commands,
    edges: Query<(Entity, &GraphEdge)>,
    nodes: Query<Entity, With<GraphNode>>,
) {
    for (edge_entity, edge) in edges.iter() {
        // Check if both source and target nodes still exist
        if nodes.get(edge.source).is_err() || nodes.get(edge.target).is_err() {
            commands.entity(edge_entity).despawn_recursive();
            warn!("Cleaned up orphaned edge {:?}", edge_entity);
        }
    }
}

/// Resource for layout algorithms
#[derive(Resource, Default)]
pub struct GraphLayoutEngine {
    iterations: u32,
    temperature: f32,
}

impl GraphLayoutEngine {
    pub fn new() -> Self {
        Self {
            iterations: 0,
            temperature: 10.0,
        }
    }
    
    pub fn step(&mut self) {
        self.iterations += 1;
        self.temperature *= 0.99; // Cool down
    }
}

/// System to load graph from file
pub fn load_graph_from_file(
    mut commands: Commands,
    mut graph_ops: EventWriter<GraphOperationEvent>,
    file_path: String,
    graph_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(&file_path)?;
    
    let (nodes, edges) = if file_path.ends_with(".json") {
        graph_parser::parse_json_graph(&content)?
    } else if file_path.ends_with(".nix") {
        graph_parser::parse_nix_graph(&content)?
    } else if file_path.ends_with(".md") {
        graph_parser::parse_markdown_graph(&content)?
    } else {
        return Err("Unsupported file format".into());
    };
    
    // Create nodes
    let mut entity_map: HashMap<String, Entity> = HashMap::new();
    for node in nodes {
        let position = Vec3::new(node.position[0], node.position[1], node.position[2]);
        
        graph_ops.send(GraphOperationEvent {
            operation: GraphOperation::CreateNode {
                id: node.id.clone(),
                label: node.label,
                position,
            },
            graph_id: graph_id.clone(),
            entities: vec![],
        });
    }
    
    // Create edges (would need entity mapping in real implementation)
    for edge in edges {
        // This would look up actual entities from the node IDs
        // For now, we just log it
        info!("Would create edge: {} -> {}", edge.source_id, edge.target_id);
    }
    
    Ok(())
}