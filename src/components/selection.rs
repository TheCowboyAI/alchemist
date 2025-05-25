use bevy::prelude::*;

/// Component to mark a node as selected
#[derive(Component)]
pub struct Selected;

/// Component to mark a node as hovered
#[derive(Component)]
pub struct Hovered;

/// Component to mark a node as focused
#[derive(Component)]
pub struct Focused;

/// Component for multi-selection groups
#[derive(Component)]
pub struct SelectionGroup {
    pub group_id: u32,
}

/// Component to track selection state
#[derive(Component)]
pub struct SelectionState {
    pub selected_at: f64,
    pub selection_order: u32,
}
