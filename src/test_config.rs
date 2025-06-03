// Test configuration module for minimal Bevy setup

use bevy::prelude::*;

/// Create a test app with minimal plugins
pub fn create_test_app() -> App {
    let mut app = App::new();

    // Use MinimalPlugins which is specifically designed for headless/testing scenarios
    // This avoids the experimental occlusion culling components that cause linker issues
    app.add_plugins(MinimalPlugins);

    app
}

/// Test helper to run an app update cycle
pub fn run_test_cycle(app: &mut App) {
    app.update();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_app_creation() {
        let mut app = create_test_app();
        run_test_cycle(&mut app);
    }
}
