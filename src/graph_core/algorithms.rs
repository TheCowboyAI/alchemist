use bevy::prelude::*;
use petgraph::{
    algo::{
        dijkstra, has_path_connecting, is_cyclic_directed, tarjan_scc,
    },
    graph::{DiGraph, NodeIndex as PetNodeIndex},
    visit::{Bfs, Dfs, EdgeRef, Topo},
    Direction,
};
use std::collections::HashMap;
use uuid::Uuid;

use super::graph_data::{EdgeData, GraphData, NodeData};

/// Graph algorithm examples and utilities
#[derive(Resource)]
pub struct GraphAlgorithms;

impl GraphAlgorithms {
    /// Find shortest path between two nodes using Dijkstra's algorithm
    pub fn shortest_path(
        graph_data: &GraphData,
        start_id: Uuid,
        end_id: Uuid,
    ) -> Option<(Vec<Uuid>, f64)> {
        // Get node indices
        let start_idx = graph_data.uuid_to_node.get(&start_id)?;
        let end_idx = graph_data.uuid_to_node.get(&end_id)?;

        // Run Dijkstra's algorithm with unit weights
        let node_map = dijkstra(&graph_data.graph, *start_idx, Some(*end_idx), |_| 1.0);

        // Check if path exists
        let cost = node_map.get(end_idx)?;

        // Reconstruct path by following predecessors
        let path = Self::reconstruct_path(&graph_data.graph, *start_idx, *end_idx, &node_map)?;

        // Convert to UUIDs
        let uuid_path: Vec<Uuid> = path
            .iter()
            .filter_map(|idx| graph_data.graph.node_weight(*idx).map(|n| n.id))
            .collect();

        Some((uuid_path, *cost))
    }

    /// Check if two nodes are connected
    pub fn are_connected(graph_data: &GraphData, start_id: Uuid, end_id: Uuid) -> bool {
        if let (Some(start_idx), Some(end_idx)) = (
            graph_data.uuid_to_node.get(&start_id),
            graph_data.uuid_to_node.get(&end_id),
        ) {
            has_path_connecting(&graph_data.graph, *start_idx, *end_idx, None)
        } else {
            false
        }
    }

    /// Find all connected components
    pub fn find_components(graph_data: &GraphData) -> Vec<Vec<Uuid>> {
        let n = graph_data.graph.node_count();
        if n == 0 {
            return vec![];
        }

        // Get strongly connected components using Tarjan's algorithm
        let sccs = tarjan_scc(&graph_data.graph);

        // Convert to UUIDs
        sccs.into_iter()
            .map(|component| {
                component
                    .into_iter()
                    .filter_map(|idx| graph_data.graph.node_weight(idx).map(|n| n.id))
                    .collect()
            })
            .collect()
    }

    /// Detect if the graph has cycles
    pub fn has_cycles(graph_data: &GraphData) -> bool {
        is_cyclic_directed(&graph_data.graph)
    }

    /// Perform breadth-first search from a node
    pub fn bfs_from_node(
        graph_data: &GraphData,
        start_id: Uuid,
        max_depth: Option<usize>,
    ) -> Vec<(Uuid, usize)> {
        let start_idx = match graph_data.uuid_to_node.get(&start_id) {
            Some(idx) => *idx,
            None => return vec![],
        };

        let mut bfs = Bfs::new(&graph_data.graph, start_idx);
        let mut result = vec![];
        let mut depths = HashMap::new();
        depths.insert(start_idx, 0);

        while let Some(node_idx) = bfs.next(&graph_data.graph) {
            let depth = depths[&node_idx];

            // Check max depth
            if let Some(max) = max_depth {
                if depth > max {
                    continue;
                }
            }

            // Add neighbors to depth map
            for neighbor in graph_data.graph.neighbors(node_idx) {
                depths.entry(neighbor).or_insert_with(|| depth + 1);
            }

            // Add to result
            if let Some(node) = graph_data.graph.node_weight(node_idx) {
                result.push((node.id, depth));
            }
        }

        result
    }

    /// Perform depth-first search from a node
    pub fn dfs_from_node(graph_data: &GraphData, start_id: Uuid) -> Vec<Uuid> {
        let start_idx = match graph_data.uuid_to_node.get(&start_id) {
            Some(idx) => *idx,
            None => return vec![],
        };

        let mut dfs = Dfs::new(&graph_data.graph, start_idx);
        let mut result = vec![];

        while let Some(node_idx) = dfs.next(&graph_data.graph) {
            if let Some(node) = graph_data.graph.node_weight(node_idx) {
                result.push(node.id);
            }
        }

        result
    }

    /// Get topological ordering (only valid for DAGs)
    pub fn topological_sort(graph_data: &GraphData) -> Result<Vec<Uuid>, String> {
        if Self::has_cycles(graph_data) {
            return Err("Graph contains cycles, cannot perform topological sort".to_string());
        }

        let mut topo = Topo::new(&graph_data.graph);
        let mut result = vec![];

        while let Some(node_idx) = topo.next(&graph_data.graph) {
            if let Some(node) = graph_data.graph.node_weight(node_idx) {
                result.push(node.id);
            }
        }

        Ok(result)
    }

    /// Calculate node centrality (degree centrality)
    pub fn degree_centrality(graph_data: &GraphData) -> HashMap<Uuid, (usize, usize, usize)> {
        let mut centrality = HashMap::new();

        for (node_idx, node) in graph_data.nodes() {
            let in_degree = graph_data
                .graph
                .edges_directed(node_idx, Direction::Incoming)
                .count();
            let out_degree = graph_data
                .graph
                .edges_directed(node_idx, Direction::Outgoing)
                .count();
            let total_degree = in_degree + out_degree;

            centrality.insert(node.id, (in_degree, out_degree, total_degree));
        }

        centrality
    }

    /// Find all paths between two nodes (limited to avoid exponential explosion)
    pub fn find_all_paths(
        graph_data: &GraphData,
        start_id: Uuid,
        end_id: Uuid,
        max_paths: usize,
    ) -> Vec<Vec<Uuid>> {
        let start_idx = match graph_data.uuid_to_node.get(&start_id) {
            Some(idx) => *idx,
            None => return vec![],
        };

        let end_idx = match graph_data.uuid_to_node.get(&end_id) {
            Some(idx) => *idx,
            None => return vec![],
        };

        let mut all_paths = vec![];
        let mut current_path = vec![start_idx];
        let mut visited = vec![false; graph_data.graph.node_count()];

        Self::dfs_all_paths(
            &graph_data.graph,
            start_idx,
            end_idx,
            &mut visited,
            &mut current_path,
            &mut all_paths,
            max_paths,
        );

        // Convert to UUID paths
        all_paths
            .into_iter()
            .map(|path| {
                path.into_iter()
                    .filter_map(|idx| graph_data.graph.node_weight(idx).map(|n| n.id))
                    .collect()
            })
            .collect()
    }

    /// Helper for finding all paths using DFS
    fn dfs_all_paths(
        graph: &DiGraph<NodeData, EdgeData>,
        current: PetNodeIndex,
        target: PetNodeIndex,
        visited: &mut Vec<bool>,
        current_path: &mut Vec<PetNodeIndex>,
        all_paths: &mut Vec<Vec<PetNodeIndex>>,
        max_paths: usize,
    ) {
        if all_paths.len() >= max_paths {
            return;
        }

        visited[current.index()] = true;

        if current == target {
            all_paths.push(current_path.clone());
        } else {
            for neighbor in graph.neighbors(current) {
                if !visited[neighbor.index()] {
                    current_path.push(neighbor);
                    Self::dfs_all_paths(
                        graph,
                        neighbor,
                        target,
                        visited,
                        current_path,
                        all_paths,
                        max_paths,
                    );
                    current_path.pop();
                }
            }
        }

        visited[current.index()] = false;
    }

    /// Reconstruct path from Dijkstra results
    fn reconstruct_path(
        graph: &DiGraph<NodeData, EdgeData>,
        start: PetNodeIndex,
        end: PetNodeIndex,
        predecessors: &HashMap<PetNodeIndex, f64>,
    ) -> Option<Vec<PetNodeIndex>> {
        if !predecessors.contains_key(&end) {
            return None;
        }

        let mut path = vec![end];
        let mut current = end;

        // Work backwards from end to start
        while current != start {
            let mut found_predecessor = false;

            // Check all nodes that have edges to current
            for edge in graph.edges_directed(current, Direction::Incoming) {
                let predecessor = edge.source();
                if predecessors.contains_key(&predecessor) {
                    path.push(predecessor);
                    current = predecessor;
                    found_predecessor = true;
                    break;
                }
            }

            if !found_predecessor {
                return None;
            }
        }

        path.reverse();
        Some(path)
    }
}

/// System to run graph algorithm examples
pub fn demonstrate_algorithms(graph_data: Res<GraphData>) {
    if graph_data.node_count() < 2 {
        return;
    }

    // Get first two nodes for demo
    let nodes: Vec<_> = graph_data.nodes().take(2).collect();
    if nodes.len() < 2 {
        return;
    }

    let (_, node1) = nodes[0];
    let (_, node2) = nodes[1];

    // Shortest path example
    if let Some((path, cost)) = GraphAlgorithms::shortest_path(&graph_data, node1.id, node2.id) {
        // info!(
        //     "Shortest path from {} to {}: {:?} (cost: {})",
        //     node1.name, node2.name, path, cost
        // );
    }

    // Connectivity check
    let connected = GraphAlgorithms::are_connected(&graph_data, node1.id, node2.id);
    // info!(
    //     "Nodes {} and {} are connected: {}",
    //     node1.name, node2.name, connected
    // );

    // Find components
    let components = GraphAlgorithms::find_components(&graph_data);
    // info!("Found {} connected components", components.len());

    // Check for cycles
    let has_cycles = GraphAlgorithms::has_cycles(&graph_data);
    // info!("Graph has cycles: {}", has_cycles);

    // Topological sort (if DAG)
    match GraphAlgorithms::topological_sort(&graph_data) {
        Ok(order) => {} // info!("Topological order: {:?}", order),
        Err(e) => {} // info!("Cannot perform topological sort: {}", e),
    }

    // Degree centrality
    let centrality = GraphAlgorithms::degree_centrality(&graph_data);
    // for (node_id, (in_deg, out_deg, total)) in centrality.iter().take(5) {
    //     info!(
    //         "Node {:?} - In: {}, Out: {}, Total: {}",
    //         node_id, in_deg, out_deg, total
    //     );
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_core::components::{DomainEdgeType, DomainNodeType};

    fn create_test_graph() -> GraphData {
        let mut graph = GraphData::new();

        // Create nodes
        let node1 = NodeData {
            id: Uuid::new_v4(),
            name: "Node1".to_string(),
            domain_type: DomainNodeType::Process,
            position: Vec3::ZERO,
            labels: vec![],
            properties: HashMap::new(),
        };

        let node2 = NodeData {
            id: Uuid::new_v4(),
            name: "Node2".to_string(),
            domain_type: DomainNodeType::Process,
            position: Vec3::new(1.0, 0.0, 0.0),
            labels: vec![],
            properties: HashMap::new(),
        };

        let edge = EdgeData {
            id: Uuid::new_v4(),
            edge_type: DomainEdgeType::DataFlow,
            labels: vec![],
            properties: Default::default(),
        };

        let _idx1 = graph.add_node(node1.clone());
        let _idx2 = graph.add_node(node2.clone());

        graph.add_edge(node1.id, node2.id, edge).unwrap();

        graph
    }

    #[test]
    fn test_shortest_path() {
        let graph = create_test_graph();
        let nodes: Vec<_> = graph.nodes().collect();

        let path = GraphAlgorithms::shortest_path(&graph, nodes[0].1.id, nodes[1].1.id);
        assert!(path.is_some());

        let (path_nodes, cost) = path.unwrap();
        assert_eq!(path_nodes.len(), 2);
        assert_eq!(cost, 1.0);
    }

    #[test]
    fn test_connectivity() {
        let graph = create_test_graph();
        let nodes: Vec<_> = graph.nodes().collect();

        let connected = GraphAlgorithms::are_connected(&graph, nodes[0].1.id, nodes[1].1.id);
        assert!(connected);
    }
}
