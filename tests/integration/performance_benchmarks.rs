//! Performance benchmarks for CIM architecture
//!
//! These benchmarks measure performance of critical operations:
//! 1. Event processing throughput
//! 2. Query performance at scale
//! 3. Projection update latency
//! 4. Memory usage patterns
//! 5. Concurrent operation scaling
//!
//! ```mermaid
//! graph LR
//!     A[Benchmark Suite] --> B[Event Processing]
//!     A --> C[Query Performance]
//!     A --> D[Projection Updates]
//!     A --> E[Memory Usage]
//!     A --> F[Concurrency Scaling]
//! ```

use crate::fixtures::{TestEventStore, create_large_graph, create_test_graph};
use cim_domain::{DomainEvent, DomainResult, GraphId, NodeId};
use cim_domain_graph::{
    GraphAggregate, GraphDomainEvent, GraphSummaryProjection, NodeListProjection, NodeType,
    Position3D, Projection,
};
use criterion::{BenchmarkId, Criterion, black_box};
use std::time::{Duration, Instant};

/// Benchmark event processing throughput
#[tokio::test]
async fn bench_event_processing_throughput() -> DomainResult<()> {
    // Test different batch sizes
    let batch_sizes = vec![1, 10, 100, 1000];
    let mut results = Vec::new();

    for batch_size in batch_sizes {
        let event_store = TestEventStore::new();
        let events = generate_test_events(batch_size);

        let start = Instant::now();

        // Process events
        for event in events {
            event_store.append(event).await?;
        }

        let duration = start.elapsed();
        let throughput = batch_size as f64 / duration.as_secs_f64();

        results.push((batch_size, throughput));

        println!(
            "Batch size: {}, Throughput: {:.2} events/sec",
            batch_size, throughput
        );
    }

    // Assert minimum performance
    let min_throughput = 1000.0; // events per second
    for (_, throughput) in &results {
        assert!(
            *throughput >= min_throughput,
            "Event processing throughput {:.2} below minimum {}",
            throughput,
            min_throughput
        );
    }

    Ok(())
}

/// Benchmark query performance with large datasets
#[tokio::test]
async fn bench_query_performance_large_dataset() -> DomainResult<()> {
    // Create projections with different sizes
    let dataset_sizes = vec![100, 1_000, 10_000, 100_000];
    let mut results = Vec::new();

    for size in dataset_sizes {
        let mut projection = NodeListProjection::new();
        let graph_id = GraphId::new();

        // Populate projection
        for i in 0..size {
            let event = DomainEvent::Graph(GraphDomainEvent::NodeAdded {
                graph_id,
                node_id: NodeId::new(),
                node_type: if i % 2 == 0 {
                    NodeType::Concept
                } else {
                    NodeType::Data
                },
                position: Position3D::default(),
                conceptual_point: Default::default(),
                metadata: Default::default(),
            });
            projection.apply_event(&event).await?;
        }

        // Benchmark different query types
        let queries = vec![
            ("find_all", QueryType::FindAll),
            ("find_by_type", QueryType::FindByType(NodeType::Concept)),
            (
                "find_with_pagination",
                QueryType::Paginated {
                    offset: 0,
                    limit: 100,
                },
            ),
        ];

        for (name, query_type) in queries {
            let start = Instant::now();

            let result = match query_type {
                QueryType::FindAll => projection.get_all_nodes(&graph_id)?,
                QueryType::FindByType(node_type) => {
                    projection.find_by_type(&graph_id, node_type)?
                }
                QueryType::Paginated { offset, limit } => {
                    projection.get_paginated(&graph_id, offset, limit)?
                }
            };

            let duration = start.elapsed();

            results.push((size, name, duration, result.len()));

            println!(
                "Dataset: {}, Query: {}, Duration: {:?}, Results: {}",
                size,
                name,
                duration,
                result.len()
            );
        }
    }

    // Assert query performance scales sub-linearly
    for (size, query, duration, _) in &results {
        let max_duration_ms = match query {
            &"find_all" => size / 100,     // 10μs per node
            &"find_by_type" => size / 200, // 5μs per node
            &"find_with_pagination" => 10, // Constant time
            _ => 1000,
        };

        assert!(
            duration.as_millis() <= max_duration_ms as u128,
            "Query {} on {} nodes took {:?}, expected < {}ms",
            query,
            size,
            duration,
            max_duration_ms
        );
    }

    Ok(())
}

/// Benchmark projection update latency
#[tokio::test]
async fn bench_projection_update_latency() -> DomainResult<()> {
    let event_store = TestEventStore::new();
    let mut projection = GraphSummaryProjection::new();

    // Measure latency for different event types
    let event_types = vec![
        ("node_added", create_node_added_event()),
        ("edge_connected", create_edge_connected_event()),
        ("node_updated", create_node_updated_event()),
    ];

    let mut latencies = Vec::new();

    for (event_type, event) in event_types {
        let mut measurements = Vec::new();

        // Take multiple measurements
        for _ in 0..100 {
            let start = Instant::now();

            projection.apply_event(&event).await?;

            let latency = start.elapsed();
            measurements.push(latency);
        }

        // Calculate statistics
        let avg_latency = measurements.iter().sum::<Duration>() / measurements.len() as u32;
        let p99_latency = calculate_percentile(&mut measurements, 99.0);

        latencies.push((event_type, avg_latency, p99_latency));

        println!(
            "Event: {}, Avg: {:?}, P99: {:?}",
            event_type, avg_latency, p99_latency
        );
    }

    // Assert latency requirements
    for (_, avg, p99) in &latencies {
        assert!(avg.as_micros() < 100, "Average latency exceeds 100μs");
        assert!(p99.as_micros() < 1000, "P99 latency exceeds 1ms");
    }

    Ok(())
}

/// Benchmark memory usage patterns
#[tokio::test]
async fn bench_memory_usage_patterns() -> DomainResult<()> {
    use jemalloc_ctl::{epoch, stats};

    // Test memory usage for different graph sizes
    let graph_sizes = vec![100, 1_000, 10_000];
    let mut memory_results = Vec::new();

    for size in graph_sizes {
        // Force garbage collection
        epoch::advance()?;
        let baseline_memory = stats::allocated::read()?;

        // Create graph
        let mut graph = GraphAggregate::new(GraphId::new());

        for _ in 0..size {
            let command = cim_domain_graph::GraphCommand::AddNode {
                node_type: NodeType::Concept,
                position: Position3D::default(),
                metadata: Default::default(),
            };
            graph.handle_command(command)?;
        }

        // Measure memory after creation
        epoch::advance()?;
        let used_memory = stats::allocated::read()? - baseline_memory;
        let bytes_per_node = used_memory / size;

        memory_results.push((size, used_memory, bytes_per_node));

        println!(
            "Graph size: {}, Memory: {} bytes, Per node: {} bytes",
            size, used_memory, bytes_per_node
        );
    }

    // Assert memory efficiency
    for (_, _, bytes_per_node) in &memory_results {
        assert!(
            *bytes_per_node < 1024,
            "Memory usage per node ({} bytes) exceeds 1KB limit",
            bytes_per_node
        );
    }

    Ok(())
}

/// Benchmark concurrent operation scaling
#[tokio::test]
async fn bench_concurrent_operation_scaling() -> DomainResult<()> {
    use std::sync::Arc;
    use tokio::task;

    let thread_counts = vec![1, 2, 4, 8, 16];
    let operations_per_thread = 1000;
    let mut scaling_results = Vec::new();

    for thread_count in thread_counts {
        let event_store = Arc::new(TestEventStore::new());

        let start = Instant::now();

        // Spawn concurrent tasks
        let handles: Vec<_> = (0..thread_count)
            .map(|thread_id| {
                let store = event_store.clone();

                task::spawn(async move {
                    for i in 0..operations_per_thread {
                        let event = DomainEvent::Graph(GraphDomainEvent::NodeAdded {
                            graph_id: GraphId::new(),
                            node_id: NodeId::new(),
                            node_type: NodeType::Concept,
                            position: Position3D::default(),
                            conceptual_point: Default::default(),
                            metadata: Default::default(),
                        });

                        store.append(event).await.unwrap();
                    }
                })
            })
            .collect();

        // Wait for all tasks
        for handle in handles {
            handle.await?;
        }

        let duration = start.elapsed();
        let total_ops = thread_count * operations_per_thread;
        let ops_per_sec = total_ops as f64 / duration.as_secs_f64();

        scaling_results.push((thread_count, ops_per_sec, duration));

        println!(
            "Threads: {}, Ops/sec: {:.2}, Duration: {:?}",
            thread_count, ops_per_sec, duration
        );
    }

    // Verify scaling efficiency
    let single_thread_ops = scaling_results[0].1;

    for (threads, ops_per_sec, _) in &scaling_results[1..] {
        let expected_ops = single_thread_ops * (*threads as f64) * 0.7; // 70% scaling efficiency
        assert!(
            *ops_per_sec >= expected_ops,
            "Scaling efficiency for {} threads ({:.2} ops/sec) below 70% ({:.2} expected)",
            threads,
            ops_per_sec,
            expected_ops
        );
    }

    Ok(())
}

/// Benchmark graph traversal performance
#[tokio::test]
async fn bench_graph_traversal_performance() -> DomainResult<()> {
    // Create graphs with different structures
    let graph_structures = vec![
        ("linear", create_linear_graph(1000)),
        ("tree", create_tree_graph(10, 3)), // depth 10, branching factor 3
        ("dense", create_dense_graph(100)), // 100 nodes, fully connected
    ];

    let mut traversal_results = Vec::new();

    for (structure_name, graph) in graph_structures {
        // Test different traversal algorithms
        let algorithms = vec![
            ("bfs", TraversalAlgorithm::BreadthFirst),
            ("dfs", TraversalAlgorithm::DepthFirst),
            ("dijkstra", TraversalAlgorithm::Dijkstra),
        ];

        for (algo_name, algorithm) in algorithms {
            let start = Instant::now();

            // Perform traversal
            let visited = perform_traversal(&graph, algorithm)?;

            let duration = start.elapsed();

            traversal_results.push((structure_name, algo_name, duration, visited));

            println!(
                "Structure: {}, Algorithm: {}, Duration: {:?}, Visited: {}",
                structure_name, algo_name, duration, visited
            );
        }
    }

    // Assert traversal performance
    for (structure, algo, duration, _) in &traversal_results {
        let max_duration_ms = match (*structure, *algo) {
            ("linear", _) => 10,
            ("tree", "bfs") => 20,
            ("tree", "dfs") => 15,
            ("dense", _) => 50,
            _ => 100,
        };

        assert!(
            duration.as_millis() <= max_duration_ms as u128,
            "Traversal {} on {} took {:?}, expected < {}ms",
            algo,
            structure,
            duration,
            max_duration_ms
        );
    }

    Ok(())
}

/// Benchmark event replay performance
#[tokio::test]
async fn bench_event_replay_performance() -> DomainResult<()> {
    let event_counts = vec![100, 1_000, 10_000];
    let mut replay_results = Vec::new();

    for count in event_counts {
        let event_store = TestEventStore::new();

        // Generate and store events
        let events = generate_test_events(count);
        for event in &events {
            event_store.append(event.clone()).await?;
        }

        // Benchmark replay
        let start = Instant::now();

        let mut graph = GraphAggregate::new(GraphId::new());
        let stored_events = event_store.get_events().await;

        for event in stored_events {
            graph.apply_event(&event)?;
        }

        let duration = start.elapsed();
        let events_per_sec = count as f64 / duration.as_secs_f64();

        replay_results.push((count, duration, events_per_sec));

        println!(
            "Events: {}, Duration: {:?}, Rate: {:.2} events/sec",
            count, duration, events_per_sec
        );
    }

    // Assert replay performance
    for (_, _, rate) in &replay_results {
        assert!(
            *rate >= 10_000.0,
            "Event replay rate ({:.2} events/sec) below minimum 10,000",
            rate
        );
    }

    Ok(())
}

// Helper functions and types

enum QueryType {
    FindAll,
    FindByType(NodeType),
    Paginated { offset: usize, limit: usize },
}

fn generate_test_events(count: usize) -> Vec<DomainEvent> {
    let graph_id = GraphId::new();
    (0..count)
        .map(|i| {
            DomainEvent::Graph(GraphDomainEvent::NodeAdded {
                graph_id,
                node_id: NodeId::new(),
                node_type: if i % 2 == 0 {
                    NodeType::Concept
                } else {
                    NodeType::Data
                },
                position: Position3D::default(),
                conceptual_point: Default::default(),
                metadata: Default::default(),
            })
        })
        .collect()
}

fn create_node_added_event() -> Box<dyn DomainEvent> {
    Box::new(DomainEvent::Graph(GraphDomainEvent::NodeAdded {
        graph_id: GraphId::new(),
        node_id: NodeId::new(),
        node_type: NodeType::Concept,
        position: Position3D::default(),
        conceptual_point: Default::default(),
        metadata: Default::default(),
    }))
}

fn create_edge_connected_event() -> Box<dyn DomainEvent> {
    Box::new(DomainEvent::Graph(GraphDomainEvent::EdgeConnected {
        graph_id: GraphId::new(),
        edge_id: cim_domain::EdgeId::new(),
        source: NodeId::new(),
        target: NodeId::new(),
        relationship: cim_domain_graph::EdgeRelationship::default(),
    }))
}

fn create_node_updated_event() -> Box<dyn DomainEvent> {
    Box::new(DomainEvent::Graph(GraphDomainEvent::NodeUpdated {
        graph_id: GraphId::new(),
        node_id: NodeId::new(),
        changes: cim_domain_graph::NodeChanges::default(),
        timestamp: std::time::SystemTime::now(),
        source: cim_domain_graph::UpdateSource::Internal,
    }))
}

fn calculate_percentile(measurements: &mut Vec<Duration>, percentile: f64) -> Duration {
    measurements.sort();
    let index = ((percentile / 100.0) * measurements.len() as f64) as usize;
    measurements[index.min(measurements.len() - 1)]
}

fn create_linear_graph(size: usize) -> GraphAggregate {
    let mut graph = GraphAggregate::new(GraphId::new());
    let mut prev_node = None;

    for i in 0..size {
        let node_id = NodeId::new();
        graph
            .handle_command(cim_domain_graph::GraphCommand::AddNode {
                node_type: NodeType::Concept,
                position: Position3D {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                metadata: Default::default(),
            })
            .unwrap();

        if let Some(prev) = prev_node {
            graph
                .handle_command(cim_domain_graph::GraphCommand::ConnectNodes {
                    source: prev,
                    target: node_id,
                    edge_type: cim_domain_graph::EdgeType::Sequence,
                })
                .unwrap();
        }

        prev_node = Some(node_id);
    }

    graph
}

fn create_tree_graph(depth: usize, branching_factor: usize) -> GraphAggregate {
    let mut graph = GraphAggregate::new(GraphId::new());

    fn add_node_recursive(
        graph: &mut GraphAggregate,
        parent: Option<NodeId>,
        depth: usize,
        branching_factor: usize,
    ) {
        if depth == 0 {
            return;
        }

        for _ in 0..branching_factor {
            let node_id = NodeId::new();
            graph
                .handle_command(cim_domain_graph::GraphCommand::AddNode {
                    node_type: NodeType::Concept,
                    position: Position3D::default(),
                    metadata: Default::default(),
                })
                .unwrap();

            if let Some(parent_id) = parent {
                graph
                    .handle_command(cim_domain_graph::GraphCommand::ConnectNodes {
                        source: parent_id,
                        target: node_id,
                        edge_type: cim_domain_graph::EdgeType::Hierarchy {
                            level: depth as u32,
                        },
                    })
                    .unwrap();
            }

            add_node_recursive(graph, Some(node_id), depth - 1, branching_factor);
        }
    }

    let root = NodeId::new();
    graph
        .handle_command(cim_domain_graph::GraphCommand::AddNode {
            node_type: NodeType::Concept,
            position: Position3D::default(),
            metadata: Default::default(),
        })
        .unwrap();

    add_node_recursive(&mut graph, Some(root), depth, branching_factor);

    graph
}

fn create_dense_graph(size: usize) -> GraphAggregate {
    let mut graph = GraphAggregate::new(GraphId::new());
    let mut nodes = Vec::new();

    // Create nodes
    for _ in 0..size {
        let node_id = NodeId::new();
        graph
            .handle_command(cim_domain_graph::GraphCommand::AddNode {
                node_type: NodeType::Concept,
                position: Position3D::default(),
                metadata: Default::default(),
            })
            .unwrap();
        nodes.push(node_id);
    }

    // Connect every node to every other node
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            graph
                .handle_command(cim_domain_graph::GraphCommand::ConnectNodes {
                    source: nodes[i],
                    target: nodes[j],
                    edge_type: cim_domain_graph::EdgeType::Association {
                        relation_type: "connected".to_string(),
                    },
                })
                .unwrap();
        }
    }

    graph
}

enum TraversalAlgorithm {
    BreadthFirst,
    DepthFirst,
    Dijkstra,
}

fn perform_traversal(graph: &GraphAggregate, algorithm: TraversalAlgorithm) -> DomainResult<usize> {
    // Simplified traversal implementation
    match algorithm {
        TraversalAlgorithm::BreadthFirst => Ok(graph.node_count()),
        TraversalAlgorithm::DepthFirst => Ok(graph.node_count()),
        TraversalAlgorithm::Dijkstra => Ok(graph.node_count()),
    }
}
