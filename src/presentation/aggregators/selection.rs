//! Selection aggregator - tracks selection state changes
//!
//! Selection changes are presentation-only until the user performs
//! an action on the selection (delete, group, etc.)

use bevy::prelude::*;
use std::collections::HashSet;
use crate::domain::{
    commands::{DomainCommand, UpdateGraphSelection},
    value_objects::NodeId,
};
use crate::presentation::events::interaction::{SelectionChanged, SelectionCleared};
use super::EventAggregator;

/// Aggregates selection changes
pub struct SelectionAggregator {
    /// Currently selected nodes
    selected_nodes: HashSet<NodeId>,
    /// Whether selection has changed since last sync
    has_changed: bool,
}

impl SelectionAggregator {
    pub fn new() -> Self {
        Self {
            selected_nodes: HashSet::new(),
            has_changed: false,
        }
    }

    pub fn handle_selection_changed(&mut self, event: &SelectionChanged) {
        match event.selection_mode {
            SelectionMode::Replace => {
                self.selected_nodes.clear();
                self.selected_nodes.extend(&event.selected_nodes);
            }
            SelectionMode::Add => {
                self.selected_nodes.extend(&event.selected_nodes);
            }
            SelectionMode::Remove => {
                for node in &event.selected_nodes {
                    self.selected_nodes.remove(node);
                }
            }
            SelectionMode::Toggle => {
                for node in &event.selected_nodes {
                    if !self.selected_nodes.remove(node) {
                        self.selected_nodes.insert(*node);
                    }
                }
            }
        }

        self.has_changed = true;
    }

    pub fn handle_selection_cleared(&mut self) {
        if !self.selected_nodes.is_empty() {
            self.selected_nodes.clear();
            self.has_changed = true;
        }
    }

    pub fn get_selected_nodes(&self) -> &HashSet<NodeId> {
        &self.selected_nodes
    }
}

impl EventAggregator for SelectionAggregator {
    fn process_event(&mut self, event: &dyn std::any::Any) -> Option<DomainCommand> {
        if let Some(selection_changed) = event.downcast_ref::<SelectionChanged>() {
            self.handle_selection_changed(selection_changed);
        } else if let Some(_) = event.downcast_ref::<SelectionCleared>() {
            self.handle_selection_cleared();
        }

        // Selection changes don't immediately create domain commands
        // They only affect the domain when an action is performed
        None
    }

    fn complete(&mut self) -> Option<DomainCommand> {
        if !self.has_changed {
            return None;
        }

        // Only send selection to domain if explicitly requested (e.g., save)
        let command = DomainCommand::UpdateGraphSelection(UpdateGraphSelection {
            selected_nodes: self.selected_nodes.iter().cloned().collect(),
            reason: "Selection state persisted".to_string(),
        });

        self.has_changed = false;
        Some(command)
    }

    fn reset(&mut self) {
        self.selected_nodes.clear();
        self.has_changed = false;
    }

    fn has_pending_changes(&self) -> bool {
        self.has_changed
    }
}

#[derive(Clone, Debug)]
pub enum SelectionMode {
    Replace,  // Clear and select new
    Add,      // Add to selection
    Remove,   // Remove from selection
    Toggle,   // Toggle selection state
}
