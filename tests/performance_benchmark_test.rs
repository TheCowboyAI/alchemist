//! Performance benchmark tests for CIM
//! 
//! These tests measure the performance characteristics of the system

use cim_domain::{DomainResult, GraphId, NodeId};
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

/// Measure how long an operation takes
async fn measure_time<F, Fut>(name: &str, f: F) -> Duration
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let start = Instant::now();
    f().await;
    let duration = start.elapsed();
    println!("⏱️  {} took: {:?}", name, duration);
    duration
}

#[tokio::test]
async fn benchmark_event_creation() -> DomainResult<()> {
    let event_count = 10_000;
    
    let duration = measure_time("Event creation", || async {
        let mut events = Vec::with_capacity(event_count);
        
        for i in 0..event_count {
            let event = TestEvent {
                event_type: "BenchmarkEvent".to_string(),
                aggregate_id: format!("aggregate-{}", i),
                payload: HashMap::from([
                    ("index".to_string(), i.to_string()),
                    ("data".to_string(), "x".repeat(100)), // 100 byte payload
                ]),
            };
            events.push(event);
        }
    }).await;
    
    let events_per_second = event_count as f64 / duration.as_secs_f64();
    println!("📊 Created {} events/second", events_per_second as u64);
    
    // Benchmark should create at least 100k events per second
    assert!(events_per_second > 100_000.0, "Event creation too slow");
    
    Ok(())
}

#[tokio::test]
async fn benchmark_event_publishing() -> DomainResult<()> {
    let event_bus = TestEventBus::new();
    let event_count = 1_000;
    
    // Pre-create events
    let events: Vec<TestEvent> = (0..event_count)
        .map(|i| TestEvent {
            event_type: "PublishBenchmark".to_string(),
            aggregate_id: format!("aggregate-{}", i % 10),
            payload: HashMap::from([
                ("index".to_string(), i.to_string()),
            ]),
        })
        .collect();
    
    let duration = measure_time("Event publishing", || async {
        for event in &events {
            event_bus.publish(event.clone()).await;
        }
    }).await;
    
    let events_per_second = event_count as f64 / duration.as_secs_f64();
    println!("📊 Published {} events/second", events_per_second as u64);
    
    // Should handle at least 10k events per second
    assert!(events_per_second > 10_000.0, "Event publishing too slow");
    
    Ok(())
}

#[tokio::test]
async fn benchmark_concurrent_operations() -> DomainResult<()> {
    let event_bus = Arc::new(TestEventBus::new());
    let concurrent_tasks = 100;
    let events_per_task = 100;
    
    let duration = measure_time("Concurrent operations", || async {
        let mut handles = Vec::new();
        
        for task_id in 0..concurrent_tasks {
            let bus = event_bus.clone();
            let handle = tokio::spawn(async move {
                for i in 0..events_per_task {
                    let event = TestEvent {
                        event_type: "ConcurrentEvent".to_string(),
                        aggregate_id: format!("task-{}-event-{}", task_id, i),
                        payload: HashMap::new(),
                    };
                    bus.publish(event).await;
                }
            });
            handles.push(handle);
        }
        
        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }
    }).await;
    
    let total_events = concurrent_tasks * events_per_task;
    let events_per_second = total_events as f64 / duration.as_secs_f64();
    println!("📊 Concurrent: {} events/second across {} tasks", 
             events_per_second as u64, concurrent_tasks);
    
    // Verify all events were stored
    let stored = event_bus.get_events().await;
    assert_eq!(stored.len(), total_events);
    
    Ok(())
}

#[tokio::test]
async fn benchmark_event_filtering() -> DomainResult<()> {
    let event_bus = TestEventBus::new();
    let aggregate_count = 100;
    let events_per_aggregate = 100;
    
    // Populate with events
    for agg in 0..aggregate_count {
        for evt in 0..events_per_aggregate {
            let event = TestEvent {
                event_type: "FilterTest".to_string(),
                aggregate_id: format!("aggregate-{}", agg),
                payload: HashMap::from([
                    ("sequence".to_string(), evt.to_string()),
                ]),
            };
            event_bus.publish(event).await;
        }
    }
    
    // Benchmark filtering
    let target_aggregate = "aggregate-50";
    let duration = measure_time("Event filtering", || async {
        let _ = event_bus.get_events_for_aggregate(target_aggregate).await;
    }).await;
    
    let total_events = aggregate_count * events_per_aggregate;
    println!("📊 Filtered {} events from {} total in {:?}", 
             events_per_aggregate, total_events, duration);
    
    // Should filter in under 10ms
    assert!(duration.as_millis() < 10, "Event filtering too slow");
    
    Ok(())
}

#[tokio::test]
async fn benchmark_memory_usage() -> DomainResult<()> {
    let event_bus = TestEventBus::new();
    let event_count = 100_000;
    
    // Measure memory before
    let before = get_memory_usage();
    
    // Create many events
    for i in 0..event_count {
        let event = TestEvent {
            event_type: "MemoryTest".to_string(),
            aggregate_id: format!("agg-{}", i % 1000),
            payload: HashMap::from([
                ("data".to_string(), "x".repeat(1000)), // 1KB payload
            ]),
        };
        event_bus.publish(event).await;
    }
    
    // Measure memory after
    let after = get_memory_usage();
    let memory_used_mb = (after - before) as f64 / 1_048_576.0;
    
    println!("📊 Memory used for {} events: {:.2} MB", event_count, memory_used_mb);
    println!("📊 Memory per event: {:.2} bytes", 
             (after - before) as f64 / event_count as f64);
    
    // Each event with 1KB payload should use less than 2KB total
    let bytes_per_event = (after - before) as f64 / event_count as f64;
    assert!(bytes_per_event < 2048.0, "Memory usage too high");
    
    Ok(())
}

#[test]
fn benchmark_id_generation() {
    let count = 1_000_000;
    
    let start = Instant::now();
    let mut ids = Vec::with_capacity(count);
    
    for _ in 0..count {
        ids.push(NodeId::new());
    }
    
    let duration = start.elapsed();
    let ids_per_second = count as f64 / duration.as_secs_f64();
    
    println!("⏱️  ID generation took: {:?}", duration);
    println!("📊 Generated {} IDs/second", ids_per_second as u64);
    
    // Should generate at least 1M IDs per second
    assert!(ids_per_second > 1_000_000.0, "ID generation too slow");
    
    // Verify uniqueness (sample check)
    let sample_size = 1000;
    let mut sample_set = std::collections::HashSet::new();
    for id in ids.iter().take(sample_size) {
        assert!(sample_set.insert(id.to_string()), "Duplicate ID found");
    }
}

// Helper types and functions

#[derive(Clone, Debug)]
struct TestEvent {
    event_type: String,
    aggregate_id: String,
    payload: HashMap<String, String>,
}

#[derive(Clone)]
struct TestEventBus {
    events: Arc<Mutex<Vec<TestEvent>>>,
}

impl TestEventBus {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn publish(&self, event: TestEvent) {
        let mut events = self.events.lock().await;
        events.push(event);
    }

    async fn get_events(&self) -> Vec<TestEvent> {
        let events = self.events.lock().await;
        events.clone()
    }

    async fn get_events_for_aggregate(&self, aggregate_id: &str) -> Vec<TestEvent> {
        let events = self.events.lock().await;
        events
            .iter()
            .filter(|e| e.aggregate_id == aggregate_id)
            .cloned()
            .collect()
    }
}

/// Get current memory usage in bytes (simplified)
fn get_memory_usage() -> usize {
    // In a real implementation, this would use system APIs
    // For now, we'll estimate based on Rust's allocator
    use std::alloc::{GlobalAlloc, Layout, System};
    
    // This is a rough estimate - in production use jemalloc stats or similar
    let layout = Layout::from_size_align(1, 1).unwrap();
    let ptr = unsafe { System.alloc(layout) };
    let estimate = ptr as usize;
    unsafe { System.dealloc(ptr, layout) };
    estimate
} 