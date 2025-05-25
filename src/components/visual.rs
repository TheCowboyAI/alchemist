use bevy::prelude::*;

/// Component for visual representation of nodes
#[derive(Component)]
pub struct NodeVisual {
    pub base_color: Color,
    pub current_color: Color,
}

/// Component for edge visual properties
#[derive(Component, Debug, Clone)]
pub struct EdgeVisual {
    pub width: f32,
    pub color: Color,
}

impl Default for EdgeVisual {
    fn default() -> Self {
        Self {
            width: 2.0,
            color: Color::srgb(0.255, 0.412, 0.882), // Royal blue
        }
    }
}

/// Component for material handles
#[derive(Component)]
pub struct MaterialHandle(pub Handle<StandardMaterial>);

/// Component for mesh handles
#[derive(Component)]
pub struct MeshHandle(pub Handle<Mesh>);

/// Component for level of detail
#[derive(Component)]
pub struct LevelOfDetail {
    pub current_level: u8,
    pub max_level: u8,
}
