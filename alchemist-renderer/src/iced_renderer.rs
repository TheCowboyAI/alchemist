//! Iced 2D renderer implementation

use anyhow::Result;
use iced::{
    widget::{button, column, container, row, scrollable, text, text_editor},
    window, executor, Element, Length, Settings, Theme, Alignment,
};
use iced::application::{Application, Update};
use alchemist::renderer::{RenderRequest, RenderData};

mod dashboard_app;
use dashboard_app::DashboardApp;

mod rss_feed_view;
use rss_feed_view::{RssFeedView, RssFeedMessage};

mod dialog_ui_simple;
use dialog_ui_simple::{DialogUI, DialogConfig};

pub fn run(request: RenderRequest) -> Result<()> {
    let settings = Settings {
        window: window::Settings {
            size: (request.config.width, request.config.height),
            resizable: request.config.resizable,
            decorations: true,
            ..Default::default()
        },
        ..Default::default()
    };
    
    match &request.data {
        RenderData::Dialog { dialog_id, ai_model, messages, system_prompt } => {
            let config = DialogConfig {
                dialog_id: dialog_id.clone(),
                ai_model: ai_model.clone(),
                messages: messages.clone(),
                system_prompt: system_prompt.clone(),
            };
            DialogUI::run(settings.with_flags(config))?;
        }
        RenderData::Document { content, format } => {
            DocumentViewer::run(settings)?;
        }
        RenderData::TextEditor { file_path, content, language } => {
            TextEditorApp::run(settings)?;
        }
        RenderData::Custom { data } => {
            // Check if it's a dashboard
            if let Some(data_type) = data.get("type").and_then(|v| v.as_str()) {
                if data_type == "dashboard" {
                    if let Some(dashboard_data) = data.get("data") {
                        let dashboard: alchemist::dashboard::DashboardData = 
                            serde_json::from_value(dashboard_data.clone())?;
                        DashboardApp::run(settings.with_flags(dashboard))?;
                        return Ok(());
                    }
                }
            }
            // Default viewer for other custom types
            GenericViewer::run(settings)?;
        }
        _ => {
            // Default viewer for other types
            GenericViewer::run(settings)?;
        }
    }
    
    Ok(())
}

// Document Viewer Application
struct DocumentViewer {
    content: String,
    scroll_offset: f32,
}

#[derive(Debug, Clone)]
enum DocumentMessage {
    ScrollChanged(f32),
}

impl Application for DocumentViewer {
    type Executor = iced::executor::Default;
    type Message = DocumentMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                content: "# Document Viewer\n\nThis is a placeholder document viewer.".to_string(),
                scroll_offset: 0.0,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        "Document Viewer".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            DocumentMessage::ScrollChanged(offset) => {
                self.scroll_offset = offset;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let content_column = column![
            text(&self.content).size(16),
        ]
        .spacing(10)
        .padding(20);

        container(
            scrollable(content_column)
                .height(Length::Fill)
                .width(Length::Fill)
        )
        .padding(10)
        .center_x()
        .into()
    }
}

// Text Editor Application
struct TextEditorApp {
    content: text_editor::Content,
    file_path: Option<String>,
    is_modified: bool,
}

#[derive(Debug, Clone)]
enum EditorMessage {
    Edit(text_editor::Action),
    Save,
    New,
}

impl Application for TextEditorApp {
    type Executor = iced::executor::Default;
    type Message = EditorMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                content: text_editor::Content::new(),
                file_path: None,
                is_modified: false,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        let modified = if self.is_modified { " *" } else { "" };
        match &self.file_path {
            Some(path) => format!("Text Editor - {}{}", path, modified),
            None => format!("Text Editor - Untitled{}", modified),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            EditorMessage::Edit(action) => {
                self.content.edit(action);
                self.is_modified = true;
            }
            EditorMessage::Save => {
                // In a real implementation, save the file
                self.is_modified = false;
            }
            EditorMessage::New => {
                self.content = text_editor::Content::new();
                self.file_path = None;
                self.is_modified = false;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let toolbar = row![
            button("New").on_press(EditorMessage::New),
            button("Save").on_press(EditorMessage::Save),
        ]
        .spacing(10)
        .padding(10);

        let editor = text_editor(&self.content)
            .on_edit(EditorMessage::Edit)
            .height(Length::Fill);

        column![toolbar, editor]
            .into()
    }
}

// Generic Viewer for other data types
struct GenericViewer {
    title: String,
    data: String,
}

#[derive(Debug, Clone)]
enum GenericMessage {
    Close,
}

impl Application for GenericViewer {
    type Executor = iced::executor::Default;
    type Message = GenericMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                title: "Alchemist Viewer".to_string(),
                data: "Generic data viewer".to_string(),
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            GenericMessage::Close => {
                // Close window
                window::close(window::Id::MAIN)
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        container(
            column![
                text(&self.title).size(24),
                text(&self.data).size(16),
                button("Close").on_press(GenericMessage::Close),
            ]
            .spacing(20)
            .align_x(Alignment::Center)
        )
        .padding(20)
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}