//! Graph plugin for Bevy integration
//!
//! This plugin registers all graph-related components, events, and systems
//! with the Bevy application.

use bevy::prelude::*;
use crate::graph::events::*;
use crate::graph::examples::{create_example_graph, handle_graph_events};

/// Plugin that adds graph functionality to the Bevy application
pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        // Register all graph events
        app
            // Graph lifecycle events
            .add_event::<GraphCreatedEvent>()
            .add_event::<GraphMetadataUpdatedEvent>()
            .add_event::<GraphDeletedEvent>()

            // Node events
            .add_event::<NodeAddedEvent>()
            .add_event::<NodeUpdatedEvent>()
            .add_event::<NodeRemovedEvent>()

            // Edge events
            .add_event::<EdgeCreatedEvent>()
            .add_event::<EdgeUpdatedEvent>()
            .add_event::<EdgeRemovedEvent>()

            // Interaction events
            .add_event::<ElementSelectedEvent>()
            .add_event::<ElementDeselectedEvent>()
            .add_event::<MultipleElementsSelectedEvent>()

            // Drag events
            .add_event::<DragStartedEvent>()
            .add_event::<DragUpdatedEvent>()
            .add_event::<DragEndedEvent>()

            // Analysis events
            .add_event::<LayoutAppliedEvent>()
            .add_event::<GraphAnalysisCompletedEvent>()

            // Batch operations
            .add_event::<BatchOperationEvent>()

            // Add systems in appropriate stages
            .add_systems(Startup, (
                setup_graph_resources,
                create_example_graph,
            ))
            .add_systems(Update, (
                handle_graph_events,
                // These systems will be implemented later
                // handle_node_operations,
                // handle_edge_operations,
                // handle_selection,
                // handle_dragging,
            ));
    }
}

/// System to initialize graph-related resources
fn setup_graph_resources(_commands: Commands) {
    // This will be expanded later to set up any necessary resources
    // For now, just log that the graph plugin is initialized
    info!("Graph plugin initialized");
}
