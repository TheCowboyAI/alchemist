//! Comprehensive health check system for Alchemist
//! 
//! Provides health monitoring for all subsystems with detailed diagnostics

use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn, error};
use std::collections::HashMap;

use crate::{
    redis_checker::{check_redis_health, RedisHealth},
    cpu_monitor::{CpuMonitor, get_load_average},
    connection_tracker::global_tracker,
    system_monitor::get_memory_usage_mb,
    performance_monitor::PerformanceMonitor,
    nats_client::NatsClient,
};

/// Overall system health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Some non-critical issues
    Degraded,
    /// Critical systems failing
    Unhealthy,
    /// Unable to determine status
    Unknown,
}

impl HealthStatus {
    pub fn as_emoji(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "🟢",
            HealthStatus::Degraded => "🟡",
            HealthStatus::Unhealthy => "🔴",
            HealthStatus::Unknown => "⚪",
        }
    }
}

/// Comprehensive health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub checks: HashMap<String, ComponentHealth>,
    pub diagnostics: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Individual component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub latency_ms: Option<f64>,
    pub details: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
}

/// Health check configuration
#[derive(Clone)]
pub struct HealthCheckConfig {
    pub enable_redis_check: bool,
    pub enable_nats_check: bool,
    pub enable_system_check: bool,
    pub enable_performance_check: bool,
    pub cpu_threshold_percent: f32,
    pub memory_threshold_mb: f32,
    pub load_threshold_per_core: f32,
    pub check_timeout: Duration,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enable_redis_check: true,
            enable_nats_check: true,
            enable_system_check: true,
            enable_performance_check: true,
            cpu_threshold_percent: 80.0,
            memory_threshold_mb: 1000.0,
            load_threshold_per_core: 2.0,
            check_timeout: Duration::from_secs(5),
        }
    }
}

/// Health check manager
pub struct HealthCheckManager {
    config: HealthCheckConfig,
    cpu_monitor: Arc<tokio::sync::Mutex<CpuMonitor>>,
    nats_client: Option<Arc<NatsClient>>,
    performance_monitor: Option<Arc<PerformanceMonitor>>,
}

impl HealthCheckManager {
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            config,
            cpu_monitor: Arc::new(tokio::sync::Mutex::new(CpuMonitor::new())),
            nats_client: None,
            performance_monitor: None,
        }
    }
    
    pub fn with_nats(mut self, client: Arc<NatsClient>) -> Self {
        self.nats_client = Some(client);
        self
    }
    
    pub fn with_performance_monitor(mut self, monitor: Arc<PerformanceMonitor>) -> Self {
        self.performance_monitor = Some(monitor);
        self
    }
    
    /// Run comprehensive health check
    pub async fn check_health(&self) -> HealthCheckResult {
        let start = Instant::now();
        let mut checks = HashMap::new();
        let mut diagnostics = Vec::new();
        let mut recommendations = Vec::new();
        
        // System health check
        if self.config.enable_system_check {
            let system_health = self.check_system_health().await;
            
            // Add diagnostics
            if let Some(cpu) = system_health.details.get("cpu_percent") {
                if let Some(cpu_val) = cpu.as_f64() {
                    if cpu_val > self.config.cpu_threshold_percent as f64 {
                        diagnostics.push(format!("High CPU usage: {:.1}%", cpu_val));
                        recommendations.push("Consider scaling up compute resources".to_string());
                    }
                }
            }
            
            checks.insert("system".to_string(), system_health);
        }
        
        // Redis health check
        if self.config.enable_redis_check {
            let redis_health = self.check_redis_health().await;
            
            if redis_health.status == HealthStatus::Unhealthy {
                diagnostics.push("Redis is not accessible".to_string());
                recommendations.push("Check Redis connection and configuration".to_string());
            }
            
            checks.insert("redis".to_string(), redis_health);
        }
        
        // NATS health check
        if self.config.enable_nats_check && self.nats_client.is_some() {
            let nats_health = self.check_nats_health().await;
            
            if nats_health.status != HealthStatus::Healthy {
                diagnostics.push("NATS connectivity issues detected".to_string());
                recommendations.push("Verify NATS server is running and accessible".to_string());
            }
            
            checks.insert("nats".to_string(), nats_health);
        }
        
        // Performance health check
        if self.config.enable_performance_check && self.performance_monitor.is_some() {
            let perf_health = self.check_performance_health().await;
            
            if let Some(cache_hit_rate) = perf_health.details.get("cache_hit_rate") {
                if let Some(rate) = cache_hit_rate.as_f64() {
                    if rate < 50.0 {
                        diagnostics.push(format!("Low cache hit rate: {:.1}%", rate));
                        recommendations.push("Review cache keys and TTL settings".to_string());
                    }
                }
            }
            
            checks.insert("performance".to_string(), perf_health);
        }
        
        // Connections health check
        let conn_health = self.check_connections_health().await;
        checks.insert("connections".to_string(), conn_health);
        
        // Determine overall status
        let overall_status = self.determine_overall_status(&checks);
        
        let duration = start.elapsed();
        diagnostics.push(format!("Health check completed in {:.2}ms", duration.as_secs_f64() * 1000.0));
        
        HealthCheckResult {
            status: overall_status,
            timestamp: chrono::Utc::now(),
            checks,
            diagnostics,
            recommendations,
        }
    }
    
    /// Check system resources health
    async fn check_system_health(&self) -> ComponentHealth {
        let start = Instant::now();
        let mut details = HashMap::new();
        let mut status = HealthStatus::Healthy;
        let mut error = None;
        
        // CPU usage
        let cpu_usage = {
            let mut monitor = self.cpu_monitor.lock().await;
            monitor.get_usage()
        };
        details.insert("cpu_percent".to_string(), cpu_usage.into());
        
        if cpu_usage > self.config.cpu_threshold_percent {
            status = HealthStatus::Degraded;
        }
        
        // Memory usage
        let memory_mb = get_memory_usage_mb();
        details.insert("memory_mb".to_string(), memory_mb.into());
        
        if memory_mb > self.config.memory_threshold_mb {
            status = HealthStatus::Degraded;
        }
        
        // Load average
        if let Some((load1, load5, load15)) = get_load_average() {
            details.insert("load_1min".to_string(), load1.into());
            details.insert("load_5min".to_string(), load5.into());
            details.insert("load_15min".to_string(), load15.into());
            
            let cores = CpuMonitor::get_core_count();
            let load_per_core = load1 / cores as f32;
            details.insert("load_per_core".to_string(), load_per_core.into());
            
            if load_per_core > self.config.load_threshold_per_core {
                status = HealthStatus::Degraded;
                error = Some(format!("High system load: {:.2} per core", load_per_core));
            }
        }
        
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        ComponentHealth {
            name: "System Resources".to_string(),
            status,
            latency_ms: Some(latency_ms),
            details,
            error,
        }
    }
    
    /// Check Redis health
    async fn check_redis_health(&self) -> ComponentHealth {
        let start = Instant::now();
        let mut details = HashMap::new();
        
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let health = check_redis_health(&redis_url).await;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        let status = if health.is_healthy() {
            HealthStatus::Healthy
        } else if health.is_connected {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };
        
        details.insert("connected".to_string(), health.is_connected.into());
        
        if let Some(redis_latency) = health.latency_ms {
            details.insert("redis_latency_ms".to_string(), redis_latency.into());
        }
        
        if let Some(version) = &health.version {
            details.insert("version".to_string(), version.clone().into());
        }
        
        if let Some(memory) = health.used_memory_mb {
            details.insert("memory_mb".to_string(), memory.into());
        }
        
        if let Some(clients) = health.connected_clients {
            details.insert("connected_clients".to_string(), clients.into());
        }
        
        ComponentHealth {
            name: "Redis Cache".to_string(),
            status,
            latency_ms: Some(latency_ms),
            details,
            error: health.error,
        }
    }
    
    /// Check NATS health
    async fn check_nats_health(&self) -> ComponentHealth {
        let start = Instant::now();
        let mut details = HashMap::new();
        let mut status = HealthStatus::Unknown;
        let mut error = None;
        
        if let Some(client) = &self.nats_client {
            // Simple ping test
            match tokio::time::timeout(
                Duration::from_secs(2),
                client.publish("_PING.test", b"ping").await
            ).await {
                Ok(Ok(_)) => {
                    status = HealthStatus::Healthy;
                    details.insert("ping_success".to_string(), true.into());
                }
                Ok(Err(e)) => {
                    status = HealthStatus::Unhealthy;
                    error = Some(format!("NATS publish failed: {}", e));
                }
                Err(_) => {
                    status = HealthStatus::Unhealthy;
                    error = Some("NATS ping timeout".to_string());
                }
            }
            
            // Get connection stats
            let stats = client.connection_stats();
            details.insert("messages_sent".to_string(), stats.messages_sent.into());
            details.insert("messages_received".to_string(), stats.messages_received.into());
            details.insert("reconnects".to_string(), stats.reconnects.into());
        }
        
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        ComponentHealth {
            name: "NATS Messaging".to_string(),
            status,
            latency_ms: Some(latency_ms),
            details,
            error,
        }
    }
    
    /// Check performance metrics health
    async fn check_performance_health(&self) -> ComponentHealth {
        let start = Instant::now();
        let mut details = HashMap::new();
        let mut status = HealthStatus::Healthy;
        let error = None;
        
        if let Some(monitor) = &self.performance_monitor {
            let summary = monitor.get_summary().await;
            
            details.insert("cache_hit_rate".to_string(), summary.cache_hit_rate.into());
            details.insert("rate_limit_violations".to_string(), summary.rate_limit_violations.into());
            
            if summary.cache_hit_rate < 50.0 {
                status = HealthStatus::Degraded;
            }
            
            if summary.rate_limit_violations > 100 {
                status = HealthStatus::Degraded;
            }
            
            if let Some(resources) = &summary.latest_resources {
                details.insert("latest_memory_mb".to_string(), resources.memory_mb.into());
                details.insert("latest_cpu_percent".to_string(), resources.cpu_percent.into());
            }
        }
        
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        ComponentHealth {
            name: "Performance Metrics".to_string(),
            status,
            latency_ms: Some(latency_ms),
            details,
            error,
        }
    }
    
    /// Check connections health
    async fn check_connections_health(&self) -> ComponentHealth {
        let start = Instant::now();
        let mut details = HashMap::new();
        let status = HealthStatus::Healthy;
        let error = None;
        
        let conn_stats = global_tracker().get_connection_count().await;
        
        details.insert("total_connections".to_string(), conn_stats.total.into());
        details.insert("active_connections".to_string(), conn_stats.active.into());
        details.insert("nats_connections".to_string(), conn_stats.nats.into());
        details.insert("redis_connections".to_string(), conn_stats.redis.into());
        details.insert("http_connections".to_string(), conn_stats.http.into());
        
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        ComponentHealth {
            name: "Connection Pool".to_string(),
            status,
            latency_ms: Some(latency_ms),
            details,
            error,
        }
    }
    
    /// Determine overall health status
    fn determine_overall_status(&self, checks: &HashMap<String, ComponentHealth>) -> HealthStatus {
        let mut has_unhealthy = false;
        let mut has_degraded = false;
        
        for (_, health) in checks {
            match health.status {
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
                _ => {}
            }
        }
        
        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
    
    /// Get health check endpoint response (for HTTP endpoints)
    pub async fn get_health_endpoint_response(&self) -> HealthEndpointResponse {
        let result = self.check_health().await;
        
        HealthEndpointResponse {
            status: result.status.as_emoji().to_string(),
            healthy: result.status == HealthStatus::Healthy,
            timestamp: result.timestamp,
            version: env!("CARGO_PKG_VERSION").to_string(),
            checks: result.checks.into_iter()
                .map(|(k, v)| (k, HealthCheckSummary {
                    status: v.status.as_emoji().to_string(),
                    latency_ms: v.latency_ms,
                    error: v.error,
                }))
                .collect(),
        }
    }
}

/// Health endpoint response for HTTP/REST APIs
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthEndpointResponse {
    pub status: String,
    pub healthy: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub checks: HashMap<String, HealthCheckSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckSummary {
    pub status: String,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_check() {
        let config = HealthCheckConfig {
            enable_redis_check: false, // Disable for test
            enable_nats_check: false,  // Disable for test
            ..Default::default()
        };
        
        let manager = HealthCheckManager::new(config);
        let result = manager.check_health().await;
        
        assert!(result.checks.contains_key("system"));
        assert!(result.checks.contains_key("connections"));
        assert!(!result.diagnostics.is_empty());
    }
}