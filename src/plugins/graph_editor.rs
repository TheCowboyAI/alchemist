//! Graph Editor Plugin - Complete graph editing functionality

use crate::{
    components::{EdgeEntity, NodeEntity, Selected},
    events::{EdgeAdded, EdgeRemoved, NodeAdded, NodeRemoved},
    value_objects::{EdgeId, EdgeRelationship, GraphId, NodeId, NodeType, Position3D},
};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use tracing::{info, warn};
use uuid::Uuid;

/// Plugin for graph editing functionality
pub struct GraphEditorPlugin;

impl Plugin for GraphEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<GraphEditorState>()
            .init_resource::<GraphStats>()
            .init_resource::<FrameCounter>()
            // Command Events
            .add_event::<CreateNodeCommand>()
            .add_event::<CreateEdgeCommand>()
            .add_event::<DeleteSelectedCommand>()
            // Domain Events
            .add_event::<NodeAdded>()
            .add_event::<EdgeAdded>()
            .add_event::<NodeRemoved>()
            .add_event::<EdgeRemoved>()
            // Systems
            .add_systems(Startup, setup_graph_editor)
            .add_systems(
                Update,
                (
                    // Input handling
                    handle_mouse_input,
                    handle_keyboard_input,
                    // Graph operations
                    create_nodes_system,
                    create_edges_system,
                    delete_selected_system,
                    // Selection
                    update_selection_system,
                    // Statistics must run before UI
                    update_graph_stats,
                    // Visualization
                    visualize_nodes,
                    visualize_edges,
                    highlight_selected,
                    // Animation and effects
                    update_hover_effects,
                    animate_scale_transitions,
                    update_material_effects,
                ),
            )
            // UI runs separately to ensure proper ordering
            .add_systems(Update, render_graph_ui);
    }
}

/// State for the graph editor
#[derive(Resource)]
pub struct GraphEditorState {
    /// Current editing mode
    pub mode: EditorMode,
    /// Entity being dragged from (for edge creation)
    pub dragging_from: Option<Entity>,
    /// Entity currently under the cursor
    pub hover_entity: Option<Entity>,
    /// List of currently selected entities
    pub selected_entities: Vec<Entity>,
    /// Whether to snap to grid
    pub grid_snap: bool,
    /// Size of the grid for snapping
    pub grid_size: f32,
}

impl Default for GraphEditorState {
    fn default() -> Self {
        Self {
            mode: EditorMode::default(),
            dragging_from: None,
            hover_entity: None,
            selected_entities: Vec::new(),
            grid_snap: false,
            grid_size: 1.0,
        }
    }
}

/// Editor modes
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum EditorMode {
    /// Select entities mode
    #[default]
    Select,
    /// Create node mode
    CreateNode,
    /// Create edge mode
    CreateEdge,
    /// Delete entities mode
    Delete,
}

/// Graph statistics
#[derive(Resource, Default)]
pub struct GraphStats {
    /// Number of nodes in the graph
    pub node_count: usize,
    /// Number of edges in the graph
    pub edge_count: usize,
    /// Number of selected entities
    pub selected_count: usize,
}

/// Frame counter to skip first frame for egui
#[derive(Resource, Default)]
pub struct FrameCounter {
    /// Number of frames elapsed
    pub count: u32,
}

/// Command to create a node
#[derive(Event)]
pub struct CreateNodeCommand {
    /// Position where the node should be created
    pub position: Vec3,
    /// Type of node to create
    pub node_type: NodeType,
}

/// Command to create an edge
#[derive(Event)]
pub struct CreateEdgeCommand {
    /// Source node entity
    pub source: Entity,
    /// Target node entity
    pub target: Entity,
}

/// Command to delete selected entities
#[derive(Event)]
pub struct DeleteSelectedCommand;

/// Animation state for smooth transitions
#[derive(Component)]
pub struct AnimationState {
    /// Original scale
    pub base_scale: Vec3,
    /// Target scale
    pub target_scale: Vec3,
    /// Animation progress (0.0 to 1.0)
    pub progress: f32,
    /// Animation speed
    pub speed: f32,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            base_scale: Vec3::ONE,
            target_scale: Vec3::ONE,
            progress: 1.0,
            speed: 5.0,
        }
    }
}

/// Hover effect component
#[derive(Component)]
pub struct HoverEffect {
    /// Whether the entity is currently hovered
    pub is_hovered: bool,
    /// Glow intensity
    pub glow_intensity: f32,
    /// Target glow intensity
    pub target_glow: f32,
}

impl Default for HoverEffect {
    fn default() -> Self {
        Self {
            is_hovered: false,
            glow_intensity: 0.0,
            target_glow: 0.0,
        }
    }
}

/// Handle mouse input for graph editing
fn handle_mouse_input(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut editor_state: ResMut<GraphEditorState>,
    mut create_node_events: EventWriter<CreateNodeCommand>,
    mut create_edge_events: EventWriter<CreateEdgeCommand>,
    nodes: Query<(Entity, &Transform), With<NodeEntity>>,
) {
    let Ok(window) = windows.single() else {
        warn!("No window found");
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        warn!("No camera found");
        return;
    };

    if let Some(cursor_position) = window.cursor_position() {
        // Debug log cursor position
        if mouse_button.just_pressed(MouseButton::Left) {
            info!("Mouse clicked at screen position: {:?}", cursor_position);
        }

        // Convert screen position to world position
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            // Project onto XZ plane (Y=0)
            let distance = if ray.direction.y.abs() > 0.001 {
                -ray.origin.y / ray.direction.y
            } else {
                10.0 // Default distance if ray is parallel to plane
            };
            let world_position = ray.origin + ray.direction * distance;

            if mouse_button.just_pressed(MouseButton::Left) {
                info!("World position: {:?}", world_position);
            }

            // Find entity under cursor
            let mut closest_entity = None;
            let mut closest_distance = f32::MAX;

            for (entity, transform) in nodes.iter() {
                let distance = transform.translation.distance(world_position);
                if distance < 0.5 && distance < closest_distance {
                    closest_entity = Some(entity);
                    closest_distance = distance;
                }
            }

            editor_state.hover_entity = closest_entity;

            // Handle mouse clicks based on mode
            if mouse_button.just_pressed(MouseButton::Left) {
                info!("Current mode: {:?}", editor_state.mode);
                match editor_state.mode {
                    EditorMode::Select => {
                        if let Some(entity) = closest_entity {
                            // Toggle selection
                            if editor_state.selected_entities.contains(&entity) {
                                editor_state.selected_entities.retain(|&e| e != entity);
                                commands.entity(entity).remove::<Selected>();
                            } else {
                                editor_state.selected_entities.push(entity);
                                commands.entity(entity).insert(Selected);
                            }
                        } else {
                            // Clear selection
                            for entity in editor_state.selected_entities.drain(..) {
                                commands.entity(entity).remove::<Selected>();
                            }
                        }
                    }
                    EditorMode::CreateNode => {
                        let position = if editor_state.grid_snap {
                            snap_to_grid(world_position, editor_state.grid_size)
                        } else {
                            world_position
                        };

                        create_node_events.write(CreateNodeCommand {
                            position,
                            node_type: NodeType::Process,
                        });
                    }
                    EditorMode::CreateEdge => {
                        if let Some(entity) = closest_entity {
                            if let Some(source) = editor_state.dragging_from {
                                if source != entity {
                                    create_edge_events.write(CreateEdgeCommand {
                                        source,
                                        target: entity,
                                    });
                                    editor_state.dragging_from = None;
                                }
                            } else {
                                editor_state.dragging_from = Some(entity);
                            }
                        }
                    }
                    EditorMode::Delete => {
                        if let Some(entity) = closest_entity {
                            editor_state.selected_entities.push(entity);
                            commands.entity(entity).insert(Selected);
                        }
                    }
                }
            }

            // Cancel edge creation on right click
            if mouse_button.just_pressed(MouseButton::Right) {
                editor_state.dragging_from = None;
            }
        }
    }
}

/// Handle keyboard shortcuts
fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor_state: ResMut<GraphEditorState>,
    mut delete_events: EventWriter<DeleteSelectedCommand>,
) {
    // Log any key press for debugging
    for key in keyboard.get_just_pressed() {
        info!("Key pressed: {:?}", key);
    }

    // Mode switching
    if keyboard.just_pressed(KeyCode::KeyS) {
        info!("Switching to Select mode");
        editor_state.mode = EditorMode::Select;
        editor_state.dragging_from = None;
    }
    if keyboard.just_pressed(KeyCode::KeyN) {
        info!("Switching to CreateNode mode");
        editor_state.mode = EditorMode::CreateNode;
        editor_state.dragging_from = None;
    }
    if keyboard.just_pressed(KeyCode::KeyE) {
        editor_state.mode = EditorMode::CreateEdge;
    }
    if keyboard.just_pressed(KeyCode::KeyD) {
        editor_state.mode = EditorMode::Delete;
    }

    // Grid snap toggle
    if keyboard.just_pressed(KeyCode::KeyG) {
        editor_state.grid_snap = !editor_state.grid_snap;
    }

    // Delete selected
    if keyboard.just_pressed(KeyCode::Delete) {
        delete_events.write(DeleteSelectedCommand);
    }

    // Select all
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyA) {
        // This would need to be implemented
    }
}

/// Create nodes from commands
fn create_nodes_system(
    mut commands: Commands,
    mut create_events: EventReader<CreateNodeCommand>,
    mut node_events: EventWriter<NodeAdded>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in create_events.read() {
        let node_id = NodeId(Uuid::new_v4());
        let graph_id = GraphId(Uuid::new_v4()); // Should come from current graph

        // Spawn visual representation
        let entity = commands
            .spawn((
                NodeEntity { node_id, graph_id },
                Mesh3d(meshes.add(Sphere::new(0.3).mesh())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.5, 0.8),
                    emissive: LinearRgba::BLACK,
                    ..default()
                })),
                Transform::from_translation(event.position).with_scale(Vec3::ZERO),
                AnimationState {
                    base_scale: Vec3::ONE,
                    target_scale: Vec3::ONE,
                    progress: 0.0,
                    speed: 5.0,
                },
                HoverEffect::default(),
            ))
            .id();

        // Store entity reference for future operations
        info!("Created node entity {:?} for node {:?}", entity, node_id);

        // Emit domain event
        node_events.write(NodeAdded {
            node_id,
            graph_id,
            node_type: event.node_type.clone(),
            position: Position3D {
                x: event.position.x,
                y: event.position.y,
                z: event.position.z,
            },
            metadata: Default::default(),
        });

        info!("Created node {:?} at {:?}", node_id, event.position);
    }
}

/// Create edges from commands
fn create_edges_system(
    mut commands: Commands,
    mut create_events: EventReader<CreateEdgeCommand>,
    mut edge_events: EventWriter<EdgeAdded>,
    nodes: Query<(&NodeEntity, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in create_events.read() {
        if let (Ok((source_node, source_transform)), Ok((target_node, target_transform))) =
            (nodes.get(event.source), nodes.get(event.target))
        {
            let edge_id = EdgeId(Uuid::new_v4());

            // Calculate edge position and rotation
            let start = source_transform.translation;
            let end = target_transform.translation;
            let direction = end - start;
            let distance = direction.length();
            let midpoint = start + direction * 0.5;

            // Create cylinder mesh for edge
            let mesh = meshes.add(Cylinder::new(0.05, distance).mesh());

            // Calculate rotation to align cylinder
            let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());

            // Spawn edge entity
            commands.spawn((
                EdgeEntity {
                    edge_id,
                    source: source_node.node_id,
                    target: target_node.node_id,
                },
                Mesh3d(mesh),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.6, 0.6, 0.6),
                    ..default()
                })),
                Transform::from_translation(midpoint).with_rotation(rotation),
            ));

            // Emit domain event
            edge_events.write(EdgeAdded {
                edge_id,
                graph_id: source_node.graph_id,
                source: source_node.node_id,
                target: target_node.node_id,
                relationship: EdgeRelationship::Connects,
            });

            info!(
                "Created edge from {:?} to {:?}",
                source_node.node_id, target_node.node_id
            );
        }
    }
}

/// Delete selected entities
fn delete_selected_system(
    mut commands: Commands,
    mut delete_events: EventReader<DeleteSelectedCommand>,
    mut editor_state: ResMut<GraphEditorState>,
    mut node_events: EventWriter<NodeRemoved>,
    mut edge_events: EventWriter<EdgeRemoved>,
    nodes: Query<&NodeEntity>,
    edges: Query<&EdgeEntity>,
) {
    for _ in delete_events.read() {
        for &entity in editor_state.selected_entities.iter() {
            // Check if it's a node
            if let Ok(node) = nodes.get(entity) {
                node_events.write(NodeRemoved {
                    node_id: node.node_id,
                    graph_id: node.graph_id,
                });
                commands.entity(entity).despawn();
                info!("Deleted node {:?}", node.node_id);
            }

            // Check if it's an edge
            if let Ok(edge) = edges.get(entity) {
                edge_events.write(EdgeRemoved {
                    edge_id: edge.edge_id,
                    graph_id: GraphId(Uuid::new_v4()), // Should track this
                });
                commands.entity(entity).despawn();
                info!("Deleted edge {:?}", edge.edge_id);
            }
        }

        editor_state.selected_entities.clear();
    }
}

/// Update selection visualization
fn update_selection_system(
    selected_query: Query<Entity, Added<Selected>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_handles: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    // Update material colors for newly selected entities
    for entity in selected_query.iter() {
        if let Ok(material_handle) = material_handles.get(entity) {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                // Change to selection color
                material.base_color = Color::srgb(1.0, 0.8, 0.2);
                material.emissive = LinearRgba::rgb(0.2, 0.1, 0.0);
            }
        }
    }
}

/// Render graph editor UI
fn render_graph_ui(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<GraphEditorState>,
    graph_stats: Res<GraphStats>,
    mut frame_counter: ResMut<FrameCounter>,
) {
    // Skip first few frames to ensure egui is initialized
    frame_counter.count += 1;
    if frame_counter.count < 3 {
        return;
    }

    let ctx = contexts.ctx_mut();

    egui::Window::new("Graph Editor")
        .default_pos(egui::Pos2::new(10.0, 10.0))
        .show(ctx, |ui| {
            ui.heading("Graph Editor Controls");

            ui.separator();

            // Mode selection
            ui.label("Mode:");
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(editor_state.mode == EditorMode::Select, "Select (S)")
                    .clicked()
                {
                    info!("UI: Switching to Select mode");
                    editor_state.mode = EditorMode::Select;
                    editor_state.dragging_from = None;
                }
                if ui
                    .selectable_label(editor_state.mode == EditorMode::CreateNode, "Node (N)")
                    .clicked()
                {
                    info!("UI: Switching to CreateNode mode");
                    editor_state.mode = EditorMode::CreateNode;
                    editor_state.dragging_from = None;
                }
                if ui
                    .selectable_label(editor_state.mode == EditorMode::CreateEdge, "Edge (E)")
                    .clicked()
                {
                    info!("UI: Switching to CreateEdge mode");
                    editor_state.mode = EditorMode::CreateEdge;
                }
                if ui
                    .selectable_label(editor_state.mode == EditorMode::Delete, "Delete (D)")
                    .clicked()
                {
                    info!("UI: Switching to Delete mode");
                    editor_state.mode = EditorMode::Delete;
                }
            });

            ui.separator();

            // Grid settings
            ui.checkbox(&mut editor_state.grid_snap, "Grid Snap (G)");
            if editor_state.grid_snap {
                ui.add(egui::Slider::new(&mut editor_state.grid_size, 0.1..=2.0).text("Grid Size"));
            }

            ui.separator();

            // Statistics
            ui.label("Graph Statistics:");
            ui.label(format!("Nodes: {}", graph_stats.node_count));
            ui.label(format!("Edges: {}", graph_stats.edge_count));
            ui.label(format!("Selected: {}", graph_stats.selected_count));

            ui.separator();

            // Help
            ui.collapsing("Help", |ui| {
                ui.label("S - Select mode");
                ui.label("N - Create node mode");
                ui.label("E - Create edge mode");
                ui.label("D - Delete mode");
                ui.label("G - Toggle grid snap");
                ui.label("Delete - Delete selected");
                ui.label("Left click - Select/Create");
                ui.label("Right click - Cancel");
            });
        });
}

/// Update graph statistics
fn update_graph_stats(
    mut stats: ResMut<GraphStats>,
    nodes: Query<Entity, With<NodeEntity>>,
    edges: Query<Entity, With<EdgeEntity>>,
    selected: Query<Entity, With<Selected>>,
) {
    stats.node_count = nodes.iter().count();
    stats.edge_count = edges.iter().count();
    stats.selected_count = selected.iter().count();
}

/// Visualize nodes
fn visualize_nodes(
    nodes: Query<(&Transform, &MeshMaterial3d<StandardMaterial>), With<NodeEntity>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Update node visualization based on state
    for (transform, material_handle) in nodes.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            // Adjust material properties based on node position
            let height_factor = (transform.translation.y + 5.0) / 10.0;
            let brightness = height_factor.clamp(0.5, 1.0);

            // Only update if not selected (selected nodes have their own color)
            if material.base_color != Color::srgb(1.0, 0.8, 0.2) {
                material.base_color =
                    Color::srgb(0.2 * brightness, 0.5 * brightness, 0.8 * brightness);
            }
        }
    }
}

/// Visualize edges
fn visualize_edges(
    mut edges: Query<(&EdgeEntity, &mut Transform)>,
    nodes: Query<(&NodeEntity, &Transform), Without<EdgeEntity>>,
) {
    // Update edge positions when nodes move
    for (edge, mut edge_transform) in edges.iter_mut() {
        // Find source and target nodes
        let mut source_pos = None;
        let mut target_pos = None;

        for (node, node_transform) in nodes.iter() {
            if node.node_id == edge.source {
                source_pos = Some(node_transform.translation);
            }
            if node.node_id == edge.target {
                target_pos = Some(node_transform.translation);
            }
        }

        // Update edge position and rotation if both nodes found
        if let (Some(start), Some(end)) = (source_pos, target_pos) {
            let direction = end - start;
            let distance = direction.length();
            let midpoint = start + direction * 0.5;

            // Update position
            edge_transform.translation = midpoint;

            // Update rotation to align with new direction
            if distance > 0.001 {
                edge_transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
            }

            // Update scale to match new distance
            edge_transform.scale.y = distance;
        }
    }
}

/// Highlight selected entities
fn highlight_selected(
    selected: Query<&MeshMaterial3d<StandardMaterial>, Added<Selected>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for material_handle in selected.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color = Color::srgb(1.0, 0.8, 0.2);
        }
    }
}

/// Snap position to grid
fn snap_to_grid(position: Vec3, grid_size: f32) -> Vec3 {
    Vec3::new(
        (position.x / grid_size).round() * grid_size,
        position.y,
        (position.z / grid_size).round() * grid_size,
    )
}

/// Setup the graph editor environment
fn setup_graph_editor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add a grid plane for reference
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.0)).mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.01, 0.0),
    ));
}

/// Update hover effects based on editor state
fn update_hover_effects(
    editor_state: Res<GraphEditorState>,
    mut nodes: Query<(Entity, &mut HoverEffect)>,
) {
    for (entity, mut hover) in nodes.iter_mut() {
        hover.is_hovered = Some(entity) == editor_state.hover_entity;
        hover.target_glow = if hover.is_hovered { 0.3 } else { 0.0 };
    }
}

/// Animate scale transitions for smooth appearance
fn animate_scale_transitions(
    mut nodes: Query<(&mut Transform, &mut AnimationState)>,
    time: Res<Time>,
) {
    for (mut transform, mut anim) in nodes.iter_mut() {
        if anim.progress < 1.0 {
            anim.progress += time.delta_secs() * anim.speed;
            anim.progress = anim.progress.clamp(0.0, 1.0);

            // Smooth easing function
            let t = ease_out_cubic(anim.progress);
            transform.scale = anim.base_scale.lerp(anim.target_scale, t);
        }
    }
}

/// Update material effects based on hover and selection state
fn update_material_effects(
    mut materials: ResMut<Assets<StandardMaterial>>,
    nodes: Query<(&MeshMaterial3d<StandardMaterial>, &HoverEffect, Option<&Selected>)>,
    time: Res<Time>,
) {
    for (material_handle, hover, selected) in nodes.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            // Smooth hover glow transition
            let current_glow = material.emissive.to_f32_array()[0];
            let target_glow = if selected.is_some() {
                0.4 // Selected glow
            } else {
                hover.target_glow
            };

            let new_glow = current_glow + (target_glow - current_glow) * time.delta_secs() * 5.0;
            material.emissive = LinearRgba::rgb(new_glow * 0.8, new_glow * 0.6, new_glow * 0.2);

            // Pulse effect for selected nodes
            if selected.is_some() {
                let pulse = (time.elapsed_secs() * 2.0).sin() * 0.1 + 0.9;
                material.emissive = material.emissive * pulse;
            }
        }
    }
}

/// Easing function for smooth animations
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}
