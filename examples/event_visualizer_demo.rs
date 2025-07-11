//! Event visualizer demo - Real-time CIM event monitoring

use alchemist::event_visualizer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("📊 Alchemist Event Visualizer Demo");
    println!("===================================");
    println!();
    println!("This demo shows real-time visualization of CIM domain events.");
    println!("Features:");
    println!("  • Live event stream from NATS or demo mode");
    println!("  • Domain filtering");
    println!("  • Event statistics");
    println!("  • Pause/resume functionality");
    println!();
    println!("If NATS is running, it will show real events.");
    println!("Otherwise, it will generate demo events.");
    println!();
    
    event_visualizer::run_event_visualizer().await
}