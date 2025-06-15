//! Conceptual Space Events

use crate::domain::{
    conceptual_graph::{ConceptId, ConceptualPoint, DistanceMetric, QualityDimension},
    value_objects::{ConceptualSpaceId, DimensionId, RegionId, UserId},
};
use serde::{Deserialize, Serialize};

/// A new conceptual space was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptualSpaceCreated {
    pub space_id: ConceptualSpaceId,
    pub name: String,
    pub description: String,
    pub created_by: UserId,
}

/// A quality dimension was added to the space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDimensionAdded {
    pub space_id: ConceptualSpaceId,
    pub dimension_id: DimensionId,
    pub dimension: QualityDimension,
}

/// A concept was mapped to a position in the space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMapped {
    pub space_id: ConceptualSpaceId,
    pub concept_id: ConceptId,
    pub position: ConceptualPoint,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// A convex region was defined in the space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionDefined {
    pub space_id: ConceptualSpaceId,
    pub region_id: RegionId,
    pub name: String,
    pub prototype: ConceptualPoint,
    pub member_concepts: Vec<ConceptId>,
}

/// Similarity between two concepts was calculated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityCalculated {
    pub space_id: ConceptualSpaceId,
    pub concept1: ConceptId,
    pub concept2: ConceptId,
    pub similarity: f32,
    pub metric: DistanceMetric,
}

/// The distance metric for the space was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricUpdated {
    pub space_id: ConceptualSpaceId,
    pub old_metric: DistanceMetric,
    pub new_metric: DistanceMetric,
}
