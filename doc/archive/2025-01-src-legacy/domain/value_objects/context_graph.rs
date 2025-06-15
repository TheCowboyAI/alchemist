//! ContextGraph - A graph with a designated ContextRoot entity as semantic anchor
//!
//! This module implements the ContextGraph pattern where every graph has a root entity
//! that defines its boundary and serves as the entry point for all operations.
//! This aligns with DDD principles of bounded contexts and aggregate roots.

use super::*;
use crate::domain::DomainError;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fmt;

/// Represents a composable context graph with a semantic root
pub trait ContextBounded: Sized {
    type Output;

    /// Compose two contexts into a new context
    fn compose(&self, other: &Self) -> Result<Self::Output, ContextError>;

    /// Check if composition respects context boundaries
    fn can_compose_with(&self, other: &Self) -> bool;
}

/// Errors that can occur during context operations
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ContextError {
    #[error("Incompatible context types: {0} and {1}")]
    IncompatibleContexts(String, String),

    #[error("Invalid context: {0}")]
    InvalidContext(String),

    #[error("Context root not found: {0}")]
    ContextRootNotFound(NodeId),

    #[error("Context boundary violation: {0}")]
    BoundaryViolation(String),

    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("Cycle detected in context")]
    CycleDetected,

    #[error("Context root must exist in nodes")]
    InvalidContextRoot,
}

/// Types of context boundaries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContextType {
    /// Bounded Context (top-level domain boundary)
    BoundedContext { domain: String },

    /// Aggregate Context (consistency boundary)
    Aggregate { aggregate_type: String },

    /// Module Context (functional grouping)
    Module { module_name: String },

    /// Service Context (operational boundary)
    Service { service_type: String },

    /// Conceptual Space Context (semantic boundary)
    ConceptualSpace { space_name: String },

    /// Workflow Context (process boundary)
    Workflow { workflow_type: String },
}

impl fmt::Display for ContextType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextType::BoundedContext { domain } => write!(f, "BoundedContext({})", domain),
            ContextType::Aggregate { aggregate_type } => write!(f, "Aggregate({})", aggregate_type),
            ContextType::Module { module_name } => write!(f, "Module({})", module_name),
            ContextType::Service { service_type } => write!(f, "Service({})", service_type),
            ContextType::ConceptualSpace { space_name } => {
                write!(f, "ConceptualSpace({})", space_name)
            }
            ContextType::Workflow { workflow_type } => write!(f, "Workflow({})", workflow_type),
        }
    }
}

/// A node in the context graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub label: String,
    pub data: JsonValue,
    pub metadata: HashMap<String, JsonValue>,
    pub is_context_root: bool,
}

impl ContextNode {
    pub fn new(node_type: NodeType, label: String, data: JsonValue) -> Self {
        Self {
            id: NodeId::new(),
            node_type,
            label,
            data,
            metadata: HashMap::new(),
            is_context_root: false,
        }
    }

    pub fn new_root(node_type: NodeType, label: String, data: JsonValue) -> Self {
        Self {
            id: NodeId::new(),
            node_type,
            label,
            data,
            metadata: HashMap::new(),
            is_context_root: true,
        }
    }

    pub fn with_metadata(mut self, key: String, value: JsonValue) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// An edge in the context graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextEdge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub relationship: EdgeRelationship,
}

impl ContextEdge {
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

/// Metadata for a context graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub tags: Vec<String>,
    pub properties: HashMap<String, JsonValue>,
    pub ubiquitous_language: HashMap<String, String>,
}

impl Default for ContextMetadata {
    fn default() -> Self {
        Self {
            name: "unnamed_context".to_string(),
            description: None,
            version: "1.0.0".to_string(),
            tags: Vec::new(),
            properties: HashMap::new(),
            ubiquitous_language: HashMap::new(),
        }
    }
}

/// The main ContextGraph structure with a semantic root
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextGraph {
    pub id: GraphId,
    pub context_root: NodeId, // The semantic anchor
    pub context_type: ContextType,
    pub nodes: HashMap<NodeId, ContextNode>,
    pub edges: HashMap<EdgeId, ContextEdge>,
    pub metadata: ContextMetadata,
}

impl ContextGraph {
    /// Create a new bounded context
    pub fn new_bounded_context(domain: &str, root_label: &str) -> Result<Self, ContextError> {
        let root_node = ContextNode::new_root(
            NodeType::Aggregate,
            root_label.to_string(),
            serde_json::json!({ "domain": domain }),
        );
        let root_id = root_node.id;

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root_node);

        Ok(Self {
            id: GraphId::new(),
            context_root: root_id,
            context_type: ContextType::BoundedContext {
                domain: domain.to_string(),
            },
            nodes,
            edges: HashMap::new(),
            metadata: ContextMetadata {
                name: domain.to_string(),
                ..Default::default()
            },
        })
    }

    /// Create an aggregate context
    pub fn new_aggregate(
        aggregate_type: &str,
        aggregate_id: impl Into<String>,
    ) -> Result<Self, ContextError> {
        let root_node = ContextNode::new_root(
            NodeType::Aggregate,
            format!("{} Root", aggregate_type),
            serde_json::json!({ "id": aggregate_id.into(), "type": aggregate_type }),
        );
        let root_id = root_node.id;

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root_node);

        Ok(Self {
            id: GraphId::new(),
            context_root: root_id,
            context_type: ContextType::Aggregate {
                aggregate_type: aggregate_type.to_string(),
            },
            nodes,
            edges: HashMap::new(),
            metadata: ContextMetadata {
                name: aggregate_type.to_string(),
                ..Default::default()
            },
        })
    }

    /// Create a module context
    pub fn new_module(module_name: &str, root_label: &str) -> Result<Self, ContextError> {
        let root_node = ContextNode::new_root(
            NodeType::Aggregate,
            root_label.to_string(),
            serde_json::json!({ "module": module_name }),
        );
        let root_id = root_node.id;

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root_node);

        Ok(Self {
            id: GraphId::new(),
            context_root: root_id,
            context_type: ContextType::Module {
                module_name: module_name.to_string(),
            },
            nodes,
            edges: HashMap::new(),
            metadata: ContextMetadata {
                name: module_name.to_string(),
                ..Default::default()
            },
        })
    }

    /// Validate that the context root exists and is valid
    pub fn validate_context_root(&self) -> Result<(), ContextError> {
        if !self.nodes.contains_key(&self.context_root) {
            return Err(ContextError::ContextRootNotFound(self.context_root));
        }

        if let Some(root_node) = self.nodes.get(&self.context_root) {
            if !root_node.is_context_root {
                return Err(ContextError::InvalidContext(
                    "Context root node must be marked as root".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Add an entity related to the context root
    pub fn add_entity_related_to_root(
        &mut self,
        label: &str,
        data: JsonValue,
        relationship: RelationshipType,
    ) -> Result<NodeId, ContextError> {
        self.validate_context_root()?;

        let node = ContextNode::new(NodeType::Entity, label.to_string(), data);
        let node_id = node.id;

        self.nodes.insert(node_id, node);

        // Connect to root
        let edge = ContextEdge::new(self.context_root, node_id, relationship);
        self.edges.insert(edge.id, edge);

        Ok(node_id)
    }

    /// Add a value object related to a parent node
    pub fn add_value_object(
        &mut self,
        parent_id: NodeId,
        label: &str,
        data: JsonValue,
    ) -> Result<NodeId, ContextError> {
        if !self.nodes.contains_key(&parent_id) {
            return Err(ContextError::NodeNotFound(parent_id));
        }

        let node = ContextNode::new(NodeType::ValueObject, label.to_string(), data);
        let node_id = node.id;

        self.nodes.insert(node_id, node);

        // Connect to parent
        let edge = ContextEdge::new(parent_id, node_id, RelationshipType::Contains);
        self.edges.insert(edge.id, edge);

        Ok(node_id)
    }

    /// Add a nested context
    pub fn add_nested_context(&mut self, nested: ContextGraph) -> Result<(), ContextError> {
        self.validate_context_root()?;
        nested.validate_context_root()?;

        // Add all nodes from nested context
        for (id, node) in nested.nodes {
            self.nodes.insert(id, node);
        }

        // Add all edges from nested context
        for (id, edge) in nested.edges {
            self.edges.insert(id, edge);
        }

        // Connect root to nested root
        let edge = ContextEdge::new(
            self.context_root,
            nested.context_root,
            RelationshipType::Contains,
        );
        self.edges.insert(edge.id, edge);

        Ok(())
    }

    /// Get all nodes directly connected to the context root
    pub fn get_root_children(&self) -> Vec<&ContextNode> {
        self.edges
            .values()
            .filter(|e| e.source == self.context_root)
            .filter_map(|e| self.nodes.get(&e.target))
            .collect()
    }

    /// Check if a node is within this context boundary
    pub fn contains_node(&self, node_id: &NodeId) -> bool {
        self.nodes.contains_key(node_id)
    }

    /// Enforce invariants through the context root
    pub fn enforce_invariants(&self) -> Result<(), ContextError> {
        self.validate_context_root()?;

        // Context-specific invariant checks
        match &self.context_type {
            ContextType::Aggregate { .. } => {
                // Aggregate invariants
                self.check_aggregate_invariants()?;
            }
            ContextType::BoundedContext { .. } => {
                // Bounded context invariants
                self.check_bounded_context_invariants()?;
            }
            _ => {}
        }

        Ok(())
    }

    fn check_aggregate_invariants(&self) -> Result<(), ContextError> {
        // Example: All entities must be connected to root
        for node_id in self.nodes.keys() {
            if *node_id != self.context_root {
                let has_path_to_root = self.has_path_to_root(node_id);
                if !has_path_to_root {
                    return Err(ContextError::InvariantViolation(format!(
                        "Node {:?} is not connected to aggregate root",
                        node_id
                    )));
                }
            }
        }
        Ok(())
    }

    fn check_bounded_context_invariants(&self) -> Result<(), ContextError> {
        // Example: No external references
        // This would check that all edges are internal to the context
        Ok(())
    }

    fn has_path_to_root(&self, node_id: &NodeId) -> bool {
        // Simple check: is there any edge that has this node as target
        // and source is either root or has path to root
        if node_id == &self.context_root {
            return true;
        }

        // Find edges pointing to this node
        for edge in self.edges.values() {
            if edge.target == *node_id {
                if edge.source == self.context_root || self.has_path_to_root(&edge.source) {
                    return true;
                }
            }
        }

        false
    }

    /// Add a term to the ubiquitous language
    pub fn define_term(&mut self, term: &str, definition: &str) {
        self.metadata
            .ubiquitous_language
            .insert(term.to_string(), definition.to_string());
    }

    /// Get the ubiquitous language for this context
    pub fn get_ubiquitous_language(&self) -> &HashMap<String, String> {
        &self.metadata.ubiquitous_language
    }
}

impl ContextBounded for ContextGraph {
    type Output = ContextGraph;

    fn compose(&self, other: &Self) -> Result<Self::Output, ContextError> {
        // Contexts can only be composed if they respect boundaries
        if !self.can_compose_with(other) {
            return Err(ContextError::IncompatibleContexts(
                self.context_type.to_string(),
                other.context_type.to_string(),
            ));
        }

        // Create a new context that contains both
        let mut result = ContextGraph::new_bounded_context("ComposedContext", "Composed Root")?;

        // Add both contexts as nested contexts
        result.add_nested_context(self.clone())?;
        result.add_nested_context(other.clone())?;

        Ok(result)
    }

    fn can_compose_with(&self, other: &Self) -> bool {
        // Define composition rules based on context types
        match (&self.context_type, &other.context_type) {
            // Modules can be composed within bounded contexts
            (ContextType::BoundedContext { .. }, ContextType::Module { .. }) => true,
            (ContextType::Module { .. }, ContextType::BoundedContext { .. }) => true,

            // Services can be composed within modules
            (ContextType::Module { .. }, ContextType::Service { .. }) => true,
            (ContextType::Service { .. }, ContextType::Module { .. }) => true,

            // Same types can be composed if they're different instances
            (ContextType::Module { .. }, ContextType::Module { .. }) => self.id != other.id,

            // Aggregates are isolated
            (ContextType::Aggregate { .. }, _) => false,
            (_, ContextType::Aggregate { .. }) => false,

            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_context_creation() {
        let context =
            ContextGraph::new_bounded_context("UserManagement", "User Aggregate Root").unwrap();

        assert_eq!(context.nodes.len(), 1);
        assert!(context.nodes.contains_key(&context.context_root));
        assert!(
            context
                .nodes
                .get(&context.context_root)
                .unwrap()
                .is_context_root
        );
    }

    #[test]
    fn test_add_entity_to_context() {
        let mut context = ContextGraph::new_aggregate("Order", "order-123").unwrap();

        let item_id = context
            .add_entity_related_to_root(
                "OrderItem",
                serde_json::json!({ "product": "Widget", "quantity": 2 }),
                RelationshipType::Contains,
            )
            .unwrap();

        assert_eq!(context.nodes.len(), 2);
        assert!(context.contains_node(&item_id));
        assert_eq!(context.edges.len(), 1);
    }

    #[test]
    fn test_context_invariants() {
        let mut context = ContextGraph::new_aggregate("Order", "order-123").unwrap();

        // Add a disconnected node (violates invariant)
        let orphan = ContextNode::new(
            NodeType::Entity,
            "Orphan".to_string(),
            serde_json::json!({}),
        );
        context.nodes.insert(orphan.id, orphan);

        // Should fail invariant check
        let result = context.enforce_invariants();
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_contexts() {
        let mut user_context =
            ContextGraph::new_bounded_context("UserManagement", "User Root").unwrap();

        let auth_module = ContextGraph::new_module("Authentication", "Auth Root").unwrap();

        user_context.add_nested_context(auth_module).unwrap();

        assert_eq!(user_context.nodes.len(), 2);
        assert_eq!(user_context.edges.len(), 1);
    }

    #[test]
    fn test_ubiquitous_language() {
        let mut context = ContextGraph::new_bounded_context("Inventory", "Inventory Root").unwrap();

        context.define_term("SKU", "Stock Keeping Unit - unique product identifier");
        context.define_term("Reorder Point", "Minimum quantity before reordering");

        assert_eq!(context.get_ubiquitous_language().len(), 2);
        assert_eq!(
            context.get_ubiquitous_language().get("SKU"),
            Some(&"Stock Keeping Unit - unique product identifier".to_string())
        );
    }
}
