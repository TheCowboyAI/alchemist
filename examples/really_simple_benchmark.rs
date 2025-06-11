//! Simple benchmark comparing HashMap vs Vec performance for graph operations

use std::collections::HashMap;
use std::time::Instant;

fn main() {
    println!("Simple Graph Performance Comparison");
    println!("===================================");
    println!("Creating 10,000 person nodes and 5,000 edges\n");

    // Test HashMap-based implementation
    println!("Testing HashMap-based Implementation:");
    let hashmap_time = benchmark_hashmap();

    // Test Vec-based implementation
    println!("\nTesting Vec-based Implementation:");
    let vec_time = benchmark_vec();

    // Compare results
    println!("\n\nResults Summary:");
    println!("================");
    println!("HashMap Implementation:");
    println!("  - Total time: {:.2}s", hashmap_time.as_secs_f64());

    println!("\nVec Implementation:");
    println!("  - Total time: {:.2}s", vec_time.as_secs_f64());

    let speedup = hashmap_time.as_secs_f64() / vec_time.as_secs_f64();
    if speedup > 1.0 {
        println!("\nVec is {:.2}x faster!", speedup);
    } else {
        println!("\nHashMap is {:.2}x faster!", 1.0 / speedup);
    }
}

// HashMap-based graph
struct HashMapGraph {
    nodes: HashMap<usize, String>,
    edges: HashMap<usize, (usize, usize)>,
}

fn benchmark_hashmap() -> std::time::Duration {
    let start = Instant::now();

    let mut graph = HashMapGraph {
        nodes: HashMap::new(),
        edges: HashMap::new(),
    };

    // Add 10,000 nodes
    let node_start = Instant::now();
    for i in 0..10_000 {
        graph.nodes.insert(i, format!("Person {}", i));
    }
    println!("  - Added 10,000 nodes in {:.2}ms", node_start.elapsed().as_millis());

    // Add 5,000 edges (sequential for reproducibility)
    let edge_start = Instant::now();
    for i in 0..5_000 {
        let source = i % 10_000;
        let target = (i + 1) % 10_000;
        graph.edges.insert(i, (source, target));
    }
    println!("  - Added 5,000 edges in {:.2}ms", edge_start.elapsed().as_millis());

    // Find neighbors of node 0
    let query_start = Instant::now();
    let mut neighbors = Vec::new();
    for (_, (source, target)) in &graph.edges {
        if *source == 0 {
            neighbors.push(*target);
        }
    }
    println!("  - Found {} neighbors of node 0 in {:.2}μs",
             neighbors.len(),
             query_start.elapsed().as_micros());

    start.elapsed()
}

// Vec-based graph (adjacency list)
struct VecGraph {
    nodes: Vec<String>,
    edges: Vec<Vec<usize>>, // adjacency list
}

fn benchmark_vec() -> std::time::Duration {
    let start = Instant::now();

    let mut graph = VecGraph {
        nodes: Vec::with_capacity(10_000),
        edges: vec![Vec::new(); 10_000],
    };

    // Add 10,000 nodes
    let node_start = Instant::now();
    for i in 0..10_000 {
        graph.nodes.push(format!("Person {}", i));
    }
    println!("  - Added 10,000 nodes in {:.2}ms", node_start.elapsed().as_millis());

    // Add 5,000 edges
    let edge_start = Instant::now();
    for i in 0..5_000 {
        let source = i % 10_000;
        let target = (i + 1) % 10_000;
        graph.edges[source].push(target);
    }
    println!("  - Added 5,000 edges in {:.2}ms", edge_start.elapsed().as_millis());

    // Find neighbors of node 0
    let query_start = Instant::now();
    let neighbors = &graph.edges[0];
    println!("  - Found {} neighbors of node 0 in {:.2}μs",
             neighbors.len(),
             query_start.elapsed().as_micros());

    start.elapsed()
}
