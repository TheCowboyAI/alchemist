//! Performance monitor demo - System resource visualization

use alchemist::performance_monitor_ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("📈 Alchemist Performance Monitor Demo");
    println!("======================================");
    println!();
    println!("This demo shows real-time system performance monitoring.");
    println!("Features:");
    println!("  • CPU usage tracking");
    println!("  • Memory consumption visualization");
    println!("  • Network activity monitoring");
    println!("  • Process list with sorting");
    println!("  • Historical graphs");
    println!("  • Metrics export functionality");
    println!();
    
    performance_monitor_ui::run_performance_monitor().await
}