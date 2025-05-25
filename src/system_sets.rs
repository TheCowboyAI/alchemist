use bevy::prelude::*;

/// System sets for graph operations
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GraphSystemSet {
    /// Input collection (raw input events)
    Input,
    /// Event processing (handle events, validate)
    EventProcessing,
    /// Graph structure updates
    GraphUpdate,
    /// Visual component updates
    VisualUpdate,
    /// Rendering preparation
    RenderPrep,
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

/// System sets for camera operations
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CameraSystemSet {
    /// Process camera input
    Input,
    /// Update camera state
    StateUpdate,
    /// Apply camera transform
    TransformUpdate,
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
            GraphSystemSet::GraphUpdate,
            GraphSystemSet::VisualUpdate,
        )
            .chain(),
    );

    // Configure camera system ordering
    app.configure_sets(
        Update,
        (
            CameraSystemSet::Input,
            CameraSystemSet::StateUpdate,
            CameraSystemSet::TransformUpdate,
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
            .after(GraphSystemSet::GraphUpdate),
    );

    // Auto-save runs independently at the end
    app.configure_sets(
        Update,
        IOSystemSet::AutoSave
            .after(IOSystemSet::Save),
    );
}
