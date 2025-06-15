//! Graph factory implementations
//!
//! Factories create properly configured aggregates with all their
//! dependencies injected.

use crate::contexts::graph::domain::commands::{CreateGraph, GraphFactory};
use crate::contexts::graph::domain::invariants::{DefaultValidatorFactory, ValidatorFactory};
use crate::contexts::graph::domain::{
    ContextGraph,
    context_graph::{InvariantValidator, PositionCalculator},
};
use crate::shared::types::Result;
use std::sync::Arc;

/// Default graph factory implementation
pub struct DefaultGraphFactory {
    validator_factory: Arc<dyn ValidatorFactory>,
    position_calculator: Arc<dyn PositionCalculator>,
}

impl DefaultGraphFactory {
    pub fn new(
        validator_factory: Arc<dyn ValidatorFactory>,
        position_calculator: Arc<dyn PositionCalculator>,
    ) -> Self {
        Self {
            validator_factory,
            position_calculator,
        }
    }

    /// Create with default dependencies
    pub fn with_defaults() -> Self {
        use crate::contexts::graph::domain::context_graph::DefaultPositionCalculator;

        Self {
            validator_factory: Arc::new(DefaultValidatorFactory),
            position_calculator: Arc::new(DefaultPositionCalculator),
        }
    }
}

impl GraphFactory for DefaultGraphFactory {
    fn create_graph(&self, command: CreateGraph) -> Result<ContextGraph> {
        // Create appropriate validator based on context type
        let validator = self
            .validator_factory
            .create_validator(&command.context_type);

        // Create a new position calculator instance
        use crate::contexts::graph::domain::context_graph::DefaultPositionCalculator;
        let position_calculator = Box::new(DefaultPositionCalculator);

        // Create the graph with injected dependencies
        let mut graph = ContextGraph::new(
            command.graph_id,
            command.name,
            command.context_type,
            command.root_node_id,
            validator,
            position_calculator,
        )?;

        // Add the root node
        graph.add_node(
            command.root_node_id,
            command.root_node_type,
            std::collections::HashMap::new(),
        )?;

        Ok(graph)
    }
}
