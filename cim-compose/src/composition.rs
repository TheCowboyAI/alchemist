//! GraphComposition - The fundamental building block where every domain concept is a graph
//!
//! This module implements the GraphComposition pattern where everything in the system
//! is represented as a composable graph structure. This enables uniform operations,
//! type-safe composition, and category theory-based transformations.

use crate::base_types::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Represents a composable graph structure that can be combined with other graphs
pub trait Composable: Sized {
    type Output;

    /// Compose two graphs into a new graph
    fn compose(&self, other: &Self) -> Result<Self::Output, CompositionError>;

    /// Check if composition is valid
    fn can_compose_with(&self, other: &Self) -> bool;
}

/// Errors that can occur during graph composition
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CompositionError {
    #[error("Incompatible composition types: {0} and {1}")]
    IncompatibleTypes(String, String),

    #[error("Invalid composition: {0}")]
    InvalidComposition(String),

    #[error("Morphism error: {0}")]
    MorphismError(String),

    #[error("Functor error: {0}")]
    FunctorError(String),

    #[error("Monad error: {0}")]
    MonadError(String),

    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("Cycle detected in composition")]
    CycleDetected,
}

/// Types of graph composition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompositionType {
    /// Single node, no edges - represents a value
    Atomic { value_type: String },

    /// Multiple nodes/edges - represents a structure
    Composite { structure_type: String },

    /// Maps one graph to another - represents transformation
    Functor {
        source_type: String,
        target_type: String,
    },

    /// Wraps a graph-returning computation - represents context
    Monad { context_type: String },

    /// Represents a DDD concept
    Domain(DomainCompositionType),
}

/// Domain-specific composition types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DomainCompositionType {
    Entity { entity_type: String },
    ValueObject { value_type: String },
    Aggregate { aggregate_type: String },
    Service { service_type: String },
    Event { event_type: String },
    Command { command_type: String },
    BoundedContext { domain: String },
}

/// A node in the composition graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionNode<N = BaseNodeType> {
    pub id: NodeId,
    pub node_type: N,
    pub label: String,
    pub data: JsonValue,
    pub metadata: HashMap<String, JsonValue>,
}

impl<N> CompositionNode<N> {
    pub fn new(node_type: N, label: String, data: JsonValue) -> Self {
        Self {
            id: NodeId::new(),
            node_type,
            label,
            data,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: JsonValue) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_field(self, field: &str, value: JsonValue) -> Self
    where
        N: Clone,
    {
        let mut node = self;
        if let JsonValue::Object(ref mut map) = node.data {
            map.insert(field.to_string(), value);
        }
        node
    }
}

impl CompositionNode<BaseNodeType> {
    pub fn is_type(&self, type_name: &str) -> bool {
        match &self.node_type {
            BaseNodeType::Custom(name) => name == type_name,
            _ => false,
        }
    }
}

/// An edge in the composition graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionEdge<R = BaseRelationshipType> {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub relationship: Relationship<R>,
}

impl<R> CompositionEdge<R> {
    pub fn new(source: NodeId, target: NodeId, relationship_type: R) -> Self {
        Self {
            id: EdgeId::new(),
            source,
            target,
            relationship: Relationship::new(relationship_type),
        }
    }
}

/// The main GraphComposition structure
#[derive(Serialize, Deserialize)]
pub struct GraphComposition<N = BaseNodeType, R = BaseRelationshipType> {
    pub id: GraphId,
    pub composition_root: NodeId,
    pub composition_type: CompositionType,
    pub nodes: HashMap<NodeId, CompositionNode<N>>,
    pub edges: HashMap<EdgeId, CompositionEdge<R>>,
    pub metadata: Metadata,
    #[serde(skip)]
    invariants: Vec<Box<dyn Fn(&GraphComposition<N, R>) -> bool>>,
}

impl<N, R> Clone for GraphComposition<N, R>
where
    N: Clone,
    R: Clone,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            composition_root: self.composition_root,
            composition_type: self.composition_type.clone(),
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            metadata: self.metadata.clone(),
            invariants: Vec::new(), // Invariants cannot be cloned
        }
    }
}

impl<N, R> std::fmt::Debug for GraphComposition<N, R>
where
    N: std::fmt::Debug,
    R: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphComposition")
            .field("id", &self.id)
            .field("composition_root", &self.composition_root)
            .field("composition_type", &self.composition_type)
            .field("nodes", &self.nodes)
            .field("edges", &self.edges)
            .field("metadata", &self.metadata)
            .field("invariants", &format!("<{} invariants>", self.invariants.len()))
            .finish()
    }
}

impl<N, R> PartialEq for GraphComposition<N, R>
where
    N: PartialEq,
    R: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.composition_root == other.composition_root
            && self.composition_type == other.composition_type
            && self.nodes == other.nodes
            && self.edges == other.edges
            && self.metadata == other.metadata
        // Note: invariants are not compared
    }
}

impl<N, R> GraphComposition<N, R>
where
    N: Clone + Serialize + for<'de> Deserialize<'de>,
    R: Clone + Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new graph with a root node
    pub fn new(root_type: N, composition_type: CompositionType) -> Self {
        let root_node = CompositionNode::new(
            root_type,
            "root".to_string(),
            JsonValue::Object(serde_json::Map::new()),
        );
        let root_id = root_node.id;

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root_node);

        Self {
            id: GraphId::new(),
            composition_root: root_id,
            composition_type,
            nodes,
            edges: HashMap::new(),
            metadata: Metadata::default(),
            invariants: Vec::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(mut self, node_type: N, label: &str, data: impl Into<JsonValue>) -> Self {
        let node = CompositionNode::new(node_type, label.to_string(), data.into());
        self.nodes.insert(node.id, node);
        self
    }

    /// Add a node with a specific ID
    pub fn add_node_with_id(
        mut self,
        id: NodeId,
        node_type: N,
        label: &str,
        data: impl Into<JsonValue>,
    ) -> Self {
        let mut node = CompositionNode::new(node_type, label.to_string(), data.into());
        node.id = id;
        self.nodes.insert(id, node);
        self
    }

    /// Add an edge between nodes
    pub fn add_edge(mut self, source: NodeId, target: NodeId, relationship: R) -> Self {
        let edge = CompositionEdge::new(source, target, relationship);
        self.edges.insert(edge.id, edge);
        self
    }

    /// Add an edge by node labels
    pub fn add_edge_by_label(
        self,
        source_label: &str,
        target_label: &str,
        relationship: R,
    ) -> Self {
        let source_id = if source_label == "root" {
            Some(self.composition_root)
        } else {
            self.nodes
                .values()
                .find(|n| n.label == source_label)
                .map(|n| n.id)
        };

        let target_id = if target_label == "root" {
            Some(self.composition_root)
        } else {
            self.nodes
                .values()
                .find(|n| n.label == target_label)
                .map(|n| n.id)
        };

        if let (Some(source), Some(target)) = (source_id, target_id) {
            self.add_edge(source, target, relationship)
        } else {
            self
        }
    }

    /// Add an invariant constraint
    pub fn with_invariant<F>(mut self, invariant: F) -> Self
    where
        F: Fn(&GraphComposition<N, R>) -> bool + 'static,
    {
        self.invariants.push(Box::new(invariant));
        self
    }

    /// Check if all invariants hold
    pub fn check_invariants(&self) -> Result<(), CompositionError> {
        for (i, invariant) in self.invariants.iter().enumerate() {
            if !invariant(self) {
                return Err(CompositionError::InvariantViolation(format!(
                    "Invariant {} failed",
                    i
                )));
            }
        }
        Ok(())
    }

    /// Find leaf nodes (nodes with no outgoing edges)
    pub fn find_leaves(&self) -> Vec<NodeId> {
        let mut leaves = Vec::new();

        for node_id in self.nodes.keys() {
            let has_outgoing = self.edges.values().any(|e| e.source == *node_id);
            if !has_outgoing {
                leaves.push(*node_id);
            }
        }

        leaves
    }

    /// Find root nodes (nodes with no incoming edges)
    pub fn find_roots(&self) -> Vec<NodeId> {
        let mut roots = Vec::new();

        for node_id in self.nodes.keys() {
            let has_incoming = self.edges.values().any(|e| e.target == *node_id);
            if !has_incoming {
                roots.push(*node_id);
            }
        }

        roots
    }

    /// Get all nodes connected to a given node
    pub fn get_connected_nodes(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut connected = Vec::new();

        // Outgoing connections
        for edge in self.edges.values() {
            if edge.source == node_id {
                connected.push(edge.target);
            }
            if edge.relationship.bidirectional && edge.target == node_id {
                connected.push(edge.source);
            }
        }

        connected
    }

    /// Map a function over all nodes
    pub fn map_nodes<F, N2>(self, f: F) -> GraphComposition<N2, R>
    where
        F: Fn(&CompositionNode<N>) -> CompositionNode<N2>,
        N2: Clone + Serialize + for<'de> Deserialize<'de>,
    {
        let mut new_nodes = HashMap::new();
        for (id, node) in self.nodes {
            let new_node = f(&node);
            new_nodes.insert(id, new_node);
        }

        GraphComposition {
            id: self.id,
            composition_root: self.composition_root,
            composition_type: self.composition_type,
            nodes: new_nodes,
            edges: self.edges,
            metadata: self.metadata,
            invariants: Vec::new(), // Invariants don't transfer across type changes
        }
    }

    /// Fold the graph to a value
    pub fn fold<T, F>(&self, init: T, f: F) -> T
    where
        F: Fn(T, &CompositionNode<N>) -> T,
    {
        self.nodes.values().fold(init, f)
    }
}

// Specialized constructors for BaseNodeType
impl GraphComposition<BaseNodeType, BaseRelationshipType> {
    /// Create an atomic graph (single node, no edges)
    pub fn atomic(value_type: &str, data: JsonValue) -> Self {
        let mut graph = Self::new(
            BaseNodeType::Value,
            CompositionType::Atomic {
                value_type: value_type.to_string(),
            },
        );
        graph.metadata.name = value_type.to_string();

        // Update root node with data
        if let Some(root) = graph.nodes.get_mut(&graph.composition_root) {
            root.data = data;
            root.label = value_type.to_string();
        }

        graph
    }

    /// Create a composite graph
    pub fn composite(structure_type: &str) -> Self {
        let mut graph = Self::new(
            BaseNodeType::Aggregate,
            CompositionType::Composite {
                structure_type: structure_type.to_string(),
            },
        );
        graph.metadata.name = structure_type.to_string();
        graph
    }

    /// Create an entity graph
    pub fn entity(entity_type: &str, entity_id: impl Into<String>) -> Self {
        let mut graph = Self::new(
            BaseNodeType::EntityReference,
            CompositionType::Domain(DomainCompositionType::Entity {
                entity_type: entity_type.to_string(),
            }),
        );
        graph.metadata.name = entity_type.to_string();

        // Update root node with entity ID
        if let Some(root) = graph.nodes.get_mut(&graph.composition_root) {
            root.data = serde_json::json!({ "id": entity_id.into() });
            root.label = entity_type.to_string();
        }

        graph
    }

    /// Create an aggregate graph
    pub fn aggregate(aggregate_type: &str, aggregate_id: impl Into<String>) -> Self {
        let mut graph = Self::new(
            BaseNodeType::Aggregate,
            CompositionType::Domain(DomainCompositionType::Aggregate {
                aggregate_type: aggregate_type.to_string(),
            }),
        );
        graph.metadata.name = aggregate_type.to_string();

        // Update root node with aggregate ID
        if let Some(root) = graph.nodes.get_mut(&graph.composition_root) {
            root.data = serde_json::json!({ "id": aggregate_id.into() });
            root.label = aggregate_type.to_string();
        }

        graph
    }

    /// Sequential composition: self then other
    pub fn then(
        &self,
        other: &GraphComposition<BaseNodeType, BaseRelationshipType>,
    ) -> Result<GraphComposition<BaseNodeType, BaseRelationshipType>, CompositionError> {
        let mut result = self.clone();
        result.id = GraphId::new();

        // Add all nodes from other
        for (_, node) in &other.nodes {
            result.nodes.insert(node.id, node.clone());
        }

        // Add all edges from other
        for (_, edge) in &other.edges {
            result.edges.insert(edge.id, edge.clone());
        }

        // Connect self's leaves to other's root
        let leaves = self.find_leaves();
        for leaf_id in leaves {
            result = result.add_edge(leaf_id, other.composition_root, BaseRelationshipType::Sequence);
        }

        result.composition_type = CompositionType::Composite {
            structure_type: "Sequential".to_string(),
        };

        Ok(result)
    }

    /// Parallel composition: self and other
    pub fn parallel(
        &self,
        other: &GraphComposition<BaseNodeType, BaseRelationshipType>,
    ) -> Result<GraphComposition<BaseNodeType, BaseRelationshipType>, CompositionError> {
        let mut result = GraphComposition::composite("Parallel");

        // Add all nodes from both graphs
        for (_, node) in &self.nodes {
            result.nodes.insert(node.id, node.clone());
        }
        for (_, node) in &other.nodes {
            result.nodes.insert(node.id, node.clone());
        }

        // Add all edges from both graphs
        for (_, edge) in &self.edges {
            result.edges.insert(edge.id, edge.clone());
        }
        for (_, edge) in &other.edges {
            result.edges.insert(edge.id, edge.clone());
        }

        // Connect new root to both subgraph roots
        let root = result.composition_root;
        result = result
            .add_edge(root, self.composition_root, BaseRelationshipType::Parallel)
            .add_edge(root, other.composition_root, BaseRelationshipType::Parallel);

        Ok(result)
    }

    /// Choice composition: self or other
    pub fn choice(
        &self,
        other: &GraphComposition<BaseNodeType, BaseRelationshipType>,
    ) -> Result<GraphComposition<BaseNodeType, BaseRelationshipType>, CompositionError> {
        let mut result = GraphComposition::composite("Choice");

        // Add all nodes from both graphs
        for (_, node) in &self.nodes {
            result.nodes.insert(node.id, node.clone());
        }
        for (_, node) in &other.nodes {
            result.nodes.insert(node.id, node.clone());
        }

        // Add all edges from both graphs
        for (_, edge) in &self.edges {
            result.edges.insert(edge.id, edge.clone());
        }
        for (_, edge) in &other.edges {
            result.edges.insert(edge.id, edge.clone());
        }

        // Connect new root to both subgraph roots with choice edges
        let root = result.composition_root;
        result = result
            .add_edge(root, self.composition_root, BaseRelationshipType::Choice)
            .add_edge(root, other.composition_root, BaseRelationshipType::Choice);

        Ok(result)
    }
}

impl<N, R> Composable for GraphComposition<N, R>
where
    N: Clone + Serialize + for<'de> Deserialize<'de>,
    R: Clone + Serialize + for<'de> Deserialize<'de>,
{
    type Output = GraphComposition<N, R>;

    fn compose(&self, other: &Self) -> Result<Self::Output, CompositionError> {
        // Default composition merges graphs
        let mut result = self.clone();

        // Add all nodes from other
        for (_, node) in &other.nodes {
            result.nodes.insert(node.id, node.clone());
        }

        // Add all edges from other
        for (_, edge) in &other.edges {
            result.edges.insert(edge.id, edge.clone());
        }

        Ok(result)
    }

    fn can_compose_with(&self, _other: &Self) -> bool {
        // For now, any graphs can be composed
        true
    }
}

/// Morphism between graphs
pub trait GraphMorphism<N1, R1, N2, R2>: Send + Sync
where
    N1: Clone + Serialize + for<'de> Deserialize<'de>,
    R1: Clone + Serialize + for<'de> Deserialize<'de>,
    N2: Clone + Serialize + for<'de> Deserialize<'de>,
    R2: Clone + Serialize + for<'de> Deserialize<'de>,
{
    fn apply(
        &self,
        graph: &GraphComposition<N1, R1>,
    ) -> Result<GraphComposition<N2, R2>, CompositionError>;
}

/// Functor trait for structure-preserving maps
pub trait GraphFunctor<N, R>
where
    N: Clone + Serialize + for<'de> Deserialize<'de>,
    R: Clone + Serialize + for<'de> Deserialize<'de>,
{
    fn fmap<F, N2>(&self, f: F) -> GraphComposition<N2, R>
    where
        F: Fn(&CompositionNode<N>) -> CompositionNode<N2>,
        N2: Clone + Serialize + for<'de> Deserialize<'de>;
}

impl<N, R> GraphFunctor<N, R> for GraphComposition<N, R>
where
    N: Clone + Serialize + for<'de> Deserialize<'de>,
    R: Clone + Serialize + for<'de> Deserialize<'de>,
{
    fn fmap<F, N2>(&self, f: F) -> GraphComposition<N2, R>
    where
        F: Fn(&CompositionNode<N>) -> CompositionNode<N2>,
        N2: Clone + Serialize + for<'de> Deserialize<'de>,
    {
        self.clone().map_nodes(f)
    }
}

/// Monad trait for composition with context
pub trait GraphMonad<N, R>
where
    N: Clone + Serialize + for<'de> Deserialize<'de>,
    R: Clone + Serialize + for<'de> Deserialize<'de>,
{
    fn pure(value: CompositionNode<N>) -> GraphComposition<N, R>;

    fn bind<F>(&self, f: F) -> Result<GraphComposition<N, R>, CompositionError>
    where
        F: Fn(&CompositionNode<N>) -> GraphComposition<N, R>;
}

/// Helper function to create a line item graph
pub fn line_item_graph(product: &str, quantity: i32, price: f64) -> GraphComposition {
    GraphComposition::composite("LineItem")
        .add_node(BaseNodeType::Value, "product", serde_json::json!({ "name": product }))
        .add_node(BaseNodeType::Value, "quantity", quantity)
        .add_node(BaseNodeType::Value, "price", price)
        .add_node(BaseNodeType::Value, "total", quantity as f64 * price)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_graph_creation() {
        let money = GraphComposition::atomic(
            "Money",
            serde_json::json!({ "amount": 100, "currency": "USD" }),
        );

        assert_eq!(money.nodes.len(), 1);
        assert_eq!(money.edges.len(), 0);
        assert!(matches!(
            money.composition_type,
            CompositionType::Atomic { .. }
        ));
    }

    #[test]
    fn test_composite_graph_creation() {
        let address = GraphComposition::composite("Address")
            .add_node(BaseNodeType::Value, "street", "123 Main St")
            .add_node(BaseNodeType::Value, "city", "Springfield")
            .add_node(BaseNodeType::Value, "zip", "12345")
            .add_edge_by_label("root", "street", BaseRelationshipType::Contains)
            .add_edge_by_label("root", "city", BaseRelationshipType::Contains)
            .add_edge_by_label("root", "zip", BaseRelationshipType::Contains);

        assert_eq!(address.nodes.len(), 4); // root + 3 nodes
        assert_eq!(address.edges.len(), 3);
        assert!(matches!(
            address.composition_type,
            CompositionType::Composite { .. }
        ));
    }

    #[test]
    fn test_entity_creation() {
        let user = GraphComposition::entity("User", "user-123");

        assert_eq!(user.nodes.len(), 1);
        assert!(matches!(
            user.composition_type,
            CompositionType::Domain(DomainCompositionType::Entity { .. })
        ));

        // Check that the entity has an ID
        let root_node = &user.nodes[&user.composition_root];
        if let JsonValue::Object(ref map) = root_node.data {
            assert_eq!(map.get("id").unwrap(), "user-123");
        } else {
            panic!("Entity should have object data");
        }
    }

    #[test]
    fn test_sequential_composition() {
        let validate = GraphComposition::composite("ValidateOrder");
        let calculate = GraphComposition::composite("CalculatePricing");

        let workflow = validate.then(&calculate).unwrap();

        // Should have nodes from both graphs plus connections
        assert!(workflow.nodes.len() >= 2);
        assert!(workflow.edges.len() >= 1); // At least one sequence edge
    }

    #[test]
    fn test_parallel_composition() {
        let check_inventory = GraphComposition::composite("CheckInventory");
        let verify_payment = GraphComposition::composite("VerifyPayment");

        let parallel = check_inventory.parallel(&verify_payment).unwrap();

        // Should have a new root connected to both subgraphs
        assert!(parallel.nodes.len() >= 3); // new root + 2 subgraph roots
        assert!(parallel.edges.len() >= 2); // 2 parallel edges
    }

    #[test]
    fn test_generic_graph_composition() {
        // Create a graph with custom node and relationship types
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        enum CustomNode {
            Start,
            Process,
            End,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        enum CustomRelation {
            Next,
            Error,
        }

        let graph = GraphComposition::<CustomNode, CustomRelation>::new(
            CustomNode::Start,
            CompositionType::Composite {
                structure_type: "Workflow".to_string(),
            },
        )
        .add_node(CustomNode::Process, "process", serde_json::json!({}))
        .add_node(CustomNode::End, "end", serde_json::json!({}));

        assert_eq!(graph.nodes.len(), 3);
    }

    #[test]
    fn test_functor_map() {
        let graph = GraphComposition::composite("Test")
            .add_node(BaseNodeType::Value, "a", serde_json::json!({ "value": 1 }))
            .add_node(BaseNodeType::Value, "b", serde_json::json!({ "value": 2 }));

        let doubled = graph.fmap(|node| {
            let mut new_node = node.clone();
            if let JsonValue::Object(ref mut map) = new_node.data {
                if let Some(JsonValue::Number(n)) = map.get("value") {
                    if let Some(v) = n.as_i64() {
                        map.insert("value".to_string(), serde_json::json!(v * 2));
                    }
                }
            }
            new_node
        });

        // Verify values were doubled
        for node in doubled.nodes.values() {
            if let JsonValue::Object(ref map) = node.data {
                if let Some(JsonValue::Number(n)) = map.get("value") {
                    let v = n.as_i64().unwrap_or(0);
                    assert!(v == 0 || v == 2 || v == 4); // 0 for root, 2 and 4 for doubled values
                }
            }
        }
    }
}
