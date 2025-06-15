//! Workflow execution service
//!
//! Manages workflow instances and processes transitions via NATS messages

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use cim_contextgraph::WorkflowGraph;
use cim_domain::workflow::{
    TransitionInput, TransitionOutput, WorkflowAggregate, WorkflowCommand, WorkflowEvent,
    WorkflowState,
};

use crate::domain::value_objects::{GraphId, WorkflowId};
use crate::infrastructure::event_store::distributed::DistributedEventStore as EventStore;
use crate::infrastructure::nats::{NatsClient, WorkflowEventType, WorkflowSubjectMapper};
use crate::shared::types::DomainError;

/// Service for executing workflows via NATS
pub struct WorkflowExecutionService<S, I, O>
where
    S: WorkflowState + Clone + Send + Sync + 'static,
    I: TransitionInput + Clone + Send + Sync + 'static,
    O: TransitionOutput + Clone + Send + Sync + 'static,
{
    /// Active workflow instances
    workflows: Arc<RwLock<HashMap<WorkflowId, WorkflowAggregate<S, I, O>>>>,

    /// Workflow definitions
    definitions: Arc<RwLock<HashMap<GraphId, WorkflowGraph<S, I, O, f32>>>>,

    /// NATS subject mapper
    subject_mapper: WorkflowSubjectMapper<I, O>,

    /// Base subject for workflow events
    base_subject: String,

    /// NATS client
    nats_client: Arc<NatsClient>,

    /// Event store
    event_store: Arc<EventStore>,
}

impl<S, I, O> WorkflowExecutionService<S, I, O>
where
    S: WorkflowState
        + Clone
        + Send
        + Sync
        + 'static
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>,
    I: TransitionInput
        + Clone
        + Send
        + Sync
        + 'static
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>,
    O: TransitionOutput
        + Clone
        + Send
        + Sync
        + 'static
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>,
{
    /// Create a new workflow execution service
    pub fn new(
        nats_client: Arc<NatsClient>,
        event_store: Arc<EventStore>,
        base_subject: String,
    ) -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            definitions: Arc::new(RwLock::new(HashMap::new())),
            subject_mapper: WorkflowSubjectMapper::new(base_subject.clone()),
            base_subject,
            nats_client,
            event_store,
        }
    }

    /// Register a workflow definition
    pub async fn register_definition(
        &self,
        definition: WorkflowGraph<S, I, O, f32>,
    ) -> Result<(), DomainError> {
        let mut definitions = self.definitions.write().await;
        definitions.insert(definition.id.clone(), definition);
        Ok(())
    }

    /// Start a new workflow instance
    pub async fn start_workflow(
        &self,
        definition_id: GraphId,
        initial_context: cim_domain::workflow::WorkflowContext,
    ) -> Result<WorkflowId, DomainError> {
        // Get the definition
        let definitions = self.definitions.read().await;
        let definition = definitions.get(&definition_id).ok_or_else(|| {
            DomainError::NotFound(format!("Workflow definition not found: {}", definition_id))
        })?;

        // Get initial state from definition
        let initial_state = definition
            .get_initial_state()
            .ok_or_else(|| {
                DomainError::Validation("No initial state in workflow definition".to_string())
            })?
            .clone();

        let workflow =
            WorkflowAggregate::new(definition_id.clone(), initial_state, initial_context);

        // Get the workflow ID from the aggregate
        let workflow_id = WorkflowId::from(workflow.id.to_string());

        // Store the workflow
        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow_id.clone(), workflow);

        // Publish workflow started event
        let event = WorkflowStartedEvent {
            workflow_id: workflow_id.clone(),
            definition_id,
        };

        let (subject, payload) = self.subject_mapper.map_output_to_subject(
            &workflow_id,
            &event,
            WorkflowEventType::Started,
        )?;

        self.nats_client
            .publish(&subject, payload)
            .await
            .map_err(|e| DomainError::Infrastructure(format!("Failed to publish event: {}", e)))?;

        Ok(workflow_id)
    }

    /// Execute a transition in a workflow
    pub async fn execute_transition(
        &self,
        workflow_id: &WorkflowId,
        input: I,
    ) -> Result<O, DomainError> {
        // Get the workflow
        let mut workflows = self.workflows.write().await;
        let workflow = workflows.get_mut(workflow_id).ok_or_else(|| {
            DomainError::NotFound(format!("Workflow instance not found: {}", workflow_id))
        })?;

        // Get the definition
        let definitions = self.definitions.read().await;
        let definition = definitions
            .get(&workflow.definition_id)
            .ok_or_else(|| DomainError::NotFound("Workflow definition not found".to_string()))?;

        // Find applicable transition
        let current_state = &workflow.current_state;
        let transitions = definition.find_transitions(current_state, &input, &workflow.context);

        if transitions.is_empty() {
            return Err(DomainError::Validation(
                "No applicable transition found".to_string(),
            ));
        }

        // Use the first valid transition (or could use find_optimal_transition)
        let (transition, _enrichment, _edge_idx) = transitions.into_iter().next().unwrap();

        // Check guard
        if !transition.guard(&workflow.context) {
            return Err(DomainError::Validation(
                "Transition guard failed".to_string(),
            ));
        }

        // Execute the transition (side effects only)
        transition
            .execute(&mut workflow.context)
            .map_err(|e| DomainError::Validation(format!("Transition execution failed: {}", e)))?;

        // Get the output from the transition
        let output = transition.output().clone();

        // Update workflow state
        workflow.record_transition(
            transition.source().clone(),
            transition.target().clone(),
            input.clone(),
            output.clone(),
            std::time::Duration::from_millis(10), // TODO: measure actual duration
        );

        // Publish transition executed event
        let event = TransitionExecutedEvent {
            workflow_id: workflow_id.clone(),
            from_state: transition.source().clone(),
            to_state: transition.target().clone(),
            input: input.clone(),
            output: output.clone(),
        };

        let (subject, payload) = self.subject_mapper.map_output_to_subject(
            workflow_id,
            &event,
            WorkflowEventType::TransitionExecuted,
        )?;

        self.nats_client
            .publish(&subject, payload)
            .await
            .map_err(|e| DomainError::Infrastructure(format!("Failed to publish event: {}", e)))?;

        // Check if workflow is completed
        if workflow.current_state.is_terminal() {
            self.complete_workflow(workflow_id).await?;
        }

        Ok(output)
    }

    /// Complete a workflow
    async fn complete_workflow(&self, workflow_id: &WorkflowId) -> Result<(), DomainError> {
        let event = WorkflowCompletedEvent {
            workflow_id: workflow_id.clone(),
        };

        let (subject, payload) = self.subject_mapper.map_output_to_subject(
            workflow_id,
            &event,
            WorkflowEventType::Completed,
        )?;

        self.nats_client
            .publish(&subject, payload)
            .await
            .map_err(|e| DomainError::Infrastructure(format!("Failed to publish event: {}", e)))?;

        Ok(())
    }

    /// Handle incoming NATS message
    pub async fn handle_message(&self, msg: async_nats::Message) -> Result<(), DomainError> {
        // Parse workflow ID from subject
        let workflow_id = self.subject_mapper.parse_workflow_id(&msg.subject)?;

        // Parse event type
        let event_type = self.subject_mapper.parse_event_type(&msg.subject)?;

        match event_type {
            WorkflowEventType::Command => {
                // Parse input from payload
                let input = self
                    .subject_mapper
                    .map_subject_to_input(&msg.subject, &msg.payload)?;

                // Execute transition
                self.execute_transition(&workflow_id, input).await?;
            }
            _ => {
                // Other event types are informational
                tracing::debug!("Received workflow event: {:?}", event_type);
            }
        }

        Ok(())
    }

    /// Subscribe to workflow commands
    pub async fn subscribe_to_commands(&self) -> Result<(), DomainError> {
        let subject = format!("{}.*>.command", self.base_subject);

        let mut subscriber = self
            .nats_client
            .subscribe(&subject)
            .await
            .map_err(|e| DomainError::Infrastructure(format!("Failed to subscribe: {}", e)))?;

        let service = self.clone();

        tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                if let Err(e) = service.handle_message(msg).await {
                    tracing::error!("Error handling workflow message: {}", e);
                }
            }
        });

        Ok(())
    }
}

// Event types for serialization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WorkflowStartedEvent {
    workflow_id: WorkflowId,
    definition_id: GraphId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TransitionExecutedEvent<S, I, O> {
    workflow_id: WorkflowId,
    from_state: S,
    to_state: S,
    input: I,
    output: O,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WorkflowCompletedEvent {
    workflow_id: WorkflowId,
}

// Clone implementation for the service
impl<S, I, O> Clone for WorkflowExecutionService<S, I, O>
where
    S: WorkflowState + Clone + Send + Sync + 'static,
    I: TransitionInput + Clone + Send + Sync + 'static,
    O: TransitionOutput + Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            workflows: self.workflows.clone(),
            definitions: self.definitions.clone(),
            subject_mapper: WorkflowSubjectMapper::new(self.base_subject.clone()),
            base_subject: self.base_subject.clone(),
            nats_client: self.nats_client.clone(),
            event_store: self.event_store.clone(),
        }
    }
}
