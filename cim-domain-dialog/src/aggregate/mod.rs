//! Dialog aggregate - represents a conversation between participants
//!
//! Dialogs are the core aggregate for managing conversations. They track:
//! - Multiple participants (users, agents, systems)
//! - Turn-by-turn conversation flow
//! - Context and state management
//! - Topic tracking and relevance

use cim_domain::{
    AggregateRoot, DomainError, DomainResult, DomainEvent,
    Entity, EntityId,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::value_objects::{
    Turn, Topic, Participant, ContextVariable,
    ConversationMetrics, TopicStatus,
};

/// Marker type for Dialog entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DialogMarker;

/// Dialog aggregate root
#[derive(Debug, Clone)]
pub struct Dialog {
    /// Entity base
    entity: Entity<DialogMarker>,
    
    /// Dialog type
    dialog_type: DialogType,
    
    /// Current status
    status: DialogStatus,
    
    /// Participants in the dialog
    participants: HashMap<Uuid, Participant>,
    
    /// Primary participant (initiator)
    primary_participant: Uuid,
    
    /// Conversation context
    context: ConversationContext,
    
    /// Turns in the conversation
    turns: Vec<Turn>,
    
    /// Active topics
    topics: HashMap<Uuid, Topic>,
    
    /// Current active topic
    current_topic: Option<Uuid>,
    
    /// Conversation metrics
    metrics: ConversationMetrics,
    
    /// Dialog metadata
    metadata: HashMap<String, serde_json::Value>,
    
    /// Version for optimistic concurrency
    version: u64,
}

/// Types of dialogs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DialogType {
    /// One-on-one conversation
    Direct,
    /// Multi-party conversation
    Group,
    /// Support/help conversation
    Support,
    /// Task-oriented dialog
    Task,
    /// Social/casual conversation
    Social,
    /// System interaction
    System,
}

/// Dialog operational status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DialogStatus {
    /// Dialog is active
    Active,
    /// Dialog is paused
    Paused,
    /// Dialog has ended
    Ended,
    /// Dialog was abandoned
    Abandoned,
}

/// Conversation context management
#[derive(Debug, Clone)]
pub struct ConversationContext {
    /// Current context state
    pub state: ContextState,
    
    /// Context variables
    pub variables: HashMap<String, ContextVariable>,
    
    /// Context history (for backtracking)
    pub history: Vec<ContextSnapshot>,
    
    /// Maximum history size
    pub max_history: usize,
}

/// State of the conversation context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextState {
    /// Normal conversation flow
    Normal,
    /// Waiting for clarification
    AwaitingClarification,
    /// Processing complex request
    Processing,
    /// Error state
    Error,
}

/// Snapshot of context at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// When snapshot was taken
    pub timestamp: DateTime<Utc>,
    /// Turn number at snapshot
    pub turn_number: u32,
    /// Active topic at snapshot
    pub active_topic: Option<Uuid>,
    /// Variables at snapshot
    pub variables: HashMap<String, ContextVariable>,
}

impl Dialog {
    /// Create a new dialog
    pub fn new(
        id: Uuid,
        dialog_type: DialogType,
        primary_participant: Participant,
    ) -> Self {
        let mut participants = HashMap::new();
        participants.insert(primary_participant.id, primary_participant.clone());
        
        Self {
            entity: Entity::with_id(EntityId::from_uuid(id)),
            dialog_type,
            status: DialogStatus::Active,
            participants,
            primary_participant: primary_participant.id,
            context: ConversationContext {
                state: ContextState::Normal,
                variables: HashMap::new(),
                history: Vec::new(),
                max_history: 10,
            },
            turns: Vec::new(),
            topics: HashMap::new(),
            current_topic: None,
            metrics: ConversationMetrics {
                turn_count: 0,
                avg_response_time_ms: 0.0,
                topic_switches: 0,
                clarification_count: 0,
                sentiment_trend: 0.0,
                coherence_score: 1.0,
            },
            metadata: HashMap::new(),
            version: 0,
        }
    }
    
    /// Get the dialog's ID
    pub fn id(&self) -> Uuid {
        *self.entity.id.as_uuid()
    }
    
    /// Get the dialog type
    pub fn dialog_type(&self) -> DialogType {
        self.dialog_type
    }
    
    /// Get the current status
    pub fn status(&self) -> DialogStatus {
        self.status
    }
    
    /// Get participants
    pub fn participants(&self) -> &HashMap<Uuid, Participant> {
        &self.participants
    }
    
    /// Get conversation context
    pub fn context(&self) -> &ConversationContext {
        &self.context
    }
    
    /// Get turns
    pub fn turns(&self) -> &[Turn] {
        &self.turns
    }
    
    /// Get current topic
    pub fn current_topic(&self) -> Option<&Topic> {
        self.current_topic.and_then(|id| self.topics.get(&id))
    }
    
    /// Add a participant to the dialog
    pub fn add_participant(&mut self, participant: Participant) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.status != DialogStatus::Active {
            return Err(DomainError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: "Active (required for adding participants)".to_string(),
            });
        }
        
        if self.participants.contains_key(&participant.id) {
            return Err(DomainError::ValidationError(
                "Participant already in dialog".to_string()
            ));
        }
        
        self.participants.insert(participant.id, participant.clone());
        self.entity.touch();
        self.version += 1;
        
        let event = crate::events::ParticipantAdded {
            dialog_id: self.id(),
            participant,
            added_at: Utc::now(),
        };
        
        Ok(vec![Box::new(event)])
    }
    
    /// Add a turn to the conversation
    pub fn add_turn(&mut self, turn: Turn) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.status != DialogStatus::Active {
            return Err(DomainError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: "Active (required for adding turns)".to_string(),
            });
        }
        
        if !self.participants.contains_key(&turn.participant_id) {
            return Err(DomainError::ValidationError(
                "Participant not in dialog".to_string()
            ));
        }
        
        // Update metrics
        self.metrics.turn_count += 1;
        
        // Add turn
        self.turns.push(turn.clone());
        self.entity.touch();
        self.version += 1;
        
        let event = crate::events::TurnAdded {
            dialog_id: self.id(),
            turn,
            turn_number: self.metrics.turn_count,
        };
        
        Ok(vec![Box::new(event)])
    }
    
    /// Switch to a new topic
    pub fn switch_topic(&mut self, topic: Topic) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.status != DialogStatus::Active {
            return Err(DomainError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: "Active (required for topic switching)".to_string(),
            });
        }
        
        // Mark current topic as paused if exists
        if let Some(current_id) = self.current_topic {
            if let Some(current) = self.topics.get_mut(&current_id) {
                current.status = TopicStatus::Paused;
            }
        }
        
        // Add new topic
        let topic_id = topic.id;
        self.topics.insert(topic_id, topic.clone());
        self.current_topic = Some(topic_id);
        
        // Update metrics
        self.metrics.topic_switches += 1;
        
        self.entity.touch();
        self.version += 1;
        
        let event = crate::events::ContextSwitched {
            dialog_id: self.id(),
            previous_topic: self.current_topic,
            new_topic: topic,
            switched_at: Utc::now(),
        };
        
        Ok(vec![Box::new(event)])
    }
    
    /// Add a context variable
    pub fn add_context_variable(&mut self, variable: ContextVariable) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.status == DialogStatus::Ended || self.status == DialogStatus::Abandoned {
            return Err(DomainError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: "Active/Paused (required for context updates)".to_string(),
            });
        }
        
        self.context.variables.insert(variable.name.clone(), variable.clone());
        self.entity.touch();
        self.version += 1;
        
        let event = crate::events::ContextVariableAdded {
            dialog_id: self.id(),
            variable,
            added_at: Utc::now(),
        };
        
        Ok(vec![Box::new(event)])
    }
    
    /// Pause the dialog
    pub fn pause(&mut self) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.status != DialogStatus::Active {
            return Err(DomainError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: "Paused".to_string(),
            });
        }
        
        // Take context snapshot
        let snapshot = ContextSnapshot {
            timestamp: Utc::now(),
            turn_number: self.metrics.turn_count,
            active_topic: self.current_topic,
            variables: self.context.variables.clone(),
        };
        
        self.context.history.push(snapshot);
        if self.context.history.len() > self.context.max_history {
            self.context.history.remove(0);
        }
        
        self.status = DialogStatus::Paused;
        self.entity.touch();
        self.version += 1;
        
        let event = crate::events::DialogPaused {
            dialog_id: self.id(),
            paused_at: Utc::now(),
            context_snapshot: self.context.variables.clone(),
        };
        
        Ok(vec![Box::new(event)])
    }
    
    /// Resume the dialog
    pub fn resume(&mut self) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.status != DialogStatus::Paused {
            return Err(DomainError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: "Active".to_string(),
            });
        }
        
        self.status = DialogStatus::Active;
        self.entity.touch();
        self.version += 1;
        
        let event = crate::events::DialogResumed {
            dialog_id: self.id(),
            resumed_at: Utc::now(),
        };
        
        Ok(vec![Box::new(event)])
    }
    
    /// End the dialog
    pub fn end(&mut self, reason: Option<String>) -> DomainResult<Vec<Box<dyn DomainEvent>>> {
        if self.status == DialogStatus::Ended || self.status == DialogStatus::Abandoned {
            return Err(DomainError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: "Ended".to_string(),
            });
        }
        
        self.status = DialogStatus::Ended;
        self.entity.touch();
        self.version += 1;
        
        let event = crate::events::DialogEnded {
            dialog_id: self.id(),
            ended_at: Utc::now(),
            reason,
            final_metrics: self.metrics.clone(),
        };
        
        Ok(vec![Box::new(event)])
    }
}

impl AggregateRoot for Dialog {
    type Id = EntityId<DialogMarker>;
    
    fn id(&self) -> Self::Id {
        self.entity.id
    }
    
    fn version(&self) -> u64 {
        self.version
    }
    
    fn increment_version(&mut self) {
        self.version += 1;
        self.entity.touch();
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self {
            state: ContextState::Normal,
            variables: HashMap::new(),
            history: Vec::new(),
            max_history: 10,
        }
    }
} 