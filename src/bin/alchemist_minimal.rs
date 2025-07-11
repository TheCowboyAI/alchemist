//! Minimal Alchemist launcher that compiles and runs
//! 
//! This version includes only the components that work with the current dependencies.

use alchemist::{dashboard_minimal, dialog_window_minimal};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("═══════════════════════════════════════════");
    println!("    🧪 ALCHEMIST - Minimal UI Demo");
    println!("═══════════════════════════════════════════");
    println!();
    println!("Choose a component to launch:");
    println!("1. Dashboard - System monitoring");
    println!("2. Dialog - AI conversation interface");
    println!("3. Exit");
    println!();
    print!("Enter choice (1-3): ");
    
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    match input.trim() {
        "1" => {
            println!("Launching dashboard...");
            dashboard_minimal::run_minimal_dashboard().await
        }
        "2" => {
            println!("Launching dialog window...");
            dialog_window_minimal::run_dialog_window("AI Assistant".to_string()).await
        }
        _ => {
            println!("Exiting...");
            Ok(())
        }
    }
}