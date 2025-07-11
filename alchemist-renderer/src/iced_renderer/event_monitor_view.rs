//! Event monitoring view for Iced renderer

use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Column, Container, Row},
    Alignment, Application, Command, Element, Length, Settings, Theme,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use chrono::{DateTime, Utc};

/// Event data structure for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDisplay {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub domain: String,
    pub event_type: String,
    pub severity: String,
    pub subject: String,
    pub correlation_id: Option<String>,
    pub payload: serde_json::Value,
}

/// Event monitor application state
#[derive(Debug)]
pub struct EventMonitorApp {
    /// Title of the window
    title: String,
    /// Events to display
    events: VecDeque<EventDisplay>,
    /// Maximum number of events to display
    max_events: usize,
    /// Filter input value
    filter_input: String,
    /// Current filter
    current_filter: Option<String>,
    /// Auto-scroll to bottom
    auto_scroll: bool,
    /// Selected event for details
    selected_event: Option<usize>,
}

/// Messages for the event monitor
#[derive(Debug, Clone)]
pub enum Message {
    /// New event received
    EventReceived(EventDisplay),
    /// Filter input changed
    FilterChanged(String),
    /// Apply filter
    ApplyFilter,
    /// Clear filter
    ClearFilter,
    /// Toggle auto-scroll
    ToggleAutoScroll,
    /// Select event for details
    SelectEvent(usize),
    /// Clear selection
    ClearSelection,
    /// Clear all events
    ClearEvents,
}

impl EventMonitorApp {
    /// Create a new event monitor app
    pub fn new(title: String, max_events: usize) -> Self {
        Self {
            title,
            events: VecDeque::with_capacity(max_events),
            max_events,
            filter_input: String::new(),
            current_filter: None,
            auto_scroll: true,
            selected_event: None,
        }
    }

    /// Add a new event
    pub fn add_event(&mut self, event: EventDisplay) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Check if event matches filter
    fn matches_filter(&self, event: &EventDisplay) -> bool {
        if let Some(filter) = &self.current_filter {
            let filter_lower = filter.to_lowercase();
            event.domain.to_lowercase().contains(&filter_lower) ||
            event.event_type.to_lowercase().contains(&filter_lower) ||
            event.severity.to_lowercase().contains(&filter_lower) ||
            event.subject.to_lowercase().contains(&filter_lower)
        } else {
            true
        }
    }

    /// Get filtered events
    fn filtered_events(&self) -> Vec<(usize, &EventDisplay)> {
        self.events
            .iter()
            .enumerate()
            .filter(|(_, event)| self.matches_filter(event))
            .collect()
    }
}

impl Application for EventMonitorApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = (String, usize);

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            EventMonitorApp::new(flags.0, flags.1),
            Command::none()
        )
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::EventReceived(event) => {
                self.add_event(event);
            }
            Message::FilterChanged(value) => {
                self.filter_input = value;
            }
            Message::ApplyFilter => {
                if !self.filter_input.is_empty() {
                    self.current_filter = Some(self.filter_input.clone());
                }
            }
            Message::ClearFilter => {
                self.current_filter = None;
                self.filter_input.clear();
            }
            Message::ToggleAutoScroll => {
                self.auto_scroll = !self.auto_scroll;
            }
            Message::SelectEvent(index) => {
                self.selected_event = Some(index);
            }
            Message::ClearSelection => {
                self.selected_event = None;
            }
            Message::ClearEvents => {
                self.events.clear();
                self.selected_event = None;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let header = row![
            text("Event Monitor").size(24),
            text(format!("{} events", self.filtered_events().len())).size(16),
        ]
        .spacing(20)
        .align_items(Alignment::Center);

        let filter_row = row![
            text_input("Filter events...", &self.filter_input)
                .on_input(Message::FilterChanged)
                .on_submit(Message::ApplyFilter)
                .padding(10)
                .width(Length::Fill),
            button("Apply")
                .on_press(Message::ApplyFilter)
                .padding(10),
            button("Clear")
                .on_press(Message::ClearFilter)
                .padding(10),
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let controls = row![
            button(if self.auto_scroll { "Auto-scroll: ON" } else { "Auto-scroll: OFF" })
                .on_press(Message::ToggleAutoScroll)
                .padding(10),
            button("Clear All")
                .on_press(Message::ClearEvents)
                .padding(10),
        ]
        .spacing(10);

        // Event list
        let mut event_list = Column::new().spacing(5);
        
        for (idx, event) in self.filtered_events() {
            let severity_color = match event.severity.as_str() {
                "CRITICAL" => [1.0, 0.0, 0.0],
                "ERROR" => [1.0, 0.5, 0.0],
                "WARNING" => [1.0, 1.0, 0.0],
                "INFO" => [0.0, 0.8, 0.0],
                _ => [0.5, 0.5, 0.5],
            };

            let event_row = container(
                row![
                    text(format!("[{}]", event.severity))
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(
                            severity_color[0],
                            severity_color[1],
                            severity_color[2]
                        ))),
                    text(event.timestamp.format("%H:%M:%S").to_string()).size(12),
                    text(&event.domain).size(12),
                    text(&event.event_type).size(12),
                    text(&event.subject).size(12).width(Length::Fill),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            )
            .padding(5);

            let event_button = button(event_row)
                .on_press(Message::SelectEvent(idx))
                .style(if Some(idx) == self.selected_event {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .width(Length::Fill);

            event_list = event_list.push(event_button);
        }

        let event_scroll = scrollable(event_list)
            .height(Length::FillPortion(2));

        // Event details panel
        let details_panel = if let Some(idx) = self.selected_event {
            if let Some((_, event)) = self.filtered_events().into_iter().find(|(i, _)| *i == idx) {
                let details = column![
                    row![
                        text("Event Details").size(18),
                        button("X")
                            .on_press(Message::ClearSelection)
                            .padding(5),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    text(format!("ID: {}", event.id)).size(12),
                    text(format!("Timestamp: {}", event.timestamp.to_rfc3339())).size(12),
                    text(format!("Domain: {}", event.domain)).size(12),
                    text(format!("Type: {}", event.event_type)).size(12),
                    text(format!("Severity: {}", event.severity)).size(12),
                    text(format!("Subject: {}", event.subject)).size(12),
                    text(format!("Correlation: {}", 
                        event.correlation_id.as_ref().unwrap_or(&"N/A".to_string())
                    )).size(12),
                    text("Payload:").size(12),
                    scrollable(
                        text(serde_json::to_string_pretty(&event.payload).unwrap_or_default())
                            .size(10)
                            .font(iced::Font::MONOSPACE)
                    )
                    .height(Length::Fill),
                ]
                .spacing(5);

                container(details)
                    .padding(10)
                    .style(iced::theme::Container::Box)
                    .height(Length::FillPortion(1))
            } else {
                container(text("No event selected"))
                    .height(Length::FillPortion(1))
            }
        } else {
            container(text("Select an event to view details"))
                .center_x()
                .center_y()
                .height(Length::FillPortion(1))
        };

        let content = column![
            header,
            filter_row,
            controls,
            event_scroll,
            details_panel,
        ]
        .spacing(10)
        .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

/// Launch the event monitor window
pub fn launch_event_monitor(title: String, max_events: usize) -> iced::Result {
    EventMonitorApp::run(Settings {
        window: iced::window::Settings {
            size: (1200, 800),
            position: iced::window::Position::Centered,
            ..Default::default()
        },
        flags: (title, max_events),
        ..Default::default()
    })
}