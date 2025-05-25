//! UI-related events for panel management and user interface interactions
//!
//! These events handle:
//! - Panel visibility and layout
//! - Workspace modes and states
//! - Context menus and tooltips
//! - Theme and appearance changes
//! - UI notifications and feedback

use bevy::prelude::*;

/// Event for toggling standard graph editor visibility
///
/// ## Producers
/// - Menu bar toggle buttons
/// - Keyboard shortcuts (e.g., Ctrl+1)
///
/// ## Consumers
/// - `editor_visibility_system` - Shows/hides editor UI
#[derive(Event)]
pub struct ToggleStandardGraphEditorEvent(pub bool);

/// Event for toggling workflow editor visibility
#[derive(Event)]
pub struct ToggleWorkflowEditorEvent(pub bool);

/// Event for toggling DDD editor visibility
#[derive(Event)]
pub struct ToggleDddEditorEvent(pub bool);

/// Event for toggling ECS editor visibility
#[derive(Event)]
pub struct ToggleEcsEditorEvent(pub bool);

/// Event for toggling panel visibility
#[derive(Event)]
pub struct TogglePanelEvent {
    pub panel: PanelType,
    pub visible: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PanelType {
    LeftPanel,
    RightPanel,
    BottomPanel,
    Inspector,
    Control,
}

/// Event for showing tooltips
#[derive(Event)]
pub struct ShowTooltipEvent {
    pub text: String,
    pub position: Vec2,
}

/// Event for hiding tooltips
#[derive(Event)]
pub struct HideTooltipEvent;

/// Event for showing context menu
#[derive(Event)]
pub struct ShowContextMenuEvent {
    pub position: Vec2,
    pub context: ContextMenuContext,
}

#[derive(Clone)]
pub enum ContextMenuContext {
    Node(Entity),
    Edge(Entity),
    Background,
}

/// Event for workspace mode changes
#[derive(Event)]
pub struct ChangeWorkspaceModeEvent {
    pub mode: WorkspaceMode,
}

#[derive(Clone, Copy, PartialEq)]
pub enum WorkspaceMode {
    Edit,
    View,
    Debug,
}

/// Event for theme changes
#[derive(Event)]
pub struct ChangeThemeEvent {
    pub theme_name: String,
}

/// Event for UI notification display
///
/// ## Producers
/// - File operation completion
/// - Validation errors
/// - System messages
///
/// ## Consumers
/// - `notification_system` - Displays toast notifications
#[derive(Event)]
pub struct ShowNotificationEvent {
    pub message: String,
    pub notification_type: NotificationType,
    pub duration_seconds: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

/// Event for modal dialog display
///
/// ## Producers
/// - Confirmation prompts
/// - File dialogs
/// - Settings windows
///
/// ## Consumers
/// - `modal_system` - Manages modal dialogs
#[derive(Event)]
pub struct ShowModalEvent {
    pub modal_type: ModalType,
    pub title: String,
    pub content: ModalContent,
}

#[derive(Clone)]
pub enum ModalType {
    Confirmation,
    FileDialog,
    Settings,
    About,
    Custom(String),
}

#[derive(Clone)]
pub enum ModalContent {
    Text(String),
    FileSelector { filter: String, save_mode: bool },
    Settings { tab: String },
    Custom(String),
}

/// Event for status bar updates
///
/// ## Producers
/// - Graph metrics systems
/// - Selection systems
/// - Operation status systems
///
/// ## Consumers
/// - `status_bar_system` - Updates status bar display
#[derive(Event)]
pub struct UpdateStatusBarEvent {
    pub section: StatusBarSection,
    pub text: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StatusBarSection {
    Left,
    Center,
    Right,
    Coordinates,
    Selection,
    Mode,
}

/// Event for toolbar state changes
///
/// ## Producers
/// - Tool selection UI
/// - Keyboard shortcuts
///
/// ## Consumers
/// - `toolbar_system` - Updates active tool
/// - `cursor_system` - Changes cursor appearance
#[derive(Event)]
pub struct ChangeToolEvent {
    pub tool: EditorTool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EditorTool {
    Select,
    Move,
    CreateNode,
    CreateEdge,
    Delete,
    Pan,
    Zoom,
}

/// Event for sidebar content changes
///
/// ## Producers
/// - Tab selection
/// - Context-sensitive updates
///
/// ## Consumers
/// - `sidebar_system` - Updates sidebar content
#[derive(Event)]
pub struct UpdateSidebarEvent {
    pub sidebar: SidebarType,
    pub content: SidebarContent,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SidebarType {
    Left,
    Right,
}

#[derive(Clone)]
pub enum SidebarContent {
    NodeList,
    Properties,
    Layers,
    History,
    Search,
}

/// Event for keyboard shortcut registration
///
/// ## Producers
/// - Settings changes
/// - Plugin initialization
///
/// ## Consumers
/// - `shortcut_system` - Updates shortcut mappings
#[derive(Event)]
pub struct RegisterShortcutEvent {
    pub key_combination: KeyCombination,
    pub action: ShortcutAction,
}

#[derive(Clone, PartialEq)]
pub struct KeyCombination {
    pub key: bevy::input::keyboard::KeyCode,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

#[derive(Clone)]
pub enum ShortcutAction {
    TogglePanel(PanelType),
    ChangeTool(EditorTool),
    FileOperation(FileAction),
    GraphOperation(GraphAction),
    Custom(String),
}

#[derive(Clone)]
pub enum FileAction {
    New,
    Open,
    Save,
    SaveAs,
    Export,
}

#[derive(Clone)]
pub enum GraphAction {
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    Delete,
    SelectAll,
}

/// Event for UI layout changes
///
/// ## Producers
/// - Window resize
/// - Layout preset selection
/// - Panel drag handles
///
/// ## Consumers
/// - `layout_system` - Adjusts UI layout
#[derive(Event)]
pub struct ChangeLayoutEvent {
    pub layout: LayoutPreset,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LayoutPreset {
    Default,
    Minimal,
    Focus,
    Wide,
    Custom,
}

/// Event for help system activation
///
/// ## Producers
/// - F1 key press
/// - Help menu items
/// - Context-sensitive help buttons
///
/// ## Consumers
/// - `help_system` - Shows help content
#[derive(Event)]
pub struct ShowHelpEvent {
    pub help_topic: HelpTopic,
}

#[derive(Clone)]
pub enum HelpTopic {
    General,
    Tool(EditorTool),
    Feature(String),
    Shortcut(String),
}
