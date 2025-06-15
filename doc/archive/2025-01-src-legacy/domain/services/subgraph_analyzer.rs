use crate::domain::value_objects::{
    ClusteringAlgorithm, EdgeId, NodeId, OptimizationType, SplitCriteria, SubgraphAnalysis,
    SubgraphStatistics, SubgraphType, SuggestedOperation,
};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};

/// Service for analyzing subgraphs and providing insights
pub struct SubgraphAnalyzer {
    metrics_calculator: MetricsCalculator,
    pattern_detector: PatternDetector,
    complexity_analyzer: ComplexityAnalyzer,
}

impl SubgraphAnalyzer {
    pub fn new() -> Self {
        Self {
            metrics_calculator: MetricsCalculator::new(),
            pattern_detector: PatternDetector::new(),
            complexity_analyzer: ComplexityAnalyzer::new(),
        }
    }

    /// Analyze a subgraph and provide comprehensive analysis
    pub fn analyze_subgraph(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_nodes: &HashSet<NodeId>,
    ) -> SubgraphAnalysis {
        // Calculate basic statistics
        let statistics = self
            .metrics_calculator
            .calculate_statistics(graph, subgraph_nodes);

        // Calculate quality metrics
        let cohesion_score = self.analyze_cohesion(graph, subgraph_nodes);
        let coupling_score = self.analyze_coupling(graph, subgraph_nodes);
        let complexity_score = self
            .complexity_analyzer
            .calculate_complexity(graph, subgraph_nodes);

        // Detect patterns and generate suggestions
        let suggested_operations = self.generate_suggestions(
            graph,
            subgraph_nodes,
            &statistics,
            cohesion_score,
            coupling_score,
            complexity_score,
        );

        SubgraphAnalysis {
            statistics,
            cohesion_score,
            coupling_score,
            complexity_score,
            suggested_operations,
        }
    }

    /// Analyze cohesion - how well connected nodes are within the subgraph
    pub fn analyze_cohesion(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_nodes: &HashSet<NodeId>,
    ) -> f32 {
        if subgraph_nodes.len() < 2 {
            return 1.0; // Single node is perfectly cohesive
        }

        let internal_edges = self.count_internal_edges(graph, subgraph_nodes);
        let max_possible_edges = (subgraph_nodes.len() * (subgraph_nodes.len() - 1)) / 2;

        if max_possible_edges == 0 {
            return 0.0;
        }

        internal_edges as f32 / max_possible_edges as f32
    }

    /// Analyze coupling - how dependent the subgraph is on external nodes
    pub fn analyze_coupling(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_nodes: &HashSet<NodeId>,
    ) -> f32 {
        let internal_edges = self.count_internal_edges(graph, subgraph_nodes);
        let external_edges = self.count_external_edges(graph, subgraph_nodes);

        if internal_edges + external_edges == 0 {
            return 0.0;
        }

        external_edges as f32 / (internal_edges + external_edges) as f32
    }

    /// Find optimal split points for a subgraph
    pub fn find_split_points(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_nodes: &HashSet<NodeId>,
        max_components: usize,
    ) -> Vec<SplitCriteria> {
        let mut suggestions = Vec::new();

        // Try connectivity-based splitting
        if let Some(criteria) = self.find_min_cut_split(graph, subgraph_nodes, max_components) {
            suggestions.push(criteria);
        }

        // Try clustering-based splitting
        if subgraph_nodes.len() >= 4 {
            suggestions.push(SplitCriteria::Clustering {
                algorithm: ClusteringAlgorithm::Spectral,
                num_clusters: 2.min(subgraph_nodes.len() / 2),
            });
        }

        suggestions
    }

    /// Calculate the cost of merging multiple subgraphs
    pub fn calculate_merge_cost(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_groups: &[HashSet<NodeId>],
    ) -> f32 {
        if subgraph_groups.len() < 2 {
            return 0.0;
        }

        // Calculate total nodes after merge
        let total_nodes: usize = subgraph_groups.iter().map(|g| g.len()).sum();

        // Calculate connections between subgraphs
        let mut inter_connections = 0;
        for i in 0..subgraph_groups.len() {
            for j in (i + 1)..subgraph_groups.len() {
                inter_connections +=
                    self.count_connections_between(graph, &subgraph_groups[i], &subgraph_groups[j]);
            }
        }

        // Cost is based on size and lack of connections
        let size_factor = total_nodes as f32 / 100.0; // Normalize by typical max size
        let connection_factor = if inter_connections > 0 {
            1.0 / inter_connections as f32
        } else {
            10.0 // High cost for unconnected subgraphs
        };

        size_factor * connection_factor
    }

    // Helper methods
    fn count_internal_edges(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_nodes: &HashSet<NodeId>,
    ) -> usize {
        let node_indices: HashMap<NodeId, NodeIndex> = graph
            .node_indices()
            .filter_map(|idx| graph.node_weight(idx).map(|&node_id| (node_id, idx)))
            .collect();

        graph
            .edge_indices()
            .filter(|&edge_idx| {
                if let Some((source_idx, target_idx)) = graph.edge_endpoints(edge_idx) {
                    if let (Some(&source_id), Some(&target_id)) =
                        (graph.node_weight(source_idx), graph.node_weight(target_idx))
                    {
                        return subgraph_nodes.contains(&source_id)
                            && subgraph_nodes.contains(&target_id);
                    }
                }
                false
            })
            .count()
    }

    fn count_external_edges(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_nodes: &HashSet<NodeId>,
    ) -> usize {
        graph
            .edge_indices()
            .filter(|&edge_idx| {
                if let Some((source_idx, target_idx)) = graph.edge_endpoints(edge_idx) {
                    if let (Some(&source_id), Some(&target_id)) =
                        (graph.node_weight(source_idx), graph.node_weight(target_idx))
                    {
                        let source_in = subgraph_nodes.contains(&source_id);
                        let target_in = subgraph_nodes.contains(&target_id);
                        return source_in != target_in; // One in, one out
                    }
                }
                false
            })
            .count()
    }

    fn count_connections_between(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        group1: &HashSet<NodeId>,
        group2: &HashSet<NodeId>,
    ) -> usize {
        graph
            .edge_indices()
            .filter(|&edge_idx| {
                if let Some((source_idx, target_idx)) = graph.edge_endpoints(edge_idx) {
                    if let (Some(&source_id), Some(&target_id)) =
                        (graph.node_weight(source_idx), graph.node_weight(target_idx))
                    {
                        return (group1.contains(&source_id) && group2.contains(&target_id))
                            || (group2.contains(&source_id) && group1.contains(&target_id));
                    }
                }
                false
            })
            .count()
    }

    fn find_min_cut_split(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_nodes: &HashSet<NodeId>,
        max_components: usize,
    ) -> Option<SplitCriteria> {
        // Simple heuristic: if the subgraph has low connectivity, suggest splitting
        let cohesion = self.analyze_cohesion(graph, subgraph_nodes);
        if cohesion < 0.3 && subgraph_nodes.len() >= max_components {
            Some(SplitCriteria::Connectivity {
                min_cut: true,
                max_components,
            })
        } else {
            None
        }
    }

    fn generate_suggestions(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_nodes: &HashSet<NodeId>,
        statistics: &SubgraphStatistics,
        cohesion_score: f32,
        coupling_score: f32,
        complexity_score: f32,
    ) -> Vec<SuggestedOperation> {
        let mut suggestions = Vec::new();

        // Suggest split if cohesion is low and size is large
        if cohesion_score < 0.3 && statistics.node_count > 10 {
            if let Some(criteria) = self.find_min_cut_split(graph, subgraph_nodes, 2) {
                suggestions.push(SuggestedOperation::Split {
                    reason: format!(
                        "Low cohesion ({:.2}) with {} nodes suggests splitting",
                        cohesion_score, statistics.node_count
                    ),
                    criteria,
                    confidence: 0.8,
                });
            }
        }

        // Suggest optimization if complexity is high
        if complexity_score > 0.7 {
            suggestions.push(SuggestedOperation::Optimize {
                reason: format!("High complexity score ({:.2})", complexity_score),
                optimization_type: OptimizationType::SimplifyStructure,
                confidence: 0.7,
            });
        }

        // Suggest refactoring based on patterns
        if let Some(suggested_type) = self
            .pattern_detector
            .detect_type(statistics, cohesion_score)
        {
            suggestions.push(SuggestedOperation::Refactor {
                reason: "Pattern analysis suggests different organization".to_string(),
                suggested_type,
                confidence: 0.6,
            });
        }

        suggestions
    }
}

/// Calculator for subgraph metrics
pub struct MetricsCalculator {
    // Could add configuration here
}

impl MetricsCalculator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn calculate_statistics(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_nodes: &HashSet<NodeId>,
    ) -> SubgraphStatistics {
        let node_count = subgraph_nodes.len();
        let mut internal_edges = 0;
        let mut external_edges = 0;
        let mut degree_sum = 0;

        // Create a mapping of NodeId to NodeIndex for efficient lookup
        let node_indices: HashMap<NodeId, NodeIndex> = graph
            .node_indices()
            .filter_map(|idx| graph.node_weight(idx).map(|&node_id| (node_id, idx)))
            .collect();

        // Count edges and calculate degrees
        for &node_id in subgraph_nodes {
            if let Some(&node_idx) = node_indices.get(&node_id) {
                let mut node_degree = 0;

                for edge in graph.edges(node_idx) {
                    node_degree += 1;
                    let target_idx = edge.target();
                    if let Some(&target_id) = graph.node_weight(target_idx) {
                        if subgraph_nodes.contains(&target_id) {
                            internal_edges += 1;
                        } else {
                            external_edges += 1;
                        }
                    }
                }

                degree_sum += node_degree;
            }
        }

        // Internal edges are counted twice (once from each end)
        internal_edges /= 2;

        let edge_count = internal_edges + external_edges;
        let density = if node_count > 1 {
            (2.0 * internal_edges as f32) / (node_count as f32 * (node_count - 1) as f32)
        } else {
            0.0
        };

        let average_degree = if node_count > 0 {
            degree_sum as f32 / node_count as f32
        } else {
            0.0
        };

        // Calculate clustering coefficient (simplified)
        let clustering_coefficient =
            self.calculate_clustering_coefficient(graph, subgraph_nodes, &node_indices);

        SubgraphStatistics {
            node_count,
            edge_count,
            internal_edges,
            external_edges,
            depth: 1, // Would need tree analysis for accurate depth
            density,
            clustering_coefficient,
            average_degree,
        }
    }

    fn calculate_clustering_coefficient(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_nodes: &HashSet<NodeId>,
        node_indices: &HashMap<NodeId, NodeIndex>,
    ) -> f32 {
        let mut total_coefficient = 0.0;
        let mut counted_nodes = 0;

        for &node_id in subgraph_nodes {
            if let Some(&node_idx) = node_indices.get(&node_id) {
                let neighbors: Vec<NodeIndex> = graph
                    .neighbors(node_idx)
                    .filter(|&neighbor_idx| {
                        if let Some(&neighbor_id) = graph.node_weight(neighbor_idx) {
                            subgraph_nodes.contains(&neighbor_id)
                        } else {
                            false
                        }
                    })
                    .collect();

                if neighbors.len() >= 2 {
                    let mut neighbor_connections = 0;
                    let max_connections = neighbors.len() * (neighbors.len() - 1) / 2;

                    for i in 0..neighbors.len() {
                        for j in (i + 1)..neighbors.len() {
                            if graph.find_edge(neighbors[i], neighbors[j]).is_some()
                                || graph.find_edge(neighbors[j], neighbors[i]).is_some()
                            {
                                neighbor_connections += 1;
                            }
                        }
                    }

                    if max_connections > 0 {
                        total_coefficient += neighbor_connections as f32 / max_connections as f32;
                        counted_nodes += 1;
                    }
                }
            }
        }

        if counted_nodes > 0 {
            total_coefficient / counted_nodes as f32
        } else {
            0.0
        }
    }
}

/// Detector for common patterns in subgraphs
pub struct PatternDetector {
    // Pattern detection configuration
}

impl PatternDetector {
    pub fn new() -> Self {
        Self {}
    }

    pub fn detect_type(
        &self,
        statistics: &SubgraphStatistics,
        cohesion: f32,
    ) -> Option<SubgraphType> {
        // Simple heuristics for type detection
        if statistics.density > 0.8 && statistics.node_count < 10 {
            Some(SubgraphType::Cluster)
        } else if statistics.average_degree < 2.0 && cohesion < 0.3 {
            Some(SubgraphType::Namespace)
        } else if statistics.internal_edges > statistics.external_edges * 3 {
            Some(SubgraphType::Module)
        } else {
            None
        }
    }
}

/// Analyzer for subgraph complexity
pub struct ComplexityAnalyzer {
    // Complexity analysis configuration
}

impl ComplexityAnalyzer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn calculate_complexity(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_nodes: &HashSet<NodeId>,
    ) -> f32 {
        let node_count = subgraph_nodes.len() as f32;
        let edge_count = self.count_edges(graph, subgraph_nodes) as f32;

        if node_count == 0.0 {
            return 0.0;
        }

        // Simple complexity metric based on edge-to-node ratio and size
        let edge_complexity = (edge_count / node_count).min(5.0) / 5.0;
        let size_complexity = (node_count / 50.0).min(1.0);

        (edge_complexity * 0.7 + size_complexity * 0.3).min(1.0)
    }

    fn count_edges(
        &self,
        graph: &Graph<NodeId, EdgeId>,
        subgraph_nodes: &HashSet<NodeId>,
    ) -> usize {
        graph
            .edge_indices()
            .filter(|&edge_idx| {
                if let Some((source_idx, target_idx)) = graph.edge_endpoints(edge_idx) {
                    if let (Some(&source_id), Some(&target_id)) =
                        (graph.node_weight(source_idx), graph.node_weight(target_idx))
                    {
                        return subgraph_nodes.contains(&source_id)
                            || subgraph_nodes.contains(&target_id);
                    }
                }
                false
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::Graph;

    #[test]
    fn test_cohesion_analysis() {
        let mut graph = Graph::new();
        let n1 = graph.add_node(NodeId::new());
        let n2 = graph.add_node(NodeId::new());
        let n3 = graph.add_node(NodeId::new());

        graph.add_edge(n1, n2, EdgeId::new());
        graph.add_edge(n2, n3, EdgeId::new());
        graph.add_edge(n3, n1, EdgeId::new());

        let subgraph_nodes: HashSet<NodeId> = graph.node_weights().cloned().collect();

        let analyzer = SubgraphAnalyzer::new();
        let cohesion = analyzer.analyze_cohesion(&graph, &subgraph_nodes);

        assert_eq!(cohesion, 1.0); // Fully connected triangle
    }

    #[test]
    fn test_coupling_analysis() {
        let mut graph = Graph::new();
        let n1 = graph.add_node(NodeId::new());
        let n2 = graph.add_node(NodeId::new());
        let n3 = graph.add_node(NodeId::new());
        let n4 = graph.add_node(NodeId::new());

        graph.add_edge(n1, n2, EdgeId::new());
        graph.add_edge(n2, n3, EdgeId::new());
        graph.add_edge(n3, n4, EdgeId::new());

        let mut subgraph_nodes = HashSet::new();
        subgraph_nodes.insert(*graph.node_weight(n1).unwrap());
        subgraph_nodes.insert(*graph.node_weight(n2).unwrap());

        let analyzer = SubgraphAnalyzer::new();
        let coupling = analyzer.analyze_coupling(&graph, &subgraph_nodes);

        assert!(coupling > 0.0 && coupling < 1.0); // Has external connections
    }
}
