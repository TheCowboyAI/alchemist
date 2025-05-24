use super::components::*;
use crate::camera::{GraphViewCamera, ViewMode};
use bevy::prelude::*;

/// System to render graph nodes in the appropriate mode
pub fn render_graph_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_3d: ResMut<Assets<StandardMaterial>>,
    mut materials_2d: ResMut<Assets<ColorMaterial>>,
    camera_query: Query<&GraphViewCamera>,
    node_query: Query<(Entity, &GraphNode, &GraphPosition, &NodeVisual), Changed<GraphPosition>>,
    existing_meshes: Query<(Entity, &ChildOf), Or<(With<Mesh3d>, With<Mesh2d>)>>,
) {
    let Ok(camera) = camera_query.single() else {
        return;
    };

    // Clean up existing meshes when switching modes
    for (mesh_entity, child_of) in &existing_meshes {
        if node_query.contains(child_of.parent()) {
            commands.entity(mesh_entity).despawn();
        }
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
    node_query: &Query<(Entity, &GraphNode, &GraphPosition, &NodeVisual), Changed<GraphPosition>>,
) {
    for (entity, _node, _position, visual) in node_query {
        // Create 3D sphere for node
        let mesh = meshes.add(Sphere::new(0.5).mesh().uv(32, 18));
        let material = materials.add(StandardMaterial {
            base_color: visual.current_color,
            ..default()
        });

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::ZERO),
            ));
        });
    }
}

fn render_nodes_2d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    node_query: &Query<(Entity, &GraphNode, &GraphPosition, &NodeVisual), Changed<GraphPosition>>,
) {
    for (entity, _node, _position, visual) in node_query {
        // Create 2D circle for node
        let mesh = meshes.add(Circle::new(20.0));
        let material = materials.add(ColorMaterial::from(visual.current_color));

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_translation(Vec3::ZERO),
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
    edge_query: Query<(Entity, &GraphEdge, &EdgeVisual), Changed<Transform>>,
    existing_meshes: Query<(Entity, &ChildOf), Or<(With<Mesh3d>, With<Mesh2d>)>>,
) {
    let Ok(camera) = camera_query.single() else {
        return;
    };

    // Clean up existing meshes when switching modes
    for (mesh_entity, child_of) in &existing_meshes {
        if edge_query.contains(child_of.parent()) {
            commands.entity(mesh_entity).despawn();
        }
    }

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
    edge_query: &Query<(Entity, &GraphEdge, &EdgeVisual), Changed<Transform>>,
) {
    for (entity, _edge, visual) in edge_query {
        // Create 3D cylinder for edge
        let mesh = meshes.add(Cylinder::new(visual.width * 0.1, 1.0).mesh());
        let material = materials.add(StandardMaterial {
            base_color: visual.color,
            ..default()
        });

        commands.entity(entity).with_children(|parent| {
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
    edge_query: &Query<(Entity, &GraphEdge, &EdgeVisual), Changed<Transform>>,
) {
    for (entity, _edge, visual) in edge_query {
        // Create 2D rectangle for edge
        let mesh = meshes.add(Rectangle::new(1.0, visual.width));
        let material = materials.add(ColorMaterial::from(visual.color));

        commands.entity(entity).with_children(|parent| {
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
