//! Domain Layer - Business Logic and Rules

pub mod aggregates;
pub mod commands;
pub mod events;
pub mod services;
pub mod value_objects;

pub mod prelude {
    pub use super::aggregates::*;
    pub use super::commands::*;
    pub use super::events::*;
    pub use super::value_objects::*;
}
