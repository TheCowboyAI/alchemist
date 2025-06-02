use bevy::prelude::*;

/// Create a test app with minimal plugins to avoid render dependencies
pub fn create_test_app() -> App {
    let mut app = App::new();

    // Use minimal plugins instead of DefaultPlugins
    app.add_plugins(MinimalPlugins.set(TaskPoolPlugin {
        task_pool_options: TaskPoolOptions::with_num_threads(1),
    }));

    // Add only ECS-related plugins needed for tests
    app.add_plugins(TransformPlugin);

    app
}
