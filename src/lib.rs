//! Information Alchemist - Event-Sourced Graph Editor
//!
//! A high-performance graph editor built with Event Sourcing, CQRS, and Bevy ECS.

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;

pub use domain::prelude::*;
pub use presentation::plugins::GraphEditorPlugin;
