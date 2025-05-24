use super::components::*;
use crate::camera::{GraphViewCamera, ViewMode};
use bevy::prelude::*;

/// Marker component for nodes that have been rendered
#[derive(Component)]
pub struct NodeRendered;

/// Marker component for edges that have been rendered
#[derive(Component)]
pub struct EdgeRendered;

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
    for (entity, node, position, visual) in node_query {
        // Create 3D sphere for node
        let mesh = meshes.add(Sphere::new(0.5).mesh().uv(32, 18));
        let material = materials.add(StandardMaterial {
            base_color: visual.current_color,
            ..default()
        });

        commands
            .entity(entity)
            .insert(NodeRendered)
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::ZERO),
                ));
            });

        // Also spawn a bright cube above the node as a debug marker
        let debug_mesh = meshes.add(Cuboid::new(0.3, 0.3, 0.3));
        let debug_material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 0.0), // Bright yellow
            emissive: Color::srgb(1.0, 1.0, 0.0).into(),
            emissive_exposure_weight: 2.0,
            ..default()
        });

        // Use the GraphPosition
        let world_pos = position.0;

        // Spawn debug cube above node in world space
        commands.spawn((
            Mesh3d(debug_mesh),
            MeshMaterial3d(debug_material),
            Transform::from_translation(world_pos + Vec3::new(0.0, 1.5, 0.0)),
            Name::new(format!("DebugCube_{}", node.name)),
        ));

        info!("Created node '{}' at {:?} with debug cube", node.name, world_pos);
    }
}

fn render_nodes_2d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    node_query: &Query<(Entity, &GraphNode, &GraphPosition, &NodeVisual), Without<NodeRendered>>,
) {
    for (entity, node, _position, visual) in node_query {
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

                // Add text label
                parent.spawn((
                    Text2d::new(&node.name),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 30.0, 0.1), // Position above the node
                    bevy::sprite::Anchor::Center, // Center the text
                ));
            });
    }
}

/// System to render graph edges in the appropriate mode
pub fn render_graph_edges(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_3d: ResMut<Assets<StandardMaterial>>,
    mut materials_2d: ResMut<Assets<ColorMaterial>>,
    camera_query: Query<&GraphViewCamera>,
    edge_query: Query<(Entity, &GraphEdge, &EdgeVisual), Without<EdgeRendered>>,
    node_query: Query<&Transform, With<GraphNode>>,
) {
    let Ok(camera) = camera_query.single() else {
        return;
    };

    let edge_count = edge_query.iter().count();
    if edge_count > 0 {
        info!("Rendering {} edges", edge_count);
    }

    // Render edges based on current view mode
    match camera.view_mode {
        ViewMode::ThreeD(_) => {
            render_edges_3d(&mut commands, &mut meshes, &mut materials_3d, &edge_query, &node_query);
        }
        ViewMode::TwoD(_) => {
            render_edges_2d(&mut commands, &mut meshes, &mut materials_2d, &edge_query);
        }
    }
}

fn render_edges_3d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    edge_query: &Query<(Entity, &GraphEdge, &EdgeVisual), Without<EdgeRendered>>,
    node_query: &Query<&Transform, With<GraphNode>>,
) {
    for (entity, edge, visual) in edge_query {
        // Get source and target positions
        if let (Ok(source_transform), Ok(target_transform)) =
            (node_query.get(edge.source), node_query.get(edge.target))
        {
            let source_pos = source_transform.translation;
            let target_pos = target_transform.translation;
            let direction = target_pos - source_pos;
            let distance = direction.length();

            if distance < 0.01 {
                continue;
            }

            // Create a simple cylinder
            let cylinder = Cylinder::new(0.05, distance);
            let mesh = meshes.add(cylinder.mesh());

            let material = materials.add(StandardMaterial {
                base_color: visual.color,
                ..default()
            });

            // Position at midpoint
            let midpoint = source_pos + direction * 0.5;

            // Create rotation to align cylinder (Y-axis) with edge direction
            let rotation = if direction.normalize() != Vec3::Y {
                Quat::from_rotation_arc(Vec3::Y, direction.normalize())
            } else {
                Quat::IDENTITY
            };

            commands
                .entity(entity)
                .insert(EdgeRendered)
                .insert(Transform {
                    translation: midpoint,
                    rotation,
                    scale: Vec3::ONE,
                })
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform::default(),
                        Name::new("EdgeCylinder"),
                    ));
                });

            info!("Rendered edge from {:?} to {:?}", source_pos, target_pos);
        }
    }
}

fn render_edges_2d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    edge_query: &Query<(Entity, &GraphEdge, &EdgeVisual), Without<EdgeRendered>>,
) {
    for (entity, _edge, visual) in edge_query {
        // Create 2D rectangle for edge
        let mesh = meshes.add(Rectangle::new(1.0, visual.width));
        let material = materials.add(ColorMaterial::from(visual.color));

        commands
            .entity(entity)
            .insert(EdgeRendered)
            .with_children(|parent| {
                parent.spawn((
                    Mesh2d(mesh),
                    MeshMaterial2d(material),
                    // Don't override positioning - parent transform handles that
                    Transform::default(),
                ));
            });
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
            // Create 3D grid plane using a rectangle rotated to be horizontal
            let mesh = meshes.add(Rectangle::new(50.0, 50.0));
            let material = materials_3d.add(StandardMaterial {
                base_color: Color::srgba(0.5, 0.5, 0.5, 0.3),
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(0.0, -0.1, 0.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
                ReferenceGrid,
            ));
        }
        ViewMode::TwoD(_) => {
            // 2D mode doesn't need a grid plane, could add grid lines if needed
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
    rendered_edges: Query<Entity, With<EdgeRendered>>,
    children_query: Query<&Children>,
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
                commands.entity(*child).despawn_recursive();
            }
        }
    }

    // Remove EdgeRendered component and despawn children
    for entity in &rendered_edges {
        commands.entity(entity).remove::<EdgeRendered>();
        if let Ok(children) = children_query.get(entity) {
            for child in children {
                commands.entity(*child).despawn_recursive();
            }
        }
    }
}
