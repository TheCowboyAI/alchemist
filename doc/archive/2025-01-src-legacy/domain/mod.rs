//! Domain Layer - Business Logic and Rules

use thiserror::Error;

pub mod aggregates;
pub mod commands;
pub mod conceptual_graph;
pub mod content_types;
pub mod events;
pub mod services;
pub mod value_objects;

/// Domain-level errors
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Aggregate already exists")]
    AggregateAlreadyExists,

    #[error("Aggregate not found")]
    AggregateNotFound,

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Duplicate entity: {0}")]
    DuplicateEntity(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),

    #[error("Concurrent modification detected")]
    ConcurrentModification,

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Node not found")]
    NodeNotFound,

    #[error("Node already exists")]
    NodeAlreadyExists,

    #[error("Edge not found")]
    EdgeNotFound,

    #[error("Edge already exists")]
    EdgeAlreadyExists,

    #[error("Workflow already exists")]
    WorkflowAlreadyExists,

    #[error("Step already exists")]
    StepAlreadyExists,

    #[error("Step not found")]
    StepNotFound,

    #[error("Transition already exists")]
    TransitionAlreadyExists,

    #[error("Invalid workflow state")]
    InvalidWorkflowState,

    #[error("Workflow not valid")]
    WorkflowNotValid,

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Invalid position: {0}")]
    InvalidPosition(String),

    #[error("Invalid node type: {0}")]
    InvalidNodeType(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub mod prelude {
    // Re-export aggregates
    pub use super::aggregates::{Graph, Workflow};

    // Re-export commands
    pub use super::commands::{
        Command, GraphCommand, NodeCommand, EdgeCommand, WorkflowCommand,
        DomainCommand, UpdateNodePositions, UpdateGraphSelection,
        RecognizeGraphModel, ApplyGraphMorphism, MorphismType
    };

    // Re-export content types
    pub use super::content_types::*;

    // Re-export events
    pub use super::events::{
        DomainEvent, GraphEvent, NodeEvent, EdgeEvent,
        ChainedEvent, EventChain
    };

    // Re-export workflow events specifically
    pub use super::events::workflow::{
        WorkflowCreated, StepAdded, StepsConnected, WorkflowValidated,
        WorkflowStarted, StepCompleted, WorkflowPaused, WorkflowResumed,
        WorkflowCompleted, WorkflowFailed, ValidationResult
    };

    // Re-export value objects
    pub use super::value_objects::*;

    // Re-export error type
    pub use super::DomainError;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_domain_error_display() {
        // Test that all error variants have proper display implementations
        let errors = vec![
            DomainError::AggregateAlreadyExists,
            DomainError::AggregateNotFound,
            DomainError::InvalidState("Test state".to_string()),
            DomainError::EntityNotFound("Test entity".to_string()),
            DomainError::DuplicateEntity("Test duplicate".to_string()),
            DomainError::ValidationError("Test validation".to_string()),
            DomainError::BusinessRuleViolation("Test rule".to_string()),
            DomainError::ConcurrentModification,
            DomainError::InvalidCommand("Test command".to_string()),
            DomainError::NodeNotFound,
            DomainError::NodeAlreadyExists,
            DomainError::EdgeNotFound,
            DomainError::EdgeAlreadyExists,
            DomainError::WorkflowAlreadyExists,
            DomainError::StepAlreadyExists,
            DomainError::StepNotFound,
            DomainError::TransitionAlreadyExists,
            DomainError::InvalidWorkflowState,
            DomainError::WorkflowNotValid,
            DomainError::ValidationFailed("Test failed".to_string()),
            DomainError::Other("Test other".to_string()),
        ];

        for error in errors {
            // Verify Display trait works
            let display = format!("{}", error);
            assert!(!display.is_empty());

            // Verify error messages contain expected content
            match &error {
                DomainError::InvalidState(msg) => assert!(display.contains(msg)),
                DomainError::EntityNotFound(msg) => assert!(display.contains(msg)),
                DomainError::DuplicateEntity(msg) => assert!(display.contains(msg)),
                DomainError::ValidationError(msg) => assert!(display.contains(msg)),
                DomainError::BusinessRuleViolation(msg) => assert!(display.contains(msg)),
                DomainError::InvalidCommand(msg) => assert!(display.contains(msg)),
                DomainError::ValidationFailed(msg) => assert!(display.contains(msg)),
                DomainError::Other(msg) => assert!(display.contains(msg)),
                _ => {} // Other variants have static messages
            }
        }
    }

    #[test]
    fn test_domain_error_is_error() {
        // Verify that DomainError implements std::error::Error
        let error: Box<dyn std::error::Error> = Box::new(DomainError::AggregateNotFound);
        assert_eq!(error.to_string(), "Aggregate not found");
    }

    #[test]
    fn test_prelude_exports() {
        // Test that prelude exports are accessible
        use prelude::*;

        // Value objects
        let _graph_id = GraphId::new();
        let _node_id = NodeId::new();
        let _edge_id = EdgeId::new();
        let _workflow_id = WorkflowId::new();
        let _step_id = StepId::new();
        let _user_id = UserId::new();

        // Position
        let _pos = Position3D::new(1.0, 2.0, 3.0).unwrap();

        // Node types
        let _node_type = NodeType::Entity;

        // Relationship types
        let _rel_type = RelationshipType::DependsOn;

        // Commands
        let _cmd = Command::Graph(GraphCommand::CreateGraph {
            id: GraphId::new(),
            name: "Test".to_string(),
            metadata: HashMap::new(),
        });

        // Events
        let _event = DomainEvent::Graph(GraphEvent::GraphCreated {
            id: GraphId::new(),
            metadata: GraphMetadata::new("Test".to_string()),
        });

        // Verify types compile and are accessible
        assert!(true);
    }
}
