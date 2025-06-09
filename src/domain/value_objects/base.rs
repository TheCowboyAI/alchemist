//! Value Objects for the Domain Layer

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;
use uuid::Uuid;
use std::hash::{Hash, Hasher};

/// Type alias for aggregate IDs (using String for flexibility)
pub type AggregateId = String;

/// Type alias for event IDs (using u64 for sequence numbers)
pub type EventId = u64;

/// Unique identifier for a graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphId(pub Uuid);

impl GraphId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for GraphId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for GraphId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for GraphId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Unique identifier for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Component)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for an edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Component)]
pub struct EdgeId(pub Uuid);

impl EdgeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EdgeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(pub Uuid);

impl WorkflowId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WorkflowId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for WorkflowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a workflow step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(pub Uuid);

impl StepId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for StepId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StepId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a user
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 3D position in space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Component)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Check if all coordinates are finite (not NaN or infinite)
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

impl Default for Position3D {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl From<Position3D> for Vec3 {
    fn from(pos: Position3D) -> Self {
        Vec3::new(pos.x, pos.y, pos.z)
    }
}

impl From<Vec3> for Position3D {
    fn from(vec: Vec3) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        }
    }
}

/// Content of a node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeContent {
    pub label: String,
    pub node_type: NodeType,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Types of nodes in the graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Basic,  // Default node type
    Concept,
    Category,
    Instance,
    Relationship,
    Process,
    Event,
    Constraint,
    Custom(String),
}

/// Relationship between nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeRelationship {
    pub relationship_type: RelationshipType,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
    pub bidirectional: bool,
}

/// Types of relationships
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    // DDD relationships
    Contains,
    References,
    DependsOn,
    Publishes,
    Subscribes,
    Implements,
    Extends,

    // Git relationships
    Parent,
    Merged,
    Branched,
    Tagged,

    // Progress relationships
    Sequence,
    Hierarchy,
    Blocks,

    // Generic
    Custom(String),
}

impl fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelationshipType::Contains => write!(f, "Contains"),
            RelationshipType::References => write!(f, "References"),
            RelationshipType::DependsOn => write!(f, "DependsOn"),
            RelationshipType::Publishes => write!(f, "Publishes"),
            RelationshipType::Subscribes => write!(f, "Subscribes"),
            RelationshipType::Implements => write!(f, "Implements"),
            RelationshipType::Extends => write!(f, "Extends"),
            RelationshipType::Parent => write!(f, "Parent"),
            RelationshipType::Merged => write!(f, "Merged"),
            RelationshipType::Branched => write!(f, "Branched"),
            RelationshipType::Tagged => write!(f, "Tagged"),
            RelationshipType::Sequence => write!(f, "Sequence"),
            RelationshipType::Hierarchy => write!(f, "Hierarchy"),
            RelationshipType::Blocks => write!(f, "Blocks"),
            RelationshipType::Custom(s) => write!(f, "{s}"),
        }
    }
}

/// Metadata for a graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub name: String,
    pub bounded_context: String,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
    pub tags: Vec<String>,
}

impl GraphMetadata {
    pub fn new(name: String) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            name,
            bounded_context: String::from("default"),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }
}

impl Default for GraphMetadata {
    fn default() -> Self {
        Self::new(String::from("default"))
    }
}

/// Unique identifier for a subgraph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubgraphId(Uuid);

impl SubgraphId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SubgraphId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SubgraphId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_id_creation() {
        let id1 = GraphId::new();
        let id2 = GraphId::new();

        // Each ID should be unique
        assert_ne!(id1, id2);

        // Display should work
        assert!(!id1.to_string().is_empty());
    }

    #[test]
    fn test_graph_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = GraphId::from(uuid);

        assert_eq!(id.0, uuid);
        assert_eq!(id.to_string(), uuid.to_string());
    }

    #[test]
    fn test_node_id_uniqueness() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_edge_id_default() {
        let id1 = EdgeId::default();
        let id2 = EdgeId::default();

        // Default should create new unique IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_position3d_creation() {
        let pos = Position3D::new(1.0, 2.0, 3.0);

        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        assert_eq!(pos.z, 3.0);
    }

    #[test]
    fn test_position3d_default() {
        let pos = Position3D::default();

        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
        assert_eq!(pos.z, 0.0);
    }

    #[test]
    fn test_position3d_vec3_conversion() {
        let pos = Position3D::new(1.0, 2.0, 3.0);
        let vec: Vec3 = pos.into();

        assert_eq!(vec.x, 1.0);
        assert_eq!(vec.y, 2.0);
        assert_eq!(vec.z, 3.0);

        let pos2: Position3D = vec.into();
        assert_eq!(pos2.x, 1.0);
        assert_eq!(pos2.y, 2.0);
        assert_eq!(pos2.z, 3.0);
    }

    #[test]
    fn test_node_content_creation() {
        let mut properties = std::collections::HashMap::new();
        properties.insert("key".to_string(), serde_json::json!("value"));

        let content = NodeContent {
            label: "Test Node".to_string(),
            node_type: NodeType::Basic,
            properties: properties.clone(),
        };

        assert_eq!(content.label, "Test Node");
        assert_eq!(content.node_type, NodeType::Basic);
        assert_eq!(content.properties, properties);
    }

    #[test]
    fn test_node_type_variants() {
        // Test equality
        assert_eq!(NodeType::Basic, NodeType::Basic);
        assert_ne!(NodeType::Basic, NodeType::Concept);

        // Test custom type
        let custom = NodeType::Custom("MyType".to_string());
        match custom {
            NodeType::Custom(name) => assert_eq!(name, "MyType"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_edge_relationship_creation() {
        let mut properties = std::collections::HashMap::new();
        properties.insert("weight".to_string(), serde_json::json!(1.0));

        let rel = EdgeRelationship {
            relationship_type: RelationshipType::DependsOn,
            properties: properties.clone(),
            bidirectional: false,
        };

        assert_eq!(rel.relationship_type, RelationshipType::DependsOn);
        assert_eq!(rel.properties, properties);
        assert!(!rel.bidirectional);
    }

    #[test]
    fn test_relationship_type_variants() {
        // Test DDD relationships
        assert_eq!(RelationshipType::Contains, RelationshipType::Contains);
        assert_ne!(RelationshipType::Contains, RelationshipType::References);

        // Test custom relationship
        let custom = RelationshipType::Custom("MyRelation".to_string());
        match custom {
            RelationshipType::Custom(name) => assert_eq!(name, "MyRelation"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_graph_metadata_creation() {
        let name = "Test Graph".to_string();
        let metadata = GraphMetadata::new(name.clone());

        assert_eq!(metadata.name, name);
        assert_eq!(metadata.bounded_context, "default");
        assert!(metadata.tags.is_empty());

        // Times should be set
        assert!(metadata.created_at <= std::time::SystemTime::now());
        assert_eq!(metadata.created_at, metadata.updated_at);
    }

    #[test]
    fn test_graph_metadata_default() {
        let metadata = GraphMetadata::default();

        assert_eq!(metadata.name, "default");
        assert_eq!(metadata.bounded_context, "default");
        assert!(metadata.tags.is_empty());
    }

    #[test]
    fn test_value_object_immutability() {
        // Value objects should be immutable after creation
        let id = GraphId::new();
        let id_copy = id; // Copy semantics

        assert_eq!(id, id_copy);

        // Position is also a value object
        let pos = Position3D::new(1.0, 2.0, 3.0);
        let pos_copy = pos;

        assert_eq!(pos, pos_copy);
    }

    #[test]
    fn test_serialization_roundtrip() {
        // Test GraphId serialization
        let id = GraphId::new();
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: GraphId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);

        // Test Position3D serialization
        let pos = Position3D::new(1.5, 2.5, 3.5);
        let serialized = serde_json::to_string(&pos).unwrap();
        let deserialized: Position3D = serde_json::from_str(&serialized).unwrap();
        assert_eq!(pos, deserialized);

        // Test NodeType serialization
        let node_type = NodeType::Custom("TestType".to_string());
        let serialized = serde_json::to_string(&node_type).unwrap();
        let deserialized: NodeType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(node_type, deserialized);
    }

    // New comprehensive tests for better coverage

    #[test]
    fn test_position3d_is_finite() {
        // Test finite positions
        let finite_pos = Position3D::new(1.0, 2.0, 3.0);
        assert!(finite_pos.is_finite());

        // Test NaN positions
        let nan_x = Position3D::new(f32::NAN, 2.0, 3.0);
        assert!(!nan_x.is_finite());

        let nan_y = Position3D::new(1.0, f32::NAN, 3.0);
        assert!(!nan_y.is_finite());

        let nan_z = Position3D::new(1.0, 2.0, f32::NAN);
        assert!(!nan_z.is_finite());

        // Test infinite positions
        let inf_x = Position3D::new(f32::INFINITY, 2.0, 3.0);
        assert!(!inf_x.is_finite());

        let neg_inf_y = Position3D::new(1.0, f32::NEG_INFINITY, 3.0);
        assert!(!neg_inf_y.is_finite());
    }

    #[test]
    fn test_workflow_id_creation() {
        let id1 = WorkflowId::new();
        let id2 = WorkflowId::new();

        // Each ID should be unique
        assert_ne!(id1, id2);

        // Display should work
        assert!(!id1.to_string().is_empty());
        assert_eq!(id1.to_string().len(), 36); // UUID string length
    }

    #[test]
    fn test_step_id_creation() {
        let id1 = StepId::new();
        let id2 = StepId::default();

        // Each ID should be unique
        assert_ne!(id1, id2);

        // Display should work
        assert!(!id1.to_string().is_empty());
    }

    #[test]
    fn test_user_id_creation() {
        let id1 = UserId::new();
        let id2 = UserId::default();

        // Each ID should be unique
        assert_ne!(id1, id2);

        // Display should work
        assert!(!id1.to_string().is_empty());
    }

    #[test]
    fn test_relationship_type_display() {
        // Test all relationship type display implementations
        assert_eq!(RelationshipType::Contains.to_string(), "Contains");
        assert_eq!(RelationshipType::References.to_string(), "References");
        assert_eq!(RelationshipType::DependsOn.to_string(), "DependsOn");
        assert_eq!(RelationshipType::Publishes.to_string(), "Publishes");
        assert_eq!(RelationshipType::Subscribes.to_string(), "Subscribes");
        assert_eq!(RelationshipType::Implements.to_string(), "Implements");
        assert_eq!(RelationshipType::Extends.to_string(), "Extends");
        assert_eq!(RelationshipType::Parent.to_string(), "Parent");
        assert_eq!(RelationshipType::Merged.to_string(), "Merged");
        assert_eq!(RelationshipType::Branched.to_string(), "Branched");
        assert_eq!(RelationshipType::Tagged.to_string(), "Tagged");
        assert_eq!(RelationshipType::Sequence.to_string(), "Sequence");
        assert_eq!(RelationshipType::Hierarchy.to_string(), "Hierarchy");
        assert_eq!(RelationshipType::Blocks.to_string(), "Blocks");
        assert_eq!(RelationshipType::Custom("MyRelation".to_string()).to_string(), "MyRelation");
    }

    #[test]
    fn test_graph_metadata_with_tags() {
        let mut metadata = GraphMetadata::new("Tagged Graph".to_string());
        metadata.tags.push("important".to_string());
        metadata.tags.push("v2".to_string());

        assert_eq!(metadata.name, "Tagged Graph");
        assert_eq!(metadata.tags.len(), 2);
        assert!(metadata.tags.contains(&"important".to_string()));
        assert!(metadata.tags.contains(&"v2".to_string()));
    }

    #[test]
    fn test_graph_model_expected_nodes() {
        // Test complete graph
        assert_eq!(GraphModel::CompleteGraph { order: 5 }.expected_nodes(), Some(5));
        assert_eq!(GraphModel::CompleteGraph { order: 0 }.expected_nodes(), Some(0));

        // Test cycle graph
        assert_eq!(GraphModel::CycleGraph { order: 4 }.expected_nodes(), Some(4));

        // Test path graph
        assert_eq!(GraphModel::PathGraph { order: 3 }.expected_nodes(), Some(3));

        // Test bipartite graph
        assert_eq!(GraphModel::BipartiteGraph { m: 3, n: 4 }.expected_nodes(), Some(7));

        // Test star graph
        assert_eq!(GraphModel::StarGraph { satellites: 5 }.expected_nodes(), Some(6)); // 5 satellites + 1 center

        // Test tree
        let tree = GraphModel::Tree { branching_factor: 2, depth: 2 };
        assert_eq!(tree.expected_nodes(), Some(7)); // 1 + 2 + 4 = 7 nodes

        let tree2 = GraphModel::Tree { branching_factor: 3, depth: 1 };
        assert_eq!(tree2.expected_nodes(), Some(4)); // 1 + 3 = 4 nodes

        // Test models with unknown node count
        assert_eq!(GraphModel::MealyMachine {
            states: vec![],
            inputs: vec![],
            outputs: vec![]
        }.expected_nodes(), None);

        assert_eq!(GraphModel::AddressGraph.expected_nodes(), None);
        assert_eq!(GraphModel::WorkflowGraph { workflow_type: "test".to_string() }.expected_nodes(), None);
    }

    #[test]
    fn test_graph_model_expected_edges() {
        // Test complete graph edges: n(n-1)/2
        assert_eq!(GraphModel::CompleteGraph { order: 5 }.expected_edges(), Some(10)); // 5*4/2 = 10
        assert_eq!(GraphModel::CompleteGraph { order: 4 }.expected_edges(), Some(6)); // 4*3/2 = 6
        assert_eq!(GraphModel::CompleteGraph { order: 0 }.expected_edges(), Some(0));

        // Test cycle graph edges: n
        assert_eq!(GraphModel::CycleGraph { order: 4 }.expected_edges(), Some(4));

        // Test path graph edges: n-1
        assert_eq!(GraphModel::PathGraph { order: 3 }.expected_edges(), Some(2));
        assert_eq!(GraphModel::PathGraph { order: 1 }.expected_edges(), Some(0));
        assert_eq!(GraphModel::PathGraph { order: 0 }.expected_edges(), Some(0)); // saturating_sub

        // Test bipartite graph edges: m*n
        assert_eq!(GraphModel::BipartiteGraph { m: 3, n: 4 }.expected_edges(), Some(12));

        // Test star graph edges: satellites
        assert_eq!(GraphModel::StarGraph { satellites: 5 }.expected_edges(), Some(5));

        // Test tree edges: nodes - 1
        let tree = GraphModel::Tree { branching_factor: 2, depth: 2 };
        assert_eq!(tree.expected_edges(), Some(6)); // 7 nodes - 1 = 6 edges

        // Test models with unknown edge count
        assert_eq!(GraphModel::ConceptualGraph { space_name: "test".to_string() }.expected_edges(), None);
    }

    #[test]
    fn test_graph_model_variants() {
        // Test state machine creation
        let mealy = GraphModel::MealyMachine {
            states: vec!["S0".to_string(), "S1".to_string()],
            inputs: vec!["0".to_string(), "1".to_string()],
            outputs: vec!["A".to_string(), "B".to_string()],
        };

        match mealy {
            GraphModel::MealyMachine { states, inputs, outputs } => {
                assert_eq!(states.len(), 2);
                assert_eq!(inputs.len(), 2);
                assert_eq!(outputs.len(), 2);
            }
            _ => panic!("Expected MealyMachine"),
        }

        // Test custom model
        let custom = GraphModel::Custom {
            name: "MyModel".to_string(),
            properties: serde_json::json!({
                "key": "value",
                "count": 42
            }),
        };

        match custom {
            GraphModel::Custom { name, properties } => {
                assert_eq!(name, "MyModel");
                assert_eq!(properties["key"], "value");
                assert_eq!(properties["count"], 42);
            }
            _ => panic!("Expected Custom"),
        }
    }

    #[test]
    fn test_edge_relationship_bidirectional() {
        let mut rel = EdgeRelationship {
            relationship_type: RelationshipType::References,
            properties: HashMap::new(),
            bidirectional: true,
        };

        assert!(rel.bidirectional);

        // Test with properties
        rel.properties.insert("strength".to_string(), serde_json::json!(0.8));
        rel.properties.insert("verified".to_string(), serde_json::json!(true));

        assert_eq!(rel.properties.len(), 2);
        assert_eq!(rel.properties["strength"], 0.8);
        assert_eq!(rel.properties["verified"], true);
    }

    #[test]
    fn test_all_node_types() {
        // Test all DDD node types
        let types = vec![
            NodeType::Basic,
            NodeType::Concept,
            NodeType::Category,
            NodeType::Instance,
            NodeType::Relationship,
            NodeType::Process,
            NodeType::Event,
            NodeType::Constraint,
        ];

        for node_type in types {
            // Verify they're all different
            assert_ne!(node_type, NodeType::Custom("test".to_string()));
        }

        // Test progress tracking types
        assert_eq!(NodeType::Milestone, NodeType::Milestone);
        assert_eq!(NodeType::Phase, NodeType::Phase);
        assert_eq!(NodeType::Task, NodeType::Task);

        // Test Git types
        assert_eq!(NodeType::GitCommit, NodeType::GitCommit);
        assert_eq!(NodeType::GitBranch, NodeType::GitBranch);
        assert_eq!(NodeType::GitTag, NodeType::GitTag);
        assert_eq!(NodeType::GitMerge, NodeType::GitMerge);
    }

    #[test]
    fn test_graph_metadata_timestamps() {
        let before = std::time::SystemTime::now();
        let metadata = GraphMetadata::new("Timed Graph".to_string());
        let after = std::time::SystemTime::now();

        // Created and updated times should be the same initially
        assert_eq!(metadata.created_at, metadata.updated_at);

        // Times should be within reasonable bounds
        assert!(metadata.created_at >= before);
        assert!(metadata.created_at <= after);
    }

    #[test]
    fn test_id_copy_semantics() {
        // All ID types should implement Copy
        let graph_id = GraphId::new();
        let graph_id_copy = graph_id; // This is a copy, not a move
        assert_eq!(graph_id, graph_id_copy);

        let node_id = NodeId::new();
        let node_id_copy = node_id;
        assert_eq!(node_id, node_id_copy);

        let edge_id = EdgeId::new();
        let edge_id_copy = edge_id;
        assert_eq!(edge_id, edge_id_copy);

        let workflow_id = WorkflowId::new();
        let workflow_id_copy = workflow_id;
        assert_eq!(workflow_id, workflow_id_copy);

        let step_id = StepId::new();
        let step_id_copy = step_id;
        assert_eq!(step_id, step_id_copy);

        let user_id = UserId::new();
        let user_id_copy = user_id;
        assert_eq!(user_id, user_id_copy);
    }

    #[test]
    fn test_complex_graph_models() {
        // Test tree with different parameters
        let binary_tree = GraphModel::Tree { branching_factor: 2, depth: 3 };
        assert_eq!(binary_tree.expected_nodes(), Some(15)); // 1 + 2 + 4 + 8 = 15

        let ternary_tree = GraphModel::Tree { branching_factor: 3, depth: 2 };
        assert_eq!(ternary_tree.expected_nodes(), Some(13)); // 1 + 3 + 9 = 13

        // Test edge cases
        let single_node_tree = GraphModel::Tree { branching_factor: 0, depth: 0 };
        assert_eq!(single_node_tree.expected_nodes(), Some(1)); // Just root

        let zero_depth_tree = GraphModel::Tree { branching_factor: 5, depth: 0 };
        assert_eq!(zero_depth_tree.expected_nodes(), Some(1)); // Just root
    }

    #[test]
    fn test_moore_machine_model() {
        let moore = GraphModel::MooreMachine {
            states: vec!["Idle".to_string(), "Processing".to_string(), "Done".to_string()],
            inputs: vec!["start".to_string(), "stop".to_string()],
            outputs: vec!["0".to_string(), "1".to_string()],
        };

        match moore {
            GraphModel::MooreMachine { states, inputs, outputs } => {
                assert_eq!(states.len(), 3);
                assert_eq!(inputs.len(), 2);
                assert_eq!(outputs.len(), 2);
                assert!(states.contains(&"Idle".to_string()));
            }
            _ => panic!("Expected MooreMachine"),
        }
    }
}

/// Mathematical graph models
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GraphModel {
    /// Complete graph Kn - every vertex connected to every other vertex
    CompleteGraph { order: usize },

    /// Cycle graph Cn - vertices connected in a cycle
    CycleGraph { order: usize },

    /// Path graph Pn - vertices connected in a line
    PathGraph { order: usize },

    /// Bipartite graph Km,n
    BipartiteGraph { m: usize, n: usize },

    /// Star graph - one central node connected to all others
    StarGraph { satellites: usize },

    /// Tree structure
    Tree {
        branching_factor: usize,
        depth: usize,
    },

    /// Mealy state machine
    MealyMachine {
        states: Vec<String>,
        inputs: Vec<String>,
        outputs: Vec<String>,
    },

    /// Moore state machine
    MooreMachine {
        states: Vec<String>,
        inputs: Vec<String>,
        outputs: Vec<String>,
    },

    /// Domain-specific models
    AddressGraph,
    WorkflowGraph { workflow_type: String },
    ConceptualGraph { space_name: String },

    /// Unknown or custom model
    Custom { name: String, properties: serde_json::Value },
}

impl GraphModel {
    /// Get the expected number of nodes for this model
    pub fn expected_nodes(&self) -> Option<usize> {
        match self {
            GraphModel::CompleteGraph { order } => Some(*order),
            GraphModel::CycleGraph { order } => Some(*order),
            GraphModel::PathGraph { order } => Some(*order),
            GraphModel::BipartiteGraph { m, n } => Some(m + n),
            GraphModel::StarGraph { satellites } => Some(satellites + 1),
            GraphModel::Tree { branching_factor, depth } => {
                // Calculate total nodes in a complete tree
                let mut total = 0;
                let mut level_nodes = 1;
                for _ in 0..=*depth {
                    total += level_nodes;
                    level_nodes *= branching_factor;
                }
                Some(total)
            }
            _ => None, // Variable or unknown
        }
    }

    /// Get the expected number of edges for this model
    pub fn expected_edges(&self) -> Option<usize> {
        match self {
            GraphModel::CompleteGraph { order } => {
                // Complete graph has n(n-1)/2 edges
                // Use saturating operations to prevent overflow
                if *order == 0 {
                    Some(0)
                } else {
                    Some(order.saturating_sub(1).saturating_mul(*order) / 2)
                }
            },
            GraphModel::CycleGraph { order } => Some(*order),
            GraphModel::PathGraph { order } => Some(order.saturating_sub(1)),
            GraphModel::BipartiteGraph { m, n } => Some(m * n),
            GraphModel::StarGraph { satellites } => Some(*satellites),
            GraphModel::Tree { .. } => self.expected_nodes().map(|n| n.saturating_sub(1)),
            _ => None, // Variable or unknown
        }
    }
}
