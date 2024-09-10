use bevy::prelude::*;
use bevy::color::palettes::css::*;

use crate::camera::CameraPlugin;
use crate::entity::EntityPlugin;  
use crate::lights::LightsPlugin;  
//use crate::floor::FloorPlugin;  

pub struct SurfacePlugin;

impl Plugin for SurfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EntityPlugin,
            LightsPlugin, 
            CameraPlugin,
            FloorPlugin,
        ));
    }
}
