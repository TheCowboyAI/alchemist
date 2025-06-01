// Testing modules for automated UI testing

#[cfg(test)]
pub mod headless_integration_test;

#[cfg(test)]
pub mod graph_editor_automated_tests;

#[cfg(feature = "integration-tests")]
pub mod enigo_integration;

pub mod bevy_test_framework;
