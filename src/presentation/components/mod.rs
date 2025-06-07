//! Components for graph visualization

use crate::domain::value_objects::{EdgeId, GraphId, NodeId};
use bevy::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

/// Component marking an entity as a graph node
#[derive(Component)]
pub struct GraphNode {
    pub node_id: NodeId,
    pub graph_id: GraphId,
}

/// Component for graph edges
#[derive(Component)]
pub struct GraphEdge {
    pub edge_id: EdgeId,
    pub graph_id: GraphId,
    pub source: Entity,
    pub target: Entity,
}

/// Component for node labels
#[derive(Component)]
pub struct NodeLabel {
    pub text: String,
}

/// Component marking the graph container
#[derive(Component)]
pub struct GraphContainer {
    pub graph_id: GraphId,
    pub name: String,
}

/// Component for scheduling commands to be executed at a specific time
#[derive(Component)]
pub struct ScheduledCommand {
    pub execute_at: f32, // Time in seconds when this command should execute
    pub command: crate::domain::commands::Command,
}

/// Component for smooth node appearance animation
#[derive(Component)]
pub struct NodeAppearanceAnimation {
    pub start_time: f32,
    pub duration: f32,
    pub start_scale: f32,
    pub target_scale: f32,
}

/// Component for smooth edge drawing animation
#[derive(Component)]
pub struct EdgeDrawAnimation {
    pub start_time: f32,
    pub duration: f32,
    pub progress: f32,
}

/// Component for force-directed layout physics
#[derive(Component)]
pub struct ForceNode {
    pub velocity: bevy::prelude::Vec3,
    pub mass: f32,
    pub charge: f32, // For repulsion between nodes
}

/// Component marking nodes that should participate in force-directed layout
#[derive(Component)]
pub struct ForceLayoutParticipant;

/// Resource for force-directed layout parameters
#[derive(bevy::prelude::Resource)]
pub struct ForceLayoutSettings {
    pub spring_strength: f32,    // Edge attraction force
    pub spring_length: f32,      // Ideal edge length
    pub repulsion_strength: f32, // Node repulsion force
    pub damping: f32,            // Velocity damping
    pub min_velocity: f32,       // Threshold to stop simulation
    pub center_force: f32,       // Pull towards center
}

impl Default for ForceLayoutSettings {
    fn default() -> Self {
        Self {
            spring_strength: 0.15,    // Increased from 0.1 to pull nodes together more
            spring_length: 2.5,       // Decreased from 3.3 to make edges shorter
            repulsion_strength: 40.0, // Decreased from 55.0 to reduce node repulsion
            damping: 0.9,
            min_velocity: 0.01,
            center_force: 0.02, // Increased from 0.01 to pull nodes toward center
        }
    }
}

/// Component for recording events with timestamps
#[derive(Component)]
pub struct EventRecorder {
    pub events: Vec<RecordedEvent>,
    pub recording_start_time: f32,
}

/// A recorded event with timing information
#[derive(Clone)]
pub struct RecordedEvent {
    pub event: crate::domain::events::DomainEvent,
    pub timestamp: f32, // Relative to recording start
}

/// Component for replaying recorded events
#[derive(Component)]
pub struct EventReplayer {
    pub events: Vec<RecordedEvent>,
    pub replay_start_time: f32,
    pub current_index: usize,
    pub speed_multiplier: f32, // 1.0 = normal speed, 2.0 = double speed, etc.
}

/// Component for animation progress
#[derive(Component)]
pub struct AnimationProgress(pub f32);

/// Simple orbit camera controller component
#[derive(Component, Debug)]
pub struct OrbitCamera {
    /// Focus point the camera orbits around
    pub focus: Vec3,
    /// Distance from the focus point
    pub distance: f32,
    /// Rotation around the Y axis (yaw)
    pub yaw: f32,
    /// Rotation around the X axis (pitch)
    pub pitch: f32,
    /// Mouse sensitivity for rotation
    pub sensitivity: f32,
    /// Zoom speed
    pub zoom_speed: f32,
    /// Minimum zoom distance
    pub min_distance: f32,
    /// Maximum zoom distance
    pub max_distance: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::new(100.0, 0.0, 0.0), // Focus on the center of imported nodes
            distance: 150.0,
            yaw: 0.0,
            pitch: -0.5,
            sensitivity: 0.005,
            zoom_speed: 2.0,
            min_distance: 10.0,
            max_distance: 500.0,
        }
    }
}

/// Component for subgraph regions
#[derive(Component, Debug, Clone)]
pub struct SubgraphRegion {
    pub subgraph_id: SubgraphId,
    pub name: String,
    pub color: Color,
    pub nodes: HashSet<NodeId>,
    pub boundary_type: BoundaryType,
}

/// Unique identifier for subgraphs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct SubgraphId(pub Uuid);

impl SubgraphId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Types of visual boundaries for subgraphs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundaryType {
    ConvexHull,
    BoundingBox,
    Circle,
    Voronoi,
}

/// Component for nodes that belong to a subgraph
#[derive(Component, Debug, Clone)]
pub struct SubgraphMember {
    pub subgraph_id: SubgraphId,
}

/// Visual boundary mesh for subgraph
#[derive(Component)]
pub struct SubgraphBoundary {
    pub subgraph_id: SubgraphId,
    pub mesh_needs_update: bool,
}

/// Component marking an entity as selected
#[derive(Component, Debug)]
pub struct Selected;

/// Component for Voronoi cell in conceptual space
#[derive(Component, Debug, Clone)]
pub struct VoronoiCell {
    pub subgraph_id: SubgraphId,
    pub prototype: Vec3,  // Center point (prototype) of this quality dimension
    pub vertices: Vec<Vec3>,  // Vertices of the Voronoi cell
    pub neighbors: HashSet<SubgraphId>,  // Adjacent cells
}

/// Component for conceptual space partitioning
#[derive(Component, Debug)]
pub struct ConceptualSpacePartition {
    pub cells: Vec<VoronoiCell>,
    pub bounds: (Vec3, Vec3),  // Min and max bounds of the space
}

/// Component for quality dimension in conceptual space
#[derive(Component, Debug, Clone)]
pub struct QualityDimension {
    pub subgraph_id: SubgraphId,
    pub name: String,
    pub prototype: Vec3,  // Prototype point in conceptual space
    pub weight: f32,  // Importance/weight of this dimension
    pub metric: DistanceMetric,
}

/// Distance metrics for conceptual space
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistanceMetric {
    Euclidean,
    Manhattan,
    WeightedEuclidean { weights: [f32; 3] },
    Conceptual,  // Custom metric based on semantic similarity
}

/// Component for nodes positioned in conceptual space
#[derive(Component, Debug, Clone)]
pub struct ConceptualPosition {
    pub coordinates: Vec3,  // Position in quality dimensions
    pub cell_id: Option<SubgraphId>,  // Which Voronoi cell it belongs to
    pub distance_to_prototype: f32,  // Distance to nearest prototype
}

/// Resource for Voronoi tessellation settings
#[derive(Resource, Debug)]
pub struct VoronoiSettings {
    pub update_frequency: f32,  // How often to recalculate (seconds)
    pub smoothing_factor: f32,  // For Lloyd's relaxation
    pub min_cell_size: f32,  // Minimum size for a cell
    pub boundary_padding: f32,  // Padding around the space
    pub visualization_height: f32,  // Y-offset for 2D visualization
}

impl Default for VoronoiSettings {
    fn default() -> Self {
        Self {
            update_frequency: 0.5,
            smoothing_factor: 0.3,
            min_cell_size: 10.0,
            boundary_padding: 20.0,
            visualization_height: 0.1,  // Slightly above ground plane
        }
    }
}
