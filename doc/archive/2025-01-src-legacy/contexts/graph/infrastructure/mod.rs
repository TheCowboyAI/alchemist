//! Graph Context Infrastructure Layer
//!
//! This layer contains implementations for external dependencies like
//! event stores, repositories, and external service integrations.

pub mod event_store;
pub mod factories;
pub mod repositories;

// Re-export commonly used types
pub use event_store::GraphEventStore;
pub use factories::DefaultGraphFactory;

// Re-export from application layer where it's defined
pub use crate::contexts::graph::application::GraphRepository;
