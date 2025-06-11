//! Information Alchemist Library
//!
//! A minimal shell ready for domain integration

// Re-export Bevy for convenience
pub use bevy;

// Placeholder modules - to be implemented
pub mod application {
    //! Application layer - command handlers, queries, services
}

pub mod domain {
    //! Domain layer - aggregates, entities, value objects
}

pub mod infrastructure;
pub mod presentation;
