use crate::domain::conceptual_graph::{
    ConceptCluster, ConceptId, MetricContextId, MetricType, Path,
};
use serde::{Deserialize, Serialize};

/// Events related to metric context operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricContextEvent {
    /// A new metric context was created
    MetricContextCreated {
        context_id: MetricContextId,
        name: String,
        base_context: ConceptId,
        metric_type: MetricType,
    },

    /// Distance was set between concepts
    DistanceSet {
        context_id: MetricContextId,
        from: ConceptId,
        to: ConceptId,
        distance: f64,
    },

    /// Shortest path was calculated
    ShortestPathCalculated {
        context_id: MetricContextId,
        from: ConceptId,
        to: ConceptId,
        path: Path,
    },

    /// Nearest neighbors were found
    NearestNeighborsFound {
        context_id: MetricContextId,
        concept: ConceptId,
        neighbors: Vec<(ConceptId, f64)>,
    },

    /// Concepts were clustered
    ConceptsClustered {
        context_id: MetricContextId,
        threshold: f64,
        clusters: Vec<ConceptCluster>,
    },

    /// Concepts within radius were found
    ConceptsWithinRadiusFound {
        context_id: MetricContextId,
        center: ConceptId,
        radius: f64,
        concepts: Vec<ConceptId>,
    },

    /// Metric properties were updated
    MetricPropertiesUpdated {
        context_id: MetricContextId,
        is_symmetric: Option<bool>,
        satisfies_triangle_inequality: Option<bool>,
        has_zero_self_distance: Option<bool>,
    },
}
