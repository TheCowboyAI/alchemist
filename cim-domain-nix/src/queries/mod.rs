//! Query handlers for Nix domain

use crate::{projections::*, value_objects::*, Result};
use std::path::PathBuf;
use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Query to find a flake by path
#[derive(Debug, Clone)]
pub struct FindFlakeQuery {
    pub path: PathBuf,
}

/// Query to find a package by name
#[derive(Debug, Clone)]
pub struct FindPackageQuery {
    pub name: String,
    pub system: Option<String>,
}

/// Query to find a configuration by name
#[derive(Debug, Clone)]
pub struct FindConfigurationQuery {
    pub name: String,
}

/// Query to search nixpkgs
#[derive(Debug, Clone)]
pub struct SearchNixPackagesQuery {
    pub query: String,
    pub limit: Option<usize>,
}

/// Basic query handler for Nix domain
pub struct NixQueryHandler {
    projection: NixProjection,
}

impl NixQueryHandler {
    pub fn new(projection: NixProjection) -> Self {
        Self { projection }
    }

    pub fn find_flake(&self, query: FindFlakeQuery) -> Result<Option<FlakeView>> {
        if let Some(flake_id) = self.projection.flake_projection.flakes_by_path.get(&query.path) {
            Ok(self.projection.flake_projection.flakes.get(flake_id).map(|info| FlakeView {
                id: info.id,
                path: info.path.clone(),
                description: info.description.clone(),
                inputs: info.inputs.clone(),
                outputs: info.outputs.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn find_package(&self, query: FindPackageQuery) -> Result<Option<PackageView>> {
        let key = format!("{}#{}", 
            query.system.as_deref().unwrap_or("x86_64-linux"), 
            query.name
        );
        
        Ok(self.projection.package_projection.packages.get(&key).map(|info| PackageView {
            name: info.name.clone(),
            system: info.system.clone(),
            version: info.version.clone(),
            description: info.description.clone(),
        }))
    }

    pub fn find_configuration(&self, query: FindConfigurationQuery) -> Result<Option<ConfigurationView>> {
        Ok(self.projection.configuration_projection.configurations.get(&query.name).map(|info| ConfigurationView {
            id: info.id,
            name: info.name.clone(),
            system: info.system.clone(),
            current_generation: info.current_generation,
            last_activated: info.last_activated,
        }))
    }
}

/// Advanced query handler with external data sources
pub struct AdvancedNixQueryHandler {
    projection: NixProjection,
}

impl AdvancedNixQueryHandler {
    pub fn new(projection: NixProjection) -> Self {
        Self { projection }
    }

    /// Search nixpkgs for packages
    pub async fn search_nixpkgs(&self, query: SearchNixPackagesQuery) -> Result<Vec<PackageSearchResult>> {
        // Search in our projection first
        let mut results = vec![];
        
        for (key, package) in &self.projection.package_projection.packages {
            if package.name.contains(&query.query) || 
               package.description.as_ref().map(|d| d.contains(&query.query)).unwrap_or(false) {
                results.push(PackageSearchResult {
                    name: package.name.clone(),
                    version: package.version.clone(),
                    description: package.description.clone(),
                    attribute_path: key.clone(),
                });
                
                if let Some(limit) = query.limit {
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }
        
        Ok(results)
    }
}

/// View model for flake information
#[derive(Debug, Clone)]
pub struct FlakeView {
    pub id: Uuid,
    pub path: PathBuf,
    pub description: String,
    pub inputs: HashMap<String, FlakeRef>,
    pub outputs: FlakeOutputs,
}

/// View model for package information
#[derive(Debug, Clone)]
pub struct PackageView {
    pub name: String,
    pub system: String,
    pub version: Option<String>,
    pub description: Option<String>,
}

/// View model for configuration information
#[derive(Debug, Clone)]
pub struct ConfigurationView {
    pub id: Uuid,
    pub name: String,
    pub system: String,
    pub current_generation: u32,
    pub last_activated: Option<DateTime<Utc>>,
}

/// Search result for packages
#[derive(Debug, Clone)]
pub struct PackageSearchResult {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub attribute_path: String,
} 