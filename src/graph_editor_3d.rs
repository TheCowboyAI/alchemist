use crate::graph::AlchemistGraph;
use crate::graph_layout::{LayoutUpdateEvent, apply_initial_layout};
use crate::graph_patterns::{GraphPattern, PatternCatalog, generate_pattern};
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use std::collections::HashMap;
use uuid::Uuid;

// Component to mark an entity as a 3D graph node
#[derive(Component)]
pub struct GraphNode3D {
    pub id: Uuid,
}

// Component to mark an entity as a 3D graph edge
#[derive(Component)]
pub struct GraphEdge3D {
    pub id: Uuid,
    pub source: Uuid,
    pub target: Uuid,
}

// Component to store the original position of a node
#[derive(Component)]
pub struct OriginalPosition(pub Vec3);

// Resources
#[derive(Resource)]
pub struct GraphEditor3D {
    pub graph: AlchemistGraph,
    pub selected_node: Option<Uuid>,
    pub node_entities: HashMap<Uuid, Entity>,
    pub edge_entities: HashMap<Uuid, Entity>,
    pub pattern_catalog: PatternCatalog,
}

impl Default for GraphEditor3D {
    fn default() -> Self {
        Self {
            graph: AlchemistGraph::new(),
            selected_node: None,
            node_entities: HashMap::new(),
            edge_entities: HashMap::new(),
            pattern_catalog: PatternCatalog::new(),
        }
    }
}

// Event to request a graph update
#[derive(Event)]
pub struct UpdateGraph3DEvent;

// Event to create a pattern
#[derive(Event)]
pub struct CreatePatternEvent {
    pub pattern: GraphPattern,
    pub pattern_name: String,
}

#[derive(Resource, Default)]
pub struct UiInteractionState {
    pub mouse_over_ui: bool,
}

// Plugin to set up the 3D graph editor
pub struct GraphEditor3DPlugin;

impl Plugin for GraphEditor3DPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .init_resource::<GraphEditor3D>()
            .init_resource::<UiInteractionState>()
            .add_event::<UpdateGraph3DEvent>()
            .add_event::<CreatePatternEvent>()
            .add_systems(Startup, (setup_3d_editor, create_default_graph))
            .add_systems(
                Update,
                (
                    update_graph_3d,
                    handle_node_selection,
                    handle_create_pattern,
                    update_edge_positions,
                    block_camera_input_on_ui,
                ),
            );
    }
}

// System to set up the 3D graph editor
fn setup_3d_editor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Setup camera with PanOrbitCamera for 3D navigation
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: Some(15.0),
            pan_sensitivity: 1.0,
            ..default()
        },
    ));

    // Add a light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add a grid for reference
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.5, 0.5, 0.5, 0.2),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));
}

// System to update the 3D graph visualization
fn update_graph_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graph_editor: ResMut<GraphEditor3D>,
    mut update_events: EventReader<UpdateGraph3DEvent>,
    node_query: Query<Entity, With<GraphNode3D>>,
    edge_query: Query<Entity, With<GraphEdge3D>>,
) {
    if update_events.read().next().is_none() {
        return;
    }

    // Clear old entities
    for entity in node_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in edge_query.iter() {
        commands.entity(entity).despawn();
    }

    graph_editor.node_entities.clear();
    graph_editor.edge_entities.clear();

    // Collect node data first to avoid borrowing issues
    let node_data: Vec<(Uuid, String, Vec3, Color)> = graph_editor
        .graph
        .nodes
        .iter()
        .map(|(id, node)| {
            // Determine position
            let position = if let Some(pos) = graph_editor.graph.node_positions.get(id) {
                Vec3::new(pos.x, 1.0, pos.y) // Elevate to y=1.0 instead of y=0.0
            } else {
                // Use a deterministic initial position based on node id hash
                // This will create a more consistent layout without randomness
                let id_hash = id.as_u128() as u32;
                let x_factor = ((id_hash & 0xFF0000) >> 16) as f32 / 128.0 - 1.0;
                let z_factor = ((id_hash & 0x00FF00) >> 8) as f32 / 128.0 - 1.0;
                let y_factor = ((id_hash & 0x0000FF) as f32 / 255.0) * 2.0; // Add some vertical variation
                Vec3::new(x_factor * 5.0, 1.0 + y_factor, z_factor * 5.0) // Elevate base height to y=1.0
            };

            // Generate a more diverse color palette based on node type/labels
            let color = if node.labels.contains(&"start".to_string()) {
                Color::srgb(0.1, 0.7, 0.3) // Green
            } else if node.labels.contains(&"end".to_string()) {
                Color::srgb(0.8, 0.2, 0.2) // Red
            } else if node.labels.contains(&"decision".to_string()) {
                Color::srgb(0.95, 0.75, 0.1) // Gold
            } else if node.labels.contains(&"process".to_string()) {
                Color::srgb(0.2, 0.4, 0.8) // Royal Blue
            } else if node.labels.contains(&"input".to_string()) {
                Color::srgb(0.4, 0.7, 0.9) // Light Blue
            } else if node.labels.contains(&"output".to_string()) {
                Color::srgb(0.9, 0.5, 0.1) // Orange
            } else if node.labels.contains(&"storage".to_string()) {
                Color::srgb(0.5, 0.3, 0.8) // Purple
            } else if node.labels.contains(&"compute".to_string()) {
                Color::srgb(0.3, 0.7, 0.5) // Teal
            } else if node.labels.contains(&"conditional".to_string()) {
                Color::srgb(0.9, 0.3, 0.5) // Pink
            } else if node.labels.contains(&"loop".to_string()) {
                Color::srgb(0.5, 0.8, 0.2) // Lime Green
            } else {
                // Generate a color based on the node ID hash for variety
                let id_hash = id.as_u128() as u32;
                let r = ((id_hash & 0xFF0000) >> 16) as f32 / 255.0;
                let g = ((id_hash & 0x00FF00) >> 8) as f32 / 255.0;
                let b = (id_hash & 0x0000FF) as f32 / 255.0;
                // Ensure reasonable brightness
                let min_component = 0.3;
                Color::srgb(
                    r.max(min_component),
                    g.max(min_component),
                    b.max(min_component),
                )
            };

            (*id, node.name.clone(), position, color)
        })
        .collect();

    // Create node entities
    for (id, name, position, color) in &node_data {
        // Create the node entity with mesh and material
        let node_entity = commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.3).mesh().uv(16, 16))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: *color,
                ..default()
            })),
            Transform::from_translation(*position),
            GraphNode3D { id: *id },
            OriginalPosition(*position),
            Name::new(name.clone()),
        ));

        // Store the entity
        graph_editor.node_entities.insert(*id, node_entity.id());

        // Add text for the node name as a separate entity with better styling
        commands.spawn((
            // Simpler Text component to avoid dependency issues
            Text::new(name.clone()),
            // Position the text above the node
            Transform::from_translation(*position + Vec3::new(0.0, 0.5, 0.0)),
        ));
    }

    // Collect edge data to avoid borrowing issues
    let edge_data: Vec<(Uuid, Uuid, Uuid, f32)> = graph_editor
        .graph
        .edges
        .iter()
        .map(|(id, edge)| (*id, edge.source, edge.target, edge.weight)) // Use the actual edge weight
        .collect();

    // Create edge entities with proper meshes and materials
    for (id, source, target, weight) in edge_data {
        // Calculate initial edge geometry if both nodes have positions
        let source_pos = node_data
            .iter()
            .find(|(node_id, _, _, _)| *node_id == source)
            .map(|(_, _, pos, _)| *pos);
        let target_pos = node_data
            .iter()
            .find(|(node_id, _, _, _)| *node_id == target)
            .map(|(_, _, pos, _)| *pos);

        let edge_entity = if let (Some(source_pos), Some(target_pos)) = (source_pos, target_pos) {
            // Calculate edge geometry
            let direction = target_pos - source_pos;
            let distance = direction.length();

            if distance > 0.01 {
                let normalized_dir = direction / distance;
                let mid_point = source_pos + direction * 0.5;

                // Get rotation to align cylinder with direction
                let default_dir = Vec3::Y;
                let rotation = Quat::from_rotation_arc(default_dir, normalized_dir);

                // Create edge with proper mesh and transform
                commands.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.05, distance - 0.6).mesh())),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.7, 0.7, 0.7),
                        ..default()
                    })),
                    Transform::from_translation(mid_point).with_rotation(rotation),
                    GraphEdge3D { id, source, target },
                    Name::new(format!("Edge {} (weight: {})", id, weight)),
                ))
            } else {
                // Fallback for very close nodes
                commands.spawn((
                    GraphEdge3D { id, source, target },
                    Transform::default(),
                    Name::new(format!("Edge {} (weight: {})", id, weight)),
                ))
            }
        } else {
            // No positions available yet
            commands.spawn((
                GraphEdge3D { id, source, target },
                Transform::default(),
                Name::new(format!("Edge {} (weight: {})", id, weight)),
            ))
        };

        graph_editor.edge_entities.insert(id, edge_entity.id());
    }
}

// System to handle selection of nodes
fn handle_node_selection(
    mut graph_editor: ResMut<GraphEditor3D>,
    buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    node_query: Query<(Entity, &GraphNode3D, &GlobalTransform)>,
    ui_state: Res<UiInteractionState>,
) {
    // Don't process selection if mouse is over UI
    if ui_state.mouse_over_ui {
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    // Get the cursor position from the primary window
    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Get the camera for ray casting
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Cast a ray from the cursor position
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Check for intersections with node entities
    let mut closest_node = None;
    let mut closest_distance = f32::MAX;

    for (_, node, transform) in node_query.iter() {
        let node_position = transform.translation();
        // Simple sphere intersection check
        let to_node = node_position - ray.origin;
        let closest_point_on_ray = ray.origin + ray.direction * to_node.dot(*ray.direction);
        let distance_squared = (node_position - closest_point_on_ray).length_squared();

        if distance_squared < 0.3 * 0.3 && to_node.dot(*ray.direction) > 0.0 {
            let distance = to_node.length();
            if distance < closest_distance {
                closest_distance = distance;
                closest_node = Some(node.id);
            }
        }
    }

    // Update selected node
    graph_editor.selected_node = closest_node;
}

// System to handle pattern creation
fn handle_create_pattern(
    mut graph_editor: ResMut<GraphEditor3D>,
    mut create_events: EventReader<CreatePatternEvent>,
    mut update_events: EventWriter<UpdateGraph3DEvent>,
    mut layout_events: EventWriter<LayoutUpdateEvent>,
) {
    for event in create_events.read() {
        // Generate pattern and update the graph
        graph_editor.graph = generate_pattern(event.pattern.clone());

        // Apply initial layout based on pattern type
        apply_initial_layout(&mut graph_editor.graph, &event.pattern_name);

        // Send event to update the 3D visualization
        update_events.write(UpdateGraph3DEvent);

        // Trigger force-directed layout to refine the positions
        layout_events.write(LayoutUpdateEvent);
    }
}

// System to update edge positions (only when graph updates)
fn update_edge_positions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    edge_query: Query<(Entity, &GraphEdge3D, Option<&Mesh3d>)>,
    node_query: Query<(&GraphNode3D, &Transform)>,
    mut update_events: EventReader<UpdateGraph3DEvent>,
) {
    // Only update when graph update events occur
    if update_events.read().next().is_none() {
        return;
    }

    // Create map of node IDs to their positions
    let mut node_positions = HashMap::new();
    for (node, transform) in node_query.iter() {
        node_positions.insert(node.id, transform.translation);
    }

    // Update edge transforms and add meshes if missing
    for (entity, edge, mesh_opt) in edge_query.iter() {
        if let (Some(source_pos), Some(target_pos)) = (
            node_positions.get(&edge.source),
            node_positions.get(&edge.target),
        ) {
            // Calculate edge geometry
            let direction = *target_pos - *source_pos;
            let distance = direction.length();

            if distance < 0.01 {
                continue; // Skip if nodes are too close
            }

            let normalized_dir = direction / distance;
            let mid_point = *source_pos + direction * 0.5;

            // Get rotation to align cylinder with direction
            let default_dir = Vec3::Y;
            let rotation = Quat::from_rotation_arc(default_dir, normalized_dir);

            // Update transform
            commands
                .entity(entity)
                .insert(Transform::from_translation(mid_point).with_rotation(rotation));

            // Add mesh and material if missing
            if mesh_opt.is_none() {
                commands.entity(entity).insert((
                    Mesh3d(meshes.add(Cylinder::new(0.05, distance - 0.6).mesh())),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.7, 0.7, 0.7),
                        ..default()
                    })),
                ));
            }
        }
    }
}

// System to handle camera input based on UI state
fn block_camera_input_on_ui(
    mut camera_query: Query<&mut PanOrbitCamera>,
    ui_state: Res<UiInteractionState>,
) {
    // When mouse is over UI, disable camera controls
    // by setting pan and orbit sensitivities to zero
    for mut camera in camera_query.iter_mut() {
        if ui_state.mouse_over_ui {
            // Store original values if needed (could add more fields to UiInteractionState to restore later)
            camera.pan_sensitivity = 0.0;
            camera.orbit_sensitivity = 0.0;
            camera.zoom_sensitivity = 0.0;
        } else {
            // Restore normal camera controls
            camera.pan_sensitivity = 1.0;
            camera.orbit_sensitivity = 1.0;
            camera.zoom_sensitivity = 1.0;
        }
    }
}

// System to create a default graph for initial visualization
fn create_default_graph(
    mut graph_editor: ResMut<GraphEditor3D>,
    mut update_events: EventWriter<UpdateGraph3DEvent>,
) {
    // Only create if the graph is empty
    if graph_editor.graph.nodes.is_empty() {
        // Create a simple example graph
        let pattern = GraphPattern::Star { points: 6 };
        graph_editor.graph = generate_pattern(pattern);

        // Apply initial layout
        apply_initial_layout(&mut graph_editor.graph, "small_star");

        // Trigger 3D update
        update_events.write(UpdateGraph3DEvent);
    }
}
