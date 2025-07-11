//! Stress tests for performance features

#[cfg(test)]
mod stress_tests {
    use alchemist::{
        cache::{MemoryCache, Cache},
        rate_limiter::{RateLimiter, RateLimiterConfig},
    };
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::Instant;
    
    #[tokio::test]
    #[ignore] // Run only with --ignored flag
    async fn stress_test_cache() {
        println!("Starting cache stress test...");
        
        let cache = Arc::new(MemoryCache::new(10000));
        let start = Instant::now();
        let mut handles = vec![];
        
        // Spawn 100 concurrent tasks
        for i in 0..100 {
            let cache_clone = cache.clone();
            let handle = tokio::spawn(async move {
                // Each task performs 1000 operations
                for j in 0..1000 {
                    let key = format!("stress_key_{}_{}", i, j);
                    let value = format!("stress_value_{}_{}", i, j);
                    
                    // Set value
                    cache_clone.set(&key, &value, Duration::from_secs(300))
                        .await
                        .expect("Failed to set cache");
                    
                    // Get value
                    let retrieved: Option<String> = cache_clone.get(&key).await;
                    assert_eq!(retrieved, Some(value));
                    
                    // Random delete (10% chance)
                    if j % 10 == 0 {
                        cache_clone.delete(&key).await.ok();
                    }
                }
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.expect("Task failed");
        }
        
        let elapsed = start.elapsed();
        let total_ops = 100 * 1000 * 3; // set + get + occasional delete
        let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();
        
        println!("Cache stress test completed:");
        println!("  Total operations: {}", total_ops);
        println!("  Duration: {:.2}s", elapsed.as_secs_f64());
        println!("  Operations/sec: {:.0}", ops_per_sec);
        
        assert!(ops_per_sec > 10000.0, "Cache performance too low");
    }
    
    #[tokio::test]
    #[ignore] // Run only with --ignored flag
    async fn stress_test_rate_limiter() {
        println!("Starting rate limiter stress test...");
        
        let config = RateLimiterConfig {
            capacity: 1000,
            refill_rate: 100.0,
            window: Duration::from_secs(60),
        };
        
        let limiter = Arc::new(RateLimiter::new(config));
        let start = Instant::now();
        let mut handles = vec![];
        
        // Track metrics
        let allowed = Arc::new(tokio::sync::Mutex::new(0u64));
        let denied = Arc::new(tokio::sync::Mutex::new(0u64));
        
        // Spawn 50 concurrent users
        for user_id in 0..50 {
            let limiter_clone = limiter.clone();
            let allowed_clone = allowed.clone();
            let denied_clone = denied.clone();
            
            let handle = tokio::spawn(async move {
                let user_key = format!("user_{}", user_id);
                
                // Each user makes 100 requests
                for _ in 0..100 {
                    let is_allowed = limiter_clone
                        .check_rate_limit(&user_key, 1.0)
                        .await
                        .expect("Rate limit check failed");
                    
                    if is_allowed {
                        *allowed_clone.lock().await += 1;
                    } else {
                        *denied_clone.lock().await += 1;
                    }
                    
                    // Small delay between requests
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            });
            handles.push(handle);
        }
        
        // Wait for all tasks
        for handle in handles {
            handle.await.expect("Task failed");
        }
        
        let elapsed = start.elapsed();
        let total_allowed = *allowed.lock().await;
        let total_denied = *denied.lock().await;
        let total_requests = total_allowed + total_denied;
        
        println!("Rate limiter stress test completed:");
        println!("  Total requests: {}", total_requests);
        println!("  Allowed: {} ({:.1}%)", total_allowed, 
            total_allowed as f64 / total_requests as f64 * 100.0);
        println!("  Denied: {} ({:.1}%)", total_denied,
            total_denied as f64 / total_requests as f64 * 100.0);
        println!("  Duration: {:.2}s", elapsed.as_secs_f64());
        println!("  Requests/sec: {:.0}", total_requests as f64 / elapsed.as_secs_f64());
        
        // Verify rate limiting is working (should have some denials)
        assert!(total_denied > 0, "No rate limiting occurred");
        assert!(total_allowed > total_denied, "Too many requests denied");
    }
    
    #[tokio::test]
    #[ignore] // Run only with --ignored flag
    async fn stress_test_concurrent_cache_and_rate_limit() {
        println!("Starting combined stress test...");
        
        let cache = Arc::new(MemoryCache::new(5000));
        let limiter = Arc::new(RateLimiter::new(RateLimiterConfig {
            capacity: 500,
            refill_rate: 50.0,
            window: Duration::from_secs(60),
        }));
        
        let start = Instant::now();
        let mut handles = vec![];
        
        // Simulate 20 concurrent API clients
        for client_id in 0..20 {
            let cache_clone = cache.clone();
            let limiter_clone = limiter.clone();
            
            let handle = tokio::spawn(async move {
                let client_key = format!("client_{}", client_id);
                let mut cache_hits = 0;
                let mut cache_misses = 0;
                let mut rate_limited = 0;
                
                for i in 0..500 {
                    // Check rate limit first
                    if !limiter_clone.check_rate_limit(&client_key, 1.0).await.unwrap() {
                        rate_limited += 1;
                        continue;
                    }
                    
                    // Simulate API call with caching
                    let cache_key = format!("api_response_{}_{}", client_id, i % 10);
                    
                    if let Some(_value): Option<String> = cache_clone.get(&cache_key).await {
                        cache_hits += 1;
                    } else {
                        cache_misses += 1;
                        // Simulate API call and cache result
                        let response = format!("response_{}_{}", client_id, i);
                        cache_clone.set(&cache_key, &response, Duration::from_secs(60))
                            .await
                            .ok();
                    }
                    
                    // Small delay
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
                
                (cache_hits, cache_misses, rate_limited)
            });
            handles.push(handle);
        }
        
        // Collect results
        let mut total_hits = 0;
        let mut total_misses = 0;
        let mut total_limited = 0;
        
        for handle in handles {
            let (hits, misses, limited) = handle.await.expect("Task failed");
            total_hits += hits;
            total_misses += misses;
            total_limited += limited;
        }
        
        let elapsed = start.elapsed();
        let total_requests = total_hits + total_misses + total_limited;
        
        println!("Combined stress test completed:");
        println!("  Total requests: {}", total_requests);
        println!("  Cache hits: {} ({:.1}%)", total_hits,
            total_hits as f64 / (total_hits + total_misses) as f64 * 100.0);
        println!("  Cache misses: {}", total_misses);
        println!("  Rate limited: {}", total_limited);
        println!("  Duration: {:.2}s", elapsed.as_secs_f64());
        println!("  Effective requests/sec: {:.0}", 
            (total_hits + total_misses) as f64 / elapsed.as_secs_f64());
        
        // Verify both systems are working
        assert!(total_hits > 0, "No cache hits");
        assert!(total_limited > 0, "No rate limiting");
        assert!(total_hits > total_misses * 3, "Cache hit rate too low");
    }
}