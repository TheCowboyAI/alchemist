//! Full system demonstration
//! 
//! This example shows all the Alchemist UI components working together
//! with NATS event-based communication.

use alchemist::launcher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("════════════════════════════════════════════════════");
    println!("     🧪 ALCHEMIST - Full System Demonstration");
    println!("════════════════════════════════════════════════════");
    println!();
    println!("This demo showcases the complete Alchemist system:");
    println!();
    println!("✅ Working UI Components:");
    println!("   • Unified Launcher - Central control panel");
    println!("   • Dashboard - System status and metrics");
    println!("   • Dialog Windows - AI conversation interface");
    println!("   • Event Visualizer - Real-time domain events");
    println!("   • NATS Monitor - Event streaming status");
    println!();
    println!("🔌 Communication:");
    println!("   • NATS-based event streaming (if available)");
    println!("   • Renderer API for UI component communication");
    println!("   • Event-driven architecture following CIM principles");
    println!();
    println!("📚 Domain Integration:");
    println!("   • Workflow management");
    println!("   • Document handling");
    println!("   • Location services");
    println!("   • Nix deployment");
    println!();
    println!("🚀 Starting launcher...");
    println!();
    
    // Set up environment
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "alchemist=info");
    }
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    // Run the launcher
    launcher::run_launcher().await
}