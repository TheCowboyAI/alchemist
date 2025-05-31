//! Graph domain module
//!
//! This module contains all graph-related components, events, and systems
//! for the Information Alchemist ECS architecture.

pub mod components;
pub mod events;
pub mod plugin;
pub mod examples;

// Re-export commonly used types
pub use components::{
    Graph, GraphId, GraphMetadata, GraphBundle,
    GraphNode, NodeId, NodeBundle,
    GraphEdge, EdgeId, EdgeBundle,
    ElementState, Selectable, Draggable,
};

pub use events::{
    GraphCreatedEvent, GraphMetadataUpdatedEvent, GraphDeletedEvent,
    NodeAddedEvent, NodeUpdatedEvent, NodeRemovedEvent,
    EdgeCreatedEvent, EdgeUpdatedEvent, EdgeRemovedEvent,
    ElementSelectedEvent, ElementDeselectedEvent, MultipleElementsSelectedEvent,
    DragStartedEvent, DragUpdatedEvent, DragEndedEvent,
    LayoutAppliedEvent, GraphAnalysisCompletedEvent,
    BatchOperationEvent, GraphOperation,
    ElementType, ElementId,
};

pub use plugin::GraphPlugin;
