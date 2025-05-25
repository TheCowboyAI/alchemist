use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Central manager for all UI panels
#[derive(Resource, Clone, Debug)]
pub struct PanelManager {
    pub current_workspace: WorkspaceMode,
    pub panels: PanelStates,
    pub show_panel_config: bool,
    pub last_toggle_time: Instant,
    pub toggle_debounce_duration: Duration,
}

/// Different workspace modes for different user needs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WorkspaceMode {
    Minimal,      // Just the graph view - for beginners
    Standard,     // Control panel + inspector - balanced
    Advanced,     // All panels available - power users
    DDD,          // DDD-focused layout
    ECS,          // ECS-focused layout
    Algorithms,   // Algorithm analysis focused
    Custom,       // User-defined layout
}

/// States for all available panels
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PanelStates {
    // Core panels
    pub control_panel: PanelState,
    pub inspector_panel: PanelState,

    // Specialized panels
    pub properties_panel: PanelState,
    pub algorithms_panel: PanelState,
    pub ddd_panel: PanelState,
    pub ecs_panel: PanelState,
    pub console_panel: PanelState,
    pub minimap_panel: PanelState,

    // Tool panels
    pub search_panel: PanelState,
    pub history_panel: PanelState,
    pub bookmarks_panel: PanelState,
}

/// Individual panel state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PanelState {
    pub visible: bool,
    pub width: f32,
    pub height: f32,
    pub position: PanelPosition,
    pub docked: bool,
    pub minimized: bool,
}

/// Panel positioning
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PanelPosition {
    Left,
    Right,
    Top,
    Bottom,
    Floating { x: f32, y: f32 },
}

impl Default for PanelManager {
    fn default() -> Self {
        Self {
            current_workspace: WorkspaceMode::Standard,
            panels: PanelStates::default(),
            show_panel_config: false,
            last_toggle_time: Instant::now(),
            toggle_debounce_duration: Duration::from_millis(100), // 100ms debounce
        }
    }
}

impl Default for PanelStates {
    fn default() -> Self {
        Self {
            control_panel: PanelState {
                visible: true,
                width: 300.0,
                height: 0.0, // Auto height for side panels
                position: PanelPosition::Left,
                docked: true,
                minimized: false,
            },
            inspector_panel: PanelState {
                visible: true,
                width: 300.0,
                height: 0.0,
                position: PanelPosition::Right,
                docked: true,
                minimized: false,
            },
            properties_panel: PanelState {
                visible: false,
                width: 250.0,
                height: 200.0,
                position: PanelPosition::Floating { x: 100.0, y: 100.0 },
                docked: false,
                minimized: false,
            },
            algorithms_panel: PanelState {
                visible: false,
                width: 350.0,
                height: 400.0,
                position: PanelPosition::Floating { x: 200.0, y: 150.0 },
                docked: false,
                minimized: false,
            },
            ddd_panel: PanelState {
                visible: false,
                width: 300.0,
                height: 350.0,
                position: PanelPosition::Floating { x: 150.0, y: 100.0 },
                docked: false,
                minimized: false,
            },
            ecs_panel: PanelState {
                visible: false,
                width: 300.0,
                height: 350.0,
                position: PanelPosition::Floating { x: 250.0, y: 120.0 },
                docked: false,
                minimized: false,
            },
            console_panel: PanelState {
                visible: false,
                width: 0.0, // Full width for bottom panel
                height: 150.0,
                position: PanelPosition::Bottom,
                docked: true,
                minimized: false,
            },
            minimap_panel: PanelState {
                visible: false,
                width: 200.0,
                height: 200.0,
                position: PanelPosition::Floating { x: 50.0, y: 50.0 },
                docked: false,
                minimized: false,
            },
            search_panel: PanelState {
                visible: false,
                width: 300.0,
                height: 250.0,
                position: PanelPosition::Floating { x: 300.0, y: 200.0 },
                docked: false,
                minimized: false,
            },
            history_panel: PanelState {
                visible: false,
                width: 250.0,
                height: 300.0,
                position: PanelPosition::Floating { x: 350.0, y: 180.0 },
                docked: false,
                minimized: false,
            },
            bookmarks_panel: PanelState {
                visible: false,
                width: 200.0,
                height: 250.0,
                position: PanelPosition::Floating { x: 400.0, y: 160.0 },
                docked: false,
                minimized: false,
            },
        }
    }
}

impl PanelManager {
    /// Switch to a different workspace mode
    pub fn set_workspace(&mut self, mode: WorkspaceMode) {
        let now = Instant::now();
        if now.duration_since(self.last_toggle_time) < self.toggle_debounce_duration {
            // Ignore rapid workspace changes
            return;
        }
        self.last_toggle_time = now;

        self.current_workspace = mode.clone();
        self.panels = self.get_workspace_layout(&mode);
    }

    /// Get the panel layout for a specific workspace
    pub fn get_workspace_layout(&self, mode: &WorkspaceMode) -> PanelStates {
        match mode {
            WorkspaceMode::Minimal => {
                let mut panels = PanelStates::default();
                // Hide all panels for minimal distraction
                panels.control_panel.visible = false;
                panels.inspector_panel.visible = false;
                panels
            },
            WorkspaceMode::Standard => {
                let mut panels = PanelStates::default();
                // Show only essential panels
                panels.control_panel.visible = true;
                panels.inspector_panel.visible = true;
                panels
            },
            WorkspaceMode::Advanced => {
                let mut panels = PanelStates::default();
                // Show most panels for power users
                panels.control_panel.visible = true;
                panels.inspector_panel.visible = true;
                panels.properties_panel.visible = true;
                panels.console_panel.visible = true;
                panels.minimap_panel.visible = true;
                panels
            },
            WorkspaceMode::DDD => {
                let mut panels = PanelStates::default();
                panels.control_panel.visible = true;
                panels.inspector_panel.visible = true;
                panels.ddd_panel.visible = true;
                panels.properties_panel.visible = true;
                panels
            },
            WorkspaceMode::ECS => {
                let mut panels = PanelStates::default();
                panels.control_panel.visible = true;
                panels.inspector_panel.visible = true;
                panels.ecs_panel.visible = true;
                panels.properties_panel.visible = true;
                panels
            },
            WorkspaceMode::Algorithms => {
                let mut panels = PanelStates::default();
                panels.control_panel.visible = true;
                panels.inspector_panel.visible = true;
                panels.algorithms_panel.visible = true;
                panels.console_panel.visible = true;
                panels
            },
            WorkspaceMode::Custom => {
                // Keep current panel states for custom layout
                self.panels.clone()
            },
        }
    }

    /// Toggle a specific panel
    pub fn toggle_panel(&mut self, panel_name: &str) {
        let now = Instant::now();
        if now.duration_since(self.last_toggle_time) < self.toggle_debounce_duration {
            // Ignore rapid toggle attempts
            return;
        }
        self.last_toggle_time = now;

        match panel_name {
            "control" => self.panels.control_panel.visible = !self.panels.control_panel.visible,
            "inspector" => self.panels.inspector_panel.visible = !self.panels.inspector_panel.visible,
            "properties" => self.panels.properties_panel.visible = !self.panels.properties_panel.visible,
            "algorithms" => self.panels.algorithms_panel.visible = !self.panels.algorithms_panel.visible,
            "ddd" => self.panels.ddd_panel.visible = !self.panels.ddd_panel.visible,
            "ecs" => self.panels.ecs_panel.visible = !self.panels.ecs_panel.visible,
            "console" => self.panels.console_panel.visible = !self.panels.console_panel.visible,
            "minimap" => self.panels.minimap_panel.visible = !self.panels.minimap_panel.visible,
            "search" => self.panels.search_panel.visible = !self.panels.search_panel.visible,
            "history" => self.panels.history_panel.visible = !self.panels.history_panel.visible,
            "bookmarks" => self.panels.bookmarks_panel.visible = !self.panels.bookmarks_panel.visible,
            _ => warn!("Unknown panel: {}", panel_name),
        }
    }

    /// Get visible panel count for UI feedback
    pub fn visible_panel_count(&self) -> usize {
        let mut count = 0;
        if self.panels.control_panel.visible { count += 1; }
        if self.panels.inspector_panel.visible { count += 1; }
        if self.panels.properties_panel.visible { count += 1; }
        if self.panels.algorithms_panel.visible { count += 1; }
        if self.panels.ddd_panel.visible { count += 1; }
        if self.panels.ecs_panel.visible { count += 1; }
        if self.panels.console_panel.visible { count += 1; }
        if self.panels.minimap_panel.visible { count += 1; }
        if self.panels.search_panel.visible { count += 1; }
        if self.panels.history_panel.visible { count += 1; }
        if self.panels.bookmarks_panel.visible { count += 1; }
        count
    }
}
