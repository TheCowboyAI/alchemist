#![allow(non_snake_case)]

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

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

impl Base16Theme {
    pub fn tokyo_night() -> Self {
        Self {
            // Tokyo Night Theme - our default
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

    pub fn gruvbox_dark() -> Self {
        Self {
            // Gruvbox Dark theme
            base00: egui::Color32::from_rgb(40, 40, 40),   // Background
            base01: egui::Color32::from_rgb(60, 56, 54),   // Lighter background
            base02: egui::Color32::from_rgb(80, 73, 69),   // Selection background
            base03: egui::Color32::from_rgb(102, 92, 84),  // Comments
            base04: egui::Color32::from_rgb(189, 174, 147),// Dark foreground
            base05: egui::Color32::from_rgb(213, 196, 161),// Default foreground
            base06: egui::Color32::from_rgb(235, 219, 178),// Light foreground
            base07: egui::Color32::from_rgb(251, 241, 199),// Light background
            base08: egui::Color32::from_rgb(251, 73, 52),  // Red
            base09: egui::Color32::from_rgb(254, 128, 25), // Orange
            base0A: egui::Color32::from_rgb(250, 189, 47), // Yellow
            base0B: egui::Color32::from_rgb(184, 187, 38), // Green
            base0C: egui::Color32::from_rgb(142, 192, 124),// Aqua
            base0D: egui::Color32::from_rgb(131, 165, 152),// Blue
            base0E: egui::Color32::from_rgb(211, 134, 155),// Purple
            base0F: egui::Color32::from_rgb(214, 93, 14),  // Brown
            name: "Gruvbox Dark".to_string(),
        }
    }

    /// Convert base16 theme colors to Bevy colors for 3D objects
    pub fn to_bevy_color(&self, base16_color: egui::Color32) -> Color {
        Color::srgb_u8(base16_color.r(), base16_color.g(), base16_color.b())
    }

    /// Get color scheme for subgraph visualization
    pub fn get_subgraph_colors(&self) -> [Color; 8] {
        [
            self.to_bevy_color(self.base08), // Red
            self.to_bevy_color(self.base0B), // Green
            self.to_bevy_color(self.base0D), // Blue
            self.to_bevy_color(self.base0A), // Yellow
            self.to_bevy_color(self.base0E), // Purple
            self.to_bevy_color(self.base0C), // Aqua
            self.to_bevy_color(self.base09), // Orange
            self.to_bevy_color(self.base0F), // Brown
        ]
    }
}

// Resource to store theme state
#[derive(Resource)]
pub struct AlchemistTheme {
    pub current_theme: Base16Theme,
    pub available_themes: Vec<Base16Theme>,
    pub selected_theme_index: usize,
    pub theme_changed: bool,
}

impl Default for AlchemistTheme {
    fn default() -> Self {
        let themes = vec![
            Base16Theme::tokyo_night(),      // Default theme as requested
            Base16Theme::dracula(),
            Base16Theme::nord(),
            Base16Theme::gruvbox_dark(),
            Base16Theme::solarized_dark(),
            Base16Theme::default_light(),
        ];
        
        Self {
            current_theme: themes[0].clone(),    // Start with Tokyo Night
            available_themes: themes,
            selected_theme_index: 0,
            theme_changed: true, // Start with true to apply theme on first frame
        }
    }
}

// Plugin for theming
pub struct ThemingPlugin;

impl Plugin for ThemingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AlchemistTheme>()
            .add_systems(Startup, setup_theme)
            .add_systems(Update, apply_theme_changes);
    }
}

// System to setup the theme on startup
fn setup_theme(mut contexts: EguiContexts, theme: Res<AlchemistTheme>) {
    apply_base16_theme(contexts.ctx_mut(), &theme.current_theme);
}

// System to apply theme changes when the theme is changed
fn apply_theme_changes(
    mut contexts: EguiContexts,
    mut theme: ResMut<AlchemistTheme>,
) {
    if theme.theme_changed {
        apply_base16_theme(contexts.ctx_mut(), &theme.current_theme);
        theme.theme_changed = false;
    }
}

// Function to apply the base16 theme to egui
pub fn apply_base16_theme(ctx: &mut egui::Context, theme: &Base16Theme) {
    let mut style = (*ctx.style()).clone();
    
    // Background colors
    style.visuals.window_fill = theme.base01;
    style.visuals.panel_fill = theme.base01;
    style.visuals.faint_bg_color = theme.base02;
    style.visuals.extreme_bg_color = theme.base00;
    
    // Text colors
    style.visuals.override_text_color = Some(theme.base05);
    
    // Interactive widgets
    style.visuals.widgets.noninteractive.bg_fill = theme.base01;
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, theme.base04);
    
    style.visuals.widgets.inactive.bg_fill = theme.base02;
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, theme.base05);
    
    style.visuals.widgets.hovered.bg_fill = theme.base0D;
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, theme.base06);
    
    style.visuals.widgets.active.bg_fill = theme.base0C;
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, theme.base07);
    
    // Selection
    style.visuals.selection.bg_fill = theme.base0D;
    style.visuals.selection.stroke = egui::Stroke::new(1.0, theme.base06);
    
    // Resize borders
    style.visuals.window_stroke = egui::Stroke::new(2.0, theme.base0C);
    
    // Rounding
    style.visuals.window_corner_radius = egui::CornerRadius::same(8);
    style.visuals.menu_corner_radius = egui::CornerRadius::same(6);
    style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
    
    // Apply the custom style
    ctx.set_style(style);
}

/// Helper function to render a theme selector UI
pub fn theme_selector_ui(ui: &mut egui::Ui, theme: &mut AlchemistTheme) {
    ui.group(|ui| {
        ui.label("🎨 Theme Settings");
        
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
                            theme.theme_changed = true;
                        }
                    }
                });
        });
        
        // Display color palette
        ui.collapsing("Color Palette", |ui| {
            let theme_colors = &theme.current_theme;
            
            ui.label("Base colors:");
            ui.horizontal(|ui| {
                ui.colored_label(theme_colors.base00, "■ bg");
                ui.colored_label(theme_colors.base01, "■ bg+");
                ui.colored_label(theme_colors.base02, "■ sel");
                ui.colored_label(theme_colors.base03, "■ com");
            });
            
            ui.horizontal(|ui| {
                ui.colored_label(theme_colors.base04, "■ fg-");
                ui.colored_label(theme_colors.base05, "■ fg");
                ui.colored_label(theme_colors.base06, "■ fg+");
                ui.colored_label(theme_colors.base07, "■ fg++");
            });
            
            ui.label("Accent colors:");
            ui.horizontal(|ui| {
                ui.colored_label(theme_colors.base08, "■ red");
                ui.colored_label(theme_colors.base09, "■ org");
                ui.colored_label(theme_colors.base0A, "■ yel");
                ui.colored_label(theme_colors.base0B, "■ grn");
            });
            
            ui.horizontal(|ui| {
                ui.colored_label(theme_colors.base0C, "■ aqu");
                ui.colored_label(theme_colors.base0D, "■ blu");
                ui.colored_label(theme_colors.base0E, "■ pur");
                ui.colored_label(theme_colors.base0F, "■ brn");
            });
        });
    });
} 