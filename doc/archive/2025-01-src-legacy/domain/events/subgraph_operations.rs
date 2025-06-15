use crate::domain::value_objects::{
    CollapseStrategy, GraphId, LayoutStrategy, MergeStrategy, NodeId, Position3D, SplitCriteria,
    SubgraphAnalysis, SubgraphId, SubgraphMetadata, SubgraphState, SubgraphStyle, SubgraphType,
};
use serde::{Deserialize, Serialize};

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
        resulting_subgraphs: Vec<SubgraphId>,
        split_criteria: SplitCriteria,
        node_distribution: Vec<(SubgraphId, Vec<NodeId>)>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subgraph state transition started
    SubgraphTransitionStarted {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        from_state: SubgraphState,
        to_state: SubgraphState,
        duration_ms: u64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subgraph state transition completed
    SubgraphTransitionCompleted {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        final_state: SubgraphState,
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

    /// Subgraph style was changed
    SubgraphStyleChanged {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        old_style: SubgraphStyle,
        new_style: SubgraphStyle,
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

    /// Subgraph analysis was performed
    SubgraphAnalyzed {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        analysis: SubgraphAnalysis,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Nodes were grouped into a new subgraph
    NodesGroupedIntoSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        grouped_nodes: Vec<NodeId>,
        subgraph_type: SubgraphType,
        initial_layout: LayoutStrategy,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A subgraph was ungrouped (dissolved)
    SubgraphUngrouped {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        released_nodes: Vec<NodeId>,
        final_positions: Vec<(NodeId, Position3D)>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subgraph hierarchy was established
    SubgraphHierarchyEstablished {
        graph_id: GraphId,
        parent_subgraph: SubgraphId,
        child_subgraph: SubgraphId,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subgraph hierarchy was removed
    SubgraphHierarchyRemoved {
        graph_id: GraphId,
        parent_subgraph: SubgraphId,
        child_subgraph: SubgraphId,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subgraph layout was recalculated
    SubgraphLayoutRecalculated {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        layout_strategy: LayoutStrategy,
        old_positions: Vec<(NodeId, Position3D)>,
        new_positions: Vec<(NodeId, Position3D)>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subgraph boundary was updated
    SubgraphBoundaryUpdated {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        boundary_type: BoundaryType,
        boundary_data: serde_json::Value,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Type of boundary for subgraph visualization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BoundaryType {
    ConvexHull,
    BoundingBox,
    Circle,
    Custom(String),
}

// Helper methods for events
impl SubgraphOperationEvent {
    /// Get the graph ID associated with this event
    pub fn graph_id(&self) -> GraphId {
        match self {
            Self::SubgraphCollapsed { graph_id, .. } => *graph_id,
            Self::SubgraphExpanded { graph_id, .. } => *graph_id,
            Self::SubgraphsMerged { graph_id, .. } => *graph_id,
            Self::SubgraphSplit { graph_id, .. } => *graph_id,
            Self::SubgraphTransitionStarted { graph_id, .. } => *graph_id,
            Self::SubgraphTransitionCompleted { graph_id, .. } => *graph_id,
            Self::SubgraphMetadataUpdated { graph_id, .. } => *graph_id,
            Self::SubgraphStyleChanged { graph_id, .. } => *graph_id,
            Self::SubgraphTypeChanged { graph_id, .. } => *graph_id,
            Self::SubgraphAnalyzed { graph_id, .. } => *graph_id,
            Self::NodesGroupedIntoSubgraph { graph_id, .. } => *graph_id,
            Self::SubgraphUngrouped { graph_id, .. } => *graph_id,
            Self::SubgraphHierarchyEstablished { graph_id, .. } => *graph_id,
            Self::SubgraphHierarchyRemoved { graph_id, .. } => *graph_id,
            Self::SubgraphLayoutRecalculated { graph_id, .. } => *graph_id,
            Self::SubgraphBoundaryUpdated { graph_id, .. } => *graph_id,
        }
    }

    /// Get the timestamp of this event
    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            Self::SubgraphCollapsed { timestamp, .. } => *timestamp,
            Self::SubgraphExpanded { timestamp, .. } => *timestamp,
            Self::SubgraphsMerged { timestamp, .. } => *timestamp,
            Self::SubgraphSplit { timestamp, .. } => *timestamp,
            Self::SubgraphTransitionStarted { timestamp, .. } => *timestamp,
            Self::SubgraphTransitionCompleted { timestamp, .. } => *timestamp,
            Self::SubgraphMetadataUpdated { timestamp, .. } => *timestamp,
            Self::SubgraphStyleChanged { timestamp, .. } => *timestamp,
            Self::SubgraphTypeChanged { timestamp, .. } => *timestamp,
            Self::SubgraphAnalyzed { timestamp, .. } => *timestamp,
            Self::NodesGroupedIntoSubgraph { timestamp, .. } => *timestamp,
            Self::SubgraphUngrouped { timestamp, .. } => *timestamp,
            Self::SubgraphHierarchyEstablished { timestamp, .. } => *timestamp,
            Self::SubgraphHierarchyRemoved { timestamp, .. } => *timestamp,
            Self::SubgraphLayoutRecalculated { timestamp, .. } => *timestamp,
            Self::SubgraphBoundaryUpdated { timestamp, .. } => *timestamp,
        }
    }

    /// Get the primary subgraph ID affected by this event
    pub fn primary_subgraph_id(&self) -> Option<SubgraphId> {
        match self {
            Self::SubgraphCollapsed { subgraph_id, .. } => Some(*subgraph_id),
            Self::SubgraphExpanded { subgraph_id, .. } => Some(*subgraph_id),
            Self::SubgraphsMerged {
                target_subgraph, ..
            } => Some(*target_subgraph),
            Self::SubgraphSplit {
                source_subgraph, ..
            } => Some(*source_subgraph),
            Self::SubgraphTransitionStarted { subgraph_id, .. } => Some(*subgraph_id),
            Self::SubgraphTransitionCompleted { subgraph_id, .. } => Some(*subgraph_id),
            Self::SubgraphMetadataUpdated { subgraph_id, .. } => Some(*subgraph_id),
            Self::SubgraphStyleChanged { subgraph_id, .. } => Some(*subgraph_id),
            Self::SubgraphTypeChanged { subgraph_id, .. } => Some(*subgraph_id),
            Self::SubgraphAnalyzed { subgraph_id, .. } => Some(*subgraph_id),
            Self::NodesGroupedIntoSubgraph { subgraph_id, .. } => Some(*subgraph_id),
            Self::SubgraphUngrouped { subgraph_id, .. } => Some(*subgraph_id),
            Self::SubgraphHierarchyEstablished {
                parent_subgraph, ..
            } => Some(*parent_subgraph),
            Self::SubgraphHierarchyRemoved {
                parent_subgraph, ..
            } => Some(*parent_subgraph),
            Self::SubgraphLayoutRecalculated { subgraph_id, .. } => Some(*subgraph_id),
            Self::SubgraphBoundaryUpdated { subgraph_id, .. } => Some(*subgraph_id),
        }
    }

    /// Get the event type as a string
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::SubgraphCollapsed { .. } => "SubgraphCollapsed",
            Self::SubgraphExpanded { .. } => "SubgraphExpanded",
            Self::SubgraphsMerged { .. } => "SubgraphsMerged",
            Self::SubgraphSplit { .. } => "SubgraphSplit",
            Self::SubgraphTransitionStarted { .. } => "SubgraphTransitionStarted",
            Self::SubgraphTransitionCompleted { .. } => "SubgraphTransitionCompleted",
            Self::SubgraphMetadataUpdated { .. } => "SubgraphMetadataUpdated",
            Self::SubgraphStyleChanged { .. } => "SubgraphStyleChanged",
            Self::SubgraphTypeChanged { .. } => "SubgraphTypeChanged",
            Self::SubgraphAnalyzed { .. } => "SubgraphAnalyzed",
            Self::NodesGroupedIntoSubgraph { .. } => "NodesGroupedIntoSubgraph",
            Self::SubgraphUngrouped { .. } => "SubgraphUngrouped",
            Self::SubgraphHierarchyEstablished { .. } => "SubgraphHierarchyEstablished",
            Self::SubgraphHierarchyRemoved { .. } => "SubgraphHierarchyRemoved",
            Self::SubgraphLayoutRecalculated { .. } => "SubgraphLayoutRecalculated",
            Self::SubgraphBoundaryUpdated { .. } => "SubgraphBoundaryUpdated",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_graph_id() {
        let graph_id = GraphId::new();
        let subgraph_id = SubgraphId::new();
        let event = SubgraphOperationEvent::SubgraphCollapsed {
            graph_id,
            subgraph_id,
            collapsed_at: Position3D::default(),
            contained_nodes: vec![],
            collapse_strategy: CollapseStrategy::Centroid,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(event.graph_id(), graph_id);
    }

    #[test]
    fn test_event_timestamp() {
        let now = chrono::Utc::now();
        let event = SubgraphOperationEvent::SubgraphExpanded {
            graph_id: GraphId::new(),
            subgraph_id: SubgraphId::new(),
            expansion_layout: LayoutStrategy::ForceDirected {
                iterations: 100,
                spring_strength: 0.1,
                repulsion_strength: 0.2,
            },
            node_positions: vec![],
            timestamp: now,
        };

        assert_eq!(event.timestamp(), now);
    }

    #[test]
    fn test_primary_subgraph_id() {
        let subgraph_id = SubgraphId::new();
        let event = SubgraphOperationEvent::SubgraphAnalyzed {
            graph_id: GraphId::new(),
            subgraph_id,
            analysis: SubgraphAnalysis {
                statistics: SubgraphStatistics {
                    node_count: 5,
                    edge_count: 4,
                    internal_edges: 3,
                    external_edges: 1,
                    depth: 2,
                    density: 0.4,
                    clustering_coefficient: 0.6,
                    average_degree: 1.6,
                },
                cohesion_score: 0.8,
                coupling_score: 0.2,
                complexity_score: 0.5,
                suggested_operations: vec![],
            },
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(event.primary_subgraph_id(), Some(subgraph_id));
    }
}
