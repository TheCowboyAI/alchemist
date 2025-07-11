//! Rate limiting for API protection

use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use std::collections::HashMap;
use tracing::{debug, warn};
use crate::user_context::{UserContext, UserTier};

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    config: RateLimiterConfig,
}

#[derive(Clone, Debug)]
pub struct RateLimiterConfig {
    /// Maximum tokens in bucket
    pub capacity: u32,
    /// Tokens refilled per second
    pub refill_rate: f64,
    /// Time window for rate limiting
    pub window: Duration,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            capacity: 100,
            refill_rate: 10.0,
            window: Duration::from_secs(60),
        }
    }
}

#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    capacity: f64,
    refill_rate: f64,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
            capacity,
            refill_rate,
        }
    }
    
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = elapsed.as_secs_f64() * self.refill_rate;
        
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }
    
    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }
    
    fn tokens_available(&self) -> f64 {
        self.tokens
    }
}

impl RateLimiter {
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }
    
    /// Check if request is allowed
    pub async fn check_rate_limit(&self, key: &str, tokens: f64) -> Result<bool> {
        let mut buckets = self.buckets.lock().await;
        
        let bucket = buckets.entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(
                self.config.capacity as f64,
                self.config.refill_rate,
            ));
        
        let allowed = bucket.try_consume(tokens);
        
        if !allowed {
            debug!(
                "Rate limit exceeded for key: {} (available: {:.2})",
                key,
                bucket.tokens_available()
            );
        }
        
        Ok(allowed)
    }
    
    /// Get remaining tokens
    pub async fn get_remaining_tokens(&self, key: &str) -> f64 {
        let mut buckets = self.buckets.lock().await;
        
        if let Some(bucket) = buckets.get_mut(key) {
            bucket.refill();
            bucket.tokens_available()
        } else {
            self.config.capacity as f64
        }
    }
    
    /// Reset rate limit for a key
    pub async fn reset(&self, key: &str) {
        let mut buckets = self.buckets.lock().await;
        buckets.remove(key);
    }
    
    /// Clean up old buckets
    pub async fn cleanup(&self) {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();
        
        buckets.retain(|_, bucket| {
            now.duration_since(bucket.last_refill) < self.config.window
        });
    }
}

/// Semaphore-based concurrency limiter
pub struct ConcurrencyLimiter {
    semaphore: Arc<Semaphore>,
    name: String,
}

impl ConcurrencyLimiter {
    pub fn new(name: String, max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            name,
        }
    }
    
    /// Acquire a permit
    pub async fn acquire(&self) -> Result<SemaphorePermit> {
        match self.semaphore.clone().acquire_owned().await {
            Ok(permit) => {
                debug!("Acquired concurrency permit for {}", self.name);
                Ok(SemaphorePermit { permit })
            }
            Err(_) => Err(anyhow::anyhow!("Failed to acquire semaphore permit")),
        }
    }
    
    /// Get available permits
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

pub struct SemaphorePermit {
    permit: tokio::sync::OwnedSemaphorePermit,
}

/// Hierarchical rate limiter for different tiers
pub struct TieredRateLimiter {
    limiters: HashMap<String, RateLimiter>,
}

impl TieredRateLimiter {
    pub fn new() -> Self {
        let mut limiters = HashMap::new();
        
        // Free tier
        limiters.insert("free".to_string(), RateLimiter::new(RateLimiterConfig {
            capacity: 100,
            refill_rate: 1.0,
            window: Duration::from_secs(3600),
        }));
        
        // Pro tier
        limiters.insert("pro".to_string(), RateLimiter::new(RateLimiterConfig {
            capacity: 1000,
            refill_rate: 10.0,
            window: Duration::from_secs(3600),
        }));
        
        // Enterprise tier
        limiters.insert("enterprise".to_string(), RateLimiter::new(RateLimiterConfig {
            capacity: 10000,
            refill_rate: 100.0,
            window: Duration::from_secs(3600),
        }));
        
        Self { limiters }
    }
    
    pub async fn check_rate_limit(
        &self,
        tier: &str,
        key: &str,
        tokens: f64,
    ) -> Result<bool> {
        let limiter = self.limiters.get(tier)
            .ok_or_else(|| anyhow::anyhow!("Unknown tier: {}", tier))?;
        
        limiter.check_rate_limit(key, tokens).await
    }
}

/// Rate limiting middleware for AI requests
pub struct AiRateLimiter {
    model_limits: HashMap<String, RateLimiterConfig>,
    limiters: Arc<Mutex<HashMap<String, RateLimiter>>>,
}

impl AiRateLimiter {
    pub fn new() -> Self {
        let mut model_limits = HashMap::new();
        
        // Claude models
        model_limits.insert("claude-3-sonnet".to_string(), RateLimiterConfig {
            capacity: 100,
            refill_rate: 2.0,
            window: Duration::from_secs(60),
        });
        
        model_limits.insert("claude-3-opus".to_string(), RateLimiterConfig {
            capacity: 50,
            refill_rate: 1.0,
            window: Duration::from_secs(60),
        });
        
        // GPT models
        model_limits.insert("gpt-4".to_string(), RateLimiterConfig {
            capacity: 60,
            refill_rate: 1.0,
            window: Duration::from_secs(60),
        });
        
        model_limits.insert("gpt-3.5-turbo".to_string(), RateLimiterConfig {
            capacity: 200,
            refill_rate: 3.33,
            window: Duration::from_secs(60),
        });
        
        Self {
            model_limits,
            limiters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn check_model_limit(
        &self,
        model: &str,
        user_id: &str,
    ) -> Result<bool> {
        let base_config = self.model_limits.get(model)
            .ok_or_else(|| anyhow::anyhow!("Unknown model: {}", model))?;
        
        // Get user context and apply tier multiplier
        let context = UserContext::current();
        let tier_multiplier = context.tier().rate_limit_multiplier();
        
        // Adjust config based on user tier
        let adjusted_config = RateLimiterConfig {
            capacity: (base_config.capacity as f64 * tier_multiplier) as u32,
            refill_rate: base_config.refill_rate * tier_multiplier,
            window: base_config.window,
        };
        
        let mut limiters = self.limiters.lock().await;
        let key = format!("{}:{}", model, user_id);
        
        let limiter = limiters.entry(key.clone())
            .or_insert_with(|| RateLimiter::new(adjusted_config));
        
        limiter.check_rate_limit(&key, 1.0).await
    }
}

/// Circuit breaker for fault tolerance
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    config: CircuitBreakerConfig,
}

#[derive(Debug, Clone)]
enum CircuitState {
    Closed,
    Open(Instant),
    HalfOpen,
}

#[derive(Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
        }
    }
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            config,
        }
    }
    
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        let mut state = self.state.lock().await;
        
        match &*state {
            CircuitState::Open(since) => {
                if since.elapsed() >= self.config.timeout {
                    *state = CircuitState::HalfOpen;
                } else {
                    return Err(anyhow::anyhow!("Circuit breaker is open"));
                }
            }
            _ => {}
        }
        
        drop(state);
        
        match f.await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(e)
            }
        }
    }
    
    async fn record_success(&self) {
        let mut state = self.state.lock().await;
        if let CircuitState::HalfOpen = *state {
            *state = CircuitState::Closed;
            debug!("Circuit breaker closed");
        }
    }
    
    async fn record_failure(&self) {
        let mut state = self.state.lock().await;
        match *state {
            CircuitState::Closed => {
                *state = CircuitState::Open(Instant::now());
                warn!("Circuit breaker opened");
            }
            CircuitState::HalfOpen => {
                *state = CircuitState::Open(Instant::now());
                warn!("Circuit breaker reopened");
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            capacity: 10,
            refill_rate: 1.0,
            window: Duration::from_secs(60),
        });
        
        // Consume tokens
        for i in 0..10 {
            assert!(limiter.check_rate_limit("test", 1.0).await.unwrap());
        }
        
        // Should be rate limited
        assert!(!limiter.check_rate_limit("test", 1.0).await.unwrap());
        
        // Wait for refill
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Should have ~2 tokens now
        assert!(limiter.check_rate_limit("test", 1.0).await.unwrap());
        assert!(limiter.check_rate_limit("test", 1.0).await.unwrap());
        assert!(!limiter.check_rate_limit("test", 1.0).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_concurrency_limiter() {
        let limiter = ConcurrencyLimiter::new("test".to_string(), 2);
        
        // Acquire permits
        let _permit1 = limiter.acquire().await.unwrap();
        let _permit2 = limiter.acquire().await.unwrap();
        
        // Should have no permits left
        assert_eq!(limiter.available_permits(), 0);
        
        // Drop a permit
        drop(_permit1);
        
        // Should have 1 permit available
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(limiter.available_permits(), 1);
    }
}