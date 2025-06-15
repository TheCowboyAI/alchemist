//! Raw benchmark comparing HashMap vs PetGraph performance

use petgraph::graph::{Graph, NodeIndex};
use rand::Rng;
use std::collections::HashMap;
use std::time::Instant;

fn main() {
    println!("Raw Performance Comparison: HashMap vs PetGraph");
    println!("==============================================");
    println!("Creating 10,000 person nodes and 5,000 random edges\n");

    // Test HashMap-based implementation
    println!("Testing HashMap-based Implementation:");
    let hashmap_time = benchmark_hashmap();

    // Test PetGraph-based implementation
    println!("\nTesting PetGraph-based Implementation:");
    let petgraph_time = benchmark_petgraph();

    // Compare results
    println!("\n\nResults Summary:");
    println!("================");
    println!("HashMap Implementation:");
    println!("  - Total time: {:.2}s", hashmap_time.as_secs_f64());

    println!("\nPetGraph Implementation:");
    println!("  - Total time: {:.2}s", petgraph_time.as_secs_f64());

    let speedup = hashmap_time.as_secs_f64() / petgraph_time.as_secs_f64();
    if speedup > 1.0 {
        println!("\nPetGraph is {:.2}x faster!", speedup);
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
    let mut rng = rand::thread_rng();

    let mut graph = HashMapGraph {
        nodes: HashMap::new(),
        edges: HashMap::new(),
    };

    // Add 10,000 nodes
    let node_start = Instant::now();
    for i in 0..10_000 {
        graph.nodes.insert(i, format!("Person {}", i));
    }
    println!(
        "  - Added 10,000 nodes in {:.2}ms",
        node_start.elapsed().as_millis()
    );

    // Add 5,000 random edges
    let edge_start = Instant::now();
    for i in 0..5_000 {
        let source = rng.gen_range(0..10_000);
        let target = rng.gen_range(0..10_000);
        graph.edges.insert(i, (source, target));
    }
    println!(
        "  - Added 5,000 edges in {:.2}ms",
        edge_start.elapsed().as_millis()
    );

    // Find neighbors of node 0
    let query_start = Instant::now();
    let mut neighbors = Vec::new();
    for (_, (source, target)) in &graph.edges {
        if *source == 0 {
            neighbors.push(*target);
        }
    }
    println!(
        "  - Found {} neighbors of node 0 in {:.2}μs",
        neighbors.len(),
        query_start.elapsed().as_micros()
    );

    start.elapsed()
}

// PetGraph-based graph
fn benchmark_petgraph() -> std::time::Duration {
    let start = Instant::now();
    let mut rng = rand::thread_rng();

    let mut graph = Graph::<String, ()>::new();
    let mut node_indices = Vec::new();

    // Add 10,000 nodes
    let node_start = Instant::now();
    for i in 0..10_000 {
        let idx = graph.add_node(format!("Person {}", i));
        node_indices.push(idx);
    }
    println!(
        "  - Added 10,000 nodes in {:.2}ms",
        node_start.elapsed().as_millis()
    );

    // Add 5,000 random edges
    let edge_start = Instant::now();
    for _ in 0..5_000 {
        let source = rng.gen_range(0..10_000);
        let target = rng.gen_range(0..10_000);
        graph.add_edge(node_indices[source], node_indices[target], ());
    }
    println!(
        "  - Added 5,000 edges in {:.2}ms",
        edge_start.elapsed().as_millis()
    );

    // Find neighbors of node 0
    let query_start = Instant::now();
    let neighbors: Vec<NodeIndex> = graph.neighbors(node_indices[0]).collect();
    println!(
        "  - Found {} neighbors of node 0 in {:.2}μs",
        neighbors.len(),
        query_start.elapsed().as_micros()
    );

    start.elapsed()
}
