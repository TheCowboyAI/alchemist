use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin};

// Import your existing modules
mod ecs;
mod events;
mod graph;
mod graph_patterns;
mod models;
mod workflow_editor;
mod graph_editor;

// Import the 3D graph editor modules
mod graph_editor_3d;
mod graph_editor_ui;
// Import the new force-directed layout module
mod graph_layout;
// Import the DDD editor modules
mod ddd_editor;
mod ddd_editor_3d;
// Import the dashboard UI module
mod dashboard_ui;

use workflow_editor::{WorkflowEditor, WorkflowEditorPlugin};
use graph_editor::{GraphEditor, GraphEditorPlugin};
use graph_editor_3d::GraphEditor3DPlugin;
use graph_editor_ui::{GraphEditorUiPlugin, GraphEditorTheme};
// Import the new layout plugin
use graph_layout::GraphLayoutPlugin;
// Import the DDD editor plugins
use ddd_editor::DddEditorPlugin;
use ddd_editor_3d::DddEditor3dPlugin;
// Import the dashboard UI plugin
use dashboard_ui::DashboardUiPlugin;
use ecs::EcsEditorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Alchemist Graph Editor".to_string(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        })
        .set(ImagePlugin::default_nearest()) // Use nearest filtering for sharper pixel art textures
        )
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: false })
        .init_resource::<WorkflowEditor>()
        .init_resource::<GraphEditor>()
        .init_resource::<GraphEditorTheme>()
        // Add our 3D graph editor plugins
        .add_plugins(GraphEditor3DPlugin)
        .add_plugins(GraphEditorUiPlugin)
        // Add our new force-directed layout plugin
        .add_plugins(GraphLayoutPlugin)
        // Add the DDD editor plugins
        .add_plugins(DddEditorPlugin)
        .add_plugins(DddEditor3dPlugin)
        // Add the editor plugins for visibility control
        .add_plugins(GraphEditorPlugin)
        .add_plugins(WorkflowEditorPlugin)
        .add_plugins(EcsEditorPlugin)
        // Add the dashboard UI plugin
        .add_plugins(DashboardUiPlugin)
        .add_systems(Update, (
            toggle_theme_system,
            ui_editor_system,
        ))
        .run();
}

// System to allow toggling theme with keyboard shortcuts
fn toggle_theme_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut theme: ResMut<GraphEditorTheme>,
) {
    // Toggle theme with Ctrl+T
    if keyboard_input.just_pressed(KeyCode::KeyT) && 
       (keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight)) {
        theme.use_custom_theme = !theme.use_custom_theme;
    }
}

// Main UI system that renders all editor windows
fn ui_editor_system(
    mut contexts: EguiContexts,
    mut graph_editor: ResMut<GraphEditor>,
    mut workflow_editor: ResMut<WorkflowEditor>,
) {
    // Render the standard graph editor if visible
    graph_editor.ui(contexts.ctx_mut());
    
    // Render the workflow editor if visible
    workflow_editor.ui(contexts.ctx_mut());
    
    // The DDD editor and ECS editor are rendered through their own plugin systems
}