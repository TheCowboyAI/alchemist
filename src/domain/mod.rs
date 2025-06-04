//! Domain Layer - Business Logic and Rules

pub mod events;
pub mod commands;
pub mod aggregates;
pub mod services;

pub mod prelude {
    pub use super::events::*;
    pub use super::commands::*;
    pub use super::aggregates::*;
}
