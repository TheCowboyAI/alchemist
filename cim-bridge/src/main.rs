//! CIM Bridge Service - Main entry point

use clap::Parser;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use cim_bridge::BridgeService;

#[derive(Parser)]
#[command(name = "cim-bridge")]
#[command(about = "CIM Bridge - AI Provider Bridge Service")]
struct Args {
    /// NATS server URL
    #[arg(short, long, default_value = "nats://localhost:4222")]
    nats_url: String,
    
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&args.log_level))
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("Starting CIM Bridge Service");
    info!("NATS URL: {}", args.nats_url);
    info!("Log level: {}", args.log_level);
    
    // Create and start bridge service
    match BridgeService::new(&args.nats_url).await {
        Ok(bridge) => {
            info!("Bridge service created successfully");
            
            if let Err(e) = bridge.start().await {
                error!("Bridge service error: {}", e);
                return Err(e.into());
            }
        }
        Err(e) => {
            error!("Failed to create bridge service: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
} 