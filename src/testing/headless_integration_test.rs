#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::render::settings::{RenderCreation, WgpuSettings};
    use bevy::render::RenderPlugin;
    use bevy::window::WindowPlugin;
    use bevy::winit::WinitPlugin;
    use bevy::input::ButtonState;
    use bevy::input::mouse::MouseButtonInput;
    use std::time::Duration;

    fn setup_headless_app() -> App {
        let mut app = App::new();

        app.add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: None,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<WinitPlugin>()
                .set(WindowPlugin {
                    primary_window: None,
                    ..default()
                })
        );

        app
    }

    #[test]
    fn test_ui_interaction() {
        let mut app = setup_headless_app();

        // Add your UI components
        app.add_systems(Startup, setup_ui);
        app.add_systems(Update, button_interaction_system);
        app.insert_resource(ButtonClickState::default());

        // Simulate mouse click
        app.world_mut().send_event(CursorMoved {
            window: Entity::PLACEHOLDER,
            position: Vec2::new(100.0, 100.0),
            delta: None,
        });

        app.world_mut().send_event(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
            window: Entity::PLACEHOLDER,
        });

        // Run the app for a few frames
        for _ in 0..10 {
            app.update();
        }

        // Assert expected state changes
        let button_state = app.world().resource::<ButtonClickState>();
        assert!(button_state.was_clicked);
    }

    fn setup_ui(mut commands: Commands) {
        commands.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                ..default()
            },
            Transform::from_xyz(100.0, 100.0, 0.0),
        ));
    }

    #[derive(Resource, Default)]
    struct ButtonClickState {
        was_clicked: bool,
    }

    fn button_interaction_system(
        interaction_query: Query<&Interaction, Changed<Interaction>>,
        mut state: ResMut<ButtonClickState>,
    ) {
        for interaction in &interaction_query {
            if *interaction == Interaction::Pressed {
                state.was_clicked = true;
            }
        }
    }
}
