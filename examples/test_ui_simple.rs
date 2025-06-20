//! Simple test of the UI without agent integration

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "UI Test".to_string(),
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin,
        ))
        .init_resource::<UiState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle_window, render_ui))
        .run();
}

#[derive(Resource, Default)]
struct UiState {
    show_window: bool,
    text: String,
    messages: Vec<String>,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    println!("Press F1 to toggle the UI window");
}

fn toggle_window(keyboard: Res<ButtonInput<KeyCode>>, mut state: ResMut<UiState>) {
    if keyboard.just_pressed(KeyCode::F1) {
        state.show_window = !state.show_window;
        println!("Window toggled: {}", state.show_window);
    }
}

fn render_ui(mut contexts: EguiContexts, mut state: ResMut<UiState>) {
    if !state.show_window {
        return;
    }

    let ctx = contexts.ctx_mut();

    egui::Window::new("Test UI").show(ctx, |ui| {
        ui.label("This is a test UI!");

        ui.separator();

        // Messages
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for msg in &state.messages {
                    ui.label(msg);
                }
            });

        ui.separator();

        // Input
        ui.horizontal(|ui| {
            let response = ui.text_edit_singleline(&mut state.text);

            if ui.button("Send").clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                if !state.text.is_empty() {
                    println!("Sending: {}", state.text);
                    state.messages.push(format!("You: {}", state.text));
                    state.messages.push(format!("Bot: Echo - {}", state.text));
                    state.text.clear();
                }
            }
        });

        ui.separator();

        if ui.button("Clear").clicked() {
            state.messages.clear();
        }
    });
}
