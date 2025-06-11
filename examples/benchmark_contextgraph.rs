//! Simple benchmark comparing ContextGraph implementations

use std::time::Instant;
use rand::Rng;

fn main() {
    println!("ContextGraph Performance Comparison");
    println!("===================================");
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
        println!("\nPetGraph is {:.2}x faster than HashMap", speedup);
    } else {
        println!("\nHashMap is {:.2}x faster than PetGraph", 1.0 / speedup);
    }
}

fn benchmark_hashmap() -> std::time::Duration {
    use std::collections::HashMap;

    let start = Instant::now();

    // Simple graph structure
    let mut nodes: HashMap<usize, String> = HashMap::new();
    let mut edges: HashMap<(usize, usize), String> = HashMap::new();

    // Add nodes
    let node_start = Instant::now();
    for i in 0..10_000 {
        nodes.insert(i, format!("Person_{}", i));
    }
    println!("  - Added 10,000 nodes in {:.3}s", node_start.elapsed().as_secs_f64());

    // Add random edges
    let edge_start = Instant::now();
    let mut rng = rand::thread_rng();
    let mut edge_count = 0;

    while edge_count < 5_000 {
        let from = rng.gen_range(0..10_000);
        let to = rng.gen_range(0..10_000);

        if from != to && !edges.contains_key(&(from, to)) {
            edges.insert((from, to), format!("knows_{}", edge_count));
            edge_count += 1;
        }
    }

    println!("  - Added 5,000 edges in {:.3}s", edge_start.elapsed().as_secs_f64());

    // Simple query: find all neighbors of node 0
    let query_start = Instant::now();
    let mut neighbors = Vec::new();
    for ((from, to), _) in &edges {
        if *from == 0 {
            neighbors.push(*to);
        }
    }
    println!("  - Found {} neighbors of node 0 in {:.6}s",
             neighbors.len(),
             query_start.elapsed().as_secs_f64());

    start.elapsed()
}

fn benchmark_petgraph() -> std::time::Duration {
    use petgraph::graph::{Graph, NodeIndex};

    let start = Instant::now();

    let mut graph = Graph::<String, String>::new();
    let mut node_indices = Vec::with_capacity(10_000);

    // Add nodes
    let node_start = Instant::now();
    for i in 0..10_000 {
        let idx = graph.add_node(format!("Person_{}", i));
        node_indices.push(idx);
    }
    println!("  - Added 10,000 nodes in {:.3}s", node_start.elapsed().as_secs_f64());

    // Add random edges
    let edge_start = Instant::now();
    let mut rng = rand::thread_rng();
    let mut edge_count = 0;

    while edge_count < 5_000 {
        let from_idx = rng.gen_range(0..10_000);
        let to_idx = rng.gen_range(0..10_000);

        if from_idx != to_idx {
            graph.add_edge(
                node_indices[from_idx],
                node_indices[to_idx],
                format!("knows_{}", edge_count)
            );
            edge_count += 1;
        }
    }

    println!("  - Added 5,000 edges in {:.3}s", edge_start.elapsed().as_secs_f64());

    // Simple query: find all neighbors of node 0
    let query_start = Instant::now();
    let neighbors: Vec<_> = graph.neighbors(node_indices[0]).collect();
    println!("  - Found {} neighbors of node 0 in {:.6}s",
             neighbors.len(),
             query_start.elapsed().as_secs_f64());

    // Bonus: Run some algorithms
    let algo_start = Instant::now();
    use petgraph::algo::{is_cyclic_directed, kosaraju_scc};

    let is_cyclic = is_cyclic_directed(&graph);
    let sccs = kosaraju_scc(&graph);

    println!("  - Graph is cyclic: {}", is_cyclic);
    println!("  - Strongly connected components: {}", sccs.len());
    println!("  - Algorithm tests took: {:.3}s", algo_start.elapsed().as_secs_f64());

    start.elapsed()
}
