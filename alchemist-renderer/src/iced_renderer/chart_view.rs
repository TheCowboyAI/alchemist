//! Chart renderer view for Iced
//! 
//! Renders various chart types with support for:
//! - Multiple chart types (line, bar, scatter, pie)
//! - Real-time data updates
//! - Interactive tooltips and legends
//! - Zoom and pan functionality
//! - Export to PNG/SVG

use iced::{
    widget::{canvas, column, container, row, button, text, pick_list, Canvas},
    window, Element, Length, Point, Rectangle, Size, Vector, Color, Theme,
    mouse,
};
use iced::widget::canvas::{Cache, Cursor, Frame, Geometry, Path, Program, Stroke, Fill, Text as CanvasText};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Chart types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    Line,
    Bar,
    Scatter,
    Pie,
    Area,
}

impl std::fmt::Display for ChartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChartType::Line => write!(f, "Line"),
            ChartType::Bar => write!(f, "Bar"),
            ChartType::Scatter => write!(f, "Scatter"),
            ChartType::Pie => write!(f, "Pie"),
            ChartType::Area => write!(f, "Area"),
        }
    }
}

/// Chart data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
    pub label: Option<String>,
}

/// Chart series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub name: String,
    pub data: Vec<DataPoint>,
    pub color: Option<[f32; 4]>,
}

/// Chart options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOptions {
    pub title: Option<String>,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub show_grid: bool,
    pub show_legend: bool,
    pub interactive: bool,
    pub animation_duration: f32,
}

impl Default for ChartOptions {
    fn default() -> Self {
        Self {
            title: None,
            x_label: None,
            y_label: None,
            show_grid: true,
            show_legend: true,
            interactive: true,
            animation_duration: 0.5,
        }
    }
}

/// Chart view state
pub struct ChartView {
    data: Vec<Series>,
    chart_type: ChartType,
    options: ChartOptions,
    cache: Cache,
    bounds: Option<ChartBounds>,
    hover_point: Option<(usize, usize)>, // (series_index, point_index)
    zoom_level: f32,
    pan_offset: Vector,
    is_panning: bool,
    last_cursor_position: Point,
}

#[derive(Debug, Clone)]
struct ChartBounds {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

#[derive(Debug, Clone)]
pub enum ChartMessage {
    ChartTypeChanged(ChartType),
    DataUpdated(Vec<Series>),
    Zoom(f32),
    Pan(Vector),
    MouseMoved(Point),
    MousePressed,
    MouseReleased,
    ExportPNG,
    ExportSVG,
    ToggleGrid,
    ToggleLegend,
}

impl ChartView {
    pub fn new(data: Vec<Series>, chart_type: ChartType, options: ChartOptions) -> Self {
        let mut view = Self {
            data,
            chart_type,
            options,
            cache: Cache::default(),
            bounds: None,
            hover_point: None,
            zoom_level: 1.0,
            pan_offset: Vector::new(0.0, 0.0),
            is_panning: false,
            last_cursor_position: Point::ORIGIN,
        };
        view.calculate_bounds();
        view
    }

    fn calculate_bounds(&mut self) {
        if self.data.is_empty() {
            return;
        }

        let mut x_min = f64::MAX;
        let mut x_max = f64::MIN;
        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;

        for series in &self.data {
            for point in &series.data {
                x_min = x_min.min(point.x);
                x_max = x_max.max(point.x);
                y_min = y_min.min(point.y);
                y_max = y_max.max(point.y);
            }
        }

        // Add padding
        let x_padding = (x_max - x_min) * 0.1;
        let y_padding = (y_max - y_min) * 0.1;

        self.bounds = Some(ChartBounds {
            x_min: x_min - x_padding,
            x_max: x_max + x_padding,
            y_min: y_min - y_padding,
            y_max: y_max + y_padding,
        });
    }

    pub fn update(&mut self, message: ChartMessage) -> iced::Command<ChartMessage> {
        match message {
            ChartMessage::ChartTypeChanged(chart_type) => {
                self.chart_type = chart_type;
                self.cache.clear();
            }
            ChartMessage::DataUpdated(data) => {
                self.data = data;
                self.calculate_bounds();
                self.cache.clear();
            }
            ChartMessage::Zoom(delta) => {
                self.zoom_level = (self.zoom_level + delta).max(0.1).min(10.0);
                self.cache.clear();
            }
            ChartMessage::Pan(delta) => {
                self.pan_offset = self.pan_offset + delta;
                self.cache.clear();
            }
            ChartMessage::MouseMoved(position) => {
                if self.is_panning {
                    let delta = position - self.last_cursor_position;
                    self.pan_offset = self.pan_offset + Vector::new(delta.x, delta.y);
                    self.cache.clear();
                }
                self.last_cursor_position = position;
                // TODO: Update hover point detection
            }
            ChartMessage::MousePressed => {
                self.is_panning = true;
            }
            ChartMessage::MouseReleased => {
                self.is_panning = false;
            }
            ChartMessage::ExportPNG => {
                // TODO: Implement PNG export
                tracing::info!("Export to PNG requested");
            }
            ChartMessage::ExportSVG => {
                // TODO: Implement SVG export
                tracing::info!("Export to SVG requested");
            }
            ChartMessage::ToggleGrid => {
                self.options.show_grid = !self.options.show_grid;
                self.cache.clear();
            }
            ChartMessage::ToggleLegend => {
                self.options.show_legend = !self.options.show_legend;
                self.cache.clear();
            }
        }
        iced::Command::none()
    }

    pub fn view(&self) -> Element<ChartMessage> {
        let chart_types = vec![
            ChartType::Line,
            ChartType::Bar,
            ChartType::Scatter,
            ChartType::Pie,
            ChartType::Area,
        ];

        let controls = row![
            text("Chart Type:").size(14),
            pick_list(
                chart_types,
                Some(self.chart_type),
                ChartMessage::ChartTypeChanged
            ),
            button("Grid").on_press(ChartMessage::ToggleGrid),
            button("Legend").on_press(ChartMessage::ToggleLegend),
            button("Export PNG").on_press(ChartMessage::ExportPNG),
            button("Export SVG").on_press(ChartMessage::ExportSVG),
        ]
        .spacing(10)
        .padding(10);

        let canvas = Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill);

        column![
            controls,
            container(canvas)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(Color::WHITE)),
                    border: iced::Border {
                        width: 1.0,
                        color: Color::from_rgb(0.8, 0.8, 0.8),
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
        ]
        .into()
    }
}

impl Program<ChartMessage> for ChartView {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (canvas::event::Status, Option<ChartMessage>) {
        match event {
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { position } => {
                    if bounds.contains(position) {
                        return (
                            canvas::event::Status::Captured,
                            Some(ChartMessage::MouseMoved(position)),
                        );
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(position) = cursor.position_in(bounds) {
                        return (
                            canvas::event::Status::Captured,
                            Some(ChartMessage::MousePressed),
                        );
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    return (
                        canvas::event::Status::Captured,
                        Some(ChartMessage::MouseReleased),
                    );
                }
                mouse::Event::WheelScrolled { delta } => {
                    if let Some(_position) = cursor.position_in(bounds) {
                        let zoom_delta = match delta {
                            mouse::ScrollDelta::Lines { y, .. } => y * 0.1,
                            mouse::ScrollDelta::Pixels { y, .. } => y * 0.001,
                        };
                        return (
                            canvas::event::Status::Captured,
                            Some(ChartMessage::Zoom(zoom_delta)),
                        );
                    }
                }
                _ => {}
            },
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
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_chart(frame, bounds.size());
        });

        vec![geometry]
    }
}

impl ChartView {
    fn draw_chart(&self, frame: &mut Frame, size: Size) {
        let padding = 60.0;
        let chart_area = Rectangle {
            x: padding,
            y: padding,
            width: size.width - 2.0 * padding,
            height: size.height - 2.0 * padding,
        };

        // Apply zoom and pan transformations
        frame.translate(Vector::new(
            chart_area.x + self.pan_offset.x,
            chart_area.y + self.pan_offset.y,
        ));
        frame.scale(self.zoom_level);

        // Draw background
        frame.fill_rectangle(
            Point::new(0.0, 0.0),
            Size::new(chart_area.width, chart_area.height),
            Color::WHITE,
        );

        // Draw grid if enabled
        if self.options.show_grid {
            self.draw_grid(frame, chart_area);
        }

        // Draw axes
        self.draw_axes(frame, chart_area);

        // Draw chart based on type
        match self.chart_type {
            ChartType::Line => self.draw_line_chart(frame, chart_area),
            ChartType::Bar => self.draw_bar_chart(frame, chart_area),
            ChartType::Scatter => self.draw_scatter_chart(frame, chart_area),
            ChartType::Pie => self.draw_pie_chart(frame, chart_area),
            ChartType::Area => self.draw_area_chart(frame, chart_area),
        }

        // Draw legend if enabled
        if self.options.show_legend {
            self.draw_legend(frame, chart_area);
        }

        // Draw title if present
        if let Some(title) = &self.options.title {
            frame.fill_text(CanvasText {
                content: title.clone(),
                position: Point::new(chart_area.width / 2.0, -40.0),
                size: iced::Pixels(20.0),
                color: Color::BLACK,
                font: iced::Font::default(),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Top,
            });
        }
    }

    fn draw_grid(&self, frame: &mut Frame, area: Rectangle) {
        let grid_color = Color::from_rgb(0.9, 0.9, 0.9);
        let grid_stroke = Stroke::default()
            .with_color(grid_color)
            .with_width(1.0);

        // Vertical grid lines
        let v_lines = 10;
        for i in 0..=v_lines {
            let x = area.width * i as f32 / v_lines as f32;
            let path = Path::line(
                Point::new(x, 0.0),
                Point::new(x, area.height),
            );
            frame.stroke(&path, grid_stroke);
        }

        // Horizontal grid lines
        let h_lines = 8;
        for i in 0..=h_lines {
            let y = area.height * i as f32 / h_lines as f32;
            let path = Path::line(
                Point::new(0.0, y),
                Point::new(area.width, y),
            );
            frame.stroke(&path, grid_stroke);
        }
    }

    fn draw_axes(&self, frame: &mut Frame, area: Rectangle) {
        let axes_stroke = Stroke::default()
            .with_color(Color::BLACK)
            .with_width(2.0);

        // X-axis
        let x_axis = Path::line(
            Point::new(0.0, area.height),
            Point::new(area.width, area.height),
        );
        frame.stroke(&x_axis, axes_stroke);

        // Y-axis
        let y_axis = Path::line(
            Point::new(0.0, 0.0),
            Point::new(0.0, area.height),
        );
        frame.stroke(&y_axis, axes_stroke);

        // Axis labels
        if let Some(x_label) = &self.options.x_label {
            frame.fill_text(CanvasText {
                content: x_label.clone(),
                position: Point::new(area.width / 2.0, area.height + 40.0),
                size: iced::Pixels(14.0),
                color: Color::BLACK,
                font: iced::Font::default(),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Top,
            });
        }

        if let Some(y_label) = &self.options.y_label {
            // Note: Iced doesn't support text rotation easily, so we'll place it horizontally
            frame.fill_text(CanvasText {
                content: y_label.clone(),
                position: Point::new(-50.0, area.height / 2.0),
                size: iced::Pixels(14.0),
                color: Color::BLACK,
                font: iced::Font::default(),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Center,
            });
        }
    }

    fn draw_line_chart(&self, frame: &mut Frame, area: Rectangle) {
        if let Some(bounds) = &self.bounds {
            for (series_idx, series) in self.data.iter().enumerate() {
                if series.data.is_empty() {
                    continue;
                }

                let color = series.color
                    .map(|c| Color::from_rgba(c[0], c[1], c[2], c[3]))
                    .unwrap_or_else(|| self.get_default_color(series_idx));

                let mut path_builder = canvas::path::Builder::new();
                let mut first = true;

                for point in &series.data {
                    let x = self.map_x(point.x, bounds, area.width);
                    let y = self.map_y(point.y, bounds, area.height);

                    if first {
                        path_builder.move_to(Point::new(x, y));
                        first = false;
                    } else {
                        path_builder.line_to(Point::new(x, y));
                    }
                }

                let path = path_builder.build();
                let stroke = Stroke::default()
                    .with_color(color)
                    .with_width(2.0);
                frame.stroke(&path, stroke);

                // Draw points
                for point in &series.data {
                    let x = self.map_x(point.x, bounds, area.width);
                    let y = self.map_y(point.y, bounds, area.height);
                    frame.fill(
                        &Path::circle(Point::new(x, y), 3.0),
                        color,
                    );
                }
            }
        }
    }

    fn draw_bar_chart(&self, frame: &mut Frame, area: Rectangle) {
        if let Some(bounds) = &self.bounds {
            let num_series = self.data.len();
            let num_points = self.data.iter().map(|s| s.data.len()).max().unwrap_or(0);
            
            if num_points == 0 {
                return;
            }

            let bar_width = area.width / (num_points as f32 * (num_series + 1) as f32);
            let group_width = bar_width * num_series as f32;

            for (series_idx, series) in self.data.iter().enumerate() {
                let color = series.color
                    .map(|c| Color::from_rgba(c[0], c[1], c[2], c[3]))
                    .unwrap_or_else(|| self.get_default_color(series_idx));

                for (point_idx, point) in series.data.iter().enumerate() {
                    let x = point_idx as f32 * (group_width + bar_width) + series_idx as f32 * bar_width;
                    let y = self.map_y(point.y, bounds, area.height);
                    let height = area.height - y;

                    frame.fill_rectangle(
                        Point::new(x, y),
                        Size::new(bar_width * 0.8, height),
                        color,
                    );
                }
            }
        }
    }

    fn draw_scatter_chart(&self, frame: &mut Frame, area: Rectangle) {
        if let Some(bounds) = &self.bounds {
            for (series_idx, series) in self.data.iter().enumerate() {
                let color = series.color
                    .map(|c| Color::from_rgba(c[0], c[1], c[2], c[3]))
                    .unwrap_or_else(|| self.get_default_color(series_idx));

                for point in &series.data {
                    let x = self.map_x(point.x, bounds, area.width);
                    let y = self.map_y(point.y, bounds, area.height);
                    
                    frame.fill(
                        &Path::circle(Point::new(x, y), 5.0),
                        color,
                    );
                }
            }
        }
    }

    fn draw_pie_chart(&self, frame: &mut Frame, area: Rectangle) {
        // For pie charts, we'll use the first series and interpret y values as pie slice sizes
        if let Some(series) = self.data.first() {
            let center = Point::new(area.width / 2.0, area.height / 2.0);
            let radius = area.width.min(area.height) / 2.0 * 0.8;
            
            let total: f64 = series.data.iter().map(|p| p.y).sum();
            if total == 0.0 {
                return;
            }

            let mut start_angle = -std::f32::consts::PI / 2.0; // Start at top

            for (idx, point) in series.data.iter().enumerate() {
                let slice_angle = (point.y / total) as f32 * 2.0 * std::f32::consts::PI;
                let color = self.get_default_color(idx);

                // Draw pie slice
                let mut path_builder = canvas::path::Builder::new();
                path_builder.move_to(center);
                path_builder.arc(canvas::path::Arc {
                    center,
                    radius,
                    start_angle: start_angle.into(),
                    end_angle: (start_angle + slice_angle).into(),
                });
                path_builder.close();

                let path = path_builder.build();
                frame.fill(&path, color);
                frame.stroke(&path, Stroke::default().with_color(Color::WHITE).with_width(2.0));

                start_angle += slice_angle;
            }
        }
    }

    fn draw_area_chart(&self, frame: &mut Frame, area: Rectangle) {
        if let Some(bounds) = &self.bounds {
            for (series_idx, series) in self.data.iter().enumerate() {
                if series.data.is_empty() {
                    continue;
                }

                let color = series.color
                    .map(|c| Color::from_rgba(c[0], c[1], c[2], c[3]))
                    .unwrap_or_else(|| self.get_default_color(series_idx));
                
                let mut fill_color = color;
                fill_color.a = 0.3; // Make semi-transparent

                let mut path_builder = canvas::path::Builder::new();
                
                // Start at bottom-left of first point
                let first_x = self.map_x(series.data[0].x, bounds, area.width);
                path_builder.move_to(Point::new(first_x, area.height));

                // Draw line to first point and then through all points
                for point in &series.data {
                    let x = self.map_x(point.x, bounds, area.width);
                    let y = self.map_y(point.y, bounds, area.height);
                    path_builder.line_to(Point::new(x, y));
                }

                // Close the path by going to bottom-right and back to start
                let last_x = self.map_x(series.data.last().unwrap().x, bounds, area.width);
                path_builder.line_to(Point::new(last_x, area.height));
                path_builder.close();

                let path = path_builder.build();
                frame.fill(&path, fill_color);
                
                // Draw the line on top
                self.draw_line_chart(frame, area);
            }
        }
    }

    fn draw_legend(&self, frame: &mut Frame, area: Rectangle) {
        let legend_x = area.width - 150.0;
        let legend_y = 20.0;
        let item_height = 25.0;

        for (idx, series) in self.data.iter().enumerate() {
            let color = series.color
                .map(|c| Color::from_rgba(c[0], c[1], c[2], c[3]))
                .unwrap_or_else(|| self.get_default_color(idx));

            let y = legend_y + idx as f32 * item_height;

            // Color box
            frame.fill_rectangle(
                Point::new(legend_x, y),
                Size::new(20.0, 15.0),
                color,
            );

            // Series name
            frame.fill_text(CanvasText {
                content: series.name.clone(),
                position: Point::new(legend_x + 25.0, y + 7.5),
                size: iced::Pixels(14.0),
                color: Color::BLACK,
                font: iced::Font::default(),
                horizontal_alignment: iced::alignment::Horizontal::Left,
                vertical_alignment: iced::alignment::Vertical::Center,
            });
        }
    }

    fn map_x(&self, value: f64, bounds: &ChartBounds, width: f32) -> f32 {
        ((value - bounds.x_min) / (bounds.x_max - bounds.x_min)) as f32 * width
    }

    fn map_y(&self, value: f64, bounds: &ChartBounds, height: f32) -> f32 {
        height - ((value - bounds.y_min) / (bounds.y_max - bounds.y_min)) as f32 * height
    }

    fn get_default_color(&self, index: usize) -> Color {
        const COLORS: &[Color] = &[
            Color::from_rgb(0.12, 0.47, 0.71),  // Blue
            Color::from_rgb(1.0, 0.5, 0.05),     // Orange
            Color::from_rgb(0.17, 0.63, 0.17),   // Green
            Color::from_rgb(0.84, 0.15, 0.16),   // Red
            Color::from_rgb(0.58, 0.4, 0.74),    // Purple
            Color::from_rgb(0.55, 0.34, 0.29),   // Brown
            Color::from_rgb(0.89, 0.47, 0.76),   // Pink
            Color::from_rgb(0.5, 0.5, 0.5),      // Gray
            Color::from_rgb(0.74, 0.74, 0.13),   // Yellow
            Color::from_rgb(0.09, 0.75, 0.81),   // Cyan
        ];
        COLORS[index % COLORS.len()]
    }
}