use crate::domain::{
    events::DomainEvent,
    value_objects::{GraphId, NodeId, EdgeId, Position3D, RelatedBy},
};
use cim_ipld::types::{Cid, ContentType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur in ConceptGraph operations
#[derive(Debug, Error)]
pub enum ConceptGraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("Edge not found: {0}")]
    EdgeNotFound(EdgeId),

    #[error("Circular reference detected")]
    CircularReference,
}

/// A graph that can contain other graphs - recursive by nature
///
/// This is the fundamental abstraction: everything is a graph,
/// and graphs can contain other graphs as nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptGraph {
    pub id: GraphId,
    pub nodes: HashMap<NodeId, ConceptNode>,
    pub edges: HashMap<EdgeId, ConceptEdge>,
}

/// A node in a ConceptGraph - which can itself be a graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptNode {
    /// A simple value or concept
    Atom {
        id: NodeId,
        value: serde_json::Value,
        position: Position3D,
    },

    /// A nested graph
    Graph {
        id: NodeId,
        graph: Box<ConceptGraph>,
        position: Position3D,
    },
}

impl ConceptNode {
    pub fn id(&self) -> NodeId {
        match self {
            ConceptNode::Atom { id, .. } => *id,
            ConceptNode::Graph { id, .. } => *id,
        }
    }

    pub fn position(&self) -> Position3D {
        match self {
            ConceptNode::Atom { position, .. } => *position,
            ConceptNode::Graph { position, .. } => *position,
        }
    }

    /// Check if this node is a graph
    pub fn is_graph(&self) -> bool {
        matches!(self, ConceptNode::Graph { .. })
    }

    /// Get the nested graph if this node is a graph
    pub fn as_graph(&self) -> Option<&ConceptGraph> {
        match self {
            ConceptNode::Graph { graph, .. } => Some(graph),
            _ => None,
        }
    }
}

/// An edge connecting nodes in a ConceptGraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptEdge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub relationship: RelatedBy,
}

impl ConceptGraph {
    /// Create a new empty ConceptGraph
    pub fn new() -> Self {
        Self {
            id: GraphId::new(),
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Add an atomic node
    pub fn add_atom(&mut self, value: serde_json::Value, position: Position3D) -> NodeId {
        let id = NodeId::new();
        let node = ConceptNode::Atom { id, value, position };
        self.nodes.insert(id, node);
        id
    }

    /// Add a graph as a node
    pub fn add_graph(&mut self, graph: ConceptGraph, position: Position3D) -> NodeId {
        let id = NodeId::new();
        let node = ConceptNode::Graph {
            id,
            graph: Box::new(graph),
            position
        };
        self.nodes.insert(id, node);
        id
    }

    /// Connect two nodes
    pub fn connect(&mut self, source: NodeId, target: NodeId, relationship: RelatedBy) -> Result<EdgeId, ConceptGraphError> {
        // Verify nodes exist
        if !self.nodes.contains_key(&source) {
            return Err(ConceptGraphError::NodeNotFound(source));
        }
        if !self.nodes.contains_key(&target) {
            return Err(ConceptGraphError::NodeNotFound(target));
        }

        let edge_id = EdgeId::new();
        let edge = ConceptEdge {
            id: edge_id,
            source,
            target,
            relationship,
        };

        self.edges.insert(edge_id, edge);
        Ok(edge_id)
    }

    /// Traverse the graph recursively
    pub fn traverse<F>(&self, visitor: &mut F)
    where
        F: FnMut(&ConceptGraph, usize), // graph, depth
    {
        self.traverse_internal(visitor, 0);
    }

    fn traverse_internal<F>(&self, visitor: &mut F, depth: usize)
    where
        F: FnMut(&ConceptGraph, usize),
    {
        // Visit this graph
        visitor(self, depth);

        // Visit nested graphs
        for node in self.nodes.values() {
            if let ConceptNode::Graph { graph, .. } = node {
                graph.traverse_internal(visitor, depth + 1);
            }
        }
    }

    /// Count total nodes including nested graphs
    pub fn total_node_count(&self) -> usize {
        let mut count = 0;
        self.traverse(&mut |graph, _| {
            count += graph.nodes.len();
        });
        count
    }

    /// Find all graphs at a specific depth
    pub fn graphs_at_depth(&self, target_depth: usize) -> Vec<&ConceptGraph> {
        let mut graphs = Vec::new();
        self.traverse(&mut |graph, depth| {
            if depth == target_depth {
                graphs.push(graph);
            }
        });
        graphs
    }
}

/// Example: Invoice as a ConceptGraph
impl ConceptGraph {
    /// Create an Invoice concept graph
    pub fn invoice_example() -> Self {
        let mut invoice = ConceptGraph::new();

        // Create buyer graph (Party from party domain)
        let mut buyer = ConceptGraph::new();
        buyer.add_atom(serde_json::json!({
            "name": "Acme Corp",
            "type": "Organization"
        }), Position3D::default());

        // Create seller graph (Party from party domain)
        let mut seller = ConceptGraph::new();
        seller.add_atom(serde_json::json!({
            "name": "Widgets Inc",
            "type": "Organization"
        }), Position3D::default());

        // Create line items graph
        let mut line_items = ConceptGraph::new();
        let item1 = line_items.add_atom(serde_json::json!({
            "product": "Widget A",
            "quantity": 10,
            "price": 9.99
        }), Position3D::default());

        // Add these as nodes in the invoice
        let buyer_id = invoice.add_graph(buyer, Position3D::new(0.0, 0.0, 0.0).unwrap());
        let seller_id = invoice.add_graph(seller, Position3D::new(10.0, 0.0, 0.0).unwrap());
        let items_id = invoice.add_graph(line_items, Position3D::new(5.0, 5.0, 0.0).unwrap());

        // Connect them
        invoice.connect(buyer_id, items_id, RelatedBy::Contains).unwrap();
        invoice.connect(seller_id, items_id, RelatedBy::Produces).unwrap();

        invoice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recursive_graph_structure() {
        let mut root = ConceptGraph::new();

        // Create a nested graph
        let mut nested = ConceptGraph::new();
        nested.add_atom(serde_json::json!("value"), Position3D::default());

        // Add the nested graph as a node
        let nested_id = root.add_graph(nested, Position3D::default());

        // Verify structure
        assert_eq!(root.nodes.len(), 1);
        assert!(root.nodes.get(&nested_id).unwrap().is_graph());

        // Count total nodes (1 in root + 1 in nested)
        assert_eq!(root.total_node_count(), 2);
    }

    #[test]
    fn test_invoice_structure() {
        let invoice = ConceptGraph::invoice_example();

        // Invoice has 3 nodes (buyer, seller, line_items)
        assert_eq!(invoice.nodes.len(), 3);

        // All nodes are graphs
        for node in invoice.nodes.values() {
            assert!(node.is_graph());
        }

        // Has 2 edges
        assert_eq!(invoice.edges.len(), 2);
    }
}
