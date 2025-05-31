use crate::contexts::graph_management::domain::*;
use bevy::prelude::*;
use uuid::Uuid;

// ============= Graph Management Events =============
// Events are past-tense facts about what happened in the domain

/// A new graph was created
#[derive(Event, Debug, Clone)]
pub struct GraphCreated {
    pub graph: GraphIdentity,
    pub metadata: GraphMetadata,
    pub timestamp: std::time::SystemTime,
}

/// A node was added to the graph
#[derive(Event, Debug, Clone)]
pub struct NodeAdded {
    pub graph: GraphIdentity,
    pub node: NodeIdentity,
    pub content: NodeContent,
    pub position: SpatialPosition,
}

/// An edge was connected between nodes
#[derive(Event, Debug, Clone)]
pub struct EdgeConnected {
    pub graph: GraphIdentity,
    pub edge: EdgeIdentity,
    pub relationship: EdgeRelationship,
}

/// A node was removed from the graph
#[derive(Event, Debug, Clone)]
pub struct NodeRemoved {
    pub graph: GraphIdentity,
    pub node: NodeIdentity,
}

/// An edge was disconnected
#[derive(Event, Debug, Clone)]
pub struct EdgeDisconnected {
    pub graph: GraphIdentity,
    pub edge: EdgeIdentity,
}

/// A node's position was updated
#[derive(Event, Debug, Clone)]
pub struct NodeMoved {
    pub graph: GraphIdentity,
    pub node: NodeIdentity,
    pub from_position: SpatialPosition,
    pub to_position: SpatialPosition,
}

/// A property was updated on a node or edge
#[derive(Event, Debug, Clone)]
pub struct PropertyUpdated {
    pub graph: GraphIdentity,
    pub element_id: Uuid, // Could be NodeIdentity or EdgeIdentity
    pub property_key: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
}

/// A label was applied to a node
#[derive(Event, Debug, Clone)]
pub struct LabelApplied {
    pub graph: GraphIdentity,
    pub node: NodeIdentity,
    pub label: String,
}

/// The graph was deleted
#[derive(Event, Debug, Clone)]
pub struct GraphDeleted {
    pub graph: GraphIdentity,
    pub reason: DeletionReason,
}

#[derive(Debug, Clone)]
pub enum DeletionReason {
    UserRequested,
    SystemMaintenance,
    PolicyViolation,
}

// ============= Subgraph Events =============

/// An external graph was imported as a subgraph
#[derive(Event, Debug, Clone)]
pub struct SubgraphImported {
    pub parent_graph: GraphIdentity,
    pub subgraph: GraphIdentity,
    pub source_file: String,
    pub position_offset: Vec3,
}

/// A subgraph was extracted as an independent graph
#[derive(Event, Debug, Clone)]
pub struct SubgraphExtracted {
    pub parent_graph: GraphIdentity,
    pub extracted_graph: GraphIdentity,
    pub node_ids: Vec<NodeIdentity>,
}

/// An edge was created between subgraphs
#[derive(Event, Debug, Clone)]
pub struct InterSubgraphEdgeCreated {
    pub graph: GraphIdentity,
    pub edge: EdgeIdentity,
    pub from_subgraph: GraphIdentity,
    pub to_subgraph: GraphIdentity,
}
