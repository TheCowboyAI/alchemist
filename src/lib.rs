//! Information Alchemist - Event-Sourced Graph Editor
//!
//! A high-performance graph editor built with Event Sourcing, CQRS, and Bevy ECS.

pub mod domain;
pub mod infrastructure;
pub mod application;
pub mod presentation;

pub use domain::prelude::*;
pub use presentation::GraphEditorPlugin;
