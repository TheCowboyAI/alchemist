//! Tests for high-level Nix services

use cim_domain_nix::{
    projections::NixProjection,
    services::{NixServiceFactory, BuildReport},
    value_objects::*,
};
use tempfile::TempDir;

#[tokio::test]
async fn test_development_service_init_project() {
    let projection = NixProjection::default();
    let factory = NixServiceFactory::new(projection);
    let dev_service = factory.development_service();

    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test-project");

    let result = dev_service.init_project(
        project_path.clone(),
        "rust",
        "Test project".to_string(),
    ).await;

    assert!(result.is_ok());
    let flake_id = result.unwrap();
    assert_ne!(flake_id, uuid::Uuid::nil());

    // Check that flake.nix was created
    assert!(project_path.join("flake.nix").exists());
}

#[tokio::test]
async fn test_development_service_add_dependency() {
    let projection = NixProjection::default();
    let factory = NixServiceFactory::new(projection);
    let dev_service = factory.development_service();

    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("test-project");

    // First create a project
    dev_service.init_project(
        project_path.clone(),
        "rust",
        "Test project".to_string(),
    ).await.unwrap();

    // Then add a dependency
    let result = dev_service.add_dependency(
        project_path.clone(),
        "nixpkgs".to_string(),
        "github:NixOS/nixpkgs".to_string(),
    ).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_package_service_search() {
    let projection = NixProjection::default();
    let factory = NixServiceFactory::new(projection);
    let pkg_service = factory.package_service();

    // This test might fail if nix is not available
    let result = pkg_service.search_packages("hello".to_string(), Some(3)).await;

    // We don't assert success here because it depends on system configuration
    if let Ok(packages) = result {
        assert!(packages.len() <= 3);
        for pkg in packages {
            assert!(!pkg.name.is_empty());
        }
    }
}

#[tokio::test]
async fn test_service_factory() {
    let projection = NixProjection::default();
    let factory = NixServiceFactory::new(projection);

    // Test that we can create all services
    let _dev_service = factory.development_service();
    let _config_service = factory.configuration_service();
    let _pkg_service = factory.package_service();

    // Services should be created successfully
    assert!(true);
}

#[test]
fn test_build_report() {
    let report = BuildReport {
        success: true,
        flake_path: PathBuf::from("/tmp/test"),
        outputs: vec!["result".to_string()],
    };

    assert!(report.success);
    assert_eq!(report.flake_path.to_str().unwrap(), "/tmp/test");
    assert_eq!(report.outputs.len(), 1);
}

#[tokio::test]
async fn test_configuration_service_create() {
    let projection = NixProjection::default();
    let factory = NixServiceFactory::new(projection);
    let config_service = factory.configuration_service();

    let result = config_service.create_configuration(
        "test-config".to_string(),
        "x86_64-linux".to_string(),
        vec![],
    ).await;

    assert!(result.is_ok());
    let config_id = result.unwrap();
    assert_ne!(config_id, uuid::Uuid::nil());
}

use std::path::PathBuf; 