use bevy::prelude::*;
use bevy::ecs::resource::Resource;
use crate::ecs::GraphSystem;
use crate::events::EventStream;
use crate::graph::AlchemistGraph;

// Define a struct for graph settings
#[derive(Clone, Debug)]
pub struct GraphSettings {
    pub node_size: f32,
    pub node_color: Color,
    pub edge_thickness: f32,
    pub edge_color: Color,
    pub show_arrows: bool,
    pub show_node_labels: bool,
    pub show_node_properties: bool,
}

impl Default for GraphSettings {
    fn default() -> Self {
        Self {
            node_size: 20.0,
            node_color: Color::srgb(0.2, 0.6, 0.9),
            edge_thickness: 2.0,
            edge_color: Color::srgba(0.5, 0.5, 0.5, 1.0),
            show_arrows: true,
            show_node_labels: true,
            show_node_properties: true,
        }
    }
}

// Define types for your application state
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ViewType {
    Graph,
    Workflow,
    ThreeD,
    Events,
}

// Application state storage
#[derive(Clone, Debug, Resource)]
pub struct AlchemistAppState {
    pub current_view: ViewType,
    pub show_immediate_viewport: bool,
    pub information_graph: AlchemistGraph,
    pub graph_system: GraphSystem,
    pub event_stream: EventStream,
    pub graph_settings: GraphSettings,
}

impl Default for AlchemistAppState {
    fn default() -> Self {
        Self {
            current_view: ViewType::Graph,
            show_immediate_viewport: false,
            information_graph: AlchemistGraph::new(),
            graph_system: GraphSystem::new(),
            event_stream: EventStream::new(),
            graph_settings: GraphSettings::default(),
        }
    }
}

impl AlchemistAppState {
    pub fn create_new_graph(&mut self) {
        self.information_graph = AlchemistGraph::new();
    }
}

// Create the app state plugin
pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AlchemistAppState>();
    }
} 