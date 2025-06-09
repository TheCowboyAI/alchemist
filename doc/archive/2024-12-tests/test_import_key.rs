//! Test import key press

use bevy::prelude::*;
use ia::application::CommandEvent;
use ia::presentation::plugins::GraphEditorPlugin;
use ia::domain::commands::{Command, GraphCommand, ImportSource, ImportOptions};
use ia::domain::commands::graph_commands::MergeBehavior;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphEditorPlugin)
        .add_event::<CommandEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, simulate_key_press)
        .run();
}

fn setup() {
    println!("Test import key press app started");
    println!("Will simulate pressing 'I' after 2 seconds...");
}

fn simulate_key_press(
    time: Res<Time>,
    mut simulated: Local<bool>,
    mut keyboard_events: EventWriter<bevy::input::keyboard::KeyboardInput>,
) {
    if !*simulated && time.elapsed_secs() > 2.0 {
        println!("Simulating 'I' key press...");

        // Simulate key press
        keyboard_events.write(bevy::input::keyboard::KeyboardInput {
            key_code: KeyCode::KeyI,
            logical_key: bevy::input::keyboard::Key::Character("i".into()),
            state: bevy::input::ButtonState::Pressed,
            window: Entity::PLACEHOLDER,
            repeat: false,
            text: None,
        });

        // Simulate key release
        keyboard_events.write(bevy::input::keyboard::KeyboardInput {
            key_code: KeyCode::KeyI,
            logical_key: bevy::input::keyboard::Key::Character("i".into()),
            state: bevy::input::ButtonState::Released,
            window: Entity::PLACEHOLDER,
            repeat: false,
            text: None,
        });

        *simulated = true;
        println!("Key press simulated!");
    }
}
