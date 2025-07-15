//! Chart visualization using Iced Canvas

use iced::{
    Task, Element, Theme, Length, Color, Point, Vector, Size, Rectangle,
    widget::{
        column, container, text, button, row, Space, canvas::{self, Canvas, Path, Stroke, Fill},
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Message {
    CloseWindow,
    ToggleLegend,
    ToggleGrid,
    CycleChartType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub series: Vec<Series>,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub name: String,
    pub data: Vec<DataPoint>,
    pub color: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
    pub label: Option<String>,
}

#[derive(Debug)]
pub struct ChartView {
    title: String,
    data: ChartData,
    chart_type: ChartType,
    show_legend: bool,
    show_grid: bool,
    cache: canvas::Cache,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ChartType {
    Line,
    Bar,
    Scatter,
    Area,
}

impl ChartView {
    pub fn new(title: String, data: serde_json::Value, chart_type: String) -> (Self, Task<Message>) {
        let chart_data: ChartData = serde_json::from_value(data)
            .unwrap_or_else(|_| ChartData {
                series: vec![],
                x_label: None,
                y_label: None,
                title: None,
            });
        
        let chart_type = match chart_type.as_str() {
            "bar" => ChartType::Bar,
            "scatter" => ChartType::Scatter,
            "area" => ChartType::Area,
            _ => ChartType::Line,
        };
        
        (
            ChartView {
                title,
                data: chart_data,
                chart_type,
                show_legend: true,
                show_grid: true,
                cache: canvas::Cache::new(),
            },
            Task::none()
        )
    }

    pub fn title(&self) -> String {
        format!("{} - Chart Viewer", self.title)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CloseWindow => {
                std::process::exit(0);
            }
            Message::ToggleLegend => {
                self.show_legend = !self.show_legend;
                self.cache.clear();
                Task::none()
            }
            Message::ToggleGrid => {
                self.show_grid = !self.show_grid;
                self.cache.clear();
                Task::none()
            }
            Message::CycleChartType => {
                self.chart_type = match self.chart_type {
                    ChartType::Line => ChartType::Bar,
                    ChartType::Bar => ChartType::Scatter,
                    ChartType::Scatter => ChartType::Area,
                    ChartType::Area => ChartType::Line,
                };
                self.cache.clear();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = row![
            text(&self.title).size(24),
            Space::with_width(Length::Fill),
            button(text(format!("Type: {:?}", self.chart_type)))
                .on_press(Message::CycleChartType),
            button(text(if self.show_grid { "Grid: On" } else { "Grid: Off" }))
                .on_press(Message::ToggleGrid),
            button(text(if self.show_legend { "Legend: On" } else { "Legend: Off" }))
                .on_press(Message::ToggleLegend),
            button(text("✕ Close")).on_press(Message::CloseWindow),
        ]
        .spacing(10)
        .padding(10);

        let chart_canvas = Canvas::new(ChartCanvas {
            data: &self.data,
            chart_type: self.chart_type,
            show_grid: self.show_grid,
            cache: &self.cache,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let mut content = column![
            container(header)
                .style(container::rounded_box)
                .width(Length::Fill),
            container(chart_canvas)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(20),
        ]
        .spacing(10);

        // Add legend if enabled
        if self.show_legend && !self.data.series.is_empty() {
            let mut legend_row = row![].spacing(20);
            
            for series in &self.data.series {
                let color = series.color.unwrap_or([0.5, 0.5, 0.5]);
                let color_box = container("")
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
                    .style(move |_theme| container::Style {
                        background: Some(Color::from_rgb(color[0], color[1], color[2]).into()),
                        border: iced::Border {
                            width: 1.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                
                legend_row = legend_row.push(
                    row![
                        color_box,
                        text(&series.name).size(14),
                    ]
                    .spacing(5)
                    .align_y(iced::Alignment::Center)
                );
            }
            
            content = content.push(
                container(legend_row)
                    .style(container::rounded_box)
                    .padding(10)
                    .width(Length::Fill)
                    .center_x(Length::Fill)
            );
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}

struct ChartCanvas<'a> {
    data: &'a ChartData,
    chart_type: ChartType,
    show_grid: bool,
    cache: &'a canvas::Cache,
}

impl<'a> canvas::Program<Message> for ChartCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Clear background
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Color::from_rgba(0.1, 0.1, 0.1, 1.0),
            );

            if self.data.series.is_empty() {
                return;
            }

            // Calculate data bounds
            let (x_min, x_max, y_min, y_max) = calculate_bounds(&self.data.series);
            
            // Define chart area with margins
            let margin = 60.0;
            let chart_bounds = Rectangle {
                x: margin,
                y: margin / 2.0,
                width: bounds.width - 2.0 * margin,
                height: bounds.height - 2.0 * margin,
            };

            // Draw grid if enabled
            if self.show_grid {
                draw_grid(frame, &chart_bounds, x_min, x_max, y_min, y_max);
            }

            // Draw axes
            draw_axes(frame, &chart_bounds, &self.data);

            // Draw data based on chart type
            match self.chart_type {
                ChartType::Line => draw_line_chart(frame, &chart_bounds, &self.data.series, x_min, x_max, y_min, y_max),
                ChartType::Bar => draw_bar_chart(frame, &chart_bounds, &self.data.series, x_min, x_max, y_min, y_max),
                ChartType::Scatter => draw_scatter_chart(frame, &chart_bounds, &self.data.series, x_min, x_max, y_min, y_max),
                ChartType::Area => draw_area_chart(frame, &chart_bounds, &self.data.series, x_min, x_max, y_min, y_max),
            }

            // Draw axis labels
            draw_axis_labels(frame, &chart_bounds, &self.data, x_min, x_max, y_min, y_max);
        });

        vec![geometry]
    }
}

fn calculate_bounds(series: &[Series]) -> (f64, f64, f64, f64) {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for s in series {
        for point in &s.data {
            x_min = x_min.min(point.x);
            x_max = x_max.max(point.x);
            y_min = y_min.min(point.y);
            y_max = y_max.max(point.y);
        }
    }

    // Add some padding
    let x_padding = (x_max - x_min) * 0.1;
    let y_padding = (y_max - y_min) * 0.1;
    
    (
        x_min - x_padding,
        x_max + x_padding,
        y_min - y_padding,
        y_max + y_padding,
    )
}

fn draw_grid(frame: &mut canvas::Frame, bounds: &Rectangle, x_min: f64, x_max: f64, y_min: f64, y_max: f64) {
    let grid_color = Color::from_rgba(0.3, 0.3, 0.3, 0.5);
    let stroke = Stroke::default()
        .with_color(grid_color)
        .with_width(1.0);

    // Vertical grid lines
    for i in 0..=10 {
        let x = bounds.x + (i as f32 / 10.0) * bounds.width;
        let path = Path::line(
            Point::new(x, bounds.y),
            Point::new(x, bounds.y + bounds.height),
        );
        frame.stroke(&path, stroke.clone());
    }

    // Horizontal grid lines
    for i in 0..=10 {
        let y = bounds.y + (i as f32 / 10.0) * bounds.height;
        let path = Path::line(
            Point::new(bounds.x, y),
            Point::new(bounds.x + bounds.width, y),
        );
        frame.stroke(&path, stroke.clone());
    }
}

fn draw_axes(frame: &mut canvas::Frame, bounds: &Rectangle, data: &ChartData) {
    let axis_color = Color::from_rgba(0.8, 0.8, 0.8, 1.0);
    let stroke = Stroke::default()
        .with_color(axis_color)
        .with_width(2.0);

    // X axis
    let x_axis = Path::line(
        Point::new(bounds.x, bounds.y + bounds.height),
        Point::new(bounds.x + bounds.width, bounds.y + bounds.height),
    );
    frame.stroke(&x_axis, stroke.clone());

    // Y axis
    let y_axis = Path::line(
        Point::new(bounds.x, bounds.y),
        Point::new(bounds.x, bounds.y + bounds.height),
    );
    frame.stroke(&y_axis, stroke);

    // Draw title if present
    if let Some(title) = &data.title {
        frame.fill_text(canvas::Text {
            content: title.clone(),
            position: Point::new(bounds.x + bounds.width / 2.0, bounds.y - 20.0),
            size: iced::Pixels(18.0),
            color: Color::WHITE,
            font: iced::Font::default(),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Center,
            line_height: iced::widget::text::LineHeight::default(),
            shaping: iced::widget::text::Shaping::Basic,
        });
    }
}

fn draw_line_chart(
    frame: &mut canvas::Frame,
    bounds: &Rectangle,
    series: &[Series],
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) {
    for (series_idx, s) in series.iter().enumerate() {
        if s.data.is_empty() {
            continue;
        }

        let color = s.color
            .map(|c| Color::from_rgb(c[0], c[1], c[2]))
            .unwrap_or_else(|| get_series_color(series_idx));

        let stroke = Stroke::default()
            .with_color(color)
            .with_width(2.0);

        let mut path = canvas::path::Builder::new();
        
        for (i, point) in s.data.iter().enumerate() {
            let x = bounds.x + ((point.x - x_min) / (x_max - x_min)) as f32 * bounds.width;
            let y = bounds.y + bounds.height - ((point.y - y_min) / (y_max - y_min)) as f32 * bounds.height;
            
            if i == 0 {
                path.move_to(Point::new(x, y));
            } else {
                path.line_to(Point::new(x, y));
            }
        }

        frame.stroke(&path.build(), stroke);

        // Draw points
        for point in &s.data {
            let x = bounds.x + ((point.x - x_min) / (x_max - x_min)) as f32 * bounds.width;
            let y = bounds.y + bounds.height - ((point.y - y_min) / (y_max - y_min)) as f32 * bounds.height;
            
            frame.fill(
                &Path::circle(Point::new(x, y), 3.0),
                color,
            );
        }
    }
}

fn draw_bar_chart(
    frame: &mut canvas::Frame,
    bounds: &Rectangle,
    series: &[Series],
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) {
    let num_series = series.len() as f32;
    let total_points = series.first().map(|s| s.data.len()).unwrap_or(0);
    
    if total_points == 0 {
        return;
    }

    let bar_width = bounds.width / (total_points as f32 * (num_series + 1.0));
    
    for (series_idx, s) in series.iter().enumerate() {
        let color = s.color
            .map(|c| Color::from_rgb(c[0], c[1], c[2]))
            .unwrap_or_else(|| get_series_color(series_idx));

        for (point_idx, point) in s.data.iter().enumerate() {
            let x_center = bounds.x + ((point.x - x_min) / (x_max - x_min)) as f32 * bounds.width;
            let x = x_center - (bar_width * num_series / 2.0) + (series_idx as f32 * bar_width);
            let height = ((point.y - y_min) / (y_max - y_min)) as f32 * bounds.height;
            let y = bounds.y + bounds.height - height;
            
            frame.fill_rectangle(
                Point::new(x, y),
                Size::new(bar_width * 0.8, height),
                color,
            );
        }
    }
}

fn draw_scatter_chart(
    frame: &mut canvas::Frame,
    bounds: &Rectangle,
    series: &[Series],
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) {
    for (series_idx, s) in series.iter().enumerate() {
        let color = s.color
            .map(|c| Color::from_rgb(c[0], c[1], c[2]))
            .unwrap_or_else(|| get_series_color(series_idx));

        for point in &s.data {
            let x = bounds.x + ((point.x - x_min) / (x_max - x_min)) as f32 * bounds.width;
            let y = bounds.y + bounds.height - ((point.y - y_min) / (y_max - y_min)) as f32 * bounds.height;
            
            frame.fill(
                &Path::circle(Point::new(x, y), 5.0),
                color,
            );
        }
    }
}

fn draw_area_chart(
    frame: &mut canvas::Frame,
    bounds: &Rectangle,
    series: &[Series],
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) {
    for (series_idx, s) in series.iter().enumerate() {
        if s.data.is_empty() {
            continue;
        }

        let color = s.color
            .map(|c| Color::from_rgba(c[0], c[1], c[2], 0.5))
            .unwrap_or_else(|| {
                let base = get_series_color(series_idx);
                Color::from_rgba(base.r, base.g, base.b, 0.5)
            });

        let mut path = canvas::path::Builder::new();
        
        // Start from bottom
        let first_x = bounds.x + ((s.data[0].x - x_min) / (x_max - x_min)) as f32 * bounds.width;
        path.move_to(Point::new(first_x, bounds.y + bounds.height));
        
        // Draw area
        for point in &s.data {
            let x = bounds.x + ((point.x - x_min) / (x_max - x_min)) as f32 * bounds.width;
            let y = bounds.y + bounds.height - ((point.y - y_min) / (y_max - y_min)) as f32 * bounds.height;
            path.line_to(Point::new(x, y));
        }
        
        // Close path
        let last_x = bounds.x + ((s.data.last().unwrap().x - x_min) / (x_max - x_min)) as f32 * bounds.width;
        path.line_to(Point::new(last_x, bounds.y + bounds.height));
        path.close();
        
        frame.fill(&path.build(), color);
    }
}

fn draw_axis_labels(
    frame: &mut canvas::Frame,
    bounds: &Rectangle,
    data: &ChartData,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) {
    let text_color = Color::from_rgba(0.8, 0.8, 0.8, 1.0);
    
    // X-axis labels
    for i in 0..=5 {
        let value = x_min + (i as f64 / 5.0) * (x_max - x_min);
        let x = bounds.x + (i as f32 / 5.0) * bounds.width;
        
        frame.fill_text(canvas::Text {
            content: format!("{:.1}", value),
            position: Point::new(x, bounds.y + bounds.height + 15.0),
            size: iced::Pixels(12.0),
            color: text_color,
            font: iced::Font::default(),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Top,
            line_height: iced::widget::text::LineHeight::default(),
            shaping: iced::widget::text::Shaping::default(),
        });
    }
    
    // Y-axis labels
    for i in 0..=5 {
        let value = y_min + (i as f64 / 5.0) * (y_max - y_min);
        let y = bounds.y + bounds.height - (i as f32 / 5.0) * bounds.height;
        
        frame.fill_text(canvas::Text {
            content: format!("{:.1}", value),
            position: Point::new(bounds.x - 10.0, y),
            size: iced::Pixels(12.0),
            color: text_color,
            font: iced::Font::default(),
            horizontal_alignment: iced::alignment::Horizontal::Right,
            vertical_alignment: iced::alignment::Vertical::Center,
            line_height: iced::widget::text::LineHeight::default(),
            shaping: iced::widget::text::Shaping::default(),
        });
    }
    
    // Axis labels
    if let Some(x_label) = &data.x_label {
        frame.fill_text(canvas::Text {
            content: x_label.clone(),
            position: Point::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height + 35.0),
            size: iced::Pixels(14.0),
            color: text_color,
            font: iced::Font::default(),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Top,
            line_height: iced::widget::text::LineHeight::default(),
            shaping: iced::widget::text::Shaping::default(),
        });
    }
    
    if let Some(y_label) = &data.y_label {
        // Note: Rotating text is not directly supported, so we place it horizontally
        frame.fill_text(canvas::Text {
            content: y_label.clone(),
            position: Point::new(20.0, bounds.y + bounds.height / 2.0),
            size: iced::Pixels(14.0),
            color: text_color,
            font: iced::Font::default(),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Center,
            line_height: iced::widget::text::LineHeight::default(),
            shaping: iced::widget::text::Shaping::default(),
        });
    }
}

fn get_series_color(index: usize) -> Color {
    const COLORS: &[[f32; 3]] = &[
        [0.2, 0.6, 1.0],  // Blue
        [1.0, 0.4, 0.2],  // Orange
        [0.2, 0.8, 0.4],  // Green
        [0.8, 0.2, 0.8],  // Purple
        [1.0, 0.8, 0.2],  // Yellow
        [0.2, 0.8, 0.8],  // Cyan
    ];
    
    let color = COLORS[index % COLORS.len()];
    Color::from_rgb(color[0], color[1], color[2])
}

pub fn run_chart_viewer(
    title: String,
    data: serde_json::Value,
    chart_type: String,
) -> Result<(), iced::Error> {
    iced::application(
        ChartView::title,
        ChartView::update,
        ChartView::view
    )
    .window(iced::window::Settings {
        size: iced::Size::new(800.0, 600.0),
        position: iced::window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(ChartView::theme)
    .run_with(|| ChartView::new(title, data, chart_type))
}