//! Dialog window demo - AI conversation interface

use alchemist::dialog_window_minimal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🤖 Alchemist Dialog Window Demo");
    println!("================================");
    println!();
    println!("This demo shows the AI conversation interface.");
    println!("Features:");
    println!("  • Send messages to AI");
    println!("  • Switch between AI models");
    println!("  • Real-time response display");
    println!();
    
    dialog_window_minimal::run_dialog_window("AI Assistant Demo".to_string()).await
}