//! Application Layer - Use Cases and Orchestration

pub mod command_handlers;
pub mod query_handlers;
pub mod projections;

use bevy::prelude::*;
use crate::domain::commands::Command;

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
