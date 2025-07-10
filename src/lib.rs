//! # Alchemist
//!
//! A shell-based control system for CIM (Content Information Model).
//!
//! This application provides:
//! - CLI shell for CIM control and AI dialog management
//! - Domain-driven workflow management  
//! - Policy and deployment management
//! - Progress tracking and monitoring

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

// Shell modules
pub mod ai;
pub mod config;
pub mod deployment;
pub mod dialog;
pub mod domain;
pub mod policy;
pub mod progress;
pub mod shell;
pub mod shell_commands;
pub mod shell_enhanced;
pub mod renderer;
pub mod renderer_api;
pub mod render_commands;
pub mod dashboard;
pub mod dashboard_events;
pub mod rss_feed_manager;

// Re-export for tests
pub use ai::{AiManager, ModelStatus, TestResult, StreamingResponse, StreamingResponseStream};

// Re-export domain modules from CIM
pub use cim_domain;
pub use cim_domain_document;
pub use cim_domain_location;
pub use cim_domain_nix;
pub use cim_domain_workflow;

// Re-export infrastructure
pub use cim_ipld;
pub use cim_keys;
pub use cim_subject;

// Re-export graph modules
pub use cim_contextgraph;
pub use cim_ipld_graph;
pub use cim_workflow_graph;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::shell::AlchemistShell;
    pub use crate::shell_commands::Commands;
    pub use crate::config::AlchemistConfig;
}