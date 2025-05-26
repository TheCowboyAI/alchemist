use super::components::*;
use crate::camera::{GraphViewCamera, ViewMode};
use crate::resources::EdgeMeshTracker;
use bevy::prelude::*;

/// Marker component for nodes that have been rendered
#[derive(Component)]
pub struct NodeRendered;

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

    // Only render if there are unrendered nodes
    let node_count = node_query.iter().count();
    if node_count == 0 {
        return;
    }

    debug!("Rendering {} nodes", node_count);

    // Render nodes based on current view mode
    match camera.view_mode {
        ViewMode::ThreeD(_) => {
            render_nodes_3d(&mut commands, &mut meshes, &mut materials_3d, &node_query);
        }
        ViewMode::TwoD(state) => {
            render_nodes_2d(&mut commands, &mut meshes, &mut materials_2d, &node_query, state.zoom_level);
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
                // Add the sphere mesh - it will inherit parent's transform
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
    zoom_level: f32,
) {
    for (entity, node, _position, visual) in node_query {
        // Scale nodes based on zoom, but maintain minimum/maximum size
        // When zoom_level is high (zoomed out), we want larger nodes
        // When zoom_level is low (zoomed in), we want smaller nodes
        let base_size = 0.5;
        let scaled_size = base_size * zoom_level.max(0.5).min(5.0);

        let mesh = meshes.add(Circle::new(scaled_size));
        let material = materials.add(ColorMaterial::from(visual.current_color));

        commands
            .entity(entity)
            .insert(NodeRendered)
            .with_children(|parent| {
                parent.spawn((
                    Mesh2d(mesh),
                    MeshMaterial2d(material),
                    Transform::from_translation(Vec3::ZERO),
                    Name::new(format!("NodeCircle_{}", node.name)),
                ));
            });
    }
}

/// System to render edges using OutgoingEdges components on node entities
pub fn render_graph_edges(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    node_query: Query<(Entity, &Transform, Option<&OutgoingEdges>), With<GraphNode>>,
    all_nodes: Query<&Transform, With<GraphNode>>,
    mut edge_tracker: ResMut<EdgeMeshTracker>,
    // Track specific changes
    changed_nodes: Query<(Entity, &OutgoingEdges), (With<GraphNode>, Changed<OutgoingEdges>)>,
    added_nodes: Query<(Entity, &OutgoingEdges), (With<GraphNode>, Added<OutgoingEdges>)>,
    mut removed_edges: RemovedComponents<OutgoingEdges>,
) {
    let mut edges_updated = 0;

    // Count how many nodes have OutgoingEdges components
    let nodes_with_edges = node_query.iter()
        .filter(|(_, _, edges)| edges.is_some() && !edges.unwrap().edges.is_empty())
        .count();

    if nodes_with_edges > 0 || !added_nodes.is_empty() || !changed_nodes.is_empty() {
        debug!("render_graph_edges: {} nodes with edges, {} added, {} changed",
              nodes_with_edges, added_nodes.iter().count(), changed_nodes.iter().count());
    }

    // Handle removed edges
    for _entity in removed_edges.read() {
        // We can't easily track which specific edge was removed without more complex tracking
        // For now, we'll handle this in the main loop
    }

    // Handle added edges
    for (source_entity, outgoing_edges) in &added_nodes {
        debug!("Processing {} added edges from entity {:?}", outgoing_edges.edges.len(), source_entity);
        if let Ok(source_tf) = node_query.get(source_entity) {
            for outgoing_edge in &outgoing_edges.edges {
                if !edge_tracker.has_edge(&outgoing_edge.id) {
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
    }

    // Handle changed edges (update existing ones)
    for (source_entity, outgoing_edges) in &changed_nodes {
        // Remove old meshes and create new ones
        for outgoing_edge in &outgoing_edges.edges {
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
    }

    // Initial render - create all edges that don't exist yet
    if edge_tracker.needs_initial_render() {
        debug!("Initial edge render - checking all nodes for edges");
        for (_entity, source_tf, outgoing_edges_opt) in &node_query {
            if let Some(outgoing_edges) = outgoing_edges_opt {
                for outgoing_edge in &outgoing_edges.edges {
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
        }
        edge_tracker.mark_initial_render_done();
    }

    if edges_updated > 0 {
        debug!("Updated {} edge meshes", edges_updated);
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

    // Create cylinder mesh with appropriate size
    let cylinder = Cylinder::new(0.1, distance);  // Reduced from 0.15 to 0.1
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
    camera_query: Query<&GraphViewCamera, Changed<GraphViewCamera>>,
    grid_query: Query<Entity, With<ReferenceGrid>>,
) {
    // Only run when camera view mode actually changed
    let Ok(camera) = camera_query.single() else {
        return;
    };

    // Remove existing grid
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
    camera_query: Query<&GraphViewCamera, Changed<GraphViewCamera>>,
    rendered_nodes: Query<Entity, With<NodeRendered>>,
    children_query: Query<&Children>,
    mut edge_tracker: ResMut<EdgeMeshTracker>,
) {
    // Only run when camera view mode actually changed
    if camera_query.single().is_err() {
        return;
    }

    debug!("Clearing rendering due to view mode change");

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
}

/// System to ensure edges are rendered even if change detection misses them
pub fn ensure_edges_rendered(
    node_query: Query<(Entity, &Transform, &OutgoingEdges), With<GraphNode>>,
    all_nodes: Query<&Transform, With<GraphNode>>,
    mut edge_tracker: ResMut<EdgeMeshTracker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Check if there are edges that need rendering
    let mut unrendered_edges = Vec::new();

    for (entity, source_tf, outgoing_edges) in &node_query {
        for edge in &outgoing_edges.edges {
            if !edge_tracker.has_edge(&edge.id) {
                unrendered_edges.push((entity, source_tf, edge));
            }
        }
    }

    if !unrendered_edges.is_empty() {
        debug!("Found {} unrendered edges, rendering them now", unrendered_edges.len());

        for (entity, source_tf, outgoing_edge) in unrendered_edges {
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
                debug!("Rendered edge {:?} from {:?} to {:?}",
                      outgoing_edge.id, entity, outgoing_edge.target);
            }
        }
    }
}
