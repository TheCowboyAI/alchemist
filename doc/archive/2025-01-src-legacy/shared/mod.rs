//! Shared kernel - minimal cross-context types and infrastructure
//!
//! This module contains only the absolute minimum shared between contexts.
//! Each bounded context should define its own domain-specific types.

pub mod events;
pub mod types;

// Re-export commonly used types
pub use events::{CausationId, CorrelationId, DomainEvent, EventEnvelope, EventId, EventMetadata};
pub use types::{EdgeId, Error, GraphId, NodeId, Position3D, Result, Timestamp};
