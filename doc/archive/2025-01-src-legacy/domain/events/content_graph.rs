use crate::domain::{
    aggregates::content_graph::{BusinessMetrics, NodeContent, PatternType, SemanticCluster},
    value_objects::{
        EdgeId, GraphId, NodeId, Position3D, RelatedBy,
    },
};
use cim_ipld::types::Cid;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::SystemTime};

/// Event emitted when a content graph is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentGraphCreated {
    pub graph_id: GraphId,
    pub created_at: SystemTime,
}

/// Event emitted when content is added to the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAdded {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub content: NodeContent,
    pub position: Position3D,
    pub metadata: HashMap<String, serde_json::Value>,
    pub content_cid: Option<Cid>,
}

/// Event emitted when content is removed from the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRemoved {
    pub graph_id: GraphId,
    pub node_id: NodeId,
}

/// Event emitted when a relationship is established
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEstablished {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub relationship: RelatedBy,
    pub strength: f64,
}

/// Event emitted when a relationship is removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipRemoved {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
}

/// Event emitted when a relationship is discovered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipDiscovered {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub relationship: RelatedBy,
    pub confidence: f64,
}

/// Event emitted when semantic clusters are updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticClustersUpdated {
    pub graph_id: GraphId,
    pub clusters: Vec<SemanticCluster>,
}

/// Event emitted when business metrics are calculated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsCalculated {
    pub graph_id: GraphId,
    pub metrics: BusinessMetrics,
}

/// Event emitted when a subgraph view is defined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphDefined {
    pub graph_id: GraphId,
    pub name: String,
    pub description: String,
    pub node_ids: Vec<NodeId>,
}

/// Event emitted when a subgraph view is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphUpdated {
    pub graph_id: GraphId,
    pub name: String,
    pub node_ids: Vec<NodeId>,
}

/// Event emitted when a subgraph view is removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphRemoved {
    pub graph_id: GraphId,
    pub name: String,
}

/// Event emitted when a pattern is detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDetected {
    pub graph_id: GraphId,
    pub pattern_type: PatternType,
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeId>,
    pub confidence: f64,
    pub detected_at: SystemTime,
}
