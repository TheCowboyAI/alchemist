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
        // transform.translation.y = 0.0;  // REMOVED: Allow Y positioning for grid layouts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{NodeId, GraphId};
    use std::time::Duration;

    #[test]
    fn test_force_layout_settings_default() {
        let settings = ForceLayoutSettings::default();
        assert_eq!(settings.repulsion_strength, 50.0);
        assert_eq!(settings.spring_strength, 0.1);
        assert_eq!(settings.spring_length, 3.0);
        assert_eq!(settings.damping, 0.9);
        assert!(settings.enabled);
    }

    #[test]
    fn test_force_node_default() {
        let force_node = ForceNode::default();
        assert_eq!(force_node.velocity, Vec3::ZERO);
        assert_eq!(force_node.mass, 1.0);
    }

    #[test]
    fn test_force_layout_disabled() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add disabled settings
        app.insert_resource(ForceLayoutSettings {
            enabled: false,
            ..Default::default()
        });

        // Add time resource
        app.insert_resource(Time::<()>::default());

        // Add test entities
        let node1 = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            ForceNode::default(),
        )).id();

        let node2 = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_xyz(1.0, 0.0, 0.0),
            ForceNode::default(),
        )).id();

        // Add system
        app.add_systems(Update, apply_force_directed_layout);

        // Run update
        app.update();

        // Positions should not change when disabled
        let transform1 = app.world().get::<Transform>(node1).unwrap();
        let transform2 = app.world().get::<Transform>(node2).unwrap();

        assert_eq!(transform1.translation, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(transform2.translation, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_repulsion_forces() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add settings
        app.insert_resource(ForceLayoutSettings::default());

        // Add time with fixed delta
        app.insert_resource(Time::<()>::default());

        // Add two nodes close together
        let node1 = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            ForceNode::default(),
        )).id();

        let node2 = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_xyz(0.5, 0.0, 0.0),
            ForceNode::default(),
        )).id();

        // Add system
        app.add_systems(Update, apply_force_directed_layout);

        // Advance time and update the app to ensure time delta is available
        app.update(); // First update to initialize systems
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(0.016)); // ~60 FPS

        // Run update with time delta
        app.update();

        // Nodes should repel each other
        let transform1 = app.world().get::<Transform>(node1).unwrap();
        let transform2 = app.world().get::<Transform>(node2).unwrap();

        // Node 1 should move left (negative X)
        assert!(transform1.translation.x < 0.0, "Node 1 should move left, but is at {}", transform1.translation.x);
        // Node 2 should move right (positive X)
        assert!(transform2.translation.x > 0.5, "Node 2 should move right, but is at {}", transform2.translation.x);
        // Y should remain 0
        assert_eq!(transform1.translation.y, 0.0);
        assert_eq!(transform2.translation.y, 0.0);
    }

    #[test]
    fn test_spring_forces() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add settings with strong spring force
        app.insert_resource(ForceLayoutSettings {
            repulsion_strength: 0.0, // Disable repulsion for this test
            spring_strength: 10.0,    // Increased from 1.0 for more noticeable effect
            spring_length: 2.0,
            damping: 0.9,
            enabled: true,
        });

        // Add time with fixed delta
        app.insert_resource(Time::<()>::default());

        // Add two connected nodes far apart
        let node1 = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            ForceNode::default(),
        )).id();

        let node2 = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_xyz(5.0, 0.0, 0.0),
            ForceNode::default(),
        )).id();

        // Add edge between nodes
        app.world_mut().spawn(GraphEdge {
            edge_id: crate::domain::value_objects::EdgeId::new(),
            graph_id: GraphId::new(),
            source: node1,
            target: node2,
        });

        // Add system
        app.add_systems(Update, apply_force_directed_layout);

        // Advance time and update
        app.update(); // First update to initialize
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(0.016));

        // Run update with time delta
        app.update();

        // Check velocities instead of positions for more reliable test
        let force1 = app.world().get::<ForceNode>(node1).unwrap();
        let force2 = app.world().get::<ForceNode>(node2).unwrap();

        // Node 1 should have positive X velocity (moving right toward node 2)
        assert!(force1.velocity.x > 0.0, "Node 1 should have positive X velocity, but has {}", force1.velocity.x);
        // Node 2 should have negative X velocity (moving left toward node 1)
        assert!(force2.velocity.x < 0.0, "Node 2 should have negative X velocity, but has {}", force2.velocity.x);

        // Velocities should be equal in magnitude but opposite in direction
        assert!((force1.velocity.x + force2.velocity.x).abs() < 0.0001,
            "Velocities should be equal and opposite");
    }

    #[test]
    fn test_damping_effect() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add settings
        app.insert_resource(ForceLayoutSettings::default());

        // Add time
        app.insert_resource(Time::<()>::default());
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(0.016));

        // Add node with initial velocity
        let node = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            ForceNode {
                velocity: Vec3::new(10.0, 0.0, 0.0),
                mass: 1.0,
            },
        )).id();

        // Add system
        app.add_systems(Update, apply_force_directed_layout);

        // Run update
        app.update();

        // Velocity should be reduced by damping
        let force_node = app.world().get::<ForceNode>(node).unwrap();
        assert!(force_node.velocity.x < 10.0);
        assert!(force_node.velocity.x > 0.0); // Still moving but slower
    }

    #[test]
    fn test_mass_effect() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add settings
        app.insert_resource(ForceLayoutSettings::default());

        // Add time
        app.insert_resource(Time::<()>::default());

        // Add two nodes with different masses at the same position
        // They will repel each other, and the lighter one should move more
        let light_node = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            ForceNode {
                velocity: Vec3::ZERO,
                mass: 1.0,
            },
        )).id();

        let heavy_node = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_xyz(0.1, 0.0, 0.0), // Slightly offset to create repulsion
            ForceNode {
                velocity: Vec3::ZERO,
                mass: 10.0,
            },
        )).id();

        // Add system
        app.add_systems(Update, apply_force_directed_layout);

        // Advance time and update
        app.update(); // First update to initialize
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(0.016));

        // Run update with time delta
        app.update();

        // Light node should move more than heavy node
        let light_transform = app.world().get::<Transform>(light_node).unwrap();
        let heavy_transform = app.world().get::<Transform>(heavy_node).unwrap();

        // Both should have moved due to repulsion
        let light_displacement = light_transform.translation.x.abs();
        let heavy_displacement = (heavy_transform.translation.x - 0.1).abs();

        // Light node should have moved more
        assert!(light_displacement > heavy_displacement,
            "Light node displacement ({}) should be greater than heavy node displacement ({})",
            light_displacement, heavy_displacement);
    }

    #[test]
    fn test_y_plane_constraint() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add settings
        app.insert_resource(ForceLayoutSettings::default());

        // Add time
        app.insert_resource(Time::<()>::default());

        // Add node with Y displacement
        let node = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_xyz(0.0, 5.0, 0.0), // Y = 5.0
            ForceNode {
                velocity: Vec3::new(0.0, 10.0, 0.0), // Y velocity
                mass: 1.0,
            },
        )).id();

        // Add system
        app.add_systems(Update, apply_force_directed_layout);

        // Advance time and update
        app.update(); // First update to initialize
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(0.016));

        // Run update with time delta
        app.update();

        // Y should NOT be reset to 0 anymore - it should have moved based on velocity
        let transform = app.world().get::<Transform>(node).unwrap();
        assert_ne!(transform.translation.y, 0.0, "Y position should not be constrained to 0");

        // With initial Y=5.0 and velocity Y=10.0, after one frame with damping (0.9) and delta (0.016)
        // Expected: 5.0 + (10.0 * 0.9 * 0.016) = 5.0 + 0.144 = 5.144
        assert!(transform.translation.y > 5.0, "Y position should have increased due to velocity");

        // Also check that velocity was damped
        let force_node = app.world().get::<ForceNode>(node).unwrap();
        assert!(force_node.velocity.y < 10.0 && force_node.velocity.y > 0.0,
            "Y velocity should be damped but still positive");
    }
}
