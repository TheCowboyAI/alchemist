use bevy::prelude::*;

/// Component to mark entities that can be interacted with via UI
#[derive(Component)]
pub struct UIInteractable;

/// Component for panel anchoring
#[derive(Component)]
pub struct PanelAnchor {
    pub side: PanelSide,
    pub offset: Vec2,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PanelSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// Component for tooltip display
#[derive(Component)]
pub struct Tooltip {
    pub text: String,
    pub show_delay: f32,
    pub visible: bool,
}

/// Component for context menu
#[derive(Component)]
pub struct ContextMenu {
    pub items: Vec<MenuItem>,
    pub position: Vec2,
    pub visible: bool,
}

#[derive(Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
    pub enabled: bool,
}

#[derive(Clone)]
pub enum MenuAction {
    CreateNode,
    DeleteNode,
    CreateEdge,
    DeleteEdge,
    Custom(String),
}
