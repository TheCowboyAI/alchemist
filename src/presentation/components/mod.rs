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
    pub execute_at: f32,  // Time in seconds when this command should execute
    pub command: crate::domain::commands::Command,
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
    pub timestamp: f32,  // Relative to recording start
}

/// Component for replaying recorded events
#[derive(Component)]
pub struct EventReplayer {
    pub events: Vec<RecordedEvent>,
    pub replay_start_time: f32,
    pub current_index: usize,
    pub speed_multiplier: f32,  // 1.0 = normal speed, 2.0 = double speed, etc.
}
