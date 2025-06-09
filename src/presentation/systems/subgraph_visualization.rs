//! Subgraph visualization system
//!
//! Provides visual boundaries and regions for subgraphs in the graph editor.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::presentation::components::{
    GraphNode, SubgraphRegion, SubgraphBoundary, SubgraphMember,
    SubgraphId, BoundaryType,
    SubgraphOrigin, SubgraphBoundaryStyle,
};
use crate::domain::value_objects::{Position3D};
use std::collections::HashSet;
use tracing::info;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;

/// Settings for subgraph boundary visualization
#[derive(Resource)]
pub struct SubgraphBoundarySettings {
    pub enabled: bool,
    pub default_style: SubgraphBoundaryStyle,
    pub update_interval: f32,
}

impl Default for SubgraphBoundarySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            default_style: SubgraphBoundaryStyle::ConvexHull,
            update_interval: 0.5,
        }
    }
}

/// Updates subgraph boundaries when nodes move or membership changes
pub fn update_subgraph_boundaries(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    subgraphs: Query<(Entity, &SubgraphRegion, Option<&Children>)>,
    nodes: Query<(&GraphNode, &Transform), Without<SubgraphBoundary>>,
    mut boundaries: Query<(&mut Mesh3d, &SubgraphBoundary)>,
) {
    for (subgraph_entity, region, children) in subgraphs.iter() {
        // Collect positions of all nodes in this subgraph
        let mut node_positions = Vec::new();

        for node_id in &region.nodes {
            for (graph_node, transform) in nodes.iter() {
                if &graph_node.node_id == node_id {
                    node_positions.push(transform.translation);
                }
            }
        }

        if node_positions.is_empty() {
            continue;
        }

        // Find existing boundary entity if it exists
        let boundary_entity = if let Some(children) = children {
            children.iter()
                .find(|&child| boundaries.get(child).is_ok())
                .map(|entity| entity)
        } else {
            None
        };

        // Create or update boundary mesh
        match region.boundary_type {
            BoundaryType::ConvexHull => {
                let mesh = create_convex_hull_mesh(&node_positions, region.color);

                if let Some(entity) = boundary_entity {
                    if let Ok((mut mesh_handle, _)) = boundaries.get_mut(entity) {
                        *mesh_handle = Mesh3d(meshes.add(mesh));
                    }
                } else {
                    spawn_boundary_entity(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        subgraph_entity,
                        region,
                        mesh,
                        &node_positions,
                    );
                }
            }
            BoundaryType::BoundingBox => {
                let mesh = create_bounding_box_mesh(&node_positions, region.color);

                if let Some(entity) = boundary_entity {
                    if let Ok((mut mesh_handle, _)) = boundaries.get_mut(entity) {
                        *mesh_handle = Mesh3d(meshes.add(mesh));
                    }
                } else {
                    spawn_boundary_entity(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        subgraph_entity,
                        region,
                        mesh,
                        &node_positions,
                    );
                }
            }
            BoundaryType::Circle => {
                let mesh = create_circle_boundary_mesh(&node_positions, region.color);

                if let Some(entity) = boundary_entity {
                    if let Ok((mut mesh_handle, _)) = boundaries.get_mut(entity) {
                        *mesh_handle = Mesh3d(meshes.add(mesh));
                    }
                } else {
                    spawn_boundary_entity(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        subgraph_entity,
                        region,
                        mesh,
                        &node_positions,
                    );
                }
            }
            BoundaryType::Voronoi => {
                // Voronoi boundaries are handled by the voronoi tessellation system
                // Just skip here
            }
        }
    }
}

/// Spawn a new boundary entity
fn spawn_boundary_entity(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent: Entity,
    region: &SubgraphRegion,
    mesh: Mesh,
    node_positions: &[Vec3],
) {
    let material = materials.add(StandardMaterial {
        base_color: region.color.with_alpha(0.2),
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    let center = if node_positions.is_empty() {
        Vec3::ZERO
    } else {
        node_positions.iter().sum::<Vec3>() / node_positions.len() as f32
    };

    let mesh_handle = meshes.add(mesh);

    let boundary_entity = commands.spawn((
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(material),
        Transform::default(),
        SubgraphBoundary {
            subgraph_id: region.subgraph_id,
            mesh_needs_update: false,
            center,
            mesh: Some(mesh_handle),
        },
    )).id();

    commands.entity(parent).add_child(boundary_entity);
}

/// Create a convex hull mesh from node positions
fn create_convex_hull_mesh(positions: &[Vec3], color: Color) -> Mesh {
    if positions.len() < 3 {
        return Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    }

    // Project to 2D for convex hull calculation
    let points_2d: Vec<(f32, f32)> = positions
        .iter()
        .map(|p| (p.x, p.z))
        .collect();

    // Calculate convex hull
    let hull_indices = convex_hull_2d(&points_2d);

    // Create mesh vertices with color
    let vertices: Vec<[f32; 3]> = hull_indices
        .iter()
        .map(|&i| [positions[i].x, positions[i].y, positions[i].z])
        .collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());

    // Add vertex colors based on the provided color
    let vertex_colors: Vec<[f32; 4]> = vertices
        .iter()
        .map(|_| {
            let rgba = color.to_linear();
            [rgba.red, rgba.green, rgba.blue, rgba.alpha]
        })
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);

    // Generate normals (pointing up)
    let normals: Vec<[f32; 3]> = vertices.iter().map(|_| [0.0, 1.0, 0.0]).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    // Generate indices for triangulation
    let indices = triangulate_convex_polygon_indices(vertices.len());
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Create a bounding box mesh from node positions
fn create_bounding_box_mesh(positions: &[Vec3], color: Color) -> Mesh {
    if positions.is_empty() {
        return Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    }

    // Find min and max bounds
    let mut min = positions[0];
    let mut max = positions[0];

    for pos in positions {
        min = min.min(*pos);
        max = max.max(*pos);
    }

    // Add padding
    let padding = Vec3::splat(2.0);
    min -= padding;
    max += padding;

    // Create box vertices
    let vertices = vec![
        [min.x, min.y, min.z], // 0
        [max.x, min.y, min.z], // 1
        [max.x, max.y, min.z], // 2
        [min.x, max.y, min.z], // 3
        [min.x, min.y, max.z], // 4
        [max.x, min.y, max.z], // 5
        [max.x, max.y, max.z], // 6
        [min.x, max.y, max.z], // 7
    ];

    // Create vertex colors
    let vertex_colors: Vec<[f32; 4]> = vertices
        .iter()
        .map(|_| {
            let rgba = color.to_linear();
            [rgba.red, rgba.green, rgba.blue, rgba.alpha]
        })
        .collect();

    // Box indices (12 triangles, 2 per face)
    let indices = vec![
        // Front
        0, 1, 2, 0, 2, 3,
        // Back
        5, 4, 7, 5, 7, 6,
        // Left
        4, 0, 3, 4, 3, 7,
        // Right
        1, 5, 6, 1, 6, 2,
        // Top
        3, 2, 6, 3, 6, 7,
        // Bottom
        4, 5, 1, 4, 1, 0,
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();

    mesh
}

/// Create a circular boundary mesh from node positions
fn create_circle_boundary_mesh(positions: &[Vec3], color: Color) -> Mesh {
    if positions.is_empty() {
        return Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    }

    // Find center and radius
    let center = positions.iter().sum::<Vec3>() / positions.len() as f32;
    let radius = positions
        .iter()
        .map(|p| (*p - center).length())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(10.0) + 2.0; // Add padding

    // Create circle vertices
    let segments = 32;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Center vertex
    vertices.push([center.x, center.y, center.z]);

    // Circle vertices
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = center.x + radius * angle.cos();
        let z = center.z + radius * angle.sin();
        vertices.push([x, center.y, z]);
    }

    // Create vertex colors
    let vertex_colors: Vec<[f32; 4]> = vertices
        .iter()
        .map(|_| {
            let rgba = color.to_linear();
            [rgba.red, rgba.green, rgba.blue, rgba.alpha * 0.5] // Semi-transparent
        })
        .collect();

    // Create triangles
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(0);
        indices.push((i + 1) as u32);
        indices.push((next + 1) as u32);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();

    mesh
}

/// Create a subgraph from selected nodes
pub fn create_subgraph_from_selection(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    selected_nodes: Query<(Entity, &GraphNode), With<Selected>>,
) {
    // Ctrl+G to create subgraph from selection
    if keyboard.just_pressed(KeyCode::KeyG) && keyboard.pressed(KeyCode::ControlLeft) {
        let nodes: Vec<_> = selected_nodes.iter().collect();

        if nodes.len() < 2 {
            info!("Need at least 2 nodes to create a subgraph");
            return;
        }

        let subgraph_id = SubgraphId::new();
        let mut node_ids = HashSet::new();

        // Add SubgraphMember component to selected nodes
        for (entity, node) in &nodes {
            node_ids.insert(node.node_id);

            // Convert UUID to usize using hash
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            subgraph_id.0.hash(&mut hasher);
            let subgraph_id_usize = hasher.finish() as usize;

            commands.entity(*entity).insert(SubgraphMember {
                subgraph_id: subgraph_id_usize,
                relative_position: crate::domain::value_objects::Position3D { x: 0.0, y: 0.0, z: 0.0 },
            });
        }

        // Create subgraph region
        commands.spawn((
            SubgraphRegion {
                subgraph_id,
                name: format!("Subgraph {}", subgraph_id.0),
                color: Color::srgb(0.2, 0.6, 0.9),
                nodes: node_ids,
                boundary_type: BoundaryType::ConvexHull,
            },
            Transform::default(),
            Visibility::default(),
        ));

        info!("Created subgraph with {} nodes", nodes.len());
    }
}

/// Marker component for selected nodes
#[derive(Component)]
pub struct Selected;

/// System to toggle subgraph boundary types
pub fn toggle_subgraph_boundary_type(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut subgraphs: Query<&mut SubgraphRegion>,
) {
    if keyboard.just_pressed(KeyCode::KeyB) {
        for mut subgraph in subgraphs.iter_mut() {
            subgraph.boundary_type = match subgraph.boundary_type {
                BoundaryType::ConvexHull => BoundaryType::BoundingBox,
                BoundaryType::BoundingBox => BoundaryType::Circle,
                BoundaryType::Circle => BoundaryType::Voronoi,
                BoundaryType::Voronoi => BoundaryType::ConvexHull,
            };
            info!("Switched to {:?} boundary type", subgraph.boundary_type);
        }
    }
}

/// Display subgraph controls help
pub fn display_subgraph_help() {
    eprintln!("Subgraph Controls:");
    eprintln!("  Ctrl+G - Create subgraph from selected nodes");
    eprintln!("  B - Toggle boundary type (ConvexHull/BoundingBox/Circle)");

    info!("Subgraph Controls:");
    info!("  Ctrl+G - Create subgraph from selected nodes");
    info!("  B - Toggle boundary type (ConvexHull/BoundingBox/Circle)");
}

/// Plugin for subgraph visualization
pub struct SubgraphVisualizationPlugin;

impl Plugin for SubgraphVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SubgraphBoundarySettings>()
            .add_systems(
                Update,
                (
                    update_subgraph_boundaries,
                    visualize_subgraph_boundaries,
                )
                    .chain(),
            );
    }
}

// Helper function to calculate 2D convex hull
fn convex_hull_2d(points: &[(f32, f32)]) -> Vec<usize> {
    if points.len() < 3 {
        return (0..points.len()).collect();
    }

    // Simple Graham scan algorithm
    let mut indices: Vec<usize> = (0..points.len()).collect();

    // Find the bottom-most point (or left-most if tied)
    let start = indices.iter()
        .min_by(|&&a, &&b| {
            let pa = points[a];
            let pb = points[b];
            pa.1.partial_cmp(&pb.1).unwrap()
                .then(pa.0.partial_cmp(&pb.0).unwrap())
        })
        .copied()
        .unwrap();

    // Sort points by polar angle with respect to start point
    let start_point = points[start];
    indices.sort_by(|&a, &b| {
        if a == start { return std::cmp::Ordering::Less; }
        if b == start { return std::cmp::Ordering::Greater; }

        let pa = points[a];
        let pb = points[b];

        let angle_a = (pa.1 - start_point.1).atan2(pa.0 - start_point.0);
        let angle_b = (pb.1 - start_point.1).atan2(pb.0 - start_point.0);

        angle_a.partial_cmp(&angle_b).unwrap()
    });

    // Build convex hull
    let mut hull = vec![indices[0], indices[1]];

    for &idx in &indices[2..] {
        while hull.len() > 1 {
            let p1 = points[hull[hull.len() - 2]];
            let p2 = points[hull[hull.len() - 1]];
            let p3 = points[idx];

            // Check if we make a left turn
            let cross = (p2.0 - p1.0) * (p3.1 - p1.1) - (p2.1 - p1.1) * (p3.0 - p1.0);
            if cross > 0.0 {
                break;
            }
            hull.pop();
        }
        hull.push(idx);
    }

    hull
}

// Helper function to triangulate a convex polygon
fn triangulate_convex_polygon_indices(vertex_count: usize) -> Vec<u32> {
    let mut indices = Vec::new();

    // Simple fan triangulation from first vertex
    for i in 1..vertex_count - 1 {
        indices.push(0);
        indices.push(i as u32);
        indices.push((i + 1) as u32);
    }

    indices
}

/// Visualizes subgraph boundaries using gizmos
pub fn visualize_subgraph_boundaries(
    mut gizmos: Gizmos,
    boundaries: Query<(&SubgraphBoundary, &SubgraphBoundaryStyle)>,
    settings: Res<SubgraphBoundarySettings>,
) {
    if !settings.enabled {
        return;
    }

    for (boundary, style) in boundaries.iter() {
        let base_color = style.color();
        let alpha = style.alpha();
        let color = base_color.with_alpha(alpha);

        match style {
            SubgraphBoundaryStyle::ConvexHull => {
                // Draw convex hull outline
                if let Some(_mesh) = &boundary.mesh {
                    // For now, draw a simple outline
                    // In a real implementation, extract vertices from mesh
                    gizmos.circle(
                        Isometry3d::from_translation(boundary.center),
                        10.0,
                        color,
                    );
                }
            }
            SubgraphBoundaryStyle::BoundingBox => {
                // Draw bounding box
                let half_size = Vec3::new(10.0, 1.0, 10.0);
                gizmos.cuboid(
                    Transform::from_translation(boundary.center)
                        .with_scale(half_size * 2.0),
                    color,
                );
            }
            SubgraphBoundaryStyle::Circle => {
                // Draw circle
                gizmos.circle(
                    Isometry3d::from_translation(boundary.center),
                    15.0,
                    color,
                );
            }
        }
    }
}
