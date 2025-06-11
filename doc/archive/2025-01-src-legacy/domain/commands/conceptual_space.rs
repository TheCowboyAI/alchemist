//! Conceptual Space Commands

use crate::domain::{
    value_objects::{ConceptualSpaceId, DimensionId, RegionId, UserId},
    conceptual_graph::{QualityDimension, DistanceMetric, ConceptualPoint, ConceptId},
};
use serde::{Deserialize, Serialize};

/// Commands for conceptual space operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptualSpaceCommand {
    CreateConceptualSpace(CreateConceptualSpace),
    AddQualityDimension(AddQualityDimension),
    MapConcept(MapConcept),
    DefineRegion(DefineRegion),
    CalculateSimilarity(CalculateSimilarity),
    UpdateMetric(UpdateMetric),
}

/// Create a new conceptual space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConceptualSpace {
    pub space_id: ConceptualSpaceId,
    pub name: String,
    pub description: String,
    pub created_by: UserId,
}

/// Add a quality dimension to the space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddQualityDimension {
    pub space_id: ConceptualSpaceId,
    pub dimension_id: DimensionId,
    pub dimension: QualityDimension,
}

/// Map a concept to a position in the space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConcept {
    pub space_id: ConceptualSpaceId,
    pub concept_id: ConceptId,
    pub position: ConceptualPoint,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Define a convex region in the space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefineRegion {
    pub space_id: ConceptualSpaceId,
    pub region_id: RegionId,
    pub name: String,
    pub prototype: ConceptualPoint,
    pub member_concepts: Vec<ConceptId>,
}

/// Calculate similarity between two concepts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateSimilarity {
    pub space_id: ConceptualSpaceId,
    pub concept1: ConceptId,
    pub concept2: ConceptId,
}

/// Update the distance metric for the space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMetric {
    pub space_id: ConceptualSpaceId,
    pub metric: DistanceMetric,
}
