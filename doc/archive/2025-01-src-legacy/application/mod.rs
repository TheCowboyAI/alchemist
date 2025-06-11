//! Application Layer - Use Cases and Orchestration
//!
//! This module contains the application services that orchestrate
//! domain operations and handle cross-cutting concerns.

pub mod command_handlers;
pub mod commands;
pub mod projections;
pub mod query_handlers;
pub mod services;

use crate::domain::commands::Command;
use bevy::prelude::*;

/// Event for sending commands through Bevy's event system
#[derive(Event, Debug, Clone)]
pub struct CommandEvent {
    pub command: Command,
}

/// Event for notifying about domain events
#[derive(Event, Debug, Clone)]
pub struct EventNotification {
    pub event: crate::domain::events::DomainEvent,
}

pub use command_handlers::*;
pub use commands::*;
pub use projections::*;
pub use query_handlers::*;
pub use services::*;
