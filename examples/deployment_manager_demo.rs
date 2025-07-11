//! Deployment manager demo - Nix deployment visualization

use alchemist::deployment_ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Alchemist Deployment Manager Demo");
    println!("=====================================");
    println!();
    println!("This demo shows the Nix deployment management interface.");
    println!("Features:");
    println!("  • Deploy from flake URLs");
    println!("  • Environment management (dev/staging/prod)");
    println!("  • Deployment status tracking");
    println!("  • Rollback capability");
    println!("  • Live deployment logs");
    println!("  • Configuration management");
    println!();
    
    deployment_ui::run_deployment_manager().await
}