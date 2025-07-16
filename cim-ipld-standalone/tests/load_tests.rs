//! Load tests for CIM-IPLD
//!
//! Tests system behavior under heavy load and stress conditions.
//!
//! ## Test Scenarios
//!
//! ```mermaid
//! graph TD
//!     A[Load Tests] --> B[Throughput Tests]
//!     A --> C[Scalability Tests]
//!     A --> D[Stress Tests]
//!     A --> E[Endurance Tests]
//!
//!     B --> B1[Write Throughput]
//!     B --> B2[Read Throughput]
//!     B --> B3[Mixed Workload]
//!
//!     C --> C1[Linear Scaling]
//!     C --> C2[Horizontal Scaling]
//!     C --> C3[Resource Utilization]
//!
//!     D --> D1[Peak Load]
//!     D --> D2[Burst Traffic]
//!     D --> D3[Recovery Time]
//!
//!     E --> E1[Long Running]
//!     E --> E2[Memory Leaks]
//!     E --> E3[Performance Degradation]
//! ```

use cim_ipld::*;
use cim_ipld::object_store::NatsObjectStore;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use futures::stream::{self, StreamExt};

mod common;
use common::*;

/// Performance metrics collector
#[derive(Debug, Default)]
struct PerformanceMetrics {
    total_operations: AtomicUsize,
    successful_operations: AtomicUsize,
    failed_operations: AtomicUsize,
    total_bytes_written: AtomicU64,
    total_bytes_read: AtomicU64,
    write_latencies: RwLock<Vec<Duration>>,
    read_latencies: RwLock<Vec<Duration>>,
}

impl PerformanceMetrics {
    async fn record_write(&self, size: usize, latency: Duration, success: bool) {
        if success {
            self.successful_operations.fetch_add(1, Ordering::Relaxed);
            self.total_bytes_written.fetch_add(size as u64, Ordering::Relaxed);
            self.write_latencies.write().await.push(latency);
        } else {
            self.failed_operations.fetch_add(1, Ordering::Relaxed);
        }
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    async fn record_read(&self, size: usize, latency: Duration, success: bool) {
        if success {
            self.successful_operations.fetch_add(1, Ordering::Relaxed);
            self.total_bytes_read.fetch_add(size as u64, Ordering::Relaxed);
            self.read_latencies.write().await.push(latency);
        } else {
            self.failed_operations.fetch_add(1, Ordering::Relaxed);
        }
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    async fn calculate_percentiles(&self) -> (Duration, Duration, Duration) {
        let mut write_latencies = self.write_latencies.read().await.clone();
        let mut read_latencies = self.read_latencies.read().await.clone();

        write_latencies.extend(read_latencies);
        write_latencies.sort();

        if write_latencies.is_empty() {
            return (Duration::ZERO, Duration::ZERO, Duration::ZERO);
        }

        let p50_idx = write_latencies.len() / 2;
        let p95_idx = (write_latencies.len() as f64 * 0.95) as usize;
        let p99_idx = (write_latencies.len() as f64 * 0.99) as usize;

        (
            write_latencies[p50_idx],
            write_latencies[p95_idx.min(write_latencies.len() - 1)],
            write_latencies[p99_idx.min(write_latencies.len() - 1)],
        )
    }

    async fn print_summary(&self, duration: Duration) {
        let total_ops = self.total_operations.load(Ordering::Relaxed);
        let successful_ops = self.successful_operations.load(Ordering::Relaxed);
        let failed_ops = self.failed_operations.load(Ordering::Relaxed);
        let bytes_written = self.total_bytes_written.load(Ordering::Relaxed);
        let bytes_read = self.total_bytes_read.load(Ordering::Relaxed);

        let (p50, p95, p99) = self.calculate_percentiles().await;

        println!("\n=== Performance Summary ===");
        println!("Duration: {:?}", duration);
        println!("Total operations: {total_ops}");
        println!("Successful: {successful_ops} ({:.2}%)", (successful_ops as f64 / total_ops as f64) * 100.0);
        println!("Failed: {failed_ops} ({:.2}%)", (failed_ops as f64 / total_ops as f64) * 100.0);
        println!("Throughput: {:.2} ops/sec", total_ops as f64 / duration.as_secs_f64());
        println!("Data written: {:.2} MB", bytes_written as f64 / 1_048_576.0);
        println!("Data read: {:.2} MB", bytes_read as f64 / 1_048_576.0);
        println!("Latency percentiles:");
        println!("  P50: {:?}", p50);
        println!("  P95: {:?}", p95);
        println!("  P99: {:?}", p99);
    }
}

/// Generate test data of specified size
fn generate_test_data(size: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen::<u8>()).collect()
}

#[tokio::test]
#[ignore] // Run manually - requires NATS server and extended runtime
async fn test_write_throughput() {
    /// Test maximum write throughput
    ///
    /// Given: High volume of write requests
    /// When: Continuous writes for 60 seconds
    /// Then: Measure throughput and latency

    let context = TestContext::new().await
        .expect("Test context creation should succeed");
    let storage = Arc::new(context.storage);
    let metrics = Arc::new(PerformanceMetrics::default());

    let num_workers = 50;
    let test_duration = Duration::from_secs(60);
    let data_sizes = vec![1024, 10240, 102400]; // 1KB, 10KB, 100KB

    let start_time = Instant::now();
    let mut handles = Vec::new();

    for worker_id in 0..num_workers {
        let storage_clone = storage.clone();
        let metrics_clone = metrics.clone();
        let data_sizes_clone = data_sizes.clone();

        let handle = tokio::spawn(async move {
            let mut operation_count = 0;

            while start_time.elapsed() < test_duration {
                let size = data_sizes_clone[operation_count % data_sizes_clone.len()];
                let data = generate_test_data(size);

                let content = TestContent {
                    data: data.clone(),
                    metadata: vec![
                        ("worker_id".to_string(), worker_id.to_string()),
                        ("operation".to_string(), operation_count.to_string()),
                        ("size".to_string(), size.to_string()),
                    ].into_iter().collect(),
                };

                let op_start = Instant::now();
                let result = storage_clone.put(&content).await;
                let latency = op_start.elapsed();

                metrics_clone.record_write(size, latency, result.is_ok()).await;
                operation_count += 1;

                // Small delay to prevent overwhelming the system
                tokio::time::sleep(Duration::from_millis(10)).await;
            }

            operation_count
        });

        handles.push(handle);
    }

    // Wait for all workers to complete
    let operations_per_worker: Vec<usize> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    let total_duration = start_time.elapsed();
    metrics.print_summary(total_duration).await;

    // Verify performance targets
    let total_ops = metrics.total_operations.load(Ordering::Relaxed);
    let ops_per_second = total_ops as f64 / total_duration.as_secs_f64();

    println!("\nWorker statistics:");
    println!("  Workers: {num_workers}");
    println!("  Avg ops/worker: {:.2}",
        operations_per_worker.iter().sum::<usize>() as f64 / num_workers as f64);

    assert!(ops_per_second > 100.0,
        "Write throughput should exceed 100 ops/sec, got {:.2}", ops_per_second);
}

#[tokio::test]
#[ignore] // Run manually - requires NATS server and extended runtime
async fn test_read_throughput() {
    /// Test maximum read throughput
    ///
    /// Given: Pre-populated data
    /// When: Continuous reads for 60 seconds
    /// Then: Measure throughput and latency

    let context = TestContext::new().await
        .expect("Test context creation should succeed");
    let storage = Arc::new(context.storage);
    let metrics = Arc::new(PerformanceMetrics::default());

    // Pre-populate data
    println!("Pre-populating test data...");
    let mut test_cids = Vec::new();
    let data_sizes = vec![1024, 10240, 102400]; // 1KB, 10KB, 100KB

    for i in 0..1000 {
        let size = data_sizes[i % data_sizes.len()];
        let content = TestContent {
            data: generate_test_data(size),
            metadata: vec![
                ("index".to_string(), i.to_string()),
                ("size".to_string(), size.to_string()),
            ].into_iter().collect(),
        };

        let cid = storage.put(&content).await
            .expect("Pre-population should succeed");
        test_cids.push((cid, size));
    }

    println!("Starting read throughput test with {} items...", test_cids.len());

    let test_cids = Arc::new(test_cids);
    let num_workers = 100;
    let test_duration = Duration::from_secs(60);

    let start_time = Instant::now();
    let mut handles = Vec::new();

    for worker_id in 0..num_workers {
        let storage_clone = storage.clone();
        let metrics_clone = metrics.clone();
        let cids_clone = test_cids.clone();

        let handle = tokio::spawn(async move {
            let mut operation_count = 0;
            let mut rng = rand::thread_rng();

            while start_time.elapsed() < test_duration {
                use rand::Rng;
                let idx = rng.gen_range(0..cids_clone.len());
                let (cid, expected_size) = &cids_clone[idx];

                let op_start = Instant::now();
                let result = storage_clone.get::<TestContent>(cid).await;
                let latency = op_start.elapsed();

                let success = result.is_ok();
                if let Ok(content) = result {
                    assert_eq!(content.data.len(), *expected_size,
                        "Retrieved data size should match");
                }

                metrics_clone.record_read(*expected_size, latency, success).await;
                operation_count += 1;
            }

            operation_count
        });

        handles.push(handle);
    }

    // Wait for all workers to complete
    let _operations_per_worker: Vec<usize> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    let total_duration = start_time.elapsed();
    metrics.print_summary(total_duration).await;

    // Verify performance targets
    let total_ops = metrics.total_operations.load(Ordering::Relaxed);
    let ops_per_second = total_ops as f64 / total_duration.as_secs_f64();

    assert!(ops_per_second > 1000.0,
        "Read throughput should exceed 1000 ops/sec, got {:.2}", ops_per_second);
}

#[tokio::test]
#[ignore] // Run manually - requires NATS server and extended runtime
async fn test_mixed_workload() {
    /// Test mixed read/write workload
    ///
    /// Given: 80% reads, 20% writes
    /// When: Continuous mixed operations
    /// Then: Measure overall performance

    let context = TestContext::new().await
        .expect("Test context creation should succeed");
    let storage = Arc::new(context.storage);
    let metrics = Arc::new(PerformanceMetrics::default());

    // Shared CID storage for reads
    let stored_cids = Arc::new(RwLock::new(Vec::new()));

    let num_workers = 50;
    let test_duration = Duration::from_secs(60);
    let read_ratio = 0.8; // 80% reads

    let start_time = Instant::now();
    let mut handles = Vec::new();

    for worker_id in 0..num_workers {
        let storage_clone = storage.clone();
        let metrics_clone = metrics.clone();
        let cids_clone = stored_cids.clone();

        let handle = tokio::spawn(async move {
            let mut operation_count = 0;
            let mut rng = rand::thread_rng();

            while start_time.elapsed() < test_duration {
                use rand::Rng;
                let is_read = rng.gen::<f64>() < read_ratio;

                if is_read {
                    // Read operation
                    let cids = cids_clone.read().await;
                    if !cids.is_empty() {
                        let idx = rng.gen_range(0..cids.len());
                        let (cid, size) = cids[idx];
                        drop(cids); // Release read lock

                        let op_start = Instant::now();
                        let result = storage_clone.get::<TestContent>(&cid).await;
                        let latency = op_start.elapsed();

                        metrics_clone.record_read(size, latency, result.is_ok()).await;
                    }
                } else {
                    // Write operation
                    let size = [1024, 10240, 102400][rng.gen_range(0..3)];
                    let content = TestContent {
                        data: generate_test_data(size),
                        metadata: vec![
                            ("worker_id".to_string(), worker_id.to_string()),
                            ("operation".to_string(), operation_count.to_string()),
                        ].into_iter().collect(),
                    };

                    let op_start = Instant::now();
                    let result = storage_clone.put(&content).await;
                    let latency = op_start.elapsed();

                    if let Ok(cid) = result {
                        cids_clone.write().await.push((cid, size));
                        metrics_clone.record_write(size, latency, true).await;
                    } else {
                        metrics_clone.record_write(size, latency, false).await;
                    }
                }

                operation_count += 1;

                // Small delay to prevent overwhelming
                tokio::time::sleep(Duration::from_millis(5)).await;
            }

            operation_count
        });

        handles.push(handle);
    }

    // Wait for all workers to complete
    let _operations_per_worker: Vec<usize> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    let total_duration = start_time.elapsed();
    metrics.print_summary(total_duration).await;

    // Additional mixed workload statistics
    let write_count = metrics.write_latencies.read().await.len();
    let read_count = metrics.read_latencies.read().await.len();
    let total_count = write_count + read_count;

    println!("\nWorkload distribution:");
    println!("  Writes: {write_count} ({:.2}%)", (write_count as f64 / total_count as f64) * 100.0);
    println!("  Reads: {read_count} ({:.2}%)", (read_count as f64 / total_count as f64) * 100.0);
}

#[tokio::test]
#[ignore] // Run manually - requires NATS server and extended runtime
async fn test_large_scale_storage() {
    /// Test large-scale storage capacity
    ///
    /// Given: Empty system
    /// When: Store 1 million objects
    /// Then: Performance degradation < 10%

    let context = TestContext::new().await
        .expect("Test context creation should succeed");
    let storage = Arc::new(context.storage);

    let target_objects = 1_000_000;
    let batch_size = 10_000;
    let num_workers = 20;

    // Semaphore to limit concurrent operations
    let semaphore = Arc::new(Semaphore::new(num_workers));

    println!("Starting large-scale storage test: {target_objects} objects");

    let mut batch_times = Vec::new();
    let start_time = Instant::now();

    for batch_num in 0..(target_objects / batch_size) {
        let batch_start = Instant::now();
        let mut handles = Vec::new();

        for i in 0..batch_size {
            let storage_clone = storage.clone();
            let semaphore_clone = semaphore.clone();
            let object_id = batch_num * batch_size + i;

            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();

                let content = TestContent {
                    data: format!("Object {object_id}").into_bytes(),
                    metadata: vec![
                        ("object_id".to_string(), object_id.to_string()),
                        ("batch".to_string(), batch_num.to_string()),
                    ].into_iter().collect(),
                };

                storage_clone.put(&content).await
            });

            handles.push(handle);
        }

        // Wait for batch to complete
        let results: Vec<_> = futures::future::join_all(handles).await;
        let successful = results.iter().filter(|r| r.is_ok()).count();

        let batch_duration = batch_start.elapsed();
        batch_times.push(batch_duration);

        println!("Batch {batch_num}: {batch_size} objects in {:?} ({batch_duration} successful)", successful);

        // Check for performance degradation
        if batch_num > 0 {
            let first_batch_time = batch_times[0].as_secs_f64();
            let current_batch_time = batch_duration.as_secs_f64();
            let degradation = ((current_batch_time - first_batch_time) / first_batch_time) * 100.0;

            if degradation > 10.0 {
                println!("WARNING: Performance degradation: {:.2}%", degradation);
            }
        }
    }

    let total_duration = start_time.elapsed();
    let avg_batch_time = batch_times.iter().sum::<Duration>() / batch_times.len() as u32;

    println!("\n=== Large Scale Storage Summary ===");
    println!("Total objects: {target_objects}");
    println!("Total duration: {:?}", total_duration);
    println!("Average batch time: {:?}", avg_batch_time);
    println!("Objects per second: {:.2}",
        target_objects as f64 / total_duration.as_secs_f64());

    // Verify performance degradation
    let first_batch = batch_times[0].as_secs_f64();
    let last_batch = batch_times.last().unwrap().as_secs_f64();
    let degradation = ((last_batch - first_batch) / first_batch) * 100.0;

    assert!(degradation < 10.0,
        "Performance degradation should be less than 10%, got {:.2}%", degradation);
}

#[tokio::test]
#[ignore] // Run manually - requires NATS server and extended runtime
async fn test_burst_traffic() {
    /// Test system behavior under burst traffic
    ///
    /// Given: Normal load with periodic bursts
    /// When: Traffic spikes 10x normal
    /// Then: System handles gracefully

    let context = TestContext::new().await
        .expect("Test context creation should succeed");
    let storage = Arc::new(context.storage);
    let metrics = Arc::new(PerformanceMetrics::default());

    let normal_workers = 10;
    let burst_workers = 100;
    let test_duration = Duration::from_secs(120);
    let burst_interval = Duration::from_secs(30);
    let burst_duration = Duration::from_secs(5);

    let start_time = Instant::now();
    let mut handles = Vec::new();

    // Normal load workers
    for worker_id in 0..normal_workers {
        let storage_clone = storage.clone();
        let metrics_clone = metrics.clone();

        let handle = tokio::spawn(async move {
            while start_time.elapsed() < test_duration {
                let content = TestContent {
                    data: generate_test_data(10240), // 10KB
                    metadata: vec![
                        ("worker_type".to_string(), "normal".to_string()),
                        ("worker_id".to_string(), worker_id.to_string()),
                    ].into_iter().collect(),
                };

                let op_start = Instant::now();
                let result = storage_clone.put(&content).await;
                let latency = op_start.elapsed();

                metrics_clone.record_write(10240, latency, result.is_ok()).await;

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        handles.push(handle);
    }

    // Burst traffic controller
    let burst_handle = tokio::spawn(async move {
        let mut burst_count = 0;

        while start_time.elapsed() < test_duration {
            tokio::time::sleep(burst_interval).await;

            if start_time.elapsed() >= test_duration {
                break;
            }

            burst_count += 1;
            println!("Starting burst #{burst_count} at {:?}", start_time.elapsed());

            // Create burst workers
            let burst_start = Instant::now();
            let mut burst_handles = Vec::new();

            for worker_id in 0..burst_workers {
                let storage_clone = storage.clone();
                let metrics_clone = metrics.clone();

                let handle = tokio::spawn(async move {
                    while burst_start.elapsed() < burst_duration {
                        let content = TestContent {
                            data: generate_test_data(1024), // 1KB for burst
                            metadata: vec![
                                ("worker_type".to_string(), "burst".to_string()),
                                ("worker_id".to_string(), worker_id.to_string()),
                                ("burst_num".to_string(), burst_count.to_string()),
                            ].into_iter().collect(),
                        };

                        let op_start = Instant::now();
                        let result = storage_clone.put(&content).await;
                        let latency = op_start.elapsed();

                        metrics_clone.record_write(1024, latency, result.is_ok()).await;
                    }
                });

                burst_handles.push(handle);
            }

            // Wait for burst to complete
            futures::future::join_all(burst_handles).await;
            println!("Burst #{burst_count} completed");
        }

        burst_count
    });

    // Wait for all workers to complete
    futures::future::join_all(handles).await;
    let burst_count = burst_handle.await.unwrap();

    let total_duration = start_time.elapsed();
    metrics.print_summary(total_duration).await;

    println!("\nBurst traffic summary:");
    println!("  Total bursts: {burst_count}");
    println!("  Burst workers: {burst_workers}");
    println!("  Normal workers: {normal_workers}");

    // Verify system handled bursts
    let failure_rate = metrics.failed_operations.load(Ordering::Relaxed) as f64 /
        metrics.total_operations.load(Ordering::Relaxed) as f64;

    assert!(failure_rate < 0.05,
        "Failure rate should be less than 5% even with bursts, got {:.2}%",
        failure_rate * 100.0);
}
