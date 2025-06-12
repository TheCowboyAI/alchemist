//! GraphComposition Library
//!
//! A standalone library for composable graph structures based on category theory.
//! This library provides the foundation for building domain models as graphs
//! that can be composed, transformed, and reasoned about mathematically.

pub mod base_types;
pub mod composition;
pub mod mapping;

// Re-export main types
pub use base_types::*;
pub use composition::*;
pub use mapping::*;
