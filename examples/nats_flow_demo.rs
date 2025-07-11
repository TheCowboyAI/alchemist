//! NATS flow visualizer demo - Message flow visualization

use alchemist::nats_flow_visualizer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("📡 Alchemist NATS Flow Visualizer Demo");
    println!("=======================================");
    println!();
    println!("This demo visualizes NATS message flow in real-time.");
    println!("Features:");
    println!("  • Real-time message animations");
    println!("  • Subject node visualization");
    println!("  • Message count tracking");
    println!("  • Activity-based highlighting");
    println!("  • Message filtering by subject");
    println!();
    println!("If NATS is running, it will show real messages.");
    println!("Otherwise, it will generate demo messages.");
    println!();
    
    nats_flow_visualizer::run_nats_flow_visualizer().await
}