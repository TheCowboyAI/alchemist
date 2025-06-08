use bevy::prelude::*;
use crate::presentation::systems::SystemsPlugin;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SystemsPlugin);
    }
}
