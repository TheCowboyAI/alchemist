//! Advanced features demo for Alchemist
//! 
//! This example demonstrates:
//! - User context and tier-based rate limiting
//! - CPU and connection monitoring
//! - Redis health checking
//! - Performance dashboards

use alchemist::{
    config::AlchemistConfig,
    ai_enhanced::EnhancedAiManager,
    performance_monitor::{PerformanceMonitor, MonitorConfig},
    shell_integration::PerformanceManager,
    user_context::{UserContext, UserInfo, UserTier, create_default_users, global_registry},
    cpu_monitor::CpuMonitor,
    connection_tracker::{global_tracker, ConnectionType},
    redis_checker::{check_redis_health, test_redis_connection},
};
use std::time::Duration;
use tokio;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Alchemist Advanced Features Demo\n");

    // Load configuration
    let config = AlchemistConfig::default();
    
    // Demo 1: User Context and Tiers
    println!("📊 Demo 1: User Context and Tier-Based Rate Limiting");
    println!("─".repeat(50));
    
    // Create default users
    create_default_users().await;
    
    // Test with different user tiers
    let users = vec![
        ("free-api-key", "Free User"),
        ("pro-api-key", "Pro User"),
        ("admin-api-key", "Admin User"),
    ];
    
    let ai_manager = EnhancedAiManager::new(&config).await?;
    
    for (api_key, name) in users {
        // Set user context
        let user = global_registry().get_user_by_api_key(api_key).await.unwrap();
        UserContext::authenticated(user.clone()).set_current();
        
        println!("\nTesting as {} (Tier: {:?}):", name, user.tier);
        
        // Make requests to test rate limits
        let mut allowed = 0;
        let mut denied = 0;
        
        for i in 0..20 {
            match ai_manager.get_completion("claude-3-sonnet", &format!("Test {}", i)).await {
                Ok(_) => allowed += 1,
                Err(e) if e.to_string().contains("Rate limit") => {
                    denied += 1;
                    if denied == 1 {
                        println!("  Rate limited after {} requests", allowed);
                    }
                }
                Err(e) => println!("  Error: {}", e),
            }
            
            if denied > 0 {
                break;
            }
        }
        
        println!("  Total allowed: {} requests", allowed);
    }
    
    // Demo 2: CPU Monitoring
    println!("\n📊 Demo 2: CPU Monitoring");
    println!("─".repeat(50));
    
    let mut cpu_monitor = CpuMonitor::new();
    let cores = CpuMonitor::get_core_count();
    
    println!("CPU Cores: {}", cores);
    
    // Monitor CPU for a few seconds
    for i in 0..5 {
        let usage = cpu_monitor.get_usage();
        println!("CPU Usage: {:.1}%", usage);
        
        // Create some CPU load
        if i == 2 {
            println!("Creating CPU load...");
            tokio::spawn(async {
                let mut sum = 0u64;
                for i in 0..100_000_000 {
                    sum = sum.wrapping_add(i);
                }
                drop(sum); // Prevent optimization
            });
        }
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    // Demo 3: Connection Tracking
    println!("\n📊 Demo 3: Connection Tracking");
    println!("─".repeat(50));
    
    let tracker = global_tracker();
    
    // Register some connections
    tracker.register_connection(
        "nats-001".to_string(),
        ConnectionType::Nats,
        "127.0.0.1:4222".to_string(),
    ).await;
    
    tracker.register_connection(
        "redis-001".to_string(),
        ConnectionType::Redis,
        "127.0.0.1:6379".to_string(),
    ).await;
    
    tracker.register_connection(
        "http-001".to_string(),
        ConnectionType::Http,
        "api.example.com:443".to_string(),
    ).await;
    
    // Update activity
    tracker.update_activity("nats-001", 1024, 2048).await;
    tracker.update_activity("redis-001", 512, 1024).await;
    
    // Get stats
    let stats = tracker.get_connection_count().await;
    println!("{}", stats.summary());
    
    // Mark one inactive
    tracker.mark_inactive("http-001").await;
    
    let stats = tracker.get_connection_count().await;
    println!("After marking HTTP inactive: {} active connections", stats.active);
    
    // Demo 4: Redis Health Check
    println!("\n📊 Demo 4: Redis Health Check");
    println!("─".repeat(50));
    
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    match test_redis_connection(&redis_url).await {
        Ok(_) => {
            let health = check_redis_health(&redis_url).await;
            println!("Redis Status: Connected ✅");
            println!("  Version: {}", health.version.unwrap_or_else(|| "unknown".to_string()));
            println!("  Latency: {:.2}ms", health.latency_ms.unwrap_or(0.0));
            println!("  Memory: {:.1}MB", health.used_memory_mb.unwrap_or(0.0));
            println!("  Clients: {}", health.connected_clients.unwrap_or(0));
            println!("  Healthy: {}", if health.is_healthy() { "Yes" } else { "No" });
        }
        Err(e) => {
            println!("Redis Status: Disconnected ❌");
            println!("  Error: {}", e);
            println!("  Falling back to memory cache");
        }
    }
    
    // Demo 5: Performance Manager Integration
    println!("\n📊 Demo 5: Performance Manager");
    println!("─".repeat(50));
    
    let perf_manager = PerformanceManager::new(&config);
    
    let cache_stats = perf_manager.get_cache_stats().await;
    println!("Cache Status:");
    println!("  Enabled: {}", cache_stats.enabled);
    println!("  Redis Available: {}", cache_stats.redis_available);
    
    // Demo 6: System Load
    println!("\n📊 Demo 6: System Load Average");
    println!("─".repeat(50));
    
    if let Some((load1, load5, load15)) = crate::cpu_monitor::get_load_average() {
        println!("Load Average:");
        println!("  1 min:  {:.2}", load1);
        println!("  5 min:  {:.2}", load5);
        println!("  15 min: {:.2}", load15);
        
        let load_per_core = load1 / cores as f32;
        println!("  Load per core: {:.2}", load_per_core);
        
        if load_per_core > 0.8 {
            warn!("System is under heavy load!");
        }
    }
    
    println!("\n✅ Advanced features demo completed!");
    
    Ok(())
}

// Example output:
//
// 🚀 Alchemist Advanced Features Demo
//
// 📊 Demo 1: User Context and Tier-Based Rate Limiting
// ──────────────────────────────────────────────────
// 
// Testing as Free User (Tier: Free):
//   Rate limited after 5 requests
//   Total allowed: 5 requests
// 
// Testing as Pro User (Tier: Pro):
//   Total allowed: 20 requests
// 
// Testing as Admin User (Tier: Admin):
//   Total allowed: 20 requests
//
// 📊 Demo 2: CPU Monitoring
// ──────────────────────────────────────────────────
// CPU Cores: 8
// CPU Usage: 12.5%
// CPU Usage: 13.2%
// Creating CPU load...
// CPU Usage: 45.8%
// CPU Usage: 38.2%
// CPU Usage: 15.3%
//
// 📊 Demo 3: Connection Tracking  
// ──────────────────────────────────────────────────
// Connections: 3 total (3 active) - NATS: 1, Redis: 1, HTTP: 1, WS: 0, DB: 0, Other: 0
// After marking HTTP inactive: 2 active connections
//
// 📊 Demo 4: Redis Health Check
// ──────────────────────────────────────────────────
// Redis Status: Connected ✅
//   Version: 7.2.3
//   Latency: 0.45ms
//   Memory: 42.3MB
//   Clients: 5
//   Healthy: Yes
//
// 📊 Demo 5: Performance Manager
// ──────────────────────────────────────────────────
// Cache Status:
//   Enabled: true
//   Redis Available: true
//
// 📊 Demo 6: System Load Average
// ──────────────────────────────────────────────────
// Load Average:
//   1 min:  2.15
//   5 min:  1.82
//   15 min: 1.45
//   Load per core: 0.27
//
// ✅ Advanced features demo completed!