//! Information Alchemist - Main Entry Point

use bevy::prelude::*;
use ia::GraphEditorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphEditorPlugin)
        .run();
}
