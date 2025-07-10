//! RSS Feed Processor Example
//! 
//! Demonstrates how CIM processes RSS feeds and publishes them as event streams

use anyhow::Result;
use async_nats::jetstream;
use chrono::Utc;
use alchemist::rss_feed_manager::{
    ProcessedRssItem, Sentiment, Entity, RssFeedConfig, FeedFilter, FilterType,
    FeedTransformation, TransformType,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    println!("📰 RSS Feed Processor Example");
    println!("==============================\n");
    
    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = jetstream::new(client);
    
    // Create RSS stream
    create_rss_stream(&jetstream).await?;
    
    // Simulate RSS feed processing
    println!("Processing RSS feeds...\n");
    
    // Process tech news feed
    process_tech_news_feed(&jetstream).await?;
    
    // Process research papers feed
    process_arxiv_feed(&jetstream).await?;
    
    // Process custom feeds
    process_custom_feeds(&jetstream).await?;
    
    println!("\n✅ RSS feeds processed and published to NATS!");
    println!("\nFeatures demonstrated:");
    println!("- RSS item enrichment with NLP");
    println!("- Sentiment analysis");
    println!("- Entity extraction");
    println!("- Keyword extraction");
    println!("- Content summarization");
    println!("- Filtering and transformation");
    
    Ok(())
}

async fn create_rss_stream(js: &jetstream::Context) -> Result<()> {
    let stream_name = "RSS-FEEDS";
    
    match js.get_stream(stream_name).await {
        Ok(_) => {
            println!("✓ RSS stream '{}' already exists", stream_name);
        }
        Err(_) => {
            println!("Creating RSS stream '{}'...", stream_name);
            
            js.create_stream(jetstream::stream::Config {
                name: stream_name.to_string(),
                subjects: vec![
                    "rss.feeds.*.items".to_string(),
                    "rss.feeds.*.status".to_string(),
                    "rss.processed.items".to_string(),
                    "rss.config.*".to_string(),
                ],
                retention: jetstream::stream::RetentionPolicy::Limits,
                max_messages: 100_000,
                max_age: std::time::Duration::from_secs(86400 * 7), // 7 days
                storage: jetstream::stream::StorageType::File,
                num_replicas: 1,
                ..Default::default()
            }).await?;
            
            println!("✓ RSS stream created");
        }
    }
    
    Ok(())
}

async fn process_tech_news_feed(js: &jetstream::Context) -> Result<()> {
    println!("📡 Processing Tech News Feed...");
    
    // Simulate processing multiple RSS items
    let items = vec![
        ProcessedRssItem {
            id: Uuid::new_v4().to_string(),
            feed_id: "tech-news".to_string(),
            feed_name: "Hacker News".to_string(),
            title: "GraphQL Federation: Building Distributed Graph APIs at Scale".to_string(),
            description: "A deep dive into how major tech companies are using GraphQL federation to build massive distributed APIs that span multiple services and teams.".to_string(),
            link: "https://example.com/graphql-federation".to_string(),
            pub_date: Utc::now() - chrono::Duration::hours(1),
            author: Some("Tech Blog".to_string()),
            categories: vec!["API".to_string(), "GraphQL".to_string(), "Architecture".to_string()],
            guid: Some("hn-12345".to_string()),
            sentiment: Some(Sentiment {
                score: 0.7,
                label: "positive".to_string(),
                confidence: 0.92,
            }),
            entities: vec![
                Entity {
                    text: "GraphQL".to_string(),
                    entity_type: "TECHNOLOGY".to_string(),
                    confidence: 0.99,
                },
                Entity {
                    text: "API".to_string(),
                    entity_type: "TECHNOLOGY".to_string(),
                    confidence: 0.95,
                },
            ],
            keywords: vec!["graphql".to_string(), "federation".to_string(), "distributed systems".to_string(), "api design".to_string()],
            summary: Some("Major tech companies share best practices for building federated GraphQL APIs that scale across teams.".to_string()),
            relevance_score: 0.88,
            processed_at: Utc::now(),
            event_id: Uuid::new_v4(),
        },
        ProcessedRssItem {
            id: Uuid::new_v4().to_string(),
            feed_id: "tech-news".to_string(),
            feed_name: "Hacker News".to_string(),
            title: "The Dark Side of AI: Energy Consumption Reaches Critical Levels".to_string(),
            description: "New research reveals the environmental impact of training large language models, with some models consuming as much energy as small cities.".to_string(),
            link: "https://example.com/ai-energy".to_string(),
            pub_date: Utc::now() - chrono::Duration::hours(3),
            author: Some("Environmental Tech Report".to_string()),
            categories: vec!["AI".to_string(), "Environment".to_string()],
            guid: Some("hn-12346".to_string()),
            sentiment: Some(Sentiment {
                score: -0.6,
                label: "negative".to_string(),
                confidence: 0.88,
            }),
            entities: vec![
                Entity {
                    text: "AI".to_string(),
                    entity_type: "TECHNOLOGY".to_string(),
                    confidence: 0.99,
                },
                Entity {
                    text: "language models".to_string(),
                    entity_type: "CONCEPT".to_string(),
                    confidence: 0.94,
                },
            ],
            keywords: vec!["ai".to_string(), "energy consumption".to_string(), "environmental impact".to_string(), "sustainability".to_string()],
            summary: Some("Research highlights concerning energy usage of AI model training and calls for more sustainable practices.".to_string()),
            relevance_score: 0.91,
            processed_at: Utc::now(),
            event_id: Uuid::new_v4(),
        },
    ];
    
    // Publish items
    for item in items {
        let subject = "rss.processed.items";
        let payload = serde_json::to_vec(&item)?;
        js.publish(subject, payload.into()).await?;
        println!("  ✓ Published: {}", item.title);
    }
    
    // Publish feed status
    let status = serde_json::json!({
        "feed_id": "tech-news",
        "item_count": 2,
        "timestamp": Utc::now(),
    });
    
    js.publish("rss.feeds.tech-news.status", serde_json::to_vec(&status)?.into()).await?;
    
    Ok(())
}

async fn process_arxiv_feed(js: &jetstream::Context) -> Result<()> {
    println!("\n📚 Processing ArXiv Research Feed...");
    
    let item = ProcessedRssItem {
        id: Uuid::new_v4().to_string(),
        feed_id: "arxiv-cs".to_string(),
        feed_name: "ArXiv Computer Science".to_string(),
        title: "Categorical Event Sourcing: A Category-Theoretic Approach to Event-Driven Systems".to_string(),
        description: "We present a novel formalization of event sourcing using category theory, demonstrating how functors and natural transformations provide a rigorous foundation for event-driven architectures.".to_string(),
        link: "https://arxiv.org/abs/2401.98765".to_string(),
        pub_date: Utc::now() - chrono::Duration::hours(12),
        author: Some("Smith, J. et al.".to_string()),
        categories: vec!["cs.SE".to_string(), "cs.PL".to_string()],
        guid: Some("arxiv-2401.98765".to_string()),
        sentiment: Some(Sentiment {
            score: 0.1,
            label: "neutral".to_string(),
            confidence: 0.95,
        }),
        entities: vec![
            Entity {
                text: "Event Sourcing".to_string(),
                entity_type: "CONCEPT".to_string(),
                confidence: 0.99,
            },
            Entity {
                text: "Category Theory".to_string(),
                entity_type: "FIELD".to_string(),
                confidence: 0.98,
            },
            Entity {
                text: "Smith, J.".to_string(),
                entity_type: "PERSON".to_string(),
                confidence: 0.92,
            },
        ],
        keywords: vec!["event sourcing".to_string(), "category theory".to_string(), "functors".to_string(), "natural transformations".to_string(), "formal methods".to_string()],
        summary: Some("Authors formalize event sourcing using category theory, showing how categorical constructs map to event-driven patterns.".to_string()),
        relevance_score: 0.96,
        processed_at: Utc::now(),
        event_id: Uuid::new_v4(),
    };
    
    let payload = serde_json::to_vec(&item)?;
    js.publish("rss.processed.items", payload.into()).await?;
    println!("  ✓ Published: {}", item.title);
    
    Ok(())
}

async fn process_custom_feeds(js: &jetstream::Context) -> Result<()> {
    println!("\n🔧 Publishing Custom Feed Configurations...");
    
    // Example: AI/ML focused feed with filters
    let ai_feed = RssFeedConfig {
        id: "ai-ml-weekly".to_string(),
        name: "AI/ML Weekly Digest".to_string(),
        url: "https://example.com/ai-ml-feed.xml".to_string(),
        category: "artificial-intelligence".to_string(),
        update_interval: 3600, // 1 hour
        filters: vec![
            FeedFilter {
                filter_type: FilterType::TitleContains,
                value: "machine learning".to_string(),
            },
            FeedFilter {
                filter_type: FilterType::TitleContains,
                value: "neural network".to_string(),
            },
            FeedFilter {
                filter_type: FilterType::CategoryEquals,
                value: "AI".to_string(),
            },
        ],
        transformations: vec![
            FeedTransformation {
                transform_type: TransformType::SentimentAnalysis,
                config: serde_json::json!({ "model": "bert-sentiment" }),
            },
            FeedTransformation {
                transform_type: TransformType::ExtractEntities,
                config: serde_json::json!({ "types": ["PERSON", "ORG", "TECHNOLOGY"] }),
            },
            FeedTransformation {
                transform_type: TransformType::Summarize,
                config: serde_json::json!({ "max_length": 150, "model": "bart-summarization" }),
            },
        ],
        enabled: true,
    };
    
    let payload = serde_json::to_vec(&ai_feed)?;
    js.publish("rss.config.ai-ml-weekly", payload.into()).await?;
    println!("  ✓ Published config for: {}", ai_feed.name);
    
    // Example: Security alerts feed
    let security_feed = RssFeedConfig {
        id: "security-alerts".to_string(),
        name: "Security Vulnerability Alerts".to_string(),
        url: "https://nvd.nist.gov/feeds/xml/cve/misc/nvd-rss.xml".to_string(),
        category: "security".to_string(),
        update_interval: 900, // 15 minutes
        filters: vec![
            FeedFilter {
                filter_type: FilterType::TitleRegex,
                value: r"CVE-\d{4}-\d+".to_string(),
            },
        ],
        transformations: vec![
            FeedTransformation {
                transform_type: TransformType::ExtractKeywords,
                config: serde_json::json!({ "max_keywords": 10, "focus": ["vulnerability", "exploit", "patch"] }),
            },
            FeedTransformation {
                transform_type: TransformType::Categorize,
                config: serde_json::json!({ "categories": ["critical", "high", "medium", "low"] }),
            },
        ],
        enabled: true,
    };
    
    let payload = serde_json::to_vec(&security_feed)?;
    js.publish("rss.config.security-alerts", payload.into()).await?;
    println!("  ✓ Published config for: {}", security_feed.name);
    
    Ok(())
}

// Helper to show how filters work
fn demonstrate_filtering() {
    println!("\n🔍 RSS Feed Filtering Examples:");
    println!("- Title Contains: Filter items with specific keywords");
    println!("- Title Regex: Advanced pattern matching (e.g., CVE-*)");
    println!("- Category Equals: Exact category matching");
    println!("- Date After/Before: Time-based filtering");
    
    println!("\n🔄 RSS Feed Transformations:");
    println!("- Sentiment Analysis: Classify as positive/negative/neutral");
    println!("- Entity Extraction: Find people, orgs, technologies");
    println!("- Summarization: AI-powered content summaries");
    println!("- Keyword Extraction: Identify key topics");
    println!("- Auto-categorization: Smart content classification");
}