//! Drag aggregator - converts multiple drag events into position updates
//!
//! During a drag operation, we may receive hundreds of position updates.
//! We aggregate these and only send the final position to the domain.

use super::EventAggregator;
use crate::domain::{
    commands::{DomainCommand, UpdateNodePositions},
    value_objects::{NodeId, Position3D},
};
use crate::presentation::events::interaction::{DragEnd, DragStart, DragUpdate};
use bevy::prelude::*;
use std::collections::HashMap;

/// Aggregates drag operations into position update commands
pub struct DragAggregator {
    /// Active drag operations by entity
    active_drags: HashMap<Entity, DragState>,
    /// Accumulated position changes
    position_changes: HashMap<NodeId, (Position3D, Position3D)>, // (original, current)
}

struct DragState {
    node_id: NodeId,
    #[allow(dead_code)]
    start_position: Vec3,
    current_position: Vec3,
    is_multi_select: bool,
}

impl Default for DragAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl DragAggregator {
    pub fn new() -> Self {
        Self {
            active_drags: HashMap::new(),
            position_changes: HashMap::new(),
        }
    }

    pub fn handle_drag_start(&mut self, event: &DragStart) {
        if let Some(node_id) = event.node_id {
            let state = DragState {
                node_id,
                start_position: event.start_position,
                current_position: event.start_position,
                is_multi_select: event.is_multi_select,
            };

            self.active_drags.insert(event.entity, state);

            // Record original position
            let original = Position3D {
                x: event.start_position.x,
                y: event.start_position.y,
                z: event.start_position.z,
            };

            self.position_changes.insert(node_id, (original, original));
        }
    }

    pub fn handle_drag_update(&mut self, event: &DragUpdate) {
        if let Some(state) = self.active_drags.get_mut(&event.entity) {
            state.current_position = event.current_position;

            // Update accumulated position
            if let Some((original, _)) = self.position_changes.get(&state.node_id).cloned() {
                let current = Position3D {
                    x: event.current_position.x,
                    y: event.current_position.y,
                    z: event.current_position.z,
                };

                self.position_changes
                    .insert(state.node_id, (original, current));
            }

            // Handle multi-select drag using world delta
            if state.is_multi_select && event.node_id.is_some() {
                // In a real implementation, we'd track additional selected entities
                // For now, we'll just update the main dragged node
                // Additional nodes would be handled by a separate selection system
            }
        }
    }

    pub fn handle_drag_end(&mut self, event: &DragEnd) -> Option<DomainCommand> {
        self.active_drags.remove(&event.entity);

        // If all drags are complete and we have changes, create command
        if self.active_drags.is_empty() && !self.position_changes.is_empty() {
            return self.create_update_command();
        }

        None
    }

    fn create_update_command(&mut self) -> Option<DomainCommand> {
        if self.position_changes.is_empty() {
            return None;
        }

        // Only include nodes that actually moved
        let updates: Vec<(NodeId, Position3D)> = self
            .position_changes
            .iter()
            .filter(|(_, (original, current))| original != current)
            .map(|(node_id, (_, current))| (*node_id, *current))
            .collect();

        if updates.is_empty() {
            self.position_changes.clear();
            return None;
        }

        let command = DomainCommand::UpdateNodePositions(UpdateNodePositions {
            updates,
            reason: "User drag operation".to_string(),
        });

        self.position_changes.clear();
        Some(command)
    }
}

impl EventAggregator for DragAggregator {
    fn process_event(&mut self, event: &dyn std::any::Any) -> Option<DomainCommand> {
        if let Some(drag_start) = event.downcast_ref::<DragStart>() {
            self.handle_drag_start(drag_start);
            None
        } else if let Some(drag_update) = event.downcast_ref::<DragUpdate>() {
            self.handle_drag_update(drag_update);
            None
        } else if let Some(drag_end) = event.downcast_ref::<DragEnd>() {
            self.handle_drag_end(drag_end)
        } else {
            None
        }
    }

    fn complete(&mut self) -> Option<DomainCommand> {
        // Force completion of any pending drags
        self.active_drags.clear();
        self.create_update_command()
    }

    fn reset(&mut self) {
        self.active_drags.clear();
        self.position_changes.clear();
    }

    fn has_pending_changes(&self) -> bool {
        !self.position_changes.is_empty()
    }
}
