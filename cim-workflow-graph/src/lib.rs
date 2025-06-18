//! Workflow Graph implementation for CIM workflows
//!
//! This module provides a graph structure that integrates the new CIM workflow domain
//! with ContextGraph format for visualization and analysis.

use cim_domain_workflow::{
    aggregate::Workflow,
    projections::WorkflowContextGraph,
    value_objects::{StepId, StepStatus, StepType, WorkflowId, WorkflowStatus},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

pub use cim_domain_workflow::projections::{
    ContextGraphEdge, ContextGraphEdgeValue, ContextGraphNode, ContextGraphNodeValue,
    WorkflowContextGraph as ContextGraph, WorkflowGraphStatistics,
};

/// Enhanced workflow graph that bridges domain workflows with visualization
#[derive(Debug, Clone)]
pub struct WorkflowGraph {
    /// The underlying workflow aggregate
    pub workflow: Workflow,
    /// ContextGraph representation for visualization
    pub context_graph: WorkflowContextGraph,
    /// Graph metadata
    pub metadata: WorkflowGraphMetadata,
}

/// Metadata for workflow graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGraphMetadata {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

impl Default for WorkflowGraphMetadata {
    fn default() -> Self {
        Self {
            name: "Untitled Workflow".to_string(),
            description: String::new(),
            tags: Vec::new(),
            properties: HashMap::new(),
        }
    }
}

impl WorkflowGraph {
    /// Create a new workflow graph
    pub fn new(name: String, description: String) -> Result<Self, WorkflowGraphError> {
        let metadata = HashMap::new();
        let (workflow, _events) = Workflow::new(name.clone(), description.clone(), metadata, None)
            .map_err(|e| WorkflowGraphError::DomainError(e.to_string()))?;

        let context_graph = WorkflowContextGraph::from_workflow(&workflow);

        Ok(Self {
            workflow,
            context_graph,
            metadata: WorkflowGraphMetadata {
                name,
                description,
                tags: Vec::new(),
                properties: HashMap::new(),
            },
        })
    }

    /// Create a workflow graph from an existing workflow
    pub fn from_workflow(workflow: Workflow) -> Self {
        let context_graph = WorkflowContextGraph::from_workflow(&workflow);

        Self {
            metadata: WorkflowGraphMetadata {
                name: workflow.name.clone(),
                description: workflow.description.clone(),
                tags: Vec::new(),
                properties: workflow.metadata.clone(),
            },
            workflow,
            context_graph,
        }
    }

    /// Add a step to the workflow
    pub fn add_step(
        &mut self,
        name: String,
        description: String,
        step_type: StepType,
        config: HashMap<String, serde_json::Value>,
        dependencies: Vec<StepId>,
        estimated_duration_minutes: Option<u32>,
        assigned_to: Option<String>,
    ) -> Result<StepId, WorkflowGraphError> {
        let events = self
            .workflow
            .add_step(
                name,
                description,
                step_type,
                config,
                dependencies,
                estimated_duration_minutes,
                assigned_to,
                Some("system".to_string()),
            )
            .map_err(|e| WorkflowGraphError::DomainError(e.to_string()))?;

        // Extract the step ID from the events
        if let Some(cim_domain_workflow::WorkflowDomainEvent::StepAdded(ref event)) = events.first()
        {
            let step_id = event.step_id;

            // Refresh the context graph
            self.refresh_context_graph();

            Ok(step_id)
        } else {
            Err(WorkflowGraphError::InvalidOperation(
                "Failed to create step".to_string(),
            ))
        }
    }

    /// Start the workflow
    pub fn start(
        &mut self,
        context: HashMap<String, serde_json::Value>,
    ) -> Result<(), WorkflowGraphError> {
        let mut workflow_context = cim_domain_workflow::value_objects::WorkflowContext::new();
        workflow_context.variables = context;
        workflow_context.set_actor("system".to_string());

        let _events = self
            .workflow
            .start(workflow_context, Some("system".to_string()))
            .map_err(|e| WorkflowGraphError::DomainError(e.to_string()))?;

        // Refresh the context graph
        self.refresh_context_graph();

        Ok(())
    }

    /// Complete the workflow
    pub fn complete(&mut self) -> Result<(), WorkflowGraphError> {
        let _events = self
            .workflow
            .complete()
            .map_err(|e| WorkflowGraphError::DomainError(e.to_string()))?;

        // Refresh the context graph
        self.refresh_context_graph();

        Ok(())
    }

    /// Get workflow status
    pub fn status(&self) -> &WorkflowStatus {
        &self.workflow.status
    }

    /// Get workflow ID
    pub fn id(&self) -> WorkflowId {
        self.workflow.id
    }

    /// Get the workflow name
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Get the workflow description
    pub fn description(&self) -> &str {
        &self.metadata.description
    }

    /// Get all step nodes from the context graph
    pub fn get_step_nodes(&self) -> Vec<&ContextGraphNode> {
        self.context_graph.get_step_nodes()
    }

    /// Get all dependency edges from the context graph
    pub fn get_dependency_edges(&self) -> Vec<&ContextGraphEdge> {
        self.context_graph.get_dependency_edges()
    }

    /// Get workflow statistics
    pub fn statistics(&self) -> WorkflowGraphStatistics {
        self.context_graph.statistics()
    }

    /// Export as JSON
    pub fn to_json(&self) -> Result<String, WorkflowGraphError> {
        self.context_graph
            .to_json()
            .map_err(|e| WorkflowGraphError::SerializationError(e.to_string()))
    }

    /// Import from JSON
    pub fn from_json(json: &str) -> Result<WorkflowContextGraph, WorkflowGraphError> {
        WorkflowContextGraph::from_json(json)
            .map_err(|e| WorkflowGraphError::SerializationError(e.to_string()))
    }

    /// Export as DOT format for Graphviz
    pub fn to_dot(&self) -> String {
        self.context_graph.to_dot()
    }

    /// Get executable steps (steps that can be run now)
    pub fn get_executable_steps(&self) -> Vec<StepId> {
        self.workflow
            .get_executable_steps()
            .into_iter()
            .map(|step| step.id)
            .collect()
    }

    /// Find steps by status
    pub fn find_steps_by_status(&self, status: StepStatus) -> Vec<StepId> {
        self.workflow
            .steps
            .values()
            .filter(|step| step.status == status)
            .map(|step| step.id)
            .collect()
    }

    /// Find steps by type
    pub fn find_steps_by_type(&self, step_type: StepType) -> Vec<StepId> {
        self.workflow
            .steps
            .values()
            .filter(|step| step.step_type == step_type)
            .map(|step| step.id)
            .collect()
    }

    /// Add metadata tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.metadata.tags.contains(&tag) {
            self.metadata.tags.push(tag);
        }
    }

    /// Set metadata property
    pub fn set_property(&mut self, key: String, value: serde_json::Value) {
        self.metadata.properties.insert(key, value);
    }

    /// Get metadata property
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.properties.get(key)
    }

    /// Refresh the context graph representation
    fn refresh_context_graph(&mut self) {
        self.context_graph = WorkflowContextGraph::from_workflow(&self.workflow);
    }

    /// Validate the workflow graph
    pub fn validate(&self) -> Result<(), WorkflowGraphError> {
        // Check for circular dependencies
        for (step_id, step) in &self.workflow.steps {
            if self.has_circular_dependency(step_id, &step.dependencies) {
                return Err(WorkflowGraphError::CircularDependency(format!(
                    "Step {} has circular dependency",
                    step_id.as_uuid()
                )));
            }
        }

        // Check that all dependencies exist
        for step in self.workflow.steps.values() {
            for dep_id in &step.dependencies {
                if !self.workflow.steps.contains_key(dep_id) {
                    return Err(WorkflowGraphError::InvalidDependency(format!(
                        "Step {} depends on non-existent step {}",
                        step.id.as_uuid(),
                        dep_id.as_uuid()
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check for circular dependencies
    fn has_circular_dependency(&self, step_id: &StepId, dependencies: &[StepId]) -> bool {
        for dep_id in dependencies {
            if dep_id == step_id {
                return true;
            }
            if let Some(dep_step) = self.workflow.steps.get(dep_id) {
                if self.has_circular_dependency(step_id, &dep_step.dependencies) {
                    return true;
                }
            }
        }
        false
    }
}

/// Errors that can occur when working with workflow graphs
#[derive(Debug, thiserror::Error)]
pub enum WorkflowGraphError {
    #[error("Domain error: {0}")]
    DomainError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Invalid dependency: {0}")]
    InvalidDependency(String),

    #[error("Step not found: {0}")]
    StepNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use cim_domain_workflow::value_objects::StepType;

    #[test]
    fn test_workflow_graph_creation() {
        let workflow_graph =
            WorkflowGraph::new("Test Workflow".to_string(), "A test workflow".to_string()).unwrap();

        assert_eq!(workflow_graph.name(), "Test Workflow");
        assert_eq!(workflow_graph.description(), "A test workflow");
        assert_eq!(workflow_graph.status(), &WorkflowStatus::Draft);
    }

    #[test]
    fn test_add_step() {
        let mut workflow_graph =
            WorkflowGraph::new("Test Workflow".to_string(), "A test workflow".to_string()).unwrap();

        let step_id = workflow_graph
            .add_step(
                "Test Step".to_string(),
                "A test step".to_string(),
                StepType::Manual,
                HashMap::new(),
                Vec::new(),
                Some(30),
                Some("test-user".to_string()),
            )
            .unwrap();

        let stats = workflow_graph.statistics();
        assert_eq!(stats.step_nodes, 1);
        assert!(workflow_graph.workflow.steps.contains_key(&step_id));
    }

    #[test]
    fn test_step_dependencies() {
        let mut workflow_graph =
            WorkflowGraph::new("Test Workflow".to_string(), "A test workflow".to_string()).unwrap();

        // Add first step
        let step1_id = workflow_graph
            .add_step(
                "Step 1".to_string(),
                "First step".to_string(),
                StepType::Manual,
                HashMap::new(),
                Vec::new(),
                Some(30),
                None,
            )
            .unwrap();

        // Add second step that depends on first
        let step2_id = workflow_graph
            .add_step(
                "Step 2".to_string(),
                "Second step".to_string(),
                StepType::Automated,
                HashMap::new(),
                vec![step1_id],
                Some(15),
                None,
            )
            .unwrap();

        let stats = workflow_graph.statistics();
        assert_eq!(stats.step_nodes, 2);
        assert!(stats.dependency_edges > 0);

        // Validate the graph
        assert!(workflow_graph.validate().is_ok());
    }

    #[test]
    fn test_json_export() {
        let mut workflow_graph =
            WorkflowGraph::new("Export Test".to_string(), "Testing JSON export".to_string())
                .unwrap();

        workflow_graph
            .add_step(
                "Test Step".to_string(),
                "A test step".to_string(),
                StepType::Manual,
                HashMap::new(),
                Vec::new(),
                Some(30),
                None,
            )
            .unwrap();

        let json = workflow_graph.to_json().unwrap();
        assert!(json.contains("Export Test"));
        assert!(json.contains("Test Step"));

        // Test round-trip
        let _reconstructed = WorkflowGraph::from_json(&json).unwrap();
    }

    #[test]
    fn test_dot_export() {
        let mut workflow_graph =
            WorkflowGraph::new("DOT Test".to_string(), "Testing DOT export".to_string()).unwrap();

        workflow_graph
            .add_step(
                "Test Step".to_string(),
                "A test step".to_string(),
                StepType::Manual,
                HashMap::new(),
                Vec::new(),
                Some(30),
                None,
            )
            .unwrap();

        let dot = workflow_graph.to_dot();
        assert!(dot.contains("digraph"));
        assert!(dot.contains("DOT Test"));
        assert!(dot.contains("Start"));
        assert!(dot.contains("End"));
    }

    #[test]
    fn test_workflow_lifecycle() {
        let mut workflow_graph = WorkflowGraph::new(
            "Lifecycle Test".to_string(),
            "Testing workflow lifecycle".to_string(),
        )
        .unwrap();

        // Add a step
        workflow_graph
            .add_step(
                "Test Step".to_string(),
                "A test step".to_string(),
                StepType::Manual,
                HashMap::new(),
                Vec::new(),
                Some(30),
                None,
            )
            .unwrap();

        // Start the workflow
        assert!(workflow_graph.start(HashMap::new()).is_ok());
        assert_eq!(workflow_graph.status(), &WorkflowStatus::Running);

        // Note: Complete would require all steps to be completed
        // For now, just verify the workflow is in running state
    }
}
