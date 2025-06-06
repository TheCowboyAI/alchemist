//! Presentation-layer events that stay within Bevy
//!
//! These events are ephemeral, UI-specific, and NEVER sent to NATS.
//! They represent visual state changes, animations, and user interactions
//! that don't constitute business-meaningful state changes.

pub mod animation;
pub mod interaction;
pub mod layout;

pub use animation::*;
pub use interaction::*;
pub use layout::*;

use bevy::prelude::*;

/// Marker trait for presentation events
pub trait PresentationEvent: Event + Clone + Send + Sync + 'static {
    /// Whether this event should be aggregated before domain conversion
    fn requires_aggregation(&self) -> bool {
        true
    }
}

/// Event fired when multiple presentation events should be aggregated
/// into a single domain command
#[derive(Event, Clone, Debug)]
pub struct AggregationComplete {
    pub aggregation_type: AggregationType,
    pub entity_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AggregationType {
    DragOperation,
    LayoutCalculation,
    BatchSelection,
    AnimationSequence,
}
