//! RSS Feed Processor Service - Fetches RSS feeds and processes them with NLP

use anyhow::Result;
use async_nats::jetstream;
use chrono::{DateTime, Utc};
use reqwest::Client;
use rss::{Channel, Item};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::{
    nlp_processor::{NlpProcessor, NlpConfig},
    rss_feed_manager::{RssFeedConfig, ProcessedRssItem, FeedFilter, FilterType},
    ai::AiManager,
};

/// RSS Feed Processor that fetches feeds and publishes to NATS
pub struct RssFeedProcessor {
    http_client: Client,
    nlp_processor: Arc<NlpProcessor>,
    jetstream: jetstream::Context,
    feed_configs: Arc<RwLock<HashMap<String, RssFeedConfig>>>,
    processed_guids: Arc<RwLock<HashMap<String, Vec<String>>>>, // feed_id -> processed GUIDs
}

impl RssFeedProcessor {
    pub async fn new(
        nats_client: async_nats::Client,
        ai_manager: AiManager,
        nlp_config: NlpConfig,
    ) -> Result<Self> {
        let jetstream = jetstream::new(nats_client);
        let nlp_processor = Arc::new(NlpProcessor::new(ai_manager, nlp_config.ai_model));
        
        Ok(Self {
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("Alchemist RSS Processor/1.0")
                .build()?,
            nlp_processor,
            jetstream,
            feed_configs: Arc::new(RwLock::new(HashMap::new())),
            processed_guids: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start processing RSS feeds
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("Starting RSS feed processor");
        
        // Load initial feed configurations
        self.load_feed_configs().await?;
        
        // Subscribe to feed configuration updates
        let config_self = self.clone();
        tokio::spawn(async move {
            if let Err(e) = config_self.subscribe_to_config_updates().await {
                error!("Failed to subscribe to config updates: {}", e);
            }
        });
        
        // Start feed processing loop
        let mut interval = interval(Duration::from_secs(60)); // Check every minute
        
        loop {
            interval.tick().await;
            
            let configs = self.feed_configs.read().await;
            for (feed_id, config) in configs.iter() {
                if !config.enabled {
                    continue;
                }
                
                let processor = self.clone();
                let feed_id = feed_id.clone();
                let config = config.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = processor.process_feed(&feed_id, &config).await {
                        error!("Failed to process feed {}: {}", feed_id, e);
                        processor.publish_feed_error(&feed_id, &e.to_string()).await.ok();
                    }
                });
            }
        }
    }

    async fn process_feed(&self, feed_id: &str, config: &RssFeedConfig) -> Result<()> {
        debug!("Processing feed: {}", feed_id);
        
        // Fetch RSS feed
        let response = self.http_client
            .get(&config.url)
            .send()
            .await?
            .bytes()
            .await?;
        
        // Parse RSS
        let channel = Channel::read_from(&response[..])?;
        
        // Get processed GUIDs for this feed
        let mut processed_guids = self.processed_guids.write().await;
        let feed_guids = processed_guids.entry(feed_id.to_string()).or_default();
        
        let mut new_items = Vec::new();
        let mut filtered_count = 0;
        
        for item in channel.items() {
            let guid = item.guid()
                .map(|g| g.value().to_string())
                .unwrap_or_else(|| item.link().unwrap_or("").to_string());
            
            // Skip if already processed
            if feed_guids.contains(&guid) {
                continue;
            }
            
            // Apply filters
            if let Some(reason) = self.should_filter_item(&item, &config.filters) {
                filtered_count += 1;
                self.publish_item_filtered(feed_id, &guid, &reason).await?;
                continue;
            }
            
            // Convert to ProcessedRssItem
            let mut processed_item = self.convert_to_processed_item(
                feed_id,
                &config.name,
                item,
                guid.clone(),
            )?;
            
            // Apply NLP processing
            if config.transformations.iter().any(|t| matches!(
                t.transform_type,
                crate::rss_feed_manager::TransformType::ExtractEntities |
                crate::rss_feed_manager::TransformType::SentimentAnalysis |
                crate::rss_feed_manager::TransformType::Summarize |
                crate::rss_feed_manager::TransformType::ExtractKeywords
            )) {
                self.nlp_processor.process_rss_item(&mut processed_item).await?;
            }
            
            new_items.push(processed_item);
            feed_guids.push(guid);
        }
        
        // Keep only recent GUIDs (last 1000)
        if feed_guids.len() > 1000 {
            *feed_guids = feed_guids[feed_guids.len() - 1000..].to_vec();
        }
        
        drop(processed_guids); // Release the lock
        
        // Publish new items
        for item in new_items.iter() {
            self.publish_processed_item(item).await?;
        }
        
        // Publish feed status
        self.publish_feed_status(feed_id, new_items.len()).await?;
        
        info!(
            "Processed feed {}: {} new items, {} filtered",
            feed_id, new_items.len(), filtered_count
        );
        
        Ok(())
    }

    fn should_filter_item(&self, item: &Item, filters: &[FeedFilter]) -> Option<String> {
        for filter in filters {
            match &filter.filter_type {
                FilterType::TitleContains => {
                    if let Some(title) = item.title() {
                        if !title.to_lowercase().contains(&filter.value.to_lowercase()) {
                            return Some(format!("Title doesn't contain '{}'", filter.value));
                        }
                    }
                }
                FilterType::TitleRegex => {
                    if let Some(title) = item.title() {
                        if let Ok(re) = regex::Regex::new(&filter.value) {
                            if !re.is_match(title) {
                                return Some(format!("Title doesn't match regex '{}'", filter.value));
                            }
                        }
                    }
                }
                FilterType::DescriptionContains => {
                    if let Some(desc) = item.description() {
                        if !desc.to_lowercase().contains(&filter.value.to_lowercase()) {
                            return Some(format!("Description doesn't contain '{}'", filter.value));
                        }
                    }
                }
                FilterType::CategoryEquals => {
                    let categories: Vec<String> = item.categories()
                        .iter()
                        .map(|c| c.name().to_string())
                        .collect();
                    if !categories.contains(&filter.value) {
                        return Some(format!("Category doesn't match '{}'", filter.value));
                    }
                }
                FilterType::AuthorEquals => {
                    if let Some(author) = item.author() {
                        if author != filter.value {
                            return Some(format!("Author doesn't match '{}'", filter.value));
                        }
                    }
                }
                _ => {} // Date filters not implemented in this example
            }
        }
        None
    }

    fn convert_to_processed_item(
        &self,
        feed_id: &str,
        feed_name: &str,
        item: &Item,
        guid: String,
    ) -> Result<ProcessedRssItem> {
        let pub_date = item.pub_date()
            .and_then(|d| DateTime::parse_from_rfc2822(d).ok())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        
        Ok(ProcessedRssItem {
            id: Uuid::new_v4().to_string(),
            feed_id: feed_id.to_string(),
            feed_name: feed_name.to_string(),
            title: item.title().unwrap_or("Untitled").to_string(),
            description: item.description().unwrap_or("").to_string(),
            link: item.link().unwrap_or("").to_string(),
            pub_date,
            author: item.author().map(|s| s.to_string()),
            categories: item.categories()
                .iter()
                .map(|c| c.name().to_string())
                .collect(),
            guid: Some(guid),
            sentiment: None,
            entities: Vec::new(),
            keywords: Vec::new(),
            summary: None,
            relevance_score: 0.5,
            processed_at: Utc::now(),
            event_id: Uuid::new_v4(),
        })
    }

    async fn publish_processed_item(&self, item: &ProcessedRssItem) -> Result<()> {
        let subject = "rss.processed.items".to_string();
        let payload = serde_json::to_vec(item)?;
        
        self.jetstream.publish(subject, payload.into()).await?;
        Ok(())
    }

    async fn publish_feed_status(&self, feed_id: &str, item_count: usize) -> Result<()> {
        let subject = format!("rss.feeds.{}.status", feed_id);
        let payload = serde_json::to_vec(&serde_json::json!({
            "feed_id": feed_id,
            "item_count": item_count,
            "timestamp": Utc::now(),
        }))?;
        
        self.jetstream.publish(subject, payload.into()).await?;
        Ok(())
    }

    async fn publish_feed_error(&self, feed_id: &str, error: &str) -> Result<()> {
        let subject = format!("rss.feeds.{}.errors", feed_id);
        let payload = serde_json::to_vec(&serde_json::json!({
            "feed_id": feed_id,
            "error": error,
            "timestamp": Utc::now(),
        }))?;
        
        self.jetstream.publish(subject, payload.into()).await?;
        Ok(())
    }

    async fn publish_item_filtered(&self, feed_id: &str, item_id: &str, reason: &str) -> Result<()> {
        let subject = format!("rss.feeds.{}.filtered", feed_id);
        let payload = serde_json::to_vec(&serde_json::json!({
            "feed_id": feed_id,
            "item_id": item_id,
            "reason": reason,
            "timestamp": Utc::now(),
        }))?;
        
        self.jetstream.publish(subject, payload.into()).await?;
        Ok(())
    }

    async fn load_feed_configs(&self) -> Result<()> {
        // Load default configs (same as in RssFeedManager)
        let mut configs = HashMap::new();
        
        configs.insert("tech-news".to_string(), RssFeedConfig {
            id: "tech-news".to_string(),
            name: "Tech News Aggregator".to_string(),
            url: "https://news.ycombinator.com/rss".to_string(),
            category: "technology".to_string(),
            update_interval: 300,
            filters: vec![],
            transformations: vec![
                crate::rss_feed_manager::FeedTransformation {
                    transform_type: crate::rss_feed_manager::TransformType::ExtractEntities,
                    config: serde_json::json!({}),
                },
                crate::rss_feed_manager::FeedTransformation {
                    transform_type: crate::rss_feed_manager::TransformType::SentimentAnalysis,
                    config: serde_json::json!({}),
                },
                crate::rss_feed_manager::FeedTransformation {
                    transform_type: crate::rss_feed_manager::TransformType::ExtractKeywords,
                    config: serde_json::json!({}),
                },
            ],
            enabled: true,
        });
        
        configs.insert("arxiv-cs".to_string(), RssFeedConfig {
            id: "arxiv-cs".to_string(),
            name: "ArXiv Computer Science".to_string(),
            url: "http://arxiv.org/rss/cs".to_string(),
            category: "research".to_string(),
            update_interval: 3600,
            filters: vec![],
            transformations: vec![
                crate::rss_feed_manager::FeedTransformation {
                    transform_type: crate::rss_feed_manager::TransformType::Summarize,
                    config: serde_json::json!({ "max_length": 200 }),
                },
                crate::rss_feed_manager::FeedTransformation {
                    transform_type: crate::rss_feed_manager::TransformType::ExtractKeywords,
                    config: serde_json::json!({ "max_keywords": 5 }),
                },
            ],
            enabled: true,
        });
        
        *self.feed_configs.write().await = configs;
        Ok(())
    }

    async fn subscribe_to_config_updates(&self) -> Result<()> {
        // In production, subscribe to NATS for config updates
        // For now, just reload configs periodically
        let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes
        
        loop {
            interval.tick().await;
            if let Err(e) = self.load_feed_configs().await {
                warn!("Failed to reload feed configs: {}", e);
            }
        }
    }
}