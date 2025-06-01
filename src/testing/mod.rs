// Testing modules for automated UI testing

#[cfg(test)]
pub mod headless_integration_test;

#[cfg(test)]
pub mod graph_editor_automated_tests;

#[cfg(test)]
pub mod domain_isolated_tests;

#[cfg(test)]
pub mod tdd_compliant_ecs_tests;

#[cfg(test)]
pub mod event_validation_helpers;

#[cfg(test)]
pub mod repository_integration_tests;

#[cfg(feature = "integration-tests")]
pub mod enigo_integration;

pub mod bevy_test_framework;
