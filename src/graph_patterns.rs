use crate::graph::AlchemistGraph;
use uuid::Uuid;
use rand::Rng;

pub enum GraphPattern {
    Tree { branch_factor: usize, depth: usize },
    Star { points: usize },
    Cycle { nodes: usize },
    Complete { nodes: usize },
    Grid { width: usize, height: usize },
    Random { nodes: usize, edge_probability: f32 },
}

pub fn generate_pattern(pattern: GraphPattern) -> AlchemistGraph {
    let mut graph = AlchemistGraph::new();
    
    match pattern {
        GraphPattern::Tree { branch_factor, depth } => {
            generate_tree(&mut graph, branch_factor, depth)
        },
        GraphPattern::Star { points } => {
            generate_star(&mut graph, points)
        },
        GraphPattern::Cycle { nodes } => {
            generate_cycle(&mut graph, nodes)
        },
        GraphPattern::Complete { nodes } => {
            generate_complete(&mut graph, nodes)
        },
        GraphPattern::Grid { width, height } => {
            generate_grid(&mut graph, width, height)
        },
        GraphPattern::Random { nodes, edge_probability } => {
            generate_random(&mut graph, nodes, edge_probability)
        },
    }
    
    graph
}

fn generate_tree(graph: &mut AlchemistGraph, branch_factor: usize, depth: usize) {
    if depth == 0 || branch_factor == 0 {
        return;
    }
    
    // Create root node
    let root_id = graph.add_node("Root", vec!["tree".to_string()]);
    
    // Recursively build the tree
    build_tree_recursive(graph, root_id, branch_factor, depth, 1);
}

fn build_tree_recursive(graph: &mut AlchemistGraph, parent_id: Uuid, branch_factor: usize, max_depth: usize, current_depth: usize) {
    if current_depth >= max_depth {
        return;
    }
    
    for i in 0..branch_factor {
        let child_id = graph.add_node(
            &format!("Node {}-{}", current_depth, i), 
            vec!["tree".to_string()]
        );
        
        // Connect child to parent
        graph.add_edge(parent_id, child_id, vec!["tree_edge".to_string()]);
        
        // Recursively build children
        build_tree_recursive(graph, child_id, branch_factor, max_depth, current_depth + 1);
    }
}

fn generate_star(graph: &mut AlchemistGraph, points: usize) {
    if points == 0 {
        return;
    }
    
    // Create center node
    let center_id = graph.add_node("Center", vec!["star".to_string(), "center".to_string()]);
    
    // Create and connect all the points
    for i in 0..points {
        let point_id = graph.add_node(
            &format!("Point {}", i),
            vec!["star".to_string(), "point".to_string()]
        );
        
        // Connect point to center
        graph.add_edge(center_id, point_id, vec!["star_edge".to_string()]);
    }
}

fn generate_cycle(graph: &mut AlchemistGraph, nodes: usize) {
    if nodes < 2 {
        return;
    }
    
    let mut node_ids = Vec::with_capacity(nodes);
    
    // Create all nodes
    for i in 0..nodes {
        let node_id = graph.add_node(
            &format!("Node {}", i),
            vec!["cycle".to_string()]
        );
        node_ids.push(node_id);
    }
    
    // Connect nodes in a cycle
    for i in 0..nodes {
        let next_i = (i + 1) % nodes;
        graph.add_edge(node_ids[i], node_ids[next_i], vec!["cycle_edge".to_string()]);
    }
}

fn generate_complete(graph: &mut AlchemistGraph, nodes: usize) {
    if nodes < 2 {
        return;
    }
    
    let mut node_ids = Vec::with_capacity(nodes);
    
    // Create all nodes
    for i in 0..nodes {
        let node_id = graph.add_node(
            &format!("Node {}", i),
            vec!["complete".to_string()]
        );
        node_ids.push(node_id);
    }
    
    // Connect each node to every other node
    for i in 0..nodes {
        for j in 0..nodes {
            if i != j {
                graph.add_edge(node_ids[i], node_ids[j], vec!["complete_edge".to_string()]);
            }
        }
    }
}

fn generate_grid(graph: &mut AlchemistGraph, width: usize, height: usize) {
    if width == 0 || height == 0 {
        return;
    }
    
    let mut node_grid = vec![vec![Uuid::nil(); width]; height];
    
    // Create grid nodes
    for y in 0..height {
        for x in 0..width {
            let node_id = graph.add_node(
                &format!("Node ({},{})", x, y),
                vec!["grid".to_string()]
            );
            node_grid[y][x] = node_id;
        }
    }
    
    // Connect horizontally
    for y in 0..height {
        for x in 0..width-1 {
            graph.add_edge(
                node_grid[y][x], 
                node_grid[y][x+1], 
                vec!["grid_edge".to_string(), "horizontal".to_string()]
            );
        }
    }
    
    // Connect vertically
    for y in 0..height-1 {
        for x in 0..width {
            graph.add_edge(
                node_grid[y][x], 
                node_grid[y+1][x], 
                vec!["grid_edge".to_string(), "vertical".to_string()]
            );
        }
    }
}

fn generate_random(graph: &mut AlchemistGraph, nodes: usize, edge_probability: f32) {
    if nodes < 2 {
        return;
    }
    
    let mut node_ids = Vec::with_capacity(nodes);
    
    // Create all nodes
    for i in 0..nodes {
        let node_id = graph.add_node(
            &format!("Node {}", i),
            vec!["random".to_string()]
        );
        node_ids.push(node_id);
    }
    
    // Create edges with probability
    let mut rng = rand::rng();
    for i in 0..nodes {
        for j in 0..nodes {
            if i != j && rng.random::<f32>() < edge_probability {
                graph.add_edge(node_ids[i], node_ids[j], vec!["random_edge".to_string()]);
            }
        }
    }
} 