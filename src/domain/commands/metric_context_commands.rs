use crate::domain::conceptual_graph::{ConceptId, MetricContext, MetricContextId, MetricType};
use serde::{Deserialize, Serialize};

/// Commands for metric context operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricContextCommand {
    /// Create a new metric context
    CreateMetricContext {
        name: String,
        base_context: ConceptId,
        metric_type: MetricType,
    },

    /// Set distance between two concepts
    SetDistance {
        context_id: MetricContextId,
        from: ConceptId,
        to: ConceptId,
        distance: f64,
    },

    /// Calculate shortest path between concepts
    CalculateShortestPath {
        context_id: MetricContextId,
        from: ConceptId,
        to: ConceptId,
    },

    /// Find nearest neighbors
    FindNearestNeighbors {
        context_id: MetricContextId,
        concept: ConceptId,
        k: usize,
    },

    /// Cluster concepts by distance
    ClusterByDistance {
        context_id: MetricContextId,
        threshold: f64,
    },

    /// Find concepts within radius
    FindWithinRadius {
        context_id: MetricContextId,
        center: ConceptId,
        radius: f64,
    },

    /// Update metric properties
    UpdateMetricProperties {
        context_id: MetricContextId,
        is_symmetric: Option<bool>,
        satisfies_triangle_inequality: Option<bool>,
        has_zero_self_distance: Option<bool>,
    },
}
