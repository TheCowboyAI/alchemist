//! UI components for agent interaction

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::simple_agent::{AgentQuestionEvent, AgentResponseEvent, AgentErrorEvent};

/// State for the agent chat UI
#[derive(Resource, Default)]
pub struct AgentChatState {
    input_text: String,
    messages: Vec<ChatMessage>,
    is_waiting: bool,
    show_window: bool,  // Add flag to control window visibility
}

#[derive(Clone)]
pub struct ChatMessage {
    pub text: String,
    pub is_user: bool,
    pub timestamp: String,
}

/// Plugin for agent UI
pub struct AgentUiPlugin;

impl Plugin for AgentUiPlugin {
    fn build(&self, app: &mut App) {
        // Only add EguiPlugin if not already added
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }

        app.init_resource::<AgentChatState>()
            .add_systems(Update, (
                toggle_agent_window,
                render_agent_ui.run_if(resource_exists_and_changed::<AgentChatState>),
                handle_agent_responses,
                handle_agent_errors,
            ));
    }
}

fn toggle_agent_window(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut chat_state: ResMut<AgentChatState>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        chat_state.show_window = !chat_state.show_window;
    }
}

fn render_agent_ui(
    mut contexts: EguiContexts,
    mut chat_state: ResMut<AgentChatState>,
    mut question_events: EventWriter<AgentQuestionEvent>,
) {
    // Only render if window should be shown
    if !chat_state.show_window {
        return;
    }

    let ctx = contexts.ctx_mut();

    let mut show = chat_state.show_window;
    
    egui::Window::new("🤖 Alchemist Assistant")
        .open(&mut show)
        .default_size([400.0, 600.0])
        .resizable(true)
        .show(ctx, |ui| {
            // Chat history
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for msg in &chat_state.messages {
                        ui.horizontal(|ui| {
                            if msg.is_user {
                                ui.label("You:");
                                ui.label(&msg.text);
                            } else {
                                ui.label("🤖:");
                                ui.label(&msg.text);
                            }
                        });
                        ui.separator();
                    }

                    if chat_state.is_waiting {
                        ui.spinner();
                        ui.label("Thinking...");
                    }
                });

            ui.separator();

            // Input area
            ui.horizontal(|ui| {
                let response = ui.text_edit_singleline(&mut chat_state.input_text);
                
                if ui.button("Send").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    if !chat_state.input_text.is_empty() {
                        let question_text = chat_state.input_text.clone();
                        
                        // Add user message
                        chat_state.messages.push(ChatMessage {
                            text: question_text.clone(),
                            is_user: true,
                            timestamp: chrono::Local::now().format("%H:%M").to_string(),
                        });

                        // Send question
                        question_events.write(AgentQuestionEvent {
                            question: question_text,
                        });

                        // Clear input and set waiting
                        chat_state.input_text.clear();
                        chat_state.is_waiting = true;
                    }
                }
            });

            ui.separator();

            // Quick actions
            ui.label("Quick Questions:");
            ui.horizontal(|ui| {
                if ui.button("What is CIM?").clicked() {
                    question_events.write(AgentQuestionEvent {
                        question: "What is CIM?".to_string(),
                    });
                    chat_state.is_waiting = true;
                }
                if ui.button("How do I create a graph?").clicked() {
                    question_events.write(AgentQuestionEvent {
                        question: "How do I create a graph in the CIM editor?".to_string(),
                    });
                    chat_state.is_waiting = true;
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Explain domains").clicked() {
                    question_events.write(AgentQuestionEvent {
                        question: "What are the 8 CIM domains?".to_string(),
                    });
                    chat_state.is_waiting = true;
                }
                if ui.button("Event sourcing").clicked() {
                    question_events.write(AgentQuestionEvent {
                        question: "How does event sourcing work in CIM?".to_string(),
                    });
                    chat_state.is_waiting = true;
                }
            });
        });
    
    // Update the show state if user closed the window
    chat_state.show_window = show;
}

fn handle_agent_responses(
    mut response_events: EventReader<AgentResponseEvent>,
    mut chat_state: ResMut<AgentChatState>,
) {
    for event in response_events.read() {
        chat_state.messages.push(ChatMessage {
            text: event.response.clone(),
            is_user: false,
            timestamp: chrono::Local::now().format("%H:%M").to_string(),
        });
        chat_state.is_waiting = false;
    }
}

fn handle_agent_errors(
    mut error_events: EventReader<AgentErrorEvent>,
    mut chat_state: ResMut<AgentChatState>,
) {
    for event in error_events.read() {
        chat_state.messages.push(ChatMessage {
            text: format!("Error: {}", event.error),
            is_user: false,
            timestamp: chrono::Local::now().format("%H:%M").to_string(),
        });
        chat_state.is_waiting = false;
    }
} 