//! Commands for Nix domain operations

use crate::value_objects::*;
use crate::events::ActivationType;
use std::path::PathBuf;
use uuid::Uuid;
use std::any::Any;

/// Base trait for all Nix commands
pub trait NixCommand: Send + Sync {
    /// Get the command ID
    fn command_id(&self) -> Uuid;
    
    /// Get the command as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Command to create a new flake
#[derive(Debug, Clone)]
pub struct CreateFlake {
    pub path: PathBuf,
    pub description: String,
    pub template: Option<String>,
}

impl NixCommand for CreateFlake {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Command to update a flake
#[derive(Debug, Clone)]
pub struct UpdateFlake {
    pub path: PathBuf,
}

impl NixCommand for UpdateFlake {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Command to add an input to a flake
#[derive(Debug, Clone)]
pub struct AddFlakeInput {
    pub path: PathBuf,
    pub name: String,
    pub url: String,
}

impl NixCommand for AddFlakeInput {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Command to build a package
#[derive(Debug, Clone)]
pub struct BuildPackage {
    pub flake_ref: String,
    pub attribute: AttributePath,
    pub output_path: Option<PathBuf>,
}

impl NixCommand for BuildPackage {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Command to create a module
#[derive(Debug, Clone)]
pub struct CreateModule {
    pub command_id: Uuid,
    pub name: String,
    pub module: NixModule,
}

impl NixCommand for CreateModule {
    fn command_id(&self) -> Uuid {
        self.command_id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Command to create an overlay
#[derive(Debug, Clone)]
pub struct CreateOverlay {
    pub command_id: Uuid,
    pub name: String,
    pub overlay: Overlay,
}

impl NixCommand for CreateOverlay {
    fn command_id(&self) -> Uuid {
        self.command_id
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Command to create a configuration
#[derive(Debug, Clone)]
pub struct CreateConfiguration {
    pub name: String,
    pub configuration: NixOSConfiguration,
}

impl NixCommand for CreateConfiguration {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Command to activate a configuration
#[derive(Debug, Clone)]
pub struct ActivateConfiguration {
    pub name: String,
    pub activation_type: ActivationType,
}

impl NixCommand for ActivateConfiguration {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Command to evaluate a Nix expression
#[derive(Debug, Clone)]
pub struct EvaluateExpression {
    pub expression: String,
}

impl NixCommand for EvaluateExpression {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Command to run garbage collection
#[derive(Debug, Clone)]
pub struct RunGarbageCollection {
    pub older_than_days: Option<u32>,
}

impl NixCommand for RunGarbageCollection {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Command to check a flake
#[derive(Debug, Clone)]
pub struct CheckFlake {
    pub path: PathBuf,
}

impl NixCommand for CheckFlake {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Command to enter a development shell
#[derive(Debug, Clone)]
pub struct DevelopFlake {
    pub path: PathBuf,
    pub command: Option<String>,
}

impl NixCommand for DevelopFlake {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
} 