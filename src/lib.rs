//! # Information Alchemist
//!
//! A graph editor and workflow manager for Domain Driven Design.
//!
//! This application provides:
//! - Visual graph editing capabilities
//! - Domain-driven workflow management
//! - Event sourcing and CQRS patterns
//! - Integration with NATS messaging
//! - Conceptual space visualization

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

// Re-export core dependencies
pub use bevy;
pub use bevy_egui;

// Re-export domain modules
pub use cim_domain;
pub use cim_domain_agent;
pub use cim_domain_document;
pub use cim_domain_git;
pub use cim_domain_graph;
pub use cim_domain_location;
pub use cim_domain_nix;
pub use cim_domain_organization;
pub use cim_domain_person;
pub use cim_domain_policy;
pub use cim_domain_workflow;

// Re-export infrastructure
pub use cim_ipld;
pub use cim_keys;
pub use cim_subject;

// Re-export graph modules
pub use cim_compose;
pub use cim_contextgraph;
pub use cim_ipld_graph;
pub use cim_workflow_graph;

// Application modules
pub mod graph;
pub mod workflow;
pub mod plugins;
pub mod simple_agent;

/// Prelude module for convenient imports
pub mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_egui::*;
}
