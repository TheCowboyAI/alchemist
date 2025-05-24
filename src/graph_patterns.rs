use crate::graph::AlchemistGraph;
use rand::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;
use bevy::prelude::*;

/// Represents catalog categories for organizing graph patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternCategory {
    Workflow,
    DataFlow,
    Architecture,
    Organization,
    Custom,
}

impl std::fmt::Display for PatternCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternCategory::Workflow => write!(f, "Workflow"),
            PatternCategory::DataFlow => write!(f, "DataFlow"),
            PatternCategory::Architecture => write!(f, "Architecture"),
            PatternCategory::Organization => write!(f, "Organization"),
            PatternCategory::Custom => write!(f, "Custom"),
        }
    }
}

/// Represents all available graph patterns
#[derive(Debug, Clone)]
pub enum GraphPattern {
    Tree {
        branch_factor: usize,
        depth: usize,
    },
    Star {
        points: usize,
    },
    Cycle {
        nodes: usize,
    },
    Complete {
        nodes: usize,
    },
    Grid {
        width: usize,
        height: usize,
    },
    Random {
        nodes: usize,
        edge_probability: f32,
    },
    // New patterns
    RegularPolygon {
        sides: usize,
    },
    // Updated state machine variants
    MooreMachine, // Outputs depend only on the current state
    MealyMachine, // Outputs depend on current state and input
    FiniteAutomaton {
        states: usize,
        alphabet_size: usize,
        is_deterministic: bool,
    },
    DirectedAcyclicGraph {
        levels: usize,
        nodes_per_level: usize,
    },
    Bipartite {
        left_nodes: usize,
        right_nodes: usize,
        edge_density: f32,
    },
}

impl GraphPattern {
    /// Get a user-friendly name for the pattern
    pub fn name(&self) -> &'static str {
        match self {
            GraphPattern::Tree { .. } => "Tree",
            GraphPattern::Star { .. } => "Star",
            GraphPattern::Cycle { .. } => "Cycle",
            GraphPattern::Complete { .. } => "Complete Graph",
            GraphPattern::Grid { .. } => "Grid",
            GraphPattern::Random { .. } => "Random Graph",
            GraphPattern::RegularPolygon { .. } => "Regular Polygon",
            GraphPattern::MooreMachine => "Moore Machine",
            GraphPattern::MealyMachine => "Mealy Machine",
            GraphPattern::FiniteAutomaton {
                is_deterministic, ..
            } => {
                if *is_deterministic {
                    "Deterministic Finite Automaton (DFA)"
                } else {
                    "Non-deterministic Finite Automaton (NFA)"
                }
            }
            GraphPattern::DirectedAcyclicGraph { .. } => "Directed Acyclic Graph (DAG)",
            GraphPattern::Bipartite { .. } => "Bipartite Graph",
        }
    }

    /// Get the category this pattern belongs to
    pub fn category(&self) -> PatternCategory {
        match self {
            GraphPattern::Tree { .. } => PatternCategory::Architecture,
            GraphPattern::Star { .. } => PatternCategory::Architecture,
            GraphPattern::Cycle { .. } => PatternCategory::Architecture,
            GraphPattern::Complete { .. } => PatternCategory::Architecture,
            GraphPattern::Grid { .. } => PatternCategory::Architecture,
            GraphPattern::Random { .. } => PatternCategory::Architecture,
            GraphPattern::RegularPolygon { .. } => PatternCategory::Architecture,
            GraphPattern::MooreMachine => PatternCategory::Architecture,
            GraphPattern::MealyMachine => PatternCategory::Architecture,
            GraphPattern::FiniteAutomaton { .. } => PatternCategory::Architecture,
            GraphPattern::DirectedAcyclicGraph { .. } => PatternCategory::Architecture,
            GraphPattern::Bipartite { .. } => PatternCategory::Architecture,
        }
    }

    /// Get a brief description of the pattern
    pub fn description(&self) -> &'static str {
        match self {
            GraphPattern::Tree { .. } => {
                "A hierarchical structure with a root node and child nodes"
            }
            GraphPattern::Star { .. } => "A central node connected to multiple other nodes",
            GraphPattern::Cycle { .. } => {
                "Nodes connected in a ring where each node has exactly two neighbors"
            }
            GraphPattern::Complete { .. } => "Every node is connected to every other node",
            GraphPattern::Grid { .. } => "A 2D lattice of nodes with connections to adjacent nodes",
            GraphPattern::Random { .. } => "Random connections between nodes based on probability",
            GraphPattern::RegularPolygon { .. } => "Nodes arranged in a regular polygon shape",
            GraphPattern::MooreMachine => {
                "State machine where outputs depend only on the current state"
            }
            GraphPattern::MealyMachine => {
                "State machine where outputs depend on both current state and inputs"
            }
            GraphPattern::FiniteAutomaton {
                is_deterministic, ..
            } => {
                if *is_deterministic {
                    "State machine that accepts/rejects strings, with exactly one transition per input"
                } else {
                    "State machine that accepts/rejects strings, possibly with multiple transitions per input"
                }
            }
            GraphPattern::DirectedAcyclicGraph { .. } => {
                "A directed graph with no cycles, often used to represent dependencies"
            }
            GraphPattern::Bipartite { .. } => {
                "A graph whose vertices can be divided into two disjoint sets"
            }
        }
    }
}

#[derive(Resource)]
pub struct PatternCatalog {
    patterns: HashMap<String, GraphPattern>,
    categories: HashMap<String, Vec<String>>, // Category -> Pattern names
}

impl Default for PatternCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternCatalog {
    /// Create a new catalog with predefined pattern examples
    pub fn new() -> Self {
        let mut catalog = Self {
            patterns: HashMap::new(),
            categories: HashMap::new(),
        };

        // Add example patterns
        catalog.add_pattern(
            "binary_tree",
            GraphPattern::Tree {
                branch_factor: 2,
                depth: 3,
            },
        );
        catalog.add_pattern("small_star", GraphPattern::Star { points: 8 });
        catalog.add_pattern("pentagon", GraphPattern::RegularPolygon { sides: 5 });
        catalog.add_pattern("hexagon", GraphPattern::RegularPolygon { sides: 6 });
        catalog.add_pattern("octagon", GraphPattern::RegularPolygon { sides: 8 });
        // Add the new state machine patterns
        catalog.add_pattern("moore_machine", GraphPattern::MooreMachine);
        catalog.add_pattern("mealy_machine", GraphPattern::MealyMachine);
        catalog.add_pattern(
            "simple_dfa",
            GraphPattern::FiniteAutomaton {
                states: 4,
                alphabet_size: 2,
                is_deterministic: true,
            },
        );
        catalog.add_pattern(
            "simple_nfa",
            GraphPattern::FiniteAutomaton {
                states: 4,
                alphabet_size: 2,
                is_deterministic: false,
            },
        );
        catalog.add_pattern(
            "small_dag",
            GraphPattern::DirectedAcyclicGraph {
                levels: 3,
                nodes_per_level: 2,
            },
        );
        catalog.add_pattern(
            "basic_bipartite",
            GraphPattern::Bipartite {
                left_nodes: 3,
                right_nodes: 3,
                edge_density: 0.7,
            },
        );
        catalog.add_pattern(
            "small_grid",
            GraphPattern::Grid {
                width: 3,
                height: 3,
            },
        );
        catalog.add_pattern("triangle", GraphPattern::Complete { nodes: 3 });

        catalog
    }

    /// Add a pattern to the catalog
    pub fn add_pattern(&mut self, key: &str, pattern: GraphPattern) {
        let category = pattern.category();
        let key_owned = key.to_string();

        // Add to categories map
        self.categories
            .entry(category.to_string())
            .or_default()
            .push(key_owned.clone());

        // Add to patterns map
        self.patterns.insert(key_owned.clone(), pattern);
    }

    /// Get a pattern by key
    pub fn get_pattern(&self, key: &str) -> Option<&GraphPattern> {
        self.patterns.get(key)
    }

    /// Get all pattern keys in a specific category
    pub fn get_keys_by_category(&self, category: PatternCategory) -> Vec<&str> {
        match self.categories.get(&category.to_string()) {
            Some(keys) => keys.iter().map(|s| s.as_str()).collect(),
            None => Vec::new(),
        }
    }

    /// Get all pattern keys
    pub fn get_all_keys(&self) -> Vec<&str> {
        self.patterns.keys().map(|s| s.as_str()).collect()
    }
}

pub fn generate_pattern(pattern: GraphPattern) -> AlchemistGraph {
    let mut graph = AlchemistGraph::new();

    match pattern {
        GraphPattern::Tree {
            branch_factor,
            depth,
        } => generate_tree(&mut graph, branch_factor, depth),
        GraphPattern::Star { points } => generate_star(&mut graph, points),
        GraphPattern::Cycle { nodes } => generate_cycle(&mut graph, nodes),
        GraphPattern::Complete { nodes } => generate_complete(&mut graph, nodes),
        GraphPattern::Grid { width, height } => generate_grid(&mut graph, width, height),
        GraphPattern::Random {
            nodes,
            edge_probability,
        } => generate_random(&mut graph, nodes, edge_probability),
        GraphPattern::RegularPolygon { sides } => generate_regular_polygon(&mut graph, sides),
        GraphPattern::MooreMachine => generate_moore_machine(&mut graph),
        GraphPattern::MealyMachine => generate_mealy_machine(&mut graph),
        GraphPattern::FiniteAutomaton {
            states,
            alphabet_size,
            is_deterministic,
        } => generate_finite_automaton(&mut graph, states, alphabet_size, is_deterministic),
        GraphPattern::DirectedAcyclicGraph {
            levels,
            nodes_per_level,
        } => generate_dag(&mut graph, levels, nodes_per_level),
        GraphPattern::Bipartite {
            left_nodes,
            right_nodes,
            edge_density,
        } => generate_bipartite(&mut graph, left_nodes, right_nodes, edge_density),
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

fn build_tree_recursive(
    graph: &mut AlchemistGraph,
    parent_id: Uuid,
    branch_factor: usize,
    max_depth: usize,
    current_depth: usize,
) {
    if current_depth >= max_depth {
        return;
    }

    for i in 0..branch_factor {
        let child_id = graph.add_node(
            &format!("Node {current_depth}-{i}"),
            vec!["tree_node".to_string()],
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
        let point_id = graph.add_node(&format!("Point {i}"), vec!["mesh_node".to_string()]);

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
        let node_id = graph.add_node(&format!("Node {i}"), vec!["wheel_node".to_string()]);
        node_ids.push(node_id);
    }

    // Connect nodes in a cycle
    for i in 0..nodes {
        let next_i = (i + 1) % nodes;
        graph.add_edge(
            node_ids[i],
            node_ids[next_i],
            vec!["cycle_edge".to_string()],
        );
    }
}

fn generate_complete(graph: &mut AlchemistGraph, nodes: usize) {
    if nodes < 2 {
        return;
    }

    let mut node_ids = Vec::with_capacity(nodes);

    // Create all nodes
    for i in 0..nodes {
        let node_id = graph.add_node(&format!("Node {i}"), vec!["star_node".to_string()]);
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
            let node_id = graph.add_node(&format!("Node ({x},{y})"), vec!["grid_node".to_string()]);
            node_grid[y][x] = node_id;
        }
    }

    // Connect horizontally
    for y in 0..height {
        for x in 0..width - 1 {
            graph.add_edge(
                node_grid[y][x],
                node_grid[y][x + 1],
                vec!["grid_edge".to_string(), "horizontal".to_string()],
            );
        }
    }

    // Connect vertically
    for y in 0..height - 1 {
        for x in 0..width {
            graph.add_edge(
                node_grid[y][x],
                node_grid[y + 1][x],
                vec!["grid_edge".to_string(), "vertical".to_string()],
            );
        }
    }
}

fn generate_random(graph: &mut AlchemistGraph, nodes: usize, edge_probability: f32) {
    if nodes < 2 {
        return;
    }

    let mut node_ids = Vec::with_capacity(nodes);

    // Create random nodes
    let mut rng = rand::rng();
    for i in 0..nodes {
        let node_id = graph.add_node(&format!("Node_{i}"), vec!["random_node".to_string()]);
        node_ids.push(node_id);
    }

    // Create edges with probability
    for i in 0..nodes {
        for j in 0..nodes {
            if i != j && rng.random::<f32>() < edge_probability {
                // All edges created by add_edge will have weight 1.0 by default
                graph.add_edge(node_ids[i], node_ids[j], vec!["random_edge".to_string()]);
            }
        }
    }
}

fn generate_regular_polygon(graph: &mut AlchemistGraph, sides: usize) {
    if sides < 3 {
        // Minimum 3 sides for a polygon
        return;
    }

    let mut node_ids = Vec::with_capacity(sides);

    // Create polygon nodes
    for i in 0..sides {
        let node_id = graph.add_node(&format!("Node {i}"), vec!["circular_node".to_string()]);

        // Calculate position for this node in a regular polygon shape
        // Store the position in the node properties so it can be used by the layout system
        let angle = (2.0 * std::f32::consts::PI * i as f32) / sides as f32;
        let x = 200.0 * angle.cos(); // radius of 200 units
        let y = 200.0 * angle.sin();

        // Store positions as properties that the layout system can recognize
        graph
            .nodes
            .get_mut(&node_id)
            .unwrap()
            .properties
            .insert("x_pos".to_string(), x.to_string());
        graph
            .nodes
            .get_mut(&node_id)
            .unwrap()
            .properties
            .insert("y_pos".to_string(), y.to_string());
        graph
            .nodes
            .get_mut(&node_id)
            .unwrap()
            .properties
            .insert("fixed_position".to_string(), "true".to_string());

        node_ids.push(node_id);
    }

    // Connect nodes in a cycle to form the polygon
    for i in 0..sides {
        let next_i = (i + 1) % sides;
        graph.add_edge(
            node_ids[i],
            node_ids[next_i],
            vec!["polygon_edge".to_string()],
        );
    }
}

fn generate_moore_machine(graph: &mut AlchemistGraph) {
    // Create states
    let disconnected = graph.add_node(
        "Disconnected",
        vec![
            "state".to_string(),
            "initial".to_string(),
            "output=inactive".to_string(),
        ],
    );
    let connected = graph.add_node(
        "Connected",
        vec!["state".to_string(), "output=active".to_string()],
    );
    let error = graph.add_node(
        "Error",
        vec![
            "state".to_string(),
            "terminal".to_string(),
            "output=error".to_string(),
        ],
    );

    // Add transitions (without outputs)
    graph.add_edge(
        disconnected,
        connected,
        vec!["transition".to_string(), "input=connect".to_string()],
    );
    graph.add_edge(
        connected,
        disconnected,
        vec!["transition".to_string(), "input=disconnect".to_string()],
    );
    graph.add_edge(
        connected,
        error,
        vec!["transition".to_string(), "input=error".to_string()],
    );
    graph.add_edge(
        error,
        disconnected,
        vec!["transition".to_string(), "input=reset".to_string()],
    );
}

fn generate_mealy_machine(graph: &mut AlchemistGraph) {
    // Create states with specific positions to ensure proper 3D layout
    let waiting = graph.add_node("Waiting", vec!["state".to_string(), "initial".to_string()]);
    let processing = graph.add_node("Processing", vec!["state".to_string()]);
    let completed = graph.add_node("Completed", vec!["state".to_string()]);
    let error = graph.add_node("Error", vec!["state".to_string(), "terminal".to_string()]);

    // Add specific position properties to help with 3D layout
    if let Some(node) = graph.nodes.get_mut(&waiting) {
        node.properties
            .insert("fixed_position".to_string(), "true".to_string());
        node.properties
            .insert("x_pos".to_string(), "-200.0".to_string());
        node.properties
            .insert("y_pos".to_string(), "0.0".to_string());
        node.properties
            .insert("z_pos".to_string(), "0.0".to_string());
    }

    if let Some(node) = graph.nodes.get_mut(&processing) {
        node.properties
            .insert("fixed_position".to_string(), "true".to_string());
        node.properties
            .insert("x_pos".to_string(), "0.0".to_string());
        node.properties
            .insert("y_pos".to_string(), "0.0".to_string());
        node.properties
            .insert("z_pos".to_string(), "0.0".to_string());
    }

    if let Some(node) = graph.nodes.get_mut(&completed) {
        node.properties
            .insert("fixed_position".to_string(), "true".to_string());
        node.properties
            .insert("x_pos".to_string(), "200.0".to_string());
        node.properties
            .insert("y_pos".to_string(), "0.0".to_string());
        node.properties
            .insert("z_pos".to_string(), "0.0".to_string());
    }

    if let Some(node) = graph.nodes.get_mut(&error) {
        node.properties
            .insert("fixed_position".to_string(), "true".to_string());
        node.properties
            .insert("x_pos".to_string(), "0.0".to_string());
        node.properties
            .insert("y_pos".to_string(), "200.0".to_string());
        node.properties
            .insert("z_pos".to_string(), "0.0".to_string());
    }

    // Add transitions with outputs on the transitions (Mealy machine)
    graph.add_edge(
        waiting,
        processing,
        vec![
            "transition".to_string(),
            "input=start".to_string(),
            "output=initializing".to_string(),
        ],
    );
    graph.add_edge(
        processing,
        completed,
        vec![
            "transition".to_string(),
            "input=complete".to_string(),
            "output=success".to_string(),
        ],
    );
    graph.add_edge(
        processing,
        error,
        vec![
            "transition".to_string(),
            "input=fail".to_string(),
            "output=error".to_string(),
        ],
    );
    graph.add_edge(
        completed,
        waiting,
        vec![
            "transition".to_string(),
            "input=reset".to_string(),
            "output=ready".to_string(),
        ],
    );
    graph.add_edge(
        error,
        waiting,
        vec![
            "transition".to_string(),
            "input=reset".to_string(),
            "output=retry".to_string(),
        ],
    );
}

fn generate_finite_automaton(
    graph: &mut AlchemistGraph,
    states: usize,
    alphabet_size: usize,
    is_deterministic: bool,
) {
    if states == 0 || alphabet_size == 0 {
        return;
    }

    // Create state nodes
    let mut state_ids = Vec::with_capacity(states);
    for i in 0..states {
        let mut labels = vec!["state".to_string()];

        // Mark initial and accepting states
        if i == 0 {
            labels.push("initial".to_string());
        }

        // Make some states accepting (every third state)
        if i % 3 == 0 {
            labels.push("accepting".to_string());
        }

        let state_id = graph.add_node(&format!("q{i}"), vec!["qbert_node".to_string()]);
        state_ids.push(state_id);
    }

    // Create alphabet symbols
    let symbols: Vec<String> = (0..alphabet_size)
        .map(|i| format!("{}", (97 + i) as u8 as char)) // 'a', 'b', etc.
        .collect();

    // Create random transitions
    let mut rng = rand::rng();

    if is_deterministic {
        // For DFA: exactly one transition for each state-symbol pair
        for &from_state in &state_ids {
            for symbol in &symbols {
                // Pick a random target state
                let to_idx = rng.random_range(0..states);
                let to_state = state_ids[to_idx];

                graph.add_edge(
                    from_state,
                    to_state,
                    vec!["transition".to_string(), format!("symbol={}", symbol)],
                );
            }
        }
    } else {
        // For NFA: variable number of transitions (including ε-transitions)
        for &from_state in &state_ids {
            for symbol in &symbols {
                // For each symbol, potentially add transitions to multiple states
                for &to_state in &state_ids {
                    // Add transition with some probability
                    if rng.random::<f32>() < 0.3 {
                        graph.add_edge(
                            from_state,
                            to_state,
                            vec!["transition".to_string(), format!("symbol={}", symbol)],
                        );
                    }
                }
            }

            // Add some ε-transitions (empty string transitions)
            for &to_state in &state_ids {
                if from_state != to_state && rng.random::<f32>() < 0.2 {
                    graph.add_edge(
                        from_state,
                        to_state,
                        vec!["transition".to_string(), "symbol=ε".to_string()],
                    );
                }
            }
        }
    }
}

fn generate_dag(graph: &mut AlchemistGraph, levels: usize, nodes_per_level: usize) {
    if levels == 0 || nodes_per_level == 0 {
        return;
    }

    let mut level_nodes = Vec::with_capacity(levels);

    // Create nodes for each level
    for level in 0..levels {
        let mut nodes_at_this_level = Vec::with_capacity(nodes_per_level);

        for node in 0..nodes_per_level {
            let node_id = graph.add_node(
                &format!("L{level}-{node}"),
                vec!["binary_tree_node".to_string()],
            );
            nodes_at_this_level.push(node_id);
        }

        level_nodes.push(nodes_at_this_level);
    }

    // Connect nodes between levels (directed edges from upper to lower levels)
    for level in 0..levels - 1 {
        for &source in &level_nodes[level] {
            // Connect to some nodes in the next level
            for &target in &level_nodes[level + 1] {
                // Add some variability - not all nodes connect to all nodes in next level
                if rand::rng().random::<bool>() {
                    graph.add_edge(source, target, vec!["dag_edge".to_string()]);
                }
            }
        }
    }
}

fn generate_bipartite(
    graph: &mut AlchemistGraph,
    left_nodes: usize,
    right_nodes: usize,
    edge_density: f32,
) {
    if left_nodes == 0 || right_nodes == 0 {
        return;
    }

    let mut left_set = Vec::with_capacity(left_nodes);
    let mut right_set = Vec::with_capacity(right_nodes);

    // Create left set nodes
    for i in 0..left_nodes {
        let node_id = graph.add_node(
            &format!("L{i}"),
            vec!["balanced_tree_node".to_string(), "left".to_string()],
        );
        left_set.push(node_id);
    }

    // Create right set nodes
    for i in 0..right_nodes {
        let node_id = graph.add_node(
            &format!("R{i}"),
            vec!["balanced_tree_node".to_string(), "right".to_string()],
        );
        right_set.push(node_id);
    }

    // Connect nodes between sets based on density
    let mut rng = rand::rng();
    for &left in &left_set {
        for &right in &right_set {
            if rng.random::<f32>() < edge_density {
                // All edges created have a consistent weight of 1.0
                graph.add_edge(left, right, vec!["bipartite_edge".to_string()]);
            }
        }
    }
}
