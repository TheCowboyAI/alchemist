//! Main Alchemist launcher executable

use alchemist::launcher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("═══════════════════════════════════════════");
    println!("    🧪 ALCHEMIST - CIM Control System");
    println!("═══════════════════════════════════════════");
    println!();
    println!("Starting unified launcher interface...");
    println!();
    
    // Set up environment
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "alchemist=info");
    }
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    // Initialize settings
    if let Err(e) = alchemist::settings::initialize_settings().await {
        eprintln!("Warning: Could not initialize settings: {}", e);
    }
    
    // Run the simple launcher (works with current iced version)
    alchemist::launcher_simple::run_simple_launcher().await
}