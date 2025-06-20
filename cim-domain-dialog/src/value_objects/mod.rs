//! Value objects for the Dialog domain

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// A single turn in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Turn {
    /// Unique identifier for this turn
    pub turn_id: Uuid,
    /// Sequential turn number in the dialog
    pub turn_number: u32,
    /// Who is speaking in this turn
    pub participant_id: Uuid,
    /// The message content
    pub message: Message,
    /// When this turn occurred
    pub timestamp: DateTime<Utc>,
    /// Metadata about this turn
    pub metadata: TurnMetadata,
}

/// Type of turn in a conversation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TurnType {
    /// User initiated turn
    UserQuery,
    /// Agent response
    AgentResponse,
    /// System message (notifications, status updates)
    SystemMessage,
    /// Clarification request
    Clarification,
    /// Feedback on previous turn
    Feedback,
}

/// Metadata associated with a turn
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurnMetadata {
    /// Type of this turn
    pub turn_type: TurnType,
    /// Confidence score for agent responses
    pub confidence: Option<f32>,
    /// Processing time in milliseconds
    pub processing_time_ms: Option<u64>,
    /// References to previous turns
    pub references: Vec<Uuid>,
    /// Custom properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// A participant in a dialog
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Participant {
    /// Unique identifier
    pub id: Uuid,
    /// Type of participant
    pub participant_type: ParticipantType,
    /// Role in the conversation
    pub role: ParticipantRole,
    /// Display name
    pub name: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Type of participant
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ParticipantType {
    /// Human user
    Human,
    /// AI agent
    AIAgent,
    /// System or service
    System,
    /// External integration
    External,
}

/// Role of participant in dialog
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ParticipantRole {
    /// Primary conversation initiator
    Primary,
    /// Supporting participant
    Assistant,
    /// Observer (read-only)
    Observer,
    /// Moderator with control privileges
    Moderator,
}

/// Message content in a turn
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// The actual content
    pub content: MessageContent,
    /// Intent of the message
    pub intent: Option<MessageIntent>,
    /// Language of the message
    pub language: String,
    /// Sentiment score (-1.0 to 1.0)
    pub sentiment: Option<f32>,
    /// Embeddings for semantic analysis
    pub embeddings: Option<Vec<f32>>,
}

/// Content of a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageContent {
    /// Plain text message
    Text(String),
    /// Structured data (JSON)
    Structured(serde_json::Value),
    /// Multimodal content
    Multimodal {
        text: Option<String>,
        data: HashMap<String, serde_json::Value>,
    },
}

/// Intent classification for messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MessageIntent {
    /// Asking a question
    Question,
    /// Providing an answer
    Answer,
    /// Making a statement
    Statement,
    /// Giving a command
    Command,
    /// Expressing acknowledgment
    Acknowledgment,
    /// Requesting clarification
    Clarification,
    /// Providing feedback
    Feedback,
    /// Social/greeting
    Social,
}

/// A topic within a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Topic {
    /// Unique identifier
    pub id: Uuid,
    /// Topic name/title
    pub name: String,
    /// Current status
    pub status: TopicStatus,
    /// Relevance to current context
    pub relevance: TopicRelevance,
    /// When topic was introduced
    pub introduced_at: DateTime<Utc>,
    /// Related topics
    pub related_topics: Vec<Uuid>,
    /// Keywords associated with topic
    pub keywords: Vec<String>,
    /// Conceptual space embedding
    pub embedding: Option<Vec<f32>>,
}

/// Status of a topic
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TopicStatus {
    /// Currently being discussed
    Active,
    /// Temporarily paused
    Paused,
    /// Completed/resolved
    Completed,
    /// Abandoned without resolution
    Abandoned,
}

/// Relevance score for a topic
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TopicRelevance {
    /// Score from 0.0 to 1.0
    pub score: f32,
    /// When last updated
    pub last_updated: DateTime<Utc>,
    /// Decay rate (relevance decreases over time)
    pub decay_rate: f32,
}

/// A context variable stored in the conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextVariable {
    /// Variable name
    pub name: String,
    /// Variable value
    pub value: serde_json::Value,
    /// Scope of the variable
    pub scope: ContextScope,
    /// When set
    pub set_at: DateTime<Utc>,
    /// Expiry time (if any)
    pub expires_at: Option<DateTime<Utc>>,
    /// Source that set this variable
    pub source: Uuid,
}

/// Scope of a context variable
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextScope {
    /// Available only in current turn
    Turn,
    /// Available for current topic
    Topic,
    /// Available for entire dialog
    Dialog,
    /// Persists across dialogs for participant
    Participant,
    /// Global scope
    Global,
}

/// Metrics about a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationMetrics {
    /// Total number of turns
    pub turn_count: u32,
    /// Average response time in ms
    pub avg_response_time_ms: f64,
    /// Number of topic switches
    pub topic_switches: u32,
    /// Number of clarifications needed
    pub clarification_count: u32,
    /// Overall sentiment trend
    pub sentiment_trend: f32,
    /// Conversation coherence score
    pub coherence_score: f32,
}

/// Engagement metrics for participants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngagementMetrics {
    /// Participant ID
    pub participant_id: Uuid,
    /// Number of turns contributed
    pub turn_contributions: u32,
    /// Average message length
    pub avg_message_length: f64,
    /// Response latency in ms
    pub avg_response_latency_ms: f64,
    /// Engagement score (0.0 to 1.0)
    pub engagement_score: f32,
    /// Topics initiated
    pub topics_initiated: u32,
}

impl Turn {
    /// Create a new turn
    pub fn new(
        turn_number: u32,
        participant_id: Uuid,
        message: Message,
        turn_type: TurnType,
    ) -> Self {
        Self {
            turn_id: Uuid::new_v4(),
            turn_number,
            participant_id,
            message,
            timestamp: Utc::now(),
            metadata: TurnMetadata {
                turn_type,
                confidence: None,
                processing_time_ms: None,
                references: Vec::new(),
                properties: HashMap::new(),
            },
        }
    }
}

impl Message {
    /// Create a simple text message
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: MessageContent::Text(content.into()),
            intent: None,
            language: "en".to_string(),
            sentiment: None,
            embeddings: None,
        }
    }

    /// Create a message with intent
    pub fn with_intent(mut self, intent: MessageIntent) -> Self {
        self.intent = Some(intent);
        self
    }

    /// Add embeddings to the message
    pub fn with_embeddings(mut self, embeddings: Vec<f32>) -> Self {
        self.embeddings = Some(embeddings);
        self
    }
}

impl Topic {
    /// Create a new topic
    pub fn new(name: impl Into<String>, keywords: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            status: TopicStatus::Active,
            relevance: TopicRelevance {
                score: 1.0,
                last_updated: Utc::now(),
                decay_rate: 0.1,
            },
            introduced_at: Utc::now(),
            related_topics: Vec::new(),
            keywords,
            embedding: None,
        }
    }

    /// Calculate current relevance considering decay
    pub fn current_relevance(&self) -> f32 {
        let elapsed = Utc::now()
            .signed_duration_since(self.relevance.last_updated)
            .num_seconds() as f32;
        
        let decayed = self.relevance.score * (-self.relevance.decay_rate * elapsed / 3600.0).exp();
        decayed.max(0.0).min(1.0)
    }
} 