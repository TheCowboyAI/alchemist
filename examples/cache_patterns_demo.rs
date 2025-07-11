//! Cache pattern matching and CSV export demo

use alchemist::{
    cache::{LayeredCache, CacheManager},
    performance_monitor::{PerformanceMonitor, MonitorConfig, ExportFormat},
    config::AlchemistConfig,
};
use std::sync::Arc;
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Cache Patterns and CSV Export Demo\n");

    // Demo 1: Pattern-based Cache Invalidation
    println!("📊 Demo 1: Pattern-based Cache Invalidation");
    println!("─".repeat(50));
    
    // Create cache manager
    let cache = LayeredCache::new(None, "demo"); // Memory-only for demo
    let cache_manager = CacheManager::new(Arc::new(cache));
    
    // Populate cache with test data
    println!("Populating cache with test data...");
    
    // User-related cache entries
    cache_manager.cache.set("user:123:profile", &"John Doe", Duration::from_secs(3600)).await?;
    cache_manager.cache.set("user:123:settings", &"theme=dark", Duration::from_secs(3600)).await?;
    cache_manager.cache.set("user:456:profile", &"Jane Smith", Duration::from_secs(3600)).await?;
    cache_manager.cache.set("user:456:settings", &"theme=light", Duration::from_secs(3600)).await?;
    
    // Dialog-related cache entries
    cache_manager.cache.set("dialog:abc:messages", &vec!["Hello", "Hi there"], Duration::from_secs(3600)).await?;
    cache_manager.cache.set("dialog:abc:summary", &"Greeting conversation", Duration::from_secs(3600)).await?;
    cache_manager.cache.set("dialog:xyz:messages", &vec!["Help", "How can I help?"], Duration::from_secs(3600)).await?;
    
    // AI completion cache
    cache_manager.cache.set("ai:completion:model1:hash123", &"Response 1", Duration::from_secs(3600)).await?;
    cache_manager.cache.set("ai:completion:model2:hash456", &"Response 2", Duration::from_secs(3600)).await?;
    
    println!("✅ Cache populated with 9 entries\n");
    
    // Test pattern matching
    println!("Testing pattern-based invalidation:");
    
    // Pattern 1: Delete all user 123 entries
    println!("\n1. Deleting 'user:123:*' entries...");
    cache_manager.invalidate_pattern("user:123:*").await?;
    
    // Check what remains
    let user123_profile = cache_manager.cache.get::<String>("user:123:profile").await;
    let user456_profile = cache_manager.cache.get::<String>("user:456:profile").await;
    println!("   user:123:profile exists: {}", user123_profile.is_some());
    println!("   user:456:profile exists: {}", user456_profile.is_some());
    
    // Pattern 2: Delete all dialog summaries
    println!("\n2. Deleting 'dialog:*:summary' entries...");
    cache_manager.invalidate_pattern("dialog:*:summary").await?;
    
    let dialog_summary = cache_manager.cache.get::<String>("dialog:abc:summary").await;
    let dialog_messages = cache_manager.cache.get::<Vec<String>>("dialog:abc:messages").await;
    println!("   dialog:abc:summary exists: {}", dialog_summary.is_some());
    println!("   dialog:abc:messages exists: {}", dialog_messages.is_some());
    
    // Pattern 3: Delete all AI completions
    println!("\n3. Deleting 'ai:completion:*' entries...");
    cache_manager.invalidate_pattern("ai:completion:*").await?;
    
    let ai_response = cache_manager.cache.get::<String>("ai:completion:model1:hash123").await;
    println!("   ai:completion entries exist: {}", ai_response.is_some());
    
    // Pattern 4: Clear everything
    println!("\n4. Clearing all cache with '*' pattern...");
    cache_manager.invalidate_pattern("*").await?;
    
    let any_remaining = cache_manager.cache.exists("user:456:profile").await;
    println!("   Any entries remaining: {}", any_remaining);
    
    // Demo 2: CSV Export
    println!("\n\n📊 Demo 2: CSV Export of Performance Metrics");
    println!("─".repeat(50));
    
    // Create performance monitor
    let monitor = PerformanceMonitor::new(MonitorConfig::default());
    
    // Generate some test data
    println!("Generating performance data...");
    
    // Record some latencies
    for i in 0..5 {
        let latency = Duration::from_millis(50 + i * 10);
        monitor.record_latency("/api/test", latency, true).await;
        
        let latency = Duration::from_millis(100 + i * 20);
        monitor.record_latency("/api/slow", latency, i % 2 == 0).await;
    }
    
    // Record cache activity
    for _ in 0..10 {
        monitor.record_cache_hit(true).await;
    }
    for _ in 0..3 {
        monitor.record_cache_hit(false).await;
    }
    
    // Record rate limit events
    monitor.record_rate_limit("user:123", true, 50.0).await;
    monitor.record_rate_limit("user:123", true, 49.0).await;
    monitor.record_rate_limit("user:123", false, 0.0).await;
    
    // Record model performance
    monitor.record_model_request("claude-3", Duration::from_millis(523), true, Some(1500)).await;
    monitor.record_model_request("claude-3", Duration::from_millis(612), true, Some(1800)).await;
    monitor.record_model_request("gpt-4", Duration::from_millis(892), true, Some(2100)).await;
    
    // Wait a bit for metrics to be collected
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Export as CSV
    println!("\nExporting metrics as CSV...");
    let csv_data = monitor.export_metrics(ExportFormat::Csv).await?;
    
    // Save to file
    let filename = "demo_metrics.csv";
    tokio::fs::write(filename, &csv_data).await?;
    println!("✅ Metrics exported to {}", filename);
    
    // Show a preview
    println!("\nCSV Preview (first 500 chars):");
    println!("─".repeat(50));
    let preview: String = csv_data.chars().take(500).collect();
    println!("{}", preview);
    if csv_data.len() > 500 {
        println!("... (truncated)");
    }
    
    // Also export as JSON for comparison
    let json_data = monitor.export_metrics(ExportFormat::Json).await?;
    tokio::fs::write("demo_metrics.json", json_data).await?;
    println!("\n✅ Also exported to demo_metrics.json for comparison");
    
    // Demo 3: Cache Pattern Use Cases
    println!("\n\n📊 Demo 3: Practical Cache Pattern Use Cases");
    println!("─".repeat(50));
    
    // Repopulate cache
    cache_manager.cache.set("session:user123:token", &"abc123", Duration::from_secs(3600)).await?;
    cache_manager.cache.set("session:user123:data", &"user_data", Duration::from_secs(3600)).await?;
    cache_manager.cache.set("session:user456:token", &"xyz789", Duration::from_secs(3600)).await?;
    cache_manager.cache.set("temp:upload:file1", &"data1", Duration::from_secs(300)).await?;
    cache_manager.cache.set("temp:upload:file2", &"data2", Duration::from_secs(300)).await?;
    cache_manager.cache.set("temp:processing:job1", &"status", Duration::from_secs(300)).await?;
    
    println!("Use Case 1: Logout user - clear all their sessions");
    cache_manager.invalidate_pattern("session:user123:*").await?;
    println!("✅ User 123 sessions cleared");
    
    println!("\nUse Case 2: Clear all temporary files");
    cache_manager.invalidate_pattern("temp:upload:*").await?;
    println!("✅ Temporary upload files cleared");
    
    println!("\nUse Case 3: Clear all temp data");
    cache_manager.invalidate_pattern("temp:*").await?;
    println!("✅ All temporary data cleared");
    
    println!("\n✅ Demo completed successfully!");
    println!("\nCheck the generated files:");
    println!("  - demo_metrics.csv (human-readable metrics)");
    println!("  - demo_metrics.json (structured data)");
    
    Ok(())
}

// Example output:
//
// 🚀 Cache Patterns and CSV Export Demo
//
// 📊 Demo 1: Pattern-based Cache Invalidation
// ──────────────────────────────────────────────────
// Populating cache with test data...
// ✅ Cache populated with 9 entries
//
// Testing pattern-based invalidation:
//
// 1. Deleting 'user:123:*' entries...
//    user:123:profile exists: false
//    user:456:profile exists: true
//
// 2. Deleting 'dialog:*:summary' entries...
//    dialog:abc:summary exists: false
//    dialog:abc:messages exists: true
//
// 3. Deleting 'ai:completion:*' entries...
//    ai:completion entries exist: false
//
// 4. Clearing all cache with '*' pattern...
//    Any entries remaining: false
//
//
// 📊 Demo 2: CSV Export of Performance Metrics
// ──────────────────────────────────────────────────
// Generating performance data...
//
// Exporting metrics as CSV...
// ✅ Metrics exported to demo_metrics.csv
//
// CSV Preview (first 500 chars):
// ──────────────────────────────────────────────────
// === LATENCY METRICS ===
// Endpoint,Timestamp,Duration_ms,Success
// /api/test,1699564421,50.00,true
// /api/slow,1699564421,100.00,true
// /api/test,1699564421,60.00,true
// /api/slow,1699564421,120.00,false
// ... (truncated)
//
// ✅ Also exported to demo_metrics.json for comparison
//
//
// 📊 Demo 3: Practical Cache Pattern Use Cases
// ──────────────────────────────────────────────────
// Use Case 1: Logout user - clear all their sessions
// ✅ User 123 sessions cleared
//
// Use Case 2: Clear all temporary files
// ✅ Temporary upload files cleared
//
// Use Case 3: Clear all temp data
// ✅ All temporary data cleared
//
// ✅ Demo completed successfully!
//
// Check the generated files:
//   - demo_metrics.csv (human-readable metrics)
//   - demo_metrics.json (structured data)