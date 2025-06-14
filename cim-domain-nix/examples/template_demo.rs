//! Example demonstrating flake template usage

use cim_domain_nix::{
    commands::CreateFlake,
    handlers::NixCommandHandler,
};
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for our examples
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();

    println!("Creating flakes in: {}", base_path.display());

    // Create command handler
    let handler = NixCommandHandler::new();

    // Example 1: Create a Rust project
    println!("\n1. Creating Rust project...");
    let rust_path = base_path.join("rust-project");
    let rust_cmd = CreateFlake {
        path: rust_path.clone(),
        description: "A Rust project with Nix".to_string(),
        template: Some("rust".to_string()),
    };

    let events = handler.handle_command(Box::new(rust_cmd)).await?;
    println!("   Created Rust project with {} events", events.len());

    // Example 2: Create a Python project
    println!("\n2. Creating Python project...");
    let python_path = base_path.join("python-project");
    let python_cmd = CreateFlake {
        path: python_path.clone(),
        description: "A Python project with Poetry".to_string(),
        template: Some("python".to_string()),
    };

    let events = handler.handle_command(Box::new(python_cmd)).await?;
    println!("   Created Python project with {} events", events.len());

    // Example 3: Create a NixOS system configuration
    println!("\n3. Creating NixOS system configuration...");
    let nixos_path = base_path.join("nixos-config");
    let nixos_cmd = CreateFlake {
        path: nixos_path.clone(),
        description: "My NixOS system configuration".to_string(),
        template: Some("nixos-system".to_string()),
    };

    let events = handler.handle_command(Box::new(nixos_cmd)).await?;
    println!("   Created NixOS configuration with {} events", events.len());

    // Example 4: Create a development shell
    println!("\n4. Creating development shell...");
    let devshell_path = base_path.join("dev-shell");
    let devshell_cmd = CreateFlake {
        path: devshell_path.clone(),
        description: "Development environment".to_string(),
        template: Some("devshell".to_string()),
    };

    let events = handler.handle_command(Box::new(devshell_cmd)).await?;
    println!("   Created development shell with {} events", events.len());

    // Keep the directory for inspection
    let path = temp_dir.into_path();
    println!("\n✅ All templates created successfully!");
    println!("📁 You can inspect the generated files at: {}", path.display());

    Ok(())
} 