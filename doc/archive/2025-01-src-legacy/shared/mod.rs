//! Shared kernel - minimal cross-context types and infrastructure
//!
//! This module contains only the absolute minimum shared between contexts.
//! Each bounded context should define its own domain-specific types.

pub mod types;
pub mod events;

// Re-export commonly used types
pub use types::{GraphId, NodeId, EdgeId, Position3D, Timestamp, Result, Error};
pub use events::{EventId, CorrelationId, CausationId, EventMetadata, DomainEvent, EventEnvelope};
