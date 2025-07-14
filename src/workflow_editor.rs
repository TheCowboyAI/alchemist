//! Visual workflow editor with drag-and-drop functionality
//!
//! This module provides a visual interface for creating and editing
//! CIM workflows using a node-based graph editor.

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment, Color, Point, Vector, Size, Rectangle};
use iced::widget::{column, container, row, text, button, text_input, scrollable, Space, Canvas};
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke, Text as CanvasText};
use iced::mouse;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const GRID_SIZE: f32 = 20.0;
const NODE_WIDTH: f32 = 200.0;
const NODE_HEIGHT: f32 = 100.0;
const PORT_RADIUS: f32 = 6.0;

#[derive(Debug, Clone)]
pub enum Message {
    // Canvas interactions
    CanvasEvent(CanvasMessage),
    
    // Node operations
    AddNode(NodeType),
    DeleteNode(String),
    SelectNode(Option<String>),
    
    // Connection operations
    StartConnection(String, PortType),
    CompleteConnection(String, PortType),
    DeleteConnection(String),
    
    // Property editing
    NodePropertyChanged(String, String, String),
    
    // Workflow operations
    SaveWorkflow,
    LoadWorkflow,
    ExportYaml,
    RunWorkflow,
    
    // UI actions
    ToggleSidebar,
    ZoomIn,
    ZoomOut,
    ResetView,
    Close,
}

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    MousePressed(Point),
    MouseReleased(Point),
    MouseMoved(Point),
    MouseWheel(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    Start,
    Action,
    Condition,
    Loop,
    Parallel,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortType {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub node_type: NodeType,
    pub name: String,
    #[serde(skip)]
    pub position: Point,
    #[serde(rename = "position")]
    pub position_data: (f32, f32),
    pub properties: HashMap<String, String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    pub nodes: HashMap<String, WorkflowNode>,
    pub connections: Vec<Connection>,
}

pub struct WorkflowEditor {
    workflow: Workflow,
    selected_node: Option<String>,
    canvas_state: CanvasState,
    zoom: f32,
    pan_offset: Vector,
    show_sidebar: bool,
    node_templates: HashMap<NodeType, NodeTemplate>,
    cache: Cache,
}

struct CanvasState {
    is_dragging: bool,
    is_panning: bool,
    drag_start: Point,
    connection_start: Option<(String, PortType)>,
    mouse_position: Point,
}

struct NodeTemplate {
    name: &'static str,
    default_properties: Vec<(&'static str, &'static str)>,
    input_ports: Vec<&'static str>,
    output_ports: Vec<&'static str>,
    color: Color,
}

impl WorkflowEditor {
    pub fn new() -> (Self, Task<Message>) {
        let mut node_templates = HashMap::new();
        
        node_templates.insert(NodeType::Start, NodeTemplate {
            name: "Start",
            default_properties: vec![],
            input_ports: vec![],
            output_ports: vec!["out"],
            color: Color::from_rgb(0.2, 0.8, 0.2),
        });
        
        node_templates.insert(NodeType::Action, NodeTemplate {
            name: "Action",
            default_properties: vec![
                ("command", ""),
                ("timeout", "30"),
            ],
            input_ports: vec!["in"],
            output_ports: vec!["out", "error"],
            color: Color::from_rgb(0.2, 0.6, 1.0),
        });
        
        node_templates.insert(NodeType::Condition, NodeTemplate {
            name: "Condition",
            default_properties: vec![
                ("expression", ""),
            ],
            input_ports: vec!["in"],
            output_ports: vec!["true", "false"],
            color: Color::from_rgb(1.0, 0.8, 0.2),
        });
        
        node_templates.insert(NodeType::Loop, NodeTemplate {
            name: "Loop",
            default_properties: vec![
                ("items", ""),
                ("variable", "item"),
            ],
            input_ports: vec!["in"],
            output_ports: vec!["loop", "done"],
            color: Color::from_rgb(0.8, 0.2, 0.8),
        });
        
        node_templates.insert(NodeType::Parallel, NodeTemplate {
            name: "Parallel",
            default_properties: vec![
                ("max_concurrent", "5"),
            ],
            input_ports: vec!["in"],
            output_ports: vec!["branch1", "branch2", "branch3"],
            color: Color::from_rgb(0.8, 0.4, 0.2),
        });
        
        node_templates.insert(NodeType::End, NodeTemplate {
            name: "End",
            default_properties: vec![
                ("status", "success"),
            ],
            input_ports: vec!["in"],
            output_ports: vec![],
            color: Color::from_rgb(0.8, 0.2, 0.2),
        });
        
        // Create demo workflow
        let mut workflow = Workflow {
            name: "Example Workflow".to_string(),
            description: "A sample workflow demonstrating the editor".to_string(),
            nodes: HashMap::new(),
            connections: Vec::new(),
        };
        
        // Add start node
        let start_pos = Point::new(100.0, 200.0);
        let start_node = WorkflowNode {
            id: Uuid::new_v4().to_string(),
            node_type: NodeType::Start,
            name: "Start".to_string(),
            position: start_pos,
            position_data: (start_pos.x, start_pos.y),
            properties: HashMap::new(),
            inputs: vec![],
            outputs: vec!["out".to_string()],
        };
        workflow.nodes.insert(start_node.id.clone(), start_node);
        
        (
            WorkflowEditor {
                workflow,
                selected_node: None,
                canvas_state: CanvasState {
                    is_dragging: false,
                    is_panning: false,
                    drag_start: Point::ORIGIN,
                    connection_start: None,
                    mouse_position: Point::ORIGIN,
                },
                zoom: 1.0,
                pan_offset: Vector::new(0.0, 0.0),
                show_sidebar: true,
                node_templates,
                cache: Cache::default(),
            },
            Task::none()
        )
    }
    
    pub fn title(&self) -> String {
        format!("Workflow Editor - {}", self.workflow.name)
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CanvasEvent(canvas_msg) => {
                self.handle_canvas_message(canvas_msg);
                self.cache.clear();
                Task::none()
            }
            
            Message::AddNode(node_type) => {
                self.add_node(node_type);
                self.cache.clear();
                Task::none()
            }
            
            Message::DeleteNode(node_id) => {
                self.delete_node(&node_id);
                self.cache.clear();
                Task::none()
            }
            
            Message::SelectNode(node_id) => {
                self.selected_node = node_id;
                Task::none()
            }
            
            Message::NodePropertyChanged(node_id, key, value) => {
                if let Some(node) = self.workflow.nodes.get_mut(&node_id) {
                    node.properties.insert(key, value);
                }
                Task::none()
            }
            
            Message::SaveWorkflow => {
                self.save_workflow();
                Task::none()
            }
            
            Message::ExportYaml => {
                self.export_yaml();
                Task::none()
            }
            
            Message::ToggleSidebar => {
                self.show_sidebar = !self.show_sidebar;
                Task::none()
            }
            
            Message::ZoomIn => {
                self.zoom = (self.zoom * 1.2).min(3.0);
                self.cache.clear();
                Task::none()
            }
            
            Message::ZoomOut => {
                self.zoom = (self.zoom / 1.2).max(0.3);
                self.cache.clear();
                Task::none()
            }
            
            Message::ResetView => {
                self.zoom = 1.0;
                self.pan_offset = Vector::new(0.0, 0.0);
                self.cache.clear();
                Task::none()
            }
            
            Message::Close => {
                std::process::exit(0);
            }
            
            _ => Task::none(),
        }
    }
    
    fn handle_canvas_message(&mut self, message: CanvasMessage) {
        match message {
            CanvasMessage::MousePressed(position) => {
                let world_pos = self.screen_to_world(position);
                
                // Check if clicking on a node
                if let Some((node_id, _)) = self.find_node_at(world_pos) {
                    self.selected_node = Some(node_id.clone());
                    self.canvas_state.is_dragging = true;
                    self.canvas_state.drag_start = world_pos;
                } else {
                    // Start panning
                    self.canvas_state.is_panning = true;
                    self.canvas_state.drag_start = position;
                }
            }
            
            CanvasMessage::MouseReleased(_) => {
                self.canvas_state.is_dragging = false;
                self.canvas_state.is_panning = false;
                self.canvas_state.connection_start = None;
            }
            
            CanvasMessage::MouseMoved(position) => {
                self.canvas_state.mouse_position = position;
                
                if self.canvas_state.is_dragging {
                    if let Some(node_id) = &self.selected_node {
                        let world_pos = self.screen_to_world(position);
                        let delta = world_pos - self.canvas_state.drag_start;
                        
                        if let Some(node) = self.workflow.nodes.get_mut(node_id) {
                            node.position = node.position + delta;
                            node.position_data = (node.position.x, node.position.y);
                        }
                        
                        self.canvas_state.drag_start = world_pos;
                    }
                } else if self.canvas_state.is_panning {
                    let delta = position - self.canvas_state.drag_start;
                    self.pan_offset = self.pan_offset + delta;
                    self.canvas_state.drag_start = position;
                }
            }
            
            CanvasMessage::MouseWheel(delta) => {
                let old_zoom = self.zoom;
                self.zoom = (self.zoom * (1.0 + delta * 0.001)).clamp(0.3, 3.0);
                
                // Zoom towards mouse position
                let mouse_world_before = self.screen_to_world(self.canvas_state.mouse_position);
                let mouse_world_after = Point::new(
                    (self.canvas_state.mouse_position.x - self.pan_offset.x) / self.zoom,
                    (self.canvas_state.mouse_position.y - self.pan_offset.y) / self.zoom,
                );
                
                let correction = mouse_world_before - mouse_world_after;
                self.pan_offset = self.pan_offset + Vector::new(
                    correction.x * self.zoom,
                    correction.y * self.zoom,
                );
            }
        }
    }
    
    fn screen_to_world(&self, screen_pos: Point) -> Point {
        Point::new(
            (screen_pos.x - self.pan_offset.x) / self.zoom,
            (screen_pos.y - self.pan_offset.y) / self.zoom,
        )
    }
    
    fn world_to_screen(&self, world_pos: Point) -> Point {
        Point::new(
            world_pos.x * self.zoom + self.pan_offset.x,
            world_pos.y * self.zoom + self.pan_offset.y,
        )
    }
    
    fn find_node_at(&self, position: Point) -> Option<(String, &WorkflowNode)> {
        for (id, node) in &self.workflow.nodes {
            let node_rect = Rectangle::new(
                node.position,
                Size::new(NODE_WIDTH, NODE_HEIGHT),
            );
            
            if node_rect.contains(position) {
                return Some((id.clone(), node));
            }
        }
        None
    }
    
    fn add_node(&mut self, node_type: NodeType) {
        let template = &self.node_templates[&node_type];
        let center = self.screen_to_world(Point::new(400.0, 300.0));
        
        let node = WorkflowNode {
            id: Uuid::new_v4().to_string(),
            node_type,
            name: template.name.to_string(),
            position: center,
            position_data: (center.x, center.y),
            properties: template.default_properties.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            inputs: template.input_ports.iter()
                .map(|p| p.to_string())
                .collect(),
            outputs: template.output_ports.iter()
                .map(|p| p.to_string())
                .collect(),
        };
        
        let node_id = node.id.clone();
        self.workflow.nodes.insert(node_id.clone(), node);
        self.selected_node = Some(node_id);
    }
    
    fn delete_node(&mut self, node_id: &str) {
        self.workflow.nodes.remove(node_id);
        self.workflow.connections.retain(|conn| {
            conn.from_node != node_id && conn.to_node != node_id
        });
        
        if self.selected_node.as_ref() == Some(&node_id.to_string()) {
            self.selected_node = None;
        }
    }
    
    fn save_workflow(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.workflow) {
            let filename = format!("{}.json", self.workflow.name.replace(" ", "_").to_lowercase());
            if let Err(e) = std::fs::write(&filename, json) {
                eprintln!("Failed to save workflow: {}", e);
            } else {
                println!("Workflow saved to {}", filename);
            }
        }
    }
    
    fn export_yaml(&self) {
        let mut yaml = String::new();
        yaml.push_str(&format!("name: {}\n", self.workflow.name));
        yaml.push_str(&format!("description: {}\n", self.workflow.description));
        yaml.push_str("steps:\n");
        
        for node in self.workflow.nodes.values() {
            yaml.push_str(&format!("  - id: {}\n", node.id));
            yaml.push_str(&format!("    type: {:?}\n", node.node_type));
            yaml.push_str(&format!("    name: {}\n", node.name));
            
            if !node.properties.is_empty() {
                yaml.push_str("    properties:\n");
                for (key, value) in &node.properties {
                    yaml.push_str(&format!("      {}: {}\n", key, value));
                }
            }
        }
        
        let filename = format!("{}.yaml", self.workflow.name.replace(" ", "_").to_lowercase());
        if let Err(e) = std::fs::write(&filename, yaml) {
            eprintln!("Failed to export workflow: {}", e);
        } else {
            println!("Workflow exported to {}", filename);
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        let header = self.view_header();
        let main_content = if self.show_sidebar {
            row![
                container(self.view_sidebar())
                    .width(Length::Fixed(300.0))
                    .height(Length::Fill)
                    .style(container::rounded_box),
                container(self.view_canvas())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(container::rounded_box),
            ]
            .spacing(10)
        } else {
            row![
                container(self.view_canvas())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(container::rounded_box),
            ]
        };
        
        container(
            column![
                header,
                main_content,
            ]
            .spacing(10)
        )
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    fn view_header(&self) -> Element<Message> {
        container(
            row![
                text("🔄 Workflow Editor").size(24),
                Space::with_width(Length::Fill),
                button("Toggle Sidebar").on_press(Message::ToggleSidebar),
                button("Save").on_press(Message::SaveWorkflow),
                button("Export YAML").on_press(Message::ExportYaml),
                button("-").on_press(Message::ZoomOut),
                text(format!("{:.0}%", self.zoom * 100.0)),
                button("+").on_press(Message::ZoomIn),
                button("Reset").on_press(Message::ResetView),
                button("Close").on_press(Message::Close),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        )
        .padding(10)
        .style(container::rounded_box)
        .into()
    }
    
    fn view_sidebar(&self) -> Element<Message> {
        let mut sidebar = column![
            text("Node Types").size(20),
            Space::with_height(Length::Fixed(20.0)),
        ]
        .spacing(10);
        
        // Node palette
        let node_types = [
            (NodeType::Start, "➤ Start", "Begin workflow execution"),
            (NodeType::Action, "⚡ Action", "Execute a command or task"),
            (NodeType::Condition, "❓ Condition", "Branch based on condition"),
            (NodeType::Loop, "🔁 Loop", "Iterate over items"),
            (NodeType::Parallel, "⚡ Parallel", "Execute branches in parallel"),
            (NodeType::End, "🏁 End", "End workflow execution"),
        ];
        
        for (node_type, label, description) in &node_types {
            sidebar = sidebar.push(
                button(
                    column![
                        text(*label).size(16),
                        text(*description)
                            .size(12)
                            .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    ]
                    .spacing(2)
                )
                .on_press(Message::AddNode(*node_type))
                .style(button::secondary)
                .width(Length::Fill)
            );
        }
        
        // Selected node properties
        if let Some(node_id) = &self.selected_node {
            if let Some(node) = self.workflow.nodes.get(node_id) {
                sidebar = sidebar.push(Space::with_height(Length::Fixed(30.0)));
                sidebar = sidebar.push(text("Node Properties").size(18));
                sidebar = sidebar.push(text(&node.name).size(14));
                
                for (key, value) in &node.properties {
                    sidebar = sidebar.push(
                        column![
                            text(key).size(12),
                            text_input(key, value)
                                .on_input(move |v| Message::NodePropertyChanged(
                                    node_id.clone(),
                                    key.clone(),
                                    v
                                )),
                        ]
                        .spacing(2)
                    );
                }
                
                sidebar = sidebar.push(
                    button("Delete Node")
                        .on_press(Message::DeleteNode(node_id.clone()))
                        .style(button::danger)
                );
            }
        }
        
        scrollable(sidebar.padding(10))
            .height(Length::Fill)
            .into()
    }
    
    fn view_canvas(&self) -> Element<Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl canvas::Program<Message> for WorkflowEditor {
    type State = ();
    
    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(mouse_event) => {
                if let Some(position) = cursor.position_in(bounds) {
                    match mouse_event {
                        mouse::Event::ButtonPressed(mouse::Button::Left) => {
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::CanvasEvent(CanvasMessage::MousePressed(position)))
                            );
                        }
                        mouse::Event::ButtonReleased(mouse::Button::Left) => {
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::CanvasEvent(CanvasMessage::MouseReleased(position)))
                            );
                        }
                        mouse::Event::CursorMoved { .. } => {
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::CanvasEvent(CanvasMessage::MouseMoved(position)))
                            );
                        }
                        mouse::Event::WheelScrolled { delta } => {
                            let scroll_amount = match delta {
                                mouse::ScrollDelta::Lines { y, .. } => y * 20.0,
                                mouse::ScrollDelta::Pixels { y, .. } => y,
                            };
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::CanvasEvent(CanvasMessage::MouseWheel(scroll_amount)))
                            );
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        
        (canvas::event::Status::Ignored, None)
    }
    
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        // Draw grid
        let grid_color = Color::from_rgba(0.2, 0.2, 0.2, 0.3);
        let grid_start_x = (self.pan_offset.x % (GRID_SIZE * self.zoom)) - GRID_SIZE * self.zoom;
        let grid_start_y = (self.pan_offset.y % (GRID_SIZE * self.zoom)) - GRID_SIZE * self.zoom;
        
        let mut x = grid_start_x;
        while x < bounds.width {
            frame.stroke(
                &Path::line(
                    Point::new(x, 0.0),
                    Point::new(x, bounds.height),
                ),
                Stroke::default().with_color(grid_color).with_width(1.0),
            );
            x += GRID_SIZE * self.zoom;
        }
        
        let mut y = grid_start_y;
        while y < bounds.height {
            frame.stroke(
                &Path::line(
                    Point::new(0.0, y),
                    Point::new(bounds.width, y),
                ),
                Stroke::default().with_color(grid_color).with_width(1.0),
            );
            y += GRID_SIZE * self.zoom;
        }
        
        // Draw connections
        for connection in &self.workflow.connections {
            if let (Some(from_node), Some(to_node)) = (
                self.workflow.nodes.get(&connection.from_node),
                self.workflow.nodes.get(&connection.to_node),
            ) {
                let from_pos = self.world_to_screen(from_node.position + Vector::new(NODE_WIDTH, NODE_HEIGHT / 2.0));
                let to_pos = self.world_to_screen(to_node.position + Vector::new(0.0, NODE_HEIGHT / 2.0));
                
                let control_offset = ((to_pos.x - from_pos.x) / 2.0).abs();
                let mut path = canvas::path::Builder::new();
                path.move_to(from_pos);
                path.bezier_curve_to(
                    Point::new(from_pos.x + control_offset, from_pos.y),
                    Point::new(to_pos.x - control_offset, to_pos.y),
                    to_pos,
                );
                
                frame.stroke(
                    &path.build(),
                    Stroke::default()
                        .with_color(Color::from_rgb(0.6, 0.6, 0.6))
                        .with_width(2.0 * self.zoom),
                );
            }
        }
        
        // Draw nodes
        for (node_id, node) in &self.workflow.nodes {
            let pos = self.world_to_screen(node.position);
            let size = Size::new(NODE_WIDTH * self.zoom, NODE_HEIGHT * self.zoom);
            let node_rect = Rectangle::new(pos, size);
            
            let template = &self.node_templates[&node.node_type];
            let is_selected = self.selected_node.as_ref() == Some(node_id);
            
            // Node background
            frame.fill_rectangle(
                pos,
                size,
                if is_selected {
                    template.color
                } else {
                    Color::from_rgba(template.color.r, template.color.g, template.color.b, 0.8)
                },
            );
            
            // Node border
            frame.stroke(
                &Path::rectangle(pos, size),
                Stroke::default()
                    .with_color(if is_selected {
                        Color::WHITE
                    } else {
                        Color::from_rgb(0.3, 0.3, 0.3)
                    })
                    .with_width(if is_selected { 3.0 } else { 1.0 } * self.zoom),
            );
            
            // Node title
            frame.fill_text(CanvasText {
                content: node.name.clone(),
                position: pos + Vector::new(10.0 * self.zoom, 20.0 * self.zoom),
                color: Color::WHITE,
                size: (16.0 * self.zoom).into(),
                font: iced::Font::default(),
                horizontal_alignment: iced::alignment::Horizontal::Left,
                vertical_alignment: iced::alignment::Vertical::Top,
                line_height: iced::widget::text::LineHeight::default(),
                shaping: iced::widget::text::Shaping::Basic,
            });
            
            // Draw ports
            for (i, _port) in node.outputs.iter().enumerate() {
                let port_y = pos.y + (30.0 + i as f32 * 20.0) * self.zoom;
                let port_pos = Point::new(pos.x + size.width - PORT_RADIUS * self.zoom, port_y);
                
                frame.fill(
                    &Path::circle(port_pos, PORT_RADIUS * self.zoom),
                    Color::from_rgb(0.2, 1.0, 0.2),
                );
            }
            
            for (i, _port) in node.inputs.iter().enumerate() {
                let port_y = pos.y + (30.0 + i as f32 * 20.0) * self.zoom;
                let port_pos = Point::new(pos.x + PORT_RADIUS * self.zoom, port_y);
                
                frame.fill(
                    &Path::circle(port_pos, PORT_RADIUS * self.zoom),
                    Color::from_rgb(1.0, 0.2, 0.2),
                );
            }
        }
        
        vec![frame.into_geometry()]
    }
}

pub async fn run_workflow_editor() -> Result<()> {
    println!("Starting Workflow Editor...");
    
    iced::application(
        WorkflowEditor::title,
        WorkflowEditor::update,
        WorkflowEditor::view
    )
    .window(window::Settings {
        size: iced::Size::new(1400.0, 900.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(WorkflowEditor::new)
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}