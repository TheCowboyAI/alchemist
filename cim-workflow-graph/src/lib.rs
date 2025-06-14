//! Workflow graph implementation for category theory-based workflows
//!
//! This module provides a graph structure specifically designed for workflows where:
//! - States are nodes (objects in the category)
//! - Transitions are edges (morphisms in the category)
//! - Enrichment values capture business semantics (costs, time, value)

use cim_domain::{GraphId, StateId};
use cim_domain_workflow::{
    WorkflowState, WorkflowTransition, TransitionInput, TransitionOutput, WorkflowContext,
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

/// Type of enrichment value used in the workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnrichmentType {
    /// Business value (monetary, time, risk)
    BusinessValue,
    /// Performance metrics (latency, throughput)
    Performance,
    /// Quality metrics (accuracy, completeness)
    Quality,
    /// Custom domain-specific metrics
    Custom,
}

/// Trait for enrichment values attached to transitions
pub trait EnrichmentValue: Clone + Debug + Send + Sync {
    /// Combine two enrichment values (for path composition)
    fn combine(&self, other: &Self) -> Self;

    /// Identity element for combination
    fn identity() -> Self;

    /// Compare enrichment values (for optimization)
    fn is_better_than(&self, other: &Self) -> bool;
}

/// Metadata about a workflow graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub workflow_type: WorkflowType,
    pub enrichment_type: EnrichmentType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
}

/// A workflow graph with states, transitions, and enrichment values
pub struct WorkflowGraph<S, I, O, V>
where
    S: WorkflowState,
    I: TransitionInput,
    O: TransitionOutput,
    V: EnrichmentValue,
{
    pub id: GraphId,
    pub graph: StableGraph<S, Box<dyn WorkflowTransition<S, I, O>>>,
    pub enrichment: HashMap<EdgeIndex, V>,
    pub metadata: WorkflowMetadata,

    // Index structures for efficient lookups
    state_to_node: HashMap<StateId, NodeIndex>,
    node_to_state: HashMap<NodeIndex, StateId>,

    // Phantom data to satisfy type parameters
    _phantom: PhantomData<(I, O)>,
}

impl<S, I, O, V> WorkflowGraph<S, I, O, V>
where
    S: WorkflowState,
    I: TransitionInput,
    O: TransitionOutput,
    V: EnrichmentValue,
{
    /// Create a new workflow graph
    pub fn new(id: GraphId, workflow_type: WorkflowType) -> Self {
        Self::with_metadata(
            id,
            WorkflowMetadata {
                name: "Workflow".to_string(),
                description: String::new(),
                version: "1.0.0".to_string(),
                workflow_type,
                enrichment_type: EnrichmentType::BusinessValue,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                tags: Vec::new(),
            },
        )
    }

    /// Create a new workflow graph with full metadata
    pub fn with_metadata(id: GraphId, metadata: WorkflowMetadata) -> Self {
        Self {
            id,
            graph: StableGraph::new(),
            enrichment: HashMap::new(),
            metadata,
            state_to_node: HashMap::new(),
            node_to_state: HashMap::new(),
            _phantom: PhantomData,
        }
    }

    /// Create a new workflow graph with name
    pub fn new_with_name(name: String, workflow_type: WorkflowType, enrichment_type: EnrichmentType) -> Self {
        Self {
            id: GraphId::new(),
            graph: StableGraph::new(),
            enrichment: HashMap::new(),
            metadata: WorkflowMetadata {
                name,
                description: String::new(),
                version: "1.0.0".to_string(),
                workflow_type,
                enrichment_type,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                tags: Vec::new(),
            },
            state_to_node: HashMap::new(),
            node_to_state: HashMap::new(),
            _phantom: PhantomData,
        }
    }

    /// Add a state to the workflow graph
    pub fn add_state(&mut self, state: S) -> NodeIndex {
        let state_id = state.id();

        // Check if state already exists
        if let Some(&node_idx) = self.state_to_node.get(&state_id) {
            return node_idx;
        }

        // Add new state
        let node_idx = self.graph.add_node(state);
        self.state_to_node.insert(state_id.clone(), node_idx);
        self.node_to_state.insert(node_idx, state_id);

        self.metadata.updated_at = chrono::Utc::now();
        node_idx
    }

    /// Add a transition between states
    pub fn add_transition(
        &mut self,
        transition: Box<dyn WorkflowTransition<S, I, O>>,
        enrichment: V,
    ) -> Result<EdgeIndex, WorkflowGraphError> {
        let source_id = transition.source().id();
        let target_id = transition.target().id();

        // Ensure both states exist
        let source_idx = self.state_to_node.get(&source_id)
            .ok_or(WorkflowGraphError::StateNotFound(source_id.clone()))?;
        let target_idx = self.state_to_node.get(&target_id)
            .ok_or(WorkflowGraphError::StateNotFound(target_id))?;

        // Add the transition
        let edge_idx = self.graph.add_edge(*source_idx, *target_idx, transition);
        self.enrichment.insert(edge_idx, enrichment);

        self.metadata.updated_at = chrono::Utc::now();
        Ok(edge_idx)
    }

    /// Find applicable transitions from a given state with a specific input
    pub fn find_transitions(
        &self,
        state: &S,
        input: &I,
        context: &WorkflowContext,
    ) -> Vec<(&Box<dyn WorkflowTransition<S, I, O>>, &V, EdgeIndex)> {
        let state_id = state.id();

        if let Some(&node_idx) = self.state_to_node.get(&state_id) {
            self.graph
                .edges(node_idx)
                .filter_map(|edge| {
                    let transition = edge.weight();

                    // Check if transition accepts this input and guard passes
                    if transition.accepts_input(input) && transition.guard(context) {
                        let enrichment = self.enrichment.get(&edge.id())?;
                        Some((transition, enrichment, edge.id()))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Find the optimal transition based on enrichment values
    pub fn find_optimal_transition(
        &self,
        state: &S,
        input: &I,
        context: &WorkflowContext,
    ) -> Option<(&Box<dyn WorkflowTransition<S, I, O>>, &V, EdgeIndex)> {
        let transitions = self.find_transitions(state, input, context);

        transitions.into_iter()
            .min_by(|(_, v1, _), (_, v2, _)| {
                if v1.is_better_than(v2) {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            })
    }

    /// Get a state by its ID
    pub fn get_state(&self, state_id: &StateId) -> Option<&S> {
        self.state_to_node.get(state_id)
            .and_then(|&idx| self.graph.node_weight(idx))
    }

    /// Get all states in the workflow
    pub fn states(&self) -> impl Iterator<Item = &S> {
        self.graph.node_weights()
    }

    /// Get all transitions in the workflow
    pub fn transitions(&self) -> impl Iterator<Item = &Box<dyn WorkflowTransition<S, I, O>>> {
        self.graph.edge_weights()
    }

    /// Check if the workflow has any terminal states
    pub fn has_terminal_states(&self) -> bool {
        self.graph.node_weights().any(|state| state.is_terminal())
    }

    /// Find all terminal states
    pub fn terminal_states(&self) -> Vec<&S> {
        self.graph.node_weights()
            .filter(|state| state.is_terminal())
            .collect()
    }

    /// Find all initial states (states with no incoming edges)
    pub fn initial_states(&self) -> Vec<&S> {
        self.graph.node_indices()
            .filter(|&idx| {
                self.graph.edges_directed(idx, petgraph::Direction::Incoming).count() == 0
            })
            .filter_map(|idx| self.graph.node_weight(idx))
            .collect()
    }

    /// Get the first initial state (convenience method)
    pub fn get_initial_state(&self) -> Option<&S> {
        self.initial_states().into_iter().next()
    }

    /// Validate the workflow graph
    pub fn validate(&self) -> Result<(), WorkflowGraphError> {
        // Check for at least one initial state
        if self.initial_states().is_empty() {
            return Err(WorkflowGraphError::NoInitialState);
        }

        // Check for at least one terminal state
        if !self.has_terminal_states() {
            return Err(WorkflowGraphError::NoTerminalState);
        }

        // Check for unreachable states
        let reachable = self.find_reachable_states();
        let total_states = self.graph.node_count();

        if reachable.len() < total_states {
            return Err(WorkflowGraphError::UnreachableStates {
                total: total_states,
                reachable: reachable.len(),
            });
        }

        Ok(())
    }

    /// Find all states reachable from initial states
    fn find_reachable_states(&self) -> HashMap<NodeIndex, bool> {
        use petgraph::visit::{Dfs, Walker};

        let mut reachable = HashMap::new();

        for state in self.initial_states() {
            if let Some(&start_idx) = self.state_to_node.get(&state.id()) {
                let dfs = Dfs::new(&self.graph, start_idx);
                for node in dfs.iter(&self.graph) {
                    reachable.insert(node, true);
                }
            }
        }

        reachable
    }
}

/// Errors that can occur when working with workflow graphs
#[derive(Debug, thiserror::Error)]
pub enum WorkflowGraphError {
    #[error("State not found: {0}")]
    StateNotFound(StateId),

    #[error("No initial state found in workflow")]
    NoInitialState,

    #[error("No terminal state found in workflow")]
    NoTerminalState,

    #[error("Unreachable states detected: {reachable} of {total} states are reachable")]
    UnreachableStates { total: usize, reachable: usize },

    #[error("Invalid transition: {0}")]
    InvalidTransition(String),
}

/// Example enrichment value for business workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessValue {
    pub monetary_value: f64,
    pub time_cost: std::time::Duration,
    pub risk_factor: f64,
}

impl EnrichmentValue for BusinessValue {
    fn combine(&self, other: &Self) -> Self {
        Self {
            monetary_value: self.monetary_value + other.monetary_value,
            time_cost: self.time_cost + other.time_cost,
            risk_factor: self.risk_factor.max(other.risk_factor), // Take worst risk
        }
    }

    fn identity() -> Self {
        Self {
            monetary_value: 0.0,
            time_cost: std::time::Duration::ZERO,
            risk_factor: 0.0,
        }
    }

    fn is_better_than(&self, other: &Self) -> bool {
        // Better means higher value, lower time, lower risk
        let self_score = self.monetary_value - self.time_cost.as_secs_f64() - self.risk_factor * 1000.0;
        let other_score = other.monetary_value - other.time_cost.as_secs_f64() - other.risk_factor * 1000.0;
        self_score > other_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cim_domain_workflow::{SimpleState, SimpleTransition, SimpleInput, SimpleOutput};

    #[test]
    fn test_workflow_graph_creation() {
        let graph: WorkflowGraph<SimpleState, SimpleInput, SimpleOutput, BusinessValue> =
            WorkflowGraph::new_with_name(
                "Test Workflow".to_string(),
                WorkflowType::Sequential,
                EnrichmentType::BusinessValue,
            );

        assert_eq!(graph.metadata.name, "Test Workflow");
        assert_eq!(graph.graph.node_count(), 0);
        assert_eq!(graph.graph.edge_count(), 0);
    }

    #[test]
    fn test_add_states_and_transitions() {
        let mut graph = WorkflowGraph::new_with_name(
            "Document Ingestion Workflow".to_string(),
            WorkflowType::Sequential,
            EnrichmentType::BusinessValue,
        );

        // Add states
        let uploaded = SimpleState::new("Uploaded");
        let processed = SimpleState::terminal("Processed");

        let _uploaded_idx = graph.add_state(uploaded.clone());
        let _processed_idx = graph.add_state(processed.clone());

        // Add transition
        let transition = Box::new(SimpleTransition::new(
            "process_document",
            uploaded.clone(),
            processed.clone(),
            SimpleInput::default(),
            SimpleOutput::default(),
        ));

        let enrichment = BusinessValue {
            monetary_value: 50.0,  // Value of processing the document
            time_cost: std::time::Duration::from_secs(120),  // 2 minutes to process
            risk_factor: 0.05,  // Low risk of processing failure
        };

        let edge_idx = graph.add_transition(transition, enrichment).unwrap();

        assert_eq!(graph.graph.node_count(), 2);
        assert_eq!(graph.graph.edge_count(), 1);
        assert!(graph.enrichment.contains_key(&edge_idx));
    }

    #[test]
    fn test_workflow_validation() {
        let mut graph = WorkflowGraph::new_with_name(
            "Document Processing Pipeline".to_string(),
            WorkflowType::Sequential,
            EnrichmentType::BusinessValue,
        );

        // Create a valid workflow: Uploaded -> Validated -> Indexed
        let uploaded = SimpleState::new("Uploaded");
        let validated = SimpleState::new("Validated");
        let indexed = SimpleState::terminal("Indexed");

        graph.add_state(uploaded.clone());
        graph.add_state(validated.clone());
        graph.add_state(indexed.clone());

        // Add transitions
        let t1 = Box::new(SimpleTransition::new(
            "validate_document",
            uploaded,
            validated.clone(),
            SimpleInput::default(),
            SimpleOutput::default(),
        ));

        let t2 = Box::new(SimpleTransition::new(
            "index_document",
            validated,
            indexed,
            SimpleInput::default(),
            SimpleOutput::default(),
        ));

        graph.add_transition(t1, BusinessValue::identity()).unwrap();
        graph.add_transition(t2, BusinessValue::identity()).unwrap();

        // Should be valid
        assert!(graph.validate().is_ok());

        // Check initial and terminal states
        assert_eq!(graph.initial_states().len(), 1);
        assert_eq!(graph.terminal_states().len(), 1);
    }
}
