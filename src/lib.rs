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
// pub use cim_compose;  // Temporarily disabled for build
pub use cim_contextgraph;
pub use cim_ipld_graph;
pub use cim_workflow_graph;

// Application modules
pub mod graph;
pub mod plugins;
pub mod simple_agent;
pub mod workflow;

/// Prelude module for convenient imports
pub mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_egui::*;
}

// Agent domain module
//
// This module contains all agent-related domain logic including:
// - Agent aggregate and components
// - Agent commands and events
// - Agent command and query handlers

/// Aggregate root definitions for domain entities
pub mod aggregate;
/// Command definitions for domain operations
pub mod commands;
/// Domain event definitions
pub mod events;
/// Command and event handlers
pub mod handlers;
/// Read model projections
pub mod projections;
/// Query definitions and handlers
pub mod queries;
/// Value objects for domain modeling
pub mod value_objects;

// ECS modules
/// ECS component definitions
pub mod components;
/// ECS system implementations
pub mod systems;

// Semantic search module
/// Semantic search capabilities for CIM
pub mod semantic_search;

// Monitoring module
/// Event-based monitoring for CIM
pub mod monitoring;

// Re-export main types
pub use aggregate::{
    Agent, AgentMarker, AgentMetadata, AgentStatus, AuthenticationComponent, CapabilitiesComponent,
    ConfigurationComponent, PermissionsComponent, ToolAccessComponent, ToolDefinition,
    ToolUsageStats,
};

pub use commands::{
    ActivateAgent, ChangeAgentCapabilities, DecommissionAgent, DeployAgent, DisableAgentTools,
    EnableAgentTools, GrantAgentPermissions, RemoveAgentConfiguration, RevokeAgentPermissions,
    SetAgentConfiguration, SetAgentOffline, SuspendAgent,
};

pub use events::{
    AgentActivated, AgentCapabilitiesAdded, AgentCapabilitiesRemoved, AgentConfigurationRemoved,
    AgentConfigurationSet, AgentDecommissioned, AgentDeployed, AgentPermissionsGranted,
    AgentPermissionsRevoked, AgentSuspended, AgentToolsDisabled, AgentToolsEnabled,
    AgentWentOffline,
};

pub use handlers::{AgentCommandHandler, AgentEventHandler};
pub use projections::AgentView;
pub use queries::{AgentQuery, AgentQueryHandler};

// Re-export value objects
pub use value_objects::{AgentType, AuthMethod};

// Re-export ECS types
pub use components::*;
pub use systems::*;
