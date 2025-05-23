use bevy::ecs::component::Component;
use bevy::ecs::resource::Resource;
use bevy::prelude::*;
use bevy_egui::egui;
use egui::{Color32, ViewportId};
use std::collections::HashMap;

// Components and resources
#[derive(Component, Debug)]
pub struct ViewportScene {
    pub viewport_id: ViewportId,
}

#[derive(Component, Debug)]
pub struct ViewportCamera;

// Viewport state management
#[derive(Debug, Resource, Default)]
pub struct ViewportState {
    pub viewports: HashMap<ViewportId, ViewportData>,
    pub main_viewport_id: ViewportId,
}

pub struct ViewportData {
    pub title: String,
    pub entity: Option<Entity>,
    pub is_open: bool,
}

pub struct ViewportPlugin;

impl Plugin for ViewportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewportState>()
            .add_systems(Update, handle_viewports);
    }
}

// Create a new viewport with a 3D view
pub fn create_new_viewport(viewport_state: &mut ResMut<ViewportState>) {
    let viewport_id =
        ViewportId::from_hash_of(format!("viewport_{}", viewport_state.viewports.len()));

    let count = viewport_state.viewports.len();
    viewport_state.viewports.insert(
        viewport_id,
        ViewportData {
            title: format!("3D View {}", count + 1),
            entity: None,
            is_open: true,
        },
    );
}

// Update all immediate viewports
pub fn update_immediate_viewports(ctx: &egui::Context, viewport_state: &mut ResMut<ViewportState>) {
    let viewport_ids: Vec<ViewportId> = viewport_state.viewports.keys().cloned().collect();

    for viewport_id in viewport_ids {
        let viewport_data = viewport_state.viewports.get(&viewport_id).unwrap();

        if viewport_data.is_open {
            // Show the viewport using egui's immediate viewport
            ctx.show_viewport_immediate(
                viewport_id,
                egui::ViewportBuilder::default()
                    .with_title(&viewport_data.title)
                    .with_inner_size([600.0, 400.0]),
                |ctx, class| {
                    if class != egui::ViewportClass::Immediate {
                        return;
                    }

                    egui::CentralPanel::default().show(ctx, |ui| {
                        // Reserve space for the 3D scene and get the screen rect
                        let available_size = ui.available_size();
                        let (rect, _) =
                            ui.allocate_exact_size(available_size, egui::Sense::hover());

                        // Store the rect information for the viewport (we'll use this for rendering)
                        if let Some(data) = viewport_state.viewports.get_mut(&viewport_id) {
                            // Use this rect region for rendering the 3D scene
                            // We'll use bevy_egui to render to this area
                            let _id = ui.id().with("3d_view");
                            ui.painter()
                                .rect_filled(rect, 0.0, Color32::from_black_alpha(200));

                            // Add a simple UI overlay in the bottom right
                            ui.allocate_ui_at_rect(rect, |ui| {
                                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                                    ui.add_space(4.0);
                                    if ui.button("Close").clicked() {
                                        if let Some(data) =
                                            viewport_state.viewports.get_mut(&viewport_id)
                                        {
                                            data.is_open = false;
                                        }
                                    }
                                    ui.add_space(4.0);
                                    ui.label("3D Viewport");
                                });
                            });
                        }
                    });

                    // Check if the viewport is requested to be closed
                    if ctx.input(|i| i.viewport().close_requested()) {
                        if let Some(data) = viewport_state.viewports.get_mut(&viewport_id) {
                            data.is_open = false;
                        }
                    }
                },
            );
        }
    }
}

// Handle viewport entities in Bevy
pub fn handle_viewports(
    mut commands: Commands,
    query: Query<(Entity, &ViewportScene)>,
    mut viewport_state: ResMut<ViewportState>,
) {
    // Clean up closed viewports
    for (entity, viewport_scene) in query.iter() {
        if let Some(data) = viewport_state.viewports.get(&viewport_scene.viewport_id) {
            if !data.is_open {
                commands.entity(entity).despawn();

                // Update the viewport state
                if let Some(data) = viewport_state
                    .viewports
                    .get_mut(&viewport_scene.viewport_id)
                {
                    data.entity = None;
                }
            }
        }
    }

    // Create entities for new viewports
    for (viewport_id, data) in viewport_state.viewports.iter_mut() {
        if data.is_open && data.entity.is_none() {
            // Create a viewport entity
            let entity = commands
                .spawn((
                    ViewportScene {
                        viewport_id: *viewport_id,
                    },
                    Transform::default(),
                    GlobalTransform::default(),
                ))
                .id();

            // Store the entity for reference
            data.entity = Some(entity);
        }
    }
}

// Setup for the main scene
pub fn setup_main_scene(mut commands: Commands, mut viewport_state: ResMut<ViewportState>) {
    // Initialize the main viewport ID
    viewport_state.main_viewport_id = ViewportId::default();
    viewport_state.viewports = HashMap::new();

    // Set up a camera for the main scene
    commands.spawn((
        Camera3d { ..default() },
        Camera { ..default() },
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        ViewportCamera,
    ));

    // Add a light
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        GlobalTransform::default(),
    ));
}
