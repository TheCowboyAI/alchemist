//! Visualization context for rendering graph elements
//!
//! This context handles the visual representation of graphs, nodes, and edges.

pub mod camera;
pub mod layout;
pub mod plugin;
pub mod point_cloud;
pub mod services;

pub use plugin::VisualizationPlugin;

#[cfg(test)]
mod tests;
