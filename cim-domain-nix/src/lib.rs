//! # CIM Domain Nix
//!
//! This domain module provides functionality for working with the Nix ecosystem,
//! including flakes, modules, overlays, configurations, packages, and applications.
//!
//! ## Architecture
//!
//! The module follows Domain-Driven Design principles with:
//! - Aggregates for managing Nix entities
//! - Commands for operations
//! - Events for state changes
//! - Value objects for domain concepts
//! - Handlers for processing commands
//! - Projections for read models
//! - Queries for data retrieval

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;
pub mod templates;
pub mod services;
// pub mod git_integration;

// Re-export commonly used types
pub use aggregate::*;
pub use commands::*;
pub use events::*;
pub use handlers::*;
pub use projections::*;
// Don't re-export queries to avoid ambiguous types
pub use value_objects::*;
pub use templates::*;
pub use services::*;
// pub use git_integration::*;

use thiserror::Error;

/// Domain-specific errors for Nix operations
#[derive(Error, Debug)]
pub enum NixDomainError {
    #[error("Flake error: {0}")]
    FlakeError(String),

    #[error("Build error: {0}")]
    BuildError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error")]
    IoError(#[from] std::io::Error),

    #[error("Command execution error: {0}")]
    CommandError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Domain error: {0}")]
    DomainError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Package error: {0}")]
    PackageError(String),

    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for Nix domain operations
pub type Result<T> = std::result::Result<T, NixDomainError>; 