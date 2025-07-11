//! Cache integration for Alchemist components

use anyhow::Result;
use std::time::Duration;
use crate::{
    cache::{CacheManager, LayeredCache},
    dialog::{DialogMessage, MessageRole},
    ai::AiManager,
};

/// Dialog cache keys
pub mod dialog_keys {
    pub fn messages(dialog_id: &str) -> String {
        format!("dialog:messages:{}", dialog_id)
    }
    
    pub fn summary(dialog_id: &str) -> String {
        format!("dialog:summary:{}", dialog_id)
    }
    
    pub fn recent_dialogs(user_id: &str) -> String {
        format!("user:dialogs:recent:{}", user_id)
    }
}

/// AI cache keys
pub mod ai_keys {
    pub fn completion(model: &str, prompt_hash: u64) -> String {
        format!("ai:completion:{}:{}", model, prompt_hash)
    }
    
    pub fn model_status(model: &str) -> String {
        format!("ai:status:{}", model)
    }
    
    pub fn common_prompts() -> String {
        "ai:prompts:common".to_string()
    }
}

/// Cached AI manager
pub struct CachedAiManager {
    ai_manager: AiManager,
    cache: CacheManager,
}

impl CachedAiManager {
    pub fn new(ai_manager: AiManager, cache: CacheManager) -> Self {
        Self { ai_manager, cache }
    }
    
    /// Get completion with caching for common queries
    pub async fn get_completion_cached(
        &self,
        model: &str,
        prompt: &str,
    ) -> Result<String> {
        // Hash the prompt for cache key
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        let prompt_hash = hasher.finish();
        
        // Check if this is a common/repeated query
        let cache_key = ai_keys::completion(model, prompt_hash);
        let ttl = Duration::from_secs(3600); // 1 hour for completions
        
        self.cache.get_or_compute(
            &cache_key,
            ttl,
            || async {
                self.ai_manager.get_completion(model, prompt).await
            }
        ).await
    }
    
    /// Stream completion (no caching for streams)
    pub async fn stream_completion(
        &self,
        model: &str,
        prompt: &str,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        self.ai_manager.stream_completion(model, prompt).await
    }
}

/// Cached dialog operations
pub struct CachedDialogOperations {
    cache: CacheManager,
}

impl CachedDialogOperations {
    pub fn new(cache: CacheManager) -> Self {
        Self { cache }
    }
    
    /// Cache recent messages for a dialog
    pub async fn cache_messages(
        &self,
        dialog_id: &str,
        messages: &[DialogMessage],
    ) -> Result<()> {
        let cache_key = dialog_keys::messages(dialog_id);
        let ttl = Duration::from_secs(300); // 5 minutes
        
        // Only cache recent messages
        let recent: Vec<_> = messages.iter()
            .rev()
            .take(20)
            .cloned()
            .collect();
        
        self.cache.cache.set(&cache_key, &recent, ttl).await
    }
    
    /// Get cached messages
    pub async fn get_cached_messages(
        &self,
        dialog_id: &str,
    ) -> Option<Vec<DialogMessage>> {
        let cache_key = dialog_keys::messages(dialog_id);
        self.cache.cache.get(&cache_key).await
    }
    
    /// Generate and cache dialog summary
    pub async fn get_or_generate_summary(
        &self,
        dialog_id: &str,
        messages: &[DialogMessage],
        ai_manager: &CachedAiManager,
    ) -> Result<String> {
        let cache_key = dialog_keys::summary(dialog_id);
        let ttl = Duration::from_secs(1800); // 30 minutes
        
        self.cache.get_or_compute(
            &cache_key,
            ttl,
            || async {
                // Generate summary prompt
                let mut prompt = String::from("Summarize this conversation in 2-3 sentences:\n\n");
                
                for msg in messages.iter().take(10) {
                    let role = match msg.role {
                        MessageRole::User => "User",
                        MessageRole::Assistant => "Assistant",
                        MessageRole::System => "System",
                    };
                    prompt.push_str(&format!("{}: {}\n", role, msg.content));
                }
                
                prompt.push_str("\nSummary:");
                
                ai_manager.get_completion_cached("claude-3-sonnet", &prompt).await
            }
        ).await
    }
}

/// Performance monitoring cache
pub struct MetricsCache {
    cache: CacheManager,
}

impl MetricsCache {
    pub fn new(cache: CacheManager) -> Self {
        Self { cache }
    }
    
    /// Cache aggregated metrics
    pub async fn cache_metrics<T: serde::Serialize>(
        &self,
        metric_type: &str,
        period: &str,
        data: &T,
    ) -> Result<()> {
        let key = format!("metrics:{}:{}", metric_type, period);
        let ttl = match period {
            "1m" => Duration::from_secs(60),
            "5m" => Duration::from_secs(300),
            "1h" => Duration::from_secs(3600),
            _ => Duration::from_secs(60),
        };
        
        self.cache.cache.set(&key, data, ttl).await
    }
    
    /// Get cached metrics
    pub async fn get_metrics<T: serde::de::DeserializeOwned>(
        &self,
        metric_type: &str,
        period: &str,
    ) -> Option<T> {
        let key = format!("metrics:{}:{}", metric_type, period);
        self.cache.cache.get(&key).await
    }
}

/// Initialize caching for the application
pub fn init_cache(config: &crate::config::AlchemistConfig) -> CacheManager {
    let redis_url = config.cache.as_ref()
        .and_then(|c| c.redis_url.as_deref());
    
    let cache = LayeredCache::new(redis_url, "alchemist");
    CacheManager::new(std::sync::Arc::new(cache))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::MemoryCache;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_dialog_caching() {
        let cache = Arc::new(MemoryCache::new(100));
        let manager = CacheManager::new(cache);
        let ops = CachedDialogOperations::new(manager);
        
        let messages = vec![
            DialogMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                timestamp: chrono::Utc::now(),
                tokens: None,
            },
            DialogMessage {
                role: MessageRole::Assistant,
                content: "Hi there!".to_string(),
                timestamp: chrono::Utc::now(),
                tokens: Some(3),
            },
        ];
        
        // Cache messages
        ops.cache_messages("test_dialog", &messages).await.unwrap();
        
        // Retrieve cached messages
        let cached = ops.get_cached_messages("test_dialog").await.unwrap();
        assert_eq!(cached.len(), 2);
        assert_eq!(cached[0].content, "Hi there!"); // Reversed order
    }
}