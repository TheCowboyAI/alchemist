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

use workflow_editor::WorkflowEditor;
use graph_editor::GraphEditor;
use graph_editor_3d::GraphEditor3DPlugin;
use graph_editor_ui::{GraphEditorUiPlugin, GraphEditorTheme};

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
        .add_systems(Update, toggle_theme_system)
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