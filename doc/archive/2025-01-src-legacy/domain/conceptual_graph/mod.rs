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
pub use composition::{CompositionBuilder, CompositionOperation, GraphComposer};
pub use concept::{
    ConceptEdge, ConceptGraph, ConceptId, ConceptNode, ConceptRelationship, ConceptType, EdgeId,
    NodeId,
};
pub use context_bridge::{
    ConceptTransformation, ContextBridge, ContextBridgeId, ContextMappingType, InterfaceContract,
    PublishedLanguage, TranslationDirection, TranslationGraph, TranslationRule,
};
pub use metric_context::{
    ConceptCluster, ConsumptionFunction, CostFunction, DelayFunction, DistanceFunction,
    MetricContext, MetricContextId, MetricSpace, MetricType, Path, ProbabilityFunction,
    ResourceType,
};
pub use morphism::{GraphMorphism, InjectionMap, MorphismType, ProductType};
pub use quality_dimension::{ConceptualPoint, DimensionType, DistanceMetric, QualityDimension};
pub use rule_context::{
    Action, BusinessRule, ComparisonOperator, ComplianceResult, Condition, ExportFormat,
    FactChange, FactReference, FactSet, FactValue, ImpactAnalysis, InferredFacts, LogicalOperator,
    NotificationSeverity, RuleContext, RuleContextId, RuleEvaluation, RuleId, RuleType,
    RuleViolation, ValidationResult,
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
        let dim = QualityDimension::new("Temperature", DimensionType::Continuous, 0.0..100.0);
        assert_eq!(dim.name, "Temperature");
        assert_eq!(dim.range.start, 0.0);
        assert_eq!(dim.range.end, 100.0);
    }
}
