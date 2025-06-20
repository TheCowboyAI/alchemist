//! Dialog domain module
//!
//! This domain manages conversations, dialog contexts, and interaction history
//! for AI agents. It provides:
//! - Multi-turn conversation tracking with context preservation
//! - Topic management and semantic understanding
//! - Context switching and state management
//! - Participant tracking (agents, users, systems)
//! - Integration with conceptual spaces for semantic analysis
//!
//! The Dialog domain serves as the memory system for agent conversations,
//! storing both the structure (turns, topics) and semantics (embeddings, context)
//! of interactions.

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;

// Re-export main types
pub use aggregate::{
    Dialog, DialogMarker, DialogStatus, DialogType,
    ConversationContext, ContextState,
};

pub use commands::{
    StartDialog, EndDialog, AddTurn, SwitchContext,
    UpdateContext, PauseDialog, ResumeDialog,
    SetDialogMetadata, AddParticipant, RemoveParticipant,
    MarkTopicComplete, AddContextVariable,
};

pub use events::{
    DialogStarted, DialogEnded, TurnAdded, ContextSwitched,
    ContextUpdated, DialogPaused, DialogResumed,
    DialogMetadataSet, ParticipantAdded, ParticipantRemoved,
    TopicCompleted, ContextVariableAdded,
};

pub use handlers::{DialogCommandHandler, DialogEventHandler};
pub use projections::{DialogView, ConversationHistory, ActiveDialogs};
pub use queries::{DialogQuery, DialogQueryHandler};

pub use value_objects::{
    Turn, TurnType, TurnMetadata,
    Participant, ParticipantType, ParticipantRole,
    Message, MessageContent, MessageIntent,
    Topic, TopicStatus, TopicRelevance,
    ContextVariable, ContextScope,
    ConversationMetrics, EngagementMetrics,
}; 