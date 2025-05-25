use bevy::prelude::*;

/// Resource to track DPI scaling
#[derive(Resource)]
pub struct DpiScaling {
    pub scale_factor: f32,
    pub base_font_size: f32,
    pub manual_override: Option<f32>, // Allow manual override of DPI scaling
}

impl Default for DpiScaling {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            base_font_size: 12.0, // Base font size before scaling
            manual_override: None,
        }
    }
}

/// Resource for viewport configuration
#[derive(Resource)]
pub struct ViewportConfig {
    pub main_viewport: ViewportRect,
    pub tools_panel_width: f32,
    pub aspect_ratio: f32,
}

impl Default for ViewportConfig {
    fn default() -> Self {
        Self {
            main_viewport: ViewportRect::default(),
            tools_panel_width: 300.0,
            aspect_ratio: 16.0 / 9.0,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct ViewportRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Resource for node counter
#[derive(Resource, Default)]
pub struct NodeCounter(pub u32);

/// Resource for edge mesh tracking
#[derive(Resource, Default)]
pub struct EdgeMeshTracker {
    pub edge_entities: std::collections::HashMap<uuid::Uuid, Entity>,
    pub initial_render_done: bool,
}

impl EdgeMeshTracker {
    pub fn track(&mut self, edge_id: uuid::Uuid, entity: Entity) {
        self.edge_entities.insert(edge_id, entity);
    }

    pub fn remove(&mut self, edge_id: &uuid::Uuid, commands: &mut bevy::prelude::Commands) {
        if let Some(entity) = self.edge_entities.remove(edge_id) {
            commands.entity(entity).despawn();
        }
    }

    pub fn despawn_all(&mut self, commands: &mut bevy::prelude::Commands) {
        for entity in self.edge_entities.values() {
            commands.entity(*entity).despawn();
        }
        self.edge_entities.clear();
    }

    pub fn has_edge(&self, edge_id: &uuid::Uuid) -> bool {
        self.edge_entities.contains_key(edge_id)
    }

    pub fn mark_initial_render_done(&mut self) {
        self.initial_render_done = true;
    }

    pub fn needs_initial_render(&self) -> bool {
        !self.initial_render_done
    }
}

/// Resource for tracking last view mode
#[derive(Resource, Default)]
pub struct LastViewMode {
    pub was_2d: bool,
    pub mode: Option<crate::camera::ViewMode>,
}
