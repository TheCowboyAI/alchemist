//! Agent Integration Plugin for Alchemist
//! 
//! This plugin integrates the CIM Alchemist AI assistant into the main application,
//! providing context-aware help about the graph editor and workflow system.

use bevy::prelude::*;
use cim_agent_alchemist::{
    AlchemistAgentPlugin,
    AgentQuestionEvent,
    AgentResponseEvent,
    AgentErrorEvent,
    ask_agent,
};
use crate::graph::GraphState;
use crate::workflow::WorkflowState;

/// Plugin that integrates the AI assistant with the graph editor
pub struct AgentIntegrationPlugin;

impl Plugin for AgentIntegrationPlugin {
    fn build(&self, app: &mut App) {
        // Add the base agent plugin
        app.add_plugins(AlchemistAgentPlugin);
        
        // Add integration systems
        app.add_systems(Update, (
            handle_context_aware_questions,
            integrate_agent_with_graph,
            integrate_agent_with_workflow,
            show_agent_help_ui,
        ));
    }
}

/// Component for the agent help panel
#[derive(Component)]
pub struct AgentHelpPanel;

#[derive(Component)]
pub struct AgentResponseArea;

/// System to handle context-aware questions based on current state
fn handle_context_aware_questions(
    keyboard: Res<ButtonInput<KeyCode>>,
    graph_state: Res<GraphState>,
    workflow_state: Res<WorkflowState>,
    mut events: EventWriter<AgentQuestionEvent>,
) {
    // F1 - Context-aware help
    if keyboard.just_pressed(KeyCode::F1) {
        let context = format!(
            "I'm working with a graph that has {} nodes and {} edges. \
             The current workflow state is {:?}. \
             What should I do next?",
            graph_state.node_count(),
            graph_state.edge_count(),
            workflow_state.current_step()
        );
        ask_agent(context, events);
    }
    
    // F2 - Explain current selection
    if keyboard.just_pressed(KeyCode::F2) {
        if let Some(selected) = graph_state.selected_node() {
            let question = format!(
                "Can you explain this node: {:?} with type {:?}?",
                selected.name,
                selected.node_type
            );
            ask_agent(question, events);
        }
    }
    
    // F3 - Workflow assistance
    if keyboard.just_pressed(KeyCode::F3) {
        let question = format!(
            "I'm at step {:?} in the workflow. What are the best practices for this step?",
            workflow_state.current_step()
        );
        ask_agent(question, events);
    }
}

/// Integrate agent responses with the graph visualization
fn integrate_agent_with_graph(
    mut response_events: EventReader<AgentResponseEvent>,
    mut graph_state: ResMut<GraphState>,
) {
    for response in response_events.read() {
        // Check if response contains graph-related suggestions
        if response.response.contains("node") || response.response.contains("edge") {
            // Add annotation to graph
            graph_state.add_annotation(format!("AI Suggestion: {}", response.response));
        }
    }
}

/// Integrate agent with workflow system
fn integrate_agent_with_workflow(
    mut response_events: EventReader<AgentResponseEvent>,
    mut workflow_state: ResMut<WorkflowState>,
) {
    for response in response_events.read() {
        // Check if response contains workflow guidance
        if response.response.contains("workflow") || response.response.contains("step") {
            workflow_state.add_ai_guidance(response.response.clone());
        }
    }
}

/// Show agent help UI
fn show_agent_help_ui(
    mut commands: Commands,
    mut response_events: EventReader<AgentResponseEvent>,
    mut error_events: EventReader<AgentErrorEvent>,
    mut query: Query<&mut Text, With<AgentResponseArea>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Toggle help panel with H key
    if keyboard.just_pressed(KeyCode::KeyH) {
        // TODO: Toggle visibility of help panel
    }
    
    // Update response area with latest agent response
    for response in response_events.read() {
        for mut text in query.iter_mut() {
            // Format response for display
            let formatted = format!(
                "🤖 Alchemist says:\n\n{}\n\n(Press H to hide)",
                response.response
            );
            text.0 = formatted;
        }
    }
    
    // Show errors
    for error in error_events.read() {
        for mut text in query.iter_mut() {
            text.0 = format!("❌ Error: {}", error.error);
        }
    }
}

/// Helper function to ask about specific graph elements
pub fn ask_about_node(
    node_id: &str,
    node_type: &str,
    mut events: EventWriter<AgentQuestionEvent>,
) {
    let question = format!(
        "What is the purpose of a {} node with id {} in CIM architecture?",
        node_type, node_id
    );
    ask_agent(question, events);
}

/// Helper function to ask about workflow steps
pub fn ask_about_workflow_step(
    step_name: &str,
    mut events: EventWriter<AgentQuestionEvent>,
) {
    let question = format!(
        "Can you provide guidance for the '{}' step in a CIM workflow?",
        step_name
    );
    ask_agent(question, events);
} 