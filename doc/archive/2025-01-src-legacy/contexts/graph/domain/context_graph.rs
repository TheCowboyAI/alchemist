//! ContextGraph Aggregate - Core domain model
//!
//! This is the aggregate root for the graph context. It wraps the GraphComposition
//! from the graph-composition library and adds domain-specific behavior.

use crate::shared::types::{Result, Error};
use crate::shared::events::{EventMetadata, DomainEvent};
use graph_composition::{
    GraphComposition, GraphId, NodeId, EdgeId,
    CompositionType, DomainCompositionType,
    BaseNodeType, BaseRelationshipType,
    CompositionNode, CompositionEdge,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// The type of context this graph represents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContextType {
    /// A bounded context in DDD
    BoundedContext {
        name: String,
        domain: String
    },

    /// An aggregate context (consistency boundary)
    AggregateContext {
        name: String,
        aggregate_type: String
    },

    /// A module context (functional grouping)
    ModuleContext {
        name: String,
        purpose: String
    },

    /// A service context (capability boundary)
    ServiceContext {
        name: String,
        capability: String
    },
}

impl From<ContextType> for CompositionType {
    fn from(context_type: ContextType) -> Self {
        match context_type {
            ContextType::BoundedContext { name, domain } => {
                CompositionType::Domain(DomainCompositionType::BoundedContext { domain })
            }
            ContextType::AggregateContext { name, aggregate_type } => {
                CompositionType::Domain(DomainCompositionType::Aggregate { aggregate_type })
            }
            ContextType::ModuleContext { name, purpose } => {
                CompositionType::Composite { structure_type: format!("Module:{}", purpose) }
            }
            ContextType::ServiceContext { name, capability } => {
                CompositionType::Domain(DomainCompositionType::Service { service_type: capability })
            }
        }
    }
}

/// Trait for validating graph invariants
pub trait InvariantValidator: Send + Sync {
    /// Validate that a node can be added
    fn validate_node_addition(&self, graph: &ContextGraph, node_id: &NodeId) -> Result<()>;

    /// Validate that an edge can be created
    fn validate_edge_creation(&self, graph: &ContextGraph, source: &NodeId, target: &NodeId) -> Result<()>;

    /// Validate the context root
    fn validate_context_root(&self, graph: &ContextGraph) -> Result<()>;
}

/// Trait for calculating node positions
pub trait PositionCalculator: Send + Sync {
    /// Calculate optimal position for a new node
    fn calculate_position(&self, graph: &ContextGraph, node_id: &NodeId) -> Result<(f32, f32, f32)>;
}

/// The ContextGraph aggregate root - wraps GraphComposition with domain behavior
pub struct ContextGraph {
    // The underlying graph composition
    graph: GraphComposition<BaseNodeType, BaseRelationshipType>,

    // Domain-specific metadata
    version: u64,
    context_type: ContextType,
    ubiquitous_language: HashMap<String, String>,

    // Injected dependencies
    invariant_validator: Box<dyn InvariantValidator>,
    position_calculator: Box<dyn PositionCalculator>,
}

impl ContextGraph {
    /// Create a new ContextGraph with injected dependencies
    pub fn new(
        id: GraphId,
        name: String,
        context_type: ContextType,
        context_root: NodeId,
        invariant_validator: Box<dyn InvariantValidator>,
        position_calculator: Box<dyn PositionCalculator>,
    ) -> Result<Self> {
        // Create the underlying graph composition
        let composition_type = context_type.clone().into();
        let mut graph = GraphComposition::new(BaseNodeType::Aggregate, composition_type);

        // Set the ID and metadata
        graph.id = id;
        graph.composition_root = context_root;
        graph.metadata.name = name;

        let context_graph = Self {
            graph,
            version: 0,
            context_type,
            ubiquitous_language: HashMap::new(),
            invariant_validator,
            position_calculator,
        };

        // Validate initial state
        context_graph.invariant_validator.validate_context_root(&context_graph)?;

        Ok(context_graph)
    }

    /// Get the graph ID
    pub fn id(&self) -> GraphId {
        self.graph.id
    }

    /// Get the current version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Get the context type
    pub fn context_type(&self) -> &ContextType {
        &self.context_type
    }

    /// Get the context root node ID
    pub fn context_root(&self) -> NodeId {
        self.graph.composition_root
    }

    /// Process a command and return events
    pub fn handle_command(&mut self, command: impl Command) -> Result<Vec<Box<dyn DomainEvent>>> {
        command.execute(self)
    }

    /// Apply an event to update state
    pub fn apply_event(&mut self, event: &dyn DomainEvent) -> Result<()> {
        // This will be implemented by each specific event type
        self.version += 1;
        Ok(())
    }

    /// Add a node (called by commands)
    pub(crate) fn add_node(&mut self, id: NodeId, node_type: String, metadata: HashMap<String, serde_json::Value>) -> Result<()> {
        self.invariant_validator.validate_node_addition(self, &id)?;

        // Convert to BaseNodeType
        let base_type = match node_type.as_str() {
            "Aggregate" => BaseNodeType::Aggregate,
            "Service" => BaseNodeType::Service,
            "Command" => BaseNodeType::Command,
            "Event" => BaseNodeType::Event,
            "Value" => BaseNodeType::Value,
            "EntityReference" => BaseNodeType::EntityReference,
            _ => BaseNodeType::Custom(node_type.clone()),
        };

        // Add to the underlying graph
        self.graph = self.graph.clone().add_node_with_id(
            id,
            base_type,
            &node_type,
            serde_json::json!(metadata),
        );

        Ok(())
    }

    /// Add an edge (called by commands)
    pub(crate) fn add_edge(&mut self, id: EdgeId, source: NodeId, target: NodeId, edge_type: String, metadata: HashMap<String, serde_json::Value>) -> Result<()> {
        self.invariant_validator.validate_edge_creation(self, &source, &target)?;

        // Convert to BaseRelationshipType
        let base_rel = match edge_type.as_str() {
            "Contains" => BaseRelationshipType::Contains,
            "References" => BaseRelationshipType::References,
            "DependsOn" => BaseRelationshipType::DependsOn,
            "Sequence" => BaseRelationshipType::Sequence,
            "Parallel" => BaseRelationshipType::Parallel,
            "Choice" => BaseRelationshipType::Choice,
            _ => BaseRelationshipType::Custom(edge_type),
        };

        // Add to the underlying graph
        self.graph = self.graph.clone().add_edge(source, target, base_rel);

        Ok(())
    }

    /// Check if a node exists
    pub fn has_node(&self, node_id: &NodeId) -> bool {
        self.graph.nodes.contains_key(node_id)
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.graph.nodes.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.graph.edges.len()
    }
}

/// Trait for commands that can be executed on a ContextGraph
pub trait Command: Send + Sync {
    /// Execute the command and return events
    fn execute(&self, graph: &mut ContextGraph) -> Result<Vec<Box<dyn DomainEvent>>>;
}

/// Default invariant validator implementation
pub struct DefaultInvariantValidator;

impl InvariantValidator for DefaultInvariantValidator {
    fn validate_node_addition(&self, _graph: &ContextGraph, _node_id: &NodeId) -> Result<()> {
        // Basic validation - can be extended
        Ok(())
    }

    fn validate_edge_creation(&self, graph: &ContextGraph, source: &NodeId, target: &NodeId) -> Result<()> {
        // Ensure both nodes exist
        if !graph.has_node(source) {
            return Err(Error::NotFound(format!("Source node {} not found", source)));
        }
        if !graph.has_node(target) {
            return Err(Error::NotFound(format!("Target node {} not found", target)));
        }
        Ok(())
    }

    fn validate_context_root(&self, graph: &ContextGraph) -> Result<()> {
        // Context root validation will be implemented based on context type
        match &graph.context_type {
            ContextType::BoundedContext { .. } => {
                // Bounded contexts have specific root requirements
                Ok(())
            }
            _ => Ok(())
        }
    }
}

/// Default position calculator implementation
pub struct DefaultPositionCalculator;

impl PositionCalculator for DefaultPositionCalculator {
    fn calculate_position(&self, graph: &ContextGraph, _node_id: &NodeId) -> Result<(f32, f32, f32)> {
        // Simple grid layout based on node count
        let count = graph.node_count() as f32;
        let x = (count % 10.0) * 100.0;
        let y = (count / 10.0).floor() * 100.0;
        Ok((x, y, 0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_graph_creation() {
        let graph_id = GraphId::new();
        let root_id = NodeId::new();

        let graph = ContextGraph::new(
            graph_id,
            "Test Graph".to_string(),
            ContextType::BoundedContext {
                name: "TestContext".to_string(),
                domain: "Testing".to_string(),
            },
            root_id,
            Box::new(DefaultInvariantValidator),
            Box::new(DefaultPositionCalculator),
        ).unwrap();

        assert_eq!(graph.id(), graph_id);
        assert_eq!(graph.version(), 0);
        assert_eq!(graph.context_root(), root_id);
    }

    #[test]
    fn test_dependency_injection() {
        // Custom validator that always fails
        struct StrictValidator;
        impl InvariantValidator for StrictValidator {
            fn validate_node_addition(&self, _: &ContextGraph, _: &NodeId) -> Result<()> {
                Err(Error::InvariantViolation("No nodes allowed".to_string()))
            }
            fn validate_edge_creation(&self, _: &ContextGraph, _: &NodeId, _: &NodeId) -> Result<()> {
                Err(Error::InvariantViolation("No edges allowed".to_string()))
            }
            fn validate_context_root(&self, _: &ContextGraph) -> Result<()> {
                Ok(())
            }
        }

        let mut graph = ContextGraph::new(
            GraphId::new(),
            "Strict Graph".to_string(),
            ContextType::ModuleContext {
                name: "StrictModule".to_string(),
                purpose: "Testing".to_string(),
            },
            NodeId::new(),
            Box::new(StrictValidator),
            Box::new(DefaultPositionCalculator),
        ).unwrap();

        // Should fail due to injected validator
        let result = graph.add_node(NodeId::new(), "TestNode".to_string(), HashMap::new());
        assert!(result.is_err());
    }
}
