use crate::application::CommandEvent;
use crate::domain::commands::{Command, SubgraphOperationCommand};
use crate::domain::value_objects::{GraphId, MergeStrategy, NodeId, Position3D, SubgraphId};
use crate::presentation::components::{GraphNode, SubgraphMember, SubgraphOrigin};
use bevy::prelude::*;
use std::collections::HashSet;

/// Resource for tracking merge state
#[derive(Resource, Default)]
pub struct MergeState {
    pub selected_subgraphs: HashSet<SubgraphId>,
    pub preview_active: bool,
    pub merge_center: Option<Vec3>,
    pub can_merge: bool,
}

/// Resource for tracking split state
#[derive(Resource, Default)]
pub struct SplitState {
    pub splitting_subgraph: Option<SubgraphId>,
    pub split_line: Option<SplitLine>,
    pub preview_groups: Vec<NodeGroup>,
    pub can_split: bool,
}

#[derive(Debug, Clone)]
pub struct SplitLine {
    pub start: Vec3,
    pub end: Vec3,
    pub normal: Vec3,
}

#[derive(Debug, Clone)]
pub struct NodeGroup {
    pub nodes: HashSet<NodeId>,
    pub center: Vec3,
    pub color: Color,
}

/// Component for merge preview visualization
#[derive(Component)]
pub struct MergePreview {
    pub subgraphs: Vec<SubgraphId>,
    pub center: Vec3,
    pub radius: f32,
}

/// Component for split preview visualization
#[derive(Component)]
pub struct SplitPreview {
    pub subgraph_id: SubgraphId,
    pub groups: Vec<NodeGroup>,
}

/// Handle multi-selection for merge
pub fn handle_multi_selection(
    mut merge_state: ResMut<MergeState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    subgraph_origins: Query<(&SubgraphOrigin, &Transform)>,
) {
    // Ctrl+Click for multi-selection
    if mouse_button.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ControlLeft) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Ok(window) = windows.get_single() {
                if let Some(cursor_position) = window.cursor_position() {
                    let ray = camera.viewport_to_world(camera_transform, cursor_position);

                    if let Ok(ray) = ray {
                        // Check intersection with subgraph origins
                        for (origin, transform) in subgraph_origins.iter() {
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
                                    // Toggle selection
                                    if merge_state.selected_subgraphs.contains(&origin.subgraph_id)
                                    {
                                        merge_state.selected_subgraphs.remove(&origin.subgraph_id);
                                    } else {
                                        merge_state.selected_subgraphs.insert(origin.subgraph_id);
                                    }

                                    // Update merge capability
                                    merge_state.can_merge =
                                        merge_state.selected_subgraphs.len() >= 2;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Clear selection on regular click
    if mouse_button.just_pressed(MouseButton::Left) && !keyboard.pressed(KeyCode::ControlLeft) {
        merge_state.selected_subgraphs.clear();
        merge_state.can_merge = false;
        merge_state.preview_active = false;
    }
}

/// Preview merge operation
pub fn preview_merge(
    mut commands: Commands,
    merge_state: Res<MergeState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    subgraph_origins: Query<(&SubgraphOrigin, &Transform)>,
    nodes: Query<(&GraphNode, &SubgraphMember, &Transform)>,
    preview_query: Query<Entity, With<MergePreview>>,
) {
    // Show preview when M is held with multiple selections
    if keyboard.pressed(KeyCode::KeyM) && merge_state.can_merge {
        if !merge_state.preview_active {
            // Calculate merge center
            let mut center = Vec3::ZERO;
            let mut count = 0;

            for (origin, transform) in subgraph_origins.iter() {
                if merge_state.selected_subgraphs.contains(&origin.subgraph_id) {
                    center += transform.translation;
                    count += 1;
                }
            }

            if count > 0 {
                center /= count as f32;

                // Calculate radius
                let mut max_distance = 0.0f32;
                for (_, member, transform) in nodes.iter() {
                    if member
                        .subgraph_ids
                        .iter()
                        .any(|id| merge_state.selected_subgraphs.contains(id))
                    {
                        let distance = transform.translation.distance(center);
                        max_distance = max_distance.max(distance);
                    }
                }

                // Create preview entity
                commands.spawn(MergePreview {
                    subgraphs: merge_state.selected_subgraphs.iter().copied().collect(),
                    center,
                    radius: max_distance + 1.0,
                });
            }
        }
    } else {
        // Remove preview
        for entity in preview_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Execute merge operation
pub fn execute_merge(
    mut events: EventWriter<CommandEvent>,
    merge_state: Res<MergeState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Execute merge on M key release
    if keyboard.just_released(KeyCode::KeyM) && merge_state.can_merge {
        let subgraphs: Vec<SubgraphId> = merge_state.selected_subgraphs.iter().copied().collect();

        if subgraphs.len() >= 2 {
            events.send(CommandEvent {
                command: Command::SubgraphOperation(SubgraphOperationCommand::MergeSubgraphs {
                    graph_id: GraphId::new(), // TODO: Get actual graph ID
                    source_subgraphs: subgraphs,
                    target_subgraph_id: SubgraphId::new(),
                    strategy: MergeStrategy::Union,
                }),
            });
        }
    }
}

/// Detect split gesture (draw line across subgraph)
pub fn detect_split_gesture(
    mut split_state: ResMut<SplitState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    subgraph_origins: Query<(&SubgraphOrigin, &Transform)>,
    nodes: Query<(&GraphNode, &SubgraphMember, &Transform)>,
) {
    // Alt+Drag to draw split line
    if keyboard.pressed(KeyCode::AltLeft) {
        if mouse_button.just_pressed(MouseButton::Left) {
            // Start drawing split line
            if let Ok((camera, camera_transform)) = camera_query.get_single() {
                if let Ok(window) = windows.get_single() {
                    if let Some(cursor_position) = window.cursor_position() {
                        let ray = camera.viewport_to_world(camera_transform, cursor_position);

                        if let Ok(ray) = ray {
                            // Find which subgraph we're splitting
                            for (origin, transform) in subgraph_origins.iter() {
                                // Calculate sphere intersection manually
                                let sphere_center = transform.translation;
                                let sphere_radius = 2.0;

                                // Ray-sphere intersection
                                let oc = ray.origin - sphere_center;
                                let a = ray.direction.as_vec3().dot(ray.direction.as_vec3());
                                let b = 2.0 * oc.dot(ray.direction.as_vec3());
                                let c = oc.dot(oc) - sphere_radius * sphere_radius;
                                let discriminant = b * b - 4.0 * a * c;

                                if discriminant >= 0.0 {
                                    let distance = (-b - discriminant.sqrt()) / (2.0 * a);
                                    if distance >= 0.0 {
                                        split_state.splitting_subgraph = Some(origin.subgraph_id);

                                        // Project onto XZ plane
                                        let t = -ray.origin.y / ray.direction.y;
                                        let hit_point = ray.origin + ray.direction * t;

                                        split_state.split_line = Some(SplitLine {
                                            start: hit_point,
                                            end: hit_point,
                                            normal: Vec3::ZERO,
                                        });
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if mouse_button.pressed(MouseButton::Left) {
            // Update split line end point
            if let Some(ref mut split_line) = split_state.split_line {
                if let Ok((camera, camera_transform)) = camera_query.get_single() {
                    if let Ok(window) = windows.get_single() {
                        if let Some(cursor_position) = window.cursor_position() {
                            let ray = camera.viewport_to_world(camera_transform, cursor_position);

                            if let Ok(ray) = ray {
                                // Project onto XZ plane
                                let t = -ray.origin.y / ray.direction.y;
                                let hit_point = ray.origin + ray.direction * t;

                                split_line.end = hit_point;

                                // Calculate normal (perpendicular to line in XZ plane)
                                let direction = (split_line.end - split_line.start).normalize();
                                split_line.normal = Vec3::new(-direction.z, 0.0, direction.x);
                            }
                        }
                    }
                }
            }

            // Update preview groups after updating split line
            if let Some(subgraph_id) = split_state.splitting_subgraph {
                if let Some(split_line) = &split_state.split_line {
                    split_state.preview_groups =
                        calculate_split_groups(subgraph_id, split_line, &nodes);

                    split_state.can_split = split_state.preview_groups.len() >= 2;
                }
            }
        } else if mouse_button.just_released(MouseButton::Left) {
            // Finish split gesture
            if split_state.can_split {
                // Will be handled by execute_split
            } else {
                // Clear state
                split_state.splitting_subgraph = None;
                split_state.split_line = None;
                split_state.preview_groups.clear();
                split_state.can_split = false;
            }
        }
    }
}

/// Preview split operation
pub fn preview_split(
    mut commands: Commands,
    split_state: Res<SplitState>,
    preview_query: Query<Entity, With<SplitPreview>>,
    mut gizmos: Gizmos,
) {
    // Remove old preview
    for entity in preview_query.iter() {
        commands.entity(entity).despawn();
    }

    // Draw split line
    if let Some(ref split_line) = split_state.split_line {
        gizmos.line(split_line.start, split_line.end, Color::srgb(1.0, 1.0, 0.0));

        // Draw normal indicator
        let mid = (split_line.start + split_line.end) * 0.5;
        gizmos.arrow(
            mid,
            mid + split_line.normal * 2.0,
            Color::srgb(1.0, 0.5, 0.0),
        );
    }

    // Show preview groups
    if split_state.can_split {
        for group in &split_state.preview_groups {
            // Draw group boundary
            gizmos.circle(
                Isometry3d::from_translation(group.center),
                2.0,
                group.color.with_alpha(0.5),
            );
        }

        if let Some(subgraph_id) = split_state.splitting_subgraph {
            commands.spawn(SplitPreview {
                subgraph_id,
                groups: split_state.preview_groups.clone(),
            });
        }
    }
}

/// Execute split operation
pub fn execute_split(
    mut events: EventWriter<CommandEvent>,
    mut split_state: ResMut<SplitState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    if mouse_button.just_released(MouseButton::Left) && split_state.can_split {
        if let Some(subgraph_id) = split_state.splitting_subgraph {
            // Use geometric line split
            let split_criteria = if let Some(line) = &split_state.split_line {
                crate::domain::value_objects::SplitCriteria::GeometricLine {
                    start: Position3D {
                        x: line.start.x,
                        y: line.start.y,
                        z: line.start.z,
                    },
                    end: Position3D {
                        x: line.end.x,
                        y: line.end.y,
                        z: line.end.z,
                    },
                }
            } else {
                // Default to connectivity-based split
                crate::domain::value_objects::SplitCriteria::Connectivity {
                    min_cut: true,
                    max_components: 2,
                }
            };

            events.send(CommandEvent {
                command: Command::SubgraphOperation(SubgraphOperationCommand::SplitSubgraph {
                    graph_id: GraphId::new(), // TODO: Get actual graph ID
                    subgraph_id,
                    criteria: split_criteria,
                }),
            });

            // Clear state
            split_state.splitting_subgraph = None;
            split_state.split_line = None;
            split_state.preview_groups.clear();
            split_state.can_split = false;
        }
    }
}

/// Calculate split groups based on split line
fn calculate_split_groups(
    subgraph_id: SubgraphId,
    split_line: &SplitLine,
    nodes: &Query<(&GraphNode, &SubgraphMember, &Transform)>,
) -> Vec<NodeGroup> {
    let mut group1 = NodeGroup {
        nodes: HashSet::new(),
        center: Vec3::ZERO,
        color: Color::srgb(0.0, 1.0, 0.0),
    };

    let mut group2 = NodeGroup {
        nodes: HashSet::new(),
        center: Vec3::ZERO,
        color: Color::srgb(0.0, 0.0, 1.0),
    };

    let mut count1 = 0;
    let mut count2 = 0;

    // Classify nodes based on which side of the line they're on
    for (node, member, transform) in nodes.iter() {
        if member.subgraph_ids.contains(&subgraph_id) {
            // Calculate which side of the line the node is on
            let to_node = transform.translation - split_line.start;
            let dot = to_node.dot(split_line.normal);

            if dot >= 0.0 {
                group1.nodes.insert(node.node_id);
                group1.center += transform.translation;
                count1 += 1;
            } else {
                group2.nodes.insert(node.node_id);
                group2.center += transform.translation;
                count2 += 1;
            }
        }
    }

    // Calculate centers
    if count1 > 0 {
        group1.center /= count1 as f32;
    }
    if count2 > 0 {
        group2.center /= count2 as f32;
    }

    // Return groups if both have nodes
    let mut groups = Vec::new();
    if !group1.nodes.is_empty() {
        groups.push(group1);
    }
    if !group2.nodes.is_empty() {
        groups.push(group2);
    }

    groups
}

/// Visualize selection state
pub fn visualize_selection(
    merge_state: Res<MergeState>,
    subgraph_origins: Query<(&SubgraphOrigin, &Transform)>,
    mut gizmos: Gizmos,
) {
    for (origin, transform) in subgraph_origins.iter() {
        if merge_state.selected_subgraphs.contains(&origin.subgraph_id) {
            // Draw selection indicator
            gizmos.circle(
                Isometry3d::from_translation(transform.translation + Vec3::Y * 0.1),
                1.0,
                Color::srgb(1.0, 1.0, 0.0),
            );
        }
    }
}

/// Plugin for merge/split functionality
pub struct SubgraphMergeSplitPlugin;

impl Plugin for SubgraphMergeSplitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MergeState>()
            .init_resource::<SplitState>()
            .add_systems(
                Update,
                (
                    handle_multi_selection,
                    preview_merge,
                    execute_merge,
                    detect_split_gesture,
                    preview_split,
                    execute_split,
                    visualize_selection,
                )
                    .chain(),
            );
    }
}
