//! Performance dashboard UI for Alchemist

use iced::{
    widget::{button, column, container, row, scrollable, text, Column, Container, Row, Space},
    Alignment, Command, Element, Length, Subscription, Theme,
    time,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::performance_monitor::{PerformanceMonitor, PerformanceSummary};
use crate::cache::CacheManager;
use std::time::Duration;

pub struct PerformanceDashboard {
    performance_monitor: Arc<PerformanceMonitor>,
    cache_manager: Option<Arc<CacheManager>>,
    current_summary: Option<PerformanceSummary>,
    selected_metric: MetricView,
    refresh_rate: Duration,
    is_paused: bool,
    error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MetricView {
    Overview,
    CacheStats,
    RateLimits,
    ModelPerformance,
    SystemResources,
}

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    SummaryUpdated(PerformanceSummary),
    SelectMetric(MetricView),
    TogglePause,
    ClearCache,
    ExportMetrics,
    Error(String),
}

impl PerformanceDashboard {
    pub fn new(performance_monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            performance_monitor,
            cache_manager: None,
            current_summary: None,
            selected_metric: MetricView::Overview,
            refresh_rate: Duration::from_secs(1),
            is_paused: false,
            error_message: None,
        }
    }
    
    pub fn with_cache_manager(mut self, cache_manager: Arc<CacheManager>) -> Self {
        self.cache_manager = Some(cache_manager);
        self
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Refresh => {
                if !self.is_paused {
                    let monitor = self.performance_monitor.clone();
                    Command::perform(
                        async move {
                            monitor.get_summary().await
                        },
                        Message::SummaryUpdated,
                    )
                } else {
                    Command::none()
                }
            }
            Message::SummaryUpdated(summary) => {
                self.current_summary = Some(summary);
                self.error_message = None;
                Command::none()
            }
            Message::SelectMetric(metric) => {
                self.selected_metric = metric;
                Command::none()
            }
            Message::TogglePause => {
                self.is_paused = !self.is_paused;
                Command::none()
            }
            Message::ClearCache => {
                if let Some(cache_manager) = &self.cache_manager {
                    let cache = cache_manager.clone();
                    Command::perform(
                        async move {
                            match cache.cache.clear().await {
                                Ok(_) => Ok("Cache cleared successfully".to_string()),
                                Err(e) => Err(format!("Failed to clear cache: {}", e)),
                            }
                        },
                        |result| match result {
                            Ok(msg) => Message::Error(msg), // Using Error message to show success too
                            Err(err) => Message::Error(err),
                        }
                    )
                } else {
                    self.error_message = Some("No cache manager available".to_string());
                    Command::none()
                }
            }
            Message::ExportMetrics => {
                let monitor = self.performance_monitor.clone();
                Command::perform(
                    async move {
                        match monitor.export_metrics(crate::performance_monitor::ExportFormat::Json).await {
                            Ok(json_data) => {
                                // Save to file
                                let filename = format!("metrics_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                                match tokio::fs::write(&filename, json_data).await {
                                    Ok(_) => Ok(format!("Metrics exported to {}", filename)),
                                    Err(e) => Err(format!("Failed to write file: {}", e)),
                                }
                            }
                            Err(e) => Err(format!("Failed to export metrics: {}", e)),
                        }
                    },
                    |result| match result {
                        Ok(msg) => Message::Error(msg), // Using Error for success message too
                        Err(err) => Message::Error(err),
                    }
                )
            }
            Message::Error(error) => {
                self.error_message = Some(error);
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let title = text("Performance Dashboard")
            .size(24)
            .style(|theme: &Theme| {
                let palette = theme.palette();
                text::Style {
                    color: Some(palette.primary),
                }
            });

        // Control buttons
        let controls = row![
            button(if self.is_paused { "Resume" } else { "Pause" })
                .on_press(Message::TogglePause)
                .style(button::secondary),
            Space::with_width(10),
            button("Clear Cache")
                .on_press(Message::ClearCache)
                .style(button::danger),
            Space::with_width(10),
            button("Export")
                .on_press(Message::ExportMetrics)
                .style(button::primary),
        ]
        .align_items(Alignment::Center);

        // Metric selector tabs
        let tabs = row![
            metric_tab("Overview", MetricView::Overview, self.selected_metric),
            metric_tab("Cache", MetricView::CacheStats, self.selected_metric),
            metric_tab("Rate Limits", MetricView::RateLimits, self.selected_metric),
            metric_tab("Models", MetricView::ModelPerformance, self.selected_metric),
            metric_tab("System", MetricView::SystemResources, self.selected_metric),
        ]
        .spacing(5);

        // Main content based on selected metric
        let content = if let Some(summary) = &self.current_summary {
            match self.selected_metric {
                MetricView::Overview => self.view_overview(summary),
                MetricView::CacheStats => self.view_cache_stats(summary),
                MetricView::RateLimits => self.view_rate_limits(summary),
                MetricView::ModelPerformance => self.view_model_performance(summary),
                MetricView::SystemResources => self.view_system_resources(summary),
            }
        } else {
            column![
                text("Loading performance data...")
                    .size(16)
                    .style(|theme: &Theme| {
                        let palette = theme.palette();
                        text::Style {
                            color: Some(palette.text),
                        }
                    })
            ]
            .into()
        };

        // Error display
        let error_display = if let Some(error) = &self.error_message {
            column![
                text(format!("Error: {}", error))
                    .size(14)
                    .style(|theme: &Theme| {
                        let palette = theme.palette();
                        text::Style {
                            color: Some(palette.danger),
                        }
                    })
            ]
        } else {
            column![]
        };

        let layout = column![
            title,
            Space::with_height(10),
            controls,
            Space::with_height(10),
            tabs,
            Space::with_height(20),
            scrollable(content).height(Length::Fill),
            error_display,
        ]
        .padding(20)
        .spacing(10);

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_overview(&self, summary: &PerformanceSummary) -> Element<Message> {
        let cache_metric = metric_card(
            "Cache Hit Rate",
            &format!("{:.1}%", summary.cache_hit_rate),
            if summary.cache_hit_rate > 80.0 { "🟢" } else if summary.cache_hit_rate > 50.0 { "🟡" } else { "🔴" },
        );

        let rate_limit_metric = metric_card(
            "Rate Limit Violations",
            &summary.rate_limit_violations.to_string(),
            if summary.rate_limit_violations == 0 { "🟢" } else if summary.rate_limit_violations < 10 { "🟡" } else { "🔴" },
        );

        let system_metric = if let Some(resources) = &summary.latest_resources {
            metric_card(
                "Memory Usage",
                &format!("{:.1} MB", resources.memory_mb),
                if resources.memory_mb < 500.0 { "🟢" } else if resources.memory_mb < 1000.0 { "🟡" } else { "🔴" },
            )
        } else {
            metric_card("Memory Usage", "N/A", "⚪")
        };

        let active_models = metric_card(
            "Active Models",
            &summary.model_metrics.len().to_string(),
            "🤖",
        );

        column![
            text("Performance Overview").size(18),
            Space::with_height(10),
            row![
                cache_metric,
                Space::with_width(20),
                rate_limit_metric,
            ],
            Space::with_height(10),
            row![
                system_metric,
                Space::with_width(20),
                active_models,
            ],
        ]
        .into()
    }

    fn view_cache_stats(&self, summary: &PerformanceSummary) -> Element<Message> {
        column![
            text("Cache Statistics").size(18),
            Space::with_height(10),
            stat_row("Hit Rate", &format!("{:.1}%", summary.cache_hit_rate)),
            stat_row("Total Hits", "N/A"), // TODO: Add to summary
            stat_row("Total Misses", "N/A"),
            stat_row("Evictions", "N/A"),
            Space::with_height(20),
            text("Cache Recommendations").size(16),
            Space::with_height(10),
            recommendation_text(summary.cache_hit_rate),
        ]
        .spacing(5)
        .into()
    }

    fn view_rate_limits(&self, summary: &PerformanceSummary) -> Element<Message> {
        column![
            text("Rate Limiting").size(18),
            Space::with_height(10),
            stat_row("Total Violations", &summary.rate_limit_violations.to_string()),
            stat_row("Violations (Last Hour)", "N/A"),
            stat_row("Most Limited User", "N/A"),
            Space::with_height(20),
            text("Rate Limit Status").size(16),
            Space::with_height(10),
            rate_limit_status(summary.rate_limit_violations),
        ]
        .spacing(5)
        .into()
    }

    fn view_model_performance(&self, summary: &PerformanceSummary) -> Element<Message> {
        let mut model_stats = Column::new()
            .push(text("Model Performance").size(18))
            .push(Space::with_height(10));

        for (model_name, metrics) in &summary.model_metrics {
            let success_rate = if metrics.total_requests > 0 {
                (metrics.successful_requests as f64 / metrics.total_requests as f64) * 100.0
            } else {
                0.0
            };

            model_stats = model_stats
                .push(text(model_name).size(16))
                .push(stat_row("Total Requests", &metrics.total_requests.to_string()))
                .push(stat_row("Success Rate", &format!("{:.1}%", success_rate)))
                .push(stat_row("Avg Latency", &format!("{:.0}ms", metrics.average_latency_ms)))
                .push(stat_row("P95 Latency", &format!("{:.0}ms", metrics.p95_latency_ms)))
                .push(stat_row("Total Tokens", &metrics.total_tokens.to_string()))
                .push(Space::with_height(15));
        }

        model_stats.into()
    }

    fn view_system_resources(&self, summary: &PerformanceSummary) -> Element<Message> {
        if let Some(resources) = &summary.latest_resources {
            column![
                text("System Resources").size(18),
                Space::with_height(10),
                stat_row("Memory Usage", &format!("{:.1} MB", resources.memory_mb)),
                stat_row("CPU Usage", &format!("{:.1}%", resources.cpu_percent)),
                stat_row("Open Connections", &resources.open_connections.to_string()),
                Space::with_height(20),
                text("Resource Trends").size(16),
                Space::with_height(10),
                resource_health(resources.memory_mb, resources.cpu_percent),
            ]
            .spacing(5)
            .into()
        } else {
            column![
                text("System Resources").size(18),
                Space::with_height(10),
                text("No resource data available").size(14),
            ]
            .into()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.is_paused {
            Subscription::none()
        } else {
            time::every(self.refresh_rate).map(|_| Message::Refresh)
        }
    }
}

// Helper functions for UI components

fn metric_tab(label: &str, metric: MetricView, selected: MetricView) -> Element<'static, Message> {
    let is_selected = metric == selected;
    
    button(text(label).size(14))
        .on_press(Message::SelectMetric(metric))
        .style(if is_selected {
            button::primary
        } else {
            button::secondary
        })
        .into()
}

fn metric_card(title: &str, value: &str, indicator: &str) -> Element<'static, Message> {
    container(
        column![
            row![
                text(indicator).size(20),
                Space::with_width(10),
                text(title).size(14),
            ],
            Space::with_height(5),
            text(value).size(24),
        ]
        .padding(15)
    )
    .style(|theme: &Theme| {
        let palette = theme.palette();
        container::Style {
            background: Some(palette.background.into()),
            border: iced::Border {
                color: palette.text,
                width: 1.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

fn stat_row(label: &str, value: &str) -> Row<'static, Message> {
    row![
        text(label).size(14),
        Space::with_width(Length::Fill),
        text(value).size(14),
    ]
    .width(300)
}

fn recommendation_text(hit_rate: f64) -> Element<'static, Message> {
    let recommendation = if hit_rate > 80.0 {
        "✅ Cache performance is excellent"
    } else if hit_rate > 50.0 {
        "⚠️ Consider increasing cache TTL or size"
    } else {
        "❌ Cache hit rate is low, review cache keys"
    };
    
    text(recommendation).size(14).into()
}

fn rate_limit_status(violations: u64) -> Element<'static, Message> {
    let status = if violations == 0 {
        "✅ No rate limit violations"
    } else if violations < 10 {
        "⚠️ Minor rate limiting detected"
    } else {
        "❌ Significant rate limiting, consider upgrading"
    };
    
    text(status).size(14).into()
}

fn resource_health(memory_mb: f32, cpu_percent: f32) -> Element<'static, Message> {
    let health = if memory_mb < 500.0 && cpu_percent < 50.0 {
        "✅ System resources healthy"
    } else if memory_mb < 1000.0 && cpu_percent < 80.0 {
        "⚠️ Moderate resource usage"
    } else {
        "❌ High resource usage detected"
    };
    
    text(health).size(14).into()
}