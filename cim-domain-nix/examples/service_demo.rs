//! Example demonstrating high-level Nix services

use cim_domain_nix::{
    projections::NixProjection,
    services::{NixServiceFactory, NixDevelopmentService, NixPackageService},
};
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Nix Domain Services Demo ===\n");

    // Create a projection and service factory
    let projection = NixProjection::default();
    let factory = NixServiceFactory::new(projection);

    // Demo 1: Development Service
    demo_development_service(&factory).await?;

    // Demo 2: Package Service
    demo_package_service(&factory).await?;

    println!("\n✅ All demos completed successfully!");
    Ok(())
}

async fn demo_development_service(factory: &NixServiceFactory) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Development Service Demo");
    println!("---------------------------");

    let dev_service = factory.development_service();
    let temp_dir = TempDir::new()?;

    // Initialize a Rust project
    println!("\n1. Initializing Rust project...");
    let project_path = temp_dir.path().join("my-rust-app");
    let flake_id = dev_service.init_project(
        project_path.clone(),
        "rust",
        "My awesome Rust application".to_string(),
    ).await?;

    println!("   ✓ Created project with flake ID: {}", flake_id);

    // Add a dependency
    println!("\n2. Adding dependency...");
    dev_service.add_dependency(
        project_path.clone(),
        "rust-overlay".to_string(),
        "github:oxalica/rust-overlay".to_string(),
    ).await?;

    println!("   ✓ Added rust-overlay dependency");

    // Build and test
    println!("\n3. Building project...");
    match dev_service.build_and_test(project_path.clone()).await {
        Ok(report) => {
            println!("   ✓ Build {}", if report.success { "succeeded" } else { "failed" });
            println!("   ✓ Outputs: {:?}", report.outputs);
        }
        Err(e) => {
            println!("   ⚠ Build failed (expected in demo): {}", e);
        }
    }

    // Keep the directory for inspection
    let path = temp_dir.into_path();
    println!("\n   📁 Project created at: {}", path.display());

    Ok(())
}

async fn demo_package_service(factory: &NixServiceFactory) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📦 Package Service Demo");
    println!("------------------------");

    let pkg_service = factory.package_service();

    // Search for packages
    println!("\n1. Searching for 'hello' packages...");
    match pkg_service.search_packages("hello".to_string(), Some(5)).await {
        Ok(results) => {
            println!("   ✓ Found {} packages:", results.len());
            for (i, pkg) in results.iter().enumerate() {
                println!("     {}. {} - {}", 
                    i + 1, 
                    pkg.name, 
                    pkg.description.as_deref().unwrap_or("No description")
                );
            }
        }
        Err(e) => {
            println!("   ⚠ Search failed (nixpkgs might not be available): {}", e);
        }
    }

    // Build a package
    println!("\n2. Building 'hello' package...");
    let temp_dir = TempDir::new()?;
    let output_path = temp_dir.path().join("hello-result");

    match pkg_service.build_package("hello", Some(output_path.clone())).await {
        Ok(path) => {
            println!("   ✓ Package built at: {}", path.display());
        }
        Err(e) => {
            println!("   ⚠ Build failed (nix might not be available): {}", e);
        }
    }

    // Garbage collection demo
    println!("\n3. Running garbage collection...");
    match pkg_service.garbage_collect(Some(30)).await {
        Ok(freed) => {
            let freed_mb = freed as f64 / (1024.0 * 1024.0);
            println!("   ✓ Freed {:.2} MB", freed_mb);
        }
        Err(e) => {
            println!("   ⚠ Garbage collection failed: {}", e);
        }
    }

    Ok(())
}

// Helper function to demonstrate configuration service (not implemented in demo)
#[allow(dead_code)]
async fn demo_configuration_service(factory: &NixServiceFactory) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🖥️  Configuration Service Demo");
    println!("------------------------------");

    let config_service = factory.configuration_service();

    // Create a configuration
    println!("\n1. Creating NixOS configuration...");
    let config_id = config_service.create_configuration(
        "demo-config".to_string(),
        "x86_64-linux".to_string(),
        vec![],
    ).await?;

    println!("   ✓ Created configuration with ID: {}", config_id);

    // Test configuration
    println!("\n2. Testing configuration...");
    match config_service.test_configuration("demo-config".to_string()).await {
        Ok(_) => println!("   ✓ Configuration test passed"),
        Err(e) => println!("   ⚠ Configuration test failed: {}", e),
    }

    Ok(())
} 