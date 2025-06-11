//! Graph Context - Foundation for all ContextGraphs
//!
//! This bounded context manages the core graph functionality that all
//! other contexts build upon. It is completely independent of Bevy.

pub mod domain;
pub mod application;
pub mod infrastructure;
