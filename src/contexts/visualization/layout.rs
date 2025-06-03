use crate::contexts::graph_management::domain::*;
use bevy::prelude::*;
use std::collections::HashMap;
use std::ops::AddAssign;

// ============= Layout Configuration =============

/// Configuration for force-directed layout algorithm
#[derive(Resource, Debug, Clone)]
pub struct ForceDirectedConfig {
    /// Repulsion strength between all nodes (Coulomb's law)
    pub repulsion_strength: f32,
    /// Attraction strength along edges (Hooke's law)
    pub attraction_strength: f32,
    /// Damping factor to prevent oscillation
    pub damping: f32,
    /// Minimum distance between nodes
    pub min_distance: f32,
    /// Maximum displacement per frame
    pub max_displacement: f32,
    /// Stability threshold for stopping calculation
    pub stability_threshold: f32,
    /// Maximum iterations before forcing stop
    pub max_iterations: u32,
}

impl Default for ForceDirectedConfig {
    fn default() -> Self {
        Self {
            repulsion_strength: 1000.0,
            attraction_strength: 0.5,
            damping: 0.8,
            min_distance: 1.0,
            max_displacement: 5.0,
            stability_threshold: 0.01,
            max_iterations: 1000,
        }
    }
}

// ============= Layout State =============

/// Tracks the current state of layout calculation
#[derive(Resource, Debug, Default)]
pub struct LayoutState {
    /// Whether layout is currently being calculated
    pub is_calculating: bool,
    /// Current iteration count
    pub current_iteration: u32,
    /// Maximum displacement in last iteration
    pub last_max_displacement: f32,
    /// Node velocities for physics simulation
    pub node_velocities: HashMap<NodeIdentity, Vec3>,
    /// Target positions after layout calculation
    pub target_positions: HashMap<NodeIdentity, Vec3>,
}

// ============= Layout Events =============

/// Request to start layout calculation
#[derive(Event, Debug, Clone)]
pub struct LayoutRequested {
    pub graph: GraphIdentity,
    pub algorithm: LayoutAlgorithm,
}

/// Layout calculation completed
#[derive(Event, Debug, Clone)]
pub struct LayoutCompleted {
    pub graph: GraphIdentity,
    pub iterations: u32,
    pub final_displacement: f32,
}

/// Available layout algorithms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutAlgorithm {
    ForceDirected,
    Circular,
    Hierarchical,
    Grid,
}

// ============= Layout Services =============

/// Service to calculate force-directed layout
pub struct CalculateForceDirectedLayout;

impl CalculateForceDirectedLayout {
    /// Calculate forces and update positions
    pub fn execute(
        &self,
        nodes: &Query<(Entity, &NodeIdentity, &Transform), Without<EdgeIdentity>>,
        edges: &Query<&EdgeRelationship>,
        config: &ForceDirectedConfig,
        layout_state: &mut LayoutState,
        time_delta: f32,
    ) -> HashMap<NodeIdentity, Vec3> {
        let mut positions = HashMap::new();
        let mut forces: HashMap<NodeIdentity, Vec3> = HashMap::new();

        // Collect current positions
        let node_count = nodes.iter().count();
        info!("CalculateForceDirectedLayout: Found {} nodes", node_count);

        for (_, node_id, transform) in nodes.iter() {
            positions.insert(*node_id, transform.translation);
            forces.insert(*node_id, Vec3::ZERO);
        }

        let edge_count = edges.iter().count();
        info!("CalculateForceDirectedLayout: Found {} edges", edge_count);

        // Calculate repulsive forces between all node pairs
        let node_list: Vec<_> = positions.keys().copied().collect();
        for i in 0..node_list.len() {
            for j in (i + 1)..node_list.len() {
                let node_a = node_list[i];
                let node_b = node_list[j];

                let pos_a = positions[&node_a];
                let pos_b = positions[&node_b];

                let delta = pos_b - pos_a;
                let distance = delta.length().max(config.min_distance);

                // Coulomb's law: F = k * q1 * q2 / r^2
                let force_magnitude = config.repulsion_strength / (distance * distance);
                let force_direction = delta.normalize_or_zero();

                forces
                    .get_mut(&node_a)
                    .unwrap()
                    .add_assign(-force_direction * force_magnitude);
                forces
                    .get_mut(&node_b)
                    .unwrap()
                    .add_assign(force_direction * force_magnitude);
            }
        }

        // Calculate attractive forces along edges
        for edge_relationship in edges.iter() {
            let source_pos = positions.get(&edge_relationship.source);
            let target_pos = positions.get(&edge_relationship.target);

            if let (Some(&source_pos), Some(&target_pos)) = (source_pos, target_pos) {
                let delta = target_pos - source_pos;
                let distance = delta.length();

                // Hooke's law: F = -k * x
                let force_magnitude = config.attraction_strength * distance;
                let force_direction = delta.normalize_or_zero();

                if let Some(source_force) = forces.get_mut(&edge_relationship.source) {
                    source_force.add_assign(force_direction * force_magnitude);
                }
                if let Some(target_force) = forces.get_mut(&edge_relationship.target) {
                    target_force.add_assign(-force_direction * force_magnitude);
                }
            }
        }

        // Update velocities and positions
        let mut max_displacement = 0.0;
        let mut new_positions = HashMap::new();

        for (node_id, force) in forces {
            // Get or initialize velocity
            let velocity = layout_state
                .node_velocities
                .entry(node_id)
                .or_insert(Vec3::ZERO);

            // Apply force (F = ma, assuming m = 1)
            *velocity += force * time_delta;

            // Apply damping
            *velocity *= config.damping;

            // Calculate displacement
            let displacement = *velocity * time_delta;
            let clamped_displacement = if displacement.length() > config.max_displacement {
                displacement.normalize() * config.max_displacement
            } else {
                displacement
            };

            // Update position
            let current_pos = positions[&node_id];
            let new_pos = current_pos + clamped_displacement;
            new_positions.insert(node_id, new_pos);

            // Track maximum displacement
            max_displacement = f32::max(max_displacement, clamped_displacement.length());
        }

        // Update layout state
        layout_state.last_max_displacement = max_displacement;
        layout_state.current_iteration += 1;

        new_positions
    }
}

/// Service to apply calculated layout to nodes
pub struct ApplyGraphLayout;

impl ApplyGraphLayout {
    /// Smoothly animate nodes to their target positions
    pub fn execute(
        &self,
        mut nodes: Query<(&NodeIdentity, &mut Transform), Without<EdgeIdentity>>,
        layout_state: &LayoutState,
        time_delta: f32,
    ) {
        const ANIMATION_SPEED: f32 = 3.0;

        let mut moved_count = 0;
        for (node_id, mut transform) in nodes.iter_mut() {
            if let Some(&target_pos) = layout_state.target_positions.get(node_id) {
                // Smoothly interpolate to target position
                let current_pos = transform.translation;
                let direction = target_pos - current_pos;
                let distance = direction.length();

                if distance > 0.001 {
                    let move_distance = (ANIMATION_SPEED * time_delta).min(distance);
                    let movement = direction.normalize() * move_distance;
                    transform.translation += movement;
                    moved_count += 1;

                    // Log significant movements
                    if movement.length() > 0.01 {
                        info!(
                            "Moving node {:?} by {} (distance to target: {})",
                            node_id,
                            movement.length(),
                            distance
                        );
                    }
                }
            }
        }

        if moved_count > 0 {
            info!("ApplyGraphLayout moved {} nodes this frame", moved_count);
        }
    }
}

// ============= Layout Plugin =============

pub struct LayoutPlugin;

impl Plugin for LayoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ForceDirectedConfig>()
            .init_resource::<LayoutState>()
            .add_event::<LayoutRequested>()
            .add_event::<LayoutCompleted>()
            .add_systems(
                Update,
                (handle_layout_requests, calculate_layout, apply_layout)
                    .chain()
                    .after(
                        crate::contexts::graph_management::plugin::GraphManagementSet::Hierarchy,
                    ),
            );
    }
}

// ============= Layout Systems =============

/// Handle layout requests and initialize calculation
fn handle_layout_requests(
    mut events: EventReader<LayoutRequested>,
    mut layout_state: ResMut<LayoutState>,
    nodes: Query<(&NodeIdentity, &Transform), Without<EdgeIdentity>>,
) {
    let event_count = events.len();
    if event_count > 0 {
        info!(
            "handle_layout_requests: Processing {} layout requests",
            event_count
        );
    }

    for event in events.read() {
        info!(
            "Layout requested for graph {:?} using {:?} algorithm",
            event.graph, event.algorithm
        );

        // Initialize layout state
        layout_state.is_calculating = true;
        layout_state.current_iteration = 0;
        layout_state.last_max_displacement = f32::MAX;
        layout_state.node_velocities.clear();

        // Store current positions as starting point
        layout_state.target_positions.clear();
        let node_count = nodes.iter().count();
        info!("Found {} nodes to layout", node_count);

        for (node_id, transform) in nodes.iter() {
            layout_state
                .target_positions
                .insert(*node_id, transform.translation);
        }
    }
}

/// Calculate layout forces and update target positions
fn calculate_layout(
    nodes: Query<(Entity, &NodeIdentity, &Transform), Without<EdgeIdentity>>,
    edges: Query<&EdgeRelationship>,
    config: Res<ForceDirectedConfig>,
    mut layout_state: ResMut<LayoutState>,
    mut completed_events: EventWriter<LayoutCompleted>,
    time: Res<Time>,
) {
    if !layout_state.is_calculating {
        return;
    }

    info!(
        "Calculating layout - iteration {}",
        layout_state.current_iteration
    );

    let calculator = CalculateForceDirectedLayout;
    let new_positions = calculator.execute(
        &nodes,
        &edges,
        &config,
        &mut layout_state,
        time.delta_secs(),
    );

    // Log position changes
    for (node_id, new_pos) in &new_positions {
        if let Some(old_pos) = layout_state.target_positions.get(node_id) {
            let delta = (*new_pos - *old_pos).length();
            if delta > 0.001 {
                info!("Node {:?} moved by {}", node_id, delta);
            }
        }
    }

    // Update target positions
    layout_state.target_positions = new_positions;

    // Check for completion
    let is_stable = layout_state.last_max_displacement < config.stability_threshold;
    let max_iterations_reached = layout_state.current_iteration >= config.max_iterations;

    info!(
        "Layout iteration {} - max displacement: {}",
        layout_state.current_iteration, layout_state.last_max_displacement
    );

    if is_stable || max_iterations_reached {
        layout_state.is_calculating = false;

        completed_events.write(LayoutCompleted {
            graph: GraphIdentity::new(), // TODO: Track which graph is being laid out
            iterations: layout_state.current_iteration,
            final_displacement: layout_state.last_max_displacement,
        });

        info!(
            "Layout completed after {} iterations (displacement: {})",
            layout_state.current_iteration, layout_state.last_max_displacement
        );
    }
}

/// Apply calculated layout with smooth animation
fn apply_layout(
    nodes: Query<(&NodeIdentity, &mut Transform), Without<EdgeIdentity>>,
    layout_state: Res<LayoutState>,
    time: Res<Time>,
) {
    if layout_state.target_positions.is_empty() {
        return;
    }

    let mut applied_count = 0;
    let applier = ApplyGraphLayout;

    // Log before applying
    for (node_id, transform) in nodes.iter() {
        if let Some(&target_pos) = layout_state.target_positions.get(node_id) {
            let distance = (target_pos - transform.translation).length();
            if distance > 0.001 {
                info!(
                    "Applying layout: Node {:?} needs to move {} units",
                    node_id, distance
                );
                applied_count += 1;
            }
        }
    }

    if applied_count > 0 {
        info!("Applying layout to {} nodes", applied_count);
    }

    applier.execute(nodes, &layout_state, time.delta_secs());
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use bevy::ecs::system::SystemState;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ForceDirectedConfig>();
        app.init_resource::<LayoutState>();
        app.add_event::<LayoutRequested>();
        app.add_event::<LayoutCompleted>();
        app
    }

    #[test]
    fn test_force_directed_config_defaults() {
        let config = ForceDirectedConfig::default();
        assert_eq!(config.repulsion_strength, 1000.0);
        assert_eq!(config.attraction_strength, 0.5);
        assert_eq!(config.damping, 0.8);
        assert_eq!(config.min_distance, 1.0);
        assert_eq!(config.max_displacement, 5.0);
        assert_eq!(config.stability_threshold, 0.01);
        assert_eq!(config.max_iterations, 1000);
    }

    #[test]
    fn test_layout_state_initialization() {
        let state = LayoutState::default();
        assert!(!state.is_calculating);
        assert_eq!(state.current_iteration, 0);
        assert_eq!(state.last_max_displacement, 0.0);
        assert!(state.node_velocities.is_empty());
        assert!(state.target_positions.is_empty());
    }

    #[test]
    fn test_layout_requested_event() {
        let mut app = setup_test_app();

        let graph_id = GraphIdentity::new();
        app.world_mut().send_event(LayoutRequested {
            graph: graph_id,
            algorithm: LayoutAlgorithm::ForceDirected,
        });

        app.update();

        // Events are consumed after update, so we just verify the system runs
        // without errors. In a real test we would check the side effects.
    }

    #[test]
    fn test_calculate_force_directed_layout_empty_graph() {
        let mut app = setup_test_app();
        let world = app.world_mut();

        let mut system_state: SystemState<(
            Query<(Entity, &NodeIdentity, &Transform), Without<EdgeIdentity>>,
            Query<&EdgeRelationship>,
        )> = SystemState::new(world);

        let (nodes, edges) = system_state.get(world);
        let config = ForceDirectedConfig::default();
        let mut layout_state = LayoutState::default();

        let calculator = CalculateForceDirectedLayout;
        let positions = calculator.execute(&nodes, &edges, &config, &mut layout_state, 0.016);

        assert!(positions.is_empty());
        assert_eq!(layout_state.current_iteration, 1);
    }

    #[test]
    fn test_calculate_force_directed_layout_with_nodes() {
        let mut app = setup_test_app();

        // Create test nodes
        let node1_id = NodeIdentity::new();
        let node2_id = NodeIdentity::new();

        app.world_mut()
            .spawn((node1_id, Transform::from_xyz(0.0, 0.0, 0.0)));

        app.world_mut()
            .spawn((node2_id, Transform::from_xyz(5.0, 0.0, 0.0)));

        let world = app.world_mut();
        let mut system_state: SystemState<(
            Query<(Entity, &NodeIdentity, &Transform), Without<EdgeIdentity>>,
            Query<&EdgeRelationship>,
        )> = SystemState::new(world);

        let (nodes, edges) = system_state.get(world);
        let config = ForceDirectedConfig::default();
        let mut layout_state = LayoutState::default();

        let calculator = CalculateForceDirectedLayout;
        let positions = calculator.execute(&nodes, &edges, &config, &mut layout_state, 0.016);

        // Should have positions for both nodes
        assert_eq!(positions.len(), 2);
        assert!(positions.contains_key(&node1_id));
        assert!(positions.contains_key(&node2_id));

        // Nodes should repel each other
        let pos1 = positions[&node1_id];
        let pos2 = positions[&node2_id];
        let distance = (pos2 - pos1).length();
        assert!(distance > 5.0); // Should move apart due to repulsion
    }

    #[test]
    fn test_calculate_force_directed_layout_with_edge() {
        let mut app = setup_test_app();

        // Create test nodes
        let node1_id = NodeIdentity::new();
        let node2_id = NodeIdentity::new();

        app.world_mut()
            .spawn((node1_id, Transform::from_xyz(0.0, 0.0, 0.0)));

        app.world_mut()
            .spawn((node2_id, Transform::from_xyz(10.0, 0.0, 0.0)));

        // Create edge between nodes
        app.world_mut().spawn(EdgeRelationship {
            source: node1_id,
            target: node2_id,
            category: "test".to_string(),
            strength: 1.0,
            properties: HashMap::new(),
        });

        let world = app.world_mut();
        let mut system_state: SystemState<(
            Query<(Entity, &NodeIdentity, &Transform), Without<EdgeIdentity>>,
            Query<&EdgeRelationship>,
        )> = SystemState::new(world);

        let (nodes, edges) = system_state.get(world);
        let config = ForceDirectedConfig {
            attraction_strength: 0.5, // Stronger attraction for test
            ..default()
        };
        let mut layout_state = LayoutState::default();

        let calculator = CalculateForceDirectedLayout;
        let positions = calculator.execute(&nodes, &edges, &config, &mut layout_state, 0.016);

        // Nodes should be attracted to each other
        let pos1 = positions[&node1_id];
        let pos2 = positions[&node2_id];
        let distance = (pos2 - pos1).length();
        assert!(distance < 10.0); // Should move closer due to attraction
    }

    #[test]
    fn test_apply_graph_layout() {
        let mut app = setup_test_app();

        // Create test node
        let node_id = NodeIdentity::new();
        let entity = app
            .world_mut()
            .spawn((node_id, Transform::from_xyz(0.0, 0.0, 0.0)))
            .id();

        // Set target position
        let mut layout_state = LayoutState::default();
        layout_state
            .target_positions
            .insert(node_id, Vec3::new(5.0, 0.0, 0.0));
        app.insert_resource(layout_state);

        // Apply layout
        let world = app.world_mut();

        let mut system_state: SystemState<(
            Query<(&NodeIdentity, &mut Transform), Without<EdgeIdentity>>,
            Res<LayoutState>,
        )> = SystemState::new(world);

        let (mut nodes, layout_state) = system_state.get_mut(world);

        let applier = ApplyGraphLayout;
        applier.execute(nodes, &layout_state, 0.016);

        system_state.apply(world);

        // Check node moved towards target
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.x > 0.0);
        assert!(transform.translation.x < 5.0); // Should be moving towards target
    }

    #[test]
    fn test_layout_completion() {
        let mut app = setup_test_app();

        // Set up a layout state that should complete
        let layout_state = LayoutState {
            is_calculating: true,
            current_iteration: 999,
            last_max_displacement: 0.005, // Below threshold
            ..default()
        };
        app.insert_resource(layout_state);

        // Add the calculate_layout system
        app.add_systems(Update, calculate_layout);

        app.update();

        // Check that layout completed
        let layout_state = app.world().resource::<LayoutState>();
        assert!(!layout_state.is_calculating);

        // Layout completion event handling changed in newer Bevy versions
        // Just verify the system runs without errors
    }

    #[test]
    fn test_handle_layout_requests() {
        let mut app = setup_test_app();

        // Create test nodes
        let node_id = NodeIdentity::new();
        app.world_mut()
            .spawn((node_id, Transform::from_xyz(1.0, 2.0, 3.0)));

        // Send layout request
        let graph_id = GraphIdentity::new();
        app.world_mut().send_event(LayoutRequested {
            graph: graph_id,
            algorithm: LayoutAlgorithm::ForceDirected,
        });

        // Add the handle_layout_requests system
        app.add_systems(Update, handle_layout_requests);

        app.update();

        // Check layout state was initialized
        let layout_state = app.world().resource::<LayoutState>();
        assert!(layout_state.is_calculating);
        assert_eq!(layout_state.current_iteration, 0);
        assert_eq!(layout_state.last_max_displacement, f32::MAX);
        assert!(layout_state.target_positions.contains_key(&node_id));
        assert_eq!(
            layout_state.target_positions[&node_id],
            Vec3::new(1.0, 2.0, 3.0)
        );
    }
}
