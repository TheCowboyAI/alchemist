//! GraphComposition - The fundamental building block where every domain concept is a graph
//!
//! This module implements the GraphComposition pattern where everything in the system
//! is represented as a composable graph structure. This enables uniform operations,
//! type-safe composition, and category theory-based transformations.

use super::*;
use crate::domain::DomainError;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fmt;

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
pub struct CompositionNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub label: String,
    pub data: JsonValue,
    pub metadata: HashMap<String, JsonValue>,
}

impl CompositionNode {
    pub fn new(node_type: NodeType, label: String, data: JsonValue) -> Self {
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

    pub fn with_field(&self, field: &str, value: JsonValue) -> Self {
        let mut node = self.clone();
        if let JsonValue::Object(ref mut map) = node.data {
            map.insert(field.to_string(), value);
        }
        node
    }

    pub fn is_type(&self, type_name: &str) -> bool {
        match &self.node_type {
            NodeType::Custom(name) => name == type_name,
            _ => false,
        }
    }
}

/// An edge in the composition graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionEdge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub relationship: EdgeRelationship,
}

impl CompositionEdge {
    pub fn new(source: NodeId, target: NodeId, relationship_type: RelationshipType) -> Self {
        Self {
            id: EdgeId::new(),
            source,
            target,
            relationship: EdgeRelationship {
                relationship_type,
                properties: HashMap::new(),
                bidirectional: false,
            },
        }
    }
}

/// Metadata for a graph composition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionMetadata {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub tags: Vec<String>,
    pub properties: HashMap<String, JsonValue>,
}

impl Default for CompositionMetadata {
    fn default() -> Self {
        Self {
            name: "unnamed".to_string(),
            description: None,
            version: "1.0.0".to_string(),
            tags: Vec::new(),
            properties: HashMap::new(),
        }
    }
}

/// The main GraphComposition structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphComposition {
    pub id: GraphId,
    pub composition_root: NodeId,
    pub composition_type: CompositionType,
    pub nodes: HashMap<NodeId, CompositionNode>,
    pub edges: HashMap<EdgeId, CompositionEdge>,
    pub metadata: CompositionMetadata,
}

impl GraphComposition {
    /// Create an atomic graph (single node, no edges)
    pub fn atomic(value_type: &str, data: JsonValue) -> Self {
        let root_node = CompositionNode::new(
            NodeType::ValueObject,
            value_type.to_string(),
            data,
        );
        let root_id = root_node.id;

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root_node);

        Self {
            id: GraphId::new(),
            composition_root: root_id,
            composition_type: CompositionType::Atomic {
                value_type: value_type.to_string(),
            },
            nodes,
            edges: HashMap::new(),
            metadata: CompositionMetadata {
                name: value_type.to_string(),
                ..Default::default()
            },
        }
    }

    /// Create a composite graph
    pub fn composite(structure_type: &str) -> Self {
        let root_node = CompositionNode::new(
            NodeType::Aggregate,
            "root".to_string(),
            JsonValue::Object(serde_json::Map::new()),
        );
        let root_id = root_node.id;

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root_node);

        Self {
            id: GraphId::new(),
            composition_root: root_id,
            composition_type: CompositionType::Composite {
                structure_type: structure_type.to_string(),
            },
            nodes,
            edges: HashMap::new(),
            metadata: CompositionMetadata {
                name: structure_type.to_string(),
                ..Default::default()
            },
        }
    }

    /// Create an entity graph
    pub fn entity(entity_type: &str, entity_id: impl Into<String>) -> Self {
        let root_node = CompositionNode::new(
            NodeType::Entity,
            entity_type.to_string(),
            serde_json::json!({ "id": entity_id.into() }),
        );
        let root_id = root_node.id;

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root_node);

        Self {
            id: GraphId::new(),
            composition_root: root_id,
            composition_type: CompositionType::Domain(DomainCompositionType::Entity {
                entity_type: entity_type.to_string(),
            }),
            nodes,
            edges: HashMap::new(),
            metadata: CompositionMetadata {
                name: entity_type.to_string(),
                ..Default::default()
            },
        }
    }

    /// Create an aggregate graph
    pub fn aggregate(aggregate_type: &str, aggregate_id: impl Into<String>) -> Self {
        let root_node = CompositionNode::new(
            NodeType::Aggregate,
            aggregate_type.to_string(),
            serde_json::json!({ "id": aggregate_id.into() }),
        );
        let root_id = root_node.id;

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root_node);

        Self {
            id: GraphId::new(),
            composition_root: root_id,
            composition_type: CompositionType::Domain(DomainCompositionType::Aggregate {
                aggregate_type: aggregate_type.to_string(),
            }),
            nodes,
            edges: HashMap::new(),
            metadata: CompositionMetadata {
                name: aggregate_type.to_string(),
                ..Default::default()
            },
        }
    }

    /// Add a node to the graph
    pub fn add_node(mut self, label: &str, data: impl Into<JsonValue>) -> Self {
        let node = CompositionNode::new(
            NodeType::Custom(label.to_string()),
            label.to_string(),
            data.into(),
        );
        self.nodes.insert(node.id, node);
        self
    }

    /// Add a node graph as a subgraph
    pub fn add_node_graph(mut self, label: &str, subgraph: GraphComposition) -> Self {
        // Add all nodes from subgraph
        for (_, node) in subgraph.nodes {
            self.nodes.insert(node.id, node);
        }

        // Add all edges from subgraph
        for (_, edge) in subgraph.edges {
            self.edges.insert(edge.id, edge);
        }

        // Connect root to subgraph root
        let edge = CompositionEdge::new(
            self.composition_root,
            subgraph.composition_root,
            RelationshipType::Contains,
        );
        self.edges.insert(edge.id, edge);

        self
    }

    /// Add an edge between nodes
    pub fn add_edge(
        mut self,
        source_label: &str,
        target_label: &str,
        relationship: impl Into<RelationshipType>,
    ) -> Self {
        // Find nodes by label
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
            let edge = CompositionEdge::new(source, target, relationship.into());
            self.edges.insert(edge.id, edge);
        }

        self
    }

    /// Map a function over all nodes
    pub fn map<F>(&self, f: F) -> Result<GraphComposition, CompositionError>
    where
        F: Fn(&CompositionNode) -> CompositionNode,
    {
        let mut result = self.clone();
        result.id = GraphId::new();

        // Apply function to all nodes
        result.nodes = self
            .nodes
            .iter()
            .map(|(id, node)| (*id, f(node)))
            .collect();

        Ok(result)
    }

    /// Fold the graph to a value
    pub fn fold<T, F>(&self, init: T, f: F) -> T
    where
        F: Fn(T, &CompositionNode) -> T,
    {
        self.nodes.values().fold(init, f)
    }

    /// Find leaf nodes (nodes with no outgoing edges)
    fn find_leaves(&self) -> Vec<NodeId> {
        let mut leaves = Vec::new();

        for node_id in self.nodes.keys() {
            let has_outgoing = self.edges.values().any(|e| e.source == *node_id);
            if !has_outgoing {
                leaves.push(*node_id);
            }
        }

        leaves
    }

    /// Get the total value for aggregate calculations
    pub fn total_value(&self) -> f64 {
        self.fold(0.0, |acc, node| {
            if let JsonValue::Object(ref map) = node.data {
                if let Some(JsonValue::Number(n)) = map.get("value") {
                    if let Some(v) = n.as_f64() {
                        return acc + v;
                    }
                }
            }
            acc
        })
    }

    /// Check if the graph has a valid payment node
    pub fn has_valid_payment(&self) -> bool {
        self.nodes.values().any(|node| {
            node.label.contains("payment") || node.label.contains("Payment")
        })
    }

    /// Sequential composition: self then other
    pub fn then(&self, other: &GraphComposition) -> Result<GraphComposition, CompositionError> {
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
            let edge = CompositionEdge::new(
                leaf_id,
                other.composition_root,
                RelationshipType::Sequence,
            );
            result.edges.insert(edge.id, edge);
        }

        result.composition_type = CompositionType::Composite {
            structure_type: "Sequential".to_string(),
        };

        Ok(result)
    }

    /// Parallel composition: self and other
    pub fn parallel(&self, other: &GraphComposition) -> Result<GraphComposition, CompositionError> {
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
        let edge1 = CompositionEdge::new(
            result.composition_root,
            self.composition_root,
            RelationshipType::Parallel,
        );
        let edge2 = CompositionEdge::new(
            result.composition_root,
            other.composition_root,
            RelationshipType::Parallel,
        );
        result.edges.insert(edge1.id, edge1);
        result.edges.insert(edge2.id, edge2);

        Ok(result)
    }

    /// Choice composition: self or other
    pub fn choice(&self, other: &GraphComposition) -> Result<GraphComposition, CompositionError> {
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
        let edge1 = CompositionEdge::new(
            result.composition_root,
            self.composition_root,
            RelationshipType::Choice,
        );
        let edge2 = CompositionEdge::new(
            result.composition_root,
            other.composition_root,
            RelationshipType::Choice,
        );
        result.edges.insert(edge1.id, edge1);
        result.edges.insert(edge2.id, edge2);

        Ok(result)
    }
}

impl Composable for GraphComposition {
    type Output = GraphComposition;

    fn compose(&self, other: &Self) -> Result<Self::Output, CompositionError> {
        // Default composition is sequential
        self.then(other)
    }

    fn can_compose_with(&self, _other: &Self) -> bool {
        // For now, any graphs can be composed
        // In the future, we might check type compatibility
        true
    }
}

/// Morphism between graphs
pub trait GraphMorphism: Send + Sync {
    fn apply(&self, graph: &GraphComposition) -> Result<GraphComposition, CompositionError>;

    /// Identity morphism
    fn identity() -> Box<dyn GraphMorphism>
    where
        Self: Sized,
    {
        Box::new(IdentityMorphism)
    }
}

/// Identity morphism - returns graph unchanged
struct IdentityMorphism;

impl GraphMorphism for IdentityMorphism {
    fn apply(&self, graph: &GraphComposition) -> Result<GraphComposition, CompositionError> {
        Ok(graph.clone())
    }
}

/// Functor trait for structure-preserving maps
pub trait GraphFunctor {
    fn fmap<F>(&self, f: F) -> Result<GraphComposition, CompositionError>
    where
        F: Fn(&CompositionNode) -> CompositionNode;
}

impl GraphFunctor for GraphComposition {
    fn fmap<F>(&self, f: F) -> Result<GraphComposition, CompositionError>
    where
        F: Fn(&CompositionNode) -> CompositionNode,
    {
        self.map(f)
    }
}

/// Monad trait for composition with context
pub trait GraphMonad {
    fn pure(value: CompositionNode) -> GraphComposition;

    fn bind<F>(&self, f: F) -> Result<GraphComposition, CompositionError>
    where
        F: Fn(&CompositionNode) -> GraphComposition;
}

impl GraphMonad for GraphComposition {
    fn pure(value: CompositionNode) -> GraphComposition {
        let mut nodes = HashMap::new();
        let node_id = value.id;
        nodes.insert(node_id, value);

        Self {
            id: GraphId::new(),
            composition_root: node_id,
            composition_type: CompositionType::Monad {
                context_type: "Pure".to_string(),
            },
            nodes,
            edges: HashMap::new(),
            metadata: Default::default(),
        }
    }

    fn bind<F>(&self, f: F) -> Result<GraphComposition, CompositionError>
    where
        F: Fn(&CompositionNode) -> GraphComposition,
    {
        let mut result = GraphComposition::composite("Bind");

        // Apply f to each node and collect results
        for (_, node) in &self.nodes {
            let node_result = f(node);

            // Add all nodes from result
            for (_, n) in node_result.nodes {
                result.nodes.insert(n.id, n);
            }

            // Add all edges from result
            for (_, e) in node_result.edges {
                result.edges.insert(e.id, e);
            }

            // Connect to result
            let edge = CompositionEdge::new(
                node.id,
                node_result.composition_root,
                RelationshipType::Transforms,
            );
            result.edges.insert(edge.id, edge);
        }

        Ok(result)
    }
}

/// Helper function to create a line item graph
pub fn line_item_graph(product: &str, quantity: i32, price: f64) -> GraphComposition {
    GraphComposition::composite("LineItem")
        .add_node("product", serde_json::json!({ "name": product }))
        .add_node("quantity", quantity)
        .add_node("price", price)
        .add_node("total", quantity as f64 * price)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test creating an atomic graph
    ///
    /// ```mermaid
    /// graph LR
    ///     A[Money: $100 USD]
    /// ```
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

    /// Test creating a composite graph
    ///
    /// ```mermaid
    /// graph TD
    ///     Root[Address Root]
    ///     Street[street: 123 Main St]
    ///     City[city: Springfield]
    ///     Zip[zip: 12345]
    ///
    ///     Root --> Street
    ///     Root --> City
    ///     Root --> Zip
    ///     Street -.->|PartOf| City
    ///     City -.->|Contains| Zip
    /// ```
    #[test]
    fn test_composite_graph_creation() {
        let address = GraphComposition::composite("Address")
            .add_node("street", "123 Main St")
            .add_node("city", "Springfield")
            .add_node("zip", "12345")
            .add_edge("root", "street", RelationshipType::Contains)
            .add_edge("root", "city", RelationshipType::Contains)
            .add_edge("root", "zip", RelationshipType::Contains)
            .add_edge("street", "city", RelatedBy::PartOf)
            .add_edge("city", "zip", RelationshipType::Contains);

        assert_eq!(address.nodes.len(), 4); // root + 3 nodes
        assert_eq!(address.edges.len(), 5);
        assert!(matches!(
            address.composition_type,
            CompositionType::Composite { .. }
        ));
    }

    /// Test entity creation
    ///
    /// ```mermaid
    /// graph TD
    ///     User[User Entity: user-123]
    /// ```
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

    /// Test sequential composition
    ///
    /// ```mermaid
    /// graph LR
    ///     A[Validate Order] --> B[Calculate Pricing]
    ///     B --> C[Check Inventory]
    /// ```
    #[test]
    fn test_sequential_composition() {
        let validate = GraphComposition::composite("ValidateOrder");
        let calculate = GraphComposition::composite("CalculatePricing");

        let workflow = validate.then(&calculate).unwrap();

        // Should have nodes from both graphs plus connections
        assert!(workflow.nodes.len() >= 2);
        assert!(workflow.edges.len() >= 1); // At least one sequence edge
    }

    /// Test parallel composition
    ///
    /// ```mermaid
    /// graph TD
    ///     Root[Parallel Root]
    ///     A[Check Inventory]
    ///     B[Verify Payment]
    ///
    ///     Root ==>|parallel| A
    ///     Root ==>|parallel| B
    /// ```
    #[test]
    fn test_parallel_composition() {
        let check_inventory = GraphComposition::composite("CheckInventory");
        let verify_payment = GraphComposition::composite("VerifyPayment");

        let parallel = check_inventory.parallel(&verify_payment).unwrap();

        // Should have a new root connected to both subgraphs
        assert!(parallel.nodes.len() >= 3); // new root + 2 subgraph roots
        assert!(parallel.edges.len() >= 2); // 2 parallel edges
    }

    /// Test choice composition
    ///
    /// ```mermaid
    /// graph TD
    ///     Root[Choice Root]
    ///     A[Fulfill Order]
    ///     B[Backorder Items]
    ///
    ///     Root -->|choice| A
    ///     Root -->|choice| B
    /// ```
    #[test]
    fn test_choice_composition() {
        let fulfill = GraphComposition::composite("FulfillOrder");
        let backorder = GraphComposition::composite("BackorderItems");

        let choice = fulfill.choice(&backorder).unwrap();

        // Should have a new root with choice edges to both options
        assert!(choice.nodes.len() >= 3);
        assert!(choice.edges.len() >= 2);

        // Check that edges are choice type
        let choice_edges: Vec<_> = choice
            .edges
            .values()
            .filter(|e| e.relationship.relationship_type == RelationshipType::Choice)
            .collect();
        assert_eq!(choice_edges.len(), 2);
    }

    /// Test map operation
    ///
    /// ```mermaid
    /// graph LR
    ///     subgraph Before
    ///         A1[Item: $10]
    ///         A2[Item: $20]
    ///     end
    ///
    ///     subgraph After Map
    ///         B1[Item: $11]
    ///         B2[Item: $22]
    ///     end
    ///
    ///     A1 -.->|map +10%| B1
    ///     A2 -.->|map +10%| B2
    /// ```
    #[test]
    fn test_map_operation() {
        let items = GraphComposition::composite("Items")
            .add_node("item1", serde_json::json!({ "price": 10.0 }))
            .add_node("item2", serde_json::json!({ "price": 20.0 }));

        let with_tax = items
            .map(|node| {
                let mut new_node = node.clone();
                if let JsonValue::Object(ref mut map) = new_node.data {
                    if let Some(JsonValue::Number(price)) = map.get("price") {
                        if let Some(p) = price.as_f64() {
                            map.insert("price".to_string(), serde_json::json!(p * 1.1));
                        }
                    }
                }
                new_node
            })
            .unwrap();

        // Check that prices were updated
        for node in with_tax.nodes.values() {
            if let JsonValue::Object(ref map) = node.data {
                if let Some(JsonValue::Number(price)) = map.get("price") {
                    let p = price.as_f64().unwrap();
                    assert!(p == 11.0 || p == 22.0 || p == 0.0); // 0.0 for root
                }
            }
        }
    }

    /// Test fold operation
    #[test]
    fn test_fold_operation() {
        let order = GraphComposition::composite("Order")
            .add_node("item1", serde_json::json!({ "value": 50.0 }))
            .add_node("item2", serde_json::json!({ "value": 75.0 }));

        let total = order.total_value();
        assert_eq!(total, 125.0);
    }
}
