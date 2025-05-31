use bevy::prelude::*;

mod contexts;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add our domain contexts
        .add_plugins((
            contexts::graph_management::plugin::GraphManagementPlugin,
            contexts::visualization::plugin::VisualizationPlugin,
        ))
        .run();
}
