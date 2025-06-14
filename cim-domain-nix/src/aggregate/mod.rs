//! Aggregates for the Nix domain
//!
//! This module contains aggregate roots that maintain consistency
//! for Nix domain entities.

use crate::{commands::*, events::*, value_objects::*, Result, NixDomainError};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

/// Flake aggregate
#[derive(Debug, Clone)]
pub struct FlakeAggregate {
    /// The flake being managed
    pub flake: Flake,
}

impl FlakeAggregate {
    /// Create a new flake aggregate
    pub fn new(flake: Flake) -> Self {
        Self { flake }
    }

    /// Handle create flake command
    pub fn handle_create_flake(cmd: CreateFlake) -> Result<(Self, Vec<FlakeCreated>)> {
        let flake = Flake {
            id: Uuid::new_v4(),
            path: cmd.path.clone(),
            description: cmd.description.clone(),
            inputs: FlakeInputs {
                inputs: HashMap::new(),
            },
            outputs: FlakeOutputs {
                packages: HashMap::new(),
                dev_shells: HashMap::new(),
                nixos_modules: HashMap::new(),
                overlays: HashMap::new(),
                apps: HashMap::new(),
            },
        };

        let event = FlakeCreated {
            flake_id: flake.id,
            path: cmd.path,
            description: cmd.description,
            template: cmd.template,
            timestamp: Utc::now(),
        };

        Ok((Self::new(flake), vec![event]))
    }

    /// Handle add flake input command
    pub fn handle_add_flake_input(&mut self, cmd: AddFlakeInput) -> Result<Vec<FlakeInputAdded>> {
        // Check if input already exists
        if self.flake.inputs.inputs.contains_key(&cmd.name) {
            return Err(NixDomainError::DomainError(
                format!("Input '{}' already exists", cmd.name)
            ));
        }

        // Add the input
        self.flake.inputs.inputs.insert(
            cmd.name.clone(),
            FlakeRef::new(cmd.url.clone()),
        );

        let event = FlakeInputAdded {
            flake_id: self.flake.id,
            path: cmd.path,
            input_name: cmd.name,
            input_url: cmd.url,
            timestamp: Utc::now(),
        };

        Ok(vec![event])
    }

    /// Apply event to update state
    pub fn apply_event(&mut self, event: &FlakeInputAdded) {
        self.flake.inputs.inputs.insert(
            event.input_name.clone(),
            FlakeRef::new(event.input_url.clone()),
        );
    }
}

/// Module aggregate
#[derive(Debug, Clone)]
pub struct ModuleAggregate {
    /// The module being managed
    pub module: NixModule,
}

impl ModuleAggregate {
    /// Create a new module aggregate
    pub fn new(module: NixModule) -> Self {
        Self { module }
    }

    /// Handle create module command
    pub fn handle_create_module(cmd: CreateModule) -> Result<(Self, Vec<ModuleCreated>)> {
        let module = cmd.module.clone();

        let event = ModuleCreated {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            module: cmd.module,
        };

        Ok((Self::new(module), vec![event]))
    }
}

/// Overlay aggregate
#[derive(Debug, Clone)]
pub struct OverlayAggregate {
    /// The overlay being managed
    pub overlay: Overlay,
}

impl OverlayAggregate {
    /// Create a new overlay aggregate
    pub fn new(overlay: Overlay) -> Self {
        Self { overlay }
    }

    /// Handle create overlay command
    pub fn handle_create_overlay(cmd: CreateOverlay) -> Result<(Self, Vec<OverlayCreated>)> {
        let overlay = cmd.overlay.clone();

        let event = OverlayCreated {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            overlay: cmd.overlay,
        };

        Ok((Self::new(overlay), vec![event]))
    }
}

/// Configuration aggregate
#[derive(Debug, Clone)]
pub struct ConfigurationAggregate {
    /// The configuration being managed
    pub configuration: NixOSConfiguration,
    /// Current system generation
    pub current_generation: u32,
}

impl ConfigurationAggregate {
    /// Create a new configuration aggregate
    pub fn new(configuration: NixOSConfiguration) -> Self {
        Self {
            configuration,
            current_generation: 0,
        }
    }

    /// Handle create configuration command
    pub fn handle_create_configuration(cmd: CreateConfiguration) -> Result<(Self, Vec<ConfigurationCreated>)> {
        let configuration = cmd.configuration.clone();

        let event = ConfigurationCreated {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            configuration: cmd.configuration,
        };

        Ok((Self::new(configuration), vec![event]))
    }

    /// Handle activate configuration command
    pub fn handle_activate_configuration(&mut self, cmd: ActivateConfiguration) -> Result<Vec<ConfigurationActivated>> {
        self.current_generation += 1;

        let event = ConfigurationActivated {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            configuration_id: self.configuration.id,
            generation: self.current_generation,
            activation_type: cmd.activation_type,
        };

        Ok(vec![event])
    }
} 