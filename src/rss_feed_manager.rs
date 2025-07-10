//! RSS Feed Manager - Consumes and displays RSS feeds as event streams from NATS

use anyhow::Result;
use async_nats::jetstream::{self, consumer::PullConsumer, stream::Stream};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, debug};
use uuid::Uuid;
use futures::StreamExt;

/// RSS Feed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssFeedConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub category: String,
    pub update_interval: u64, // seconds
    pub filters: Vec<FeedFilter>,
    pub transformations: Vec<FeedTransformation>,
    pub enabled: bool,
}

/// Filter to apply to RSS items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedFilter {
    pub filter_type: FilterType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterType {
    TitleContains,
    TitleRegex,
    DescriptionContains,
    CategoryEquals,
    AuthorEquals,
    DateAfter,
    DateBefore,
}

/// Transformation to apply to RSS items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedTransformation {
    pub transform_type: TransformType,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformType {
    ExtractEntities,    // NER extraction
    SentimentAnalysis,  // Analyze sentiment
    Summarize,          // AI summarization
    Categorize,         // Auto-categorization
    ExtractKeywords,    // Keyword extraction
}

/// RSS item after processing through CIM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedRssItem {
    pub id: String,
    pub feed_id: String,
    pub feed_name: String,
    pub title: String,
    pub description: String,
    pub link: String,
    pub pub_date: DateTime<Utc>,
    pub author: Option<String>,
    pub categories: Vec<String>,
    pub guid: Option<String>,
    
    // CIM enrichments
    pub sentiment: Option<Sentiment>,
    pub entities: Vec<Entity>,
    pub keywords: Vec<String>,
    pub summary: Option<String>,
    pub relevance_score: f32,
    
    // Event metadata
    pub processed_at: DateTime<Utc>,
    pub event_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    pub score: f32, // -1.0 to 1.0
    pub label: String, // positive, negative, neutral
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub text: String,
    pub entity_type: String, // PERSON, ORG, LOCATION, etc.
    pub confidence: f32,
}

/// RSS Feed event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RssFeedEvent {
    /// New RSS item discovered
    ItemDiscovered {
        item: ProcessedRssItem,
        feed_id: String,
        timestamp: DateTime<Utc>,
    },
    
    /// RSS feed updated
    FeedUpdated {
        feed_id: String,
        item_count: usize,
        timestamp: DateTime<Utc>,
    },
    
    /// RSS feed error
    FeedError {
        feed_id: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
    
    /// RSS item filtered out
    ItemFiltered {
        feed_id: String,
        item_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
}

/// RSS Feed Manager that consumes feed events from NATS
pub struct RssFeedManager {
    jetstream: jetstream::Context,
    feeds: HashMap<String, RssFeedConfig>,
    event_tx: mpsc::Sender<RssFeedEvent>,
}

impl RssFeedManager {
    pub async fn new(
        nats_client: async_nats::Client,
        event_tx: mpsc::Sender<RssFeedEvent>,
    ) -> Result<Self> {
        let jetstream = jetstream::new(nats_client.clone());
        
        // Load feed configurations
        let feeds = Self::load_feed_configs().await?;
        
        Ok(Self {
            jetstream,
            feeds,
            event_tx,
        })
    }
    
    /// Start consuming RSS feed events from NATS
    pub async fn start(&self) -> Result<()> {
        // Subscribe to RSS feed subjects
        let subjects = vec![
            "rss.feeds.*.items".to_string(),      // Individual items
            "rss.feeds.*.status".to_string(),     // Feed status updates
            "rss.feeds.*.errors".to_string(),     // Feed errors
            "rss.processed.items".to_string(),     // Processed items
        ];
        
        // Get or create RSS stream
        let stream = self.get_or_create_rss_stream(&subjects).await?;
        
        // Create consumer
        let consumer = self.get_or_create_consumer(&stream).await?;
        
        // Process messages
        let mut messages = consumer.messages().await?;
        
        info!("RSS Feed Manager started, consuming events...");
        
        while let Some(Ok(msg)) = messages.next().await {
            let subject = msg.subject.clone();
            
            // Route based on subject pattern
            if subject.starts_with("rss.processed.items") {
                self.handle_processed_item(&msg.payload).await?;
            } else if subject.contains(".status") {
                self.handle_feed_status(&msg.payload).await?;
            } else if subject.contains(".errors") {
                self.handle_feed_error(&msg.payload).await?;
            }
            
            if let Err(e) = msg.ack().await {
                debug!("Failed to ack message: {}", e);
            }
        }
        
        Ok(())
    }
    
    async fn get_or_create_rss_stream(&self, subjects: &[String]) -> Result<Stream> {
        let stream_name = "RSS-FEEDS";
        
        match self.jetstream.get_stream(stream_name).await {
            Ok(stream) => Ok(stream),
            Err(_) => {
                info!("Creating RSS stream: {}", stream_name);
                
                let stream = self.jetstream
                    .create_stream(jetstream::stream::Config {
                        name: stream_name.to_string(),
                        subjects: subjects.to_vec(),
                        retention: jetstream::stream::RetentionPolicy::Limits,
                        max_messages: 100_000,
                        max_age: std::time::Duration::from_secs(86400 * 7), // 7 days
                        storage: jetstream::stream::StorageType::File,
                        num_replicas: 1,
                        ..Default::default()
                    })
                    .await?;
                
                Ok(stream)
            }
        }
    }
    
    async fn get_or_create_consumer(&self, stream: &Stream) -> Result<PullConsumer> {
        let consumer_name = "rss-dashboard-consumer";
        
        match stream.get_consumer(consumer_name).await {
            Ok(consumer) => Ok(consumer),
            Err(_) => {
                let consumer = stream
                    .create_consumer(jetstream::consumer::pull::Config {
                        name: Some(consumer_name.to_string()),
                        durable_name: Some(consumer_name.to_string()),
                        ..Default::default()
                    })
                    .await?;
                
                Ok(consumer)
            }
        }
    }
    
    async fn handle_processed_item(&self, payload: &[u8]) -> Result<()> {
        let item: ProcessedRssItem = serde_json::from_slice(payload)?;
        
        debug!("Received processed RSS item: {}", item.title);
        
        let event = RssFeedEvent::ItemDiscovered {
            feed_id: item.feed_id.clone(),
            timestamp: item.processed_at,
            item,
        };
        
        self.event_tx.send(event).await?;
        Ok(())
    }
    
    async fn handle_feed_status(&self, payload: &[u8]) -> Result<()> {
        #[derive(Deserialize)]
        struct FeedStatus {
            feed_id: String,
            item_count: usize,
            timestamp: DateTime<Utc>,
        }
        
        let status: FeedStatus = serde_json::from_slice(payload)?;
        
        let event = RssFeedEvent::FeedUpdated {
            feed_id: status.feed_id,
            item_count: status.item_count,
            timestamp: status.timestamp,
        };
        
        self.event_tx.send(event).await?;
        Ok(())
    }
    
    async fn handle_feed_error(&self, payload: &[u8]) -> Result<()> {
        #[derive(Deserialize)]
        struct FeedError {
            feed_id: String,
            error: String,
            timestamp: DateTime<Utc>,
        }
        
        let error: FeedError = serde_json::from_slice(payload)?;
        
        let event = RssFeedEvent::FeedError {
            feed_id: error.feed_id,
            error: error.error,
            timestamp: error.timestamp,
        };
        
        self.event_tx.send(event).await?;
        Ok(())
    }
    
    async fn load_feed_configs() -> Result<HashMap<String, RssFeedConfig>> {
        // In production, load from configuration or database
        // For demo, create example feeds
        let mut feeds = HashMap::new();
        
        feeds.insert("tech-news".to_string(), RssFeedConfig {
            id: "tech-news".to_string(),
            name: "Tech News Aggregator".to_string(),
            url: "https://news.ycombinator.com/rss".to_string(),
            category: "technology".to_string(),
            update_interval: 300, // 5 minutes
            filters: vec![
                FeedFilter {
                    filter_type: FilterType::TitleContains,
                    value: "AI".to_string(),
                },
            ],
            transformations: vec![
                FeedTransformation {
                    transform_type: TransformType::ExtractEntities,
                    config: serde_json::json!({}),
                },
                FeedTransformation {
                    transform_type: TransformType::SentimentAnalysis,
                    config: serde_json::json!({}),
                },
            ],
            enabled: true,
        });
        
        feeds.insert("arxiv-cs".to_string(), RssFeedConfig {
            id: "arxiv-cs".to_string(),
            name: "ArXiv Computer Science".to_string(),
            url: "http://arxiv.org/rss/cs".to_string(),
            category: "research".to_string(),
            update_interval: 3600, // 1 hour
            filters: vec![
                FeedFilter {
                    filter_type: FilterType::TitleContains,
                    value: "graph".to_string(),
                },
            ],
            transformations: vec![
                FeedTransformation {
                    transform_type: TransformType::Summarize,
                    config: serde_json::json!({ "max_length": 200 }),
                },
                FeedTransformation {
                    transform_type: TransformType::ExtractKeywords,
                    config: serde_json::json!({ "max_keywords": 5 }),
                },
            ],
            enabled: true,
        });
        
        Ok(feeds)
    }
    
    pub fn get_feeds(&self) -> Vec<RssFeedConfig> {
        self.feeds.values().cloned().collect()
    }
    
    pub async fn enable_feed(&mut self, feed_id: &str) -> Result<()> {
        if let Some(feed) = self.feeds.get_mut(feed_id) {
            feed.enabled = true;
        }
        // Publish configuration change after mutable borrow ends
        if let Some(feed) = self.feeds.get(feed_id) {
            self.publish_feed_config(feed).await?;
        }
        Ok(())
    }
    
    pub async fn disable_feed(&mut self, feed_id: &str) -> Result<()> {
        if let Some(feed) = self.feeds.get_mut(feed_id) {
            feed.enabled = false;
        }
        // Publish configuration change after mutable borrow ends
        if let Some(feed) = self.feeds.get(feed_id) {
            self.publish_feed_config(feed).await?;
        }
        Ok(())
    }
    
    async fn publish_feed_config(&self, feed: &RssFeedConfig) -> Result<()> {
        let subject = format!("rss.config.{}", feed.id);
        let payload = serde_json::to_vec(feed)?;
        
        self.jetstream.publish(subject, payload.into()).await?;
        Ok(())
    }
}

/// RSS Feed projection for dashboard display
#[derive(Debug, Clone)]
pub struct RssFeedProjection {
    pub feeds: HashMap<String, FeedState>,
    pub recent_items: Vec<ProcessedRssItem>,
    pub statistics: FeedStatistics,
}

#[derive(Debug, Clone)]
pub struct FeedState {
    pub config: RssFeedConfig,
    pub last_update: Option<DateTime<Utc>>,
    pub item_count: usize,
    pub error_count: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FeedStatistics {
    pub total_items: usize,
    pub items_per_hour: f64,
    pub sentiment_distribution: HashMap<String, usize>,
    pub top_entities: Vec<(String, usize)>,
    pub top_keywords: Vec<(String, usize)>,
}

impl RssFeedProjection {
    pub fn new() -> Self {
        Self {
            feeds: HashMap::new(),
            recent_items: Vec::with_capacity(100),
            statistics: FeedStatistics::default(),
        }
    }
    
    pub fn apply_event(&mut self, event: RssFeedEvent) {
        match event {
            RssFeedEvent::ItemDiscovered { item, feed_id, .. } => {
                // Update statistics
                self.statistics.total_items += 1;
                
                if let Some(sentiment) = &item.sentiment {
                    *self.statistics.sentiment_distribution
                        .entry(sentiment.label.clone())
                        .or_insert(0) += 1;
                }
                
                // Track entities
                for _entity in &item.entities {
                    // Update top entities tracking
                }
                
                // Add to recent items
                self.recent_items.push(item);
                if self.recent_items.len() > 100 {
                    self.recent_items.remove(0);
                }
                
                // Update feed state
                if let Some(feed_state) = self.feeds.get_mut(&feed_id) {
                    feed_state.item_count += 1;
                    feed_state.last_update = Some(Utc::now());
                }
            }
            
            RssFeedEvent::FeedError { feed_id, error, .. } => {
                if let Some(feed_state) = self.feeds.get_mut(&feed_id) {
                    feed_state.error_count += 1;
                    feed_state.last_error = Some(error);
                }
            }
            
            _ => {}
        }
    }
}