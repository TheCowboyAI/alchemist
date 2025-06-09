//! Node interaction systems for conceptual graphs
//!
//! Handles user interactions with concept nodes including dragging,
//! connecting, and selection operations.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::presentation::components::{
    ConceptualNodeVisual, DraggableNode, ConnectableNode, SelectableGraph,
    ConceptualSpaceVisual, DragConstraints, SelectionMode, Highlighted,
    ConceptRelationship, ConceptualEdgeVisual, EdgeVisualStyle,
};
use crate::presentation::events::PresentationCommand;
use crate::domain::commands::graph_commands::GraphCommand;
use crate::domain::value_objects::{NodeId, EdgeId, Position3D, GraphId};

/// Event for node drag operations
#[derive(Event, Debug, Clone)]
pub struct NodeDragEvent {
    pub entity: Entity,
    pub node_id: NodeId,
    pub start_position: Vec3,
    pub current_position: Vec3,
    pub delta: Vec3,
}

/// Event for requesting node connections
#[derive(Event, Debug, Clone)]
pub struct NodeConnectionRequest {
    pub source_entity: Entity,
    pub target_entity: Entity,
    pub relationship: ConceptRelationship,
}

/// Event for selection changes
#[derive(Event, Debug, Clone)]
pub struct SelectionChangedEvent {
    pub graph_id: GraphId,
    pub selected_entities: Vec<Entity>,
    pub selection_mode: SelectionMode,
}

/// Handles node dragging with mouse
pub fn handle_node_dragging(
    _commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut draggable_nodes: Query<(
        Entity,
        &mut Transform,
        &mut DraggableNode,
        &ConceptualNodeVisual,
        &GlobalTransform,
    )>,
    space_query: Query<&ConceptualSpaceVisual>,
    selectable_graphs: Query<&SelectableGraph>,
    mut drag_events: EventWriter<NodeDragEvent>,
    mut graph_commands: EventWriter<PresentationCommand>,
) {
    let window = match windows.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let (camera, camera_transform) = match camera_query.get_single() {
        Ok(c) => c,
        Err(_) => return,
    };

    let cursor_position = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    // Get the first conceptual space for bounds checking
    let space = space_query.iter().next();

    // Get the active graph ID
    let graph_id = selectable_graphs
        .iter()
        .next()
        .map(|g| g.graph_id)
        .unwrap_or_else(GraphId::new);

    if mouse_button.just_pressed(MouseButton::Left) {
        // Start dragging - find node under cursor
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            // Simple distance-based selection (should use proper raycasting)
            let mut closest_entity = None;
            let mut closest_distance = f32::MAX;

            for (entity, _transform, _, _, global_transform) in draggable_nodes.iter() {
                let node_position = global_transform.translation();
                let distance_to_ray = (node_position - ray.origin).length(); // Simplified

                if distance_to_ray < closest_distance && distance_to_ray < 2.0 {
                    closest_distance = distance_to_ray;
                    closest_entity = Some(entity);
                }
            }

            // Start dragging the closest node
            if let Some(entity) = closest_entity {
                if let Ok((_, transform, mut draggable, _, _)) = draggable_nodes.get_mut(entity) {
                    draggable.is_dragging = true;
                    // Calculate drag offset
                    if let Ok(plane_intersection) = ray_plane_intersection(&ray, Vec3::Y, 0.0) {
                        draggable.drag_offset = transform.translation - plane_intersection;
                    }
                }
            }
        }
    } else if mouse_button.pressed(MouseButton::Left) {
        // Continue dragging
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            for (entity, mut transform, draggable, node_visual, _) in draggable_nodes.iter_mut() {
                if !draggable.is_dragging {
                    continue;
                }

                // Calculate new position on drag plane
                if let Ok(plane_intersection) = ray_plane_intersection(&ray, Vec3::Y, 0.0) {
                    let mut new_position = plane_intersection + draggable.drag_offset;

                    // Apply constraints
                    new_position = apply_drag_constraints(
                        new_position,
                        &draggable.constraints,
                        space,
                    );

                    // Snap to grid if enabled
                    if draggable.snap_to_grid {
                        new_position = snap_to_grid(new_position, draggable.grid_size);
                    }

                    let old_position = transform.translation;
                    transform.translation = new_position;

                    // Emit drag event
                    drag_events.write(NodeDragEvent {
                        entity,
                        node_id: node_visual.concept_id,
                        start_position: old_position,
                        current_position: new_position,
                        delta: new_position - old_position,
                    });

                    // Send command to update domain model
                    graph_commands.write(PresentationCommand::new(
                        GraphCommand::UpdateNode {
                            graph_id,
                            node_id: node_visual.concept_id,
                            new_position: Some(Position3D {
                                x: new_position.x,
                                y: new_position.y,
                                z: new_position.z,
                            }),
                            new_content: None,
                        }
                    ));
                }
            }
        }
    } else if mouse_button.just_released(MouseButton::Left) {
        // Stop dragging
        for (_, _, mut draggable, _, _) in draggable_nodes.iter_mut() {
            draggable.is_dragging = false;
            draggable.drag_offset = Vec3::ZERO;
        }
    }
}

/// Handles node connection creation
pub fn handle_node_connections(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut connection_requests: EventReader<NodeConnectionRequest>,
    connectable_nodes: Query<(&ConceptualNodeVisual, &ConnectableNode)>,
    selectable_graphs: Query<&SelectableGraph>,
    mut graph_commands: EventWriter<PresentationCommand>,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get the active graph ID
    let graph_id = selectable_graphs
        .iter()
        .next()
        .map(|g| g.graph_id)
        .unwrap_or_else(GraphId::new);

    // Handle connection mode (Ctrl+Click)
    if keyboard.pressed(KeyCode::ControlLeft) && mouse_button.just_pressed(MouseButton::Left) {
        // Connection mode logic would go here
        // For now, we'll handle explicit connection requests
    }

    // Process connection requests
    for request in connection_requests.read() {
        // Validate connection is allowed
        let source_valid = connectable_nodes
            .get(request.source_entity)
            .map(|(_, connectable)| {
                connectable.can_be_source
                    && connectable
                        .allowed_connections
                        .contains(&request.relationship)
            })
            .unwrap_or(false);

        let target_valid = connectable_nodes
            .get(request.target_entity)
            .map(|(_, connectable)| {
                connectable.can_be_target
                    && (connectable.max_connections.is_none()
                        || connectable.connection_count < connectable.max_connections.unwrap())
            })
            .unwrap_or(false);

        if source_valid && target_valid {
            // Get node IDs
            if let (Ok((source_visual, _)), Ok((target_visual, _))) = (
                connectable_nodes.get(request.source_entity),
                connectable_nodes.get(request.target_entity),
            ) {
                // Create edge in domain model
                let edge_id = EdgeId::new();
                graph_commands.write(PresentationCommand::new(
                    GraphCommand::ConnectNodes {
                        graph_id,
                        edge_id,
                        source_id: source_visual.concept_id,
                        target_id: target_visual.concept_id,
                        edge_type: "conceptual".to_string(), // Map from relationship
                        properties: std::collections::HashMap::new(),
                    }
                ));

                // Create visual edge
                let edge_visual = ConceptualEdgeVisual {
                    edge_id,
                    source_entity: request.source_entity,
                    target_entity: request.target_entity,
                    relationship: request.relationship.clone(),
                    visual_style: EdgeVisualStyle::default(),
                    animation_progress: 0.0,
                };

                // Spawn edge entity (visual representation would be handled by another system)
                commands.spawn((
                    edge_visual,
                    Name::new(format!("ConceptualEdge_{:?}", edge_id)),
                ));
            }
        }
    }
}

/// Handles node selection
pub fn handle_node_selection(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut selectable_graphs: Query<&mut SelectableGraph>,
    nodes: Query<(Entity, &ConceptualNodeVisual, &GlobalTransform)>,
    mut selection_events: EventWriter<SelectionChangedEvent>,
) {
    let window = match windows.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let (camera, camera_transform) = match camera_query.get_single() {
        Ok(c) => c,
        Err(_) => return,
    };

    let cursor_position = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    // Handle selection on click (when not dragging)
    if mouse_button.just_pressed(MouseButton::Left) && !keyboard.pressed(KeyCode::ControlLeft) {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            // Find node under cursor
            let mut closest_entity = None;
            let mut closest_distance = f32::MAX;

            for (entity, _, global_transform) in nodes.iter() {
                let node_position = global_transform.translation();
                let distance_to_ray = (node_position - ray.origin).length(); // Simplified

                if distance_to_ray < closest_distance && distance_to_ray < 2.0 {
                    closest_distance = distance_to_ray;
                    closest_entity = Some(entity);
                }
            }

            // Update selection
            for mut graph in selectable_graphs.iter_mut() {
                let multi_select = keyboard.pressed(KeyCode::ShiftLeft);

                if let Some(entity) = closest_entity {
                    if multi_select && graph.selection_mode == SelectionMode::Multiple {
                        // Toggle selection
                        if let Some(pos) = graph.selected_entities.iter().position(|&e| e == entity) {
                            graph.selected_entities.remove(pos);
                            commands.entity(entity).remove::<Highlighted>();
                        } else {
                            graph.selected_entities.push(entity);
                            commands.entity(entity).insert(Highlighted {
                                color: Color::srgb(1.0, 0.8, 0.0),
                                intensity: 0.3,
                            });
                        }
                    } else {
                        // Clear previous selection
                        for &selected in graph.selected_entities.iter() {
                            commands.entity(selected).remove::<Highlighted>();
                        }

                        // Select new entity
                        graph.selected_entities = vec![entity];
                        commands.entity(entity).insert(Highlighted {
                            color: Color::srgb(1.0, 0.8, 0.0),
                            intensity: 0.3,
                        });
                    }
                } else if !multi_select {
                    // Clear selection when clicking empty space
                    for &selected in graph.selected_entities.iter() {
                        commands.entity(selected).remove::<Highlighted>();
                    }
                    graph.selected_entities.clear();
                }

                // Emit selection event
                selection_events.write(SelectionChangedEvent {
                    graph_id: graph.graph_id,
                    selected_entities: graph.selected_entities.clone(),
                    selection_mode: graph.selection_mode,
                });
            }
        }
    }
}

/// Helper function for ray-plane intersection
fn ray_plane_intersection(ray: &Ray3d, plane_normal: Vec3, plane_distance: f32) -> Result<Vec3, ()> {
    let denominator = ray.direction.dot(plane_normal);

    if denominator.abs() < 0.0001 {
        return Err(()); // Ray is parallel to plane
    }

    let t = (plane_distance - ray.origin.dot(plane_normal)) / denominator;

    if t < 0.0 {
        return Err(()); // Intersection is behind ray origin
    }

    Ok(ray.origin + ray.direction * t)
}

/// Apply drag constraints to a position
fn apply_drag_constraints(
    position: Vec3,
    constraints: &DragConstraints,
    space: Option<&ConceptualSpaceVisual>,
) -> Vec3 {
    let mut constrained = position;

    // Apply axis constraints
    if !constraints.allowed_axes.0 {
        constrained.x = position.x;
    }
    if !constraints.allowed_axes.1 {
        constrained.y = position.y;
    }
    if !constraints.allowed_axes.2 {
        constrained.z = position.z;
    }

    // Apply min/max constraints
    if let Some(min) = constraints.min_position {
        constrained = constrained.max(min);
    }
    if let Some(max) = constraints.max_position {
        constrained = constrained.min(max);
    }

    // Apply space bounds constraint
    if constraints.constrain_to_space {
        if let Some(space) = space {
            constrained = constrained.clamp(space.bounds.min, space.bounds.max);
        }
    }

    constrained
}

/// Snap position to grid
fn snap_to_grid(position: Vec3, grid_size: f32) -> Vec3 {
    Vec3::new(
        (position.x / grid_size).round() * grid_size,
        (position.y / grid_size).round() * grid_size,
        (position.z / grid_size).round() * grid_size,
    )
}

/// Plugin for node interaction systems
pub struct NodeInteractionPlugin;

impl Plugin for NodeInteractionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<NodeDragEvent>()
            .add_event::<NodeConnectionRequest>()
            .add_event::<SelectionChangedEvent>()
            .add_systems(
                Update,
                (
                    handle_node_dragging,
                    handle_node_connections,
                    handle_node_selection,
                )
                    .chain(),
            );
    }
}
