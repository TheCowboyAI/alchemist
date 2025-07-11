//! Test example to verify dashboard UI window works

use alchemist::dashboard::{DashboardData, launch_dashboard_inprocess};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting dashboard UI test...");
    
    // Create a channel for updates
    let (tx, rx) = mpsc::channel(100);
    
    // Get example dashboard data
    let initial_data = DashboardData::example();
    
    // Send periodic updates
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut counter = 0;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            let mut data = DashboardData::example();
            // Update some values to show it's working
            data.system_status.total_events += counter;
            counter += 10;
            
            if tx_clone.send(data).await.is_err() {
                break;
            }
        }
    });
    
    println!("Launching dashboard window...");
    println!("The window should appear with:");
    println!("- System status");
    println!("- Domain health information");
    println!("- Active dialogs");
    println!("- Recent events");
    
    // Launch the window (this blocks until closed)
    launch_dashboard_inprocess(initial_data, rx).await?;
    
    println!("Dashboard window closed.");
    Ok(())
}