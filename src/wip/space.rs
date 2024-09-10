use bevy::prelude::*;
use bevy::color::palettes::css::*;

use crate::camera::CameraPlugin;
use crate::entity::EntityPlugin;  
use crate::lights::LightsPlugin;  

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EntityPlugin,
            LightsPlugin, 
            CameraPlugin,
        ));
    }
}
