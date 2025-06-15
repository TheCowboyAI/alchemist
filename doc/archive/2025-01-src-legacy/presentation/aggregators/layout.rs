//! Layout aggregator - converts layout algorithm iterations into final positions
//!
//! Force-directed layouts may run hundreds of iterations. We aggregate these
//! and only send the final positions to the domain when the layout completes.

use super::EventAggregator;
use crate::domain::{
    commands::{DomainCommand, RecognizeGraphModel, UpdateNodePositions},
    value_objects::{GraphModel, NodeId, Position3D},
};
use crate::presentation::events::layout::{
    ConceptualForceUpdate, ForceLayoutIteration, LayoutComplete, LayoutType, TemporaryNodePosition,
};
use bevy::prelude::*;
use std::collections::HashMap;

/// Aggregates layout calculations into position update commands
pub struct LayoutAggregator {
    /// Current layout operation
    active_layout: Option<ActiveLayout>,
    /// Accumulated position updates
    position_updates: HashMap<NodeId, Vec3>,
    /// Detected graph model during layout
    detected_model: Option<GraphModel>,
}

struct ActiveLayout {
    #[allow(dead_code)]
    layout_type: LayoutType,
    #[allow(dead_code)]
    start_time: f32,
    iterations: u32,
    total_energy: f32,
}

impl Default for LayoutAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutAggregator {
    pub fn new() -> Self {
        Self {
            active_layout: None,
            position_updates: HashMap::new(),
            detected_model: None,
        }
    }

    pub fn handle_layout_iteration(&mut self, event: &ForceLayoutIteration) {
        // Start tracking if this is the first iteration
        if self.active_layout.is_none() {
            self.active_layout = Some(ActiveLayout {
                layout_type: LayoutType::ForceDirected,
                start_time: 0.0, // Would be set from Time resource
                iterations: 0,
                total_energy: event.total_energy,
            });
        }

        // Update active layout
        if let Some(layout) = &mut self.active_layout {
            layout.iterations = event.iteration;
            layout.total_energy = event.total_energy;
        }

        // Accumulate position updates
        for (node_id, _, new_pos) in &event.node_updates {
            self.position_updates.insert(*node_id, *new_pos);
        }
    }

    pub fn handle_layout_complete(&mut self, event: &LayoutComplete) -> Vec<DomainCommand> {
        let mut commands = Vec::new();

        // Create position update command
        if !event.final_positions.is_empty() {
            let updates: Vec<(NodeId, Position3D)> = event
                .final_positions
                .iter()
                .map(|(node_id, pos)| {
                    (
                        *node_id,
                        Position3D {
                            x: pos.x,
                            y: pos.y,
                            z: pos.z,
                        },
                    )
                })
                .collect();

            let reason = format!(
                "{:?} layout completed after {} iterations",
                event.layout_type, event.iterations_performed
            );

            commands.push(DomainCommand::UpdateNodePositions(UpdateNodePositions {
                updates,
                reason,
            }));
        }

        // If we detected a graph model during layout, recognize it
        if let Some(model) = &self.detected_model {
            commands.push(DomainCommand::RecognizeGraphModel(RecognizeGraphModel {
                model: model.clone(),
                confidence: 0.95, // Would be calculated based on layout convergence
            }));
        }

        // Reset state
        self.active_layout = None;
        self.position_updates.clear();
        self.detected_model = None;

        commands
    }

    pub fn handle_temporary_position(&mut self, event: &TemporaryNodePosition) {
        if !event.is_preview {
            self.position_updates.insert(event.node_id, event.position);
        }
    }

    pub fn handle_conceptual_force(&mut self, event: &ConceptualForceUpdate) {
        // Conceptual forces might help us recognize graph models
        // For example, if all nodes have equal semantic similarity,
        // it might be a complete graph (Kn)

        // This is a simplified example - real implementation would
        // analyze the force patterns to detect models
        if event.semantic_similarity > 0.9 {
            // High similarity between all nodes might indicate Kn
            self.detected_model = Some(GraphModel::CompleteGraph { order: 0 });
        }
    }
}

impl EventAggregator for LayoutAggregator {
    fn process_event(&mut self, event: &dyn std::any::Any) -> Option<DomainCommand> {
        if let Some(iteration) = event.downcast_ref::<ForceLayoutIteration>() {
            self.handle_layout_iteration(iteration);
            None
        } else if let Some(complete) = event.downcast_ref::<LayoutComplete>() {
            // Layout complete can generate multiple commands
            let commands = self.handle_layout_complete(complete);

            // Return the first command, queue others
            // In real implementation, we'd handle multiple commands properly
            commands.into_iter().next()
        } else if let Some(temp_pos) = event.downcast_ref::<TemporaryNodePosition>() {
            self.handle_temporary_position(temp_pos);
            None
        } else if let Some(force) = event.downcast_ref::<ConceptualForceUpdate>() {
            self.handle_conceptual_force(force);
            None
        } else {
            None
        }
    }

    fn complete(&mut self) -> Option<DomainCommand> {
        if self.position_updates.is_empty() {
            return None;
        }

        // Force completion with current positions
        let updates: Vec<(NodeId, Position3D)> = self
            .position_updates
            .iter()
            .map(|(node_id, pos)| {
                (
                    *node_id,
                    Position3D {
                        x: pos.x,
                        y: pos.y,
                        z: pos.z,
                    },
                )
            })
            .collect();

        let command = DomainCommand::UpdateNodePositions(UpdateNodePositions {
            updates,
            reason: "Layout aggregation completed".to_string(),
        });

        self.reset();
        Some(command)
    }

    fn reset(&mut self) {
        self.active_layout = None;
        self.position_updates.clear();
        self.detected_model = None;
    }

    fn has_pending_changes(&self) -> bool {
        !self.position_updates.is_empty()
    }
}
