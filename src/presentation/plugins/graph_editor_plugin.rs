//! Integrated graph editor plugin
//!
//! Combines all graph visualization and interaction features into a cohesive
//! editor experience for conceptual graphs.

use bevy::prelude::*;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::window::PrimaryWindow;
use std::collections::VecDeque;

use crate::presentation::systems::{
    ConceptualVisualizationPlugin, NodeInteractionPlugin, ContextBridgeVisualizationPlugin,
};
use crate::presentation::components::{
    ConceptualNodeVisual, ConceptualEdgeVisual, ConceptualSpaceVisual,
    QualityDimensionAxis, DraggableNode, ConnectableNode, SelectableGraph,
    SelectionMode, TransitionAnimation, EasingFunction,
    SpaceId, SpaceBounds, GridSettings,
};
use crate::domain::value_objects::{GraphId, NodeId, EdgeId, Position3D};
use crate::domain::commands::graph_commands::GraphCommand;
use crate::domain::conceptual_graph::{ConceptType, ConceptualPoint, QualityDimension};
use crate::presentation::events::{DragStart, DragEnd, PresentationCommand};

/// Main graph editor plugin that integrates all visualization features
pub struct GraphEditorPlugin;

impl Plugin for GraphEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add sub-plugins
            .add_plugins((
                ConceptualVisualizationPlugin,
                NodeInteractionPlugin,
                ContextBridgeVisualizationPlugin,
                GraphEditorUIPlugin,
            ))
            // Initialize resources
            .init_resource::<GraphEditorState>()
            .init_resource::<GraphEditorSettings>()
            // Add startup systems
            .add_systems(Startup, (
                setup_default_conceptual_space,
                setup_graph_editor_ui,
            ))
            // Add update systems
            .add_systems(Update, (
                handle_editor_shortcuts,
                update_editor_state,
                handle_tool_selection,
            ));
    }
}

/// State of the graph editor
#[derive(Resource, Debug, Clone)]
pub struct GraphEditorState {
    /// Currently active graph
    pub active_graph: Option<GraphId>,

    /// Current editing mode
    pub mode: EditorMode,

    /// Selected tool
    pub tool: EditorTool,

    /// Clipboard for copy/paste operations
    pub clipboard: Vec<ClipboardItem>,

    /// Undo/redo history
    pub history: EditorHistory,

    /// Current file path (if any)
    pub file_path: Option<String>,

    /// Whether there are unsaved changes
    pub has_unsaved_changes: bool,
}

impl Default for GraphEditorState {
    fn default() -> Self {
        Self {
            active_graph: None,
            mode: EditorMode::Select,
            tool: EditorTool::Select,
            clipboard: Vec::new(),
            history: EditorHistory::default(),
            file_path: None,
            has_unsaved_changes: false,
        }
    }
}

/// Editor modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorMode {
    Select,
    Create,
    Connect,
    Edit,
    View,
}

/// Editor tools
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorTool {
    Select,
    Move,
    CreateNode,
    CreateEdge,
    Delete,
    Pan,
    Zoom,
    Rotate,
}

/// Clipboard item for copy/paste
#[derive(Debug, Clone)]
pub struct ClipboardItem {
    pub item_type: ClipboardItemType,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum ClipboardItemType {
    Node,
    Edge,
    Subgraph,
}

/// Editor history for undo/redo
#[derive(Debug, Clone, Default)]
pub struct EditorHistory {
    pub undo_stack: Vec<EditorAction>,
    pub redo_stack: Vec<EditorAction>,
    pub max_history: usize,
}

impl EditorHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
        }
    }

    pub fn push_action(&mut self, action: EditorAction) {
        self.undo_stack.push(action);
        self.redo_stack.clear();

        // Limit history size
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> Option<EditorAction> {
        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<EditorAction> {
        if let Some(action) = self.redo_stack.pop() {
            self.undo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }
}

/// Represents an action that can be undone/redone
#[derive(Debug, Clone)]
pub enum EditorAction {
    CreateNode { node_id: NodeId },
    DeleteNode { node_id: NodeId },
    MoveNode {
        node_id: NodeId,
        old_position: Vec3,
        new_position: Vec3,
    },
    CreateEdge { edge_id: EdgeId },
    DeleteEdge { edge_id: EdgeId },
}

/// Settings for the graph editor
#[derive(Resource, Debug, Clone)]
pub struct GraphEditorSettings {
    /// Auto-save interval in seconds (0 to disable)
    pub auto_save_interval: f32,

    /// Show grid by default
    pub show_grid: bool,

    /// Show axes by default
    pub show_axes: bool,

    /// Enable node snapping
    pub snap_to_grid: bool,

    /// Grid size for snapping
    pub grid_size: f32,

    /// Theme
    pub theme: EditorTheme,
}

impl Default for GraphEditorSettings {
    fn default() -> Self {
        Self {
            auto_save_interval: 60.0,
            show_grid: true,
            show_axes: true,
            snap_to_grid: false,
            grid_size: 0.5,
            theme: EditorTheme::Dark,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorTheme {
    Light,
    Dark,
    HighContrast,
}

/// Sets up the default conceptual space
fn setup_default_conceptual_space(
    mut commands: Commands,
    settings: Res<GraphEditorSettings>,
) {
    // Create default quality dimensions
    let dimensions = vec![
        QualityDimensionAxis {
            dimension: QualityDimension {
                name: "Complexity".to_string(),
                dimension_type: crate::domain::conceptual_graph::DimensionType::Continuous,
                range: 0.0..1.0,
                metric: crate::domain::conceptual_graph::DistanceMetric::Euclidean,
                weight: 1.0,
            },
            axis_direction: Vec3::X,
            scale: 1.0,
            color: Color::srgb(1.0, 0.3, 0.3),
            show_labels: true,
            label_entities: Vec::new(),
        },
        QualityDimensionAxis {
            dimension: QualityDimension {
                name: "Abstraction".to_string(),
                dimension_type: crate::domain::conceptual_graph::DimensionType::Continuous,
                range: 0.0..1.0,
                metric: crate::domain::conceptual_graph::DistanceMetric::Euclidean,
                weight: 1.0,
            },
            axis_direction: Vec3::Y,
            scale: 1.0,
            color: Color::srgb(0.3, 1.0, 0.3),
            show_labels: true,
            label_entities: Vec::new(),
        },
        QualityDimensionAxis {
            dimension: QualityDimension {
                name: "Coupling".to_string(),
                dimension_type: crate::domain::conceptual_graph::DimensionType::Continuous,
                range: 0.0..1.0,
                metric: crate::domain::conceptual_graph::DistanceMetric::Euclidean,
                weight: 1.0,
            },
            axis_direction: Vec3::Z,
            scale: 1.0,
            color: Color::srgb(0.3, 0.3, 1.0),
            show_labels: true,
            label_entities: Vec::new(),
        },
    ];

    // Create conceptual space
    let space = ConceptualSpaceVisual {
        space_id: SpaceId::new(),
        dimensions,
        origin: Vec3::ZERO,
        bounds: SpaceBounds::default(),
        grid_settings: GridSettings {
            visible: settings.show_grid,
            spacing: settings.grid_size,
            ..default()
        },
    };

    // Create selectable graph
    let graph = SelectableGraph {
        graph_id: GraphId::new(),
        selection_mode: SelectionMode::Multiple,
        selected_entities: Vec::new(),
        selection_start: None,
    };

    // Spawn entities
    commands.spawn((
        space,
        graph,
        Name::new("DefaultConceptualSpace"),
    ));
}

/// Sets up the graph editor UI
fn setup_graph_editor_ui(
    mut commands: Commands,
) {
    // This would create the UI elements like toolbar, property inspector, etc.
    // For now, we'll just add a marker entity
    commands.spawn((
        GraphEditorUI,
        Name::new("GraphEditorUI"),
    ));
}

/// Marker component for the graph editor UI
#[derive(Component)]
struct GraphEditorUI;

/// Handles keyboard shortcuts for the editor
fn handle_editor_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor_state: ResMut<GraphEditorState>,
    mut graph_commands: EventWriter<PresentationCommand>,
) {
    // Tool selection shortcuts
    if keyboard.just_pressed(KeyCode::KeyV) {
        editor_state.tool = EditorTool::Select;
        editor_state.mode = EditorMode::Select;
    }
    if keyboard.just_pressed(KeyCode::KeyM) {
        editor_state.tool = EditorTool::Move;
        editor_state.mode = EditorMode::Edit;
    }
    if keyboard.just_pressed(KeyCode::KeyN) {
        editor_state.tool = EditorTool::CreateNode;
        editor_state.mode = EditorMode::Create;
    }
    if keyboard.just_pressed(KeyCode::KeyE) {
        editor_state.tool = EditorTool::CreateEdge;
        editor_state.mode = EditorMode::Connect;
    }

    // Undo/Redo
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if ctrl && keyboard.just_pressed(KeyCode::KeyZ) {
        if shift {
            // Redo
            if let Some(action) = editor_state.history.redo() {
                apply_editor_action(action, &mut graph_commands);
            }
        } else {
            // Undo
            if let Some(action) = editor_state.history.undo() {
                apply_reverse_action(action, &mut graph_commands);
            }
        }
    }

    // Save
    if ctrl && keyboard.just_pressed(KeyCode::KeyS) {
        // TODO: Implement save functionality
        editor_state.has_unsaved_changes = false;
    }

    // Delete
    if keyboard.just_pressed(KeyCode::Delete) {
        // TODO: Delete selected entities
    }
}

/// Updates the editor state based on user actions
fn update_editor_state(
    mut editor_state: ResMut<GraphEditorState>,
    graph_query: Query<&SelectableGraph>,
) {
    // Update active graph if needed
    if editor_state.active_graph.is_none() {
        if let Ok(graph) = graph_query.get_single() {
            editor_state.active_graph = Some(graph.graph_id);
        }
    }
}

/// Handles tool-specific behavior
fn handle_tool_selection(
    editor_state: Res<GraphEditorState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut graph_commands: EventWriter<PresentationCommand>,
) {
    match editor_state.tool {
        EditorTool::CreateNode => {
            if mouse_button.just_pressed(MouseButton::Left) {
                // Get click position and create node
                if let Ok(window) = windows.get_single() {
                    if let Some(cursor_pos) = window.cursor_position() {
                        if let Ok((camera, camera_transform)) = camera_query.get_single() {
                            if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
                                // Create node at intersection with Y=0 plane
                                if let Ok(position) = ray_plane_intersection(&ray, Vec3::Y, 0.0) {
                                    if let Some(graph_id) = editor_state.active_graph {
                                        graph_commands.send(PresentationCommand::new(
                                            GraphCommand::AddNode {
                                                graph_id,
                                                node_id: NodeId::new(),
                                                node_type: "concept".to_string(),
                                                position: Position3D {
                                                    x: position.x,
                                                    y: position.y,
                                                    z: position.z,
                                                },
                                                content: serde_json::json!({}),
                                            }
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {
            // Other tools handled by their respective systems
        }
    }
}

/// Helper function for ray-plane intersection
fn ray_plane_intersection(ray: &Ray3d, plane_normal: Vec3, plane_distance: f32) -> Result<Vec3, ()> {
    let denominator = ray.direction.dot(plane_normal);

    if denominator.abs() < 0.0001 {
        return Err(()); // Ray is parallel to plane
    }

    let t = (plane_distance - ray.origin.dot(plane_normal)) / denominator;

    if t < 0.0 {
        return Err(()); // Intersection is behind ray origin
    }

    Ok(ray.origin + ray.direction * t)
}

/// Applies an editor action (for redo)
fn apply_editor_action(
    action: EditorAction,
    graph_commands: &mut EventWriter<PresentationCommand>,
) {
    match action {
        EditorAction::CreateNode { node_id } => {
            // Re-create the node
            // This would need to store more data to properly recreate
        }
        EditorAction::MoveNode { node_id, new_position, .. } => {
            // Note: This assumes we have the graph_id stored somewhere
            // In a real implementation, EditorAction would include graph_id
            // For now, we'll skip this as it requires more context
        }
        _ => {
            // Handle other actions
        }
    }
}

/// Applies the reverse of an editor action (for undo)
fn apply_reverse_action(
    action: EditorAction,
    graph_commands: &mut EventWriter<PresentationCommand>,
) {
    match action {
        EditorAction::CreateNode { node_id } => {
            // Note: This assumes we have the graph_id stored somewhere
            // graph_commands.send(PresentationCommand::new(
            //     GraphCommand::RemoveNode { graph_id, node_id }
            // ));
        }
        EditorAction::MoveNode { node_id, old_position, .. } => {
            // Note: This assumes we have the graph_id stored somewhere
            // graph_commands.send(PresentationCommand::new(
            //     GraphCommand::UpdateNode {
            //         graph_id,
            //         node_id,
            //         new_position: Some(Position3D {
            //             x: old_position.x,
            //             y: old_position.y,
            //             z: old_position.z,
            //         }),
            //         new_content: None,
            //     }
            // ));
        }
        _ => {
            // Handle other actions
        }
    }
}

/// UI plugin for the graph editor
struct GraphEditorUIPlugin;

impl Plugin for GraphEditorUIPlugin {
    fn build(&self, app: &mut App) {
        // This would add UI-specific systems
        // For now, it's a placeholder
    }
}
