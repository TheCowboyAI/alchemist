//! Minimal main entry point to test UI

mod dashboard_minimal;
mod dashboard;
mod nats_dashboard_connector;
mod nats_client;
mod system_monitor;

use dashboard::DashboardData;
use std::env;

fn main() -> anyhow::Result<()> {
    println!("Alchemist Minimal UI Test");
    println!("=========================");
    
    // Check for NATS URL
    let nats_url = env::var("NATS_URL").ok();
    
    // Run async runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            if let Some(url) = nats_url {
                // Try to connect to NATS
                println!("Attempting to connect to NATS at: {}", url);
                match async_nats::connect(&url).await {
                    Ok(client) => {
                        println!("✅ Connected to NATS!");
                        
                        // Create initial dashboard data
                        let initial_data = DashboardData::example();
                        
                        // Create NATS stream
                        let (rx, _handle) = nats_dashboard_connector::create_nats_dashboard_stream(
                            client,
                            initial_data,
                        ).await;
                        
                        // Run dashboard with NATS
                        dashboard_minimal::run_dashboard_with_nats(rx).await
                    }
                    Err(e) => {
                        println!("⚠️  Could not connect to NATS: {}", e);
                        println!("Running in standalone mode");
                        dashboard_minimal::run_minimal_dashboard().await
                    }
                }
            } else {
                println!("No NATS_URL set, running in standalone mode");
                println!("Set NATS_URL environment variable to enable real-time updates");
                dashboard_minimal::run_minimal_dashboard().await
            }
        })
}