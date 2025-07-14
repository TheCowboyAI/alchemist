//! Performance monitoring UI panel
//!
//! Real-time visualization of system performance metrics including
//! CPU usage, memory consumption, network activity, and more.

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment, Color};
use iced::widget::{column, container, row, text, button, scrollable, Space, progress_bar, Canvas};
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke};
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sysinfo::{System, SystemExt, ProcessExt, CpuExt, NetworkExt, NetworksExt, PidExt};

const GRAPH_POINTS: usize = 60; // 60 seconds of history
const UPDATE_INTERVAL_MS: u64 = 1000; // Update every second

#[derive(Debug, Clone)]
pub enum Message {
    UpdateMetrics,
    ToggleProcess(String),
    SortBy(SortColumn),
    ClearHistory,
    ExportMetrics,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortColumn {
    Name,
    Cpu,
    Memory,
    Network,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_used_mb: f64,
    pub memory_total_mb: f64,
    pub network_rx_kb: f64,
    pub network_tx_kb: f64,
    pub process_count: usize,
    pub load_average: [f64; 3],
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_mb: f64,
    pub status: String,
}

pub struct PerformanceMonitor {
    system: System,
    metrics_history: VecDeque<SystemMetrics>,
    current_metrics: Option<SystemMetrics>,
    processes: Vec<ProcessInfo>,
    selected_processes: Vec<String>,
    sort_column: SortColumn,
    sort_ascending: bool,
    graph_cache: Cache,
}

impl PerformanceMonitor {
    pub fn new() -> (Self, Task<Message>) {
        let mut system = System::new_all();
        system.refresh_all();
        
        (
            PerformanceMonitor {
                system,
                metrics_history: VecDeque::with_capacity(GRAPH_POINTS),
                current_metrics: None,
                processes: Vec::new(),
                selected_processes: Vec::new(),
                sort_column: SortColumn::Cpu,
                sort_ascending: false,
                graph_cache: Cache::default(),
            },
            Task::none()
        )
    }
    
    pub fn title(&self) -> String {
        "Performance Monitor".to_string()
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateMetrics => {
                self.refresh_metrics();
                self.graph_cache.clear();
                Task::none()
            }
            
            Message::ToggleProcess(name) => {
                if self.selected_processes.contains(&name) {
                    self.selected_processes.retain(|n| n != &name);
                } else {
                    self.selected_processes.push(name);
                }
                Task::none()
            }
            
            Message::SortBy(column) => {
                if self.sort_column == column {
                    self.sort_ascending = !self.sort_ascending;
                } else {
                    self.sort_column = column;
                    self.sort_ascending = false;
                }
                self.sort_processes();
                Task::none()
            }
            
            Message::ClearHistory => {
                self.metrics_history.clear();
                self.graph_cache.clear();
                Task::none()
            }
            
            Message::ExportMetrics => {
                self.export_metrics();
                Task::none()
            }
            
            Message::Close => {
                std::process::exit(0);
            }
        }
    }
    
    fn refresh_metrics(&mut self) {
        // Refresh system information
        self.system.refresh_cpu();
        self.system.refresh_memory();
        self.system.refresh_networks();
        self.system.refresh_processes();
        
        // Calculate CPU usage
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        
        // Memory metrics
        let memory_used_mb = (self.system.used_memory() as f64) / 1024.0 / 1024.0;
        let memory_total_mb = (self.system.total_memory() as f64) / 1024.0 / 1024.0;
        
        // Network metrics
        let (rx_kb, tx_kb) = self.system.networks().iter()
            .fold((0.0, 0.0), |(rx, tx), (_, network)| {
                (
                    rx + (network.received() as f64) / 1024.0,
                    tx + (network.transmitted() as f64) / 1024.0,
                )
            });
        
        // Load average (simulated for non-Unix systems)
        let load_avg = self.system.load_average();
        let load_average = [load_avg.one, load_avg.five, load_avg.fifteen];
        
        // Create metrics
        let metrics = SystemMetrics {
            timestamp: Utc::now(),
            cpu_usage,
            memory_used_mb,
            memory_total_mb,
            network_rx_kb: rx_kb,
            network_tx_kb: tx_kb,
            process_count: self.system.processes().len(),
            load_average,
        };
        
        // Update history
        self.metrics_history.push_back(metrics.clone());
        if self.metrics_history.len() > GRAPH_POINTS {
            self.metrics_history.pop_front();
        }
        
        self.current_metrics = Some(metrics);
        
        // Update process list
        self.update_processes();
    }
    
    fn update_processes(&mut self) {
        self.processes.clear();
        
        for (pid, process) in self.system.processes() {
            self.processes.push(ProcessInfo {
                name: process.name().to_string(),
                pid: pid.as_u32(),
                cpu_usage: process.cpu_usage(),
                memory_mb: (process.memory() as f64) / 1024.0 / 1024.0,
                status: format!("{:?}", process.status()),
            });
        }
        
        self.sort_processes();
    }
    
    fn sort_processes(&mut self) {
        self.processes.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Cpu => a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap(),
                SortColumn::Memory => a.memory_mb.partial_cmp(&b.memory_mb).unwrap(),
                SortColumn::Network => std::cmp::Ordering::Equal, // Not implemented per-process
            };
            
            if self.sort_ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    }
    
    fn export_metrics(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.metrics_history) {
            let filename = format!("performance_metrics_{}.json", Utc::now().format("%Y%m%d_%H%M%S"));
            if let Err(e) = std::fs::write(&filename, json) {
                eprintln!("Failed to export metrics: {}", e);
            } else {
                println!("Metrics exported to {}", filename);
            }
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        let header = self.view_header();
        let overview = self.view_overview();
        let graphs = self.view_graphs();
        let process_list = self.view_process_list();
        
        let content = column![
            header,
            overview,
            graphs,
            process_list,
        ]
        .spacing(10);
        
        container(content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    fn view_header(&self) -> Element<Message> {
        container(
            row![
                text("🖥️ Performance Monitor").size(24),
                Space::with_width(Length::Fill),
                button("Clear History").on_press(Message::ClearHistory),
                button("Export").on_press(Message::ExportMetrics),
                button("Close").on_press(Message::Close),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        )
        .padding(10)
        .style(container::rounded_box)
        .into()
    }
    
    fn view_overview(&self) -> Element<Message> {
        if let Some(metrics) = &self.current_metrics {
            let cpu_color = if metrics.cpu_usage > 80.0 {
                Color::from_rgb(1.0, 0.2, 0.2)
            } else if metrics.cpu_usage > 50.0 {
                Color::from_rgb(1.0, 0.8, 0.0)
            } else {
                Color::from_rgb(0.2, 0.8, 0.2)
            };
            
            let memory_percent = (metrics.memory_used_mb / metrics.memory_total_mb) * 100.0;
            let memory_color = if memory_percent > 80.0 {
                Color::from_rgb(1.0, 0.2, 0.2)
            } else if memory_percent > 50.0 {
                Color::from_rgb(1.0, 0.8, 0.0)
            } else {
                Color::from_rgb(0.2, 0.8, 0.2)
            };
            
            container(
                row![
                    // CPU Usage
                    container(
                        column![
                            text("CPU Usage").size(16),
                            text(format!("{:.1}%", metrics.cpu_usage))
                                .size(32)
                                .color(cpu_color),
                            progress_bar(0.0..=100.0, metrics.cpu_usage)
                                .height(Length::Fixed(10.0)),
                        ]
                        .spacing(5)
                    )
                    .width(Length::FillPortion(1))
                    .padding(10),
                    
                    // Memory Usage
                    container(
                        column![
                            text("Memory Usage").size(16),
                            text(format!("{:.1} / {:.1} GB", 
                                metrics.memory_used_mb / 1024.0,
                                metrics.memory_total_mb / 1024.0))
                                .size(20),
                            text(format!("{:.1}%", memory_percent))
                                .size(16)
                                .color(memory_color),
                            progress_bar(0.0..=100.0, memory_percent as f32)
                                .height(Length::Fixed(10.0)),
                        ]
                        .spacing(5)
                    )
                    .width(Length::FillPortion(1))
                    .padding(10),
                    
                    // Network
                    container(
                        column![
                            text("Network").size(16),
                            text(format!("↓ {:.1} KB/s", metrics.network_rx_kb))
                                .size(14)
                                .color(Color::from_rgb(0.2, 0.8, 1.0)),
                            text(format!("↑ {:.1} KB/s", metrics.network_tx_kb))
                                .size(14)
                                .color(Color::from_rgb(1.0, 0.6, 0.2)),
                        ]
                        .spacing(5)
                    )
                    .width(Length::FillPortion(1))
                    .padding(10),
                    
                    // System Info
                    container(
                        column![
                            text("System").size(16),
                            text(format!("{} processes", metrics.process_count))
                                .size(14),
                            text(format!("Load: {:.2} {:.2} {:.2}", 
                                metrics.load_average[0],
                                metrics.load_average[1],
                                metrics.load_average[2]))
                                .size(12)
                                .color(Color::from_rgb(0.7, 0.7, 0.7)),
                        ]
                        .spacing(5)
                    )
                    .width(Length::FillPortion(1))
                    .padding(10),
                ]
                .spacing(10)
            )
            .style(container::rounded_box)
            .into()
        } else {
            container(
                text("Collecting metrics...")
                    .size(16)
                    .color(Color::from_rgb(0.6, 0.6, 0.6))
            )
            .center_x(Length::Fill)
            .center_y(Length::Fixed(100.0))
            .style(container::rounded_box)
            .into()
        }
    }
    
    fn view_graphs(&self) -> Element<Message> {
        container(
            Canvas::new(self)
                .width(Length::Fill)
                .height(Length::Fixed(200.0))
        )
        .style(container::rounded_box)
        .into()
    }
    
    fn view_process_list(&self) -> Element<Message> {
        let header = row![
            button(text("Process").width(Length::FillPortion(3)))
                .on_press(Message::SortBy(SortColumn::Name))
                .style(button::text),
            button(text("CPU %").width(Length::FillPortion(1)))
                .on_press(Message::SortBy(SortColumn::Cpu))
                .style(button::text),
            button(text("Memory").width(Length::FillPortion(1)))
                .on_press(Message::SortBy(SortColumn::Memory))
                .style(button::text),
            text("Status").width(Length::FillPortion(1)),
        ]
        .spacing(10);
        
        let mut process_list = column![header].spacing(5);
        
        // Show top 20 processes
        for process in self.processes.iter().take(20) {
            let is_selected = self.selected_processes.contains(&process.name);
            
            let row = button(
                row![
                    text(&process.name).width(Length::FillPortion(3)),
                    text(format!("{:.1}", process.cpu_usage)).width(Length::FillPortion(1)),
                    text(format!("{:.1} MB", process.memory_mb)).width(Length::FillPortion(1)),
                    text(&process.status).width(Length::FillPortion(1)),
                ]
                .spacing(10)
            )
            .on_press(Message::ToggleProcess(process.name.clone()))
            .style(if is_selected { button::primary } else { button::text })
            .width(Length::Fill);
            
            process_list = process_list.push(row);
        }
        
        container(
            scrollable(process_list.padding(10))
                .height(Length::Fill)
        )
        .style(container::rounded_box)
        .into()
    }
    
    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(UPDATE_INTERVAL_MS))
            .map(|_| Message::UpdateMetrics)
    }
}

impl<Message> canvas::Program<Message> for PerformanceMonitor {
    type State = ();
    
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        if self.metrics_history.len() < 2 {
            return vec![frame.into_geometry()];
        }
        
        let padding = 20.0;
        let graph_width = bounds.width - 2.0 * padding;
        let graph_height = bounds.height - 2.0 * padding;
        
        // Draw grid
        let grid_color = Color::from_rgba(0.3, 0.3, 0.3, 0.5);
        for i in 0..=4 {
            let y = padding + (i as f32 / 4.0) * graph_height;
            frame.stroke(
                &Path::line(
                    iced::Point::new(padding, y),
                    iced::Point::new(padding + graph_width, y),
                ),
                Stroke::default().with_color(grid_color).with_width(1.0),
            );
        }
        
        // Draw CPU usage graph
        let cpu_points: Vec<iced::Point> = self.metrics_history
            .iter()
            .enumerate()
            .map(|(i, metrics)| {
                let x = padding + (i as f32 / (GRAPH_POINTS - 1) as f32) * graph_width;
                let y = padding + graph_height - (metrics.cpu_usage / 100.0) * graph_height;
                iced::Point::new(x, y)
            })
            .collect();
        
        if cpu_points.len() > 1 {
            let mut cpu_path = canvas::path::Builder::new();
            cpu_path.move_to(cpu_points[0]);
            for point in &cpu_points[1..] {
                cpu_path.line_to(*point);
            }
            
            frame.stroke(
                &cpu_path.build(),
                Stroke::default()
                    .with_color(Color::from_rgb(0.2, 0.8, 1.0))
                    .with_width(2.0),
            );
        }
        
        // Draw memory usage graph
        let memory_points: Vec<iced::Point> = self.metrics_history
            .iter()
            .enumerate()
            .map(|(i, metrics)| {
                let x = padding + (i as f32 / (GRAPH_POINTS - 1) as f32) * graph_width;
                let memory_percent = (metrics.memory_used_mb / metrics.memory_total_mb) as f32;
                let y = padding + graph_height - memory_percent * graph_height;
                iced::Point::new(x, y)
            })
            .collect();
        
        if memory_points.len() > 1 {
            let mut memory_path = canvas::path::Builder::new();
            memory_path.move_to(memory_points[0]);
            for point in &memory_points[1..] {
                memory_path.line_to(*point);
            }
            
            frame.stroke(
                &memory_path.build(),
                Stroke::default()
                    .with_color(Color::from_rgb(0.8, 0.2, 0.8))
                    .with_width(2.0),
            );
        }
        
        // Draw legend
        frame.fill_text(canvas::Text {
            content: "CPU".to_string(),
            position: iced::Point::new(padding + 10.0, padding + 10.0),
            color: Color::from_rgb(0.2, 0.8, 1.0),
            size: 14.0.into(),
            font: iced::Font::default(),
            horizontal_alignment: iced::alignment::Horizontal::Left,
            vertical_alignment: iced::alignment::Vertical::Top,
            line_height: iced::widget::text::LineHeight::default(),
            shaping: iced::widget::text::Shaping::Basic,
        });
        
        frame.fill_text(canvas::Text {
            content: "Memory".to_string(),
            position: iced::Point::new(padding + 60.0, padding + 10.0),
            color: Color::from_rgb(0.8, 0.2, 0.8),
            size: 14.0.into(),
            font: iced::Font::default(),
            horizontal_alignment: iced::alignment::Horizontal::Left,
            vertical_alignment: iced::alignment::Vertical::Top,
            line_height: iced::widget::text::LineHeight::default(),
            shaping: iced::widget::text::Shaping::Basic,
        });
        
        vec![frame.into_geometry()]
    }
}

pub async fn run_performance_monitor() -> Result<()> {
    println!("Starting Performance Monitor...");
    
    iced::application(
        PerformanceMonitor::title,
        PerformanceMonitor::update,
        PerformanceMonitor::view
    )
    .subscription(PerformanceMonitor::subscription)
    .window(window::Settings {
        size: iced::Size::new(1000.0, 800.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(PerformanceMonitor::new)
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}