//! Graph invariant validators
//!
//! This module contains specific invariant validators that can be injected
//! into ContextGraph instances based on their type and requirements.

use crate::contexts::graph::domain::context_graph::{
    ContextGraph, ContextType, InvariantValidator,
};
use crate::shared::types::{Error, NodeId, Result};

/// Bounded Context invariant validator
pub struct BoundedContextValidator {
    max_nodes: usize,
    max_edges: usize,
}

impl BoundedContextValidator {
    pub fn new(max_nodes: usize, max_edges: usize) -> Self {
        Self {
            max_nodes,
            max_edges,
        }
    }
}

impl InvariantValidator for BoundedContextValidator {
    fn validate_node_addition(&self, graph: &ContextGraph, _node_id: &NodeId) -> Result<()> {
        if graph.node_count() >= self.max_nodes {
            return Err(Error::InvariantViolation(format!(
                "Bounded context cannot have more than {} nodes",
                self.max_nodes
            )));
        }
        Ok(())
    }

    fn validate_edge_creation(
        &self,
        graph: &ContextGraph,
        source: &NodeId,
        target: &NodeId,
    ) -> Result<()> {
        // Check nodes exist
        if !graph.has_node(source) {
            return Err(Error::NotFound(format!("Source node {} not found", source)));
        }
        if !graph.has_node(target) {
            return Err(Error::NotFound(format!("Target node {} not found", target)));
        }

        // Check edge limit
        if graph.edge_count() >= self.max_edges {
            return Err(Error::InvariantViolation(format!(
                "Bounded context cannot have more than {} edges",
                self.max_edges
            )));
        }

        Ok(())
    }

    fn validate_context_root(&self, graph: &ContextGraph) -> Result<()> {
        match graph.context_type() {
            ContextType::BoundedContext { name, .. } => {
                if name.is_empty() {
                    return Err(Error::InvariantViolation(
                        "Bounded context must have a name".to_string(),
                    ));
                }
                Ok(())
            }
            _ => Err(Error::InvariantViolation(
                "This validator is only for bounded contexts".to_string(),
            )),
        }
    }
}

/// Aggregate Context invariant validator
pub struct AggregateContextValidator {
    allowed_node_types: Vec<String>,
}

impl AggregateContextValidator {
    pub fn new(allowed_node_types: Vec<String>) -> Self {
        Self { allowed_node_types }
    }
}

impl InvariantValidator for AggregateContextValidator {
    fn validate_node_addition(&self, _graph: &ContextGraph, _node_id: &NodeId) -> Result<()> {
        // In a real implementation, we would check the node type
        // against the allowed types for this aggregate
        Ok(())
    }

    fn validate_edge_creation(
        &self,
        graph: &ContextGraph,
        source: &NodeId,
        target: &NodeId,
    ) -> Result<()> {
        // Ensure both nodes exist
        if !graph.has_node(source) || !graph.has_node(target) {
            return Err(Error::NotFound("One or both nodes not found".to_string()));
        }

        // Aggregate contexts might have specific rules about edge types
        Ok(())
    }

    fn validate_context_root(&self, graph: &ContextGraph) -> Result<()> {
        match graph.context_type() {
            ContextType::AggregateContext { aggregate_type, .. } => {
                if aggregate_type.is_empty() {
                    return Err(Error::InvariantViolation(
                        "Aggregate context must specify aggregate type".to_string(),
                    ));
                }
                Ok(())
            }
            _ => Err(Error::InvariantViolation(
                "This validator is only for aggregate contexts".to_string(),
            )),
        }
    }
}

/// Factory for creating appropriate validators based on context type
pub trait ValidatorFactory: Send + Sync {
    /// Create a validator for the given context type
    fn create_validator(&self, context_type: &ContextType) -> Box<dyn InvariantValidator>;
}

/// Default validator factory implementation
pub struct DefaultValidatorFactory;

impl ValidatorFactory for DefaultValidatorFactory {
    fn create_validator(&self, context_type: &ContextType) -> Box<dyn InvariantValidator> {
        match context_type {
            ContextType::BoundedContext { .. } => {
                Box::new(BoundedContextValidator::new(1000, 5000))
            }
            ContextType::AggregateContext { .. } => {
                Box::new(AggregateContextValidator::new(vec![]))
            }
            _ => Box::new(crate::contexts::graph::domain::context_graph::DefaultInvariantValidator),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_context_validator() {
        let validator = BoundedContextValidator::new(2, 1);

        // Create a mock graph (in real tests we'd use a proper mock)
        // For now, we just test the validator logic

        // Test that validator enforces limits
        // This would be tested with actual graph instances
    }
}
