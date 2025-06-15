//! Shared types used across all bounded contexts
//!
//! This module contains only the minimal shared kernel - types that are truly
//! shared across context boundaries. Each context should define its own types
//! when possible to maintain proper boundaries.

use serde::{Deserialize, Serialize};

// Re-export the base types from graph-composition
pub use graph_composition::{EdgeId, GraphId, NodeId};

/// 3D position used for visualization (not domain logic!)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

/// Timestamp for events and auditing
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Result type used across contexts
pub type Result<T> = std::result::Result<T, Error>;

/// Shared error type (contexts can extend with their own)
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Infrastructure error: {0}")]
    Infrastructure(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generation() {
        let graph_id1 = GraphId::new();
        let graph_id2 = GraphId::new();
        assert_ne!(graph_id1, graph_id2);

        let node_id = NodeId::new();
        let edge_id = EdgeId::new();

        // IDs should be unique
        assert_ne!(node_id.as_uuid(), edge_id.as_uuid());
    }

    #[test]
    fn test_position_creation() {
        let pos = Position3D::new(1.0, 2.0, 3.0);
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        assert_eq!(pos.z, 3.0);

        let zero = Position3D::zero();
        assert_eq!(zero.x, 0.0);
        assert_eq!(zero.y, 0.0);
        assert_eq!(zero.z, 0.0);
    }

    #[test]
    fn test_serialization() {
        let graph_id = GraphId::new();
        let json = serde_json::to_string(&graph_id).unwrap();
        let deserialized: GraphId = serde_json::from_str(&json).unwrap();
        assert_eq!(graph_id, deserialized);
    }
}
