//! ECS Components for Graph Visualization
//!
//! These components integrate with Bevy's ECS system to create
//! actual functioning graphs that can be queried, modified, and persisted.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Bundle for creating a graph node entity
#[derive(Bundle)]
pub struct GraphNodeBundle {
    pub node: GraphNode,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl GraphNodeBundle {
    pub fn new(id: String, graph_id: String, label: String, position: Vec3) -> Self {
        Self {
            node: GraphNode { id, graph_id, label },
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}

impl Default for GraphNodeBundle {
    fn default() -> Self {
        Self {
            node: GraphNode {
                id: String::new(),
                graph_id: String::new(),
                label: String::new(),
            },
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}

/// Core graph node component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub graph_id: String,
    pub label: String,
}

/// Edge component - attached to edge entities
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub graph_id: String,
    pub source: Entity,
    pub target: Entity,
    pub label: Option<String>,
    pub weight: f32,
}

/// Node metadata component
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub properties: HashMap<String, serde_json::Value>,
    pub tags: HashSet<String>,
    pub node_type: NodeType,
}

/// Node types for different kinds of graph nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Default,
    Package,      // Nix package
    Dependency,   // Dependency relationship
    Document,     // Markdown document
    Heading,      // Document heading
    Link,         // Hyperlink
    Concept,      // Conceptual node
    Workflow,     // Workflow node
    Data,         // Data node
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Default
    }
}

/// Visual style component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct NodeStyle {
    pub color: Color,
    pub size: f32,
    pub shape: NodeShape,
    pub icon: Option<String>,
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.5, 0.5, 0.8),
            size: 1.0,
            shape: NodeShape::Sphere,
            icon: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NodeShape {
    Sphere,
    Cube,
    Cylinder,
    Cone,
    Torus,
    Custom,
}

/// Component for nodes that can be selected
#[derive(Component, Default)]
pub struct Selectable {
    pub selected: bool,
    pub hovered: bool,
}

/// Component for nodes that can be dragged
#[derive(Component, Default)]
pub struct Draggable {
    pub is_dragging: bool,
    pub drag_offset: Vec3,
}

/// Layout component for automatic positioning
#[derive(Component, Debug, Clone)]
pub struct LayoutNode {
    pub layout_type: LayoutType,
    pub fixed: bool,
    pub force: Vec3,
    pub velocity: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutType {
    ForceDirected,
    Hierarchical,
    Circular,
    Grid,
    Random,
    Manual,
}

/// Graph container component - attached to a parent entity
#[derive(Component, Debug, Clone)]
pub struct Graph {
    pub id: String,
    pub name: String,
    pub nodes: HashSet<Entity>,
    pub edges: HashSet<Entity>,
    pub layout: LayoutType,
}

impl Graph {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            nodes: HashSet::new(),
            edges: HashSet::new(),
            layout: LayoutType::ForceDirected,
        }
    }

    pub fn add_node(&mut self, entity: Entity) {
        self.nodes.insert(entity);
    }

    pub fn add_edge(&mut self, entity: Entity) {
        self.edges.insert(entity);
    }

    pub fn remove_node(&mut self, entity: Entity) {
        self.nodes.remove(&entity);
    }

    pub fn remove_edge(&mut self, entity: Entity) {
        self.edges.remove(&entity);
    }
}

/// Component for graph statistics
#[derive(Component, Debug, Clone, Default)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub connected_components: usize,
    pub average_degree: f32,
    pub density: f32,
}

/// Persistence component - marks entities that should be saved
#[derive(Component)]
pub struct Persistent {
    pub collection: String,
    pub last_saved: Option<std::time::Instant>,
    pub dirty: bool,
}

/// Event component for JetStream integration
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct GraphEvent {
    pub event_type: GraphEventType,
    pub timestamp: i64,
    pub graph_id: String,
    pub entity_id: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphEventType {
    NodeAdded,
    NodeRemoved,
    NodeUpdated,
    EdgeAdded,
    EdgeRemoved,
    EdgeUpdated,
    GraphCreated,
    GraphDeleted,
    LayoutChanged,
}

/// Component for tracking node connections
#[derive(Component, Debug, Clone, Default)]
pub struct NodeConnections {
    pub incoming: HashSet<Entity>,
    pub outgoing: HashSet<Entity>,
}

impl NodeConnections {
    pub fn degree(&self) -> usize {
        self.incoming.len() + self.outgoing.len()
    }

    pub fn in_degree(&self) -> usize {
        self.incoming.len()
    }

    pub fn out_degree(&self) -> usize {
        self.outgoing.len()
    }
}

/// Animation component for smooth transitions
#[derive(Component)]
pub struct NodeAnimation {
    pub start_position: Vec3,
    pub target_position: Vec3,
    pub start_time: f32,
    pub duration: f32,
    pub easing: EasingFunction,
}

#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseInOut,
    EaseIn,
    EaseOut,
    Bounce,
}

/// Search/filter component
#[derive(Component, Default)]
pub struct Searchable {
    pub matched: bool,
    pub search_text: String,
    pub search_score: f32,
}

/// Clustering component for grouped nodes
#[derive(Component)]
pub struct ClusterMember {
    pub cluster_id: String,
    pub cluster_center: Vec3,
    pub cluster_color: Color,
}

/// Component for hierarchical graphs
#[derive(Component)]
pub struct HierarchyNode {
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
    pub level: u32,
    pub subtree_size: u32,
}

/// Physics component for force-directed layout
#[derive(Component)]
pub struct PhysicsNode {
    pub mass: f32,
    pub charge: f32,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub damping: f32,
}

impl Default for PhysicsNode {
    fn default() -> Self {
        Self {
            mass: 1.0,
            charge: -30.0,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            damping: 0.8,
        }
    }
}

/// Resource for managing multiple graphs
#[derive(Resource, Default)]
pub struct GraphManager {
    pub graphs: HashMap<String, Entity>,
    pub active_graph: Option<Entity>,
}

impl GraphManager {
    pub fn create_graph(&mut self, id: String, name: String) -> Entity {
        // This would be populated by a system
        Entity::from_raw(0)
    }

    pub fn get_graph(&self, id: &str) -> Option<Entity> {
        self.graphs.get(id).copied()
    }

    pub fn set_active(&mut self, entity: Entity) {
        self.active_graph = Some(entity);
    }
}

/// Marker component for edge mesh entities
#[derive(Component)]
pub struct EdgeMesh;

/// Component for edge styling
#[derive(Component)]
pub struct EdgeStyle {
    pub color: Color,
    pub width: f32,
    pub arrow: bool,
    pub dashed: bool,
    pub curve: f32,
}

impl Default for EdgeStyle {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.6, 0.6, 0.6),
            width: 0.1,
            arrow: true,
            dashed: false,
            curve: 0.0,
        }
    }
}

/// Bundle for creating edge entities
#[derive(Bundle)]
pub struct GraphEdgeBundle {
    pub edge: GraphEdge,
    pub style: EdgeStyle,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// Component for tracking graph modifications
#[derive(Component, Default)]
pub struct GraphHistory {
    pub version: u64,
    pub modifications: Vec<GraphModification>,
    pub undo_stack: Vec<GraphModification>,
    pub redo_stack: Vec<GraphModification>,
}

#[derive(Debug, Clone)]
pub struct GraphModification {
    pub timestamp: i64,
    pub operation: ModificationOp,
    pub entity: Entity,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum ModificationOp {
    AddNode,
    RemoveNode,
    UpdateNode,
    AddEdge,
    RemoveEdge,
    UpdateEdge,
    ChangeLayout,
}

/// Label component for rendering text
#[derive(Component)]
pub struct NodeLabel {
    pub text: String,
    pub font_size: f32,
    pub color: Color,
    pub offset: Vec3,
    pub billboard: bool,
}

impl Default for NodeLabel {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_size: 14.0,
            color: Color::WHITE,
            offset: Vec3::new(0.0, 1.0, 0.0),
            billboard: true,
        }
    }
}

/// Component for graph algorithms results
#[derive(Component, Default)]
pub struct GraphAlgorithmResult {
    pub centrality: f32,
    pub clustering_coefficient: f32,
    pub shortest_paths: HashMap<Entity, (f32, Vec<Entity>)>,
    pub component_id: usize,
}

/// Query marker for finding related components
#[derive(Component)]
pub struct GraphQuery {
    pub query_type: QueryType,
    pub parameters: HashMap<String, String>,
    pub results: Vec<Entity>,
}

#[derive(Debug, Clone)]
pub enum QueryType {
    Neighbors,
    ShortestPath,
    Subgraph,
    Pattern,
    Semantic,
}

/// Event for graph operations
#[derive(Event)]
pub struct GraphOperationEvent {
    pub operation: GraphOperation,
    pub graph_id: String,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone)]
pub enum GraphOperation {
    CreateNode { id: String, label: String, position: Vec3 },
    CreateEdge { source: Entity, target: Entity, label: Option<String> },
    DeleteNode { entity: Entity },
    DeleteEdge { entity: Entity },
    UpdateNode { entity: Entity, label: Option<String>, metadata: Option<HashMap<String, serde_json::Value>> },
    UpdateEdge { entity: Entity, weight: Option<f32>, label: Option<String> },
    ChangeLayout { layout_type: LayoutType },
    ApplyLayout { layout_type: LayoutType },
    Clear,
    SaveGraph,
    LoadGraph { path: String },
}

/// Component for semantic embeddings
#[derive(Component)]
pub struct SemanticEmbedding {
    pub vector: Vec<f32>,
    pub model: String,
    pub timestamp: i64,
}

/// Resource for graph configuration
#[derive(Resource)]
pub struct GraphConfig {
    pub auto_layout: bool,
    pub physics_enabled: bool,
    pub edge_bundling: bool,
    pub node_labels: bool,
    pub edge_labels: bool,
    pub selection_mode: SelectionMode,
    pub render_mode: RenderMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionMode {
    Single,
    Multiple,
    Box,
    Lasso,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    Solid,
    Wireframe,
    Points,
    Mixed,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            auto_layout: true,
            physics_enabled: true,
            edge_bundling: false,
            node_labels: true,
            edge_labels: false,
            selection_mode: SelectionMode::Single,
            render_mode: RenderMode::Solid,
        }
    }
}

/// Resource for graph layout settings
#[derive(Resource, Default)]
pub struct GraphLayoutSettings {
    pub node_spacing: f32,
    pub edge_length: f32,
    pub repulsion_force: f32,
    pub centering_force: f32,
}

impl GraphLayoutSettings {
    pub fn new() -> Self {
        Self {
            node_spacing: 3.0,
            edge_length: 5.0,
            repulsion_force: 100.0,
            centering_force: 0.01,
        }
    }
}