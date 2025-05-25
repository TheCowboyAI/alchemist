use bevy::prelude::*;
use bevy::sprite::ColorMaterial;
// use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use std::collections::HashMap;
use uuid::Uuid;

use crate::graph::AlchemistGraph;
use crate::graph_patterns::{GraphPattern, generate_pattern};
use crate::theming::AlchemistTheme;

/// Plugin for the unified graph editor system
pub struct UnifiedGraphEditorPlugin;

impl Plugin for UnifiedGraphEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(PanOrbitCameraPlugin) // Keep disabled for now, focus on 2D
            .init_resource::<BaseGraphResource>()
            .init_resource::<EditorState>()
            .init_resource::<EditorMode>()
            .add_event::<AddPatternToBaseGraphEvent>()
            .add_event::<SwitchEditorModeEvent>()
            .add_event::<ResetBaseGraphEvent>()
            .add_event::<AddNodeToBaseGraphEvent>()
            .add_systems(Startup, setup_editor_environment)
            .add_systems(
                Update,
                (
                    handle_add_pattern_events,
                    handle_add_node_events,
                    handle_mode_switch_events,
                    handle_reset_base_graph_events,
                    update_visual_representation,
                    handle_camera_controls, // Re-enabled camera controls
                ),
            );
    }
}

/// The main base graph that serves as our viewmodel
#[derive(Resource)]
pub struct BaseGraphResource {
    pub graph: AlchemistGraph,
    pub subgraphs: HashMap<Uuid, SubgraphInfo>,
    pub node_positions: HashMap<Uuid, Vec3>,
    pub next_subgraph_id: u32,
    pub visual_dirty: bool,
}

impl Default for BaseGraphResource {
    fn default() -> Self {
        Self {
            graph: AlchemistGraph::new(),
            subgraphs: HashMap::new(),
            node_positions: HashMap::new(),
            next_subgraph_id: 1,
            visual_dirty: true,
        }
    }
}

/// Information about subgraphs within the base graph
#[derive(Debug, Clone)]
pub struct SubgraphInfo {
    pub id: Uuid,
    pub name: String,
    pub pattern_type: String,
    pub nodes: Vec<Uuid>,
    pub color: Color,
}

/// Resource to track the current editor state
#[derive(Resource, Default)]
pub struct EditorState {
    pub selected_subgraph: Option<Uuid>,
    pub selected_node: Option<Uuid>,
    pub camera_2d: Option<Entity>,
    pub camera_3d: Option<Entity>,
}

/// Resource to track current editor mode
#[derive(Resource, Default)]
pub struct EditorMode {
    pub mode: ViewMode,
    pub show_subgraph_bounds: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ViewMode {
    #[default]
    Mode3D,
    Mode2D,
}

/// Events for graph operations
#[derive(Event)]
pub struct AddPatternToBaseGraphEvent {
    pub pattern: GraphPattern,
    pub name: String,
}

#[derive(Event)]
pub struct SwitchEditorModeEvent {
    pub mode: ViewMode,
}

#[derive(Event)]
pub struct ResetBaseGraphEvent;

#[derive(Event)]
pub struct AddNodeToBaseGraphEvent {
    pub name: String,
    pub labels: Vec<String>,
    pub position: Option<Vec3>,
    pub subgraph_id: Option<Uuid>,
}

/// Component to mark visual nodes in the world
#[derive(Component)]
pub struct VisualNodeComponent {
    pub node_id: Uuid,
    pub subgraph_id: Option<Uuid>,
}

/// Component to mark visual edges in the world
#[derive(Component)]
pub struct VisualEdgeComponent {
    pub edge_id: Uuid,
    pub source: Uuid,
    pub target: Uuid,
}

/// System to set up the editor environment
fn setup_editor_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut editor_state: ResMut<EditorState>,
) {
    info!("Setting up simple 2D environment...");

    // Fix 2D camera setup - position at origin with no scale
    let camera_2d = commands
        .spawn((
            Camera2d,
            Transform::from_xyz(0.0, 0.0, 0.0), // Position at origin for 2D
            Name::new("2D Camera"),
        ))
        .id();

    editor_state.camera_2d = Some(camera_2d);
    editor_state.camera_3d = None;

    info!("2D Camera created at origin (0,0,0)");

    // Create a larger test rectangle to verify 2D rendering
    let test_entity = commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(100.0, 100.0))), // Larger test rectangle
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(1.0, 0.0, 0.0)))),
            Transform::from_xyz(0.0, 0.0, 0.0), // At origin
            Name::new("Red Test Rectangle"),
        ))
        .id();

    info!("Created red test rectangle at origin: {:?}", test_entity);
    info!("Editor environment setup complete - 2D camera with test rectangle");
}

/// System to handle pattern addition events
fn handle_add_pattern_events(
    mut events: EventReader<AddPatternToBaseGraphEvent>,
    mut base_graph: ResMut<BaseGraphResource>,
    mut editor_state: ResMut<EditorState>,
    theme: Res<AlchemistTheme>,
) {
    for event in events.read() {
        info!("Adding pattern {:?} to base graph", event.pattern);

        // Generate the pattern graph
        let mut pattern_graph = generate_pattern(event.pattern.clone());

        // Ensure all nodes have positions - if not, generate them
        let node_ids: Vec<Uuid> = pattern_graph.nodes.keys().cloned().collect();
        let node_count = node_ids.len() as f32;

        for (i, node_id) in node_ids.iter().enumerate() {
            if !pattern_graph.node_positions.contains_key(node_id) {
                // Generate circular layout for pattern
                let angle = (i as f32 / node_count) * 2.0 * std::f32::consts::PI;
                let radius = 3.0;
                let x = radius * angle.cos();
                let y = radius * angle.sin();
                pattern_graph
                    .node_positions
                    .insert(*node_id, egui::Pos2::new(x, y));
            }
        }

        // Create subgraph info
        let subgraph_id = Uuid::new_v4();
        let color = get_color_for_subgraph(base_graph.next_subgraph_id, &theme);

        // Calculate offset for new subgraph to avoid overlaps
        let offset = calculate_subgraph_offset(&base_graph.subgraphs);

        // Add pattern nodes to base graph with offset
        let mut added_nodes = Vec::new();
        for (node_id, node) in &pattern_graph.nodes {
            let new_node_id = base_graph.graph.add_node(&node.name, node.labels.clone());
            added_nodes.push(new_node_id);

            // Apply offset to position
            if let Some(pos) = pattern_graph.node_positions.get(node_id) {
                let new_pos = egui::Pos2::new(pos.x + offset.x, pos.y + offset.y);
                base_graph.graph.node_positions.insert(new_node_id, new_pos);
                info!(
                    "Set position for node {}: ({}, {})",
                    new_node_id, new_pos.x, new_pos.y
                );
            }
        }

        // Add pattern edges to base graph
        let node_mapping: HashMap<Uuid, Uuid> = pattern_graph
            .nodes
            .keys()
            .zip(added_nodes.iter())
            .map(|(old_id, new_id)| (*old_id, *new_id))
            .collect();

        for (_, edge) in &pattern_graph.edges {
            if let (Some(&new_source), Some(&new_target)) = (
                node_mapping.get(&edge.source),
                node_mapping.get(&edge.target),
            ) {
                base_graph
                    .graph
                    .add_edge(new_source, new_target, edge.labels.clone());
            }
        }

        // Store subgraph info
        let subgraph_info = SubgraphInfo {
            id: subgraph_id,
            name: event.name.clone(),
            pattern_type: format!("{:?}", event.pattern),
            nodes: added_nodes,
            color,
        };

        base_graph.subgraphs.insert(subgraph_id, subgraph_info);
        base_graph.next_subgraph_id += 1;

        // Select the new subgraph
        editor_state.selected_subgraph = Some(subgraph_id);

        // Mark visual as dirty
        base_graph.visual_dirty = true;

        info!(
            "Added pattern '{}' with {} nodes as subgraph {} (total nodes: {}, total edges: {})",
            event.name,
            pattern_graph.nodes.len(),
            subgraph_id,
            base_graph.graph.nodes.len(),
            base_graph.graph.edges.len()
        );
    }
}

/// System to handle node addition events
fn handle_add_node_events(
    mut events: EventReader<AddNodeToBaseGraphEvent>,
    mut base_graph: ResMut<BaseGraphResource>,
) {
    for event in events.read() {
        let node_id = base_graph.graph.add_node(&event.name, event.labels.clone());

        if let Some(pos) = event.position {
            base_graph
                .graph
                .node_positions
                .insert(node_id, egui::Pos2::new(pos.x, pos.z));
        }

        // Add to subgraph if specified
        if let Some(subgraph_id) = event.subgraph_id {
            if let Some(subgraph) = base_graph.subgraphs.get_mut(&subgraph_id) {
                subgraph.nodes.push(node_id);
            }
        }

        // Mark visual as dirty
        base_graph.visual_dirty = true;

        info!("Added node '{}' to base graph", event.name);
    }
}

/// System to handle editor mode switching
fn handle_mode_switch_events(
    mut events: EventReader<SwitchEditorModeEvent>,
    mut editor_mode: ResMut<EditorMode>,
    _camera_query: Query<&mut Camera>,
    _editor_state: Res<EditorState>,
) {
    for event in events.read() {
        editor_mode.mode = event.mode.clone();

        // For now, just log the mode switch instead of changing cameras to debug rendering
        info!("Mode switch requested to: {:?}", event.mode);

        // TODO: Re-enable camera switching once basic rendering works
        /*
        // Switch camera active state
        if let (Some(cam_2d), Some(cam_3d)) = (editor_state.camera_2d, editor_state.camera_3d) {
            match event.mode {
                ViewMode::Mode2D => {
                    if let Ok(mut camera) = camera_query.get_mut(cam_2d) {
                        camera.is_active = true;
                    }
                    if let Ok(mut camera) = camera_query.get_mut(cam_3d) {
                        camera.is_active = false;
                    }
                    info!("Switched to 2D view mode");
                }
                ViewMode::Mode3D => {
                    if let Ok(mut camera) = camera_query.get_mut(cam_2d) {
                        camera.is_active = false;
                    }
                    if let Ok(mut camera) = camera_query.get_mut(cam_3d) {
                        camera.is_active = true;
                    }
                    info!("Switched to 3D view mode");
                }
            }
        }
        */
    }
}

/// System to handle base graph reset
fn handle_reset_base_graph_events(
    mut events: EventReader<ResetBaseGraphEvent>,
    mut base_graph: ResMut<BaseGraphResource>,
    mut editor_state: ResMut<EditorState>,
    mut commands: Commands,
    visual_query: Query<Entity, Or<(With<VisualNodeComponent>, With<VisualEdgeComponent>)>>,
) {
    for _event in events.read() {
        // Clear existing visual entities more safely
        for entity in visual_query.iter() {
            commands.entity(entity).despawn();
        }

        // Reset base graph
        base_graph.graph = AlchemistGraph::new();
        base_graph.subgraphs.clear();
        base_graph.next_subgraph_id = 1;

        // Reset editor state
        editor_state.selected_subgraph = None;
        editor_state.selected_node = None;

        // Mark visual as dirty
        base_graph.visual_dirty = true;

        info!("Base graph has been reset");
    }
}

/// System to update visual representation of the base graph
fn update_visual_representation(
    mut commands: Commands,
    mut base_graph: ResMut<BaseGraphResource>,
    editor_mode: Res<EditorMode>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    visual_query: Query<Entity, Or<(With<VisualNodeComponent>, With<VisualEdgeComponent>)>>,
) {
    // Only update if visual is dirty or editor mode changed
    if !base_graph.visual_dirty && !editor_mode.is_changed() {
        return;
    }

    // Clear the dirty flag early to prevent multiple updates
    base_graph.visual_dirty = false;

    // Clear existing visual entities more safely
    let visual_count = visual_query.iter().count();
    for entity in visual_query.iter() {
        commands.entity(entity).despawn();
    }

    if visual_count > 0 {
        info!("Cleared {} existing visual entities", visual_count);
    }

    if base_graph.graph.nodes.is_empty() {
        info!("Updated visual representation: 0 nodes, 0 edges");
        return;
    }

    info!(
        "Updating visual representation for {} nodes, {} edges",
        base_graph.graph.nodes.len(),
        base_graph.graph.edges.len()
    );

    // Create visual nodes with consistent positioning
    let mut nodes_created = 0;
    for (node_id, node) in &base_graph.graph.nodes {
        // Ensure node has a position - this should be set when the node is created
        let graph_pos = base_graph
            .graph
            .node_positions
            .get(node_id)
            .copied()
            .unwrap_or_else(|| {
                // Only generate fallback position if absolutely necessary
                warn!("Node {} missing position, generating fallback", node_id);
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                node_id.hash(&mut hasher);
                let hash = hasher.finish();
                let angle = (hash as f32 / u64::MAX as f32) * 2.0 * std::f32::consts::PI;
                let radius = 5.0 + (hash % 5) as f32;
                egui::Pos2::new(radius * angle.cos(), radius * angle.sin())
            });

        let position = Vec3::new(graph_pos.x * 50.0, graph_pos.y * 50.0, 0.0);

        info!("Creating 2D node at position: {:?}", position);

        // Determine color based on subgraph
        let color = base_graph
            .subgraphs
            .values()
            .find(|subgraph| subgraph.nodes.contains(node_id))
            .map(|subgraph| subgraph.color)
            .unwrap_or(Color::srgb(0.8, 0.8, 0.8));

        let subgraph_id = base_graph
            .subgraphs
            .values()
            .find(|subgraph| subgraph.nodes.contains(node_id))
            .map(|subgraph| subgraph.id);

        // Create 2D visual representation
        let mesh = meshes.add(Circle::new(30.0));

        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from(color))),
            Transform::from_translation(position),
            VisualNodeComponent {
                node_id: *node_id,
                subgraph_id,
            },
            Name::new(format!("Node: {}", node.name)),
        ));

        nodes_created += 1;
    }

    // Create 2D edges
    let mut edges_created = 0;
    for (edge_id, edge) in &base_graph.graph.edges {
        if let (Some(source_pos), Some(target_pos)) = (
            base_graph.graph.node_positions.get(&edge.source),
            base_graph.graph.node_positions.get(&edge.target),
        ) {
            let src_pos = Vec3::new(source_pos.x * 50.0, source_pos.y * 50.0, -0.1);
            let tgt_pos = Vec3::new(target_pos.x * 50.0, target_pos.y * 50.0, -0.1);

            let direction = tgt_pos - src_pos;
            let distance = direction.length();

            if distance > 0.01 {
                let mid_point = src_pos + direction * 0.5;
                let rotation = Quat::from_rotation_z(direction.y.atan2(direction.x));

                let mesh = meshes.add(Rectangle::new(distance, 3.0));

                commands.spawn((
                    Mesh2d(mesh),
                    MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.6, 0.6, 0.6)))),
                    Transform::from_translation(mid_point).with_rotation(rotation),
                    VisualEdgeComponent {
                        edge_id: *edge_id,
                        source: edge.source,
                        target: edge.target,
                    },
                    Name::new(format!("Edge: {} -> {}", edge.source, edge.target)),
                ));

                edges_created += 1;
            }
        }
    }

    info!(
        "Updated visual representation: {} nodes created, {} edges created",
        nodes_created, edges_created
    );
}

/// Simple camera controls for better navigation
fn handle_camera_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    for mut transform in camera_query.iter_mut() {
        let mut movement = Vec3::ZERO;
        let speed = 10.0; // Increased speed for better navigation

        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
            movement.y += speed;
        }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
            movement.y -= speed;
        }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            movement.x -= speed;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            movement.x += speed;
        }

        // Zoom controls
        if keys.pressed(KeyCode::KeyQ) {
            transform.scale *= 1.05; // Zoom out
        }
        if keys.pressed(KeyCode::KeyE) {
            transform.scale *= 0.95; // Zoom in
        }

        transform.translation += movement;
    }
}

/// Helper functions
fn get_color_for_subgraph(index: u32, theme: &AlchemistTheme) -> Color {
    let theme_colors = theme.current_theme.get_subgraph_colors();
    theme_colors[(index as usize - 1) % theme_colors.len()]
}

fn calculate_subgraph_offset(existing_subgraphs: &HashMap<Uuid, SubgraphInfo>) -> egui::Pos2 {
    let count = existing_subgraphs.len() as f32;
    if count == 0.0 {
        return egui::Pos2::ZERO;
    }

    // Use a more predictable grid-based layout instead of circular
    let grid_size = (count.sqrt().ceil() as i32).max(1);
    let x_offset = (count as i32 % grid_size) as f32 * 8.0; // 8.0 units apart
    let y_offset = (count as i32 / grid_size) as f32 * 8.0;

    egui::Pos2::new(x_offset, y_offset)
}

#[allow(dead_code)]
fn debug_rendering(
    _gizmos: Gizmos,
    _graph_data: Res<crate::graph_core::GraphData>,
) {
    // Debug code will be added as needed
}
