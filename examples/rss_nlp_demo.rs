//! RSS Feed Processing with Real NLP Demo
//! 
//! This example demonstrates how to:
//! - Fetch RSS feeds
//! - Process them with real AI providers for NLP analysis
//! - Publish enriched items to NATS

use alchemist::{
    ai::AiManager,
    config::AlchemistConfig,
    nlp_processor::NlpConfig,
    rss_feed_processor::RssFeedProcessor,
};
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting RSS Feed NLP Processing Demo");

    // Load configuration
    let config = AlchemistConfig::load()?;
    
    // Check if NATS is configured
    let nats_url = config.general.nats_url.as_ref()
        .ok_or_else(|| anyhow::anyhow!("NATS URL not configured"))?;
    
    info!("Connecting to NATS at {}", nats_url);
    
    // Connect to NATS
    let nats_client = async_nats::connect(nats_url).await?;
    info!("Connected to NATS");
    
    // Initialize AI manager
    let ai_manager = AiManager::new(&config).await?;
    
    // Check available AI models
    let models = ai_manager.list_models().await?;
    if models.is_empty() {
        error!("No AI models configured. Please configure at least one AI model.");
        error!("Example: alchemist ai add openai-gpt4 openai");
        return Ok(());
    }
    
    info!("Available AI models:");
    for (name, model) in &models {
        info!("  - {} ({})", name, model.provider);
    }
    
    // Use the first available model or default
    let ai_model = models.first()
        .map(|(name, _)| name.clone())
        .or_else(|| config.general.default_ai_model.clone())
        .unwrap_or_else(|| "gpt-3.5-turbo".to_string());
    
    info!("Using AI model: {}", ai_model);
    
    // Configure NLP processing
    let nlp_config = NlpConfig {
        enabled: true,
        ai_model: ai_model.clone(),
        batch_size: 5,
        timeout_seconds: 30,
        retry_attempts: 2,
    };
    
    // Create RSS feed processor
    let processor = Arc::new(
        RssFeedProcessor::new(nats_client.clone(), ai_manager, nlp_config).await?
    );
    
    info!("RSS Feed Processor initialized");
    info!("Starting feed processing...");
    info!("");
    info!("The processor will:");
    info!("  1. Fetch RSS feeds every minute");
    info!("  2. Process new items with NLP:");
    info!("     - Sentiment analysis");
    info!("     - Entity extraction");
    info!("     - Keyword extraction");
    info!("     - Summarization");
    info!("  3. Publish enriched items to NATS");
    info!("");
    info!("Monitor NATS subjects:");
    info!("  - rss.processed.items");
    info!("  - rss.feeds.*.status");
    info!("  - rss.feeds.*.errors");
    info!("");
    info!("Press Ctrl+C to stop");
    
    // Start processing
    processor.start().await?;
    
    Ok(())
}