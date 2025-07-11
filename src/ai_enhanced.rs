//! Enhanced AI manager with caching and rate limiting

use anyhow::{Result, Context};
use std::sync::Arc;
use tracing::{debug, warn, error};
use futures::StreamExt;

use crate::{
    ai::{AiManager, StreamingResponseStream},
    cache::CacheManager,
    rate_limiter::{AiRateLimiter, CircuitBreaker, CircuitBreakerConfig},
    config::AlchemistConfig,
    user_context::UserContext,
};

/// Enhanced AI manager with caching and rate limiting
pub struct EnhancedAiManager {
    ai_manager: AiManager,
    cache: Option<CacheManager>,
    rate_limiter: AiRateLimiter,
    circuit_breaker: CircuitBreaker,
}

impl EnhancedAiManager {
    pub async fn new(config: &AlchemistConfig) -> Result<Self> {
        let ai_manager = AiManager::new(config).await?;
        
        // Initialize cache if configured
        let cache = if config.cache.is_some() {
            Some(crate::cache_integration::init_cache(config))
        } else {
            None
        };
        
        let rate_limiter = AiRateLimiter::new();
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        
        Ok(Self {
            ai_manager,
            cache,
            rate_limiter,
            circuit_breaker,
        })
    }
    
    /// Get completion with caching and rate limiting
    pub async fn get_completion(&self, model: &str, prompt: &str) -> Result<String> {
        // Get user context
        let context = UserContext::current();
        let user_id = context.user_id();
        
        // Check rate limit with user context
        if !self.rate_limiter.check_model_limit(model, &user_id).await? {
            return Err(anyhow::anyhow!("Rate limit exceeded for model: {} (user: {})", model, user_id));
        }
        
        // Use circuit breaker for fault tolerance
        self.circuit_breaker.call(async {
            // Check cache first
            if let Some(cache) = &self.cache {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                
                let mut hasher = DefaultHasher::new();
                prompt.hash(&mut hasher);
                let prompt_hash = hasher.finish();
                
                let cache_key = format!("ai:completion:{}:{}", model, prompt_hash);
                let ttl = std::time::Duration::from_secs(3600);
                
                // Try to get from cache
                if let Some(cached): Option<String> = cache.cache.get(&cache_key).await {
                    debug!("Cache hit for AI completion");
                    return Ok(cached);
                }
                
                // Get completion from AI
                let completion = self.get_completion_from_stream(model, prompt).await?;
                
                // Cache the result
                if let Err(e) = cache.cache.set(&cache_key, &completion, ttl).await {
                    warn!("Failed to cache AI completion: {}", e);
                }
                
                Ok(completion)
            } else {
                // No cache, get directly
                self.get_completion_from_stream(model, prompt).await
            }
        }).await
    }
    
    /// Get completion by collecting stream response
    async fn get_completion_from_stream(&self, model: &str, prompt: &str) -> Result<String> {
        let mut stream = self.ai_manager.stream_completion(model, prompt).await?;
        let mut full_response = String::new();
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(response) => {
                    full_response.push_str(&response.content);
                    if response.is_final {
                        break;
                    }
                }
                Err(e) => {
                    error!("Stream error: {}", e);
                    return Err(e);
                }
            }
        }
        
        Ok(full_response)
    }
    
    /// Stream completion with rate limiting (no caching for streams)
    pub async fn stream_completion(
        &self,
        model: &str,
        prompt: &str,
    ) -> Result<StreamingResponseStream> {
        // Get user context
        let context = UserContext::current();
        let user_id = context.user_id();
        
        // Check rate limit with user context
        if !self.rate_limiter.check_model_limit(model, &user_id).await? {
            return Err(anyhow::anyhow!("Rate limit exceeded for model: {} (user: {})", model, user_id));
        }
        
        // Use circuit breaker
        self.circuit_breaker.call(async {
            self.ai_manager.stream_completion(model, prompt).await
        }).await
    }
    
    /// Get completion with system prompt
    pub async fn get_completion_with_context(
        &self,
        model: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<String> {
        // Build combined prompt for caching
        let full_prompt = if let Some(system) = system_prompt {
            format!("System: {}\n\nUser: {}", system, prompt)
        } else {
            prompt.to_string()
        };
        
        self.get_completion(model, &full_prompt).await
    }
    
    /// Stream completion with system prompt
    pub async fn stream_completion_with_context(
        &self,
        model: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<StreamingResponseStream> {
        // Check rate limit
        let user_id = "default";
        if !self.rate_limiter.check_model_limit(model, user_id).await? {
            return Err(anyhow::anyhow!("Rate limit exceeded for model: {}", model));
        }
        
        self.circuit_breaker.call(async {
            self.ai_manager.stream_completion_with_context(model, prompt, system_prompt).await
        }).await
    }
    
    /// Clear AI response cache
    pub async fn clear_cache(&self) -> Result<()> {
        if let Some(cache) = &self.cache {
            cache.cache.clear().await?;
            debug!("Cleared AI response cache");
        }
        Ok(())
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Option<CacheStats> {
        // TODO: Implement cache statistics
        None
    }
    
    /// Delegate other methods to inner AI manager
    pub async fn test_connection(&mut self, name: &str) -> Result<crate::ai::TestResult> {
        self.ai_manager.test_connection(name).await
    }
    
    pub async fn test_all_connections(&mut self) -> Result<std::collections::HashMap<String, crate::ai::TestResult>> {
        self.ai_manager.test_all_connections().await
    }
    
    pub async fn list_models(&self) -> Result<Vec<(String, crate::config::AiModelConfig)>> {
        self.ai_manager.list_models().await
    }
    
    pub fn get_default_model(&self) -> Option<String> {
        self.ai_manager.get_default_model()
    }
    
    pub async fn handle_command(&mut self, command: crate::shell_commands::AiCommands) -> Result<()> {
        self.ai_manager.handle_command(command).await
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: u64,
    pub memory_usage_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rate_limiting() {
        // Test rate limiting behavior
        // TODO: Implement tests
    }
    
    #[tokio::test]
    async fn test_caching() {
        // Test caching behavior
        // TODO: Implement tests
    }
}