//! Dialog domain events

use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::value_objects::{
    Turn, Topic, Participant, ContextVariable,
    ConversationMetrics,
};

/// Dialog started event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogStarted {
    pub dialog_id: Uuid,
    pub dialog_type: crate::DialogType,
    pub primary_participant: Participant,
    pub started_at: DateTime<Utc>,
}

impl DomainEvent for DialogStarted {
    fn subject(&self) -> String {
        "dialog.started.v1".to_string()
    }

    fn aggregate_id(&self) -> Uuid {
        self.dialog_id
    }

    fn event_type(&self) -> &'static str {
        "DialogStarted"
    }
}

/// Dialog ended event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogEnded {
    pub dialog_id: Uuid,
    pub ended_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub final_metrics: ConversationMetrics,
}

impl DomainEvent for DialogEnded {
    fn subject(&self) -> String {
        "dialog.ended.v1".to_string()
    }

    fn aggregate_id(&self) -> Uuid {
        self.dialog_id
    }

    fn event_type(&self) -> &'static str {
        "DialogEnded"
    }
}

/// Turn added to dialog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnAdded {
    pub dialog_id: Uuid,
    pub turn: Turn,
    pub turn_number: u32,
}

impl DomainEvent for TurnAdded {
    fn subject(&self) -> String {
        "dialog.turn.added.v1".to_string()
    }

    fn aggregate_id(&self) -> Uuid {
        self.dialog_id
    }

    fn event_type(&self) -> &'static str {
        "TurnAdded"
    }
}

/// Context switched event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSwitched {
    pub dialog_id: Uuid,
    pub previous_topic: Option<Uuid>,
    pub new_topic: Topic,
    pub switched_at: DateTime<Utc>,
}

impl DomainEvent for ContextSwitched {
    fn subject(&self) -> String {
        "dialog.context.switched.v1".to_string()
    }

    fn aggregate_id(&self) -> Uuid {
        self.dialog_id
    }

    fn event_type(&self) -> &'static str {
        "ContextSwitched"
    }
}

/// Context updated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUpdated {
    pub dialog_id: Uuid,
    pub updated_variables: HashMap<String, serde_json::Value>,
    pub updated_at: DateTime<Utc>,
}

impl DomainEvent for ContextUpdated {
    fn subject(&self) -> String {
        "dialog.context.updated.v1".to_string()
    }

    fn aggregate_id(&self) -> Uuid {
        self.dialog_id
    }

    fn event_type(&self) -> &'static str {
        "ContextUpdated"
    }
}

/// Dialog paused event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogPaused {
    pub dialog_id: Uuid,
    pub paused_at: DateTime<Utc>,
    pub context_snapshot: HashMap<String, ContextVariable>,
}

impl DomainEvent for DialogPaused {
    fn subject(&self) -> String {
        "dialog.paused.v1".to_string()
    }

    fn aggregate_id(&self) -> Uuid {
        self.dialog_id
    }

    fn event_type(&self) -> &'static str {
        "DialogPaused"
    }
}

/// Dialog resumed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogResumed {
    pub dialog_id: Uuid,
    pub resumed_at: DateTime<Utc>,
}

impl DomainEvent for DialogResumed {
    fn subject(&self) -> String {
        "dialog.resumed.v1".to_string()
    }

    fn aggregate_id(&self) -> Uuid {
        self.dialog_id
    }

    fn event_type(&self) -> &'static str {
        "DialogResumed"
    }
}

/// Dialog metadata set event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogMetadataSet {
    pub dialog_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub set_at: DateTime<Utc>,
}

impl DomainEvent for DialogMetadataSet {
    fn subject(&self) -> String {
        "dialog.metadata.set.v1".to_string()
    }

    fn aggregate_id(&self) -> Uuid {
        self.dialog_id
    }

    fn event_type(&self) -> &'static str {
        "DialogMetadataSet"
    }
}

/// Participant added event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantAdded {
    pub dialog_id: Uuid,
    pub participant: Participant,
    pub added_at: DateTime<Utc>,
}

impl DomainEvent for ParticipantAdded {
    fn subject(&self) -> String {
        "dialog.participant.added.v1".to_string()
    }

    fn aggregate_id(&self) -> Uuid {
        self.dialog_id
    }

    fn event_type(&self) -> &'static str {
        "ParticipantAdded"
    }
}

/// Participant removed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantRemoved {
    pub dialog_id: Uuid,
    pub participant_id: Uuid,
    pub removed_at: DateTime<Utc>,
    pub reason: Option<String>,
}

impl DomainEvent for ParticipantRemoved {
    fn subject(&self) -> String {
        "dialog.participant.removed.v1".to_string()
    }

    fn aggregate_id(&self) -> Uuid {
        self.dialog_id
    }

    fn event_type(&self) -> &'static str {
        "ParticipantRemoved"
    }
}

/// Topic completed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCompleted {
    pub dialog_id: Uuid,
    pub topic_id: Uuid,
    pub completed_at: DateTime<Utc>,
    pub resolution: Option<String>,
}

impl DomainEvent for TopicCompleted {
    fn subject(&self) -> String {
        "dialog.topic.completed.v1".to_string()
    }

    fn aggregate_id(&self) -> Uuid {
        self.dialog_id
    }

    fn event_type(&self) -> &'static str {
        "TopicCompleted"
    }
}

/// Context variable added event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextVariableAdded {
    pub dialog_id: Uuid,
    pub variable: ContextVariable,
    pub added_at: DateTime<Utc>,
}

impl DomainEvent for ContextVariableAdded {
    fn subject(&self) -> String {
        "dialog.context.variable.added.v1".to_string()
    }

    fn aggregate_id(&self) -> Uuid {
        self.dialog_id
    }

    fn event_type(&self) -> &'static str {
        "ContextVariableAdded"
    }
} 