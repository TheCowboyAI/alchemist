//! Workflow graph implementation for CIM workflows
//!
//! This module provides a graph structure for workflows using the simple
//! workflow types from cim-domain-workflow.

use cim_domain::GraphId;
use cim_domain_workflow::{
    StateId, StepId, TransitionInput, TransitionOutput, Workflow, WorkflowContext, WorkflowId,
    WorkflowState, WorkflowStatus, WorkflowStep, WorkflowTransition,
};
use petgraph::stable_graph::{EdgeIndex, NodeIndex, StableGraph};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

/// Type of workflow execution pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowType {
    /// Sequential workflow - one path at a time
    Sequential,
    /// Parallel workflow - multiple paths can execute simultaneously
    Parallel,
    /// State machine workflow - complex state transitions
    StateMachine,
    /// Event-driven workflow - triggered by events
    EventDriven,
}

/// A simple workflow graph using WorkflowStep as nodes
pub struct WorkflowGraph {
    pub id: GraphId,
    pub workflow_id: WorkflowId,
    pub graph: StableGraph<WorkflowStep, String>, // String for edge labels
    pub workflow_type: WorkflowType,

    // Index structures for efficient lookups
    step_to_node: HashMap<StepId, NodeIndex>,
    node_to_step: HashMap<NodeIndex, StepId>,
}

impl WorkflowGraph {
    /// Create a new workflow graph
    pub fn new(workflow_id: WorkflowId, workflow_type: WorkflowType) -> Self {
        Self {
            id: GraphId::new(),
            workflow_id,
            graph: StableGraph::new(),
            workflow_type,
            step_to_node: HashMap::new(),
            node_to_step: HashMap::new(),
        }
    }

    /// Create a workflow graph from a Workflow
    pub fn from_workflow(workflow: &Workflow) -> Self {
        let mut graph = Self::new(workflow.id, WorkflowType::Sequential);

        // Add all steps as nodes
        for step in &workflow.steps {
            graph.add_step(step.clone());
        }

        // For sequential workflows, connect steps in order
        if workflow.steps.len() > 1 {
            for i in 0..workflow.steps.len() - 1 {
                let source = workflow.steps[i].id;
                let target = workflow.steps[i + 1].id;
                let _ = graph.add_connection(source, target, "next".to_string());
            }
        }

        graph
    }

    /// Add a step to the workflow graph
    pub fn add_step(&mut self, step: WorkflowStep) -> NodeIndex {
        let step_id = step.id;

        // Check if step already exists
        if let Some(&node_idx) = self.step_to_node.get(&step_id) {
            return node_idx;
        }

        // Add new step
        let node_idx = self.graph.add_node(step);
        self.step_to_node.insert(step_id, node_idx);
        self.node_to_step.insert(node_idx, step_id);

        node_idx
    }

    /// Add a connection between steps
    pub fn add_connection(
        &mut self,
        source_step: StepId,
        target_step: StepId,
        label: String,
    ) -> Result<EdgeIndex, WorkflowGraphError> {
        // Ensure both steps exist
        let source_idx = self
            .step_to_node
            .get(&source_step)
            .ok_or(WorkflowGraphError::StepNotFound(source_step))?;
        let target_idx = self
            .step_to_node
            .get(&target_step)
            .ok_or(WorkflowGraphError::StepNotFound(target_step))?;

        // Add the connection
        let edge_idx = self.graph.add_edge(*source_idx, *target_idx, label);
        Ok(edge_idx)
    }

    /// Get a step by its ID
    pub fn get_step(&self, step_id: &StepId) -> Option<&WorkflowStep> {
        self.step_to_node
            .get(step_id)
            .and_then(|&idx| self.graph.node_weight(idx))
    }

    /// Get all steps in the workflow
    pub fn steps(&self) -> impl Iterator<Item = &WorkflowStep> {
        self.graph.node_weights()
    }

    /// Get the number of steps
    pub fn step_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get the number of connections
    pub fn connection_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Find initial steps (steps with no incoming edges)
    pub fn initial_steps(&self) -> Vec<&WorkflowStep> {
        self.graph
            .node_indices()
            .filter(|&idx| {
                self.graph
                    .edges_directed(idx, petgraph::Direction::Incoming)
                    .count()
                    == 0
            })
            .filter_map(|idx| self.graph.node_weight(idx))
            .collect()
    }

    /// Find terminal steps (steps with no outgoing edges)
    pub fn terminal_steps(&self) -> Vec<&WorkflowStep> {
        self.graph
            .node_indices()
            .filter(|&idx| {
                self.graph
                    .edges_directed(idx, petgraph::Direction::Outgoing)
                    .count()
                    == 0
            })
            .filter_map(|idx| self.graph.node_weight(idx))
            .collect()
    }

    /// Get the next steps from a given step
    pub fn next_steps(&self, step_id: &StepId) -> Vec<&WorkflowStep> {
        if let Some(&node_idx) = self.step_to_node.get(step_id) {
            self.graph
                .edges_directed(node_idx, petgraph::Direction::Outgoing)
                .filter_map(|edge| self.graph.node_weight(edge.target()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get the previous steps from a given step
    pub fn previous_steps(&self, step_id: &StepId) -> Vec<&WorkflowStep> {
        if let Some(&node_idx) = self.step_to_node.get(step_id) {
            self.graph
                .edges_directed(node_idx, petgraph::Direction::Incoming)
                .filter_map(|edge| self.graph.node_weight(edge.source()))
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// Errors that can occur when working with workflow graphs
#[derive(Debug, thiserror::Error)]
pub enum WorkflowGraphError {
    #[error("Step not found: {0:?}")]
    StepNotFound(StepId),

    #[error("No initial step found in workflow")]
    NoInitialStep,

    #[error("No terminal step found in workflow")]
    NoTerminalStep,

    #[error("Invalid connection: {0}")]
    InvalidConnection(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use cim_domain_workflow::Workflow;
    use std::collections::HashMap;

    #[test]
    fn test_workflow_graph_creation() {
        let workflow_id = WorkflowId::new();
        let graph = WorkflowGraph::new(workflow_id, WorkflowType::Sequential);

        assert_eq!(graph.workflow_id, workflow_id);
        assert_eq!(graph.step_count(), 0);
        assert_eq!(graph.connection_count(), 0);
    }

    #[test]
    fn test_from_workflow() {
        let mut workflow =
            Workflow::new("Test Workflow".to_string(), "A test workflow".to_string());

        // Add some steps
        let step1 = cim_domain_workflow::WorkflowStep {
            id: StepId::new(),
            name: "Step 1".to_string(),
            description: "First step".to_string(),
            step_type: "manual".to_string(),
            config: HashMap::new(),
        };

        let step2 = cim_domain_workflow::WorkflowStep {
            id: StepId::new(),
            name: "Step 2".to_string(),
            description: "Second step".to_string(),
            step_type: "automated".to_string(),
            config: HashMap::new(),
        };

        workflow.add_step(step1);
        workflow.add_step(step2);

        let graph = WorkflowGraph::from_workflow(&workflow);

        assert_eq!(graph.step_count(), 2);
        assert_eq!(graph.connection_count(), 1); // Sequential connection
        assert_eq!(graph.initial_steps().len(), 1);
        assert_eq!(graph.terminal_steps().len(), 1);
    }
}
