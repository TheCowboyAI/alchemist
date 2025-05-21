use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_egui::{EguiContexts, EguiPlugin};

// Import your existing modules
mod ecs;
mod events;
mod graph;
mod graph_patterns;
mod models;
mod workflow_editor;
mod graph_editor;

use workflow_editor::WorkflowEditor;
use graph_editor::GraphEditor;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: false })
        .init_resource::<WorkflowEditor>()
        .init_resource::<GraphEditor>()
        .add_systems(Startup, setup)
        .add_systems(Update, ui_system)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a 3D camera with orbit controls
    commands.spawn((
        Camera3d::default(),
        PanOrbitCamera::default(),
    ));
    
    // Add a light to see things
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn ui_system(mut contexts: EguiContexts, mut workflow_editor: ResMut<WorkflowEditor>, mut graph_editor: ResMut<GraphEditor>) {
    // Call the workflow editor UI
    workflow_editor.ui(contexts.ctx_mut());
    
    // Call the graph editor UI
    graph_editor.ui(contexts.ctx_mut());
    
    // Initialize the graph if it's empty
    if graph_editor.snarl_graph.is_none() {
        graph_editor.sync_to_snarl();
    }
}