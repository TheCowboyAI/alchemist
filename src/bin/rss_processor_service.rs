//! RSS Feed Processor Service
//! 
//! A standalone service that fetches RSS feeds, processes them with NLP,
//! and publishes to NATS for consumption by the dashboard and other services.

use alchemist::{
    ai::AiManager,
    config::AlchemistConfig,
    nlp_processor::NlpConfig,
    rss_feed_processor::RssFeedProcessor,
};
use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn, error};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(name = "rss-processor")]
#[command(about = "RSS Feed Processor Service with NLP enrichment")]
struct Args {
    /// NATS URL to connect to
    #[arg(long, env = "NATS_URL")]
    nats_url: Option<String>,
    
    /// AI model to use for NLP processing
    #[arg(long, default_value = "gpt-3.5-turbo")]
    ai_model: String,
    
    /// Enable debug logging
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    info!("Starting RSS Feed Processor Service");

    // Load configuration
    let mut config = AlchemistConfig::load()?;
    
    // Override NATS URL if provided
    if let Some(nats_url) = args.nats_url {
        config.general.nats_url = Some(nats_url);
    }
    
    // Check if NATS is configured
    let nats_url = config.general.nats_url.as_ref()
        .ok_or_else(|| anyhow::anyhow!("NATS URL not configured"))?;
    
    info!("Connecting to NATS at {}", nats_url);
    
    // Connect to NATS with retry
    let mut retry_count = 0;
    let nats_client = loop {
        match async_nats::connect(nats_url).await {
            Ok(client) => break client,
            Err(e) => {
                retry_count += 1;
                if retry_count > 5 {
                    error!("Failed to connect to NATS after 5 attempts: {}", e);
                    return Err(e.into());
                }
                warn!("Failed to connect to NATS (attempt {}): {}", retry_count, e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    };
    
    info!("Connected to NATS");
    
    // Initialize AI manager
    let ai_manager = AiManager::new(&config).await?;
    
    // Test AI model
    info!("Testing AI model: {}", args.ai_model);
    match ai_manager.test_connection(&args.ai_model).await {
        Ok(result) => {
            if result.success {
                info!("AI model {} is available (latency: {}ms)", 
                    args.ai_model, result.latency_ms);
            } else {
                error!("AI model {} test failed: {:?}", 
                    args.ai_model, result.error);
                return Err(anyhow::anyhow!("AI model not available"));
            }
        }
        Err(e) => {
            error!("Failed to test AI model: {}", e);
            warn!("Continuing anyway - NLP features may not work");
        }
    }
    
    // Configure NLP processing
    let nlp_config = NlpConfig {
        enabled: true,
        ai_model: args.ai_model.clone(),
        batch_size: 5,
        timeout_seconds: 30,
        retry_attempts: 2,
    };
    
    // Create RSS feed processor
    let processor = Arc::new(
        RssFeedProcessor::new(nats_client.clone(), ai_manager, nlp_config).await?
    );
    
    info!("RSS Feed Processor initialized");
    info!("");
    info!("Processing feeds with NLP enrichment:");
    info!("  - Sentiment analysis");
    info!("  - Entity extraction (PERSON, ORG, LOCATION, etc.)");
    info!("  - Keyword extraction");
    info!("  - AI-powered summarization");
    info!("");
    info!("Publishing to NATS subjects:");
    info!("  - rss.processed.items");
    info!("  - rss.feeds.<feed-id>.status");
    info!("  - rss.feeds.<feed-id>.errors");
    info!("  - rss.feeds.<feed-id>.filtered");
    info!("");
    
    // Start processing in background
    let processor_handle = tokio::spawn(async move {
        if let Err(e) = processor.start().await {
            error!("RSS processor error: {}", e);
        }
    });
    
    info!("Service is running. Press Ctrl+C to stop.");
    
    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received");
        }
        Err(e) => {
            error!("Unable to listen for shutdown signal: {}", e);
        }
    }
    
    // Cleanup
    processor_handle.abort();
    info!("RSS Feed Processor Service stopped");
    
    Ok(())
}