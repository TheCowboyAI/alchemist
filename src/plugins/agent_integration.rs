//! Agent Integration Plugin for Alchemist
//!
//! This plugin integrates the CIM Alchemist AI assistant into the main application,
//! providing context-aware help about the graph editor and workflow system.

use crate::simple_agent::AgentQuestionEvent;
use bevy::prelude::*;

/// Plugin that integrates the AI assistant with the graph editor
pub struct AgentIntegrationPlugin;

impl Plugin for AgentIntegrationPlugin {
    fn build(&self, app: &mut App) {
        // Add integration systems
        app.add_systems(Update, handle_keyboard_shortcuts);
    }
}

/// System to handle keyboard shortcuts for common questions
/// Note: F1 is used to toggle the AI Assistant window (handled in AgentUiPlugin)
fn handle_keyboard_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<AgentQuestionEvent>,
) {
    // F2 - How does event sourcing work?
    if keyboard.just_pressed(KeyCode::F2) {
        events.write(AgentQuestionEvent {
            question: "How does event sourcing work in CIM?".to_string(),
        });
    }

    // F3 - What are the domains?
    if keyboard.just_pressed(KeyCode::F3) {
        events.write(AgentQuestionEvent {
            question: "What are the 8 CIM domains?".to_string(),
        });
    }

    // F4 - Help with graph editing
    if keyboard.just_pressed(KeyCode::F4) {
        events.write(AgentQuestionEvent {
            question: "How do I create and edit graphs in CIM?".to_string(),
        });
    }
}
