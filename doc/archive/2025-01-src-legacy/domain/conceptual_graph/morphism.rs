//! Graph morphisms for composition
//!
//! Morphisms are structure-preserving maps between graphs that enable
//! composition and transformation of concepts.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::domain::conceptual_graph::concept::{ConceptId, NodeId, EdgeId};

/// Types of graph morphisms
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MorphismType {
    /// Structure-preserving map
    Homomorphism,
    /// Injective homomorphism (one-to-one)
    Monomorphism,
    /// Surjective homomorphism (onto)
    Epimorphism,
    /// Bijective homomorphism (one-to-one and onto)
    Isomorphism,
    /// Embedding one graph into another
    Embedding,
    /// Quotient by equivalence relation
    Quotient,
    /// Product of graphs
    Product,
    /// Coproduct (disjoint union)
    Coproduct,
    /// Pullback (fiber product)
    Pullback,
    /// Pushout (amalgamated sum)
    Pushout,
}

/// A morphism between concept graphs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GraphMorphism {
    /// Structure-preserving map between graphs
    Homomorphism {
        source_id: ConceptId,
        target_id: ConceptId,
        node_map: HashMap<NodeId, NodeId>,
        edge_map: HashMap<EdgeId, EdgeId>,
        morphism_type: MorphismType,
    },

    /// Embedding one graph into another
    Embedding {
        subgraph_id: ConceptId,
        host_id: ConceptId,
        injection: InjectionMap,
    },

    /// Quotient - collapsing parts of a graph
    Quotient {
        graph_id: ConceptId,
        equivalence: EquivalenceRelation,
    },

    /// Product - combining graphs
    Product {
        left_id: ConceptId,
        right_id: ConceptId,
        product_type: ProductType,
    },

    /// Coproduct - disjoint union
    Coproduct {
        component_ids: Vec<ConceptId>,
    },

    /// Functor - maps between categories
    Functor {
        source_category: ConceptId,
        target_category: ConceptId,
        object_map: HashMap<ConceptId, ConceptId>,
        morphism_map: HashMap<MorphismId, MorphismId>,
    },
}

/// Injection map for embeddings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InjectionMap {
    /// Maps nodes from subgraph to host graph
    pub node_injections: HashMap<NodeId, NodeId>,

    /// Maps edges from subgraph to host graph
    pub edge_injections: HashMap<EdgeId, EdgeId>,

    /// Position offset in conceptual space
    pub position_offset: Vec<f64>,
}

impl InjectionMap {
    pub fn new() -> Self {
        Self {
            node_injections: HashMap::new(),
            edge_injections: HashMap::new(),
            position_offset: Vec::new(),
        }
    }

    pub fn add_node_injection(&mut self, from: NodeId, to: NodeId) {
        self.node_injections.insert(from, to);
    }

    pub fn add_edge_injection(&mut self, from: EdgeId, to: EdgeId) {
        self.edge_injections.insert(from, to);
    }
}

/// Represents an equivalence relation for quotient operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceRelation {
    /// Name of the relation
    pub name: String,

    /// Equivalence classes (for serialization)
    pub equivalence_classes: Vec<Vec<NodeId>>,

    /// Representative for each class
    pub representatives: HashMap<NodeId, NodeId>,
}

impl PartialEq for EquivalenceRelation {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name &&
        self.equivalence_classes == other.equivalence_classes &&
        self.representatives == other.representatives
    }
}

impl EquivalenceRelation {
    /// Create a new equivalence relation
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            equivalence_classes: Vec::new(),
            representatives: HashMap::new(),
        }
    }

    /// Add an equivalence class with a representative
    pub fn add_class(&mut self, class: Vec<NodeId>, representative: NodeId) {
        // Update representatives for all nodes in the class
        for node in &class {
            self.representatives.insert(*node, representative);
        }
        self.equivalence_classes.push(class);
    }

    /// Get the representative for a node
    pub fn get_representative(&self, node: &NodeId) -> Option<NodeId> {
        self.representatives.get(node).copied()
    }
}

/// Types of graph products
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductType {
    /// Cartesian product
    Cartesian,
    /// Tensor product (for monoidal categories)
    Tensor,
    /// Strong product
    Strong,
    /// Lexicographic product
    Lexicographic,
}

/// Identifier for morphisms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MorphismId(uuid::Uuid);

impl MorphismId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for MorphismId {
    fn default() -> Self {
        Self::new()
    }
}

/// Properties of a morphism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphismProperties {
    /// Is the morphism injective (one-to-one)?
    pub is_injective: bool,

    /// Is the morphism surjective (onto)?
    pub is_surjective: bool,

    /// Does the morphism preserve structure?
    pub preserves_structure: bool,

    /// Does the morphism preserve limits?
    pub preserves_limits: bool,

    /// Does the morphism preserve colimits?
    pub preserves_colimits: bool,
}

impl MorphismProperties {
    /// Check if the morphism is an isomorphism
    pub fn is_isomorphism(&self) -> bool {
        self.is_injective && self.is_surjective && self.preserves_structure
    }

    /// Check if the morphism is a monomorphism
    pub fn is_monomorphism(&self) -> bool {
        self.is_injective
    }

    /// Check if the morphism is an epimorphism
    pub fn is_epimorphism(&self) -> bool {
        self.is_surjective
    }
}

/// Composition of morphisms
pub struct MorphismComposition;

impl MorphismComposition {
    /// Compose two morphisms f: A → B and g: B → C to get g∘f: A → C
    pub fn compose(f: &GraphMorphism, g: &GraphMorphism) -> Result<GraphMorphism, String> {
        match (f, g) {
            (
                GraphMorphism::Homomorphism {
                    target_id: f_target,
                    node_map: f_nodes,
                    edge_map: f_edges,
                    ..
                },
                GraphMorphism::Homomorphism {
                    source_id: g_source,
                    node_map: g_nodes,
                    edge_map: g_edges,
                    ..
                },
            ) => {
                // Check that f's target is g's source
                if f_target != g_source {
                    return Err("Cannot compose: target of f doesn't match source of g".to_string());
                }

                // Compose the maps
                let mut composed_nodes = HashMap::new();
                for (a, b) in f_nodes {
                    if let Some(c) = g_nodes.get(b) {
                        composed_nodes.insert(*a, *c);
                    }
                }

                let mut composed_edges = HashMap::new();
                for (a, b) in f_edges {
                    if let Some(c) = g_edges.get(b) {
                        composed_edges.insert(*a, *c);
                    }
                }

                Ok(GraphMorphism::Homomorphism {
                    source_id: match f {
                        GraphMorphism::Homomorphism { source_id, .. } => *source_id,
                        _ => unreachable!(),
                    },
                    target_id: match g {
                        GraphMorphism::Homomorphism { target_id, .. } => *target_id,
                        _ => unreachable!(),
                    },
                    node_map: composed_nodes,
                    edge_map: composed_edges,
                    morphism_type: MorphismType::Homomorphism,
                })
            }
            _ => Err("Composition not implemented for these morphism types".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injection_map() {
        let mut injection = InjectionMap::new();
        let from = NodeId::new();
        let to = NodeId::new();

        injection.add_node_injection(from, to);
        assert_eq!(injection.node_injections.get(&from), Some(&to));
    }

    #[test]
    fn test_equivalence_relation() {
        let mut equiv = EquivalenceRelation::new("test_equivalence");

        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let node3 = NodeId::new();

        equiv.add_class(vec![node1, node2, node3], node1);

        assert_eq!(equiv.get_representative(&node2), Some(node1));
        assert_eq!(equiv.get_representative(&node3), Some(node1));
        assert_eq!(equiv.equivalence_classes.len(), 1);
    }

    #[test]
    fn test_morphism_properties() {
        let props = MorphismProperties {
            is_injective: true,
            is_surjective: true,
            preserves_structure: true,
            preserves_limits: false,
            preserves_colimits: false,
        };

        assert!(props.is_isomorphism());
        assert!(props.is_monomorphism());
        assert!(props.is_epimorphism());
    }
}
