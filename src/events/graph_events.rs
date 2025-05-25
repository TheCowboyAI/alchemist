//! Graph-related events for node and edge manipulation
//!
//! These events handle all graph structure modifications including:
//! - Node lifecycle (creation, modification, deletion)
//! - Edge lifecycle (creation, modification, deletion)
//! - Selection and interaction
//! - Layout and visualization
//! - Undo/redo operations

use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::components::{DomainEdgeType, DomainNodeType};

/// Event for creating a new graph node
///
/// ## Producers
/// - UI systems (context menu, toolbar)
/// - File loading systems
/// - Pattern generation systems
///
/// ## Consumers
/// - `node_creation_system` - Spawns the entity and components
/// - `undo_system` - Records the operation
/// - `graph_validation_system` - Validates graph constraints
#[derive(Event)]
pub struct CreateNodeEvent {
    pub id: Uuid,
    pub position: Vec3,
    pub domain_type: DomainNodeType,
    pub name: String,
    pub labels: Vec<String>,
    pub properties: HashMap<String, String>,
    pub subgraph_id: Option<Uuid>,
    pub color: Option<String>, // Hex color from JSON style
}

/// Event for updating node properties
///
/// ## Producers
/// - Inspector panel UI
/// - Property editor systems
///
/// ## Consumers
/// - `node_update_system` - Updates node components
/// - `undo_system` - Records the changes
#[derive(Event)]
pub struct UpdateNodeEvent {
    pub entity: Entity,
    pub name: Option<String>,
    pub labels: Option<Vec<String>>,
    pub properties: Option<HashMap<String, String>>,
    pub domain_type: Option<DomainNodeType>,
}

/// Event for moving a node
///
/// ## Producers
/// - Mouse drag interaction system
/// - Layout algorithms
/// - Snap-to-grid system
///
/// ## Consumers
/// - `node_movement_system` - Updates Transform component
/// - `edge_update_system` - Updates connected edge visuals
/// - `undo_system` - Records the movement
#[derive(Event)]
pub struct MoveNodeEvent {
    pub entity: Entity,
    pub from: Vec3,
    pub to: Vec3,
}

/// Event for batch moving multiple nodes
///
/// ## Producers
/// - Multi-selection drag system
/// - Alignment tools
///
/// ## Consumers
/// - `batch_movement_system` - Updates multiple transforms
#[derive(Event)]
pub struct BatchMoveNodesEvent {
    pub moves: Vec<(Entity, Vec3, Vec3)>, // (entity, from, to)
}

/// Event for creating a new edge
#[derive(Event)]
pub struct CreateEdgeEvent {
    pub id: Uuid,
    pub source: Entity,
    pub target: Entity,
    pub edge_type: DomainEdgeType,
    pub labels: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// Event for creating edges after nodes have been created (with UUID references)
#[derive(Event, Debug, Clone)]
pub struct DeferredEdgeEvent {
    pub id: Uuid,
    pub source_uuid: Uuid,
    pub target_uuid: Uuid,
    pub edge_type: DomainEdgeType,
    pub labels: Vec<String>,
    pub properties: HashMap<String, String>,
    pub retry_count: u8,  // Track retry attempts
}

/// Event for deleting nodes
#[derive(Event)]
pub struct DeleteNodeEvent {
    pub entity: Entity,
}

/// Event for deleting edges
#[derive(Event)]
pub struct DeleteEdgeEvent {
    pub source: Entity, // The source node entity
    pub edge_id: Uuid,  // The UUID of the edge to delete
}

/// Event for selecting entities
#[derive(Event)]
pub struct SelectEvent {
    pub entity: Entity,
    pub multi_select: bool,
}

/// Event for deselecting all
#[derive(Event)]
pub struct DeselectAllEvent;

/// Event for hovering over entities
#[derive(Event)]
pub struct HoverEvent {
    pub entity: Option<Entity>,
}

/// Event for graph layout updates
#[derive(Event)]
pub struct LayoutUpdateEvent {
    pub layout_type: LayoutType,
}

#[derive(Debug, Clone)]
pub enum LayoutType {
    ForceDirected,
    Hierarchical,
    Circular,
    Grid,
}

/// Event to request a graph layout update
#[derive(Event)]
pub struct RequestLayoutEvent {
    pub layout_type: LayoutType,
}

/// Event for graph validation
#[derive(Event)]
pub struct ValidateGraphEvent;

/// Event for subgraph creation
#[derive(Event)]
pub struct CreateSubgraphEvent {
    pub id: Uuid,
    pub name: String,
    pub nodes: Vec<Entity>,
}

/// Event for undo operations
#[derive(Event)]
pub struct UndoEvent;

/// Event for redo operations
#[derive(Event)]
pub struct RedoEvent;

/// Event for tracking graph modifications (for undo/redo)
#[derive(Event, Debug, Clone)]
pub enum GraphModificationEvent {
    NodeCreated {
        id: Uuid,
        position: Vec3,
        domain_type: DomainNodeType,
        name: String,
    },
    NodeMoved {
        id: Uuid,
        from: Vec3,
        to: Vec3,
    },
    NodeDeleted {
        id: Uuid,
    },
    EdgeCreated {
        id: Uuid,
        source_id: Uuid,
        target_id: Uuid,
        edge_type: DomainEdgeType,
    },
    EdgeDeleted {
        id: Uuid,
    },
    GraphCleared,
}

/// Event for creating graph patterns
#[derive(Event)]
pub struct CreatePatternEvent {
    pub pattern: GraphPattern,
    pub pattern_name: String,
}

/// Event for updating the 3D graph
#[derive(Event)]
pub struct UpdateGraph3DEvent;

#[derive(Clone)]
pub enum GraphPattern {
    Complete { nodes: usize },
    Star { points: usize },
    Tree { branch_factor: usize, depth: usize },
    // Add more patterns as needed
}

/// Event for updating edge properties
///
/// ## Producers
/// - Edge inspector UI
/// - Edge type conversion tools
///
/// ## Consumers
/// - `edge_update_system` - Updates edge components
#[derive(Event)]
pub struct UpdateEdgeEvent {
    pub edge_id: Uuid,
    pub edge_type: Option<DomainEdgeType>,
    pub labels: Option<Vec<String>>,
    pub properties: Option<HashMap<String, String>>,
}

/// Event for graph structure analysis
///
/// ## Producers
/// - Analysis tool activation
/// - Validation requests
///
/// ## Consumers
/// - `graph_analysis_system` - Performs requested analysis
#[derive(Event)]
pub struct AnalyzeGraphEvent {
    pub analysis_type: GraphAnalysisType,
}

#[derive(Debug, Clone)]
pub enum GraphAnalysisType {
    CycleDetection,
    ConnectedComponents,
    ShortestPath { from: Uuid, to: Uuid },
    Centrality,
    TopologicalSort,
}

/// Event for graph metrics update
///
/// ## Producers
/// - `graph_analysis_system`
/// - Periodic update timers
///
/// ## Consumers
/// - UI status bar
/// - Metrics panel
#[derive(Event)]
pub struct GraphMetricsEvent {
    pub node_count: usize,
    pub edge_count: usize,
    pub connected_components: usize,
    pub has_cycles: bool,
}

/// Event for node property validation
///
/// ## Producers
/// - Property editor on change
/// - Batch validation requests
///
/// ## Consumers
/// - `validation_system` - Checks constraints
/// - UI feedback systems
#[derive(Event)]
pub struct ValidateNodePropertiesEvent {
    pub entity: Entity,
    pub properties: HashMap<String, String>,
}

/// Event for edge connection validation
///
/// ## Producers
/// - Edge creation preview
/// - Edge type change
///
/// ## Consumers
/// - `edge_validation_system` - Checks connection rules
#[derive(Event)]
pub struct ValidateEdgeConnectionEvent {
    pub source: Entity,
    pub target: Entity,
    pub edge_type: DomainEdgeType,
}

/// Event for graph export preparation
///
/// ## Producers
/// - Export menu actions
/// - API requests
///
/// ## Consumers
/// - `graph_serialization_system` - Prepares data for export
#[derive(Event)]
pub struct PrepareGraphExportEvent {
    pub include_visual_data: bool,
    pub include_metadata: bool,
}

/// Event for batch operations
///
/// ## Producers
/// - Multi-selection tools
/// - Script execution
///
/// ## Consumers
/// - `batch_operation_system` - Executes operations in order
#[derive(Event)]
pub struct BatchOperationEvent {
    pub operations: Vec<GraphOperation>,
}

#[derive(Clone)]
pub enum GraphOperation {
    CreateNode(CreateNodeEvent),
    DeleteNode(Entity),
    CreateEdge(CreateEdgeEvent),
    DeleteEdge { source: Entity, edge_id: Uuid },
    MoveNode { entity: Entity, to: Vec3 },
}

/// Event for graph clipboard operations
///
/// ## Producers
/// - Copy/Cut keyboard shortcuts
/// - Context menu actions
///
/// ## Consumers
/// - `clipboard_system` - Manages graph clipboard
#[derive(Event)]
pub struct GraphClipboardEvent {
    pub operation: ClipboardOperation,
}

#[derive(Clone)]
pub enum ClipboardOperation {
    Copy(Vec<Entity>),
    Cut(Vec<Entity>),
    Paste { offset: Vec3 },
}

/// Event for node grouping operations
///
/// ## Producers
/// - Group creation UI
/// - Automatic clustering
///
/// ## Consumers
/// - `grouping_system` - Creates node groups
#[derive(Event)]
pub struct GroupNodesEvent {
    pub nodes: Vec<Entity>,
    pub group_name: String,
    pub collapse: bool,
}
