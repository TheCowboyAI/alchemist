//! Benchmarking tools for Alchemist

use anyhow::Result;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::mpsc;
use futures::StreamExt;
use tracing::{info, warn};

use crate::{
    ai::AiManager,
    ai_enhanced::EnhancedAiManager,
    cache::CacheManager,
    performance_monitor::PerformanceMonitor,
};

/// Benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub test_name: String,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub total_duration: Duration,
    pub average_latency: Duration,
    pub min_latency: Duration,
    pub max_latency: Duration,
    pub p50_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    pub throughput_rps: f64,
    pub cache_hit_rate: Option<f64>,
    pub errors: Vec<String>,
}

/// Benchmark configuration
#[derive(Clone)]
pub struct BenchmarkConfig {
    pub name: String,
    pub requests: usize,
    pub concurrent_workers: usize,
    pub warmup_requests: usize,
    pub test_prompt: String,
    pub models: Vec<String>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            name: "AI Response Benchmark".to_string(),
            requests: 100,
            concurrent_workers: 4,
            warmup_requests: 5,
            test_prompt: "What is the capital of France?".to_string(),
            models: vec!["claude-3-sonnet".to_string()],
        }
    }
}

/// Benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    performance_monitor: Option<PerformanceMonitor>,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            performance_monitor: None,
        }
    }
    
    pub fn with_monitor(mut self, monitor: PerformanceMonitor) -> Self {
        self.performance_monitor = Some(monitor);
        self
    }
    
    /// Run benchmark comparing standard vs enhanced AI manager
    pub async fn compare_ai_managers(
        &self,
        standard: &mut AiManager,
        enhanced: &EnhancedAiManager,
    ) -> Result<HashMap<String, BenchmarkResults>> {
        let mut results = HashMap::new();
        
        for model in &self.config.models {
            info!("Benchmarking model: {}", model);
            
            // Test standard AI manager
            let standard_results = self.benchmark_ai_manager(
                &format!("{} (Standard)", model),
                model,
                standard,
                false,
            ).await?;
            
            results.insert(format!("{}_standard", model), standard_results);
            
            // Test enhanced AI manager with caching
            let enhanced_results = self.benchmark_enhanced_ai_manager(
                &format!("{} (Enhanced)", model),
                model,
                enhanced,
            ).await?;
            
            results.insert(format!("{}_enhanced", model), enhanced_results);
        }
        
        Ok(results)
    }
    
    /// Benchmark standard AI manager
    async fn benchmark_ai_manager(
        &self,
        test_name: &str,
        model: &str,
        ai_manager: &AiManager,
        _use_cache: bool,
    ) -> Result<BenchmarkResults> {
        info!("Starting benchmark: {}", test_name);
        
        // Warmup
        info!("Warming up with {} requests...", self.config.warmup_requests);
        for _ in 0..self.config.warmup_requests {
            let _ = ai_manager.get_completion(model, &self.config.test_prompt).await;
        }
        
        let start_time = Instant::now();
        let mut latencies = Vec::new();
        let mut errors = Vec::new();
        let mut successful = 0;
        let mut failed = 0;
        
        // Create work channel
        let (tx, mut rx) = mpsc::channel(self.config.concurrent_workers);
        
        // Spawn workers
        let workers = (0..self.config.concurrent_workers).map(|_| {
            let tx = tx.clone();
            let prompt = self.config.test_prompt.clone();
            let model = model.to_string();
            
            tokio::spawn(async move {
                for i in 0..self.config.requests / self.config.concurrent_workers {
                    let request_start = Instant::now();
                    
                    match ai_manager.get_completion(&model, &prompt).await {
                        Ok(_) => {
                            let latency = request_start.elapsed();
                            let _ = tx.send(Ok(latency)).await;
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e.to_string())).await;
                        }
                    }
                }
            })
        }).collect::<Vec<_>>();
        
        drop(tx);
        
        // Collect results
        while let Some(result) = rx.recv().await {
            match result {
                Ok(latency) => {
                    latencies.push(latency);
                    successful += 1;
                    
                    if let Some(monitor) = &self.performance_monitor {
                        monitor.record_model_request(model, latency, true, None).await;
                    }
                }
                Err(e) => {
                    errors.push(e);
                    failed += 1;
                    
                    if let Some(monitor) = &self.performance_monitor {
                        monitor.record_model_request(model, Duration::from_secs(0), false, None).await;
                    }
                }
            }
        }
        
        // Wait for all workers
        for worker in workers {
            let _ = worker.await;
        }
        
        let total_duration = start_time.elapsed();
        
        // Calculate statistics
        latencies.sort();
        
        let results = BenchmarkResults {
            test_name: test_name.to_string(),
            total_requests: successful + failed,
            successful_requests: successful,
            failed_requests: failed,
            total_duration,
            average_latency: if !latencies.is_empty() {
                Duration::from_nanos(
                    latencies.iter().map(|d| d.as_nanos()).sum::<u128>() as u64 / latencies.len() as u64
                )
            } else {
                Duration::from_secs(0)
            },
            min_latency: latencies.first().copied().unwrap_or_default(),
            max_latency: latencies.last().copied().unwrap_or_default(),
            p50_latency: latencies.get(latencies.len() / 2).copied().unwrap_or_default(),
            p95_latency: latencies.get(latencies.len() * 95 / 100).copied().unwrap_or_default(),
            p99_latency: latencies.get(latencies.len() * 99 / 100).copied().unwrap_or_default(),
            throughput_rps: successful as f64 / total_duration.as_secs_f64(),
            cache_hit_rate: None,
            errors,
        };
        
        Ok(results)
    }
    
    /// Benchmark enhanced AI manager
    async fn benchmark_enhanced_ai_manager(
        &self,
        test_name: &str,
        model: &str,
        ai_manager: &EnhancedAiManager,
    ) -> Result<BenchmarkResults> {
        info!("Starting enhanced benchmark: {}", test_name);
        
        // Clear cache before benchmark
        ai_manager.clear_cache().await?;
        
        // Warmup
        info!("Warming up with {} requests...", self.config.warmup_requests);
        for _ in 0..self.config.warmup_requests {
            let _ = ai_manager.get_completion(model, &self.config.test_prompt).await;
        }
        
        let start_time = Instant::now();
        let mut latencies = Vec::new();
        let mut errors = Vec::new();
        let mut successful = 0;
        let mut failed = 0;
        
        // Run requests sequentially to better measure cache effectiveness
        for i in 0..self.config.requests {
            let request_start = Instant::now();
            
            // Vary the prompt slightly to test cache
            let prompt = if i % 10 == 0 {
                &self.config.test_prompt
            } else {
                &self.config.test_prompt // Use same prompt to test cache hits
            };
            
            match ai_manager.get_completion(model, prompt).await {
                Ok(_) => {
                    let latency = request_start.elapsed();
                    latencies.push(latency);
                    successful += 1;
                    
                    if let Some(monitor) = &self.performance_monitor {
                        monitor.record_model_request(model, latency, true, None).await;
                    }
                }
                Err(e) => {
                    errors.push(e.to_string());
                    failed += 1;
                    
                    if let Some(monitor) = &self.performance_monitor {
                        monitor.record_model_request(model, Duration::from_secs(0), false, None).await;
                    }
                }
            }
        }
        
        let total_duration = start_time.elapsed();
        
        // Calculate statistics
        latencies.sort();
        
        // Get cache hit rate if available
        let cache_hit_rate = if let Some(monitor) = &self.performance_monitor {
            let summary = monitor.get_summary().await;
            Some(summary.cache_hit_rate)
        } else {
            None
        };
        
        let results = BenchmarkResults {
            test_name: test_name.to_string(),
            total_requests: successful + failed,
            successful_requests: successful,
            failed_requests: failed,
            total_duration,
            average_latency: if !latencies.is_empty() {
                Duration::from_nanos(
                    latencies.iter().map(|d| d.as_nanos()).sum::<u128>() as u64 / latencies.len() as u64
                )
            } else {
                Duration::from_secs(0)
            },
            min_latency: latencies.first().copied().unwrap_or_default(),
            max_latency: latencies.last().copied().unwrap_or_default(),
            p50_latency: latencies.get(latencies.len() / 2).copied().unwrap_or_default(),
            p95_latency: latencies.get(latencies.len() * 95 / 100).copied().unwrap_or_default(),
            p99_latency: latencies.get(latencies.len() * 99 / 100).copied().unwrap_or_default(),
            throughput_rps: successful as f64 / total_duration.as_secs_f64(),
            cache_hit_rate,
            errors,
        };
        
        Ok(results)
    }
    
    /// Print benchmark results
    pub fn print_results(results: &HashMap<String, BenchmarkResults>) {
        println!("\n📊 Benchmark Results\n");
        
        for (key, result) in results {
            println!("Test: {}", result.test_name);
            println!("─".repeat(50));
            println!("Total Requests: {}", result.total_requests);
            println!("Successful: {} ({:.1}%)", 
                result.successful_requests,
                result.successful_requests as f64 / result.total_requests as f64 * 100.0
            );
            println!("Failed: {}", result.failed_requests);
            println!("Total Duration: {:.2}s", result.total_duration.as_secs_f64());
            println!("Throughput: {:.2} req/s", result.throughput_rps);
            println!("\nLatency Statistics:");
            println!("  Average: {:.2}ms", result.average_latency.as_secs_f64() * 1000.0);
            println!("  Min: {:.2}ms", result.min_latency.as_secs_f64() * 1000.0);
            println!("  Max: {:.2}ms", result.max_latency.as_secs_f64() * 1000.0);
            println!("  P50: {:.2}ms", result.p50_latency.as_secs_f64() * 1000.0);
            println!("  P95: {:.2}ms", result.p95_latency.as_secs_f64() * 1000.0);
            println!("  P99: {:.2}ms", result.p99_latency.as_secs_f64() * 1000.0);
            
            if let Some(cache_hit_rate) = result.cache_hit_rate {
                println!("Cache Hit Rate: {:.1}%", cache_hit_rate);
            }
            
            if !result.errors.is_empty() {
                println!("\nErrors ({}):", result.errors.len());
                for (i, error) in result.errors.iter().take(5).enumerate() {
                    println!("  {}: {}", i + 1, error);
                }
                if result.errors.len() > 5 {
                    println!("  ... and {} more", result.errors.len() - 5);
                }
            }
            
            println!();
        }
        
        // Compare standard vs enhanced if both present
        for model in ["claude-3-sonnet", "gpt-4", "gpt-3.5-turbo"] {
            let standard_key = format!("{}_standard", model);
            let enhanced_key = format!("{}_enhanced", model);
            
            if let (Some(standard), Some(enhanced)) = (results.get(&standard_key), results.get(&enhanced_key)) {
                println!("\n🔄 Comparison for {}", model);
                println!("─".repeat(50));
                
                let speedup = standard.average_latency.as_secs_f64() / enhanced.average_latency.as_secs_f64();
                let throughput_improvement = enhanced.throughput_rps / standard.throughput_rps;
                
                println!("Average Latency Improvement: {:.2}x faster", speedup);
                println!("Throughput Improvement: {:.2}x higher", throughput_improvement);
                
                if let Some(cache_hit_rate) = enhanced.cache_hit_rate {
                    println!("Cache Effectiveness: {:.1}% hit rate", cache_hit_rate);
                }
            }
        }
    }
}

/// Run a simple benchmark
pub async fn run_simple_benchmark(ai_manager: &mut AiManager) -> Result<()> {
    let config = BenchmarkConfig {
        name: "Quick Benchmark".to_string(),
        requests: 20,
        concurrent_workers: 2,
        warmup_requests: 2,
        test_prompt: "What is 2+2?".to_string(),
        models: vec!["claude-3-sonnet".to_string()],
    };
    
    let runner = BenchmarkRunner::new(config);
    let mut results = HashMap::new();
    
    let result = runner.benchmark_ai_manager(
        "Quick Test",
        "claude-3-sonnet",
        ai_manager,
        false,
    ).await?;
    
    results.insert("quick_test".to_string(), result);
    
    runner.print_results(&results);
    
    Ok(())
}