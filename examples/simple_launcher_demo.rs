//! Simple launcher demo that works with the minimal UI components

use alchemist::launcher_simple;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("═══════════════════════════════════════════");
    println!("    🧪 ALCHEMIST - Simple Launcher Demo");
    println!("═══════════════════════════════════════════");
    println!();
    println!("This demo shows the working UI components:");
    println!("  • Dashboard - System status and metrics");
    println!("  • Dialog Window - AI conversation interface");
    println!();
    
    // Initialize settings (optional)
    if let Err(e) = alchemist::settings::initialize_settings().await {
        eprintln!("Warning: Could not initialize settings: {}", e);
    }
    
    launcher_simple::run_simple_launcher().await
}