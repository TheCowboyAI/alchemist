use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::graph_editor_3d::{GraphEditor3D, UpdateGraph3DEvent, CreatePatternEvent, UiInteractionState};
use crate::graph_patterns::PatternCategory;

// Base16 theme support
#[derive(Clone, Debug)]
pub struct Base16Theme {
    // Base16 standard colors
    pub base00: egui::Color32, // Default background
    pub base01: egui::Color32, // Lighter background
    pub base02: egui::Color32, // Selection background
    pub base03: egui::Color32, // Comments, invisibles
    pub base04: egui::Color32, // Dark foreground
    pub base05: egui::Color32, // Default foreground
    pub base06: egui::Color32, // Light foreground
    pub base07: egui::Color32, // Light background
    pub base08: egui::Color32, // Red / Error
    pub base09: egui::Color32, // Orange / Number
    pub base0A: egui::Color32, // Yellow / Key
    pub base0B: egui::Color32, // Green / String
    pub base0C: egui::Color32, // Aqua / Escape
    pub base0D: egui::Color32, // Blue / Function
    pub base0E: egui::Color32, // Purple / Keyword
    pub base0F: egui::Color32, // Brown / Deprecated
    pub name: String,
}

// Default Base16 themes
impl Base16Theme {
    pub fn default_dark() -> Self {
        Self {
            // Default "Monokai" inspired dark theme
            base00: egui::Color32::from_rgb(40, 40, 40),    // Background
            base01: egui::Color32::from_rgb(50, 50, 50),    // Lighter background
            base02: egui::Color32::from_rgb(60, 60, 60),    // Selection background
            base03: egui::Color32::from_rgb(117, 113, 94),  // Comments
            base04: egui::Color32::from_rgb(200, 200, 200), // Dark foreground
            base05: egui::Color32::from_rgb(248, 248, 242), // Default foreground
            base06: egui::Color32::from_rgb(245, 245, 245), // Light foreground
            base07: egui::Color32::from_rgb(249, 248, 245), // Light background
            base08: egui::Color32::from_rgb(249, 38, 114),  // Red
            base09: egui::Color32::from_rgb(253, 151, 31),  // Orange
            base0A: egui::Color32::from_rgb(244, 191, 117), // Yellow
            base0B: egui::Color32::from_rgb(166, 226, 46),  // Green
            base0C: egui::Color32::from_rgb(102, 217, 239), // Aqua
            base0D: egui::Color32::from_rgb(73, 166, 251),  // Blue
            base0E: egui::Color32::from_rgb(174, 129, 255), // Purple
            base0F: egui::Color32::from_rgb(204, 102, 51),  // Brown
            name: "Monokai Dark".to_string(),
        }
    }
    
    pub fn dracula() -> Self {
        Self {
            // Dracula theme colors from base16-dracula scheme
            base00: egui::Color32::from_rgb(40, 42, 54),    // Background
            base01: egui::Color32::from_rgb(58, 60, 78),    // Lighter background
            base02: egui::Color32::from_rgb(68, 71, 90),    // Selection background
            base03: egui::Color32::from_rgb(98, 104, 129),  // Comments
            base04: egui::Color32::from_rgb(186, 187, 206), // Dark foreground
            base05: egui::Color32::from_rgb(233, 233, 244), // Default foreground
            base06: egui::Color32::from_rgb(241, 242, 248), // Light foreground
            base07: egui::Color32::from_rgb(247, 247, 251), // Light background
            base08: egui::Color32::from_rgb(234, 81, 178),  // Red (pinkish in Dracula)
            base09: egui::Color32::from_rgb(180, 91, 207),  // Orange (purplish in Dracula)
            base0A: egui::Color32::from_rgb(241, 250, 140), // Yellow
            base0B: egui::Color32::from_rgb(80, 250, 123),  // Green
            base0C: egui::Color32::from_rgb(139, 233, 253), // Cyan
            base0D: egui::Color32::from_rgb(189, 147, 249), // Blue (purplish in Dracula)
            base0E: egui::Color32::from_rgb(255, 121, 198), // Purple (pinkish in Dracula)
            base0F: egui::Color32::from_rgb(255, 85, 85),   // Red (used for errors)
            name: "Dracula".to_string(),
        }
    }
    
    pub fn default_light() -> Self {
        Self {
            // Default Light theme colors from base16-default-light scheme
            base00: egui::Color32::from_rgb(248, 248, 248), // Background
            base01: egui::Color32::from_rgb(224, 224, 224), // Lighter background
            base02: egui::Color32::from_rgb(216, 216, 216), // Selection background
            base03: egui::Color32::from_rgb(184, 184, 184), // Comments
            base04: egui::Color32::from_rgb(96, 96, 96),    // Dark foreground
            base05: egui::Color32::from_rgb(56, 56, 56),    // Default foreground
            base06: egui::Color32::from_rgb(40, 40, 40),    // Light foreground
            base07: egui::Color32::from_rgb(24, 24, 24),    // Light background
            base08: egui::Color32::from_rgb(171, 70, 66),   // Red
            base09: egui::Color32::from_rgb(220, 150, 86),  // Orange
            base0A: egui::Color32::from_rgb(247, 202, 136), // Yellow
            base0B: egui::Color32::from_rgb(161, 181, 108), // Green
            base0C: egui::Color32::from_rgb(134, 193, 185), // Cyan
            base0D: egui::Color32::from_rgb(124, 175, 194), // Blue
            base0E: egui::Color32::from_rgb(186, 139, 175), // Purple
            base0F: egui::Color32::from_rgb(161, 105, 70),  // Brown
            name: "Default Light".to_string(),
        }
    }
    
    pub fn deep_blue_purple() -> Self {
        Self {
            // Deep blue and purple theme (similar to previous)
            base00: egui::Color32::from_rgb(28, 20, 45),   // Background
            base01: egui::Color32::from_rgb(48, 25, 90),   // Lighter background (DARK_PURPLE)
            base02: egui::Color32::from_rgb(59, 36, 105),  // Selection background (DEEP_PURPLE)
            base03: egui::Color32::from_rgb(90, 80, 120),  // Comments
            base04: egui::Color32::from_rgb(180, 180, 210),// Dark foreground
            base05: egui::Color32::from_rgb(220, 220, 230),// Default foreground (OFF_WHITE)
            base06: egui::Color32::from_rgb(240, 240, 245),// Light foreground (WHITE)
            base07: egui::Color32::from_rgb(245, 245, 250),// Light background
            base08: egui::Color32::from_rgb(255, 60, 120), // Red
            base09: egui::Color32::from_rgb(255, 135, 35), // Orange
            base0A: egui::Color32::from_rgb(255, 210, 70), // Yellow
            base0B: egui::Color32::from_rgb(130, 225, 90), // Green
            base0C: egui::Color32::from_rgb(65, 105, 225), // Aqua (LIGHT_COBALT)
            base0D: egui::Color32::from_rgb(0, 71, 171),   // Blue (COBALT_BLUE)
            base0E: egui::Color32::from_rgb(180, 90, 230), // Purple
            base0F: egui::Color32::from_rgb(215, 120, 70), // Brown
            name: "Deep Blue & Purple".to_string(),
        }
    }
    
    pub fn solarized_dark() -> Self {
        Self {
            // Solarized Dark
            base00: egui::Color32::from_rgb(0, 43, 54),    // Background
            base01: egui::Color32::from_rgb(7, 54, 66),    // Lighter background
            base02: egui::Color32::from_rgb(88, 110, 117), // Selection background
            base03: egui::Color32::from_rgb(101, 123, 131),// Comments
            base04: egui::Color32::from_rgb(131, 148, 150),// Dark foreground
            base05: egui::Color32::from_rgb(147, 161, 161),// Default foreground
            base06: egui::Color32::from_rgb(238, 232, 213),// Light foreground
            base07: egui::Color32::from_rgb(253, 246, 227),// Light background
            base08: egui::Color32::from_rgb(220, 50, 47),  // Red
            base09: egui::Color32::from_rgb(203, 75, 22),  // Orange
            base0A: egui::Color32::from_rgb(181, 137, 0),  // Yellow
            base0B: egui::Color32::from_rgb(133, 153, 0),  // Green
            base0C: egui::Color32::from_rgb(42, 161, 152), // Aqua
            base0D: egui::Color32::from_rgb(38, 139, 210), // Blue
            base0E: egui::Color32::from_rgb(108, 113, 196),// Purple
            base0F: egui::Color32::from_rgb(211, 54, 130), // Brown
            name: "Solarized Dark".to_string(),
        }
    }
    
    pub fn nord() -> Self {
        Self {
            // Nord Theme
            base00: egui::Color32::from_rgb(46, 52, 64),   // Background
            base01: egui::Color32::from_rgb(59, 66, 82),   // Lighter background
            base02: egui::Color32::from_rgb(67, 76, 94),   // Selection background
            base03: egui::Color32::from_rgb(76, 86, 106),  // Comments
            base04: egui::Color32::from_rgb(216, 222, 233),// Dark foreground
            base05: egui::Color32::from_rgb(229, 233, 240),// Default foreground
            base06: egui::Color32::from_rgb(236, 239, 244),// Light foreground
            base07: egui::Color32::from_rgb(242, 244, 248),// Light background
            base08: egui::Color32::from_rgb(191, 97, 106), // Red
            base09: egui::Color32::from_rgb(208, 135, 112),// Orange
            base0A: egui::Color32::from_rgb(235, 203, 139),// Yellow
            base0B: egui::Color32::from_rgb(163, 190, 140),// Green
            base0C: egui::Color32::from_rgb(143, 188, 187),// Aqua
            base0D: egui::Color32::from_rgb(94, 129, 172), // Blue
            base0E: egui::Color32::from_rgb(180, 142, 173),// Purple
            base0F: egui::Color32::from_rgb(171, 146, 147),// Brown
            name: "Nord".to_string(),
        }
    }
    
    pub fn tokyo_night() -> Self {
        Self {
            // Tokyo Night Theme
            base00: egui::Color32::from_rgb(26, 27, 38),   // Background
            base01: egui::Color32::from_rgb(36, 40, 59),   // Lighter background
            base02: egui::Color32::from_rgb(41, 46, 66),   // Selection background
            base03: egui::Color32::from_rgb(87, 92, 115),  // Comments
            base04: egui::Color32::from_rgb(169, 177, 214),// Dark foreground
            base05: egui::Color32::from_rgb(192, 202, 245),// Default foreground
            base06: egui::Color32::from_rgb(202, 211, 245),// Light foreground
            base07: egui::Color32::from_rgb(210, 220, 249),// Light background
            base08: egui::Color32::from_rgb(247, 118, 142),// Red
            base09: egui::Color32::from_rgb(255, 158, 100),// Orange
            base0A: egui::Color32::from_rgb(224, 175, 104),// Yellow
            base0B: egui::Color32::from_rgb(158, 206, 106),// Green
            base0C: egui::Color32::from_rgb(125, 207, 255),// Aqua
            base0D: egui::Color32::from_rgb(122, 162, 247),// Blue
            base0E: egui::Color32::from_rgb(187, 154, 247),// Purple
            base0F: egui::Color32::from_rgb(235, 188, 186),// Brown
            name: "Tokyo Night".to_string(),
        }
    }
}

// Resource to store theme state
#[derive(Resource)]
pub struct GraphEditorTheme {
    pub use_custom_theme: bool,
    pub current_theme: Base16Theme,
    pub available_themes: Vec<Base16Theme>,
    pub selected_theme_index: usize,
    pub use_dark_theme: bool,
}

impl Default for GraphEditorTheme {
    fn default() -> Self {
        let themes = vec![
            Base16Theme::dracula(),              // Primary dark theme (Dracula as requested)
            Base16Theme::default_light(),        // Primary light theme
            Base16Theme::deep_blue_purple(),     // Keeping the original theme
            Base16Theme::tokyo_night(),
            Base16Theme::nord(),
            Base16Theme::solarized_dark(),
        ];
        
        Self {
            use_custom_theme: true,
            current_theme: themes[0].clone(),    // Start with Dracula
            available_themes: themes,
            selected_theme_index: 0,
            use_dark_theme: true,
        }
    }
}

// Plugin for the graph editor UI
pub struct GraphEditorUiPlugin;

impl Plugin for GraphEditorUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GraphEditorTheme>()
            .add_systems(Startup, setup_custom_theme)
            .add_systems(Update, graph_editor_ui_system);
    }
}

// System to setup the custom theme on startup
fn setup_custom_theme(mut contexts: EguiContexts, mut theme: ResMut<GraphEditorTheme>) {
    theme.use_custom_theme = true;
    apply_custom_theme(contexts.ctx_mut(), &theme.current_theme);
}

// Function to apply the custom dark theme
fn apply_custom_theme(ctx: &mut egui::Context, theme_colors: &Base16Theme) {
    let mut style = (*ctx.style()).clone();
    
    // Background colors
    style.visuals.window_fill = theme_colors.base01;
    style.visuals.panel_fill = theme_colors.base01;
    style.visuals.faint_bg_color = theme_colors.base02;
    style.visuals.extreme_bg_color = theme_colors.base00;
    
    // Text colors
    style.visuals.override_text_color = Some(theme_colors.base05);
    
    // Interactive widgets
    style.visuals.widgets.noninteractive.bg_fill = theme_colors.base01;
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, theme_colors.base04);
    
    style.visuals.widgets.inactive.bg_fill = theme_colors.base02;
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, theme_colors.base05);
    
    style.visuals.widgets.hovered.bg_fill = theme_colors.base0D;
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, theme_colors.base06);
    
    style.visuals.widgets.active.bg_fill = theme_colors.base0C;
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, theme_colors.base07);
    
    // Selection
    style.visuals.selection.bg_fill = theme_colors.base0D;
    style.visuals.selection.stroke = egui::Stroke::new(1.0, theme_colors.base06);
    
    // Resize borders
    style.visuals.window_stroke = egui::Stroke::new(2.0, theme_colors.base0C);
    
    // Rounding
    style.visuals.window_corner_radius = egui::CornerRadius::same(6);
    style.visuals.menu_corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
    
    // Apply the custom style
    ctx.set_style(style);
}

// System to display the UI for the graph editor
fn graph_editor_ui_system(
    mut contexts: EguiContexts,
    mut graph_editor: ResMut<GraphEditor3D>,
    mut update_events: EventWriter<UpdateGraph3DEvent>,
    mut create_pattern_events: EventWriter<CreatePatternEvent>,
    mut ui_state: ResMut<UiInteractionState>,
    mut theme: ResMut<GraphEditorTheme>,
) {
    let ctx = contexts.ctx_mut();
    
    // Re-apply the theme if needed
    if theme.use_custom_theme {
        apply_custom_theme(ctx, &theme.current_theme);
    }
    
    // Check if mouse is over any egui element
    ui_state.mouse_over_ui = ctx.is_pointer_over_area();
    
    egui::Window::new("3D Graph Editor")
        .default_width(250.0)
        .show(ctx, |ui| {
            // Update mouse-over-UI state whenever interaction happens
            if ui.ui_contains_pointer() {
                ui_state.mouse_over_ui = true;
            }
            
            ui.heading("Graph Controls");
            
            ui.separator();
            
            if ui.button("Create Decision Workflow").clicked() {
                let mut workflow = crate::graph::GraphWorkflow::new();
                workflow.create_decision_workflow();
                graph_editor.graph = workflow.graph.clone();
                update_events.write(UpdateGraph3DEvent);
            }
            
            if ui.button("Create Example Workflow").clicked() {
                let mut workflow = crate::graph::GraphWorkflow::new();
                workflow.create_example_workflow();
                graph_editor.graph = workflow.graph.clone();
                update_events.write(UpdateGraph3DEvent);
            }
            
            ui.separator();
            ui.heading("Pattern Catalog");
            
            // Display pattern categories
            let categories = [
                PatternCategory::Basic,
                PatternCategory::Algorithmic,
                PatternCategory::Structural,
                PatternCategory::Modeling,
            ];
            
            for category in categories.iter() {
                let category_name = match category {
                    PatternCategory::Basic => "Basic",
                    PatternCategory::Algorithmic => "Algorithmic",
                    PatternCategory::Structural => "Structural",
                    PatternCategory::Modeling => "Modeling",
                };
                
                ui.collapsing(category_name, |ui| {
                    let keys = graph_editor.pattern_catalog.get_keys_by_category(*category);
                    
                    for &key in keys.iter() {
                        if let Some(pattern) = graph_editor.pattern_catalog.get_pattern(key) {
                            let pattern_name = pattern.name();
                            let response = ui.button(pattern_name);
                            
                            if response.clicked() {
                                create_pattern_events.write(CreatePatternEvent {
                                    pattern: pattern.clone(),
                                });
                            }
                            
                            response.on_hover_text(pattern.description());
                        }
                    }
                });
            }
            
            ui.separator();
            
            // Node information panel
            ui.heading("Node Information");
            
            if let Some(selected_id) = graph_editor.selected_node {
                if let Some(node) = graph_editor.graph.get_node(selected_id) {
                    ui.label(format!("Name: {}", node.name));
                    
                    ui.label("Labels:");
                    for label in &node.labels {
                        ui.label(format!("- {}", label));
                    }
                    
                    ui.label("Properties:");
                    for (key, value) in &node.properties {
                        ui.label(format!("{}: {}", key, value));
                    }
                }
            } else {
                ui.label("No node selected");
            }
            
            // Add theme settings
            ui.add_space(20.0);
            ui.separator();
            ui.checkbox(&mut theme.use_custom_theme, "Use Custom Theme");
            
            if theme.use_custom_theme {
                let theme_names: Vec<String> = theme.available_themes.iter()
                    .map(|t| t.name.clone())
                    .collect();
                
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    egui::ComboBox::new("theme_selector", "")
                        .selected_text(&theme_names[theme.selected_theme_index])
                        .show_ui(ui, |ui| {
                            for (idx, name) in theme_names.iter().enumerate() {
                                if ui.selectable_label(theme.selected_theme_index == idx, name).clicked() {
                                    theme.selected_theme_index = idx;
                                    theme.current_theme = theme.available_themes[idx].clone();
                                    // Update dark mode flag based on theme name (simple heuristic)
                                    theme.use_dark_theme = !name.contains("Light");
                                }
                            }
                        });
                });
                
                // Add a light/dark toggle
                if ui.checkbox(&mut theme.use_dark_theme, "Dark Mode").clicked() {
                    // Automatically switch between Dracula (dark) and Default Light
                    if theme.use_dark_theme {
                        // Find Dracula theme
                        if let Some(position) = theme.available_themes.iter().position(|t| t.name == "Dracula") {
                            theme.selected_theme_index = position;
                            theme.current_theme = theme.available_themes[position].clone();
                        }
                    } else {
                        // Find Default Light theme
                        if let Some(position) = theme.available_themes.iter().position(|t| t.name == "Default Light") {
                            theme.selected_theme_index = position;
                            theme.current_theme = theme.available_themes[position].clone();
                        }
                    }
                }
                
                // Display color palette
                ui.collapsing("Color Palette", |ui| {
                    let theme_colors = &theme.current_theme;
                    
                    // Base colors
                    ui.horizontal(|ui| {
                        ui.colored_label(theme_colors.base00, "■");
                        ui.colored_label(theme_colors.base01, "■");
                        ui.colored_label(theme_colors.base02, "■");
                        ui.colored_label(theme_colors.base03, "■");
                    });
                    
                    ui.horizontal(|ui| {
                        ui.colored_label(theme_colors.base04, "■");
                        ui.colored_label(theme_colors.base05, "■");
                        ui.colored_label(theme_colors.base06, "■");
                        ui.colored_label(theme_colors.base07, "■");
                    });
                    
                    // Accent colors
                    ui.horizontal(|ui| {
                        ui.colored_label(theme_colors.base08, "■");
                        ui.colored_label(theme_colors.base09, "■");
                        ui.colored_label(theme_colors.base0A, "■");
                        ui.colored_label(theme_colors.base0B, "■");
                    });
                    
                    ui.horizontal(|ui| {
                        ui.colored_label(theme_colors.base0C, "■");
                        ui.colored_label(theme_colors.base0D, "■");
                        ui.colored_label(theme_colors.base0E, "■");
                        ui.colored_label(theme_colors.base0F, "■");
                    });
                });
            }
        });
} 