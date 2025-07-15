//! Alchemist UI Application
//!
//! A simple UI for managing domains in the Alchemist system.

use iced::{
    widget::{button, column, container, row, text, text_input, Column},
    Element, Length, Theme, Task, window,
};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

fn main() -> iced::Result {
    iced::application("Alchemist - Domain Management UI", AlchemistApp::update, AlchemistApp::view)
        .window(window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            position: window::Position::Centered,
            ..Default::default()
        })
        .theme(|_| Theme::Dark)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    // Tab navigation
    SelectTab(Tab),
    
    // Document operations
    DocumentTitleChanged(String),
    DocumentContentChanged(String),
    CreateDocument,
    SelectDocument(Uuid),
    DeleteDocument(Uuid),
    
    // Generic input
    InputChanged(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Overview,
    Documents,
    Workflows,
    Graph,
}

#[derive(Debug, Clone)]
struct Document {
    id: Uuid,
    title: String,
    content: String,
    created_at: DateTime<Utc>,
}

struct AlchemistApp {
    current_tab: Tab,
    
    // Document management
    documents: HashMap<Uuid, Document>,
    document_title: String,
    document_content: String,
    selected_document: Option<Uuid>,
    
    // Generic input
    input_value: String,
}

impl Default for AlchemistApp {
    fn default() -> Self {
        Self {
            current_tab: Tab::Overview,
            documents: HashMap::new(),
            document_title: String::new(),
            document_content: String::new(),
            selected_document: None,
            input_value: String::new(),
        }
    }
}

impl AlchemistApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectTab(tab) => {
                self.current_tab = tab;
            }
            
            Message::DocumentTitleChanged(title) => {
                self.document_title = title;
            }
            
            Message::DocumentContentChanged(content) => {
                self.document_content = content;
            }
            
            Message::CreateDocument => {
                if !self.document_title.is_empty() {
                    let doc = Document {
                        id: Uuid::new_v4(),
                        title: self.document_title.clone(),
                        content: self.document_content.clone(),
                        created_at: Utc::now(),
                    };
                    self.documents.insert(doc.id, doc);
                    self.document_title.clear();
                    self.document_content.clear();
                }
            }
            
            Message::SelectDocument(id) => {
                self.selected_document = Some(id);
                if let Some(doc) = self.documents.get(&id) {
                    self.document_title = doc.title.clone();
                    self.document_content = doc.content.clone();
                }
            }
            
            Message::DeleteDocument(id) => {
                self.documents.remove(&id);
                if self.selected_document == Some(id) {
                    self.selected_document = None;
                    self.document_title.clear();
                    self.document_content.clear();
                }
            }
            
            Message::InputChanged(value) => {
                self.input_value = value;
            }
        }
        
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let navbar = row![
            button("Overview").on_press(Message::SelectTab(Tab::Overview)),
            button("Documents").on_press(Message::SelectTab(Tab::Documents)),
            button("Workflows").on_press(Message::SelectTab(Tab::Workflows)),
            button("Graph").on_press(Message::SelectTab(Tab::Graph)),
        ]
        .spacing(10)
        .padding(10);
        
        let content = match self.current_tab {
            Tab::Overview => self.view_overview(),
            Tab::Documents => self.view_documents(),
            Tab::Workflows => self.view_workflows(),
            Tab::Graph => self.view_graph(),
        };
        
        container(
            column![
                container(navbar),
                container(content).padding(20),
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    fn view_overview(&self) -> Element<Message> {
        column![
            text("Alchemist System Overview").size(30),
            text("").size(16),
            text("Welcome to the Alchemist Domain Management System").size(16),
            text("").size(16),
            column![
                text(format!("Total Documents: {}", self.documents.len())),
                text("Total Workflows: 0"),
                text("Active Graphs: 0"),
            ]
            .spacing(10),
        ]
        .spacing(20)
        .into()
    }
    
    fn view_documents(&self) -> Element<Message> {
        let mut document_list = Column::new()
            .push(text("Document List:").size(20));
            
        if self.documents.is_empty() {
            document_list = document_list.push(text("No documents yet"));
        }
        
        let mut list = Column::new().spacing(5);
        for (id, doc) in &self.documents {
            list = list.push(
                row![
                    button(text(&doc.title))
                        .on_press(Message::SelectDocument(*id))
                        .width(Length::Fill),
                    button("Delete")
                        .on_press(Message::DeleteDocument(*id)),
                ]
                .spacing(10)
            );
        }
        
        row![
            // Form column
            column![
                text("Create/Edit Document").size(24),
                text("Title:"),
                text_input("Enter title...", &self.document_title)
                    .on_input(Message::DocumentTitleChanged)
                    .padding(10),
                text("Content:"),
                text_input("Enter content...", &self.document_content)
                    .on_input(Message::DocumentContentChanged)
                    .padding(10),
                button("Create Document")
                    .on_press(Message::CreateDocument)
                    .padding(10),
            ]
            .spacing(10)
            .width(Length::FillPortion(2)),
            
            // List column
            column![document_list, list]
                .spacing(10)
                .width(Length::FillPortion(1))
                .padding(20),
        ]
        .spacing(20)
        .into()
    }
    
    fn view_workflows(&self) -> Element<Message> {
        column![
            text("Workflow Management").size(30),
            text("Workflow functionality coming soon...").size(16),
        ]
        .spacing(20)
        .into()
    }
    
    fn view_graph(&self) -> Element<Message> {
        column![
            text("Graph Visualization").size(30),
            text("Graph visualization coming soon...").size(16),
        ]
        .spacing(20)
        .into()
    }
}