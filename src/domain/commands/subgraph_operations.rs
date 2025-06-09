use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{
    GraphId, SubgraphId, NodeId, Position3D,
    CollapseStrategy, LayoutStrategy, MergeStrategy, SplitCriteria,
    SubgraphType, SubgraphMetadata, SubgraphStyle,
};

/// Commands for advanced subgraph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubgraphOperationCommand {
    /// Collapse a subgraph to a single node
    CollapseSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        strategy: CollapseStrategy,
    },

    /// Expand a collapsed subgraph
    ExpandSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        layout: LayoutStrategy,
    },

    /// Merge multiple subgraphs into one
    MergeSubgraphs {
        graph_id: GraphId,
        source_subgraphs: Vec<SubgraphId>,
        target_subgraph_id: SubgraphId,
        strategy: MergeStrategy,
    },

    /// Split a subgraph into multiple subgraphs
    SplitSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        criteria: SplitCriteria,
    },

    /// Update subgraph metadata
    UpdateSubgraphMetadata {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        metadata: SubgraphMetadata,
    },

    /// Change subgraph visual style
    ChangeSubgraphStyle {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        style: SubgraphStyle,
    },

    /// Change subgraph type
    ChangeSubgraphType {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        new_type: SubgraphType,
    },

    /// Analyze subgraph structure
    AnalyzeSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
    },

    /// Group selected nodes into a new subgraph
    GroupNodesIntoSubgraph {
        graph_id: GraphId,
        node_ids: Vec<NodeId>,
        subgraph_id: SubgraphId,
        subgraph_type: SubgraphType,
        initial_layout: LayoutStrategy,
    },

    /// Ungroup a subgraph (dissolve it)
    UngroupSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
    },

    /// Establish hierarchy between subgraphs
    EstablishSubgraphHierarchy {
        graph_id: GraphId,
        parent_subgraph: SubgraphId,
        child_subgraph: SubgraphId,
    },

    /// Remove hierarchy between subgraphs
    RemoveSubgraphHierarchy {
        graph_id: GraphId,
        parent_subgraph: SubgraphId,
        child_subgraph: SubgraphId,
    },

    /// Recalculate subgraph layout
    RecalculateSubgraphLayout {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        layout_strategy: LayoutStrategy,
    },

    /// Update subgraph boundary visualization
    UpdateSubgraphBoundary {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        boundary_type: crate::domain::events::BoundaryType,
        boundary_data: serde_json::Value,
    },

    /// Optimize subgraph structure
    OptimizeSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        optimization_type: OptimizationType,
    },

    /// Extract subgraph as independent graph
    ExtractSubgraphAsGraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        new_graph_id: GraphId,
        new_graph_name: String,
    },

    /// Import graph as subgraph
    ImportGraphAsSubgraph {
        target_graph_id: GraphId,
        source_graph_id: GraphId,
        subgraph_id: SubgraphId,
        position: Position3D,
        merge_strategy: MergeStrategy,
    },

    /// Clone subgraph
    CloneSubgraph {
        graph_id: GraphId,
        source_subgraph_id: SubgraphId,
        target_subgraph_id: SubgraphId,
        position_offset: Position3D,
    },

    /// Lock/unlock subgraph for editing
    SetSubgraphLocked {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        locked: bool,
    },

    /// Set subgraph visibility
    SetSubgraphVisibility {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        visible: bool,
    },
}

/// Types of optimization for subgraphs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationType {
    /// Reduce the number of external connections
    ReduceConnections,
    /// Improve the internal layout
    ImproveLayout,
    /// Simplify the structure by removing redundant nodes/edges
    SimplifyStructure,
    /// Extract common interface nodes
    ExtractInterface,
}

impl SubgraphOperationCommand {
    /// Get the command type as a string
    pub fn command_type(&self) -> &'static str {
        match self {
            Self::CollapseSubgraph { .. } => "collapse_subgraph",
            Self::ExpandSubgraph { .. } => "expand_subgraph",
            Self::MergeSubgraphs { .. } => "merge_subgraphs",
            Self::SplitSubgraph { .. } => "split_subgraph",
            Self::UpdateSubgraphMetadata { .. } => "update_subgraph_metadata",
            Self::ChangeSubgraphStyle { .. } => "change_subgraph_style",
            Self::ChangeSubgraphType { .. } => "change_subgraph_type",
            Self::AnalyzeSubgraph { .. } => "analyze_subgraph",
            Self::GroupNodesIntoSubgraph { .. } => "group_nodes_into_subgraph",
            Self::UngroupSubgraph { .. } => "ungroup_subgraph",
            Self::EstablishSubgraphHierarchy { .. } => "establish_subgraph_hierarchy",
            Self::RemoveSubgraphHierarchy { .. } => "remove_subgraph_hierarchy",
            Self::RecalculateSubgraphLayout { .. } => "recalculate_subgraph_layout",
            Self::UpdateSubgraphBoundary { .. } => "update_subgraph_boundary",
            Self::OptimizeSubgraph { .. } => "optimize_subgraph",
            Self::ExtractSubgraphAsGraph { .. } => "extract_subgraph_as_graph",
            Self::ImportGraphAsSubgraph { .. } => "import_graph_as_subgraph",
            Self::CloneSubgraph { .. } => "clone_subgraph",
            Self::SetSubgraphLocked { .. } => "set_subgraph_locked",
            Self::SetSubgraphVisibility { .. } => "set_subgraph_visibility",
        }
    }

    /// Get the graph ID associated with this command
    pub fn graph_id(&self) -> GraphId {
        match self {
            Self::CollapseSubgraph { graph_id, .. } => *graph_id,
            Self::ExpandSubgraph { graph_id, .. } => *graph_id,
            Self::MergeSubgraphs { graph_id, .. } => *graph_id,
            Self::SplitSubgraph { graph_id, .. } => *graph_id,
            Self::UpdateSubgraphMetadata { graph_id, .. } => *graph_id,
            Self::ChangeSubgraphStyle { graph_id, .. } => *graph_id,
            Self::ChangeSubgraphType { graph_id, .. } => *graph_id,
            Self::AnalyzeSubgraph { graph_id, .. } => *graph_id,
            Self::GroupNodesIntoSubgraph { graph_id, .. } => *graph_id,
            Self::UngroupSubgraph { graph_id, .. } => *graph_id,
            Self::EstablishSubgraphHierarchy { graph_id, .. } => *graph_id,
            Self::RemoveSubgraphHierarchy { graph_id, .. } => *graph_id,
            Self::RecalculateSubgraphLayout { graph_id, .. } => *graph_id,
            Self::UpdateSubgraphBoundary { graph_id, .. } => *graph_id,
            Self::OptimizeSubgraph { graph_id, .. } => *graph_id,
            Self::ExtractSubgraphAsGraph { graph_id, .. } => *graph_id,
            Self::ImportGraphAsSubgraph { target_graph_id, .. } => *target_graph_id,
            Self::CloneSubgraph { graph_id, .. } => *graph_id,
            Self::SetSubgraphLocked { graph_id, .. } => *graph_id,
            Self::SetSubgraphVisibility { graph_id, .. } => *graph_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_type() {
        let cmd = SubgraphOperationCommand::CollapseSubgraph {
            graph_id: GraphId::new(),
            subgraph_id: SubgraphId::new(),
            strategy: CollapseStrategy::Centroid,
        };
        assert_eq!(cmd.command_type(), "collapse_subgraph");
    }

    #[test]
    fn test_graph_id() {
        let graph_id = GraphId::new();
        let cmd = SubgraphOperationCommand::AnalyzeSubgraph {
            graph_id,
            subgraph_id: SubgraphId::new(),
        };
        assert_eq!(cmd.graph_id(), graph_id);
    }

    #[test]
    fn test_optimization_type() {
        let opt_types = vec![
            OptimizationType::ReduceConnections,
            OptimizationType::ImproveLayout,
            OptimizationType::SimplifyStructure,
            OptimizationType::ExtractInterface,
        ];

        for opt_type in opt_types {
            let cmd = SubgraphOperationCommand::OptimizeSubgraph {
                graph_id: GraphId::new(),
                subgraph_id: SubgraphId::new(),
                optimization_type: opt_type,
            };
            match cmd {
                SubgraphOperationCommand::OptimizeSubgraph { optimization_type, .. } => {
                    assert_eq!(optimization_type, opt_type);
                }
                _ => panic!("Wrong command type"),
            }
        }
    }
}
