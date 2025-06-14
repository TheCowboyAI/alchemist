//! Example of creating a Nix flake using the domain module

use cim_domain_nix::{
    aggregate::FlakeAggregate,
    commands::CreateFlake,
    events::NixDomainEvent,
};
use std::path::PathBuf;
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for our flake
    let temp_dir = TempDir::new()?;
    let flake_path = temp_dir.path().to_path_buf();

    println!("Creating flake in: {}", flake_path.display());

    // Create the command
    let cmd = CreateFlake {
        path: flake_path,
        description: "My awesome Rust project".to_string(),
        template: Some("rust".to_string()),
    };

    // Handle the command
    let (aggregate, events) = FlakeAggregate::handle_create_flake(cmd)?;

    println!("Created flake with ID: {}", aggregate.flake.id);
    println!("Generated {} events", events.len());

    // Print event details
    for event in &events {
        println!("Event: {:?}", event);
        println!("  - Event ID: {}", event.event_id());
        println!("  - Occurred at: {}", event.occurred_at());
    }

    // Keep the directory around so we can inspect it
    let path = temp_dir.into_path();
    println!("\nFlake created at: {}", path.display());
    println!("You can inspect the flake.nix file there.");

    Ok(())
} 