# Module 1: Domain Foundation - Technical Specification

## Overview

This module provides the foundational domain types for all subgraph operations. It includes value objects, events, and commands that form the basis of the subgraph advanced operations system.

## 1.1 Value Objects Implementation

### File: `src/domain/value_objects/subgraph_operations.rs`

```rust
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::domain::value_objects::{Position3D, NodeId, SubgraphId};

/// Represents the visual state of a subgraph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
```

## 1.2 Domain Events Implementation

### File: `src/domain/events/subgraph_operations.rs`

```rust
use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{
    GraphId, SubgraphId, NodeId, Position3D,
    SubgraphState, LayoutStrategy, MergeStrategy, SplitCriteria,
    SubgraphType, SubgraphMetadata, SubgraphStyle,
};

/// Events related to subgraph operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubgraphOperationEvent {
    /// A subgraph was collapsed
    SubgraphCollapsed {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        collapsed_at: Position3D,
        contained_nodes: Vec<NodeId>,
        collapse_strategy: CollapseStrategy,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A subgraph was expanded
    SubgraphExpanded {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        expansion_layout: LayoutStrategy,
        node_positions: Vec<(NodeId, Position3D)>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Multiple subgraphs were merged
    SubgraphsMerged {
        graph_id: GraphId,
        source_subgraphs: Vec<SubgraphId>,
        target_subgraph: SubgraphId,
        merge_strategy: MergeStrategy,
        merged_nodes: Vec<NodeId>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A subgraph was split into multiple subgraphs
    SubgraphSplit {
        graph_id: GraphId,
        source_subgraph: SubgraphId,
        resulting_subgraphs: Vec<(SubgraphId, Vec<NodeId>)>,
        split_criteria: SplitCriteria,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A node was dragged between subgraphs
    NodeDraggedBetweenSubgraphs {
        graph_id: GraphId,
        node_id: NodeId,
        from_subgraph: SubgraphId,
        to_subgraph: SubgraphId,
        new_position: Position3D,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Multiple nodes were moved between subgraphs
    NodesBulkMoved {
        graph_id: GraphId,
        moves: Vec<NodeMove>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subgraph type was changed
    SubgraphTypeChanged {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        old_type: SubgraphType,
        new_type: SubgraphType,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subgraph metadata was updated
    SubgraphMetadataUpdated {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        old_metadata: SubgraphMetadata,
        new_metadata: SubgraphMetadata,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subgraph visual style was changed
    SubgraphStyleChanged {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        old_style: SubgraphStyle,
        new_style: SubgraphStyle,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subgraph state transition started
    SubgraphTransitionStarted {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        from_state: SubgraphState,
        to_state: SubgraphState,
        duration_ms: u32,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subgraph state transition completed
    SubgraphTransitionCompleted {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        final_state: SubgraphState,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subgraph analysis completed
    SubgraphAnalyzed {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        analysis_result: SubgraphAnalysis,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Represents a node move operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeMove {
    pub node_id: NodeId,
    pub from_subgraph: SubgraphId,
    pub to_subgraph: SubgraphId,
    pub new_position: Position3D,
}

// Event creation helpers
impl SubgraphOperationEvent {
    /// Create a collapse event
    pub fn collapse(
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        collapsed_at: Position3D,
        contained_nodes: Vec<NodeId>,
        strategy: CollapseStrategy,
    ) -> Self {
        Self::SubgraphCollapsed {
            graph_id,
            subgraph_id,
            collapsed_at,
            contained_nodes,
            collapse_strategy: strategy,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create an expand event
    pub fn expand(
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        layout: LayoutStrategy,
        positions: Vec<(NodeId, Position3D)>,
    ) -> Self {
        Self::SubgraphExpanded {
            graph_id,
            subgraph_id,
            expansion_layout: layout,
            node_positions: positions,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a merge event
    pub fn merge(
        graph_id: GraphId,
        sources: Vec<SubgraphId>,
        target: SubgraphId,
        strategy: MergeStrategy,
        nodes: Vec<NodeId>,
    ) -> Self {
        Self::SubgraphsMerged {
            graph_id,
            source_subgraphs: sources,
            target_subgraph: target,
            merge_strategy: strategy,
            merged_nodes: nodes,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get the graph ID for any event
    pub fn graph_id(&self) -> GraphId {
        match self {
            Self::SubgraphCollapsed { graph_id, .. } |
            Self::SubgraphExpanded { graph_id, .. } |
            Self::SubgraphsMerged { graph_id, .. } |
            Self::SubgraphSplit { graph_id, .. } |
            Self::NodeDraggedBetweenSubgraphs { graph_id, .. } |
            Self::NodesBulkMoved { graph_id, .. } |
            Self::SubgraphTypeChanged { graph_id, .. } |
            Self::SubgraphMetadataUpdated { graph_id, .. } |
            Self::SubgraphStyleChanged { graph_id, .. } |
            Self::SubgraphTransitionStarted { graph_id, .. } |
            Self::SubgraphTransitionCompleted { graph_id, .. } |
            Self::SubgraphAnalyzed { graph_id, .. } => *graph_id,
        }
    }

    /// Get the timestamp for any event
    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            Self::SubgraphCollapsed { timestamp, .. } |
            Self::SubgraphExpanded { timestamp, .. } |
            Self::SubgraphsMerged { timestamp, .. } |
            Self::SubgraphSplit { timestamp, .. } |
            Self::NodeDraggedBetweenSubgraphs { timestamp, .. } |
            Self::NodesBulkMoved { timestamp, .. } |
            Self::SubgraphTypeChanged { timestamp, .. } |
            Self::SubgraphMetadataUpdated { timestamp, .. } |
            Self::SubgraphStyleChanged { timestamp, .. } |
            Self::SubgraphTransitionStarted { timestamp, .. } |
            Self::SubgraphTransitionCompleted { timestamp, .. } |
            Self::SubgraphAnalyzed { timestamp, .. } => *timestamp,
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let graph_id = GraphId::new();
        let subgraph_id = SubgraphId::new();
        let node_ids = vec![NodeId::new(), NodeId::new()];
        let position = Position3D::new(0.0, 0.0, 0.0);

        let event = SubgraphOperationEvent::collapse(
            graph_id,
            subgraph_id,
            position,
            node_ids.clone(),
            CollapseStrategy::Centroid,
        );

        assert_eq!(event.graph_id(), graph_id);
        assert!(matches!(event, SubgraphOperationEvent::SubgraphCollapsed { .. }));
    }

    #[test]
    fn test_event_timestamp() {
        let before = chrono::Utc::now();
        let event = SubgraphOperationEvent::expand(
            GraphId::new(),
            SubgraphId::new(),
            LayoutStrategy::RestorePrevious,
            vec![],
        );
        let after = chrono::Utc::now();

        let timestamp = event.timestamp();
        assert!(timestamp >= before);
        assert!(timestamp <= after);
    }
}
```

## 1.3 Domain Commands Implementation

### File: `src/domain/commands/subgraph_operations.rs`

```rust
use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{
    GraphId, SubgraphId, NodeId, Position3D,
    CollapseStrategy, LayoutStrategy, MergeStrategy, SplitCriteria,
    SubgraphType, SubgraphMetadata, SubgraphStyle,
};

/// Commands for subgraph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubgraphOperationCommand {
    /// Collapse a subgraph to a single node
    CollapseSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        collapse_strategy: CollapseStrategy,
        animate: bool,
        duration_ms: Option<u32>,
    },

    /// Expand a collapsed subgraph
    ExpandSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        layout_strategy: LayoutStrategy,
        animate: bool,
        duration_ms: Option<u32>,
    },

    /// Merge multiple subgraphs into one
    MergeSubgraphs {
        graph_id: GraphId,
        source_subgraphs: Vec<SubgraphId>,
        target_name: String,
        merge_strategy: MergeStrategy,
        preserve_metadata: bool,
    },

    /// Split a subgraph into multiple subgraphs
    SplitSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        split_criteria: SplitCriteria,
        name_pattern: String, // e.g., "{original}_part_{index}"
    },

    /// Drag a node to a different subgraph
    DragNodeToSubgraph {
        graph_id: GraphId,
        node_id: NodeId,
        target_subgraph: SubgraphId,
        position: Position3D,
        update_connections: bool,
    },

    /// Move multiple nodes between subgraphs
    BulkMoveNodes {
        graph_id: GraphId,
        moves: Vec<NodeMoveCommand>,
        update_layout: bool,
    },

    /// Update subgraph metadata
    UpdateSubgraphMetadata {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        metadata: SubgraphMetadata,
        merge_with_existing: bool,
    },

    /// Change subgraph type
    ChangeSubgraphType {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        new_type: SubgraphType,
        update_style: bool,
    },

    /// Update subgraph visual style
    UpdateSubgraphStyle {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        style: SubgraphStyle,
        apply_to_children: bool,
    },

    /// Analyze a subgraph
    AnalyzeSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        analysis_depth: AnalysisDepth,
    },

    /// Apply suggested operation
    ApplySuggestedOperation {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        operation: SuggestedOperation,
        preview: bool,
    },

    /// Create subgraph from selected nodes
    CreateSubgraphFromSelection {
        graph_id: GraphId,
        selected_nodes: Vec<NodeId>,
        name: String,
        subgraph_type: SubgraphType,
        auto_layout: bool,
    },

    /// Dissolve a subgraph (remove grouping but keep nodes)
    DissolveSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        preserve_positions: bool,
    },
}

/// Command for moving a single node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMoveCommand {
    pub node_id: NodeId,
    pub target_subgraph: SubgraphId,
    pub position: Option<Position3D>,
}

/// Depth of analysis to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisDepth {
    /// Quick analysis - basic metrics only
    Quick,
    /// Standard analysis - metrics and patterns
    Standard,
    /// Deep analysis - full analysis with suggestions
    Deep,
}

// Command validation
impl SubgraphOperationCommand {
    /// Validate the command
    pub fn validate(&self) -> Result<(), CommandValidationError> {
        match self {
            Self::CollapseSubgraph { duration_ms, .. } => {
                if let Some(duration) = duration_ms {
                    if *duration == 0 {
                        return Err(CommandValidationError::InvalidDuration);
                    }
                }
            }
            Self::MergeSubgraphs { source_subgraphs, target_name, .. } => {
                if source_subgraphs.is_empty() {
                    return Err(CommandValidationError::NoSourceSubgraphs);
                }
                if source_subgraphs.len() == 1 {
                    return Err(CommandValidationError::InsufficientSubgraphsForMerge);
                }
                if target_name.is_empty() {
                    return Err(CommandValidationError::EmptyName);
                }
            }
            Self::SplitSubgraph { name_pattern, .. } => {
                if name_pattern.is_empty() {
                    return Err(CommandValidationError::EmptyNamePattern);
                }
                if !name_pattern.contains("{original}") && !name_pattern.contains("{index}") {
                    return Err(CommandValidationError::InvalidNamePattern);
                }
            }
            Self::BulkMoveNodes { moves, .. } => {
                if moves.is_empty() {
                    return Err(CommandValidationError::NoMoves);
                }
            }
            Self::CreateSubgraphFromSelection { selected_nodes, name, .. } => {
                if selected_nodes.is_empty() {
                    return Err(CommandValidationError::NoNodesSelected);
                }
                if name.is_empty() {
                    return Err(CommandValidationError::EmptyName);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Get the graph ID for the command
    pub fn graph_id(&self) -> GraphId {
        match self {
            Self::CollapseSubgraph { graph_id, .. } |
            Self::ExpandSubgraph { graph_id, .. } |
            Self::MergeSubgraphs { graph_id, .. } |
            Self::SplitSubgraph { graph_id, .. } |
            Self::DragNodeToSubgraph { graph_id, .. } |
            Self::BulkMoveNodes { graph_id, .. } |
            Self::UpdateSubgraphMetadata { graph_id, .. } |
            Self::ChangeSubgraphType { graph_id, .. } |
            Self::UpdateSubgraphStyle { graph_id, .. } |
            Self::AnalyzeSubgraph { graph_id, .. } |
            Self::ApplySuggestedOperation { graph_id, .. } |
            Self::CreateSubgraphFromSelection { graph_id, .. } |
            Self::DissolveSubgraph { graph_id, .. } => *graph_id,
        }
    }
}

/// Command validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum CommandValidationError {
    InvalidDuration,
    NoSourceSubgraphs,
    InsufficientSubgraphsForMerge,
    EmptyName,
    EmptyNamePattern,
    InvalidNamePattern,
    NoMoves,
    NoNodesSelected,
}

// Command builders for complex commands
pub struct MergeSubgraphsBuilder {
    graph_id: GraphId,
    source_subgraphs: Vec<SubgraphId>,
    target_name: String,
    merge_strategy: MergeStrategy,
    preserve_metadata: bool,
}

impl MergeSubgraphsBuilder {
    pub fn new(graph_id: GraphId, target_name: impl Into<String>) -> Self {
        Self {
            graph_id,
            source_subgraphs: Vec::new(),
            target_name: target_name.into(),
            merge_strategy: MergeStrategy::Union,
            preserve_metadata: true,
        }
    }

    pub fn add_source(mut self, subgraph_id: SubgraphId) -> Self {
        self.source_subgraphs.push(subgraph_id);
        self
    }

    pub fn add_sources(mut self, subgraph_ids: impl IntoIterator<Item = SubgraphId>) -> Self {
        self.source_subgraphs.extend(subgraph_ids);
        self
    }

    pub fn strategy(mut self, strategy: MergeStrategy) -> Self {
        self.merge_strategy = strategy;
        self
    }

    pub fn preserve_metadata(mut self, preserve: bool) -> Self {
        self.preserve_metadata = preserve;
        self
    }

    pub fn build(self) -> Result<SubgraphOperationCommand, CommandValidationError> {
        let command = SubgraphOperationCommand::MergeSubgraphs {
            graph_id: self.graph_id,
            source_subgraphs: self.source_subgraphs,
            target_name: self.target_name,
            merge_strategy: self.merge_strategy,
            preserve_metadata: self.preserve_metadata,
        };
        command.validate()?;
        Ok(command)
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_validation() {
        // Valid collapse command
        let cmd = SubgraphOperationCommand::CollapseSubgraph {
            graph_id: GraphId::new(),
            subgraph_id: SubgraphId::new(),
            collapse_strategy: CollapseStrategy::Centroid,
            animate: true,
            duration_ms: Some(300),
        };
        assert!(cmd.validate().is_ok());

        // Invalid duration
        let cmd = SubgraphOperationCommand::CollapseSubgraph {
            graph_id: GraphId::new(),
            subgraph_id: SubgraphId::new(),
            collapse_strategy: CollapseStrategy::Centroid,
            animate: true,
            duration_ms: Some(0),
        };
        assert_eq!(cmd.validate(), Err(CommandValidationError::InvalidDuration));

        // Invalid merge - no sources
        let cmd = SubgraphOperationCommand::MergeSubgraphs {
            graph_id: GraphId::new(),
            source_subgraphs: vec![],
            target_name: "Merged".to_string(),
            merge_strategy: MergeStrategy::Union,
            preserve_metadata: true,
        };
        assert_eq!(cmd.validate(), Err(CommandValidationError::NoSourceSubgraphs));
    }

    #[test]
    fn test_merge_builder() {
        let graph_id = GraphId::new();
        let subgraph1 = SubgraphId::new();
        let subgraph2 = SubgraphId::new();

        let result = MergeSubgraphsBuilder::new(graph_id, "Merged")
            .add_source(subgraph1)
            .add_source(subgraph2)
            .strategy(MergeStrategy::Reflow)
            .preserve_metadata(false)
            .build();

        assert!(result.is_ok());
        let command = result.unwrap();
        assert_eq!(command.graph_id(), graph_id);
    }

    #[test]
    fn test_split_command_validation() {
        // Valid split command
        let cmd = SubgraphOperationCommand::SplitSubgraph {
            graph_id: GraphId::new(),
            subgraph_id: SubgraphId::new(),
            split_criteria: SplitCriteria::Connectivity {
                min_cut: true,
                max_components: 2,
            },
            name_pattern: "{original}_part_{index}".to_string(),
        };
        assert!(cmd.validate().is_ok());

        // Invalid name pattern
        let cmd = SubgraphOperationCommand::SplitSubgraph {
            graph_id: GraphId::new(),
            subgraph_id: SubgraphId::new(),
            split_criteria: SplitCriteria::Connectivity {
                min_cut: true,
                max_components: 2,
            },
            name_pattern: "invalid_pattern".to_string(),
        };
        assert_eq!(cmd.validate(), Err(CommandValidationError::InvalidNamePattern));
    }
}
```

## Integration with Existing System

### Update `src/domain/value_objects.rs`
Add:
```rust
pub use subgraph_operations::*;

pub mod subgraph_operations;
```

### Update `src/domain/events/mod.rs`
Add:
```rust
pub use subgraph_operations::SubgraphOperationEvent;

pub mod subgraph_operations;
```

### Update `src/domain/commands/mod.rs`
Add:
```rust
pub use subgraph_operations::SubgraphOperationCommand;

pub mod subgraph_operations;
```

### Update `src/domain/events/mod.rs` DomainEvent enum
Add:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Event)]
pub enum DomainEvent {
    // ... existing variants ...
    SubgraphOperation(SubgraphOperationEvent),
}
```

### Update `src/domain/commands/mod.rs` Command enum
Add:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    // ... existing variants ...
    SubgraphOperation(SubgraphOperationCommand),
}
```

## Testing Strategy

1. **Unit Tests**: Test each value object, event, and command in isolation
2. **Property Tests**: Use proptest for validating invariants
3. **Integration Tests**: Test with the Graph aggregate
4. **Serialization Tests**: Ensure all types serialize/deserialize correctly

## Next Steps

After implementing this foundation module:
1. Update the Graph aggregate to handle new commands
2. Implement domain services (SubgraphAnalyzer, LayoutCalculator)
3. Create command handlers for the new operations
4. Build the presentation layer systems
