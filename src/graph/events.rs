//! Domain events for graph operations
//!
//! Following event-driven architecture principles, all graph modifications
//! are represented as events that can be stored, replayed, and processed.

use bevy::prelude::*;
use crate::graph::components::{GraphId, GraphMetadata, NodeId, EdgeId};
use std::collections::HashMap;

/// Event fired when a new graph is created
#[derive(Debug, Clone, Event)]
pub struct GraphCreatedEvent {
    pub graph_id: GraphId,
    pub metadata: GraphMetadata,
}

/// Event fired when graph metadata is updated
#[derive(Debug, Clone, Event)]
pub struct GraphMetadataUpdatedEvent {
    pub graph_id: GraphId,
    pub metadata: GraphMetadata,
    pub previous_version: u64,
}

/// Event fired when a graph is deleted
#[derive(Debug, Clone, Event)]
pub struct GraphDeletedEvent {
    pub graph_id: GraphId,
}

/// Event fired when a node is added to a graph
#[derive(Debug, Clone, Event)]
pub struct NodeAddedEvent {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub position: Vec3,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Event fired when a node is updated
#[derive(Debug, Clone, Event)]
pub struct NodeUpdatedEvent {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub position: Option<Vec3>,
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

/// Event fired when a node is removed from a graph
#[derive(Debug, Clone, Event)]
pub struct NodeRemovedEvent {
    pub graph_id: GraphId,
    pub node_id: NodeId,
}

/// Event fired when an edge is created between nodes
#[derive(Debug, Clone, Event)]
pub struct EdgeCreatedEvent {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub weight: f32,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Event fired when an edge is updated
#[derive(Debug, Clone, Event)]
pub struct EdgeUpdatedEvent {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub weight: Option<f32>,
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

/// Event fired when an edge is removed
#[derive(Debug, Clone, Event)]
pub struct EdgeRemovedEvent {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
}

/// Event fired when an element is selected
#[derive(Debug, Clone, Event)]
pub struct ElementSelectedEvent {
    pub graph_id: GraphId,
    pub element_type: ElementType,
    pub element_id: ElementId,
}

/// Event fired when an element is deselected
#[derive(Debug, Clone, Event)]
pub struct ElementDeselectedEvent {
    pub graph_id: GraphId,
    pub element_type: ElementType,
    pub element_id: ElementId,
}

/// Event fired when multiple elements are selected
#[derive(Debug, Clone, Event)]
pub struct MultipleElementsSelectedEvent {
    pub graph_id: GraphId,
    pub elements: Vec<(ElementType, ElementId)>,
}

/// Event fired when drag operation starts
#[derive(Debug, Clone, Event)]
pub struct DragStartedEvent {
    pub graph_id: GraphId,
    pub element_type: ElementType,
    pub element_id: ElementId,
    pub start_position: Vec3,
}

/// Event fired during drag operation
#[derive(Debug, Clone, Event)]
pub struct DragUpdatedEvent {
    pub graph_id: GraphId,
    pub element_type: ElementType,
    pub element_id: ElementId,
    pub current_position: Vec3,
}

/// Event fired when drag operation ends
#[derive(Debug, Clone, Event)]
pub struct DragEndedEvent {
    pub graph_id: GraphId,
    pub element_type: ElementType,
    pub element_id: ElementId,
    pub final_position: Vec3,
}

/// Event fired when a layout algorithm is applied
#[derive(Debug, Clone, Event)]
pub struct LayoutAppliedEvent {
    pub graph_id: GraphId,
    pub layout_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Event fired when graph analysis is completed
#[derive(Debug, Clone, Event)]
pub struct GraphAnalysisCompletedEvent {
    pub graph_id: GraphId,
    pub analysis_type: String,
    pub results: HashMap<String, serde_json::Value>,
}

/// Type of graph element
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    Node,
    Edge,
}

/// Union type for element IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementId {
    Node(NodeId),
    Edge(EdgeId),
}

/// Event aggregator for batch operations
#[derive(Debug, Clone, Event)]
pub struct BatchOperationEvent {
    pub graph_id: GraphId,
    pub operation_id: uuid::Uuid,
    pub operations: Vec<GraphOperation>,
}

/// Individual graph operations for batch processing
#[derive(Debug, Clone)]
pub enum GraphOperation {
    AddNode {
        node_id: NodeId,
        position: Vec3,
        properties: HashMap<String, serde_json::Value>,
    },
    RemoveNode {
        node_id: NodeId,
    },
    AddEdge {
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        weight: f32,
        properties: HashMap<String, serde_json::Value>,
    },
    RemoveEdge {
        edge_id: EdgeId,
    },
    UpdateNode {
        node_id: NodeId,
        position: Option<Vec3>,
        properties: Option<HashMap<String, serde_json::Value>>,
    },
    UpdateEdge {
        edge_id: EdgeId,
        weight: Option<f32>,
        properties: Option<HashMap<String, serde_json::Value>>,
    },
}
