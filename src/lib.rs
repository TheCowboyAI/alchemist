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
pub mod plugins;
pub mod simple_agent;
pub mod workflow;

/// Prelude module for convenient imports
pub mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_egui::*;
}

//! Agent domain module
//!
//! This module contains all agent-related domain logic including:
//! - Agent aggregate and components
//! - Agent commands and events
//! - Agent command and query handlers

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;

// ECS modules
pub mod components;
pub mod systems;

// Re-export main types
pub use aggregate::{
    Agent, AgentMarker, AgentMetadata, AgentStatus, AgentType,
    AuthMethod, AuthenticationComponent, CapabilitiesComponent,
    ConfigurationComponent, PermissionsComponent, ToolAccessComponent,
    ToolDefinition, ToolUsageStats,
};

pub use commands::{
    ActivateAgent, DecommissionAgent, DeployAgent, DisableAgentTools,
    EnableAgentTools, GrantAgentPermissions, RemoveAgentConfiguration,
    RevokeAgentPermissions, SetAgentConfiguration, SetAgentOffline,
    SuspendAgent, ChangeAgentCapabilities,
};

pub use events::{
    AgentActivated, AgentCapabilitiesAdded, AgentCapabilitiesRemoved,
    AgentConfigurationRemoved, AgentConfigurationSet, AgentDecommissioned,
    AgentDeployed, AgentPermissionsGranted, AgentPermissionsRevoked,
    AgentSuspended, AgentToolsDisabled, AgentToolsEnabled, AgentWentOffline,
};

pub use handlers::{AgentCommandHandler, AgentEventHandler};
pub use projections::AgentView;
pub use queries::{AgentQuery, AgentQueryHandler};

// Re-export ECS types
pub use components::*;
pub use systems::*;
