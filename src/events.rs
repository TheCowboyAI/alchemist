use std::any::Any;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// Event types for graph operations
#[derive(Debug, Clone)]
pub enum GraphEventType {
    NodeCreated,
    NodeUpdated,
    NodeDeleted,
    EdgeCreated,
    EdgeUpdated,
    EdgeDeleted,
    GraphCleared,
    WorkflowStepExecuted,
    WorkflowStateChanged,
}

// GraphEvent structure - represents a change in the information graph
#[derive(Debug, Clone)]
pub struct GraphEvent {
    pub event_type: GraphEventType,
    pub entity_id: Option<Uuid>,
    pub payload: HashMap<String, String>,
    pub timestamp: u64,
}

impl GraphEvent {
    pub fn new(event_type: GraphEventType, entity_id: Option<Uuid>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            event_type,
            entity_id,
            payload: HashMap::new(),
            timestamp,
        }
    }

    pub fn with_payload(mut self, key: &str, value: &str) -> Self {
        self.payload.insert(key.to_string(), value.to_string());
        self
    }

    pub fn event_type_str(&self) -> &str {
        match self.event_type {
            GraphEventType::NodeCreated => "NodeCreated",
            GraphEventType::NodeUpdated => "NodeUpdated",
            GraphEventType::NodeDeleted => "NodeDeleted",
            GraphEventType::EdgeCreated => "EdgeCreated",
            GraphEventType::EdgeUpdated => "EdgeUpdated",
            GraphEventType::EdgeDeleted => "EdgeDeleted",
            GraphEventType::GraphCleared => "GraphCleared",
            GraphEventType::WorkflowStepExecuted => "WorkflowStepExecuted",
            GraphEventType::WorkflowStateChanged => "WorkflowStateChanged",
        }
    }
}

// Base Event trait using Any for type erasure
pub trait Event: std::fmt::Debug + Send + Sync + 'static {
    fn event_type(&self) -> &str;
    fn entity_id(&self) -> Option<Uuid>;
    fn timestamp(&self) -> u64;
    fn as_any(&self) -> &dyn Any;
}

// Implement Event for GraphEvent
impl Event for GraphEvent {
    fn event_type(&self) -> &str {
        self.event_type_str()
    }

    fn entity_id(&self) -> Option<Uuid> {
        self.entity_id
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// EventStream to store and process events
#[derive(Debug)]
pub struct EventStream {
    events: Vec<Box<dyn Event>>,
}

impl EventStream {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn append<T: Event + 'static>(&mut self, event: T) {
        self.events.push(Box::new(event));
    }

    pub fn get_events(&self) -> &[Box<dyn Event>] {
        &self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    // Get events related to a specific entity
    pub fn entity_events(&self, entity_id: Uuid) -> Vec<&dyn Event> {
        self.events
            .iter()
            .filter(|event| event.entity_id() == Some(entity_id))
            .map(|boxed| boxed.as_ref())
            .collect()
    }

    // Get events of a specific type
    pub fn events_by_type(&self, event_type: &str) -> Vec<&dyn Event> {
        self.events
            .iter()
            .filter(|event| event.event_type() == event_type)
            .map(|boxed| boxed.as_ref())
            .collect()
    }
}

// Model is something that can have events applied to it
pub trait Model {
    fn apply_event(&mut self, event: &GraphEvent);
}

// Command pattern for graph operations - Commands produce Events
pub trait Command {
    fn execute(&self) -> Vec<Box<dyn Event>>;
    fn undo(&self) -> Option<Vec<Box<dyn Event>>> {
        None // Optional, not all commands are undoable
    }
}

// Query pattern for reading data without modification
pub trait Query<T> {
    fn execute(&self) -> T;
}

// Example commands for graph operations
pub struct CreateNodeCommand {
    pub name: String,
    pub labels: Vec<String>,
}

impl Command for CreateNodeCommand {
    fn execute(&self) -> Vec<Box<dyn Event>> {
        let node_id = Uuid::new_v4();
        let event = GraphEvent::new(GraphEventType::NodeCreated, Some(node_id))
            .with_payload("name", &self.name);

        // Add labels to payload
        let labels_str = self.labels.join(",");
        let event = event.with_payload("labels", &labels_str);

        vec![Box::new(event)]
    }

    fn undo(&self) -> Option<Vec<Box<dyn Event>>> {
        // Undo would require the node ID which we don't have until execute()
        // In a real implementation, you would store the node ID after execute()
        None
    }
}

pub struct CreateEdgeCommand {
    pub source: Uuid,
    pub target: Uuid,
    pub labels: Vec<String>,
}

impl Command for CreateEdgeCommand {
    fn execute(&self) -> Vec<Box<dyn Event>> {
        let edge_id = Uuid::new_v4();
        let event = GraphEvent::new(GraphEventType::EdgeCreated, Some(edge_id))
            .with_payload("source", &self.source.to_string())
            .with_payload("target", &self.target.to_string());

        // Add labels to payload
        let labels_str = self.labels.join(",");
        let event = event.with_payload("labels", &labels_str);

        vec![Box::new(event)]
    }
}

pub struct DeleteNodeCommand {
    pub node_id: Uuid,
}

impl Command for DeleteNodeCommand {
    fn execute(&self) -> Vec<Box<dyn Event>> {
        let event = GraphEvent::new(GraphEventType::NodeDeleted, Some(self.node_id));
        vec![Box::new(event)]
    }
}

pub struct UpdateNodeCommand {
    pub node_id: Uuid,
    pub properties: HashMap<String, String>,
}

impl Command for UpdateNodeCommand {
    fn execute(&self) -> Vec<Box<dyn Event>> {
        let mut event = GraphEvent::new(GraphEventType::NodeUpdated, Some(self.node_id));

        // Add all properties to the payload
        for (key, value) in &self.properties {
            event = event.with_payload(key, value);
        }

        vec![Box::new(event)]
    }
}

pub struct WorkflowStepCommand {
    pub workflow_id: Uuid,
    pub step_id: Uuid,
    pub action: String,
}

impl Command for WorkflowStepCommand {
    fn execute(&self) -> Vec<Box<dyn Event>> {
        let event = GraphEvent::new(GraphEventType::WorkflowStepExecuted, Some(self.step_id))
            .with_payload("workflow_id", &self.workflow_id.to_string())
            .with_payload("action", &self.action);

        vec![Box::new(event)]
    }
}
