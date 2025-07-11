//! Integration test for performance features

#[cfg(test)]
mod tests {
    use alchemist::{
        config::AlchemistConfig,
        ai::AiManager,
        ai_enhanced::EnhancedAiManager,
        shell_integration::PerformanceManager,
        performance_monitor::{PerformanceMonitor, MonitorConfig},
        benchmarks::{BenchmarkRunner, BenchmarkConfig},
    };
    
    #[tokio::test]
    async fn test_performance_features_compile() {
        // This test primarily ensures all performance features compile correctly
        
        // Create config
        let config = AlchemistConfig::default();
        
        // Create performance manager
        let perf_manager = PerformanceManager::new(&config);
        assert!(perf_manager.cache.is_some());
        
        // Create performance monitor
        let monitor = PerformanceMonitor::new(MonitorConfig::default());
        
        // Create enhanced AI manager
        let enhanced_ai = EnhancedAiManager::new(&config)
            .await
            .expect("Failed to create enhanced AI manager");
        
        // Create benchmark runner
        let benchmark_config = BenchmarkConfig::default();
        let _runner = BenchmarkRunner::new(benchmark_config)
            .with_monitor(monitor);
        
        // Verify cache stats
        let stats = perf_manager.get_cache_stats().await;
        assert!(stats.enabled);
    }
}