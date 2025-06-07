//! Subgraph visualization system
//!
//! Provides visual boundaries and regions for subgraphs in the graph editor.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use crate::presentation::components::{
    GraphNode, SubgraphRegion, SubgraphBoundary, SubgraphMember,
    SubgraphId, BoundaryType,
};
use std::collections::{HashMap, HashSet};
use tracing::info;
use bevy::input::mouse::{MouseMotion, MouseWheel};

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
            children.iter().find(|&child| {
                boundaries.get(child).is_ok()
            }).copied()
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
                    );
                }
            }
            BoundaryType::Custom => {
                // Custom boundaries would be handled by specific implementations
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
) {
    let material = materials.add(StandardMaterial {
        base_color: region.color.with_alpha(0.2),
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    let boundary_entity = commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform::default(),
        SubgraphBoundary {
            subgraph_id: region.subgraph_id,
            mesh_needs_update: false,
        },
    )).id();

    commands.entity(parent).add_child(boundary_entity);
}

/// Create a convex hull mesh from node positions
fn create_convex_hull_mesh(positions: &[Vec3], color: Color) -> Mesh {
    // For now, create a simple polygon at y=0
    // In a real implementation, you'd use a proper convex hull algorithm
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Find the center
    let center = positions.iter().sum::<Vec3>() / positions.len() as f32;

    // Sort positions by angle from center
    let mut sorted_positions = positions.to_vec();
    sorted_positions.sort_by(|a, b| {
        let angle_a = (a.x - center.x).atan2(a.z - center.z);
        let angle_b = (b.x - center.x).atan2(b.z - center.z);
        angle_a.partial_cmp(&angle_b).unwrap()
    });

    // Create vertices with some padding
    let padding = 2.0;
    for pos in &sorted_positions {
        let dir = (*pos - center).normalize();
        let padded_pos = *pos + dir * padding;
        vertices.push([padded_pos.x, -0.5, padded_pos.z]); // Bottom
        vertices.push([padded_pos.x, 0.5, padded_pos.z]);  // Top
    }

    // Create triangles for the sides
    let vertex_count = sorted_positions.len();
    for i in 0..vertex_count {
        let next = (i + 1) % vertex_count;
        let bottom_current = (i * 2) as u32;
        let top_current = (i * 2 + 1) as u32;
        let bottom_next = (next * 2) as u32;
        let top_next = (next * 2 + 1) as u32;

        // Two triangles for each quad
        indices.extend_from_slice(&[bottom_current, bottom_next, top_next]);
        indices.extend_from_slice(&[bottom_current, top_next, top_current]);
    }

    // Create top and bottom faces
    for i in 0..vertex_count {
        let next = (i + 1) % vertex_count;
        // Bottom face
        indices.extend_from_slice(&[0, (next * 2) as u32, (i * 2) as u32]);
        // Top face
        indices.extend_from_slice(&[1, (i * 2 + 1) as u32, (next * 2 + 1) as u32]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();

    mesh
}

/// Create a bounding box mesh from node positions
fn create_bounding_box_mesh(positions: &[Vec3], color: Color) -> Mesh {
    // Find min and max bounds
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);

    for pos in positions {
        min = min.min(*pos);
        max = max.max(*pos);
    }

    // Add padding
    let padding = 2.0;
    min -= Vec3::splat(padding);
    max += Vec3::splat(padding);

    // Set y bounds for a thin box
    min.y = -0.5;
    max.y = 0.5;

    // Create box vertices
    let vertices = vec![
        // Bottom face
        [min.x, min.y, min.z],
        [max.x, min.y, min.z],
        [max.x, min.y, max.z],
        [min.x, min.y, max.z],
        // Top face
        [min.x, max.y, min.z],
        [max.x, max.y, min.z],
        [max.x, max.y, max.z],
        [min.x, max.y, max.z],
    ];

    let indices = vec![
        // Bottom
        0, 1, 2, 0, 2, 3,
        // Top
        4, 6, 5, 4, 7, 6,
        // Front
        0, 4, 5, 0, 5, 1,
        // Back
        2, 6, 7, 2, 7, 3,
        // Left
        0, 3, 7, 0, 7, 4,
        // Right
        1, 5, 6, 1, 6, 2,
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();

    mesh
}

/// Create a circular boundary mesh from node positions
fn create_circle_boundary_mesh(positions: &[Vec3], color: Color) -> Mesh {
    // Find center and radius
    let centroid = positions.iter().fold(Vec3::ZERO, |acc, p| acc + *p) / positions.len() as f32;

    // Find maximum radius
    let mut max_radius = 0.0f32;
    for pos in positions {
        let dist = (*pos - centroid).length();
        max_radius = max_radius.max(dist);
    }

    // Add padding
    max_radius += 2.0;

    // Create circle mesh
    let segments = 32;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Center vertices
    vertices.push([centroid.x, -0.5, centroid.z]); // Bottom center
    vertices.push([centroid.x, 0.5, centroid.z]);  // Top center

    // Circle vertices
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = centroid.x + angle.cos() * max_radius;
        let z = centroid.z + angle.sin() * max_radius;
        vertices.push([x, -0.5, z]); // Bottom
        vertices.push([x, 0.5, z]);  // Top
    }

    // Create triangles
    for i in 0..segments {
        let bottom_current = (i * 2 + 2) as u32;
        let top_current = (i * 2 + 3) as u32;
        let bottom_next = ((i + 1) * 2 + 2) as u32;
        let top_next = ((i + 1) * 2 + 3) as u32;

        // Bottom face
        indices.extend_from_slice(&[0, bottom_next, bottom_current]);
        // Top face
        indices.extend_from_slice(&[1, top_current, top_next]);
        // Side faces
        indices.extend_from_slice(&[bottom_current, bottom_next, top_next]);
        indices.extend_from_slice(&[bottom_current, top_next, top_current]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
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
            commands.entity(*entity).insert(SubgraphMember { subgraph_id });
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
