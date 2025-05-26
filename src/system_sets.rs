use bevy::prelude::*;

/// System sets for organizing graph editor systems
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GraphSystemSet {
    /// Input gathering and initial event generation
    Input,
    /// Processing events in dependency order
    EventProcessing,
    /// Updating component states based on events
    StateUpdate,
    /// Detecting changes for rendering and UI
    ChangeDetection,
    /// UI systems that need stable state
    UI,
    /// Rendering preparation in PostUpdate
    RenderPrep,
}

/// System sets for camera-specific operations
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CameraSystemSet {
    /// Camera input handling
    Input,
    /// Camera state updates and transitions
    Update,
    /// Viewport and bounds updates
    Viewport,
}

/// System sets for file operations
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileSystemSet {
    /// File event generation
    Events,
    /// File processing
    Processing,
}

/// Resource to track what changed this frame
#[derive(Resource, Default)]
pub struct GraphChangeFlags {
    pub nodes_changed: bool,
    pub edges_changed: bool,
    pub view_mode_changed: bool,
    pub selection_changed: bool,
    pub graph_structure_changed: bool,
}

impl GraphChangeFlags {
    /// Reset all flags (call at end of frame)
    pub fn reset(&mut self) {
        self.nodes_changed = false;
        self.edges_changed = false;
        self.view_mode_changed = false;
        self.selection_changed = false;
        self.graph_structure_changed = false;
    }
}

/// Run condition: nodes changed this frame
pub fn nodes_changed(flags: Res<GraphChangeFlags>) -> bool {
    flags.nodes_changed
}

/// Run condition: edges changed this frame
pub fn edges_changed(flags: Res<GraphChangeFlags>) -> bool {
    flags.edges_changed
}

/// Run condition: view mode changed this frame
pub fn view_mode_changed(flags: Res<GraphChangeFlags>) -> bool {
    flags.view_mode_changed
}

/// Run condition: selection changed this frame
pub fn selection_changed(flags: Res<GraphChangeFlags>) -> bool {
    flags.selection_changed
}

/// Run condition: graph structure changed this frame
pub fn graph_structure_changed(flags: Res<GraphChangeFlags>) -> bool {
    flags.graph_structure_changed
}

/// System to reset change flags at the end of the frame
pub fn reset_change_flags(mut flags: ResMut<GraphChangeFlags>) {
    flags.reset();
}

/// System sets for UI operations
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UISystemSet {
    /// Update UI state based on app state
    StateSync,
    /// Process UI events
    EventHandling,
    /// Render UI panels (after egui init)
    PanelRender,
    /// Render overlays and tooltips
    OverlayRender,
}

/// System sets for file I/O operations
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum IOSystemSet {
    /// File loading operations
    Load,
    /// File saving operations
    Save,
    /// Auto-save operations
    AutoSave,
}

/// Configure the system execution order
pub fn configure_system_sets(app: &mut App) {
    // Configure graph system ordering
    app.configure_sets(
        Update,
        (
            GraphSystemSet::Input,
            GraphSystemSet::EventProcessing,
            GraphSystemSet::StateUpdate,
            GraphSystemSet::ChangeDetection,
        )
            .chain(),
    );

    // GraphSystemSet::UI must run after egui initialization
    app.configure_sets(
        Update,
        GraphSystemSet::UI
            .after(GraphSystemSet::ChangeDetection)
            .after(bevy_egui::EguiPreUpdateSet::InitContexts),
    );

    // Configure camera system ordering
    app.configure_sets(
        Update,
        (
            CameraSystemSet::Input,
            CameraSystemSet::Update,
            CameraSystemSet::Viewport,
        )
            .chain()
            .after(GraphSystemSet::EventProcessing),
    );

    // Configure UI system ordering - must run after egui initialization
    app.configure_sets(
        Update,
        (
            UISystemSet::StateSync,
            UISystemSet::EventHandling,
            UISystemSet::PanelRender,
            UISystemSet::OverlayRender,
        )
            .chain()
            .after(bevy_egui::EguiPreUpdateSet::InitContexts)
            .before(bevy_egui::EguiPreUpdateSet::ProcessInput),
    );

    // Configure rendering preparation in PostUpdate
    app.configure_sets(
        PostUpdate,
        GraphSystemSet::RenderPrep,
    );

    // Configure I/O system ordering
    app.configure_sets(
        Update,
        (
            IOSystemSet::Load,
            IOSystemSet::Save,
        )
            .chain()
            .after(GraphSystemSet::StateUpdate),
    );

    // Auto-save runs independently at the end
    app.configure_sets(
        Update,
        IOSystemSet::AutoSave
            .after(IOSystemSet::Save),
    );
}
