//! Domain events for the Nix domain
//!
//! This module contains events that represent state changes
//! in the Nix ecosystem.

use crate::value_objects::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;
use std::any::Any;

/// Base trait for all Nix domain events
pub trait NixDomainEvent: Send + Sync + std::fmt::Debug {
    /// Gets the event ID
    fn event_id(&self) -> Uuid;
    
    /// Gets the timestamp when the event occurred
    fn occurred_at(&self) -> DateTime<Utc>;
    
    /// Gets the aggregate ID this event belongs to
    fn aggregate_id(&self) -> Uuid;
    
    /// Get the event as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Event emitted when a flake is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeCreated {
    /// Event ID
    pub flake_id: Uuid,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// The flake that was created
    pub path: PathBuf,
    /// The flake description
    pub description: String,
    /// The flake template
    pub template: Option<String>,
}

impl NixDomainEvent for FlakeCreated {
    fn event_id(&self) -> Uuid {
        self.flake_id
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }
    
    fn aggregate_id(&self) -> Uuid {
        self.flake_id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Event emitted when a flake is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeUpdated {
    /// Event ID
    pub flake_id: Uuid,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// The flake path
    pub path: PathBuf,
}

impl NixDomainEvent for FlakeUpdated {
    fn event_id(&self) -> Uuid {
        self.flake_id
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }
    
    fn aggregate_id(&self) -> Uuid {
        self.flake_id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Event emitted when a flake input is added
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeInputAdded {
    /// Event ID
    pub flake_id: Uuid,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// The flake path
    pub path: PathBuf,
    /// Input name
    pub input_name: String,
    /// Input URL
    pub input_url: String,
}

impl NixDomainEvent for FlakeInputAdded {
    fn event_id(&self) -> Uuid {
        self.flake_id
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }
    
    fn aggregate_id(&self) -> Uuid {
        self.flake_id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Event emitted when a package is built
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageBuilt {
    /// Event ID
    pub package_id: Uuid,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// The flake reference
    pub flake_ref: String,
    /// The attribute path
    pub attribute: AttributePath,
    /// The output path
    pub output_path: PathBuf,
    /// Build duration
    pub build_time: std::time::Duration,
}

impl NixDomainEvent for PackageBuilt {
    fn event_id(&self) -> Uuid {
        self.package_id
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }
    
    fn aggregate_id(&self) -> Uuid {
        self.package_id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Event emitted when a module is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCreated {
    /// Event ID
    pub event_id: Uuid,
    /// When the event occurred
    pub occurred_at: DateTime<Utc>,
    /// The module that was created
    pub module: NixModule,
}

impl NixDomainEvent for ModuleCreated {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn aggregate_id(&self) -> Uuid {
        self.module.id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Event emitted when an overlay is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayCreated {
    /// Event ID
    pub event_id: Uuid,
    /// When the event occurred
    pub occurred_at: DateTime<Utc>,
    /// The overlay that was created
    pub overlay: Overlay,
}

impl NixDomainEvent for OverlayCreated {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn aggregate_id(&self) -> Uuid {
        self.overlay.id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Event emitted when a NixOS configuration is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationCreated {
    /// Event ID
    pub event_id: Uuid,
    /// When the event occurred
    pub occurred_at: DateTime<Utc>,
    /// The configuration that was created
    pub configuration: NixOSConfiguration,
}

impl NixDomainEvent for ConfigurationCreated {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn aggregate_id(&self) -> Uuid {
        self.configuration.id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Event emitted when a configuration is activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationActivated {
    /// Event ID
    pub event_id: Uuid,
    /// When the event occurred
    pub occurred_at: DateTime<Utc>,
    /// The configuration ID
    pub configuration_id: Uuid,
    /// The system generation number
    pub generation: u32,
    /// Whether this was a switch or boot activation
    pub activation_type: ActivationType,
}

/// Type of configuration activation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivationType {
    /// Switch to the new configuration immediately
    Switch,
    /// Set as boot configuration
    Boot,
    /// Test configuration without making permanent
    Test,
}

impl NixDomainEvent for ConfigurationActivated {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn aggregate_id(&self) -> Uuid {
        self.configuration_id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Event emitted when a Nix expression is evaluated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionEvaluated {
    /// Event ID
    pub expression_id: Uuid,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// The expression that was evaluated
    pub expression: String,
    /// The result of the evaluation (as JSON)
    pub result: String,
}

impl NixDomainEvent for ExpressionEvaluated {
    fn event_id(&self) -> Uuid {
        self.expression_id
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }
    
    fn aggregate_id(&self) -> Uuid {
        self.expression_id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Event emitted when garbage collection is performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCollected {
    /// Event ID
    pub collection_id: Uuid,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Bytes freed
    pub freed_bytes: u64,
    /// Paths that were removed
    pub removed_paths: Vec<StorePath>,
}

impl NixDomainEvent for GarbageCollected {
    fn event_id(&self) -> Uuid {
        self.collection_id
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }
    
    fn aggregate_id(&self) -> Uuid {
        self.collection_id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
} 