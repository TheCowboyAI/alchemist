use bevy::prelude::*;
use crate::domain::value_objects::{SubgraphId, NodeId, Position3D, GraphId};
use crate::domain::commands::{Command, SubgraphCommand};
use crate::presentation::components::{GraphNode, SubgraphMember, SubgraphOrigin};
use crate::application::CommandEvent;
use std::collections::HashSet;

/// Resource for tracking drag and drop state
#[derive(Resource, Default)]
pub struct DragDropState {
    pub dragging: Option<DraggingInfo>,
    pub drop_zones: Vec<DropZoneInfo>,
    pub hover_zone: Option<SubgraphId>,
}

#[derive(Debug, Clone)]
pub struct DraggingInfo {
    pub node_id: NodeId,
    pub entity: Entity,
    pub start_position: Vec3,
    pub current_position: Vec3,
    pub offset: Vec3,
    pub from_subgraph: Option<SubgraphId>,
}

#[derive(Debug, Clone)]
pub struct DropZoneInfo {
    pub subgraph_id: SubgraphId,
    pub center: Vec3,
    pub radius: f32,
    pub is_valid: bool,
}

/// Component marking a draggable node
#[derive(Component, Debug)]
pub struct Draggable {
    pub can_drag: bool,
    pub drag_constraints: DragConstraints,
}

#[derive(Debug, Clone)]
pub enum DragConstraints {
    None,
    AxisAligned { axes: Vec3 },
    WithinRadius { center: Vec3, radius: f32 },
    WithinBounds { min: Vec3, max: Vec3 },
}

impl Default for Draggable {
    fn default() -> Self {
        Self {
            can_drag: true,
            drag_constraints: DragConstraints::None,
        }
    }
}

/// Component marking a drop zone
#[derive(Component, Debug)]
pub struct DropZone {
    pub subgraph_id: SubgraphId,
    pub accepts: DropAcceptance,
    pub highlight_on_hover: bool,
}

#[derive(Debug, Clone)]
pub enum DropAcceptance {
    All,
    FromSubgraphs(HashSet<SubgraphId>),
    NotFromSubgraphs(HashSet<SubgraphId>),
    Custom(fn(&NodeId) -> bool),
}

/// Component for drag visualization
#[derive(Component)]
pub struct DragGhost {
    pub original_entity: Entity,
    pub offset: Vec3,
}

/// Handle drag start
pub fn handle_drag_start(
    mut state: ResMut<DragDropState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    draggables: Query<(Entity, &GraphNode, &Transform, &Draggable, Option<&SubgraphMember>)>,
) {
    // Check for drag initiation (left mouse + shift key)
    if mouse_button.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ShiftLeft) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Ok(window) = windows.get_single() {
                if let Some(cursor_pos) = window.cursor_position() {
                    let ray = camera.viewport_to_world(camera_transform, cursor_pos);
                    if let Ok(ray) = ray {
                        // Find the closest draggable node
                        let mut closest_hit = None;
                        let mut closest_distance = f32::MAX;

                        for (entity, node, transform, draggable, member) in draggables.iter() {
                            if !draggable.can_drag {
                                continue;
                            }

                            // Calculate sphere intersection manually
                            let sphere_center = transform.translation;
                            let sphere_radius = 0.5;

                            // Ray-sphere intersection
                            let oc = ray.origin - sphere_center;
                            let a = ray.direction.as_vec3().dot(ray.direction.as_vec3());
                            let b = 2.0 * oc.dot(ray.direction.as_vec3());
                            let c = oc.dot(oc) - sphere_radius * sphere_radius;
                            let discriminant = b * b - 4.0 * a * c;

                            if discriminant >= 0.0 {
                                let distance = (-b - discriminant.sqrt()) / (2.0 * a);
                                if distance >= 0.0 && distance < closest_distance {
                                    closest_distance = distance;
                                    closest_hit = Some((entity, node, transform, member));
                                }
                            }
                        }

                        if let Some((entity, node, transform, member)) = closest_hit {
                            // Calculate offset from node center to cursor
                            let hit_point = ray.origin + ray.direction.as_vec3() * closest_distance;
                            let offset = hit_point - transform.translation;

                            // Start dragging
                            state.dragging = Some(DraggingInfo {
                                node_id: node.node_id,
                                entity,
                                start_position: transform.translation,
                                current_position: transform.translation,
                                offset,
                                from_subgraph: member.and_then(|m| m.subgraph_ids.iter().next().copied()),
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Update drag position
pub fn update_drag(
    mut state: ResMut<DragDropState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut dragged_nodes: Query<(&mut Transform, &Draggable)>,
    drop_zones: Query<(&SubgraphOrigin, &Transform, &DropZone)>,
) {
    if state.dragging.is_some() {
        if mouse_button.pressed(MouseButton::Left) {
            // Update position based on cursor
            if let Ok((camera, camera_transform)) = camera_query.get_single() {
                if let Ok(window) = windows.get_single() {
                    if let Some(cursor_pos) = window.cursor_position() {
                        let ray = camera.viewport_to_world(camera_transform, cursor_pos);
                        if let Ok(ray) = ray {
                            // Get dragging info
                            let dragging_info = state.dragging.as_ref().unwrap();

                            // Project onto XZ plane (Y = current height)
                            let t = (dragging_info.current_position.y - ray.origin.y) / ray.direction.y;
                            let hit_point = ray.origin + ray.direction.as_vec3() * t;

                            // Apply offset
                            let new_position = hit_point - dragging_info.offset;

                            // Apply constraints
                            let mut constrained_position = new_position;
                            if let Ok((mut transform, draggable)) = dragged_nodes.get_mut(dragging_info.entity) {
                                constrained_position = apply_drag_constraints(
                                    new_position,
                                    &draggable.drag_constraints,
                                    dragging_info.start_position,
                                );

                                transform.translation = constrained_position;
                            }

                            // Collect drop zone info
                            let mut new_drop_zones = Vec::new();
                            let mut new_hover_zone = None;

                            for (origin, zone_transform, drop_zone) in drop_zones.iter() {
                                let distance = constrained_position.distance(zone_transform.translation);
                                let radius = 2.0; // Drop zone radius

                                let is_valid = match &drop_zone.accepts {
                                    DropAcceptance::All => true,
                                    DropAcceptance::FromSubgraphs(allowed) => {
                                        dragging_info.from_subgraph.map_or(false, |id| allowed.contains(&id))
                                    }
                                    DropAcceptance::NotFromSubgraphs(forbidden) => {
                                        dragging_info.from_subgraph.map_or(true, |id| !forbidden.contains(&id))
                                    }
                                    DropAcceptance::Custom(_) => true, // TODO: Implement custom logic
                                };

                                new_drop_zones.push(DropZoneInfo {
                                    subgraph_id: origin.subgraph_id,
                                    center: zone_transform.translation,
                                    radius,
                                    is_valid,
                                });

                                if distance < radius && is_valid {
                                    new_hover_zone = Some(origin.subgraph_id);
                                }
                            }

                            // Update state
                            if let Some(ref mut dragging) = state.dragging {
                                dragging.current_position = constrained_position;
                            }
                            state.hover_zone = new_hover_zone;
                            state.drop_zones = new_drop_zones;
                        }
                    }
                }
            }
        } else {
            // Mouse released, end drag
            state.dragging = None;
        }
    }
}

/// Handle drop
pub fn handle_drop(
    mut state: ResMut<DragDropState>,
    mut events: EventWriter<CommandEvent>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    if mouse_button.just_released(MouseButton::Left) {
        if let Some(dragging) = state.dragging.take() {
            // Check if dropped on a valid zone
            if let Some(target_subgraph) = state.hover_zone {
                // Calculate relative position within the target subgraph
                let zone_info = state.drop_zones.iter()
                    .find(|z| z.subgraph_id == target_subgraph);

                let relative_pos = if let Some(zone) = zone_info {
                    Position3D {
                        x: dragging.current_position.x - zone.center.x,
                        y: dragging.current_position.y - zone.center.y,
                        z: dragging.current_position.z - zone.center.z,
                    }
                } else {
                    Position3D {
                        x: dragging.current_position.x,
                        y: dragging.current_position.y,
                        z: dragging.current_position.z,
                    }
                };

                // Generate move command
                events.send(CommandEvent {
                    command: Command::Subgraph(
                        SubgraphCommand::MoveNodeBetweenSubgraphs {
                            graph_id: GraphId::new(), // TODO: Get actual graph ID
                            node_id: dragging.node_id,
                            from_subgraph: dragging.from_subgraph.unwrap(),
                            to_subgraph: target_subgraph,
                            new_relative_position: relative_pos,
                        }
                    ),
                });
            } else if dragging.from_subgraph.is_some() {
                // Dropped outside any zone - remove from subgraph
                events.send(CommandEvent {
                    command: Command::Subgraph(
                        SubgraphCommand::RemoveNodeFromSubgraph {
                            graph_id: GraphId::new(), // TODO: Get actual graph ID
                            subgraph_id: dragging.from_subgraph.unwrap(),
                            node_id: dragging.node_id,
                        }
                    ),
                });
            }

            // Clear state
            state.hover_zone = None;
            state.drop_zones.clear();
        }
    }
}

/// Visualize drop zones
pub fn visualize_drop_zones(
    state: Res<DragDropState>,
    mut gizmos: Gizmos,
) {
    if state.dragging.is_some() {
        for zone in &state.drop_zones {
            let color = if Some(zone.subgraph_id) == state.hover_zone {
                if zone.is_valid {
                    Color::srgb(0.0, 1.0, 0.0).with_alpha(0.5) // Green for valid hover
                } else {
                    Color::srgb(1.0, 0.0, 0.0).with_alpha(0.5) // Red for invalid
                }
            } else if zone.is_valid {
                Color::srgb(0.5, 0.5, 1.0).with_alpha(0.3) // Blue for valid zones
            } else {
                Color::srgb(0.5, 0.5, 0.5).with_alpha(0.2) // Gray for invalid zones
            };

            // Draw circle on XZ plane
            gizmos.circle(
                Isometry3d::from_translation(zone.center),
                zone.radius,
                color,
            );

            // Draw vertical cylinder outline
            let segments = 32;
            for i in 0..segments {
                let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
                let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

                let p1 = zone.center + Vec3::new(
                    angle1.cos() * zone.radius,
                    0.0,
                    angle1.sin() * zone.radius,
                );
                let p2 = zone.center + Vec3::new(
                    angle2.cos() * zone.radius,
                    0.0,
                    angle2.sin() * zone.radius,
                );

                gizmos.line(p1, p2, color);
                gizmos.line(p1 + Vec3::Y * 0.5, p2 + Vec3::Y * 0.5, color);
            }
        }

        // Draw drag line
        if let Some(ref dragging) = state.dragging {
            gizmos.line(
                dragging.start_position,
                dragging.current_position,
                Color::srgb(1.0, 1.0, 0.0).with_alpha(0.5),
            );
        }
    }
}

/// Apply drag constraints
fn apply_drag_constraints(
    position: Vec3,
    constraints: &DragConstraints,
    start_position: Vec3,
) -> Vec3 {
    match constraints {
        DragConstraints::None => position,

        DragConstraints::AxisAligned { axes } => {
            Vec3::new(
                if axes.x > 0.0 { position.x } else { start_position.x },
                if axes.y > 0.0 { position.y } else { start_position.y },
                if axes.z > 0.0 { position.z } else { start_position.z },
            )
        }

        DragConstraints::WithinRadius { center, radius } => {
            let offset = position - *center;
            if offset.length() > *radius {
                *center + offset.normalize() * *radius
            } else {
                position
            }
        }

        DragConstraints::WithinBounds { min, max } => {
            position.clamp(*min, *max)
        }
    }
}

/// Create drag ghost visual
pub fn create_drag_ghost(
    mut commands: Commands,
    state: Res<DragDropState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    nodes: Query<&Transform, With<GraphNode>>,
) {
    if let Some(ref dragging) = state.dragging {
        if state.is_changed() && dragging.entity != Entity::PLACEHOLDER {
            if let Ok(transform) = nodes.get(dragging.entity) {
                // Create semi-transparent ghost
                let ghost = commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.5).mesh())),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 1.0, 1.0).with_alpha(0.3),
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                    Transform::from_translation(transform.translation),
                    DragGhost {
                        original_entity: dragging.entity,
                        offset: dragging.offset,
                    },
                )).id();
            }
        }
    }
}

/// Clean up drag ghosts
pub fn cleanup_drag_ghosts(
    mut commands: Commands,
    state: Res<DragDropState>,
    ghosts: Query<Entity, With<DragGhost>>,
) {
    if state.dragging.is_none() {
        for entity in ghosts.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Plugin for drag and drop functionality
pub struct SubgraphDragDropPlugin;

impl Plugin for SubgraphDragDropPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DragDropState>()
            .add_systems(
                Update,
                (
                    handle_drag_start,
                    update_drag,
                    handle_drop,
                    visualize_drop_zones,
                    create_drag_ghost,
                    cleanup_drag_ghosts,
                )
                    .chain(),
            );
    }
}

