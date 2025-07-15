//! Real-time collaboration domain for multi-user graph editing
//!
//! This domain provides WebSocket-based real-time collaboration features
//! for the graph visualization system, including presence tracking,
//! conflict resolution, and synchronized editing.

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod websocket;

// Re-export main types
pub use aggregate::{CollaborationSession, UserId};
pub use commands::{CollaborationCommand, CollaborationCommandError};
pub use events::{CollaborationEvent, UserPresence, CursorPosition};
pub use handlers::CollaborationCommandHandler;
pub use websocket::{CollaborationServer, CollaborationMessage};