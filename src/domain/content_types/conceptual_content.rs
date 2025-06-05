//! Conceptual space content type for CIM-IPLD

use cim_ipld::{ContentType, TypedContent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Content representing conceptual space structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptualSpaceContent {
    /// Unique identifier for the conceptual space
    pub id: String,
    /// Name of the conceptual space
    pub name: String,
    /// Dimension definitions
    pub dimensions: Vec<ConceptualDimension>,
    /// Points in the conceptual space
    pub points: Vec<ConceptualPoint>,
    /// Similarity threshold for clustering
    pub similarity_threshold: f64,
}

/// A dimension in conceptual space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptualDimension {
    /// Dimension name
    pub name: String,
    /// Dimension description
    pub description: String,
    /// Range of values (min, max)
    pub range: (f64, f64),
    /// Whether this dimension is cyclic
    pub cyclic: bool,
}

/// A point in conceptual space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptualPoint {
    /// Entity ID this point represents
    pub entity_id: String,
    /// Coordinates in each dimension
    pub coordinates: Vec<f64>,
    /// Optional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TypedContent for ConceptualSpaceContent {
    const CODEC: u64 = 0x300103;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300103);
}

impl ConceptualSpaceContent {
    /// Create a new conceptual space
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            dimensions: Vec::new(),
            points: Vec::new(),
            similarity_threshold: 0.8,
        }
    }

    /// Add a dimension
    pub fn add_dimension(&mut self, dimension: ConceptualDimension) {
        self.dimensions.push(dimension);
    }

    /// Add a point
    pub fn add_point(&mut self, point: ConceptualPoint) {
        self.points.push(point);
    }

    /// Calculate distance between two points
    pub fn distance(&self, p1: &ConceptualPoint, p2: &ConceptualPoint) -> f64 {
        p1.coordinates
            .iter()
            .zip(&p2.coordinates)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    /// Find similar points to a given point
    pub fn find_similar(&self, point: &ConceptualPoint, threshold: f64) -> Vec<&ConceptualPoint> {
        self.points
            .iter()
            .filter(|p| {
                p.entity_id != point.entity_id && self.distance(point, p) <= threshold
            })
            .collect()
    }
}
