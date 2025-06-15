//! Category types from Applied Category Theory
//!
//! Based on "Seven Sketches in Compositionality" - the fundamental
//! category theory structures used for graph composition.

use serde::{Deserialize, Serialize};

/// Category types from Applied Category Theory
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CategoryType {
    /// Order (Poset) - hierarchical relationships
    Order,

    /// Database (Schema) - structured data relationships
    Database,

    /// Monoidal Category - parallel composition
    Monoidal,

    /// Profunctor - relationships between categories
    Profunctor,

    /// Enriched Category - categories with additional structure
    Enriched { enrichment: EnrichmentType },

    /// Topos - categories with logic and computation
    Topos,

    /// Operad - compositional patterns
    Operad,

    /// Simple Category - basic objects and morphisms
    Simple,

    /// Functor Category - categories of functors
    Functor,

    /// Slice Category - objects over a fixed object
    Slice { base_object: String },
}

impl Default for CategoryType {
    fn default() -> Self {
        CategoryType::Database
    }
}

impl CategoryType {
    /// Check if this category supports parallel composition
    pub fn supports_parallel_composition(&self) -> bool {
        matches!(self, CategoryType::Monoidal | CategoryType::Operad)
    }

    /// Check if this category has logical structure
    pub fn has_logic(&self) -> bool {
        matches!(self, CategoryType::Topos)
    }

    /// Check if this category has ordering
    pub fn has_ordering(&self) -> bool {
        matches!(self, CategoryType::Order)
    }

    /// Get a description of the category type
    pub fn description(&self) -> &'static str {
        match self {
            CategoryType::Order => "Ordered sets with monotone functions",
            CategoryType::Database => "Database schemas with queries",
            CategoryType::Monoidal => "Categories with tensor product for parallel composition",
            CategoryType::Profunctor => "Bridges between different categories",
            CategoryType::Enriched { .. } => "Categories enriched over another category",
            CategoryType::Topos => "Categories with logic and subobject classifiers",
            CategoryType::Operad => "Compositional patterns and operations",
            CategoryType::Simple => "Basic category with objects and morphisms",
            CategoryType::Functor => "Categories where objects are functors",
            CategoryType::Slice { .. } => "Category of objects over a fixed base",
        }
    }
}

/// Types of enrichment for enriched categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnrichmentType {
    /// Enriched over sets (ordinary category)
    Set,

    /// Enriched over metric spaces (distances between morphisms)
    Metric,

    /// Enriched over vector spaces (linear combinations of morphisms)
    Vector,

    /// Enriched over truth values (fuzzy relationships)
    Truth,

    /// Enriched over costs (resource-aware composition)
    Cost,

    /// Enriched over probabilities (stochastic relationships)
    Probability,

    /// Custom enrichment
    Custom(String),
}

/// Properties that a category might have
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryProperties {
    /// Has identity morphisms
    pub has_identity: bool,

    /// Morphisms can be composed
    pub has_composition: bool,

    /// Has all products
    pub has_products: bool,

    /// Has all coproducts
    pub has_coproducts: bool,

    /// Has all limits
    pub is_complete: bool,

    /// Has all colimits
    pub is_cocomplete: bool,

    /// Is a cartesian closed category
    pub is_cartesian_closed: bool,

    /// Is an abelian category
    pub is_abelian: bool,
}

impl Default for CategoryProperties {
    fn default() -> Self {
        Self {
            has_identity: true,
            has_composition: true,
            has_products: false,
            has_coproducts: false,
            is_complete: false,
            is_cocomplete: false,
            is_cartesian_closed: false,
            is_abelian: false,
        }
    }
}

impl CategoryProperties {
    /// Create properties for a simple category
    pub fn simple() -> Self {
        Self::default()
    }

    /// Create properties for a complete category
    pub fn complete() -> Self {
        Self {
            has_identity: true,
            has_composition: true,
            has_products: true,
            has_coproducts: true,
            is_complete: true,
            is_cocomplete: true,
            is_cartesian_closed: false,
            is_abelian: false,
        }
    }

    /// Create properties for a topos
    pub fn topos() -> Self {
        Self {
            has_identity: true,
            has_composition: true,
            has_products: true,
            has_coproducts: true,
            is_complete: true,
            is_cocomplete: true,
            is_cartesian_closed: true,
            is_abelian: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_type_properties() {
        assert!(CategoryType::Monoidal.supports_parallel_composition());
        assert!(CategoryType::Operad.supports_parallel_composition());
        assert!(!CategoryType::Order.supports_parallel_composition());

        assert!(CategoryType::Topos.has_logic());
        assert!(!CategoryType::Simple.has_logic());

        assert!(CategoryType::Order.has_ordering());
        assert!(!CategoryType::Monoidal.has_ordering());
    }

    #[test]
    fn test_category_properties() {
        let simple = CategoryProperties::simple();
        assert!(simple.has_identity);
        assert!(simple.has_composition);
        assert!(!simple.has_products);

        let complete = CategoryProperties::complete();
        assert!(complete.has_products);
        assert!(complete.has_coproducts);
        assert!(complete.is_complete);

        let topos = CategoryProperties::topos();
        assert!(topos.is_cartesian_closed);
        assert!(topos.is_complete);
    }

    #[test]
    fn test_enrichment_types() {
        let metric_enriched = CategoryType::Enriched {
            enrichment: EnrichmentType::Metric,
        };

        assert_eq!(
            metric_enriched.description(),
            "Categories enriched over another category"
        );
    }
}
