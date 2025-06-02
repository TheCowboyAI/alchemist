use bevy::prelude::*;

mod contexts;

#[cfg(test)]
mod testing;

fn main() {
    let mut app = App::new()
        .add_plugins(DefaultPlugins)
        // Add our domain contexts
        .add_plugins((
            contexts::graph_management::plugin::GraphManagementPlugin,
            contexts::visualization::plugin::VisualizationPlugin,
            contexts::selection::plugin::SelectionPlugin,
        ));

    app.run();
}
