use serde::{Deserialize, Serialize};
use std::fmt;
use crate::domain::value_objects::{Position3D, SubgraphId};

/// Represents the visual state of a subgraph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SubgraphState {
    /// Subgraph is fully expanded showing all nodes
    Expanded,
    /// Subgraph is collapsed to a single representative node
    Collapsed,
    /// Subgraph is transitioning between states
    Transitioning {
        /// Progress from 0.0 (start) to 1.0 (complete)
        progress: f32,
        /// Direction of transition
        from: Box<SubgraphState>,
        to: Box<SubgraphState>,
    },
}

/// Strategy for collapsing a subgraph
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CollapseStrategy {
    /// Collapse to centroid of all nodes
    Centroid,
    /// Collapse to position of most connected node
    MostConnected,
    /// Collapse to weighted center based on node importance
    WeightedCenter,
    /// Collapse to specific position
    FixedPosition(Position3D),
}

/// Strategy for expanding a subgraph
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LayoutStrategy {
    /// Force-directed layout
    ForceDirected {
        iterations: u32,
        spring_strength: f32,
        repulsion_strength: f32,
    },
    /// Hierarchical layout
    Hierarchical {
        direction: LayoutDirection,
        layer_spacing: f32,
        node_spacing: f32,
    },
    /// Circular layout
    Circular {
        radius: f32,
        start_angle: f32,
    },
    /// Grid layout
    Grid {
        columns: u32,
        spacing: f32,
    },
    // Equiangular Polygon Representation based on number of nodes
    Geometric {
        spacing: f32
    },
    /// Restore to previous positions
    RestorePrevious,
}

/// Direction for hierarchical layouts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutDirection {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

/// Strategy for merging subgraphs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Keep all nodes, merge boundaries
    Union,
    /// Create new layout for merged nodes
    Reflow,
    /// Maintain relative positions within subgraphs
    PreserveRelative,
    /// Optimize layout to minimize edge crossings
    OptimizeConnections,
}

/// Criteria for splitting a subgraph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SplitCriteria {
    /// Split by drawing a line
    GeometricLine {
        start: Position3D,
        end: Position3D,
    },
    /// Split by connectivity analysis
    Connectivity {
        min_cut: bool,
        max_components: usize,
    },
    /// Split by node attributes
    Attribute {
        attribute_name: String,
        split_values: Vec<String>,
    },
    /// Split by clustering algorithm
    Clustering {
        algorithm: ClusteringAlgorithm,
        num_clusters: usize,
    },
}

/// Clustering algorithms for splitting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusteringAlgorithm {
    KMeans,
    Hierarchical,
    DBSCAN,
    Spectral,
}

/// Type of subgraph for visualization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubgraphType {
    /// Software module or component
    Module,
    /// Cluster of related nodes
    Cluster,
    /// Namespace or package
    Namespace,
    /// Workflow or process
    Workflow,
    /// Conceptual region in knowledge space
    ConceptualRegion,
    /// User-defined type
    Custom(String),
}

/// Visual style configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubgraphStyle {
    pub base_color: Color,
    pub border_style: BorderStyle,
    pub fill_pattern: FillPattern,
    pub glow_intensity: f32,
    pub opacity: f32,
}

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Border style options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BorderStyle {
    Solid { width: f32 },
    Dashed { width: f32, dash_length: f32, gap_length: f32 },
    Dotted { width: f32, dot_spacing: f32 },
    Double { width: f32, gap: f32 },
    Gradient { start_color: Color, end_color: Color, width: f32 },
}

/// Fill pattern options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FillPattern {
    Solid,
    Gradient {
        start_color: Color,
        end_color: Color,
        angle: f32
    },
    Pattern {
        pattern_type: PatternType,
        scale: f32
    },
    Transparent { opacity: f32 },
}

/// Pattern types for fills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    Dots,
    Lines,
    CrossHatch,
    Checkerboard,
    Hexagons,
}

/// Metadata for a subgraph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubgraphMetadata {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub icon: Option<IconType>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub author: Option<String>,
    pub version: u32,
    pub custom_properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Icon types for subgraphs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IconType {
    Folder,
    Module,
    Cluster,
    Workflow,
    Database,
    Cloud,
    Custom(String),
}

/// Statistics about a subgraph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubgraphStatistics {
    pub node_count: usize,
    pub edge_count: usize,
    pub internal_edges: usize,
    pub external_edges: usize,
    pub depth: usize,
    pub density: f32,
    pub clustering_coefficient: f32,
    pub average_degree: f32,
}

/// Analysis result for a subgraph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubgraphAnalysis {
    pub statistics: SubgraphStatistics,
    pub cohesion_score: f32,
    pub coupling_score: f32,
    pub complexity_score: f32,
    pub suggested_operations: Vec<SuggestedOperation>,
}

/// Suggested operations based on analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SuggestedOperation {
    Split {
        reason: String,
        criteria: SplitCriteria,
        confidence: f32,
    },
    Merge {
        reason: String,
        target_subgraphs: Vec<SubgraphId>,
        confidence: f32,
    },
    Refactor {
        reason: String,
        suggested_type: SubgraphType,
        confidence: f32,
    },
    Optimize {
        reason: String,
        optimization_type: OptimizationType,
        confidence: f32,
    },
}

/// Types of optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationType {
    ReduceConnections,
    ImproveLayout,
    SimplifyStructure,
    ExtractInterface,
}

// Implement Display traits for better debugging
impl fmt::Display for SubgraphState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubgraphState::Expanded => write!(f, "Expanded"),
            SubgraphState::Collapsed => write!(f, "Collapsed"),
            SubgraphState::Transitioning { progress, from, to } => {
                write!(f, "Transitioning ({:.1}%): {:?} -> {:?}", progress * 100.0, from, to)
            }
        }
    }
}

impl fmt::Display for SubgraphType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubgraphType::Module => write!(f, "Module"),
            SubgraphType::Cluster => write!(f, "Cluster"),
            SubgraphType::Namespace => write!(f, "Namespace"),
            SubgraphType::Workflow => write!(f, "Workflow"),
            SubgraphType::ConceptualRegion => write!(f, "Conceptual Region"),
            SubgraphType::Custom(name) => write!(f, "Custom: {}", name),
        }
    }
}

// Validation methods
impl SubgraphMetadata {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.name.is_empty() {
            return Err(ValidationError::EmptyName);
        }
        if self.name.len() > 255 {
            return Err(ValidationError::NameTooLong);
        }
        if self.version == 0 {
            return Err(ValidationError::InvalidVersion);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    EmptyName,
    NameTooLong,
    InvalidVersion,
    InvalidColor,
    InvalidProgress,
}

// Builder patterns for complex types
pub struct SubgraphMetadataBuilder {
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    icon: Option<IconType>,
    author: Option<String>,
    custom_properties: std::collections::HashMap<String, serde_json::Value>,
}

impl SubgraphMetadataBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            tags: Vec::new(),
            icon: None,
            author: None,
            custom_properties: std::collections::HashMap::new(),
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn icon(mut self, icon: IconType) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn property(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom_properties.insert(key.into(), value);
        self
    }

    pub fn build(self) -> SubgraphMetadata {
        let now = chrono::Utc::now();
        SubgraphMetadata {
            name: self.name,
            description: self.description,
            tags: self.tags,
            icon: self.icon,
            created_at: now,
            modified_at: now,
            author: self.author,
            version: 1,
            custom_properties: self.custom_properties,
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subgraph_state_transitions() {
        let state = SubgraphState::Expanded;
        assert_eq!(format!("{}", state), "Expanded");

        let transitioning = SubgraphState::Transitioning {
            progress: 0.5,
            from: Box::new(SubgraphState::Expanded),
            to: Box::new(SubgraphState::Collapsed),
        };
        assert!(format!("{}", transitioning).contains("50.0%"));
    }

    #[test]
    fn test_metadata_builder() {
        let metadata = SubgraphMetadataBuilder::new("Test Subgraph")
            .description("A test subgraph")
            .tag("test")
            .tag("example")
            .icon(IconType::Module)
            .author("Test Author")
            .build();

        assert_eq!(metadata.name, "Test Subgraph");
        assert_eq!(metadata.tags.len(), 2);
        assert_eq!(metadata.version, 1);
        assert!(metadata.validate().is_ok());
    }

    #[test]
    fn test_metadata_validation() {
        let mut metadata = SubgraphMetadataBuilder::new("Valid").build();
        assert!(metadata.validate().is_ok());

        metadata.name = String::new();
        assert_eq!(metadata.validate(), Err(ValidationError::EmptyName));

        metadata.name = "a".repeat(256);
        assert_eq!(metadata.validate(), Err(ValidationError::NameTooLong));

        metadata.name = "Valid".to_string();
        metadata.version = 0;
        assert_eq!(metadata.validate(), Err(ValidationError::InvalidVersion));
    }
}
