//! Quality dimensions for conceptual spaces
//!
//! Quality dimensions define the geometric structure of conceptual spaces
//! where concepts are positioned based on their properties.

use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Represents a quality dimension in a conceptual space
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityDimension {
    /// Name of the dimension
    pub name: String,

    /// Type of dimension
    pub dimension_type: DimensionType,

    /// Valid range of values
    pub range: Range<f64>,

    /// Distance metric for this dimension
    pub metric: DistanceMetric,

    /// Weight for this dimension in distance calculations
    pub weight: f64,
}

impl QualityDimension {
    /// Create a new quality dimension
    pub fn new(name: impl Into<String>, dimension_type: DimensionType, range: Range<f64>) -> Self {
        let metric = DistanceMetric::default_for_type(&dimension_type);
        Self {
            name: name.into(),
            dimension_type,
            range,
            metric,
            weight: 1.0,
        }
    }

    /// Set the distance metric
    pub fn with_metric(mut self, metric: DistanceMetric) -> Self {
        self.metric = metric;
        self
    }

    /// Set the weight
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Check if a value is within the valid range
    pub fn is_valid(&self, value: f64) -> bool {
        value >= self.range.start && value < self.range.end
    }

    /// Normalize a value to [0, 1] range
    pub fn normalize(&self, value: f64) -> f64 {
        if self.range.start == self.range.end {
            return 0.0;
        }
        (value - self.range.start) / (self.range.end - self.range.start)
    }

    /// Denormalize a value from [0, 1] to the dimension's range
    pub fn denormalize(&self, normalized: f64) -> f64 {
        self.range.start + normalized * (self.range.end - self.range.start)
    }
}

/// Type of quality dimension
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DimensionType {
    /// Continuous numeric dimension
    Continuous,
    /// Discrete categorical dimension
    Categorical,
    /// Ordered discrete dimension
    Ordinal,
    /// Circular dimension (e.g., hue, angle)
    Circular,
    /// Binary values (e.g., true/false)
    Binary,
    /// Interval values with meaningful differences but no true zero
    Interval,
    /// Ratio values with true zero and meaningful ratios
    Ratio,
}

impl DimensionType {
    /// Check if the dimension type supports continuous values
    pub fn is_continuous(&self) -> bool {
        matches!(self, Self::Continuous | Self::Interval | Self::Ratio | Self::Circular)
    }

    /// Check if the dimension type is discrete
    pub fn is_discrete(&self) -> bool {
        matches!(self, Self::Categorical | Self::Ordinal | Self::Binary)
    }
}

/// Distance metrics for measuring similarity in conceptual space
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DistanceMetric {
    /// Standard Euclidean distance
    Euclidean,
    /// Manhattan (city-block) distance
    Manhattan,
    /// Cosine similarity
    Cosine,
    /// Custom metric with a name
    Custom(String),
}

impl DistanceMetric {
    /// Get the default metric for a dimension type
    pub fn default_for_type(dimension_type: &DimensionType) -> Self {
        match dimension_type {
            DimensionType::Continuous | DimensionType::Interval | DimensionType::Ratio => {
                DistanceMetric::Euclidean
            }
            DimensionType::Categorical | DimensionType::Binary => DistanceMetric::Cosine,
            DimensionType::Ordinal => DistanceMetric::Manhattan,
            DimensionType::Circular => DistanceMetric::Cosine,
        }
    }

    /// Calculate distance between two values using this metric
    pub fn distance(&self, a: f64, b: f64) -> f64 {
        match self {
            DistanceMetric::Euclidean => (a - b).abs(),
            DistanceMetric::Manhattan => (a - b).abs(),
            DistanceMetric::Cosine => {
                // Handle zero values
                if a == 0.0 && b == 0.0 {
                    return 0.0; // Both zero vectors have distance 0
                }
                if a == 0.0 || b == 0.0 {
                    return 1.0; // Maximum distance for orthogonal vectors
                }

                // Normalize vectors to unit length
                let a_norm = a.abs();
                let b_norm = b.abs();
                let dot_product = a * b / (a_norm * b_norm);
                // Cosine distance = 1 - cosine similarity
                1.0 - dot_product.min(1.0).max(-1.0)
            }
            DistanceMetric::Custom(_) => (a - b).abs(), // Default to Euclidean
        }
    }
}

/// A point in conceptual space
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptualPoint {
    /// Values for each dimension
    pub coordinates: Vec<f64>,
}

impl ConceptualPoint {
    /// Create a new point in conceptual space
    pub fn new(coordinates: Vec<f64>) -> Self {
        Self { coordinates }
    }

    /// Get the coordinates
    pub fn coordinates(&self) -> &Vec<f64> {
        &self.coordinates
    }

    /// Calculate distance to another point given quality dimensions
    pub fn distance_to(&self, other: &ConceptualPoint, dimensions: &[QualityDimension]) -> f64 {
        if self.coordinates.len() != other.coordinates.len()
            || self.coordinates.len() != dimensions.len() {
            return f64::INFINITY;
        }

        let mut sum = 0.0;

        for i in 0..self.coordinates.len() {
            let dim = &dimensions[i];
            let dist = dim.metric.distance(self.coordinates[i], other.coordinates[i]);

            match dim.metric {
                DistanceMetric::Euclidean => sum += (dist * dim.weight).powi(2),
                DistanceMetric::Manhattan => sum += dist * dim.weight,
                DistanceMetric::Cosine => sum += dist * dim.weight,
                _ => sum += dist * dim.weight,
            }
        }

        match dimensions.first().map(|d| &d.metric) {
            Some(DistanceMetric::Euclidean) => sum.sqrt(),
            _ => sum,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_dimension_creation() {
        let dim = QualityDimension::new("temperature", DimensionType::Continuous, 0.0..100.0);
        assert_eq!(dim.name, "temperature");
        assert_eq!(dim.dimension_type, DimensionType::Continuous);
        assert_eq!(dim.range.start, 0.0);
        assert_eq!(dim.range.end, 100.0);
        assert_eq!(dim.weight, 1.0);
    }

    #[test]
    fn test_dimension_validation() {
        let dim = QualityDimension::new("size", DimensionType::Continuous, 0.0..10.0);
        assert!(dim.is_valid(5.0));
        assert!(dim.is_valid(0.0));
        assert!(!dim.is_valid(-1.0));
        assert!(!dim.is_valid(10.0));
    }

    #[test]
    fn test_normalization() {
        let dim = QualityDimension::new("value", DimensionType::Continuous, 10.0..20.0);
        assert_eq!(dim.normalize(10.0), 0.0);
        assert_eq!(dim.normalize(15.0), 0.5);
        assert_eq!(dim.normalize(20.0), 1.0);

        assert_eq!(dim.denormalize(0.0), 10.0);
        assert_eq!(dim.denormalize(0.5), 15.0);
        assert_eq!(dim.denormalize(1.0), 20.0);
    }

    #[test]
    fn test_distance_metrics() {
        assert_eq!(DistanceMetric::Euclidean.distance(3.0, 7.0), 4.0);
        assert_eq!(DistanceMetric::Manhattan.distance(3.0, 7.0), 4.0);

        // Cosine distance tests
        assert_eq!(DistanceMetric::Cosine.distance(1.0, 1.0), 0.0); // Same direction = 0 distance
        assert_eq!(DistanceMetric::Cosine.distance(1.0, 0.0), 1.0); // Orthogonal = max distance
        assert_eq!(DistanceMetric::Cosine.distance(0.0, 0.0), 0.0); // Both zero = 0 distance
        assert_eq!(DistanceMetric::Cosine.distance(1.0, -1.0), 2.0); // Opposite direction = 2.0
    }

    #[test]
    fn test_conceptual_point_distance() {
        let dimensions = vec![
            QualityDimension::new("x", DimensionType::Continuous, 0.0..10.0),
            QualityDimension::new("y", DimensionType::Continuous, 0.0..10.0),
        ];

        let p1 = ConceptualPoint::new(vec![0.0, 0.0]);
        let p2 = ConceptualPoint::new(vec![3.0, 4.0]);

        let distance = p1.distance_to(&p2, &dimensions);
        assert_eq!(distance, 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_weighted_distance() {
        let dimensions = vec![
            QualityDimension::new("x", DimensionType::Continuous, 0.0..10.0)
                .with_weight(2.0),
            QualityDimension::new("y", DimensionType::Continuous, 0.0..10.0)
                .with_weight(1.0),
        ];

        let p1 = ConceptualPoint::new(vec![0.0, 0.0]);
        let p2 = ConceptualPoint::new(vec![1.0, 1.0]);

        let distance = p1.distance_to(&p2, &dimensions);
        // sqrt((2*1)^2 + (1*1)^2) = sqrt(4 + 1) = sqrt(5)
        assert!((distance - 5.0_f64.sqrt()).abs() < 0.0001);
    }
}
