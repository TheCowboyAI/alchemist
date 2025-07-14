//! NLP Processor for RSS feeds using real AI providers

use anyhow::Result;
use crate::ai::AiManager;
use crate::rss_feed_manager::{ProcessedRssItem, Sentiment, Entity};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, error};

/// NLP processor that uses real AI providers for text analysis
pub struct NlpProcessor {
    ai_manager: AiManager,
    default_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NlpAnalysisRequest {
    text: String,
    title: String,
    analyses: Vec<AnalysisType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AnalysisType {
    Sentiment,
    EntityExtraction,
    Keywords,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NlpAnalysisResponse {
    sentiment: Option<SentimentResult>,
    entities: Vec<EntityResult>,
    keywords: Vec<String>,
    summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SentimentResult {
    score: f32,
    label: String,
    confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntityResult {
    text: String,
    entity_type: String,
    confidence: f32,
}

impl NlpProcessor {
    pub fn new(ai_manager: AiManager, default_model: String) -> Self {
        Self {
            ai_manager,
            default_model,
        }
    }

    /// Process an RSS item with NLP enrichments
    pub async fn process_rss_item(&self, item: &mut ProcessedRssItem) -> Result<()> {
        let combined_text = format!("{}\n\n{}", item.title, item.description);
        
        // Perform sentiment analysis
        if let Ok(sentiment) = self.analyze_sentiment(&combined_text).await {
            item.sentiment = Some(sentiment);
        }
        
        // Extract entities
        if let Ok(entities) = self.extract_entities(&combined_text).await {
            item.entities = entities;
        }
        
        // Extract keywords
        if let Ok(keywords) = self.extract_keywords(&combined_text).await {
            item.keywords = keywords;
        }
        
        // Generate summary
        if let Ok(summary) = self.generate_summary(&combined_text).await {
            item.summary = Some(summary);
        }
        
        // Calculate relevance score based on enrichments
        item.relevance_score = self.calculate_relevance_score(item);
        
        Ok(())
    }

    async fn analyze_sentiment(&self, text: &str) -> Result<Sentiment> {
        let prompt = format!(
            r#"Analyze the sentiment of the following text. Respond with ONLY a JSON object in this exact format:
{{
  "score": <number between -1.0 and 1.0>,
  "label": "<positive|negative|neutral>",
  "confidence": <number between 0.0 and 1.0>
}}

Text to analyze:
{}"#,
            text
        );

        let response = self.ai_manager.get_completion(&self.default_model, &prompt).await?;
        
        // Parse JSON response
        match serde_json::from_str::<SentimentResult>(&response) {
            Ok(result) => Ok(Sentiment {
                score: result.score,
                label: result.label,
                confidence: result.confidence,
            }),
            Err(e) => {
                debug!("Failed to parse sentiment response: {}, raw: {}", e, response);
                // Fallback to neutral sentiment
                Ok(Sentiment {
                    score: 0.0,
                    label: "neutral".to_string(),
                    confidence: 0.5,
                })
            }
        }
    }

    async fn extract_entities(&self, text: &str) -> Result<Vec<Entity>> {
        let prompt = format!(
            r#"Extract named entities from the following text. Respond with ONLY a JSON array of entities in this exact format:
[
  {{
    "text": "<entity text>",
    "entity_type": "<PERSON|ORGANIZATION|LOCATION|TECHNOLOGY|PRODUCT|EVENT>",
    "confidence": <number between 0.0 and 1.0>
  }}
]

Text to analyze:
{}"#,
            text
        );

        let response = self.ai_manager.get_completion(&self.default_model, &prompt).await?;
        
        // Parse JSON response
        match serde_json::from_str::<Vec<EntityResult>>(&response) {
            Ok(results) => Ok(results.into_iter()
                .map(|r| Entity {
                    text: r.text,
                    entity_type: r.entity_type,
                    confidence: r.confidence,
                })
                .collect()),
            Err(e) => {
                debug!("Failed to parse entities response: {}, raw: {}", e, response);
                Ok(Vec::new())
            }
        }
    }

    async fn extract_keywords(&self, text: &str) -> Result<Vec<String>> {
        let prompt = format!(
            r#"Extract the 5 most important keywords or phrases from the following text. Respond with ONLY a JSON array of strings:
["keyword1", "keyword2", "keyword3", "keyword4", "keyword5"]

Text to analyze:
{}"#,
            text
        );

        let response = self.ai_manager.get_completion(&self.default_model, &prompt).await?;
        
        // Parse JSON response
        match serde_json::from_str::<Vec<String>>(&response) {
            Ok(keywords) => Ok(keywords),
            Err(e) => {
                debug!("Failed to parse keywords response: {}, raw: {}", e, response);
                Ok(Vec::new())
            }
        }
    }

    async fn generate_summary(&self, text: &str) -> Result<String> {
        let prompt = format!(
            r#"Provide a concise summary (max 200 characters) of the following text:

Text to summarize:
{}

Summary:"#,
            text
        );

        let response = self.ai_manager.get_completion(&self.default_model, &prompt).await?;
        
        // Trim and limit response
        let summary = response.trim();
        if summary.len() > 200 {
            Ok(summary.chars().take(197).collect::<String>() + "...")
        } else {
            Ok(summary.to_string())
        }
    }

    fn calculate_relevance_score(&self, item: &ProcessedRssItem) -> f32 {
        let mut score = 0.5; // Base score
        
        // Boost score based on sentiment confidence
        if let Some(sentiment) = &item.sentiment {
            score += sentiment.confidence * 0.1;
        }
        
        // Boost score based on entity count
        let entity_boost = (item.entities.len() as f32 * 0.05).min(0.2);
        score += entity_boost;
        
        // Boost score based on keyword count
        let keyword_boost = (item.keywords.len() as f32 * 0.05).min(0.15);
        score += keyword_boost;
        
        // Boost score if summary exists
        if item.summary.is_some() {
            score += 0.05;
        }
        
        score.min(1.0) // Cap at 1.0
    }

    /// Process a batch of RSS items
    pub async fn process_batch(&self, items: &mut [ProcessedRssItem]) -> Result<()> {
        info!("Processing batch of {} RSS items with NLP", items.len());
        
        for item in items {
            if let Err(e) = self.process_rss_item(item).await {
                error!("Failed to process RSS item {}: {}", item.id, e);
            }
        }
        
        Ok(())
    }
}

/// Configuration for NLP processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlpConfig {
    pub enabled: bool,
    pub ai_model: String,
    pub batch_size: usize,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

impl Default for NlpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ai_model: "gpt-3.5-turbo".to_string(),
            batch_size: 10,
            timeout_seconds: 30,
            retry_attempts: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relevance_score_calculation() {
        let processor = NlpProcessor {
            ai_manager: todo!(), // Would be mocked in real tests
            default_model: "test-model".to_string(),
        };

        let mut item = ProcessedRssItem {
            id: "test".to_string(),
            feed_id: "test-feed".to_string(),
            feed_name: "Test Feed".to_string(),
            title: "Test Title".to_string(),
            description: "Test Description".to_string(),
            link: "http://example.com".to_string(),
            pub_date: Utc::now(),
            author: None,
            categories: vec![],
            guid: None,
            sentiment: Some(Sentiment {
                score: 0.8,
                label: "positive".to_string(),
                confidence: 0.9,
            }),
            entities: vec![
                Entity {
                    text: "Entity1".to_string(),
                    entity_type: "PERSON".to_string(),
                    confidence: 0.9,
                },
                Entity {
                    text: "Entity2".to_string(),
                    entity_type: "ORGANIZATION".to_string(),
                    confidence: 0.8,
                },
            ],
            keywords: vec!["keyword1".to_string(), "keyword2".to_string()],
            summary: Some("Summary".to_string()),
            relevance_score: 0.0,
            processed_at: Utc::now(),
            event_id: uuid::Uuid::new_v4(),
        };

        let score = processor.calculate_relevance_score(&item);
        
        // Base: 0.5
        // Sentiment confidence: 0.9 * 0.1 = 0.09
        // Entity boost: 2 * 0.05 = 0.1
        // Keyword boost: 2 * 0.05 = 0.1
        // Summary boost: 0.05
        // Total: 0.5 + 0.09 + 0.1 + 0.1 + 0.05 = 0.84
        assert!((score - 0.84).abs() < 0.01);
    }
}