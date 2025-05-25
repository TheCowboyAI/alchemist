use bevy::prelude::*;

/// Resource for control panel state
#[derive(Resource, Default)]
pub struct ControlPanelState {
    pub visible: bool,
    pub expanded: bool,
    pub selected_tab: usize,
}

/// Resource for inspector panel state
#[derive(Resource, Default)]
pub struct InspectorPanelState {
    pub visible: bool,
    pub expanded: bool,
    pub show_properties: bool,
    pub show_hierarchy: bool,
}

/// Resource for panel manager
#[derive(Resource, Clone, Debug)]
pub struct PanelManager {
    pub left_panel_visible: bool,
    pub right_panel_visible: bool,
    pub bottom_panel_visible: bool,
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub bottom_panel_height: f32,
}

impl Default for PanelManager {
    fn default() -> Self {
        Self {
            left_panel_visible: true,
            right_panel_visible: true,
            bottom_panel_visible: false,
            left_panel_width: 300.0,
            right_panel_width: 250.0,
            bottom_panel_height: 200.0,
        }
    }
}

/// Resource for dashboard state
#[derive(Resource, Default)]
pub struct DashboardState {
    pub standard_graph_editor_active: bool,
    pub workflow_editor_active: bool,
    pub ddd_editor_active: bool,
    pub ecs_editor_active: bool,
}

/// Resource for UI interaction state
#[derive(Resource, Default)]
pub struct UiInteractionState {
    pub mouse_over_ui: bool,
}
