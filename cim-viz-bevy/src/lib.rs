//! CIM-Viz-Bevy: A Categorical Functor for Graph Visualization
//!
//! This library implements an isomorphic mapping between:
//! - **Bevy ECS Category**: Visual entities, components, and systems
//! - **CIM-ContextGraph Category**: Domain graph structures, nodes, and edges
//!
//! The functor preserves the categorical structure while enabling
//! high-performance visualization of domain graphs in Bevy applications.

pub mod bridge;
pub mod components;
pub mod events;
pub mod functors;
pub mod morphisms;
pub mod plugin;
pub mod resources;

// Re-export commonly used types
pub use components::*;
pub use events::*;
pub use plugin::*;
pub use resources::*;

// Re-export bridge types selectively to avoid conflicts
pub use bridge::{CategoricalBridge, BridgeError};

// Re-export functor types
pub use functors::{DomainToVisualFunctor, VisualToDomainFunctor};
