//! Performance monitoring for Alchemist

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};
use crate::cpu_monitor::CpuMonitor;
use crate::connection_tracker::{global_tracker, ConnectionStats};

/// Performance metrics collector
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<MetricsStore>>,
    config: MonitorConfig,
    cpu_monitor: Arc<RwLock<CpuMonitor>>,
}

#[derive(Clone)]
pub struct MonitorConfig {
    /// Maximum number of data points to keep
    pub max_history: usize,
    /// Sampling interval
    pub sample_interval: Duration,
    /// Enable detailed tracing
    pub detailed_tracing: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            max_history: 1000,
            sample_interval: Duration::from_secs(1),
            detailed_tracing: false,
        }
    }
}

#[derive(Default)]
struct MetricsStore {
    /// Request latencies by endpoint
    latencies: HashMap<String, VecDeque<LatencyRecord>>,
    /// Cache hit rates
    cache_hits: VecDeque<CacheRecord>,
    /// Rate limit events
    rate_limits: VecDeque<RateLimitRecord>,
    /// System resource usage
    system_resources: VecDeque<ResourceRecord>,
    /// AI model performance
    model_metrics: HashMap<String, ModelMetrics>,
}

#[derive(Clone, Serialize, Deserialize)]
struct LatencyRecord {
    timestamp: i64,
    duration_ms: f64,
    success: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct CacheRecord {
    timestamp: i64,
    hits: u64,
    misses: u64,
    evictions: u64,
}

#[derive(Clone, Serialize, Deserialize)]
struct RateLimitRecord {
    timestamp: i64,
    key: String,
    allowed: bool,
    tokens_remaining: f64,
}

#[derive(Clone, Serialize, Deserialize)]
struct ResourceRecord {
    timestamp: i64,
    memory_mb: f32,
    cpu_percent: f32,
    open_connections: u32,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct ModelMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_tokens: u64,
    average_latency_ms: f64,
    p95_latency_ms: f64,
    p99_latency_ms: f64,
}

impl PerformanceMonitor {
    pub fn new(config: MonitorConfig) -> Self {
        let cpu_monitor = Arc::new(RwLock::new(CpuMonitor::new()));
        
        let monitor = Self {
            metrics: Arc::new(RwLock::new(MetricsStore::default())),
            config,
            cpu_monitor: cpu_monitor.clone(),
        };
        
        // Start background metrics collection
        let metrics = monitor.metrics.clone();
        let interval = monitor.config.sample_interval;
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                
                // Get CPU usage
                let cpu_usage = {
                    let mut cpu_mon = cpu_monitor.write().await;
                    cpu_mon.get_usage()
                };
                
                // Get connection stats
                let conn_stats = global_tracker().get_connection_count().await;
                
                // Collect system metrics
                let resources = ResourceRecord {
                    timestamp: chrono::Utc::now().timestamp(),
                    memory_mb: crate::system_monitor::get_memory_usage_mb(),
                    cpu_percent: cpu_usage,
                    open_connections: conn_stats.total_active(),
                };
                
                let mut store = metrics.write().await;
                store.system_resources.push_back(resources);
                
                // Trim old records
                while store.system_resources.len() > 1000 {
                    store.system_resources.pop_front();
                }
            }
        });
        
        monitor
    }
    
    /// Record a request latency
    pub async fn record_latency(
        &self,
        endpoint: &str,
        duration: Duration,
        success: bool,
    ) {
        let record = LatencyRecord {
            timestamp: chrono::Utc::now().timestamp(),
            duration_ms: duration.as_secs_f64() * 1000.0,
            success,
        };
        
        let mut metrics = self.metrics.write().await;
        let latencies = metrics.latencies
            .entry(endpoint.to_string())
            .or_insert_with(VecDeque::new);
        
        latencies.push_back(record);
        
        // Keep only recent records
        while latencies.len() > self.config.max_history {
            latencies.pop_front();
        }
        
        if self.config.detailed_tracing {
            debug!(
                "Recorded latency for {}: {:.2}ms (success: {})",
                endpoint, duration.as_secs_f64() * 1000.0, success
            );
        }
    }
    
    /// Record cache activity
    pub async fn record_cache_hit(&self, hit: bool) {
        let mut metrics = self.metrics.write().await;
        
        if let Some(last) = metrics.cache_hits.back_mut() {
            if chrono::Utc::now().timestamp() - last.timestamp < 60 {
                if hit {
                    last.hits += 1;
                } else {
                    last.misses += 1;
                }
                return;
            }
        }
        
        // Create new record
        let mut record = CacheRecord {
            timestamp: chrono::Utc::now().timestamp(),
            hits: 0,
            misses: 0,
            evictions: 0,
        };
        
        if hit {
            record.hits = 1;
        } else {
            record.misses = 1;
        }
        
        metrics.cache_hits.push_back(record);
        
        // Keep only recent records
        while metrics.cache_hits.len() > self.config.max_history {
            metrics.cache_hits.pop_front();
        }
    }
    
    /// Record rate limit event
    pub async fn record_rate_limit(
        &self,
        key: &str,
        allowed: bool,
        tokens_remaining: f64,
    ) {
        let record = RateLimitRecord {
            timestamp: chrono::Utc::now().timestamp(),
            key: key.to_string(),
            allowed,
            tokens_remaining,
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.rate_limits.push_back(record);
        
        // Keep only recent records
        while metrics.rate_limits.len() > self.config.max_history {
            metrics.rate_limits.pop_front();
        }
        
        if !allowed {
            warn!("Rate limit exceeded for key: {} (tokens: {:.2})", key, tokens_remaining);
        }
    }
    
    /// Record AI model performance
    pub async fn record_model_request(
        &self,
        model: &str,
        duration: Duration,
        success: bool,
        tokens: Option<u64>,
    ) {
        let mut metrics = self.metrics.write().await;
        let model_metrics = metrics.model_metrics
            .entry(model.to_string())
            .or_default();
        
        model_metrics.total_requests += 1;
        
        if success {
            model_metrics.successful_requests += 1;
        } else {
            model_metrics.failed_requests += 1;
        }
        
        if let Some(token_count) = tokens {
            model_metrics.total_tokens += token_count;
        }
        
        // Update latency metrics
        let latency_ms = duration.as_secs_f64() * 1000.0;
        
        // Simple moving average for now
        let weight = 0.1;
        model_metrics.average_latency_ms = 
            model_metrics.average_latency_ms * (1.0 - weight) + latency_ms * weight;
        
        // Update percentiles (simplified)
        model_metrics.p95_latency_ms = model_metrics.p95_latency_ms.max(latency_ms * 0.95);
        model_metrics.p99_latency_ms = model_metrics.p99_latency_ms.max(latency_ms * 0.99);
    }
    
    /// Get current performance summary
    pub async fn get_summary(&self) -> PerformanceSummary {
        let metrics = self.metrics.read().await;
        
        // Calculate cache hit rate
        let cache_hit_rate = if let Some(window) = metrics.cache_hits.iter().rev().take(100).collect::<Vec<_>>().as_slice() {
            let total_hits: u64 = window.iter().map(|r| r.hits).sum();
            let total_misses: u64 = window.iter().map(|r| r.misses).sum();
            let total = total_hits + total_misses;
            if total > 0 {
                (total_hits as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        // Get latest resource usage
        let latest_resources = metrics.system_resources.back().cloned();
        
        // Count rate limit violations
        let rate_limit_violations = metrics.rate_limits.iter()
            .rev()
            .take(100)
            .filter(|r| !r.allowed)
            .count() as u64;
        
        PerformanceSummary {
            cache_hit_rate,
            rate_limit_violations,
            latest_resources,
            model_metrics: metrics.model_metrics.clone(),
        }
    }
    
    /// Export metrics for analysis
    pub async fn export_metrics(&self, format: ExportFormat) -> Result<String> {
        let metrics = self.metrics.read().await;
        
        match format {
            ExportFormat::Json => {
                let export = MetricsExport {
                    timestamp: chrono::Utc::now().timestamp(),
                    latencies: metrics.latencies.clone(),
                    cache_hits: metrics.cache_hits.clone().into(),
                    rate_limits: metrics.rate_limits.clone().into(),
                    system_resources: metrics.system_resources.clone().into(),
                    model_metrics: metrics.model_metrics.clone(),
                };
                
                Ok(serde_json::to_string_pretty(&export)?)
            }
            ExportFormat::Csv => {
                self.export_csv_format(&metrics).await
            }
        }
    }
    
    /// Export metrics in CSV format
    async fn export_csv_format(&self, metrics: &MetricsStore) -> Result<String> {
        let mut csv_output = String::new();
        
        // Export latency data
        csv_output.push_str("=== LATENCY METRICS ===\n");
        csv_output.push_str("Endpoint,Timestamp,Duration_ms,Success\n");
        
        for (endpoint, records) in &metrics.latencies {
            for record in records {
                csv_output.push_str(&format!(
                    "{},{},{:.2},{}\n",
                    endpoint,
                    record.timestamp,
                    record.duration_ms,
                    record.success
                ));
            }
        }
        
        // Export cache metrics
        csv_output.push_str("\n=== CACHE METRICS ===\n");
        csv_output.push_str("Timestamp,Hits,Misses,Evictions,Hit_Rate\n");
        
        for record in &metrics.cache_hits {
            let total = record.hits + record.misses;
            let hit_rate = if total > 0 {
                (record.hits as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            
            csv_output.push_str(&format!(
                "{},{},{},{},{:.2}\n",
                record.timestamp,
                record.hits,
                record.misses,
                record.evictions,
                hit_rate
            ));
        }
        
        // Export rate limit events
        csv_output.push_str("\n=== RATE LIMIT EVENTS ===\n");
        csv_output.push_str("Timestamp,Key,Allowed,Tokens_Remaining\n");
        
        for record in &metrics.rate_limits {
            csv_output.push_str(&format!(
                "{},{},{},{:.2}\n",
                record.timestamp,
                record.key,
                record.allowed,
                record.tokens_remaining
            ));
        }
        
        // Export system resources
        csv_output.push_str("\n=== SYSTEM RESOURCES ===\n");
        csv_output.push_str("Timestamp,Memory_MB,CPU_Percent,Open_Connections\n");
        
        for record in &metrics.system_resources {
            csv_output.push_str(&format!(
                "{},{:.2},{:.2},{}\n",
                record.timestamp,
                record.memory_mb,
                record.cpu_percent,
                record.open_connections
            ));
        }
        
        // Export model metrics
        csv_output.push_str("\n=== MODEL PERFORMANCE ===\n");
        csv_output.push_str("Model,Total_Requests,Successful,Failed,Total_Tokens,Avg_Latency_ms,P95_Latency_ms,P99_Latency_ms\n");
        
        for (model, metrics) in &metrics.model_metrics {
            csv_output.push_str(&format!(
                "{},{},{},{},{},{:.2},{:.2},{:.2}\n",
                model,
                metrics.total_requests,
                metrics.successful_requests,
                metrics.failed_requests,
                metrics.total_tokens,
                metrics.average_latency_ms,
                metrics.p95_latency_ms,
                metrics.p99_latency_ms
            ));
        }
        
        Ok(csv_output)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub cache_hit_rate: f64,
    pub rate_limit_violations: u64,
    pub latest_resources: Option<ResourceRecord>,
    pub model_metrics: HashMap<String, ModelMetrics>,
}

#[derive(Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
}

#[derive(Serialize, Deserialize)]
struct MetricsExport {
    timestamp: i64,
    latencies: HashMap<String, VecDeque<LatencyRecord>>,
    cache_hits: Vec<CacheRecord>,
    rate_limits: Vec<RateLimitRecord>,
    system_resources: Vec<ResourceRecord>,
    model_metrics: HashMap<String, ModelMetrics>,
}

/// Performance monitoring middleware
pub async fn with_performance_tracking<F, T>(
    monitor: &PerformanceMonitor,
    endpoint: &str,
    f: F,
) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    let start = Instant::now();
    
    match f.await {
        Ok(result) => {
            let duration = start.elapsed();
            monitor.record_latency(endpoint, duration, true).await;
            Ok(result)
        }
        Err(e) => {
            let duration = start.elapsed();
            monitor.record_latency(endpoint, duration, false).await;
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new(MonitorConfig::default());
        
        // Record some metrics
        monitor.record_latency("test_endpoint", Duration::from_millis(100), true).await;
        monitor.record_cache_hit(true).await;
        monitor.record_cache_hit(false).await;
        monitor.record_rate_limit("test_user", true, 50.0).await;
        
        // Get summary
        let summary = monitor.get_summary().await;
        assert!(summary.cache_hit_rate >= 0.0);
        assert_eq!(summary.rate_limit_violations, 0);
    }
}