//! Simple demo showing how to load and visualize existing files
//!
//! This example shows how to:
//! - Load JSON graph files
//! - Load Nix dependency graphs
//! - Load Markdown document structures
//! - Apply automatic layouts

use alchemist::{
    graph_parser,
    graph_components::*,
    graph_algorithms::*,
    shell::AlchemistShell,
    config::AlchemistConfig,
};
use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("alchemist=info")
        .init();

    // Get file path from command line
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <file_path>", args[0]);
        println!();
        println!("Supported formats:");
        println!("  - JSON graph files (.json)");
        println!("  - Nix files (.nix) - extracts package dependencies");
        println!("  - Markdown files (.md) - extracts document structure");
        return Ok(());
    }

    let file_path = &args[1];
    println!("Loading graph from: {}", file_path);

    // Read file
    let content = std::fs::read_to_string(file_path)?;
    
    // Parse based on extension
    let (nodes, edges) = if file_path.ends_with(".json") {
        println!("Detected JSON format");
        graph_parser::parse_json_graph(&content)?
    } else if file_path.ends_with(".nix") {
        println!("Detected Nix format");
        graph_parser::parse_nix_graph(&content)?
    } else if file_path.ends_with(".md") {
        println!("Detected Markdown format");
        graph_parser::parse_markdown_graph(&content)?
    } else {
        println!("Unknown format, trying JSON parser");
        graph_parser::parse_json_graph(&content)?
    };

    println!("\n📊 Graph Statistics:");
    println!("  Nodes: {}", nodes.len());
    println!("  Edges: {}", edges.len());

    // Show sample nodes
    println!("\n🔍 Sample Nodes:");
    for (i, node) in nodes.iter().take(5).enumerate() {
        println!("  {}: {} ({})", i + 1, node.label, node.id);
    }
    if nodes.len() > 5 {
        println!("  ... and {} more", nodes.len() - 5);
    }

    // Show sample edges
    if !edges.is_empty() {
        println!("\n🔗 Sample Edges:");
        for (i, edge) in edges.iter().take(5).enumerate() {
            let label = edge.label.as_ref()
                .map(|l| format!(" [{}]", l))
                .unwrap_or_default();
            println!("  {}: {} -> {}{}", i + 1, edge.source_id, edge.target_id, label);
        }
        if edges.len() > 5 {
            println!("  ... and {} more", edges.len() - 5);
        }
    }

    // Analyze graph structure
    println!("\n🧩 Graph Structure Analysis:");
    
    // Simple component detection (without Bevy ECS)
    let mut adjacency: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    
    // Build adjacency list
    for node in &nodes {
        adjacency.insert(node.id.clone(), Vec::new());
    }
    
    for edge in &edges {
        if let Some(neighbors) = adjacency.get_mut(&edge.source_id) {
            neighbors.push(edge.target_id.clone());
        }
        // For undirected analysis
        if let Some(neighbors) = adjacency.get_mut(&edge.target_id) {
            neighbors.push(edge.source_id.clone());
        }
    }
    
    // Find connected components
    let mut visited = std::collections::HashSet::new();
    let mut components = Vec::new();
    
    for node in &nodes {
        if !visited.contains(&node.id) {
            let mut component = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            
            queue.push_back(node.id.clone());
            visited.insert(node.id.clone());
            
            while let Some(current) = queue.pop_front() {
                component.insert(current.clone());
                
                if let Some(neighbors) = adjacency.get(&current) {
                    for neighbor in neighbors {
                        if !visited.contains(neighbor) {
                            visited.insert(neighbor.clone());
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
            
            components.push(component);
        }
    }
    
    println!("  Connected components: {}", components.len());
    for (i, component) in components.iter().enumerate() {
        println!("    Component {}: {} nodes", i + 1, component.len());
    }
    
    // Check if it's a tree
    let is_tree = edges.len() == nodes.len().saturating_sub(1) && components.len() == 1;
    if is_tree {
        println!("  Graph type: Tree");
    } else if edges.len() > nodes.len() {
        println!("  Graph type: Cyclic");
    } else {
        println!("  Graph type: Forest or disconnected");
    }
    
    // Save as different format
    if args.len() > 2 {
        let output_path = &args[2];
        println!("\n💾 Saving to: {}", output_path);
        
        let output = serde_json::json!({
            "nodes": nodes,
            "edges": edges,
            "metadata": {
                "source": file_path,
                "node_count": nodes.len(),
                "edge_count": edges.len(),
                "component_count": components.len(),
                "is_tree": is_tree,
            }
        });
        
        std::fs::write(output_path, serde_json::to_string_pretty(&output)?)?;
        println!("✅ Saved successfully!");
    }

    Ok(())
}