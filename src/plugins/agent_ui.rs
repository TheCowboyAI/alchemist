//! UI components for agent interaction

use crate::simple_agent::{AgentErrorEvent, AgentQuestionEvent, AgentResponseEvent};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use tracing::info;

/// State for the agent chat UI
#[derive(Resource, Default)]
pub struct AgentChatState {
    input_text: String,
    messages: Vec<ChatMessage>,
    is_waiting: bool,
    show_window: bool,
    dock_side: DockSide,
    window_size: egui::Vec2,
}

/// Docking position for the agent UI window
#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(Default)]
pub enum DockSide {
    /// Floating window that can be moved
    Floating,
    /// Docked to the left side of the screen
    Left,
    /// Docked to the right side of the screen
    #[default]
    Right,
    /// Docked to the top of the screen
    Top,
    /// Docked to the bottom of the screen
    Bottom,
}


/// Represents a message in the chat history
#[derive(Clone)]
pub struct ChatMessage {
    /// The text content of the message
    pub text: String,
    /// Whether this message is from the user (true) or agent (false)
    pub is_user: bool,
    /// Timestamp when the message was sent
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

        app.init_resource::<AgentChatState>().add_systems(
            Update,
            (
                toggle_agent_window,
                render_agent_ui.after(toggle_agent_window),
                handle_agent_responses,
                handle_agent_errors,
            ),
        );
    }
}

fn toggle_agent_window(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut chat_state: ResMut<AgentChatState>,
) {
    // Log any key press for debugging
    for key in keyboard.get_just_pressed() {
        info!("Agent UI detected key press: {:?}", key);
    }

    if keyboard.just_pressed(KeyCode::F1) {
        info!("F1 pressed - toggling agent window");
        chat_state.show_window = !chat_state.show_window;
        info!("Agent window show state: {}", chat_state.show_window);
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

    // Debug: Log that we're rendering
    use std::sync::atomic::{AtomicU32, Ordering};
    static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);
    let frame = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
    if frame % 60 == 0 {
        // Log every second
        info!("Agent UI rendering, frame {}", frame);
    }

    // Initialize window size if not set
    if chat_state.window_size == egui::Vec2::ZERO {
        chat_state.window_size = egui::Vec2::new(400.0, 600.0);
    }

    let mut show = chat_state.show_window;

    // Create window based on dock side
    let window = match chat_state.dock_side {
        DockSide::Floating => egui::Window::new("🤖 Alchemist Assistant")
            .open(&mut show)
            .default_size(chat_state.window_size)
            .resizable(true)
            .movable(true)
            .collapsible(false),
        DockSide::Left => egui::Window::new("🤖 Alchemist Assistant")
            .open(&mut show)
            .fixed_pos(egui::Pos2::new(0.0, 0.0))
            .fixed_size([chat_state.window_size.x, ctx.screen_rect().height()])
            .resizable(false)
            .movable(false)
            .collapsible(false),
        DockSide::Right => {
            let screen_width = ctx.screen_rect().width();
            egui::Window::new("🤖 Alchemist Assistant")
                .open(&mut show)
                .fixed_pos(egui::Pos2::new(
                    screen_width - chat_state.window_size.x,
                    0.0,
                ))
                .fixed_size([chat_state.window_size.x, ctx.screen_rect().height()])
                .resizable(false)
                .movable(false)
                .collapsible(false)
        }
        DockSide::Top => egui::Window::new("🤖 Alchemist Assistant")
            .open(&mut show)
            .fixed_pos(egui::Pos2::new(0.0, 0.0))
            .fixed_size([ctx.screen_rect().width(), chat_state.window_size.y])
            .resizable(false)
            .movable(false)
            .collapsible(false),
        DockSide::Bottom => {
            let screen_height = ctx.screen_rect().height();
            egui::Window::new("🤖 Alchemist Assistant")
                .open(&mut show)
                .fixed_pos(egui::Pos2::new(
                    0.0,
                    screen_height - chat_state.window_size.y,
                ))
                .fixed_size([ctx.screen_rect().width(), chat_state.window_size.y])
                .resizable(false)
                .movable(false)
                .collapsible(false)
        }
    };

    window.show(ctx, |ui| {
        // Docking controls
        ui.horizontal(|ui| {
            ui.label("Dock:");
            if ui
                .selectable_label(chat_state.dock_side == DockSide::Floating, "Float")
                .clicked()
            {
                chat_state.dock_side = DockSide::Floating;
            }
            if ui
                .selectable_label(chat_state.dock_side == DockSide::Left, "Left")
                .clicked()
            {
                chat_state.dock_side = DockSide::Left;
            }
            if ui
                .selectable_label(chat_state.dock_side == DockSide::Right, "Right")
                .clicked()
            {
                chat_state.dock_side = DockSide::Right;
            }
            if ui
                .selectable_label(chat_state.dock_side == DockSide::Top, "Top")
                .clicked()
            {
                chat_state.dock_side = DockSide::Top;
            }
            if ui
                .selectable_label(chat_state.dock_side == DockSide::Bottom, "Bottom")
                .clicked()
            {
                chat_state.dock_side = DockSide::Bottom;
            }
        });

        // Size adjustment for docked panels
        if chat_state.dock_side == DockSide::Left || chat_state.dock_side == DockSide::Right {
            ui.horizontal(|ui| {
                ui.label("Width:");
                ui.add(egui::Slider::new(
                    &mut chat_state.window_size.x,
                    200.0..=800.0,
                ));
            });
        } else if chat_state.dock_side == DockSide::Top || chat_state.dock_side == DockSide::Bottom
        {
            ui.horizontal(|ui| {
                ui.label("Height:");
                ui.add(egui::Slider::new(
                    &mut chat_state.window_size.y,
                    200.0..=800.0,
                ));
            });
        }

        ui.separator();

        // Chat history
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 150.0)
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

            // Request focus on the text input when window is first shown
            if chat_state.show_window && chat_state.messages.is_empty() {
                response.request_focus();
            }

            let send_button = ui.button("Send");
            let should_send = send_button.clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));

            if should_send {
                info!("Send triggered! Input text: '{}'", chat_state.input_text);
                if !chat_state.input_text.is_empty() {
                    let question_text = chat_state.input_text.clone();

                    info!("Send button clicked with question: {}", question_text);

                    // Add user message
                    chat_state.messages.push(ChatMessage {
                        text: question_text.clone(),
                        is_user: true,
                        timestamp: chrono::Local::now().format("%H:%M").to_string(),
                    });

                    // Send question
                    question_events.write(AgentQuestionEvent {
                        question: question_text.clone(),
                    });
                    info!("Sent AgentQuestionEvent: {}", question_text);

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
            let button = ui.button("What is CIM?");
            if button.clicked() {
                let question = "What is CIM?".to_string();
                info!("Quick action button clicked: {}", question);

                // Add user message to chat
                chat_state.messages.push(ChatMessage {
                    text: question.clone(),
                    is_user: true,
                    timestamp: chrono::Local::now().format("%H:%M").to_string(),
                });

                question_events.write(AgentQuestionEvent {
                    question: question.clone(),
                });
                info!("Sent AgentQuestionEvent from quick action: {}", question);
                chat_state.is_waiting = true;
            }
            if ui.button("How do I create a graph?").clicked() {
                let question = "How do I create a graph in the CIM editor?".to_string();

                // Add user message to chat
                chat_state.messages.push(ChatMessage {
                    text: question.clone(),
                    is_user: true,
                    timestamp: chrono::Local::now().format("%H:%M").to_string(),
                });

                question_events.write(AgentQuestionEvent { question });
                chat_state.is_waiting = true;
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Explain domains").clicked() {
                let question = "What are the 8 CIM domains?".to_string();

                // Add user message to chat
                chat_state.messages.push(ChatMessage {
                    text: question.clone(),
                    is_user: true,
                    timestamp: chrono::Local::now().format("%H:%M").to_string(),
                });

                question_events.write(AgentQuestionEvent { question });
                chat_state.is_waiting = true;
            }
            if ui.button("Event sourcing").clicked() {
                let question = "How does event sourcing work in CIM?".to_string();

                // Add user message to chat
                chat_state.messages.push(ChatMessage {
                    text: question.clone(),
                    is_user: true,
                    timestamp: chrono::Local::now().format("%H:%M").to_string(),
                });

                question_events.write(AgentQuestionEvent { question });
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
    let response_count = response_events.len();
    if response_count > 0 {
        info!("Handling {} agent response events", response_count);
    }

    for event in response_events.read() {
        info!("Received agent response: {}", event.response);
        chat_state.messages.push(ChatMessage {
            text: event.response.clone(),
            is_user: false,
            timestamp: chrono::Local::now().format("%H:%M").to_string(),
        });
        chat_state.is_waiting = false;
        info!("Response added to chat");
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
