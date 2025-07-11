//! Shell integration helpers for performance features

use anyhow::Result;
use std::sync::Arc;
use crate::{
    config::AlchemistConfig,
    cache::{LayeredCache, CacheManager},
    rate_limiter::{RateLimiter, RateLimiterConfig, AiRateLimiter},
    ai_enhanced::EnhancedAiManager,
};

/// Initialize performance features for the shell
pub struct PerformanceManager {
    pub cache: Option<CacheManager>,
    pub rate_limiter: Arc<RateLimiter>,
    pub ai_rate_limiter: Arc<AiRateLimiter>,
}

impl PerformanceManager {
    pub fn new(config: &AlchemistConfig) -> Self {
        // Initialize cache if configured
        let cache = if let Some(cache_config) = &config.cache {
            let redis_url = cache_config.redis_url.as_deref();
            let cache = LayeredCache::new(redis_url, "alchemist");
            Some(CacheManager::new(Arc::new(cache)))
        } else {
            None
        };
        
        // Initialize general rate limiter
        let rate_limiter = Arc::new(RateLimiter::new(RateLimiterConfig::default()));
        
        // Initialize AI-specific rate limiter
        let ai_rate_limiter = Arc::new(AiRateLimiter::new());
        
        Self {
            cache,
            rate_limiter,
            ai_rate_limiter,
        }
    }
    
    /// Check if caching is enabled
    pub fn caching_enabled(&self) -> bool {
        self.cache.is_some()
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let redis_available = if let Some(cache_config) = &self.cache.as_ref()
            .and_then(|_| std::env::var("REDIS_URL").ok().or_else(|| Some("redis://localhost:6379".to_string()))) {
            crate::redis_checker::check_redis_health(&cache_config).await.is_connected
        } else {
            false
        };
        
        CacheStats {
            enabled: self.caching_enabled(),
            redis_available,
            memory_cache_entries: 0, // Memory stats are internal to cache implementation
        }
    }
    
    /// Clear all caches
    pub async fn clear_caches(&self) -> Result<()> {
        if let Some(cache) = &self.cache {
            cache.cache.clear().await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub enabled: bool,
    pub redis_available: bool,
    pub memory_cache_entries: usize,
}

/// Shell extension trait for performance features
pub trait ShellPerformanceExt {
    /// Get performance manager
    fn perf(&self) -> &PerformanceManager;
    
    /// Execute with caching
    async fn with_cache<T, F, Fut>(
        &self,
        key: &str,
        ttl: std::time::Duration,
        f: F,
    ) -> Result<T>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Clone,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>;
}

/// Add performance monitoring to shell commands
pub mod commands {
    use super::*;
    
    /// Performance-related commands
    #[derive(Debug, clap::Subcommand)]
    pub enum PerfCommands {
        /// Show performance statistics
        Stats,
        
        /// Clear all caches
        ClearCache,
        
        /// Show rate limit status
        RateLimits {
            /// User ID to check
            #[arg(long, default_value = "default")]
            user: String,
        },
        
        /// Benchmark AI responses
        Benchmark {
            /// Model to benchmark
            model: String,
            
            /// Number of requests
            #[arg(short, long, default_value = "10")]
            count: usize,
        },
    }
    
    pub async fn handle_perf_command(
        perf: &PerformanceManager,
        command: PerfCommands,
    ) -> Result<()> {
        match command {
            PerfCommands::Stats => {
                let stats = perf.get_cache_stats().await;
                println!("🔍 Performance Statistics:");
                println!("  Cache enabled: {}", stats.enabled);
                println!("  Redis available: {}", stats.redis_available);
                println!("  Memory cache entries: {}", stats.memory_cache_entries);
            }
            
            PerfCommands::ClearCache => {
                perf.clear_caches().await?;
                println!("✅ All caches cleared");
            }
            
            PerfCommands::RateLimits { user } => {
                // TODO: Show rate limit status for user
                println!("📊 Rate limit status for user: {}", user);
            }
            
            PerfCommands::Benchmark { model, count } => {
                println!("🚀 Benchmarking {} with {} requests...", model, count);
                // TODO: Implement benchmarking
            }
        }
        
        Ok(())
    }
}