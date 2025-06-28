# HUD Implementation Example

## Complete Working Example

This example shows how to implement the HUD system following our architectural principles.

### 1. Component Definitions

```rust
use bevy::prelude::*;
use crate::domain::value_objects::{GraphModel, NodeId, EdgeId};

// Core HUD component that identifies which entity is a HUD element
#[derive(Component)]
pub struct HUD;

// Model Recognition HUD
#[derive(Component)]
pub struct ModelRecognitionHUD {
    pub detected_model: Option<GraphModel>,
    pub confidence: f32,
    pub last_check: f32,
}

// Statistics HUD
#[derive(Component)]
pub struct GraphStatisticsHUD {
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f32,
    pub update_interval: f32,
    pub last_update: f32,
}

// Selection HUD
#[derive(Component)]
pub struct SelectionHUD {
    pub selected_nodes: Vec<NodeId>,
    pub selected_edges: Vec<EdgeId>,
    pub subgraph_type: Option<String>,
}

// HUD visibility and positioning
#[derive(Component)]
pub struct HUDTransform {
    pub screen_position: Vec2,
    pub anchor: Anchor,
    pub opacity: f32,
    pub target_opacity: f32,
}

#[derive(Component)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}
```

### 2. HUD Plugin

```rust
pub struct HUDPlugin;

impl Plugin for HUDPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(HUDSettings::default())

            // Events (presentation layer only)
            .add_event::<HUDToggleEvent>()
            .add_event::<HUDUpdateEvent>()

            // Systems
            .add_systems(Startup, setup_hud)
            .add_systems(Update, (
                // Input handling
                handle_hud_input,

                // Update systems (throttled)
                update_model_recognition_hud
                    .run_if(time_passed(0.5)), // Every 0.5 seconds
                update_statistics_hud
                    .run_if(time_passed(0.1)), // Every 0.1 seconds
                update_selection_hud,          // Every frame when selection changes

                // Visual updates
                update_hud_visibility,
                update_hud_positions,
                render_hud_text,
            ).chain());
    }
}
```

### 3. HUD Setup System

```rust
fn setup_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraMono-Regular.ttf");

    // Model Recognition HUD
    commands.spawn((
        HUD,
        ModelRecognitionHUD {
            detected_model: None,
            confidence: 0.0,
            last_check: 0.0,
        },
        HUDTransform {
            screen_position: Vec2::new(10.0, 10.0),
            anchor: Anchor::TopLeft,
            opacity: 0.0,
            target_opacity: 1.0,
        },
        Text2dBundle {
            text: Text::from_section(
                "Analyzing...",
                TextStyle {
                    font: font.clone(),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ),
            ..default()
        },
    ));

    // Statistics HUD
    commands.spawn((
        HUD,
        GraphStatisticsHUD {
            node_count: 0,
            edge_count: 0,
            density: 0.0,
            update_interval: 0.1,
            last_update: 0.0,
        },
        HUDTransform {
            screen_position: Vec2::new(10.0, 50.0),
            anchor: Anchor::TopLeft,
            opacity: 0.0,
            target_opacity: 1.0,
        },
        Text2dBundle {
            text: Text::from_section(
                "Nodes: 0 | Edges: 0",
                TextStyle {
                    font: font.clone(),
                    font_size: 18.0,
                    color: Color::WHITE.with_a(0.8),
                },
            ),
            ..default()
        },
    ));
}
```

### 4. Update Systems

```rust
// Model recognition (expensive, throttled)
fn update_model_recognition_hud(
    mut hud_query: Query<(&mut ModelRecognitionHUD, &mut Text), With<HUD>>,
    graph_nodes: Query<&GraphNode>,
    graph_edges: Query<&GraphEdge>,
    time: Res<Time>,
) {
    for (mut hud, mut text) in hud_query.iter_mut() {
        // Build graph structure for analysis
        let node_count = graph_nodes.iter().count();
        let edge_count = graph_edges.iter().count();

        // Simple recognition logic
        let (model, confidence) = recognize_graph_model(node_count, edge_count);

        hud.detected_model = model.clone();
        hud.confidence = confidence;
        hud.last_check = time.elapsed_seconds();

        // Update display text
        if let Some(model) = model {
            text.sections[0].value = format!(
                "{} ({}% confidence)",
                model_to_string(&model),
                (confidence * 100.0) as i32
            );
            text.sections[0].style.color = match confidence {
                c if c > 0.9 => Color::GREEN,
                c if c > 0.7 => Color::YELLOW,
                _ => Color::ORANGE,
            };
        } else {
            text.sections[0].value = "Unknown Graph Type".to_string();
            text.sections[0].style.color = Color::GRAY;
        }
    }
}

// Statistics (frequent, lightweight)
fn update_statistics_hud(
    mut hud_query: Query<(&mut GraphStatisticsHUD, &mut Text), With<HUD>>,
    graph_nodes: Query<&GraphNode>,
    graph_edges: Query<&GraphEdge>,
) {
    for (mut hud, mut text) in hud_query.iter_mut() {
        let node_count = graph_nodes.iter().count();
        let edge_count = graph_edges.iter().count();

        // Calculate density
        let max_edges = if node_count > 1 {
            node_count * (node_count - 1) / 2
        } else {
            0
        };
        let density = if max_edges > 0 {
            edge_count as f32 / max_edges as f32
        } else {
            0.0
        };

        // Update if changed
        if hud.node_count != node_count || hud.edge_count != edge_count {
            hud.node_count = node_count;
            hud.edge_count = edge_count;
            hud.density = density;

            text.sections[0].value = format!(
                "Nodes: {} | Edges: {} | Density: {:.2}",
                node_count, edge_count, density
            );
        }
    }
}

// Selection context (immediate)
fn update_selection_hud(
    mut hud_query: Query<(&mut SelectionHUD, &mut Text), With<HUD>>,
    selected_nodes: Query<&NodeId, With<Selected>>,
    selected_edges: Query<&EdgeId, (With<Selected>, With<GraphEdge>)>,
) {
    for (mut hud, mut text) in hud_query.iter_mut() {
        let nodes: Vec<_> = selected_nodes.iter().cloned().collect();
        let edges: Vec<_> = selected_edges.iter().cloned().collect();

        if nodes != hud.selected_nodes || edges != hud.selected_edges {
            hud.selected_nodes = nodes.clone();
            hud.selected_edges = edges.clone();

            // Analyze selection
            let subgraph_type = analyze_subgraph(&nodes, &edges);
            hud.subgraph_type = subgraph_type.clone();

            // Update text
            if nodes.is_empty() && edges.is_empty() {
                text.sections[0].value = "No selection".to_string();
                text.sections[0].style.color = Color::GRAY;
            } else {
                let type_str = subgraph_type.unwrap_or("Subgraph".to_string());
                text.sections[0].value = format!(
                    "{} nodes, {} edges - {}",
                    nodes.len(),
                    edges.len(),
                    type_str
                );
                text.sections[0].style.color = Color::CYAN;
            }
        }
    }
}
```

### 5. Visibility and Animation

```rust
fn update_hud_visibility(
    mut hud_query: Query<&mut HUDTransform, With<HUD>>,
    camera: Query<&Transform, With<Camera3d>>,
    settings: Res<HUDSettings>,
    time: Res<Time>,
) {
    if let Ok(camera_transform) = camera.get_single() {
        let camera_distance = camera_transform.translation.length();

        for mut hud_transform in hud_query.iter_mut() {
            // Distance-based fade
            let distance_factor = (10.0 / camera_distance).clamp(0.0, 1.0);

            // Activity-based fade
            let base_opacity = if settings.enabled {
                distance_factor
            } else {
                0.0
            };

            hud_transform.target_opacity = base_opacity;

            // Smooth transition
            let delta = time.delta_seconds();
            hud_transform.opacity = hud_transform.opacity
                .lerp(hud_transform.target_opacity, delta * 5.0);
        }
    }
}

fn update_hud_positions(
    mut hud_query: Query<(&HUDTransform, &mut Transform), With<HUD>>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.get_single() {
        let width = window.width();
        let height = window.height();

        for (hud_transform, mut transform) in hud_query.iter_mut() {
            let screen_pos = match hud_transform.anchor {
                Anchor::TopLeft => hud_transform.screen_position,
                Anchor::TopRight => Vec2::new(
                    width - hud_transform.screen_position.x,
                    hud_transform.screen_position.y
                ),
                Anchor::BottomLeft => Vec2::new(
                    hud_transform.screen_position.x,
                    height - hud_transform.screen_position.y
                ),
                Anchor::BottomRight => Vec2::new(
                    width - hud_transform.screen_position.x,
                    height - hud_transform.screen_position.y
                ),
                Anchor::Center => Vec2::new(width / 2.0, height / 2.0),
            };

            // Convert screen coordinates to world coordinates
            transform.translation.x = screen_pos.x - width / 2.0;
            transform.translation.y = height / 2.0 - screen_pos.y;
            transform.translation.z = 999.0; // Always on top
        }
    }
}
```

### 6. Helper Functions

```rust
fn recognize_graph_model(node_count: usize, edge_count: usize) -> (Option<GraphModel>, f32) {
    // Check for complete graph
    let expected_complete_edges = node_count * (node_count - 1) / 2;
    if edge_count == expected_complete_edges && node_count > 0 {
        return (Some(GraphModel::Complete(node_count)), 1.0);
    }

    // Check for cycle
    if edge_count == node_count && node_count >= 3 {
        // Would need to verify it's actually a cycle
        return (Some(GraphModel::Cycle(node_count)), 0.8);
    }

    // Check for tree
    if edge_count == node_count - 1 && node_count > 0 {
        return (Some(GraphModel::Tree), 0.7);
    }

    // Unknown
    (None, 0.0)
}

fn analyze_subgraph(nodes: &[NodeId], edges: &[EdgeId]) -> Option<String> {
    match nodes.len() {
        0 => None,
        1 => Some("Single node".to_string()),
        2 => Some("Pair".to_string()),
        3 if edges.len() == 3 => Some("Triangle".to_string()),
        n if edges.len() == n * (n - 1) / 2 => Some("Clique".to_string()),
        _ => Some("Subgraph".to_string()),
    }
}

fn model_to_string(model: &GraphModel) -> String {
    match model {
        GraphModel::Complete(n) => format!("K{} - Complete Graph", n),
        GraphModel::Cycle(n) => format!("C{} - Cycle Graph", n),
        GraphModel::Tree => "Tree".to_string(),
        GraphModel::Bipartite(m, n) => format!("K({},{}) - Bipartite", m, n),
        _ => "Custom Model".to_string(),
    }
}
```

### 7. Settings and Configuration

```rust
#[derive(Resource)]
pub struct HUDSettings {
    pub enabled: bool,
    pub detail_level: DetailLevel,
    pub theme: HUDTheme,
    pub pinned_elements: HashSet<String>,
}

impl Default for HUDSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            detail_level: DetailLevel::Normal,
            theme: HUDTheme::Technical,
            pinned_elements: HashSet::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum DetailLevel {
    Minimal,
    Normal,
    Detailed,
    Expert,
}
```

## Usage Example

```rust
// In your main game loop
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HUDPlugin)
        .add_plugins(GraphVisualizationPlugin)
        .run();
}

// Toggle HUD with Tab key
fn handle_hud_input(
    keyboard: Res<Input<KeyCode>>,
    mut settings: ResMut<HUDSettings>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        settings.enabled = !settings.enabled;
    }

    if keyboard.just_pressed(KeyCode::D) {
        settings.detail_level = match settings.detail_level {
            DetailLevel::Minimal => DetailLevel::Normal,
            DetailLevel::Normal => DetailLevel::Detailed,
            DetailLevel::Detailed => DetailLevel::Expert,
            DetailLevel::Expert => DetailLevel::Minimal,
        };
    }
}
```

## Key Points

1. **No Domain Events**: The HUD only reads from ECS components and never generates domain events
2. **Performance**: Updates are throttled based on computational cost
3. **Flexibility**: Easy to add new HUD elements by creating new components
4. **User Control**: Full keyboard shortcuts and customization options
5. **Visual Polish**: Smooth animations and intelligent positioning

This implementation provides a powerful, performant HUD system that gives users deep insight into their graphs while maintaining clean architectural boundaries.
