//! Unit tests for flake operations

use cim_domain_nix::{
    aggregate::FlakeAggregate,
    commands::{CreateFlake, AddFlakeInput},
    value_objects::FlakeRef,
};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_create_flake() {
    let cmd = CreateFlake {
        path: PathBuf::from("/tmp/test-flake"),
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
fn test_add_flake_input() {
    let temp_dir = TempDir::new().unwrap();
    let flake_path = temp_dir.path().to_path_buf();

    let cmd = CreateFlake {
        path: flake_path.clone(),
        description: "Test flake".to_string(),
        template: None,
    };

    let (mut aggregate, _) = FlakeAggregate::handle_create_flake(cmd).unwrap();

    // Add an input
    let add_cmd = AddFlakeInput {
        path: flake_path,
        name: "nixpkgs".to_string(),
        url: "github:NixOS/nixpkgs".to_string(),
    };

    let result = aggregate.handle_add_flake_input(add_cmd);
    assert!(result.is_ok());

    let events = result.unwrap();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_flake_ref() {
    let flake_ref = FlakeRef::new("github:owner/repo")
        .with_revision("main")
        .with_subflake("subflake");

    assert_eq!(flake_ref.uri, "github:owner/repo");
    assert_eq!(flake_ref.revision, Some("main".to_string()));
    assert_eq!(flake_ref.subflake, Some("subflake".to_string()));
}

#[test]
fn test_duplicate_input_error() {
    let temp_dir = TempDir::new().unwrap();
    let flake_path = temp_dir.path().to_path_buf();

    let cmd = CreateFlake {
        path: flake_path.clone(),
        description: "Test flake".to_string(),
        template: None,
    };

    let (mut aggregate, _) = FlakeAggregate::handle_create_flake(cmd).unwrap();

    // Add an input
    let add_cmd = AddFlakeInput {
        path: flake_path.clone(),
        name: "nixpkgs".to_string(),
        url: "github:NixOS/nixpkgs".to_string(),
    };

    aggregate.handle_add_flake_input(add_cmd.clone()).unwrap();

    // Try to add the same input again
    let result = aggregate.handle_add_flake_input(add_cmd);
    assert!(result.is_err());
} 