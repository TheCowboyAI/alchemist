//! High-level services for Nix domain operations

use crate::{
    commands::*,
    events::*,
    handlers::*,
    projections::NixProjection,
    queries::{
        NixQueryHandler, AdvancedNixQueryHandler,
        FindFlakeQuery, FindPackageQuery, FindConfigurationQuery, SearchNixPackagesQuery,
        FlakeView, PackageView, ConfigurationView, PackageSearchResult,
    },
    value_objects::*,
    Result, NixDomainError,
};
use std::path::PathBuf;
use uuid::Uuid;

/// Service for managing Nix development environments
pub struct NixDevelopmentService {
    command_handler: NixCommandHandler,
    query_handler: NixQueryHandler,
}

impl NixDevelopmentService {
    /// Create a new development service
    pub fn new(projection: NixProjection) -> Self {
        Self {
            command_handler: NixCommandHandler::new(),
            query_handler: NixQueryHandler::new(projection),
        }
    }

    /// Initialize a new project with a flake
    pub async fn init_project(
        &self,
        path: PathBuf,
        project_type: &str,
        description: String,
    ) -> Result<Uuid> {
        // Create flake with template
        let create_cmd = CreateFlake {
            path: path.clone(),
            description,
            template: Some(project_type.to_string()),
        };

        let events = self.command_handler
            .handle_command(Box::new(create_cmd))
            .await?;

        // Extract flake ID from events
        let flake_id = events.iter()
            .find_map(|e| {
                if let Some(created) = e.as_any().downcast_ref::<FlakeCreated>() {
                    Some(created.flake_id)
                } else {
                    None
                }
            })
            .ok_or_else(|| NixDomainError::Other("No FlakeCreated event found".to_string()))?;

        // Initialize git if needed
        self.init_git_repo(&path).await?;

        Ok(flake_id)
    }

    /// Add a dependency to a flake
    pub async fn add_dependency(
        &self,
        flake_path: PathBuf,
        dep_name: String,
        dep_url: String,
    ) -> Result<()> {
        let cmd = AddFlakeInput {
            path: flake_path,
            name: dep_name,
            url: dep_url,
        };

        self.command_handler
            .handle_command(Box::new(cmd))
            .await?;

        Ok(())
    }

    /// Build and test a project
    pub async fn build_and_test(&self, flake_path: PathBuf) -> Result<BuildReport> {
        // Check flake first
        let check_cmd = CheckFlake {
            path: flake_path.clone(),
        };

        self.command_handler
            .handle_command(Box::new(check_cmd))
            .await?;

        // Build default package
        let build_cmd = BuildPackage {
            flake_ref: format!("path:{}", flake_path.display()),
            attribute: AttributePath::from_str("defaultPackage"),
            output_path: Some(flake_path.join("result")),
        };

        let build_events = self.command_handler
            .handle_command(Box::new(build_cmd))
            .await?;

        // Extract build results
        let build_success = build_events.iter()
            .any(|e| e.as_any().downcast_ref::<PackageBuilt>().is_some());

        Ok(BuildReport {
            success: build_success,
            flake_path,
            outputs: vec!["result".to_string()],
        })
    }

    /// Enter development shell
    pub async fn enter_dev_shell(
        &self,
        flake_path: PathBuf,
        command: Option<String>,
    ) -> Result<()> {
        let cmd = DevelopFlake {
            path: flake_path,
            command,
        };

        self.command_handler
            .handle_command(Box::new(cmd))
            .await?;

        Ok(())
    }

    /// Find a flake by path
    pub fn find_flake(&self, path: PathBuf) -> Result<Option<FlakeView>> {
        self.query_handler.find_flake(FindFlakeQuery { path })
    }

    /// Find a package by name
    pub fn find_package(&self, name: String, system: Option<String>) -> Result<Option<PackageView>> {
        self.query_handler.find_package(FindPackageQuery { name, system })
    }

    async fn init_git_repo(&self, path: &PathBuf) -> Result<()> {
        use tokio::process::Command;

        if !path.join(".git").exists() {
            Command::new("git")
                .arg("init")
                .current_dir(path)
                .output()
                .await
                .map_err(|e| NixDomainError::CommandError(format!("Failed to init git: {}", e)))?;

            // Add flake.nix to git
            Command::new("git")
                .args(&["add", "flake.nix", "flake.lock"])
                .current_dir(path)
                .output()
                .await
                .map_err(|e| NixDomainError::CommandError(format!("Failed to add files: {}", e)))?;
        }

        Ok(())
    }
}

/// Service for managing NixOS configurations
pub struct NixOSConfigurationService {
    command_handler: NixCommandHandler,
    query_handler: NixQueryHandler,
}

impl NixOSConfigurationService {
    /// Create a new configuration service
    pub fn new(projection: NixProjection) -> Self {
        Self {
            command_handler: NixCommandHandler::new(),
            query_handler: NixQueryHandler::new(projection),
        }
    }

    /// Create a new NixOS configuration
    pub async fn create_configuration(
        &self,
        name: String,
        system: String,
        modules: Vec<PathBuf>,
    ) -> Result<Uuid> {
        let config = NixOSConfiguration {
            id: Uuid::new_v4(),
            name: name.clone(),
            system,
            path: PathBuf::from(format!("/etc/nixos/{}", name)),
            hostname: name.clone(), // Use name as hostname by default
            modules,
            overlays: vec![],
            packages: vec![],
            specialisations: std::collections::HashMap::new(),
        };

        let cmd = CreateConfiguration {
            name,
            configuration: config.clone(),
        };

        self.command_handler
            .handle_command(Box::new(cmd))
            .await?;

        Ok(config.id)
    }

    /// Switch to a configuration
    pub async fn switch_configuration(&self, name: String) -> Result<u32> {
        let cmd = ActivateConfiguration {
            name,
            activation_type: ActivationType::Switch,
        };

        let events = self.command_handler
            .handle_command(Box::new(cmd))
            .await?;

        // Extract generation number
        let generation = events.iter()
            .find_map(|e| {
                if let Some(activated) = e.as_any().downcast_ref::<ConfigurationActivated>() {
                    Some(activated.generation)
                } else {
                    None
                }
            })
            .unwrap_or(0);

        Ok(generation)
    }

    /// Find a configuration by name
    pub fn find_configuration(&self, name: String) -> Result<Option<ConfigurationView>> {
        self.query_handler.find_configuration(FindConfigurationQuery { name })
    }

    /// Test a configuration without switching
    pub async fn test_configuration(&self, name: String) -> Result<()> {
        let cmd = ActivateConfiguration {
            name,
            activation_type: ActivationType::Test,
        };

        self.command_handler
            .handle_command(Box::new(cmd))
            .await?;

        Ok(())
    }
}

/// Service for Nix package management
pub struct NixPackageService {
    command_handler: NixCommandHandler,
    query_handler: AdvancedNixQueryHandler,
}

impl NixPackageService {
    /// Create a new package service
    pub fn new(projection: NixProjection) -> Self {
        Self {
            command_handler: NixCommandHandler::new(),
            query_handler: AdvancedNixQueryHandler::new(projection),
        }
    }

    /// Search for packages in nixpkgs
    pub async fn search_packages(&self, query: String, limit: Option<usize>) -> Result<Vec<PackageSearchResult>> {
        let search_query = SearchNixPackagesQuery {
            query,
            limit,
        };

        self.query_handler.search_nixpkgs(search_query).await
    }

    /// Build a specific package
    pub async fn build_package(
        &self,
        package_name: &str,
        output_path: Option<PathBuf>,
    ) -> Result<PathBuf> {
        let cmd = BuildPackage {
            flake_ref: "nixpkgs".to_string(),
            attribute: AttributePath::from_str(package_name),
            output_path: output_path.clone(),
        };

        let events = self.command_handler
            .handle_command(Box::new(cmd))
            .await?;

        // Extract output path
        let output = events.iter()
            .find_map(|e| {
                if let Some(built) = e.as_any().downcast_ref::<PackageBuilt>() {
                    Some(built.output_path.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| output_path.unwrap_or_else(|| PathBuf::from("./result")));

        Ok(output)
    }

    /// Clean up old packages
    pub async fn garbage_collect(&self, older_than_days: Option<u32>) -> Result<u64> {
        let cmd = RunGarbageCollection {
            older_than_days,
        };

        let events = self.command_handler
            .handle_command(Box::new(cmd))
            .await?;

        // Extract freed bytes
        let freed = events.iter()
            .find_map(|e| {
                if let Some(gc) = e.as_any().downcast_ref::<GarbageCollected>() {
                    Some(gc.freed_bytes)
                } else {
                    None
                }
            })
            .unwrap_or(0);

        Ok(freed)
    }
}

/// Report from build operations
#[derive(Debug, Clone)]
pub struct BuildReport {
    /// Whether the build succeeded
    pub success: bool,
    /// Path to the flake
    pub flake_path: PathBuf,
    /// Output paths created
    pub outputs: Vec<String>,
}

/// Service factory for creating domain services
pub struct NixServiceFactory {
    projection: NixProjection,
}

impl NixServiceFactory {
    /// Create a new service factory
    pub fn new(projection: NixProjection) -> Self {
        Self { projection }
    }

    /// Create a development service
    pub fn development_service(&self) -> NixDevelopmentService {
        NixDevelopmentService::new(self.projection.clone())
    }

    /// Create a configuration service
    pub fn configuration_service(&self) -> NixOSConfigurationService {
        NixOSConfigurationService::new(self.projection.clone())
    }

    /// Create a package service
    pub fn package_service(&self) -> NixPackageService {
        NixPackageService::new(self.projection.clone())
    }
} 