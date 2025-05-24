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
        warn!("No camera found for rendering!");
        return;
    };

    let node_count = node_query.iter().count();
    if node_count > 0 {
        info!("Found {} nodes to render in {:?} mode", node_count, camera.view_mode);
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
    let mut rendered_count = 0;
    for (entity, node, _position, visual) in node_query {
        rendered_count += 1;
        info!("Rendering 3D node '{}' at entity {:?}", node.name, entity);

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

                // Add text label
                parent.spawn((
                    Text2d::new(&node.name),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.8, 0.1), // Position above the node
                ));
            });
    }

    if rendered_count > 0 {
        info!("Rendered {} 3D nodes", rendered_count);
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
) {
    let Ok(camera) = camera_query.single() else {
        return;
    };

    // Render edges based on current view mode
    match camera.view_mode {
        ViewMode::ThreeD(_) => {
            render_edges_3d(&mut commands, &mut meshes, &mut materials_3d, &edge_query);
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
) {
    for (entity, _edge, visual) in edge_query {
        // Create 3D cylinder for edge
        let mesh = meshes.add(Cylinder::new(visual.width * 0.1, 1.0).mesh());
        let material = materials.add(StandardMaterial {
            base_color: visual.color,
            ..default()
        });

        commands
            .entity(entity)
            .insert(EdgeRendered)
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0)),
                ));
            });
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
                    Transform::from_translation(Vec3::ZERO),
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
