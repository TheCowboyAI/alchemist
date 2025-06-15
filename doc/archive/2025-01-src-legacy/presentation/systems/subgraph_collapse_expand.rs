use crate::application::CommandEvent;
use crate::domain::commands::{Command, SubgraphOperationCommand};
use crate::domain::value_objects::{GraphId, LayoutStrategy, NodeId, SubgraphId};
use crate::presentation::components::{GraphNode, SubgraphMember, SubgraphOrigin};
use bevy::prelude::*;
use std::collections::HashMap;

/// Component marking a collapsed subgraph
#[derive(Component, Debug, Clone)]
pub struct CollapsedSubgraph {
    pub subgraph_id: SubgraphId,
    pub node_count: usize,
    pub edge_count: usize,
    pub collapsed_at: f64,
}

/// Component for collapse animation
#[derive(Component, Debug)]
pub struct CollapseAnimation {
    pub subgraph_id: SubgraphId,
    pub start_time: f64,
    pub duration: f32,
    pub node_positions: HashMap<NodeId, Vec3>,
    pub target_position: Vec3,
    pub phase: CollapsePhase,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollapsePhase {
    Collapsing,
    Collapsed,
    Expanding,
    Expanded,
}

/// Component for expand animation
#[derive(Component, Debug)]
pub struct ExpandAnimation {
    pub subgraph_id: SubgraphId,
    pub start_time: f64,
    pub duration: f32,
    pub original_positions: HashMap<NodeId, Vec3>,
    pub from_position: Vec3,
}

/// Resource for tracking collapse/expand state
#[derive(Resource, Default)]
pub struct CollapseExpandState {
    pub collapsed_subgraphs: HashMap<SubgraphId, CollapsedSubgraphInfo>,
    pub animating: HashMap<SubgraphId, AnimationState>,
}

#[derive(Debug, Clone)]
pub struct CollapsedSubgraphInfo {
    pub node_positions: HashMap<NodeId, Vec3>,
    pub center_position: Vec3,
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Debug, Clone)]
pub enum AnimationState {
    Collapsing { progress: f32 },
    Expanding { progress: f32 },
}

/// Detect collapse trigger (double-click on subgraph origin)
pub fn detect_collapse_trigger(
    mut commands: Commands,
    mut events: EventWriter<CommandEvent>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut double_click_timer: Local<Option<f64>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    subgraph_origins: Query<(Entity, &SubgraphOrigin, &Transform), Without<CollapsedSubgraph>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let current_time = time.elapsed_secs_f64();

        // Check for double-click
        if let Some(last_click) = *double_click_timer {
            if current_time - last_click < 0.3 {
                // Double-click detected, check if on subgraph origin
                if let Ok((camera, camera_transform)) = camera_query.get_single() {
                    if let Ok(window) = windows.get_single() {
                        if let Some(cursor_position) = window.cursor_position() {
                            let ray = camera.viewport_to_world(camera_transform, cursor_position);

                            if let Ok(ray) = ray {
                                // Check intersection with subgraph origins
                                for (entity, origin, transform) in subgraph_origins.iter() {
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
                                        if distance >= 0.0 {
                                            // Trigger collapse command
                                            events.send(CommandEvent {
                                                command: Command::SubgraphOperation(
                                                    SubgraphOperationCommand::CollapseSubgraph {
                                                        graph_id: GraphId::new(), // TODO: Get actual graph ID
                                                        subgraph_id: origin.subgraph_id,
                                                        strategy: crate::domain::value_objects::CollapseStrategy::Centroid,
                                                    }
                                                ),
                                            });

                                            // Add visual feedback
                                            commands.entity(entity).insert(CollapsedSubgraph {
                                                subgraph_id: origin.subgraph_id,
                                                node_count: 0, // Will be updated
                                                edge_count: 0, // Will be updated
                                                collapsed_at: current_time,
                                            });

                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                *double_click_timer = None;
            } else {
                *double_click_timer = Some(current_time);
            }
        } else {
            *double_click_timer = Some(current_time);
        }
    }
}

/// Execute collapse animation
pub fn execute_collapse(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<CollapseExpandState>,
    nodes: Query<(Entity, &GraphNode, &SubgraphMember, &Transform)>,
    origins: Query<(&SubgraphOrigin, &Transform)>,
    mut collapse_events: EventReader<CommandEvent>,
) {
    for event in collapse_events.read() {
        if let Command::SubgraphOperation(SubgraphOperationCommand::CollapseSubgraph {
            graph_id,
            subgraph_id,
            strategy,
        }) = &event.command
        {
            // Find all nodes in this subgraph
            let mut node_positions = HashMap::new();
            let mut node_entities = Vec::new();
            let mut center = Vec3::ZERO;
            let mut count = 0;

            for (entity, node, member, transform) in nodes.iter() {
                if member.subgraph_ids.contains(subgraph_id) {
                    node_positions.insert(node.node_id, transform.translation);
                    node_entities.push(entity);
                    center += transform.translation;
                    count += 1;
                }
            }

            if count > 0 {
                center /= count as f32;

                // Find origin position or use center
                let target_position = origins
                    .iter()
                    .find(|(origin, _)| origin.subgraph_id == *subgraph_id)
                    .map(|(_, transform)| transform.translation)
                    .unwrap_or(center);

                // Start collapse animation
                for entity in node_entities {
                    commands.entity(entity).insert(CollapseAnimation {
                        subgraph_id: *subgraph_id,
                        start_time: time.elapsed_secs_f64(),
                        duration: 0.5,
                        node_positions: node_positions.clone(),
                        target_position,
                        phase: CollapsePhase::Collapsing,
                    });
                }

                // Store collapsed state
                state.collapsed_subgraphs.insert(
                    *subgraph_id,
                    CollapsedSubgraphInfo {
                        node_positions,
                        center_position: target_position,
                        node_count: count,
                        edge_count: 0, // TODO: Count edges
                    },
                );

                state
                    .animating
                    .insert(*subgraph_id, AnimationState::Collapsing { progress: 0.0 });
            }
        }
    }
}

/// Animate collapse transition
pub fn animate_collapse(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<CollapseExpandState>,
    mut query: Query<(Entity, &mut Transform, &mut CollapseAnimation, &GraphNode)>,
) {
    let current_time = time.elapsed_secs_f64();

    for (entity, mut transform, mut animation, node) in query.iter_mut() {
        if animation.phase == CollapsePhase::Collapsing {
            let elapsed = (current_time - animation.start_time) as f32;
            let progress = (elapsed / animation.duration).clamp(0.0, 1.0);

            // Ease-out cubic
            let t = 1.0 - (1.0 - progress).powi(3);

            // Interpolate position
            if let Some(original_pos) = animation.node_positions.get(&node.node_id) {
                transform.translation = original_pos.lerp(animation.target_position, t);

                // Scale down as we collapse
                let scale = 1.0 - (t * 0.8); // Scale down to 20% of original
                transform.scale = Vec3::splat(scale);
            }

            // Update state
            if let Some(anim_state) = state.animating.get_mut(&animation.subgraph_id) {
                *anim_state = AnimationState::Collapsing { progress: t };
            }

            // Complete animation
            if progress >= 1.0 {
                animation.phase = CollapsePhase::Collapsed;
                // Hide the node
                commands.entity(entity).insert(Visibility::Hidden);
                state.animating.remove(&animation.subgraph_id);
            }
        }
    }
}

/// Detect expand trigger (double-click on collapsed subgraph)
pub fn detect_expand_trigger(
    mut events: EventWriter<CommandEvent>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut double_click_timer: Local<Option<f64>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    collapsed_origins: Query<(&SubgraphOrigin, &Transform, &CollapsedSubgraph)>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let current_time = time.elapsed_secs_f64();

        // Check for double-click
        if let Some(last_click) = *double_click_timer {
            if current_time - last_click < 0.3 {
                // Double-click detected, check if on collapsed subgraph
                if let Ok((camera, camera_transform)) = camera_query.get_single() {
                    if let Ok(window) = windows.get_single() {
                        if let Some(cursor_position) = window.cursor_position() {
                            let ray = camera.viewport_to_world(camera_transform, cursor_position);

                            if let Ok(ray) = ray {
                                // Check intersection with collapsed subgraphs
                                for (origin, transform, collapsed) in collapsed_origins.iter() {
                                    // Calculate sphere intersection manually
                                    let sphere_center = transform.translation;
                                    let sphere_radius = 1.0;

                                    // Ray-sphere intersection
                                    let oc = ray.origin - sphere_center;
                                    let a = ray.direction.as_vec3().dot(ray.direction.as_vec3());
                                    let b = 2.0 * oc.dot(ray.direction.as_vec3());
                                    let c = oc.dot(oc) - sphere_radius * sphere_radius;
                                    let discriminant = b * b - 4.0 * a * c;

                                    if discriminant >= 0.0 {
                                        let distance = (-b - discriminant.sqrt()) / (2.0 * a);
                                        if distance >= 0.0 {
                                            // Trigger expand command
                                            events.send(CommandEvent {
                                                command: Command::SubgraphOperation(
                                                    SubgraphOperationCommand::ExpandSubgraph {
                                                        graph_id: GraphId::new(), // TODO: Get actual graph ID
                                                        subgraph_id: origin.subgraph_id,
                                                        layout: LayoutStrategy::ForceDirected {
                                                            iterations: 100,
                                                            spring_strength: 0.1,
                                                            repulsion_strength: 100.0,
                                                        },
                                                    },
                                                ),
                                            });
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                *double_click_timer = None;
            } else {
                *double_click_timer = Some(current_time);
            }
        } else {
            *double_click_timer = Some(current_time);
        }
    }
}

/// Execute expand animation
pub fn execute_expand(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<CollapseExpandState>,
    mut nodes: Query<(
        Entity,
        &mut Visibility,
        &GraphNode,
        &SubgraphMember,
        &mut Transform,
    )>,
    collapsed_origins: Query<(Entity, &SubgraphOrigin, &CollapsedSubgraph)>,
    mut expand_events: EventReader<CommandEvent>,
) {
    for event in expand_events.read() {
        if let Command::SubgraphOperation(SubgraphOperationCommand::ExpandSubgraph {
            graph_id,
            subgraph_id,
            layout,
        }) = &event.command
        {
            // Get collapsed info
            if let Some(collapsed_info) = state.collapsed_subgraphs.get(subgraph_id) {
                // Remove collapsed marker from origin
                for (entity, origin, _) in collapsed_origins.iter() {
                    if origin.subgraph_id == *subgraph_id {
                        commands.entity(entity).remove::<CollapsedSubgraph>();
                        break;
                    }
                }

                // Start expand animation for all nodes
                for (entity, mut visibility, node, member, mut transform) in nodes.iter_mut() {
                    if member.subgraph_ids.contains(subgraph_id) {
                        // Make visible again
                        *visibility = Visibility::Visible;

                        // Start from collapsed position
                        transform.translation = collapsed_info.center_position;
                        transform.scale = Vec3::splat(0.2);

                        // Add expand animation
                        commands.entity(entity).insert(ExpandAnimation {
                            subgraph_id: *subgraph_id,
                            start_time: time.elapsed_secs_f64(),
                            duration: 0.5,
                            original_positions: collapsed_info.node_positions.clone(),
                            from_position: collapsed_info.center_position,
                        });
                    }
                }

                state
                    .animating
                    .insert(*subgraph_id, AnimationState::Expanding { progress: 0.0 });
            }
        }
    }
}

/// Animate expand transition
pub fn animate_expand(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<CollapseExpandState>,
    mut query: Query<(Entity, &mut Transform, &ExpandAnimation, &GraphNode)>,
) {
    let current_time = time.elapsed_secs_f64();
    let mut completed = Vec::new();

    for (entity, mut transform, animation, node) in query.iter_mut() {
        let elapsed = (current_time - animation.start_time) as f32;
        let progress = (elapsed / animation.duration).clamp(0.0, 1.0);

        // Ease-out cubic
        let t = 1.0 - (1.0 - progress).powi(3);

        // Interpolate position
        if let Some(target_pos) = animation.original_positions.get(&node.node_id) {
            transform.translation = animation.from_position.lerp(*target_pos, t);

            // Scale up as we expand
            let scale = 0.2 + (t * 0.8); // Scale from 20% to 100%
            transform.scale = Vec3::splat(scale);
        }

        // Update state
        if let Some(anim_state) = state.animating.get_mut(&animation.subgraph_id) {
            *anim_state = AnimationState::Expanding { progress: t };
        }

        // Complete animation
        if progress >= 1.0 {
            completed.push((entity, animation.subgraph_id));
        }
    }

    // Clean up completed animations
    for (entity, subgraph_id) in completed {
        commands.entity(entity).remove::<ExpandAnimation>();
        state.animating.remove(&subgraph_id);
        state.collapsed_subgraphs.remove(&subgraph_id);
    }
}

/// Plugin for collapse/expand functionality
pub struct SubgraphCollapseExpandPlugin;

impl Plugin for SubgraphCollapseExpandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CollapseExpandState>().add_systems(
            Update,
            (
                detect_collapse_trigger,
                execute_collapse,
                animate_collapse,
                detect_expand_trigger,
                execute_expand,
                animate_expand,
            )
                .chain(),
        );
    }
}
