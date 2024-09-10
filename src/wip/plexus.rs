use bevy::prelude::*;
use bevy::color::palettes::css::*;

use crate::camera::CameraPlugin;
use crate::lights::LightsPlugin;

/// This is a Point Cloud that has a Plexus Effect
/// Our example is going to be Metatrons cube since we know what that looks like
/// and we can perform analysis on the geometry.

// limit this to something reasonable
// unless you have a huge gpu
pub const NODES: u8 = 13;
pub const NODE_RADIUS: f32 = 1.0;
pub const EDGE_RADIUS: f32 = 0.05;


pub struct PlexusPlugin;

impl Plugin for PlexusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            LightsPlugin, 
            CameraPlugin,
        ));
    }
}

// Load our points
fn spawn_points(mut commands: Commands){

}

// connect all points
fn connect_points(mut commands: Commands) {

}
