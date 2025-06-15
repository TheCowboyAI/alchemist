//! Conceptual Space Aggregate
//!
//! Represents a geometric space for organizing concepts based on quality dimensions.
//! Based on Peter Gärdenfors' conceptual spaces theory.

use crate::domain::{
    commands::conceptual_space::{
        AddQualityDimension, CalculateSimilarity, ConceptualSpaceCommand, CreateConceptualSpace,
        DefineRegion, MapConcept, UpdateMetric,
    },
    conceptual_graph::{ConceptId, ConceptualPoint, DistanceMetric, QualityDimension},
    events::{DomainEvent, conceptual_space::*},
    value_objects::{ConceptualSpaceId, DimensionId, RegionId, UserId},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConceptualSpaceError {
    #[error("Conceptual space not found")]
    NotFound,

    #[error("Dimension already exists: {0}")]
    DimensionAlreadyExists(String),

    #[error("Dimension not found: {0}")]
    DimensionNotFound(DimensionId),

    #[error("Concept already mapped: {0}")]
    ConceptAlreadyMapped(ConceptId),

    #[error("Concept not found: {0}")]
    ConceptNotFound(ConceptId),

    #[error("Region already exists: {0}")]
    RegionAlreadyExists(RegionId),

    #[error("Invalid coordinates: expected {expected} dimensions, got {actual}")]
    InvalidCoordinates { expected: usize, actual: usize },

    #[error("Invalid metric for dimension type")]
    InvalidMetric,
}

/// A convex region in conceptual space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvexRegion {
    pub id: RegionId,
    pub name: String,
    pub prototype: ConceptualPoint,
    pub member_concepts: Vec<ConceptId>,
}

/// Conceptual Space Aggregate
#[derive(Debug, Clone)]
pub struct ConceptualSpace {
    pub id: ConceptualSpaceId,
    pub name: String,
    pub description: String,
    pub created_by: UserId,
    pub dimensions: HashMap<DimensionId, QualityDimension>,
    pub concept_positions: HashMap<ConceptId, ConceptualPoint>,
    pub regions: HashMap<RegionId, ConvexRegion>,
    pub metric: DistanceMetric,
    pub version: u64,
}

impl ConceptualSpace {
    /// Create a new conceptual space
    pub fn new(
        id: ConceptualSpaceId,
        name: String,
        description: String,
        created_by: UserId,
    ) -> Self {
        Self {
            id,
            name,
            description,
            created_by,
            dimensions: HashMap::new(),
            concept_positions: HashMap::new(),
            regions: HashMap::new(),
            metric: DistanceMetric::Euclidean,
            version: 0,
        }
    }

    /// Handle a command
    pub fn handle_command(
        &mut self,
        command: ConceptualSpaceCommand,
    ) -> Result<Vec<DomainEvent>, ConceptualSpaceError> {
        match command {
            ConceptualSpaceCommand::CreateConceptualSpace(cmd) => self.handle_create_space(cmd),
            ConceptualSpaceCommand::AddQualityDimension(cmd) => self.handle_add_dimension(cmd),
            ConceptualSpaceCommand::MapConcept(cmd) => self.handle_map_concept(cmd),
            ConceptualSpaceCommand::DefineRegion(cmd) => self.handle_define_region(cmd),
            ConceptualSpaceCommand::CalculateSimilarity(cmd) => {
                self.handle_calculate_similarity(cmd)
            }
            ConceptualSpaceCommand::UpdateMetric(cmd) => self.handle_update_metric(cmd),
        }
    }

    fn handle_create_space(
        &mut self,
        cmd: CreateConceptualSpace,
    ) -> Result<Vec<DomainEvent>, ConceptualSpaceError> {
        let event = ConceptualSpaceCreated {
            space_id: cmd.space_id,
            name: cmd.name,
            description: cmd.description,
            created_by: cmd.created_by,
        };

        Ok(vec![DomainEvent::ConceptualSpaceCreated(event)])
    }

    fn handle_add_dimension(
        &mut self,
        cmd: AddQualityDimension,
    ) -> Result<Vec<DomainEvent>, ConceptualSpaceError> {
        // Check if dimension already exists
        if self
            .dimensions
            .values()
            .any(|d| d.name == cmd.dimension.name)
        {
            return Err(ConceptualSpaceError::DimensionAlreadyExists(
                cmd.dimension.name,
            ));
        }

        let event = QualityDimensionAdded {
            space_id: self.id,
            dimension_id: cmd.dimension_id,
            dimension: cmd.dimension,
        };

        Ok(vec![DomainEvent::QualityDimensionAdded(event)])
    }

    fn handle_map_concept(
        &mut self,
        cmd: MapConcept,
    ) -> Result<Vec<DomainEvent>, ConceptualSpaceError> {
        // Check if concept already mapped
        if self.concept_positions.contains_key(&cmd.concept_id) {
            return Err(ConceptualSpaceError::ConceptAlreadyMapped(cmd.concept_id));
        }

        // Validate coordinates match dimensions
        if cmd.position.coordinates.len() != self.dimensions.len() {
            return Err(ConceptualSpaceError::InvalidCoordinates {
                expected: self.dimensions.len(),
                actual: cmd.position.coordinates.len(),
            });
        }

        let event = ConceptMapped {
            space_id: self.id,
            concept_id: cmd.concept_id,
            position: cmd.position,
            metadata: cmd.metadata,
        };

        Ok(vec![DomainEvent::ConceptMapped(event)])
    }

    fn handle_define_region(
        &mut self,
        cmd: DefineRegion,
    ) -> Result<Vec<DomainEvent>, ConceptualSpaceError> {
        // Check if region already exists
        if self.regions.contains_key(&cmd.region_id) {
            return Err(ConceptualSpaceError::RegionAlreadyExists(cmd.region_id));
        }

        // Validate prototype coordinates
        if cmd.prototype.coordinates.len() != self.dimensions.len() {
            return Err(ConceptualSpaceError::InvalidCoordinates {
                expected: self.dimensions.len(),
                actual: cmd.prototype.coordinates.len(),
            });
        }

        let event = RegionDefined {
            space_id: self.id,
            region_id: cmd.region_id,
            name: cmd.name,
            prototype: cmd.prototype,
            member_concepts: cmd.member_concepts,
        };

        Ok(vec![DomainEvent::RegionDefined(event)])
    }

    fn handle_calculate_similarity(
        &mut self,
        cmd: CalculateSimilarity,
    ) -> Result<Vec<DomainEvent>, ConceptualSpaceError> {
        // Get concept positions
        let pos1 = self
            .concept_positions
            .get(&cmd.concept1)
            .ok_or(ConceptualSpaceError::ConceptNotFound(cmd.concept1))?;
        let pos2 = self
            .concept_positions
            .get(&cmd.concept2)
            .ok_or(ConceptualSpaceError::ConceptNotFound(cmd.concept2))?;

        // Calculate similarity based on distance
        let distance = self.calculate_distance(pos1, pos2)?;
        let similarity = 1.0 / (1.0 + distance);

        let event = SimilarityCalculated {
            space_id: self.id,
            concept1: cmd.concept1,
            concept2: cmd.concept2,
            similarity,
            metric: self.metric.clone(),
        };

        Ok(vec![DomainEvent::SimilarityCalculated(event)])
    }

    fn handle_update_metric(
        &mut self,
        cmd: UpdateMetric,
    ) -> Result<Vec<DomainEvent>, ConceptualSpaceError> {
        let event = MetricUpdated {
            space_id: self.id,
            old_metric: self.metric.clone(),
            new_metric: cmd.metric,
        };

        Ok(vec![DomainEvent::MetricUpdated(event)])
    }

    fn calculate_distance(
        &self,
        pos1: &ConceptualPoint,
        pos2: &ConceptualPoint,
    ) -> Result<f32, ConceptualSpaceError> {
        if pos1.coordinates.len() != pos2.coordinates.len() {
            return Err(ConceptualSpaceError::InvalidCoordinates {
                expected: pos1.coordinates.len(),
                actual: pos2.coordinates.len(),
            });
        }

        let distance = match &self.metric {
            DistanceMetric::Euclidean => {
                let sum: f64 = pos1
                    .coordinates
                    .iter()
                    .zip(&pos2.coordinates)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum();
                sum.sqrt() as f32
            }
            DistanceMetric::Manhattan => pos1
                .coordinates
                .iter()
                .zip(&pos2.coordinates)
                .map(|(a, b)| (a - b).abs())
                .sum::<f64>() as f32,
            DistanceMetric::Cosine => {
                // Implement cosine similarity
                let dot_product: f64 = pos1
                    .coordinates
                    .iter()
                    .zip(&pos2.coordinates)
                    .map(|(a, b)| a * b)
                    .sum();

                let magnitude1: f64 = pos1
                    .coordinates
                    .iter()
                    .map(|a| a.powi(2))
                    .sum::<f64>()
                    .sqrt();

                let magnitude2: f64 = pos2
                    .coordinates
                    .iter()
                    .map(|b| b.powi(2))
                    .sum::<f64>()
                    .sqrt();

                if magnitude1 == 0.0 || magnitude2 == 0.0 {
                    1.0 // Maximum distance for zero vectors
                } else {
                    let cosine_similarity = dot_product / (magnitude1 * magnitude2);
                    (1.0 - cosine_similarity) as f32
                }
            }
            DistanceMetric::Custom(_) => {
                // Default to Euclidean for custom metrics
                let sum: f64 = pos1
                    .coordinates
                    .iter()
                    .zip(&pos2.coordinates)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum();
                sum.sqrt() as f32
            }
        };

        Ok(distance)
    }

    /// Apply an event to update state
    pub fn apply_event(&mut self, event: &DomainEvent) -> Result<(), ConceptualSpaceError> {
        match event {
            DomainEvent::ConceptualSpaceCreated(e) => {
                self.id = e.space_id;
                self.name = e.name.clone();
                self.description = e.description.clone();
                self.created_by = e.created_by;
            }
            DomainEvent::QualityDimensionAdded(e) => {
                self.dimensions.insert(e.dimension_id, e.dimension.clone());
            }
            DomainEvent::ConceptMapped(e) => {
                self.concept_positions
                    .insert(e.concept_id, e.position.clone());
            }
            DomainEvent::RegionDefined(e) => {
                self.regions.insert(
                    e.region_id,
                    ConvexRegion {
                        id: e.region_id,
                        name: e.name.clone(),
                        prototype: e.prototype.clone(),
                        member_concepts: e.member_concepts.clone(),
                    },
                );
            }
            DomainEvent::SimilarityCalculated(_) => {
                // Similarity calculation is informational, no state change
            }
            DomainEvent::MetricUpdated(e) => {
                self.metric = e.new_metric.clone();
            }
            _ => {
                // Other events not handled by this aggregate
            }
        }

        self.version += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_space() -> ConceptualSpace {
        ConceptualSpace::new(
            ConceptualSpaceId(Uuid::new_v4()),
            "Test Space".to_string(),
            "A test conceptual space".to_string(),
            UserId(Uuid::new_v4()),
        )
    }

    #[test]
    fn test_create_conceptual_space() {
        let mut space = create_test_space();
        let cmd = CreateConceptualSpace {
            space_id: space.id,
            name: "Color Space".to_string(),
            description: "RGB color space".to_string(),
            created_by: space.created_by,
        };

        let events = space
            .handle_command(ConceptualSpaceCommand::CreateConceptualSpace(cmd))
            .unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::ConceptualSpaceCreated(_)));
    }

    #[test]
    fn test_add_quality_dimension() {
        let mut space = create_test_space();
        let dimension = QualityDimension {
            name: "Hue".to_string(),
            dimension_type: DimensionType::Circular,
            range: 0.0..360.0,
            metric: DistanceMetric::Euclidean,
            weight: 1.0,
        };

        let cmd = AddQualityDimension {
            space_id: space.id,
            dimension_id: DimensionId(Uuid::new_v4()),
            dimension,
        };

        let events = space
            .handle_command(ConceptualSpaceCommand::AddQualityDimension(cmd))
            .unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::QualityDimensionAdded(_)));
        assert_eq!(space.dimensions.len(), 1);
    }

    #[test]
    fn test_map_concept() {
        let mut space = create_test_space();

        // Add dimensions first
        let dim1 = QualityDimension {
            name: "X".to_string(),
            dimension_type: DimensionType::Continuous,
            range: 0.0..100.0,
        };
        space.dimensions.insert(DimensionId(Uuid::new_v4()), dim1);

        let position = ConceptualPoint {
            coordinates: vec![50.0],
        };

        let cmd = MapConcept {
            space_id: space.id,
            concept_id: ConceptId(Uuid::new_v4()),
            position,
        };

        let events = space
            .handle_command(ConceptualSpaceCommand::MapConcept(cmd))
            .unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::ConceptMapped(_)));
    }

    #[test]
    fn test_calculate_similarity() {
        let mut space = create_test_space();

        // Add dimension
        let dim = QualityDimension {
            name: "X".to_string(),
            dimension_type: DimensionType::Continuous,
            range: 0.0..100.0,
        };
        space.dimensions.insert(DimensionId(Uuid::new_v4()), dim);

        // Map two concepts
        let concept1 = ConceptId(Uuid::new_v4());
        let concept2 = ConceptId(Uuid::new_v4());

        space.concept_positions.insert(
            concept1,
            ConceptualPoint {
                coordinates: vec![0.0],
            },
        );
        space.concept_positions.insert(
            concept2,
            ConceptualPoint {
                coordinates: vec![10.0],
            },
        );

        let cmd = CalculateSimilarity {
            space_id: space.id,
            concept1,
            concept2,
        };

        let events = space
            .handle_command(ConceptualSpaceCommand::CalculateSimilarity(cmd))
            .unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::SimilarityCalculated(_)));
    }
}
