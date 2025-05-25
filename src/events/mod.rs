//! # Event System for Alchemist Graph Editor
//!
//! This module provides a comprehensive event-driven architecture for cross-system communication.
//! Events enable decoupled, reactive systems that can respond to changes without direct dependencies.
//!
//! ## Event Categories
//!
//! ### Graph Events (`graph_events`)
//! - Node lifecycle: Creation, modification, deletion
//! - Edge lifecycle: Creation, modification, deletion
//! - Graph operations: Layout, validation, patterns
//! - Selection and interaction events
//!
//! ### UI Events (`ui_events`)
//! - Panel visibility toggles
//! - Workspace mode changes
//! - Context menus and tooltips
//! - Theme changes
//!
//! ### I/O Events (`io_events`)
//! - File loading and saving
//! - Import/export operations
//! - Auto-save triggers
//! - Operation completion notifications
//!
//! ### Camera Events (`camera_events`)
//! - View mode switching (2D/3D)
//! - Focus operations
//! - Camera position save/load
//! - Zoom and pan events
//!
//! ## Event Flow Patterns
//!
//! ### User Interaction → Event → System Response
//! ```text
//! User clicks node → SelectEvent → selection_system → Update Selected component
//! User drags node → MoveNodeEvent → movement_system → Update Transform
//! User saves file → SaveJsonFileEvent → file_system → Write to disk
//! ```
//!
//! ### System Chain Reactions
//! ```text
//! CreateNodeEvent → node_creation_system → GraphModificationEvent → undo_system
//! DeleteNodeEvent → node_deletion_system → DeleteEdgeEvent (for connected edges)
//! LoadJsonFileEvent → file_loading_system → Multiple CreateNodeEvent + CreateEdgeEvent
//! ```
//!
//! ## Migration Guide: Direct Mutations to Events
//!
//! ### Before (Direct Mutation):
//! ```rust
//! // DON'T: Direct component modification
//! if let Ok(mut transform) = transforms.get_mut(entity) {
//!     transform.translation = new_position;
//! }
//! ```
//!
//! ### After (Event-Driven):
//! ```rust
//! // DO: Send event for system to handle
//! move_events.send(MoveNodeEvent {
//!     entity,
//!     from: old_position,
//!     to: new_position,
//! });
//! ```
//!
//! ## Best Practices
//!
//! 1. **Single Responsibility**: Each event should represent one logical action
//! 2. **Immutable Data**: Events should contain immutable data snapshots
//! 3. **Validation**: Validate event data in the sending system when possible
//! 4. **Documentation**: Document what systems produce and consume each event
//! 5. **Debugging**: Use event readers to log event flow for debugging

pub mod graph_events;
pub mod ui_events;
pub mod io_events;
pub mod camera_events;

// Re-export commonly used events
pub use graph_events::*;
pub use ui_events::*;
pub use io_events::*;
pub use camera_events::*;

// Temporary compatibility layer for old event system
// TODO: Refactor graph.rs to use new event system
mod compat {
    use std::any::Any;
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub enum GraphEventType {
        NodeCreated,
        NodeUpdated,
        NodeDeleted,
        EdgeCreated,
        EdgeUpdated,
        EdgeDeleted,
        GraphCleared,
        WorkflowStepExecuted,
        WorkflowStateChanged,
    }

    #[derive(Debug, Clone)]
    pub struct GraphEvent {
        pub event_type: GraphEventType,
        pub entity_id: Option<Uuid>,
        pub payload: HashMap<String, String>,
        pub timestamp: u64,
    }

    impl GraphEvent {
        pub fn new(event_type: GraphEventType, entity_id: Option<Uuid>) -> Self {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            Self {
                event_type,
                entity_id,
                payload: HashMap::new(),
                timestamp,
            }
        }

        pub fn with_payload(mut self, key: &str, value: &str) -> Self {
            self.payload.insert(key.to_string(), value.to_string());
            self
        }
    }

    pub trait Model {
        fn apply_event(&mut self, event: &GraphEvent);
    }
}

pub use compat::{GraphEvent, GraphEventType, Model};
