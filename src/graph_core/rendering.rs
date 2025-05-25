use super::components::*;
use crate::camera::{GraphViewCamera, ViewMode};
use bevy::prelude::*;

/// Resource to track edge mesh entities for cleanup
#[derive(Resource, Default)]
pub struct EdgeMeshTracker {
    edge_meshes: Vec<Entity>,
}

impl EdgeMeshTracker {
    pub fn track(&mut self, entity: Entity) {
        self.edge_meshes.push(entity);
    }

    pub fn despawn_all(&mut self, commands: &mut Commands) {
        for entity in self.edge_meshes.drain(..) {
            commands.entity(entity).despawn();
        }
    }
}

/// Marker component for nodes that have been rendered
#[derive(Component)]
pub struct NodeRendered;

/// Marker component for edges that have been rendered (removed - edges aren't entities)
// #[derive(Component)]
// pub struct EdgeRendered;

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
) {
    // Clear previous edge meshes
    edge_tracker.despawn_all(&mut commands);

    // For each node, render all its outgoing edges
    for (entity, source_tf, outgoing_edge_opt) in &node_query {
        if let Some(outgoing_edge) = outgoing_edge_opt {
            // Get the target node's transform
            if let Ok(target_tf) = all_nodes.get(outgoing_edge.target) {
                let source_pos = source_tf.translation;
                let target_pos = target_tf.translation;
                let distance = source_pos.distance(target_pos);

                if distance < 0.01 {
                    continue;
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

                edge_tracker.track(edge_mesh);
            }
        }
    }
}

/// System to render a reference grid
pub fn render_reference_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_3d: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&GraphViewCamera>,
    grid_query: Query<Entity, With<ReferenceGrid>>,
) {
    let Ok(camera) = camera_query.single() else {
        return;
    };

    // Remove existing grid if view mode changed
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
    // Only run if camera view mode changed
    if camera_query.iter().next().is_none() {
        return;
    }

    // Remove NodeRendered component and despawn children
    for entity in &rendered_nodes {
        commands.entity(entity).remove::<NodeRendered>();
        if let Ok(children) = children_query.get(entity) {
            for child in children {
                commands.entity(*child).despawn();
            }
        }
    }

    // Clear edge meshes
    edge_tracker.despawn_all(&mut commands);
}
