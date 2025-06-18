//! Simple benchmark comparing HashMap vs Vec performance for graph-like operations

use std::collections::HashMap;
use std::time::Instant;

fn main() {
    println!("Simple Graph Performance Comparison");
    println!("===================================");
    println!("Creating 10,000 nodes and 5,000 edges\n");

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
    println!("  - Total time: {:.3}s", hashmap_time.as_secs_f64());

    println!("\nVec Implementation:");
    println!("  - Total time: {:.3}s", vec_time.as_secs_f64());

    let speedup = hashmap_time.as_secs_f64() / vec_time.as_secs_f64();
    if speedup > 1.0 {
        println!("\nVec is {:.2}x faster than HashMap", speedup);
    } else {
        println!("\nHashMap is {:.2}x faster than Vec", 1.0 / speedup);
    }
}

fn benchmark_hashmap() -> std::time::Duration {
    let start = Instant::now();

    // Simple graph structure
    let mut nodes: HashMap<usize, String> = HashMap::new();
    let mut edges: HashMap<(usize, usize), String> = HashMap::new();

    // Add nodes
    let node_start = Instant::now();
    for i in 0..10_000 {
        nodes.insert(i, format!("Person_{}", i));
    }
    println!(
        "  - Added 10,000 nodes in {:.3}s",
        node_start.elapsed().as_secs_f64()
    );

    // Add edges (simple pattern: each node connects to next 5)
    let edge_start = Instant::now();
    let mut edge_count = 0;

    for i in 0..10_000 {
        for j in 1..=5 {
            let target = (i + j) % 10_000;
            if edge_count < 5_000 {
                edges.insert((i, target), format!("edge_{}", edge_count));
                edge_count += 1;
            }
        }
        if edge_count >= 5_000 {
            break;
        }
    }

    println!(
        "  - Added 5,000 edges in {:.3}s",
        edge_start.elapsed().as_secs_f64()
    );

    // Simple query: find all neighbors of node 0
    let query_start = Instant::now();
    let mut neighbors = Vec::new();
    for ((from, to), _) in &edges {
        if *from == 0 {
            neighbors.push(*to);
        }
    }
    println!(
        "  - Found {} neighbors of node 0 in {:.6}s",
        neighbors.len(),
        query_start.elapsed().as_secs_f64()
    );

    start.elapsed()
}

fn benchmark_vec() -> std::time::Duration {
    let start = Instant::now();

    // Vec-based graph structure
    let mut nodes: Vec<String> = Vec::with_capacity(10_000);
    let mut edges: Vec<(usize, usize, String)> = Vec::with_capacity(5_000);

    // Add nodes
    let node_start = Instant::now();
    for i in 0..10_000 {
        nodes.push(format!("Person_{}", i));
    }
    println!(
        "  - Added 10,000 nodes in {:.3}s",
        node_start.elapsed().as_secs_f64()
    );

    // Add edges
    let edge_start = Instant::now();
    let mut edge_count = 0;

    for i in 0..10_000 {
        for j in 1..=5 {
            let target = (i + j) % 10_000;
            if edge_count < 5_000 {
                edges.push((i, target, format!("edge_{}", edge_count)));
                edge_count += 1;
            }
        }
        if edge_count >= 5_000 {
            break;
        }
    }

    println!(
        "  - Added 5,000 edges in {:.3}s",
        edge_start.elapsed().as_secs_f64()
    );

    // Simple query: find all neighbors of node 0
    let query_start = Instant::now();
    let mut neighbors = Vec::new();
    for (from, to, _) in &edges {
        if *from == 0 {
            neighbors.push(*to);
        }
    }
    println!(
        "  - Found {} neighbors of node 0 in {:.6}s",
        neighbors.len(),
        query_start.elapsed().as_secs_f64()
    );

    start.elapsed()
}
