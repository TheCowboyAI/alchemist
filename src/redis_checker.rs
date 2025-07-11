//! Redis connection health checking

use anyhow::Result;
use redis::{Client as RedisClient, AsyncCommands};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn, error};

/// Redis health status
#[derive(Debug, Clone)]
pub struct RedisHealth {
    pub is_connected: bool,
    pub latency_ms: Option<f64>,
    pub version: Option<String>,
    pub used_memory_mb: Option<f64>,
    pub connected_clients: Option<u32>,
    pub error: Option<String>,
}

impl RedisHealth {
    pub fn disconnected(error: String) -> Self {
        Self {
            is_connected: false,
            latency_ms: None,
            version: None,
            used_memory_mb: None,
            connected_clients: None,
            error: Some(error),
        }
    }
    
    pub fn is_healthy(&self) -> bool {
        self.is_connected && self.latency_ms.map(|l| l < 100.0).unwrap_or(false)
    }
}

/// Check Redis connection health
pub async fn check_redis_health(redis_url: &str) -> RedisHealth {
    let start = Instant::now();
    
    // Try to connect
    let client = match RedisClient::open(redis_url) {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create Redis client: {}", e);
            return RedisHealth::disconnected(format!("Connection failed: {}", e));
        }
    };
    
    let mut conn = match client.get_async_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to connect to Redis: {}", e);
            return RedisHealth::disconnected(format!("Connection failed: {}", e));
        }
    };
    
    // Measure ping latency
    let ping_start = Instant::now();
    match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
        Ok(response) if response == "PONG" => {
            let latency_ms = ping_start.elapsed().as_secs_f64() * 1000.0;
            debug!("Redis ping successful: {:.2}ms", latency_ms);
            
            // Get server info
            let info_result: Result<String, redis::RedisError> = 
                redis::cmd("INFO").query_async(&mut conn).await;
            
            let mut health = RedisHealth {
                is_connected: true,
                latency_ms: Some(latency_ms),
                version: None,
                used_memory_mb: None,
                connected_clients: None,
                error: None,
            };
            
            // Parse INFO response
            if let Ok(info) = info_result {
                for line in info.lines() {
                    if line.starts_with("redis_version:") {
                        health.version = line.split(':').nth(1).map(|s| s.to_string());
                    } else if line.starts_with("used_memory:") {
                        if let Some(bytes_str) = line.split(':').nth(1) {
                            if let Ok(bytes) = bytes_str.parse::<u64>() {
                                health.used_memory_mb = Some(bytes as f64 / 1024.0 / 1024.0);
                            }
                        }
                    } else if line.starts_with("connected_clients:") {
                        if let Some(clients_str) = line.split(':').nth(1) {
                            health.connected_clients = clients_str.parse().ok();
                        }
                    }
                }
            }
            
            let total_time = start.elapsed();
            info!(
                "Redis health check completed in {:.2}ms - Connected: v{} | Memory: {:.1}MB | Clients: {}",
                total_time.as_secs_f64() * 1000.0,
                health.version.as_deref().unwrap_or("unknown"),
                health.used_memory_mb.unwrap_or(0.0),
                health.connected_clients.unwrap_or(0)
            );
            
            health
        }
        Ok(_) => {
            warn!("Unexpected Redis ping response");
            RedisHealth::disconnected("Unexpected ping response".to_string())
        }
        Err(e) => {
            error!("Redis ping failed: {}", e);
            RedisHealth::disconnected(format!("Ping failed: {}", e))
        }
    }
}

/// Redis connection monitor
pub struct RedisMonitor {
    redis_url: String,
    check_interval: Duration,
}

impl RedisMonitor {
    pub fn new(redis_url: String, check_interval: Duration) -> Self {
        Self {
            redis_url,
            check_interval,
        }
    }
    
    /// Start monitoring Redis health
    pub async fn start_monitoring(self) {
        tokio::spawn(async move {
            let mut consecutive_failures = 0;
            
            loop {
                let health = check_redis_health(&self.redis_url).await;
                
                if health.is_connected {
                    consecutive_failures = 0;
                    
                    // Log warnings for high latency
                    if let Some(latency) = health.latency_ms {
                        if latency > 50.0 {
                            warn!("Redis latency is high: {:.2}ms", latency);
                        }
                    }
                    
                    // Log warnings for high memory usage
                    if let Some(memory_mb) = health.used_memory_mb {
                        if memory_mb > 1000.0 {
                            warn!("Redis memory usage is high: {:.1}MB", memory_mb);
                        }
                    }
                } else {
                    consecutive_failures += 1;
                    error!(
                        "Redis health check failed ({} consecutive): {:?}",
                        consecutive_failures,
                        health.error
                    );
                    
                    // Alert after 3 consecutive failures
                    if consecutive_failures >= 3 {
                        error!("Redis appears to be down! Cache functionality degraded.");
                    }
                }
                
                tokio::time::sleep(self.check_interval).await;
            }
        });
    }
}

/// Quick Redis connectivity test
pub async fn test_redis_connection(redis_url: &str) -> Result<()> {
    let health = check_redis_health(redis_url).await;
    
    if health.is_connected {
        info!(
            "Redis connection successful - Version: {} | Latency: {:.2}ms",
            health.version.unwrap_or_else(|| "unknown".to_string()),
            health.latency_ms.unwrap_or(0.0)
        );
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Redis connection failed: {}",
            health.error.unwrap_or_else(|| "Unknown error".to_string())
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_redis_health_check() {
        // Test with invalid URL
        let health = check_redis_health("redis://invalid-host:6379").await;
        assert!(!health.is_connected);
        assert!(health.error.is_some());
        
        // Test with localhost (may or may not be running)
        let health = check_redis_health("redis://localhost:6379").await;
        if health.is_connected {
            assert!(health.latency_ms.is_some());
            assert!(health.version.is_some());
        }
    }
}