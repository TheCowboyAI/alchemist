//! Domain-based UI Demo
//!
//! This example demonstrates a UI that uses the domain models
//! for Document, Workflow, and Location management.

use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Column, Container},
    Alignment, Command, Element, Length, Settings, Theme,
};
use uuid::Uuid;
use std::collections::HashMap;

// Import domain types
use cim_domain_document::{DocumentAggregate, commands::DocumentCommand, events::DocumentEvent};
use cim_domain_workflow::{WorkflowAggregate, commands::WorkflowCommand, events::WorkflowEvent};
use cim_domain_location::{LocationAggregate, commands::LocationCommand, events::LocationEvent};

fn main() -> iced::Result {
    DomainUI::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    // Tab selection
    SelectTab(Tab),
    
    // Document messages
    DocumentTitleChanged(String),
    DocumentContentChanged(String),
    CreateDocument,
    SelectDocument(Uuid),
    
    // Workflow messages
    WorkflowNameChanged(String),
    WorkflowDescriptionChanged(String),
    CreateWorkflow,
    SelectWorkflow(Uuid),
    AddWorkflowStep,
    
    // Location messages
    LocationNameChanged(String),
    LocationAddressChanged(String),
    CreateLocation,
    SelectLocation(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Documents,
    Workflows,
    Locations,
}

struct DomainUI {
    // Current tab
    current_tab: Tab,
    
    // Document state
    documents: HashMap<Uuid, DocumentAggregate>,
    document_title: String,
    document_content: String,
    selected_document: Option<Uuid>,
    
    // Workflow state
    workflows: HashMap<Uuid, WorkflowAggregate>,
    workflow_name: String,
    workflow_description: String,
    selected_workflow: Option<Uuid>,
    
    // Location state
    locations: HashMap<Uuid, LocationAggregate>,
    location_name: String,
    location_address: String,
    selected_location: Option<Uuid>,
}

impl Default for DomainUI {
    fn default() -> Self {
        Self {
            current_tab: Tab::Documents,
            
            documents: HashMap::new(),
            document_title: String::new(),
            document_content: String::new(),
            selected_document: None,
            
            workflows: HashMap::new(),
            workflow_name: String::new(),
            workflow_description: String::new(),
            selected_workflow: None,
            
            locations: HashMap::new(),
            location_name: String::new(),
            location_address: String::new(),
            selected_location: None,
        }
    }
}

impl DomainUI {
    fn new() -> Self {
        Self::default()
    }
    
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SelectTab(tab) => {
                self.current_tab = tab;
            }
            
            // Document handling
            Message::DocumentTitleChanged(title) => {
                self.document_title = title;
            }
            Message::DocumentContentChanged(content) => {
                self.document_content = content;
            }
            Message::CreateDocument => {
                if !self.document_title.is_empty() {
                    let doc_id = Uuid::new_v4();
                    let mut document = DocumentAggregate::new(doc_id);
                    
                    let cmd = DocumentCommand::CreateDocument {
                        id: doc_id,
                        title: self.document_title.clone(),
                        content: self.document_content.clone(),
                        document_type: cim_domain_document::DocumentType::Markdown,
                        author: "UI User".to_string(),
                        tags: vec![],
                        metadata: Default::default(),
                    };
                    
                    if let Ok(events) = document.handle_command(cmd) {
                        for event in events {
                            document.apply_event(&event);
                        }
                        self.documents.insert(doc_id, document);
                        self.document_title.clear();
                        self.document_content.clear();
                    }
                }
            }
            Message::SelectDocument(id) => {
                self.selected_document = Some(id);
            }
            
            // Workflow handling
            Message::WorkflowNameChanged(name) => {
                self.workflow_name = name;
            }
            Message::WorkflowDescriptionChanged(desc) => {
                self.workflow_description = desc;
            }
            Message::CreateWorkflow => {
                if !self.workflow_name.is_empty() {
                    let workflow_id = Uuid::new_v4();
                    let mut workflow = WorkflowAggregate::new(workflow_id);
                    
                    let cmd = WorkflowCommand::CreateWorkflow {
                        id: workflow_id,
                        name: self.workflow_name.clone(),
                        description: Some(self.workflow_description.clone()),
                        metadata: Default::default(),
                    };
                    
                    if let Ok(events) = workflow.handle_command(cmd) {
                        for event in events {
                            workflow.apply_event(&event);
                        }
                        self.workflows.insert(workflow_id, workflow);
                        self.workflow_name.clear();
                        self.workflow_description.clear();
                    }
                }
            }
            Message::SelectWorkflow(id) => {
                self.selected_workflow = Some(id);
            }
            Message::AddWorkflowStep => {
                // TODO: Implement step addition
            }
            
            // Location handling
            Message::LocationNameChanged(name) => {
                self.location_name = name;
            }
            Message::LocationAddressChanged(addr) => {
                self.location_address = addr;
            }
            Message::CreateLocation => {
                if !self.location_name.is_empty() {
                    let location_id = Uuid::new_v4();
                    let mut location = LocationAggregate::new(location_id);
                    
                    let cmd = LocationCommand::CreateLocation {
                        id: location_id,
                        name: self.location_name.clone(),
                        location_type: cim_domain_location::LocationType::Building,
                        address: Some(cim_domain_location::Address {
                            street: self.location_address.clone(),
                            city: String::new(),
                            state: String::new(),
                            postal_code: String::new(),
                            country: String::new(),
                        }),
                        coordinates: None,
                        metadata: Default::default(),
                    };
                    
                    if let Ok(events) = location.handle_command(cmd) {
                        for event in events {
                            location.apply_event(&event);
                        }
                        self.locations.insert(location_id, location);
                        self.location_name.clear();
                        self.location_address.clear();
                    }
                }
            }
            Message::SelectLocation(id) => {
                self.selected_location = Some(id);
            }
        }
        
        Command::none()
    }
    
    fn view(&self) -> Element<Message> {
        let tabs = row![
            button("Documents")
                .on_press(Message::SelectTab(Tab::Documents))
                .style(if self.current_tab == Tab::Documents {
                    button::primary
                } else {
                    button::secondary
                }),
            button("Workflows")
                .on_press(Message::SelectTab(Tab::Workflows))
                .style(if self.current_tab == Tab::Workflows {
                    button::primary
                } else {
                    button::secondary
                }),
            button("Locations")
                .on_press(Message::SelectTab(Tab::Locations))
                .style(if self.current_tab == Tab::Locations {
                    button::primary
                } else {
                    button::secondary
                }),
        ]
        .spacing(10);
        
        let content = match self.current_tab {
            Tab::Documents => self.view_documents(),
            Tab::Workflows => self.view_workflows(),
            Tab::Locations => self.view_locations(),
        };
        
        container(
            column![
                text("Alchemist Domain UI").size(30),
                tabs,
                container(content)
                    .padding(20)
                    .style(container::bordered_box),
            ]
            .spacing(20)
            .padding(20)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    fn view_documents(&self) -> Element<Message> {
        let document_list: Element<_> = if self.documents.is_empty() {
            text("No documents yet").into()
        } else {
            let items: Vec<Element<Message>> = self.documents
                .iter()
                .map(|(id, doc)| {
                    button(text(&doc.title))
                        .on_press(Message::SelectDocument(*id))
                        .width(Length::Fill)
                        .into()
                })
                .collect();
            
            scrollable(Column::with_children(items).spacing(5))
                .height(Length::Fixed(200.0))
                .into()
        };
        
        column![
            text("Documents").size(24),
            
            row![
                column![
                    text("Title:"),
                    text_input("Document title...", &self.document_title)
                        .on_input(Message::DocumentTitleChanged)
                        .padding(10),
                    
                    text("Content:"),
                    text_input("Document content...", &self.document_content)
                        .on_input(Message::DocumentContentChanged)
                        .padding(10),
                    
                    button("Create Document")
                        .on_press(Message::CreateDocument)
                        .style(button::primary),
                ]
                .spacing(10)
                .width(Length::FillPortion(2)),
                
                column![
                    text("Document List:"),
                    document_list,
                ]
                .spacing(10)
                .width(Length::FillPortion(1)),
            ]
            .spacing(20),
        ]
        .spacing(15)
        .into()
    }
    
    fn view_workflows(&self) -> Element<Message> {
        let workflow_list: Element<_> = if self.workflows.is_empty() {
            text("No workflows yet").into()
        } else {
            let items: Vec<Element<Message>> = self.workflows
                .iter()
                .map(|(id, wf)| {
                    button(text(&wf.name))
                        .on_press(Message::SelectWorkflow(*id))
                        .width(Length::Fill)
                        .into()
                })
                .collect();
            
            scrollable(Column::with_children(items).spacing(5))
                .height(Length::Fixed(200.0))
                .into()
        };
        
        column![
            text("Workflows").size(24),
            
            row![
                column![
                    text("Name:"),
                    text_input("Workflow name...", &self.workflow_name)
                        .on_input(Message::WorkflowNameChanged)
                        .padding(10),
                    
                    text("Description:"),
                    text_input("Workflow description...", &self.workflow_description)
                        .on_input(Message::WorkflowDescriptionChanged)
                        .padding(10),
                    
                    button("Create Workflow")
                        .on_press(Message::CreateWorkflow)
                        .style(button::primary),
                ]
                .spacing(10)
                .width(Length::FillPortion(2)),
                
                column![
                    text("Workflow List:"),
                    workflow_list,
                ]
                .spacing(10)
                .width(Length::FillPortion(1)),
            ]
            .spacing(20),
        ]
        .spacing(15)
        .into()
    }
    
    fn view_locations(&self) -> Element<Message> {
        let location_list: Element<_> = if self.locations.is_empty() {
            text("No locations yet").into()
        } else {
            let items: Vec<Element<Message>> = self.locations
                .iter()
                .map(|(id, loc)| {
                    button(text(&loc.name))
                        .on_press(Message::SelectLocation(*id))
                        .width(Length::Fill)
                        .into()
                })
                .collect();
            
            scrollable(Column::with_children(items).spacing(5))
                .height(Length::Fixed(200.0))
                .into()
        };
        
        column![
            text("Locations").size(24),
            
            row![
                column![
                    text("Name:"),
                    text_input("Location name...", &self.location_name)
                        .on_input(Message::LocationNameChanged)
                        .padding(10),
                    
                    text("Address:"),
                    text_input("Location address...", &self.location_address)
                        .on_input(Message::LocationAddressChanged)
                        .padding(10),
                    
                    button("Create Location")
                        .on_press(Message::CreateLocation)
                        .style(button::primary),
                ]
                .spacing(10)
                .width(Length::FillPortion(2)),
                
                column![
                    text("Location List:"),
                    location_list,
                ]
                .spacing(10)
                .width(Length::FillPortion(1)),
            ]
            .spacing(20),
        ]
        .spacing(15)
        .into()
    }
}

// Implement iced Application trait
impl iced::Application for DomainUI {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::new(), Command::none())
    }

    fn title(&self) -> String {
        "Alchemist Domain UI".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        self.update(message)
    }

    fn view(&self) -> Element<Message> {
        self.view()
    }
}