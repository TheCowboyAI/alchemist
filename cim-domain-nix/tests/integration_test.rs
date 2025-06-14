//! Integration tests for the Nix domain

use cim_domain_nix::{
    aggregate::*,
    commands::{CreateFlake, CreateModule},
    events::*,
    projections::*,
    value_objects::*,
    NixDomainError,
};
use tempfile::TempDir;
use uuid::Uuid;
use std::path::PathBuf;

#[test]
fn test_flake_aggregate_creation() {
    let temp_dir = TempDir::new().unwrap();
    let cmd = CreateFlake {
        path: temp_dir.path().to_path_buf(),
        description: "Test flake".to_string(),
        template: Some("rust".to_string()),
    };

    let result = FlakeAggregate::handle_create_flake(cmd);
    assert!(result.is_ok());

    let (aggregate, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(aggregate.flake.description, "Test flake");
}

#[test]
fn test_flake_projection() {
    let mut projection = NixProjection::default();

    // Create a flake created event
    let event = FlakeCreated {
        flake_id: Uuid::new_v4(),
        path: PathBuf::from("/tmp/test"),
        description: "Test flake".to_string(),
        template: None,
        timestamp: chrono::Utc::now(),
    };

    // Apply the event
    projection.flake_projection.handle_flake_created(&event);

    // Verify the projection was updated
    assert_eq!(projection.flake_projection.flakes.len(), 1);
    assert!(projection.flake_projection.flakes_by_path.contains_key(&PathBuf::from("/tmp/test")));
}

#[test]
fn test_module_aggregate() {
    let module = NixModule {
        id: Uuid::new_v4(),
        name: "test-module".to_string(),
        options: std::collections::HashMap::new(),
        config: serde_json::json!({}),
        imports: vec![],
    };

    let cmd = CreateModule {
        command_id: Uuid::new_v4(),
        name: "test-module".to_string(),
        module: module.clone(),
    };

    let result = ModuleAggregate::handle_create_module(cmd);
    assert!(result.is_ok());

    let (aggregate, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(aggregate.module.name, "test-module");
}

#[test]
fn test_attribute_path() {
    let path = AttributePath::from_str("packages.x86_64-linux.hello");
    assert_eq!(path.segments.len(), 3);
    assert_eq!(path.to_string(), "packages.x86_64-linux.hello");
}

#[test]
fn test_store_path_parsing() {
    let path_str = "/nix/store/abc123-hello-1.0";
    let store_path = StorePath::parse(path_str).unwrap();
    
    assert_eq!(store_path.hash, "abc123");
    assert_eq!(store_path.name, "hello-1.0");
    assert_eq!(store_path.to_string(), path_str);
}

#[test]
fn test_package_build_projection() {
    let mut projection = PackageBuildProjection::default();

    // Create a package built event
    let event = PackageBuilt {
        package_id: Uuid::new_v4(),
        flake_ref: "nixpkgs".to_string(),
        attribute: AttributePath::from_str("hello"),
        output_path: PathBuf::from("/nix/store/abc123-hello"),
        build_time: std::time::Duration::from_secs(10),
        timestamp: chrono::Utc::now(),
    };

    // Apply the event
    projection.handle_package_built(&event);

    // Verify the projection was updated
    assert_eq!(projection.total_builds, 1);
    assert_eq!(projection.successful_builds, 1);
    assert_eq!(projection.builds.len(), 1);
}

#[test]
fn test_configuration_projection() {
    let mut projection = ConfigurationProjection::default();

    // Create a configuration
    let config = NixOSConfiguration {
        id: Uuid::new_v4(),
        name: "test-config".to_string(),
        system: "x86_64-linux".to_string(),
        path: PathBuf::from("/etc/nixos/test-config"),
        hostname: "test-host".to_string(),
        modules: vec![],
        overlays: vec![],
        packages: vec![],
        specialisations: std::collections::HashMap::new(),
    };

    let event = ConfigurationCreated {
        event_id: Uuid::new_v4(),
        occurred_at: chrono::Utc::now(),
        configuration: config,
    };

    // Apply the event
    projection.handle_configuration_created(&event);

    // Verify the projection was updated
    assert_eq!(projection.configurations.len(), 1);
    assert!(projection.configurations.contains_key("test-config"));
} 