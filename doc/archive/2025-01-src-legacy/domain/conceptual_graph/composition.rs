//! Graph Composition Operations
//!
//! Implements various ways to compose conceptual graphs

use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::conceptual_graph::concept::{ConceptEdge, ConceptGraph, ConceptNode, NodeId};
use crate::domain::conceptual_graph::morphism::{GraphMorphism, InjectionMap, ProductType};

use crate::domain::conceptual_graph::{concept::ConceptId, quality_dimension::ConceptualPoint};

/// Types of composition operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompositionOperation {
    /// Embed one graph into another
    Embed,
    /// Product of two graphs
    Product(ProductType),
    /// Coproduct (disjoint union)
    Coproduct,
    /// Quotient by equivalence
    Quotient,
    /// Pullback (fiber product)
    Pullback,
    /// Pushout (amalgamated sum)
    Pushout,
}

/// Result of a composition operation
#[derive(Debug, Clone)]
pub struct CompositionResult {
    /// The resulting graph
    pub graph: ConceptGraph,

    /// Morphisms from input graphs to result
    pub morphisms: Vec<GraphMorphism>,

    /// Mapping of original nodes to result nodes
    pub node_mappings: HashMap<(ConceptId, NodeId), NodeIndex>,
}

/// Graph composition operations
pub struct GraphComposer;

impl GraphComposer {
    /// Embed one graph into another
    pub fn embed(
        subgraph: &ConceptGraph,
        host: &mut ConceptGraph,
        position_offset: Vec<f64>,
    ) -> Result<CompositionResult, String> {
        let mut injection = InjectionMap::new();
        injection.position_offset = position_offset;

        let mut node_mappings = HashMap::new();

        // Copy nodes from subgraph to host
        for node_idx in subgraph.structure.node_indices() {
            if let Some(node) = subgraph.structure.node_weight(node_idx) {
                let new_node = node.clone();
                let new_idx = host.add_node(new_node);

                // Track mapping
                if let Some(node_id) = Self::get_node_id(node) {
                    injection.add_node_injection(node_id, node_id);
                    node_mappings.insert((subgraph.id, node_id), new_idx);
                }
            }
        }

        // Copy edges
        for edge_idx in subgraph.structure.edge_indices() {
            if let Some((source, target)) = subgraph.structure.edge_endpoints(edge_idx) {
                if let Some(edge) = subgraph.structure.edge_weight(edge_idx) {
                    // Find corresponding nodes in host
                    let source_node = subgraph.structure.node_weight(source).unwrap();
                    let target_node = subgraph.structure.node_weight(target).unwrap();

                    if let (Some(source_id), Some(target_id)) = (
                        Self::get_node_id(source_node),
                        Self::get_node_id(target_node),
                    ) {
                        if let (Some(&new_source), Some(&new_target)) = (
                            node_mappings.get(&(subgraph.id, source_id)),
                            node_mappings.get(&(subgraph.id, target_id)),
                        ) {
                            let new_edge = edge.clone();
                            host.add_edge(new_source, new_target, new_edge);
                            injection.add_edge_injection(edge.id, edge.id);
                        }
                    }
                }
            }
        }

        // Create embedding morphism
        let morphism = GraphMorphism::Embedding {
            subgraph_id: subgraph.id,
            host_id: host.id,
            injection,
        };

        Ok(CompositionResult {
            graph: host.clone(),
            morphisms: vec![morphism],
            node_mappings,
        })
    }

    /// Compute the product of two graphs
    pub fn product(
        left: &ConceptGraph,
        right: &ConceptGraph,
        product_type: ProductType,
    ) -> Result<CompositionResult, String> {
        let mut result = ConceptGraph::new(format!("{} × {}", left.name, right.name));
        result.category = left.category.clone(); // Inherit category from left

        // Combine quality dimensions
        result.quality_dimensions = left.quality_dimensions.clone();
        result
            .quality_dimensions
            .extend(right.quality_dimensions.clone());

        let mut node_mappings = HashMap::new();
        let mut left_to_product = HashMap::new();
        let mut right_to_product = HashMap::new();

        // Create product nodes
        for left_idx in left.structure.node_indices() {
            for right_idx in right.structure.node_indices() {
                if let (Some(left_node), Some(right_node)) = (
                    left.structure.node_weight(left_idx),
                    right.structure.node_weight(right_idx),
                ) {
                    // Create product node
                    let product_node = Self::create_product_node(left_node, right_node)?;
                    let product_idx = result.add_node(product_node);

                    // Track mappings
                    if let (Some(left_id), Some(right_id)) =
                        (Self::get_node_id(left_node), Self::get_node_id(right_node))
                    {
                        left_to_product.insert(left_idx, product_idx);
                        right_to_product.insert(right_idx, product_idx);
                        node_mappings.insert((left.id, left_id), product_idx);
                        node_mappings.insert((right.id, right_id), product_idx);
                    }
                }
            }
        }

        // Create product edges based on product type
        match product_type {
            ProductType::Cartesian => {
                Self::add_cartesian_edges(
                    &left,
                    &right,
                    &mut result,
                    &left_to_product,
                    &right_to_product,
                )?;
            }
            ProductType::Tensor => {
                Self::add_tensor_edges(
                    &left,
                    &right,
                    &mut result,
                    &left_to_product,
                    &right_to_product,
                )?;
            }
            _ => return Err("Product type not implemented".to_string()),
        }

        // Create morphisms
        let morphisms = vec![GraphMorphism::Product {
            left_id: left.id,
            right_id: right.id,
            product_type,
        }];

        Ok(CompositionResult {
            graph: result,
            morphisms,
            node_mappings,
        })
    }

    /// Compute the coproduct (disjoint union) of graphs
    pub fn coproduct(graphs: &[&ConceptGraph]) -> Result<CompositionResult, String> {
        if graphs.is_empty() {
            return Err("Cannot compute coproduct of empty list".to_string());
        }

        let mut result = ConceptGraph::new(
            graphs
                .iter()
                .map(|g| g.name.clone())
                .collect::<Vec<_>>()
                .join(" ⊕ "),
        );

        let mut node_mappings = HashMap::new();
        let mut component_ids = Vec::new();

        // Copy each graph into the result
        for graph in graphs {
            component_ids.push(graph.id);

            // Copy nodes
            for node_idx in graph.structure.node_indices() {
                if let Some(node) = graph.structure.node_weight(node_idx) {
                    let new_node = node.clone();
                    let new_idx = result.add_node(new_node);

                    if let Some(node_id) = Self::get_node_id(node) {
                        node_mappings.insert((graph.id, node_id), new_idx);
                    }
                }
            }

            // Copy edges
            for edge_idx in graph.structure.edge_indices() {
                if let Some((source, target)) = graph.structure.edge_endpoints(edge_idx) {
                    if let Some(edge) = graph.structure.edge_weight(edge_idx) {
                        let source_node = graph.structure.node_weight(source).unwrap();
                        let target_node = graph.structure.node_weight(target).unwrap();

                        if let (Some(source_id), Some(target_id)) = (
                            Self::get_node_id(source_node),
                            Self::get_node_id(target_node),
                        ) {
                            if let (Some(&new_source), Some(&new_target)) = (
                                node_mappings.get(&(graph.id, source_id)),
                                node_mappings.get(&(graph.id, target_id)),
                            ) {
                                result.add_edge(new_source, new_target, edge.clone());
                            }
                        }
                    }
                }
            }
        }

        let morphism = GraphMorphism::Coproduct { component_ids };

        Ok(CompositionResult {
            graph: result,
            morphisms: vec![morphism],
            node_mappings,
        })
    }

    // Helper functions

    fn get_node_id(node: &ConceptNode) -> Option<NodeId> {
        match node {
            ConceptNode::Atom { id, .. } => Some(*id),
            ConceptNode::Composite { id, .. } => Some(*id),
            ConceptNode::Function { id, .. } => Some(*id),
        }
    }

    fn create_product_node(left: &ConceptNode, right: &ConceptNode) -> Result<ConceptNode, String> {
        // For now, create a composite node containing both
        let product_graph = ConceptGraph::new("Product Node");

        // Combine quality positions from both nodes
        let left_pos = left.quality_position();
        let right_pos = right.quality_position();

        // Concatenate coordinates from both positions
        let mut combined_coords = left_pos.coordinates.clone();
        combined_coords.extend(&right_pos.coordinates);

        let quality_position = ConceptualPoint::new(combined_coords);

        Ok(ConceptNode::Composite {
            id: NodeId::new(),
            quality_position,
            subgraph: Box::new(product_graph),
        })
    }

    fn add_cartesian_edges(
        left: &ConceptGraph,
        right: &ConceptGraph,
        result: &mut ConceptGraph,
        left_map: &HashMap<NodeIndex, NodeIndex>,
        right_map: &HashMap<NodeIndex, NodeIndex>,
    ) -> Result<(), String> {
        // In Cartesian product, (a,b) connects to (a',b) if a connects to a' in left graph
        // and (a,b) connects to (a,b') if b connects to b' in right graph

        // Add edges from left graph structure
        for edge_idx in left.structure.edge_indices() {
            if let Some((source, target)) = left.structure.edge_endpoints(edge_idx) {
                if let Some(edge) = left.structure.edge_weight(edge_idx) {
                    // For each node in right graph
                    for right_idx in right.structure.node_indices() {
                        if let (Some(&prod_source), Some(&prod_target)) =
                            (left_map.get(&source), left_map.get(&target))
                        {
                            let new_edge = ConceptEdge::new(edge.relationship.clone());
                            result.add_edge(prod_source, prod_target, new_edge);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn add_tensor_edges(
        left: &ConceptGraph,
        right: &ConceptGraph,
        result: &mut ConceptGraph,
        left_map: &HashMap<NodeIndex, NodeIndex>,
        right_map: &HashMap<NodeIndex, NodeIndex>,
    ) -> Result<(), String> {
        // Tensor product has different edge structure
        // Implementation depends on specific requirements
        Ok(())
    }
}

/// Builder for composing graphs step by step
pub struct CompositionBuilder {
    base: Option<ConceptGraph>,
    operations: Vec<(CompositionOperation, ConceptGraph)>,
}

impl CompositionBuilder {
    pub fn new() -> Self {
        Self {
            base: None,
            operations: Vec::new(),
        }
    }

    pub fn with_base(mut self, graph: ConceptGraph) -> Self {
        self.base = Some(graph);
        self
    }

    pub fn embed(mut self, graph: ConceptGraph) -> Self {
        self.operations.push((CompositionOperation::Embed, graph));
        self
    }

    pub fn product(mut self, graph: ConceptGraph, product_type: ProductType) -> Self {
        self.operations
            .push((CompositionOperation::Product(product_type), graph));
        self
    }

    pub fn build(self) -> Result<ConceptGraph, String> {
        let mut result = self.base.ok_or("No base graph provided")?;

        for (op, graph) in self.operations {
            match op {
                CompositionOperation::Embed => {
                    let composition =
                        GraphComposer::embed(&graph, &mut result, vec![0.0, 0.0, 0.0])?;
                    result = composition.graph;
                }
                CompositionOperation::Product(product_type) => {
                    let composition = GraphComposer::product(&result, &graph, product_type)?;
                    result = composition.graph;
                }
                _ => return Err("Operation not implemented in builder".to_string()),
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_operation() {
        let mut host = ConceptGraph::new("Host");
        let subgraph = ConceptGraph::new("Subgraph");

        let result = GraphComposer::embed(&subgraph, &mut host, vec![1.0, 0.0, 0.0]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_coproduct_operation() {
        let graph1 = ConceptGraph::new("Graph1");
        let graph2 = ConceptGraph::new("Graph2");

        let result = GraphComposer::coproduct(&[&graph1, &graph2]);
        assert!(result.is_ok());

        let composition = result.unwrap();
        assert_eq!(composition.graph.name, "Graph1 ⊕ Graph2");
    }

    #[test]
    fn test_composition_builder() {
        let base = ConceptGraph::new("Base");
        let addon = ConceptGraph::new("Addon");

        let result = CompositionBuilder::new()
            .with_base(base)
            .embed(addon)
            .build();

        assert!(result.is_ok());
    }
}
