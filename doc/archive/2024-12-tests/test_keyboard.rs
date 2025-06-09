//! Simple keyboard test

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, keyboard_test)
        .run();
}

fn keyboard_test(keyboard: Res<ButtonInput<KeyCode>>) {
    for key in keyboard.get_just_pressed() {
        println!("Key pressed: {:?}", key);
    }

    if keyboard.just_pressed(KeyCode::KeyI) {
        println!("I key pressed!");
    }

    if keyboard.just_pressed(KeyCode::Space) {
        println!("Space key pressed!");
    }
}
