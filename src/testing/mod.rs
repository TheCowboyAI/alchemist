// Test framework modules for TDD compliance
pub mod bevy_test_framework;
pub mod event_validation_helpers;

// Domain-isolated tests (no Bevy dependencies)
#[cfg(test)]
pub mod domain_isolated_tests;

// ECS integration tests (headless Bevy)
#[cfg(test)]
pub mod tdd_compliant_ecs_tests;

#[cfg(test)]
pub mod graph_editor_automated_tests;

#[cfg(test)]
pub mod repository_integration_tests;

// Specialized integration tests
#[cfg(test)]
pub mod headless_integration_test;

#[cfg(feature = "integration-tests")]
pub mod enigo_integration;

// Test app builder for headless testing
use bevy::prelude::*;

/// Creates a headless test app configured for CI/testing environment
/// Uses MinimalPlugins to avoid experimental occlusion culling linker issues
pub fn create_headless_test_app() -> App {
    let mut app = App::new();

    // Use MinimalPlugins which is specifically designed for headless/testing scenarios
    // This avoids the experimental occlusion culling components that cause linker issues
    // MinimalPlugins includes: TaskPoolPlugin, FrameCountPlugin, TimePlugin, ScheduleRunnerPlugin
    app.add_plugins(MinimalPlugins);

    app
}
