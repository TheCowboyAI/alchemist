use super::components::*;
use crate::camera::{GraphViewCamera, ViewMode};
use bevy::prelude::*;
use uuid;

/// Resource to track edge mesh entities for cleanup
#[derive(Resource, Default)]
pub struct EdgeMeshTracker {
    edge_meshes: std::collections::HashMap<uuid::Uuid, Entity>,
    initial_render_done: bool,
}

impl EdgeMeshTracker {
    pub fn track(&mut self, edge_id: uuid::Uuid, entity: Entity) {
        self.edge_meshes.insert(edge_id, entity);
    }

    pub fn remove(&mut self, edge_id: &uuid::Uuid, commands: &mut Commands) {
        if let Some(entity) = self.edge_meshes.remove(edge_id) {
            commands.entity(entity).despawn();
        }
    }

    pub fn despawn_all(&mut self, commands: &mut Commands) {
        for entity in self.edge_meshes.values() {
            commands.entity(*entity).despawn();
        }
        self.edge_meshes.clear();
    }

    pub fn has_edge(&self, edge_id: &uuid::Uuid) -> bool {
        self.edge_meshes.contains_key(edge_id)
    }

    pub fn mark_initial_render_done(&mut self) {
        self.initial_render_done = true;
    }

    pub fn needs_initial_render(&self) -> bool {
        !self.initial_render_done
    }
}

/// Marker component for nodes that have been rendered
#[derive(Component)]
pub struct NodeRendered;

/// Marker component for edges that have been rendered (removed - edges aren't entities)
// #[derive(Component)]
// pub struct EdgeRendered;

/// Resource to track the last view mode to detect actual mode changes
#[derive(Resource, Default)]
pub struct LastViewMode {
    pub mode: Option<ViewMode>,
}

/// System to render graph nodes in the appropriate mode
pub fn render_graph_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_3d: ResMut<Assets<StandardMaterial>>,
    mut materials_2d: ResMut<Assets<ColorMaterial>>,
    camera_query: Query<&GraphViewCamera>,
    node_query: Query<(Entity, &GraphNode, &GraphPosition, &NodeVisual), Without<NodeRendered>>,
) {
    let Ok(camera) = camera_query.single() else {
        return;
    };

    // Debug: Check if we're rendering nodes
    let node_count = node_query.iter().count();
    if node_count > 0 {
        info!("Rendering {} nodes", node_count);
    }

    // Render nodes based on current view mode
    match camera.view_mode {
        ViewMode::ThreeD(_) => {
            render_nodes_3d(&mut commands, &mut meshes, &mut materials_3d, &node_query);
        }
        ViewMode::TwoD(_) => {
            render_nodes_2d(&mut commands, &mut meshes, &mut materials_2d, &node_query);
        }
    }
}

fn render_nodes_3d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    node_query: &Query<(Entity, &GraphNode, &GraphPosition, &NodeVisual), Without<NodeRendered>>,
) {
    for (entity, node, _position, visual) in node_query {
        // Create 3D sphere for node
        let mesh = meshes.add(Sphere::new(0.8).mesh().uv(32, 18));
        let material = materials.add(StandardMaterial {
            base_color: visual.current_color,
            emissive: visual.current_color.into(),
            emissive_exposure_weight: 0.5,
            ..default()
        });

        commands
            .entity(entity)
            .insert(NodeRendered)
            .with_children(|parent| {
                // Add the sphere mesh
                parent.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::ZERO),
                    Name::new(format!("NodeSphere_{}", node.name)),
                ));
            });
    }
}

fn render_nodes_2d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    node_query: &Query<(Entity, &GraphNode, &GraphPosition, &NodeVisual), Without<NodeRendered>>,
) {
    for (entity, _node, _position, visual) in node_query {
        // Create 2D circle for node
        let mesh = meshes.add(Circle::new(20.0));
        let material = materials.add(ColorMaterial::from(visual.current_color));

        commands
            .entity(entity)
            .insert(NodeRendered)
            .with_children(|parent| {
                parent.spawn((
                    Mesh2d(mesh),
                    MeshMaterial2d(material),
                    Transform::from_translation(Vec3::ZERO),
                ));
            });
    }
}

/// System to render edges using OutgoingEdge components on node entities
pub fn render_graph_edges(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    node_query: Query<(Entity, &Transform, Option<&OutgoingEdge>), With<GraphNode>>,
    all_nodes: Query<&Transform, With<GraphNode>>,
    mut edge_tracker: ResMut<EdgeMeshTracker>,
    // Track specific changes
    changed_nodes: Query<(Entity, &OutgoingEdge), (With<GraphNode>, Changed<OutgoingEdge>)>,
    added_nodes: Query<(Entity, &OutgoingEdge), (With<GraphNode>, Added<OutgoingEdge>)>,
    mut removed_edges: RemovedComponents<OutgoingEdge>,
) {
    let mut edges_updated = 0;

    // Handle removed edges
    for _entity in removed_edges.read() {
        // We can't easily track which specific edge was removed without more complex tracking
        // For now, we'll handle this in the main loop
    }

    // Handle added edges
    for (source_entity, outgoing_edge) in &added_nodes {
        if !edge_tracker.has_edge(&outgoing_edge.id) {
            if let Ok(source_tf) = node_query.get(source_entity) {
                if let Ok(target_tf) = all_nodes.get(outgoing_edge.target) {
                    create_edge_mesh(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &mut edge_tracker,
                        outgoing_edge,
                        source_tf.1.translation,
                        target_tf.translation,
                    );
                    edges_updated += 1;
                }
            }
        }
    }

    // Handle changed edges (update existing ones)
    for (source_entity, outgoing_edge) in &changed_nodes {
        // Remove old mesh and create new one
        edge_tracker.remove(&outgoing_edge.id, &mut commands);

        if let Ok(source_tf) = node_query.get(source_entity) {
            if let Ok(target_tf) = all_nodes.get(outgoing_edge.target) {
                create_edge_mesh(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut edge_tracker,
                    outgoing_edge,
                    source_tf.1.translation,
                    target_tf.translation,
                );
                edges_updated += 1;
            }
        }
    }

    // Initial render - create all edges that don't exist yet
    if edge_tracker.needs_initial_render() {
        for (_entity, source_tf, outgoing_edge_opt) in &node_query {
            if let Some(outgoing_edge) = outgoing_edge_opt {
                if !edge_tracker.has_edge(&outgoing_edge.id) {
                    if let Ok(target_tf) = all_nodes.get(outgoing_edge.target) {
                        create_edge_mesh(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &mut edge_tracker,
                            outgoing_edge,
                            source_tf.translation,
                            target_tf.translation,
                        );
                        edges_updated += 1;
                    }
                }
            }
        }
        edge_tracker.mark_initial_render_done();
    }

    if edges_updated > 0 {
        info!("Updated {} edge meshes", edges_updated);
    }
}

/// Helper function to create an edge mesh
fn create_edge_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    edge_tracker: &mut EdgeMeshTracker,
    outgoing_edge: &OutgoingEdge,
    source_pos: Vec3,
    target_pos: Vec3,
) {
    let distance = source_pos.distance(target_pos);

    if distance < 0.01 {
        return;
    }

    // Calculate midpoint and direction
    let midpoint = (source_pos + target_pos) / 2.0;
    let direction = (target_pos - source_pos).normalize();

    // Create cylinder mesh
    let cylinder = Cylinder::new(0.15, distance);
    let mesh = meshes.add(cylinder.mesh().resolution(8).build());

    // Create royal blue material
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.255, 0.412, 0.882), // Royal blue
        emissive: Color::srgb(0.255, 0.412, 0.882).lighter(0.3).into(),
        emissive_exposure_weight: 1.0,
        unlit: true,
        ..default()
    });

    // Calculate rotation to align cylinder with edge direction
    let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

    // Spawn the cylinder mesh directly at the correct position
    let edge_mesh = commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform {
            translation: midpoint,
            rotation,
            scale: Vec3::ONE,
        },
        Name::new(format!("EdgeMesh_{:?}", outgoing_edge.id)),
    )).id();

    edge_tracker.track(outgoing_edge.id, edge_mesh);
}

/// System to render a reference grid
pub fn render_reference_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_3d: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&GraphViewCamera>,
    grid_query: Query<Entity, With<ReferenceGrid>>,
    last_view_mode: Res<LastViewMode>,
) {
    let Ok(camera) = camera_query.single() else {
        return;
    };

    // Check if the view mode actually changed
    let current_mode = std::mem::discriminant(&camera.view_mode);
    let mode_changed = match &last_view_mode.mode {
        Some(last_mode) => std::mem::discriminant(last_mode) != current_mode,
        None => true, // First time, consider it changed
    };

    // Only run if camera view mode actually changed
    if !mode_changed {
        return;
    }

    // Remove existing grid when view mode changes
    for entity in &grid_query {
        commands.entity(entity).despawn();
    }

    match camera.view_mode {
        ViewMode::ThreeD(_) => {
            // Create 3D grid plane
            let mesh = meshes.add(Rectangle::new(100.0, 100.0));
            let material = materials_3d.add(StandardMaterial {
                base_color: Color::srgba(0.3, 0.3, 0.3, 0.2),
                alpha_mode: AlphaMode::Blend,
                double_sided: true,
                ..default()
            });

            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(0.0, -0.1, 0.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
                ReferenceGrid,
                Name::new("ReferenceGrid"),
            ));
        }
        ViewMode::TwoD(_) => {
            // 2D mode doesn't need a grid plane
        }
    }
}

/// Marker component for the reference grid
#[derive(Component)]
pub struct ReferenceGrid;

/// System to clear rendering when view mode changes
pub fn clear_rendering_on_view_change(
    mut commands: Commands,
    camera_query: Query<&GraphViewCamera>,
    rendered_nodes: Query<Entity, With<NodeRendered>>,
    children_query: Query<&Children>,
    mut edge_tracker: ResMut<EdgeMeshTracker>,
    mut last_view_mode: ResMut<LastViewMode>,
) {
    let Ok(camera) = camera_query.single() else {
        return;
    };

    // Check if the view mode actually changed
    let current_mode = std::mem::discriminant(&camera.view_mode);
    let mode_changed = match &last_view_mode.mode {
        Some(last_mode) => std::mem::discriminant(last_mode) != current_mode,
        None => true, // First time, consider it changed
    };

    if !mode_changed {
        return;
    }

    info!("Clearing rendering due to view mode change");

    // Remove NodeRendered component and despawn children
    for entity in &rendered_nodes {
        commands.entity(entity).remove::<NodeRendered>();
        if let Ok(children) = children_query.get(entity) {
            for child in children {
                commands.entity(*child).despawn();
            }
        }
    }

    // Clear edge meshes and reset initial render flag
    edge_tracker.despawn_all(&mut commands);
    edge_tracker.initial_render_done = false;

    // Update the last view mode
    last_view_mode.mode = Some(camera.view_mode);
}
