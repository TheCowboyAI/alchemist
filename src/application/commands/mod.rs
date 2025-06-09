#[derive(Debug, Clone)]
pub enum GraphCommand {
    CreateGraph {
        id: GraphId,
        name: String,
        metadata: HashMap<String, Value>,
    },
    CreateConceptualGraph {
        graph_id: GraphId,
        name: String,
        category_type: crate::domain::conceptual_graph::CategoryType,
    },
    AddConceptualNode {
        graph_id: GraphId,
        node_id: NodeId,
        concept_type: crate::domain::conceptual_graph::ConceptType,
        conceptual_point: crate::domain::conceptual_graph::ConceptualPoint,
    },
    ApplyGraphMorphism {
        source_graph: GraphId,
        target_graph: GraphId,
        morphism: crate::domain::conceptual_graph::GraphMorphism,
    },
}
