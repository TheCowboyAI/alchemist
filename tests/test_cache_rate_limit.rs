//! Test caching and rate limiting functionality

#[cfg(test)]
mod tests {
    use alchemist::{
        cache::{MemoryCache, CacheManager},
        rate_limiter::{RateLimiter, RateLimiterConfig, AiRateLimiter},
        ai_enhanced::EnhancedAiManager,
        config::AlchemistConfig,
    };
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_memory_cache_basic() {
        let cache = Arc::new(MemoryCache::new(100));
        
        // Test set and get
        cache.set("test_key", &"test_value", Duration::from_secs(60))
            .await
            .expect("Failed to set cache");
            
        let value: Option<String> = cache.get("test_key").await;
        assert_eq!(value, Some("test_value".to_string()));
        
        // Test expiration
        cache.set("expire_key", &"expire_value", Duration::from_millis(100))
            .await
            .expect("Failed to set cache");
            
        sleep(Duration::from_millis(200)).await;
        
        let expired: Option<String> = cache.get("expire_key").await;
        assert!(expired.is_none());
    }
    
    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let config = RateLimiterConfig {
            capacity: 5,
            refill_rate: 1.0,
            window: Duration::from_secs(60),
        };
        
        let limiter = RateLimiter::new(config);
        
        // Consume all tokens
        for i in 0..5 {
            let allowed = limiter.check_rate_limit("test_user", 1.0)
                .await
                .expect("Rate limit check failed");
            assert!(allowed, "Request {} should be allowed", i);
        }
        
        // Next request should be rate limited
        let limited = limiter.check_rate_limit("test_user", 1.0)
            .await
            .expect("Rate limit check failed");
        assert!(!limited, "Request should be rate limited");
        
        // Wait for token refill
        sleep(Duration::from_secs(2)).await;
        
        // Should have ~2 tokens now
        let allowed = limiter.check_rate_limit("test_user", 1.0)
            .await
            .expect("Rate limit check failed");
        assert!(allowed, "Request should be allowed after refill");
    }
    
    #[tokio::test]
    async fn test_ai_rate_limiter() {
        let limiter = AiRateLimiter::new();
        
        // Test model-specific rate limiting
        let model = "claude-3-sonnet";
        let user = "test_user";
        
        // Should allow initial requests
        for _ in 0..10 {
            let allowed = limiter.check_model_limit(model, user)
                .await
                .expect("Model limit check failed");
            assert!(allowed);
        }
        
        // Keep making requests until rate limited
        let mut limited = false;
        for _ in 0..200 {
            if !limiter.check_model_limit(model, user).await.unwrap() {
                limited = true;
                break;
            }
        }
        
        assert!(limited, "Should eventually be rate limited");
    }
    
    #[tokio::test]
    async fn test_cache_manager_get_or_compute() {
        let cache = Arc::new(MemoryCache::new(100));
        let manager = CacheManager::new(cache);
        
        let mut compute_count = 0;
        
        // First call should compute
        let value = manager.get_or_compute(
            "compute_key",
            Duration::from_secs(60),
            || {
                compute_count += 1;
                async { Ok("computed_value".to_string()) }
            }
        ).await.expect("Get or compute failed");
        
        assert_eq!(value, "computed_value");
        assert_eq!(compute_count, 1);
        
        // Second call should use cache
        let value2 = manager.get_or_compute(
            "compute_key",
            Duration::from_secs(60),
            || {
                compute_count += 1;
                async { Ok("should_not_compute".to_string()) }
            }
        ).await.expect("Get or compute failed");
        
        assert_eq!(value2, "computed_value");
        assert_eq!(compute_count, 1); // Should not have computed again
    }
    
    #[tokio::test]
    async fn test_enhanced_ai_manager_caching() {
        // This test would require mocking the AI responses
        // For now, just ensure it compiles
        let config = AlchemistConfig::default();
        let _manager = EnhancedAiManager::new(&config).await
            .expect("Failed to create enhanced AI manager");
    }
}