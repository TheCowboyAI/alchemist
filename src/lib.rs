//! Library target for bevy_dylib

// Re-export bevy crate to ensure all symbols are available
pub use bevy;

// Re-export all the main bevy modules that are available in v0.16.0
pub use bevy::app;
pub use bevy::asset;
pub use bevy::audio;
pub use bevy::ecs;
pub use bevy::input;
pub use bevy::prelude::*;
pub use bevy::render;
pub use bevy::sprite;
pub use bevy::text;
pub use bevy::time;
pub use bevy::transform;
pub use bevy::ui;
pub use bevy::window;

// Export our modules
pub mod camera;
pub mod events;
pub mod graph;
pub mod graph_core;
pub mod graph_patterns;
pub mod models;

// Re-export commonly used types
pub use camera::CameraViewportPlugin;
pub use graph_core::GraphPlugin;

/// A dummy function to ensure we have a valid lib target
pub fn alchemist_graph() -> &'static str {
    "Alchemist Graph System: This function is a placeholder for the bevy_dylib target"
}
