//! Workflow command handler

use std::sync::Arc;

use crate::domain::{
    aggregates::workflow::Workflow,
    commands::workflow::WorkflowCommand,
    events::DomainEvent,
    value_objects::WorkflowId,
    DomainError,
};
use crate::infrastructure::event_store::EventStore;

/// Handles workflow-related commands
pub struct WorkflowCommandHandler {
    event_store: Arc<dyn EventStore>,
}

impl WorkflowCommandHandler {
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        Self { event_store }
    }

    /// Load workflow aggregate from event store
    async fn load_workflow(&self, workflow_id: &WorkflowId) -> Result<Workflow, DomainError> {
        let events = self.event_store
            .get_events(workflow_id.to_string())
            .await
            .map_err(|_| DomainError::AggregateNotFound)?;

        let mut workflow = Workflow::new(
            *workflow_id,
            String::new(),
            crate::domain::value_objects::UserId::new(),
        );

        for event in events {
            workflow.apply_event(&event)?;
        }

        Ok(workflow)
    }

    /// Handle a workflow command
    pub async fn handle(&self, command: WorkflowCommand) -> Result<Vec<DomainEvent>, DomainError> {
        let workflow_id = match &command {
            WorkflowCommand::CreateWorkflow(cmd) => cmd.workflow_id,
            WorkflowCommand::AddStep(cmd) => cmd.workflow_id,
            WorkflowCommand::ConnectSteps(cmd) => cmd.workflow_id,
            WorkflowCommand::ValidateWorkflow(cmd) => cmd.workflow_id,
            WorkflowCommand::StartWorkflow(cmd) => cmd.workflow_id,
            WorkflowCommand::CompleteStep(cmd) => cmd.workflow_id,
            WorkflowCommand::PauseWorkflow(cmd) => cmd.workflow_id,
            WorkflowCommand::ResumeWorkflow(cmd) => cmd.workflow_id,
            WorkflowCommand::FailWorkflow(cmd) => cmd.workflow_id,
        };

        // Load or create workflow
        let mut workflow = if matches!(command, WorkflowCommand::CreateWorkflow(_)) {
            Workflow::new(
                workflow_id,
                String::new(),
                crate::domain::value_objects::UserId::new(),
            )
        } else {
            self.load_workflow(&workflow_id).await?
        };

        // Handle command
        let events = workflow.handle_command(command)?;

        // Store events
        if !events.is_empty() {
            self.event_store
                .append_events(workflow_id.to_string(), events.clone())
                .await
                .map_err(|_| DomainError::ConcurrentModification)?;
        }

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::event_store::InMemoryEventStore;
    use crate::domain::commands::workflow::{CreateWorkflow, AddStep};
    use crate::domain::aggregates::workflow::{WorkflowStep, StepType};

    #[tokio::test]
    async fn test_create_workflow() {
        let event_store = Arc::new(InMemoryEventStore::new());
        let handler = WorkflowCommandHandler::new(event_store);

        let workflow_id = WorkflowId::new();
        let user_id = crate::domain::value_objects::UserId::new();

        let command = WorkflowCommand::CreateWorkflow(CreateWorkflow {
            workflow_id: workflow_id.clone(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            created_by: user_id,
            tags: vec!["test".to_string()],
        });

        let events = handler.handle(command).await.unwrap();

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::Workflow(_)));
    }

    #[tokio::test]
    async fn test_add_step_to_workflow() {
        let event_store = Arc::new(InMemoryEventStore::new());
        let handler = WorkflowCommandHandler::new(event_store);

        let workflow_id = WorkflowId::new();
        let user_id = crate::domain::value_objects::UserId::new();

        // First create the workflow
        let create_command = WorkflowCommand::CreateWorkflow(CreateWorkflow {
            workflow_id: workflow_id.clone(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            created_by: user_id,
            tags: vec![],
        });

        handler.handle(create_command).await.unwrap();

        // Then add a step
        let step = WorkflowStep {
            id: crate::domain::value_objects::StepId::new(),
            name: "Step 1".to_string(),
            step_type: StepType::UserTask,
            node_id: crate::domain::value_objects::NodeId::new(),
            inputs: vec![],
            outputs: vec![],
            timeout_ms: None,
            retry_policy: None,
        };

        let add_step_command = WorkflowCommand::AddStep(AddStep {
            workflow_id: workflow_id.clone(),
            step,
        });

        let events = handler.handle(add_step_command).await.unwrap();

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::Workflow(_)));
    }
}
