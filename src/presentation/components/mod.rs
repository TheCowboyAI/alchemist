//! Components for graph visualization

use crate::domain::value_objects::{EdgeId, GraphId, NodeId};
use bevy::prelude::*;

/// Component marking an entity as a graph node
#[derive(Component)]
pub struct GraphNode {
    pub node_id: NodeId,
    pub graph_id: GraphId,
}

/// Component marking an entity as a graph edge
#[derive(Component)]
pub struct GraphEdge {
    pub edge_id: EdgeId,
    pub graph_id: GraphId,
    pub source: NodeId,
    pub target: NodeId,
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
