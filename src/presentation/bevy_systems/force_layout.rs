//! Force-directed layout system for graph visualization

use bevy::prelude::*;
use crate::presentation::components::{GraphNode, GraphEdge};

/// Settings for force-directed layout
#[derive(Resource)]
pub struct ForceLayoutSettings {
    pub repulsion_strength: f32,
    pub spring_strength: f32,
    pub spring_length: f32,
    pub damping: f32,
    pub enabled: bool,
}

impl Default for ForceLayoutSettings {
    fn default() -> Self {
        Self {
            repulsion_strength: 50.0,
            spring_strength: 0.1,
            spring_length: 3.0,
            damping: 0.9,
            enabled: true,
        }
    }
}

/// Component for nodes participating in force layout
#[derive(Component)]
pub struct ForceNode {
    pub velocity: Vec3,
    pub mass: f32,
}

impl Default for ForceNode {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            mass: 1.0,
        }
    }
}

/// Apply force-directed layout to graph nodes
pub fn apply_force_directed_layout(
    mut nodes: Query<(Entity, &mut Transform, &mut ForceNode), With<GraphNode>>,
    edges: Query<&GraphEdge>,
    settings: Res<ForceLayoutSettings>,
    time: Res<Time>,
) {
    if !settings.enabled {
        return;
    }

    let delta = time.delta_secs();

    // Collect node positions for force calculations
    let mut node_positions = Vec::new();
    for (entity, transform, _) in nodes.iter() {
        node_positions.push((entity, transform.translation));
    }

    // Apply forces to each node
    for (entity, mut transform, mut force_node) in nodes.iter_mut() {
        let mut total_force = Vec3::ZERO;

        // Repulsion forces from all other nodes
        for (other_entity, other_pos) in &node_positions {
            if *other_entity != entity {
                let diff = transform.translation - *other_pos;
                let distance = diff.length().max(0.1);
                let repulsion = diff.normalize() * (settings.repulsion_strength / (distance * distance));
                total_force += repulsion;
            }
        }

        // Spring forces from connected edges
        for edge in edges.iter() {
            if edge.source == entity || edge.target == entity {
                let other_entity = if edge.source == entity {
                    edge.target
                } else {
                    edge.source
                };

                // Find other node position
                if let Some((_, other_pos)) = node_positions.iter().find(|(e, _)| *e == other_entity) {
                    let diff = *other_pos - transform.translation;
                    let distance = diff.length();
                    if distance > 0.0 {
                        let displacement = distance - settings.spring_length;
                        let spring_force = diff.normalize() * (displacement * settings.spring_strength);
                        total_force += spring_force;
                    }
                }
            }
        }

        // Apply forces
        let mass = force_node.mass;
        force_node.velocity += total_force * delta / mass;
        force_node.velocity *= settings.damping;
        transform.translation += force_node.velocity * delta;

        // Keep nodes on the same Y plane
        transform.translation.y = 0.0;
    }
}
