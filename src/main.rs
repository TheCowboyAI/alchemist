mod lights;
mod camera;
mod test_wgsl;

use crate::lights::LightsPlugin;
use crate::camera::CameraPlugin;
use crate::test_wgsl::TestWgslPlugin;

use bevy::prelude::*;

// load a default shape and render it with a shader

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LightsPlugin, CameraPlugin, TestWgslPlugin))
        .run();
}


