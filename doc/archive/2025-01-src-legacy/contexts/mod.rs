//! Bounded Contexts
//!
//! Each module represents a bounded context with its own domain model,
//! application services, and infrastructure. All contexts are independent
//! of the presentation layer (Bevy).

pub mod graph; // Foundation context for all graphs

// Future contexts will be added here:
// pub mod ddd;        // DDD modeling context
// pub mod workflow;   // Workflow management context
// pub mod conceptual; // Conceptual space context
