//! Event aggregators that convert presentation events into domain commands
//!
//! These aggregators collect multiple presentation events and determine
//! when to create domain commands, following our architectural principle
//! that NOT EVERY EVENT IS A DOMAIN EVENT.

pub mod drag;
pub mod layout;
pub mod selection;

pub use drag::DragAggregator;
pub use layout::LayoutAggregator;
pub use selection::SelectionAggregator;

use bevy::prelude::*;
use crate::domain::commands::DomainCommand;

/// Trait for all event aggregators
pub trait EventAggregator: Send + Sync + 'static {
    /// Process a presentation event
    fn process_event(&mut self, event: &dyn std::any::Any) -> Option<DomainCommand>;

    /// Force completion of aggregation (e.g., on save)
    fn complete(&mut self) -> Option<DomainCommand>;

    /// Reset the aggregator state
    fn reset(&mut self);

    /// Check if aggregator has pending changes
    fn has_pending_changes(&self) -> bool;
}

/// Resource that manages all aggregators
#[derive(Resource)]
pub struct AggregatorManager {
    pub drag: DragAggregator,
    pub layout: LayoutAggregator,
    pub selection: SelectionAggregator,
}

impl Default for AggregatorManager {
    fn default() -> Self {
        Self {
            drag: DragAggregator::new(),
            layout: LayoutAggregator::new(),
            selection: SelectionAggregator::new(),
        }
    }
}

impl AggregatorManager {
    /// Check if any aggregator has pending changes
    pub fn has_pending_changes(&self) -> bool {
        self.drag.has_pending_changes() ||
        self.layout.has_pending_changes() ||
        self.selection.has_pending_changes()
    }

    /// Complete all aggregations and return domain commands
    pub fn complete_all(&mut self) -> Vec<DomainCommand> {
        let mut commands = Vec::new();

        if let Some(cmd) = self.drag.complete() {
            commands.push(cmd);
        }

        if let Some(cmd) = self.layout.complete() {
            commands.push(cmd);
        }

        if let Some(cmd) = self.selection.complete() {
            commands.push(cmd);
        }

        commands
    }
}
