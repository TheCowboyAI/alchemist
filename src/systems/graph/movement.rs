//! Systems for node movement and positioning
//!
//! These systems handle:
//! - Node dragging and movement
//! - Snap-to-grid functionality
//! - Movement constraints
//! - Batch movement operations

use bevy::prelude::*;

use crate::{
    components::*,
    events::*,
    resources::*,
};

/// System that handles node movement events
///
/// This system applies position changes from MoveNodeEvent
pub fn handle_node_movement(
    mut events: EventReader<MoveNodeEvent>,
    mut transforms: Query<&mut Transform>,
    mut modification_events: EventWriter<GraphModificationEvent>,
    nodes: Query<&NodeId>,
) {
    for event in events.read() {
        if let Ok(mut transform) = transforms.get_mut(event.entity) {
            // Apply movement
            transform.translation = event.to;

            // Send modification event for undo
            if let Ok(node_id) = nodes.get(event.entity) {
                modification_events.send(GraphModificationEvent::NodeMoved {
                    id: node_id.0,
                    from: event.from,
                    to: event.to,
                });
            }
        }
    }
}

/// System that handles batch node movements
///
/// This system processes multiple node movements as a single operation
pub fn handle_batch_movement(
    mut events: EventReader<BatchMoveNodesEvent>,
    mut transforms: Query<&mut Transform>,
    mut move_events: EventWriter<MoveNodeEvent>,
) {
    for event in events.read() {
        for (entity, from, to) in &event.moves {
            // Update transform directly for efficiency
            if let Ok(mut transform) = transforms.get_mut(*entity) {
                transform.translation = *to;
            }

            // Send individual move events for tracking
            move_events.send(MoveNodeEvent {
                entity: *entity,
                from: *from,
                to: *to,
            });
        }
    }
}

/// System that handles node dragging with mouse
///
/// This system allows interactive dragging of selected nodes
pub fn handle_node_dragging(
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    selected_nodes: Query<(Entity, &Transform), (With<Selected>, With<NodeInteractable>)>,
    mut drag_state: ResMut<DragState>,
    mut move_events: EventWriter<MoveNodeEvent>,
    grid_settings: Res<GridSettings>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            // Calculate drag plane intersection
            let drag_plane_normal = Vec3::Y;
            let drag_plane_distance = 0.0;
            let denominator = ray.direction.dot(drag_plane_normal);

            if denominator.abs() > 0.0001 {
                let t = (drag_plane_distance - ray.origin.dot(drag_plane_normal)) / denominator;
                let world_pos = ray.origin + ray.direction * t;

                if mouse.just_pressed(MouseButton::Left) && !selected_nodes.is_empty() {
                    // Start dragging
                    drag_state.active = true;
                    drag_state.start_pos = world_pos;
                    drag_state.node_offsets.clear();

                    // Store initial offsets for each selected node
                    for (entity, transform) in selected_nodes.iter() {
                        let offset = transform.translation - world_pos;
                        drag_state.node_offsets.insert(entity, offset);
                    }
                } else if mouse.pressed(MouseButton::Left) && drag_state.active {
                    // Continue dragging
                    let delta = world_pos - drag_state.start_pos;

                    for (entity, _) in selected_nodes.iter() {
                        if let Some(offset) = drag_state.node_offsets.get(&entity) {
                            let mut new_pos = drag_state.start_pos + delta + *offset;

                            // Apply grid snapping if enabled
                            if grid_settings.snap_enabled {
                                new_pos = snap_to_grid(new_pos, grid_settings.grid_size);
                            }

                            move_events.send(MoveNodeEvent {
                                entity,
                                from: drag_state.start_pos + *offset,
                                to: new_pos,
                            });
                        }
                    }
                } else if mouse.just_released(MouseButton::Left) && drag_state.active {
                    // End dragging
                    drag_state.active = false;
                    drag_state.node_offsets.clear();
                }
            }
        }
    }
}

/// System that applies movement constraints
///
/// This system ensures nodes stay within defined boundaries
pub fn apply_movement_constraints(
    mut moved_nodes: Query<(&mut Transform, &NodeId), Changed<Transform>>,
    graph_bounds: Res<GraphBounds>,
    mut move_events: EventWriter<MoveNodeEvent>,
) {
    for (mut transform, _) in moved_nodes.iter_mut() {
        let original = transform.translation;
        let mut constrained = original;

        // Apply boundary constraints
        if let Some((min, max)) = graph_bounds.bounds {
            constrained.x = constrained.x.clamp(min.x, max.x);
            constrained.y = constrained.y.clamp(min.y, max.y);
            constrained.z = constrained.z.clamp(min.z, max.z);
        }

        // If position was constrained, update it
        if constrained != original {
            transform.translation = constrained;
        }
    }
}

/// System that handles alignment operations
///
/// This system aligns selected nodes based on various criteria
pub fn handle_alignment(
    keyboard: Res<Input<KeyCode>>,
    selected: Query<(Entity, &Transform), With<Selected>>,
    mut batch_move_events: EventWriter<BatchMoveNodesEvent>,
) {
    let nodes: Vec<(Entity, Vec3)> = selected.iter()
        .map(|(e, t)| (e, t.translation))
        .collect();

    if nodes.len() < 2 {
        return;
    }

    let mut moves = Vec::new();

    // Horizontal alignment (Ctrl+Shift+H)
    if keyboard.pressed(KeyCode::ControlLeft)
        && keyboard.pressed(KeyCode::ShiftLeft)
        && keyboard.just_pressed(KeyCode::H)
    {
        let avg_y = nodes.iter().map(|(_, p)| p.y).sum::<f32>() / nodes.len() as f32;

        for (entity, pos) in &nodes {
            let new_pos = Vec3::new(pos.x, avg_y, pos.z);
            moves.push((*entity, *pos, new_pos));
        }
    }

    // Vertical alignment (Ctrl+Shift+V)
    if keyboard.pressed(KeyCode::ControlLeft)
        && keyboard.pressed(KeyCode::ShiftLeft)
        && keyboard.just_pressed(KeyCode::V)
    {
        let avg_x = nodes.iter().map(|(_, p)| p.x).sum::<f32>() / nodes.len() as f32;

        for (entity, pos) in &nodes {
            let new_pos = Vec3::new(avg_x, pos.y, pos.z);
            moves.push((*entity, *pos, new_pos));
        }
    }

    // Distribute horizontally (Ctrl+Shift+D)
    if keyboard.pressed(KeyCode::ControlLeft)
        && keyboard.pressed(KeyCode::ShiftLeft)
        && keyboard.just_pressed(KeyCode::D)
    {
        let mut sorted_nodes = nodes.clone();
        sorted_nodes.sort_by(|a, b| a.1.x.partial_cmp(&b.1.x).unwrap());

        let min_x = sorted_nodes.first().unwrap().1.x;
        let max_x = sorted_nodes.last().unwrap().1.x;
        let spacing = (max_x - min_x) / (sorted_nodes.len() - 1) as f32;

        for (i, (entity, pos)) in sorted_nodes.iter().enumerate() {
            let new_x = min_x + spacing * i as f32;
            let new_pos = Vec3::new(new_x, pos.y, pos.z);
            moves.push((*entity, *pos, new_pos));
        }
    }

    if !moves.is_empty() {
        batch_move_events.send(BatchMoveNodesEvent { moves });
    }
}

/// System that handles arrow key movement
///
/// This system allows moving selected nodes with arrow keys
pub fn handle_arrow_key_movement(
    keyboard: Res<Input<KeyCode>>,
    selected: Query<(Entity, &Transform), With<Selected>>,
    mut move_events: EventWriter<MoveNodeEvent>,
    grid_settings: Res<GridSettings>,
) {
    if selected.is_empty() {
        return;
    }

    let mut movement = Vec3::ZERO;
    let move_distance = if keyboard.pressed(KeyCode::ShiftLeft) {
        grid_settings.grid_size * 5.0 // Fast movement
    } else if keyboard.pressed(KeyCode::ControlLeft) {
        1.0 // Precise movement
    } else {
        grid_settings.grid_size // Normal movement
    };

    // Check arrow keys
    if keyboard.just_pressed(KeyCode::Left) {
        movement.x -= move_distance;
    }
    if keyboard.just_pressed(KeyCode::Right) {
        movement.x += move_distance;
    }
    if keyboard.just_pressed(KeyCode::Up) {
        movement.z -= move_distance;
    }
    if keyboard.just_pressed(KeyCode::Down) {
        movement.z += move_distance;
    }

    if movement != Vec3::ZERO {
        for (entity, transform) in selected.iter() {
            let from = transform.translation;
            let to = from + movement;

            move_events.send(MoveNodeEvent {
                entity,
                from,
                to,
            });
        }
    }
}

/// System that updates edge positions based on node movements
///
/// This system ensures edges follow their connected nodes
pub fn update_edge_positions(
    moved_nodes: Query<&Transform, (Changed<Transform>, With<NodeId>)>,
    edges: Query<(&Edge, &Handle<Mesh>)>,
    nodes: Query<&Transform, With<NodeId>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (edge, mesh_handle) in edges.iter() {
        // Check if either connected node has moved
        let source_moved = moved_nodes.get(edge.source).is_ok();
        let target_moved = moved_nodes.get(edge.target).is_ok();

        if source_moved || target_moved {
            // Get current positions
            if let (Ok(source_transform), Ok(target_transform)) = (
                nodes.get(edge.source),
                nodes.get(edge.target)
            ) {
                // Update edge mesh
                if let Some(mesh) = meshes.get_mut(mesh_handle) {
                    *mesh = create_edge_mesh(
                        source_transform.translation,
                        target_transform.translation,
                    );
                }
            }
        }
    }
}

// Helper functions

fn snap_to_grid(position: Vec3, grid_size: f32) -> Vec3 {
    Vec3::new(
        (position.x / grid_size).round() * grid_size,
        position.y, // Don't snap Y axis
        (position.z / grid_size).round() * grid_size,
    )
}

fn create_edge_mesh(start: Vec3, end: Vec3) -> Mesh {
    let direction = end - start;
    let length = direction.length();
    let midpoint = start + direction * 0.5;

    // Create a box mesh oriented along the edge
    let mut mesh = Mesh::from(shape::Box::new(0.1, 0.1, length));

    // Transform vertices to align with edge direction
    // This is simplified - in practice would need proper rotation calculation

    mesh
}
