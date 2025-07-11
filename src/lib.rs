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
pub mod deployment_automation;
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
pub mod dashboard_minimal;
pub mod dashboard_events;
pub mod dashboard_realtime;
pub mod dashboard_window;
pub mod dialog_window;
pub mod dialog_window_minimal;
pub mod dialog_handler;
pub mod launcher;
pub mod launcher_enhanced;
pub mod launcher_simple;
pub mod dashboard_nats_stream;
pub mod nats_dashboard_connector;
pub mod renderer_nats_bridge;
pub mod system_monitor;
pub mod rss_feed_manager;
pub mod nlp_processor;
pub mod rss_feed_processor;
pub mod renderer_comm;
pub mod renderer_events;
pub mod policy_engine;
pub mod workflow;
pub mod nix_deployment;
pub mod nats_client;
pub mod error;
pub mod event_monitor;
pub mod event_visualizer;
pub mod performance_monitor_ui;
pub mod deployment_ui;
pub mod workflow_editor;
pub mod nats_flow_visualizer;
pub mod settings;
pub mod graph_parser;

// Bevy-dependent modules
#[cfg(feature = "bevy")]
pub mod graph_components;
#[cfg(feature = "bevy")]
pub mod graph_systems;
#[cfg(feature = "bevy")]
pub mod graph_algorithms;
#[cfg(feature = "bevy")]
pub mod graph_plugin;

#[cfg(feature = "bevy")]
pub mod jetstream_persistence;

#[cfg(feature = "bevy")]
use bevy::prelude::*;

/// Wrapper for tokio runtime to make it a Bevy Resource
#[cfg(feature = "bevy")]
#[derive(Resource)]
pub struct TokioRuntime(pub tokio::runtime::Runtime);

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