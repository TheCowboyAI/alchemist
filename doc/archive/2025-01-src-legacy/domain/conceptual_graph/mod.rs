//! Conceptual Graph Composition System
//!
//! A system where graphs are the fundamental building blocks of all domain models,
//! composed through Applied Category Theory (ACT) principles.

pub mod category;
pub mod composition;
pub mod concept;
pub mod context_bridge;
pub mod metric_context;
pub mod morphism;
pub mod quality_dimension;
pub mod rule_context;

// Re-export main types
pub use category::{CategoryType, EnrichmentType};
pub use composition::{CompositionOperation, GraphComposer, CompositionBuilder};
pub use concept::{ConceptGraph, ConceptNode, ConceptEdge, ConceptId, ConceptType, ConceptRelationship, NodeId, EdgeId};
pub use context_bridge::{
    ContextBridge, ContextBridgeId, ContextMappingType, TranslationRule,
    ConceptTransformation, TranslationDirection, InterfaceContract,
    TranslationGraph, PublishedLanguage
};
pub use metric_context::{
    ConceptCluster, ConsumptionFunction, CostFunction, DelayFunction, DistanceFunction,
    MetricContext, MetricContextId, MetricSpace, MetricType, Path, ProbabilityFunction,
    ResourceType,
};
pub use morphism::{GraphMorphism, MorphismType, ProductType, InjectionMap};
pub use quality_dimension::{QualityDimension, DimensionType, DistanceMetric, ConceptualPoint};
pub use rule_context::{
    RuleContext, RuleContextId, BusinessRule, RuleId, RuleType, Condition, Action,
    FactSet, FactValue, FactReference, ComparisonOperator, LogicalOperator,
    NotificationSeverity, RuleEvaluation, ComplianceResult, InferredFacts,
    ImpactAnalysis, ExportFormat, ValidationResult, FactChange, RuleViolation
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_graph_creation() {
        let graph = ConceptGraph::new("Test Graph".to_string());
        assert_eq!(graph.name, "Test Graph");
        assert_eq!(graph.structure.node_count(), 0);
        assert_eq!(graph.structure.edge_count(), 0);
    }

    #[test]
    fn test_quality_dimension() {
        let dim = QualityDimension::new(
            "Temperature",
            DimensionType::Continuous,
            0.0..100.0,
        );
        assert_eq!(dim.name, "Temperature");
        assert_eq!(dim.range.start, 0.0);
        assert_eq!(dim.range.end, 100.0);
    }
}
