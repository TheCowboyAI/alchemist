//! Subgraph visualization and spatial mapping system
//!
//! This module provides functionality for managing subgraphs as spatial units
//! with their own coordinate systems and base origins.

use bevy::prelude::*;
use std::collections::HashMap;
use crate::presentation::components::{GraphNode, GraphEdge, NodeLabel};
use crate::domain::value_objects::{NodeId, GraphId};

/// Represents a subgraph with its own spatial origin
#[derive(Component, Debug, Clone)]
pub struct SubgraphOrigin {
    pub graph_id: GraphId,
    pub base_position: Vec3,
}

/// Marks an entity as belonging to a specific subgraph
#[derive(Component, Debug, Clone)]
pub struct SubgraphMember {
    pub graph_id: GraphId,
}

/// Resource for managing subgraph spatial mappings
#[derive(Resource, Default)]
pub struct SubgraphSpatialMap {
    /// Maps graph IDs to their origin entities
    pub origins: HashMap<GraphId, Entity>,
    /// Maps graph IDs to their current base positions
    pub positions: HashMap<GraphId, Vec3>,
}

/// System to create a new subgraph with an origin
pub fn create_subgraph_origin(
    mut commands: Commands,
    mut spatial_map: ResMut<SubgraphSpatialMap>,
) -> Entity {
    let graph_id = GraphId::new();
    let base_position = Vec3::ZERO;

    // Create an invisible origin entity
    let origin_entity = commands.spawn((
        SubgraphOrigin {
            graph_id,
            base_position,
        },
        Transform::from_translation(base_position),
        GlobalTransform::default(),
        // Make it invisible but still part of the transform hierarchy
        Visibility::Hidden,
    )).id();

    // Update spatial map
    spatial_map.origins.insert(graph_id, origin_entity);
    spatial_map.positions.insert(graph_id, base_position);

    origin_entity
}

/// System to add a node to a subgraph
pub fn add_node_to_subgraph(
    mut commands: Commands,
    spatial_map: Res<SubgraphSpatialMap>,
    graph_id: GraphId,
    node_id: NodeId,
    relative_position: Vec3,
    label: String,
) -> Option<Entity> {
    // Find the origin entity for this subgraph
    let origin_entity = spatial_map.origins.get(&graph_id)?;

    // Create the node as a child of the origin
    let node_entity = commands.spawn((
        GraphNode {
            node_id,
            graph_id,
        },
        NodeLabel {
            text: label,
        },
        SubgraphMember { graph_id },
        Transform::from_translation(relative_position),
        GlobalTransform::default(),
    )).id();

    // Set the parent-child relationship
    commands.entity(*origin_entity).add_child(node_entity);

    Some(node_entity)
}

/// System to move an entire subgraph by updating its origin
pub fn move_subgraph(
    mut spatial_map: ResMut<SubgraphSpatialMap>,
    mut query: Query<&mut Transform, With<SubgraphOrigin>>,
    graph_id: GraphId,
    new_position: Vec3,
) {
    if let Some(origin_entity) = spatial_map.origins.get(&graph_id) {
        if let Ok(mut transform) = query.get_mut(*origin_entity) {
            transform.translation = new_position;
            spatial_map.positions.insert(graph_id, new_position);
        }
    }
}

/// System to get the world position of a node in a subgraph
pub fn get_node_world_position(
    nodes: Query<&GlobalTransform, With<GraphNode>>,
    node_entity: Entity,
) -> Option<Vec3> {
    nodes.get(node_entity).ok().map(|gt| gt.translation())
}

/// System to layout nodes within a subgraph relative to its origin
pub fn layout_subgraph_nodes(
    mut nodes: Query<(&mut Transform, &SubgraphMember), With<GraphNode>>,
    graph_id: GraphId,
    layout_fn: impl Fn(usize) -> Vec3,
) {
    let mut index = 0;
    for (mut transform, member) in nodes.iter_mut() {
        if member.graph_id == graph_id {
            transform.translation = layout_fn(index);
            index += 1;
        }
    }
}

/// Example circular layout function
pub fn circular_layout(radius: f32, count: usize) -> impl Fn(usize) -> Vec3 {
    move |index| {
        let angle = (index as f32) * 2.0 * std::f32::consts::PI / (count as f32);
        Vec3::new(
            radius * angle.cos(),
            0.0,
            radius * angle.sin(),
        )
    }
}

/// System to visualize subgraph boundaries
pub fn visualize_subgraph_boundaries(
    mut gizmos: Gizmos,
    spatial_map: Res<SubgraphSpatialMap>,
    nodes: Query<(&GlobalTransform, &SubgraphMember), With<GraphNode>>,
) {
    // Group nodes by subgraph
    let mut subgraph_nodes: HashMap<GraphId, Vec<Vec3>> = HashMap::new();

    for (transform, member) in nodes.iter() {
        subgraph_nodes
            .entry(member.graph_id)
            .or_default()
            .push(transform.translation());
    }

    // Draw boundaries for each subgraph
    for (graph_id, positions) in subgraph_nodes.iter() {
        if positions.len() < 2 {
            continue;
        }

        // Calculate bounding box
        let mut min = positions[0];
        let mut max = positions[0];

        for pos in positions.iter() {
            min = min.min(*pos);
            max = max.max(*pos);
        }

        // Add padding
        let padding = 1.0;
        min -= Vec3::splat(padding);
        max += Vec3::splat(padding);

        // Draw bounding box
        let color = Color::srgba(0.3, 0.7, 0.9, 0.3);

        // Bottom face
        gizmos.line(Vec3::new(min.x, min.y, min.z), Vec3::new(max.x, min.y, min.z), color);
        gizmos.line(Vec3::new(max.x, min.y, min.z), Vec3::new(max.x, min.y, max.z), color);
        gizmos.line(Vec3::new(max.x, min.y, max.z), Vec3::new(min.x, min.y, max.z), color);
        gizmos.line(Vec3::new(min.x, min.y, max.z), Vec3::new(min.x, min.y, min.z), color);

        // Top face
        gizmos.line(Vec3::new(min.x, max.y, min.z), Vec3::new(max.x, max.y, min.z), color);
        gizmos.line(Vec3::new(max.x, max.y, min.z), Vec3::new(max.x, max.y, max.z), color);
        gizmos.line(Vec3::new(max.x, max.y, max.z), Vec3::new(min.x, max.y, max.z), color);
        gizmos.line(Vec3::new(min.x, max.y, max.z), Vec3::new(min.x, max.y, min.z), color);

        // Vertical edges
        gizmos.line(Vec3::new(min.x, min.y, min.z), Vec3::new(min.x, max.y, min.z), color);
        gizmos.line(Vec3::new(max.x, min.y, min.z), Vec3::new(max.x, max.y, min.z), color);
        gizmos.line(Vec3::new(max.x, min.y, max.z), Vec3::new(max.x, max.y, max.z), color);
        gizmos.line(Vec3::new(min.x, min.y, max.z), Vec3::new(min.x, max.y, max.z), color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_layout() {
        let layout = circular_layout(5.0, 4);

        // Test that nodes are positioned in a circle
        let pos0 = layout(0);
        let pos1 = layout(1);
        let pos2 = layout(2);
        let pos3 = layout(3);

        // All should be at radius 5.0 from origin
        assert!((pos0.length() - 5.0).abs() < 0.001);
        assert!((pos1.length() - 5.0).abs() < 0.001);
        assert!((pos2.length() - 5.0).abs() < 0.001);
        assert!((pos3.length() - 5.0).abs() < 0.001);

        // All should be at y=0
        assert_eq!(pos0.y, 0.0);
        assert_eq!(pos1.y, 0.0);
        assert_eq!(pos2.y, 0.0);
        assert_eq!(pos3.y, 0.0);
    }
}
