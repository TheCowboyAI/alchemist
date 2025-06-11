//! Core concept graph types

use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use petgraph::graph::{Graph, NodeIndex, EdgeIndex};

use super::{QualityDimension, CategoryType, GraphMorphism};

/// Unique identifier for a concept
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConceptId(Uuid);

impl ConceptId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ConceptId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ConceptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of concept in the graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConceptType {
    /// Atomic concept - cannot be decomposed further
    Atom,
    /// Composite concept - contains a subgraph
    Composite,
    /// Function concept - transforms inputs to outputs
    Function,
    /// Entity concept - has identity
    Entity,
    /// Value object concept - defined by attributes
    ValueObject,
    /// Aggregate concept - consistency boundary
    Aggregate,
    /// Policy concept - business rule
    Policy,
    /// Event concept - something that happened
    Event,
    /// Command concept - request to change state
    Command,
}

/// A graph representing concepts and their relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptGraph {
    /// Unique identifier
    pub id: ConceptId,

    /// Human-readable name
    pub name: String,

    /// Category from Applied Category Theory
    pub category: CategoryType,

    /// Quality dimensions defining the conceptual space
    pub quality_dimensions: Vec<QualityDimension>,

    /// The graph structure
    pub structure: Graph<ConceptNode, ConceptEdge>,

    /// Morphisms to/from other concepts
    pub morphisms: Vec<GraphMorphism>,

    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ConceptGraph {
    /// Create a new concept graph
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: ConceptId::new(),
            name: name.into(),
            category: CategoryType::default(),
            quality_dimensions: Vec::new(),
            structure: Graph::new(),
            morphisms: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a quality dimension
    pub fn with_dimension(mut self, dimension: QualityDimension) -> Self {
        self.quality_dimensions.push(dimension);
        self
    }

    /// Set the category type
    pub fn with_category(mut self, category: CategoryType) -> Self {
        self.category = category;
        self
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: ConceptNode) -> NodeIndex {
        self.structure.add_node(node)
    }

    /// Add an edge between nodes
    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex, edge: ConceptEdge) -> EdgeIndex {
        self.structure.add_edge(source, target, edge)
    }

    /// Add a morphism to another concept
    pub fn add_morphism(&mut self, morphism: GraphMorphism) {
        self.morphisms.push(morphism);
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.structure.node_count()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.structure.edge_count()
    }
}

/// Nodes in a concept graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptNode {
    /// Atomic concept - indivisible
    Atom {
        id: NodeId,
        concept_type: ConceptType,
        quality_position: super::ConceptualPoint,
        properties: HashMap<String, Value>,
    },

    /// Composite concept - contains a subgraph
    Composite {
        id: NodeId,
        quality_position: super::ConceptualPoint,
        subgraph: Box<ConceptGraph>,
    },

    /// Function concept - transforms concepts
    Function {
        id: NodeId,
        quality_position: super::ConceptualPoint,
        input_type: ConceptType,
        output_type: ConceptType,
        implementation: FunctionImpl,
    },
}

impl ConceptNode {
    /// Get the node's ID
    pub fn id(&self) -> NodeId {
        match self {
            ConceptNode::Atom { id, .. } => *id,
            ConceptNode::Composite { id, .. } => *id,
            ConceptNode::Function { id, .. } => *id,
        }
    }

    /// Get the node's quality position
    pub fn quality_position(&self) -> &super::ConceptualPoint {
        match self {
            ConceptNode::Atom { quality_position, .. } => quality_position,
            ConceptNode::Composite { quality_position, .. } => quality_position,
            ConceptNode::Function { quality_position, .. } => quality_position,
        }
    }
}

/// Node identifier within a graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Uuid);

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

/// Function implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionImpl {
    /// Built-in function
    BuiltIn(String),
    /// User-defined function
    UserDefined {
        name: String,
        body: String,
    },
    /// External function reference
    External {
        module: String,
        function: String,
    },
}

/// Edges in a concept graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptEdge {
    /// Edge identifier
    pub id: EdgeId,

    /// Type of relationship
    pub relationship: ConceptRelationship,

    /// Edge properties
    pub properties: HashMap<String, Value>,
}

impl ConceptEdge {
    pub fn new(relationship: ConceptRelationship) -> Self {
        Self {
            id: EdgeId::new(),
            relationship,
            properties: HashMap::new(),
        }
    }
}

/// Edge identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(Uuid);

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

/// Types of relationships between concepts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConceptRelationship {
    /// Hierarchical relationship (is-a)
    IsA,
    /// Part-of relationship
    PartOf,
    /// Depends on relationship
    DependsOn,
    /// Transforms into
    TransformsTo,
    /// Composed with
    ComposedWith,
    /// Similar to
    SimilarTo,
    /// Opposite of
    OppositeOf,
    /// Triggers
    Triggers,
    /// Constrains
    Constrains,
    /// Custom relationship
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::conceptual_graph::quality_dimension::ConceptualPoint;

    #[test]
    fn test_concept_graph_creation() {
        let mut graph = ConceptGraph::new("TestConcept");
        assert_eq!(graph.name, "TestConcept");
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_nodes_and_edges() {
        let mut graph = ConceptGraph::new("TestGraph");

        // Add two atomic nodes
        let node1 = ConceptNode::Atom {
            id: NodeId::new(),
            concept_type: ConceptType::Entity,
            quality_position: ConceptualPoint::new(vec![1.0, 2.0, 3.0]),
            properties: HashMap::new(),
        };

        let node2 = ConceptNode::Atom {
            id: NodeId::new(),
            concept_type: ConceptType::ValueObject,
            quality_position: ConceptualPoint::new(vec![4.0, 5.0, 6.0]),
            properties: HashMap::new(),
        };

        let idx1 = graph.add_node(node1);
        let idx2 = graph.add_node(node2);

        // Add edge
        let edge = ConceptEdge::new(ConceptRelationship::DependsOn);
        graph.add_edge(idx1, idx2, edge);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_composite_node() {
        let subgraph = ConceptGraph::new("SubConcept");

        let composite = ConceptNode::Composite {
            id: NodeId::new(),
            quality_position: ConceptualPoint::new(vec![0.0, 0.0, 0.0]),
            subgraph: Box::new(subgraph),
        };

        let mut parent = ConceptGraph::new("ParentConcept");
        parent.add_node(composite);

        assert_eq!(parent.node_count(), 1);
    }
}
