//! Markdown viewer using Iced

use iced::{
    Task, Element, Theme, Length, widget::{
        column, container, text, scrollable, button, row, Space, Column, text_editor,
    },
    Font, font,
};
use alchemist::renderer::RenderData;
use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel};

#[derive(Debug, Clone)]
pub enum Message {
    CloseWindow,
    ThemeToggle,
    ScrollToTop,
    ScrollToBottom,
}

pub struct MarkdownView {
    title: String,
    content: String,
    parsed_content: Vec<MarkdownElement>,
    use_dark_theme: bool,
}

#[derive(Debug, Clone)]
enum MarkdownElement {
    Heading(HeadingLevel, String),
    Paragraph(String),
    CodeBlock(String, Option<String>), // code, language
    InlineCode(String),
    List(Vec<String>, bool), // items, ordered
    Blockquote(String),
    HorizontalRule,
    Link(String, String), // text, url
    Image(String, String), // alt text, url
    Bold(String),
    Italic(String),
}

impl MarkdownView {
    pub fn new(title: String, content: String, theme: Option<String>) -> (Self, Task<Message>) {
        let use_dark_theme = theme.as_deref() != Some("light");
        let parsed_content = parse_markdown(&content);
        
        (
            MarkdownView {
                title,
                content,
                parsed_content,
                use_dark_theme,
            },
            Task::none()
        )
    }

    pub fn title(&self) -> String {
        format!("{} - Markdown Viewer", self.title)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CloseWindow => {
                std::process::exit(0);
            }
            Message::ThemeToggle => {
                self.use_dark_theme = !self.use_dark_theme;
                Task::none()
            }
            Message::ScrollToTop => {
                // TODO: Implement scroll control
                Task::none()
            }
            Message::ScrollToBottom => {
                // TODO: Implement scroll control
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = row![
            text(&self.title).size(24),
            Space::with_width(Length::Fill),
            button(if self.use_dark_theme { "☀️ Light" } else { "🌙 Dark" })
                .on_press(Message::ThemeToggle),
            button("⬆️ Top").on_press(Message::ScrollToTop),
            button("⬇️ Bottom").on_press(Message::ScrollToBottom),
            button("✕ Close").on_press(Message::CloseWindow),
        ]
        .spacing(10)
        .padding(10);

        let mut content_column = Column::new();
        
        for element in &self.parsed_content {
            content_column = match element {
                MarkdownElement::Heading(level, text) => {
                    let size = match level {
                        HeadingLevel::H1 => 32,
                        HeadingLevel::H2 => 28,
                        HeadingLevel::H3 => 24,
                        HeadingLevel::H4 => 20,
                        HeadingLevel::H5 => 18,
                        HeadingLevel::H6 => 16,
                    };
                    content_column.push(
                        container(
                            iced::widget::text(text)
                                .size(size)
                                .font(Font::with_name("Helvetica Bold"))
                        )
                        .padding([10, 0])
                    )
                }
                MarkdownElement::Paragraph(text) => {
                    content_column.push(
                        container(
                            iced::widget::text(text)
                                .size(16)
                        )
                        .padding([5, 0])
                    )
                }
                MarkdownElement::CodeBlock(code, lang) => {
                    let lang_label = if let Some(l) = lang {
                        format!("```{}", l)
                    } else {
                        "```".to_string()
                    };
                    
                    content_column.push(
                        column![
                            text(lang_label)
                                .size(12)
                                .color(if self.use_dark_theme {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                } else {
                                    iced::Color::from_rgb(0.4, 0.4, 0.4)
                                }),
                            container(
                                text(code)
                                    .size(14)
                                    .font(Font::MONOSPACE)
                            )
                            .style(if self.use_dark_theme {
                                container::rounded_box
                            } else {
                                container::bordered_box
                            })
                            .padding(10)
                            .width(Length::Fill),
                        ]
                        .spacing(2)
                    )
                }
                MarkdownElement::InlineCode(code) => {
                    content_column.push(
                        container(
                            text(format!("`{}`", code))
                                .size(16)
                                .font(Font::MONOSPACE)
                        )
                        .padding([5, 0])
                    )
                }
                MarkdownElement::List(items, ordered) => {
                    let mut list_column = Column::new();
                    for (i, item) in items.iter().enumerate() {
                        let prefix = if *ordered {
                            format!("{}. ", i + 1)
                        } else {
                            "• ".to_string()
                        };
                        list_column = list_column.push(
                            row![
                                text(prefix).size(16),
                                text(item).size(16),
                            ]
                            .spacing(5)
                        );
                    }
                    content_column.push(
                        container(list_column.spacing(3))
                            .padding([5, 0, 5, 20])
                    )
                }
                MarkdownElement::Blockquote(text) => {
                    content_column.push(
                        container(
                            row![
                                container("")
                                    .width(Length::Fixed(4.0))
                                    .height(Length::Fill)
                                    .style(container::rounded_box),
                                text(text)
                                    .size(16)
                                    .color(if self.use_dark_theme {
                                        iced::Color::from_rgb(0.7, 0.7, 0.7)
                                    } else {
                                        iced::Color::from_rgb(0.3, 0.3, 0.3)
                                    }),
                            ]
                            .spacing(10)
                        )
                        .padding([5, 0])
                    )
                }
                MarkdownElement::HorizontalRule => {
                    content_column.push(
                        container("")
                            .height(Length::Fixed(1.0))
                            .width(Length::Fill)
                            .style(container::rounded_box)
                            .padding([10, 0])
                    )
                }
                MarkdownElement::Link(text, url) => {
                    content_column.push(
                        container(
                            button(text)
                                .on_press(Message::CloseWindow) // TODO: Open URL
                                .style(button::text)
                        )
                        .padding([5, 0])
                    )
                }
                MarkdownElement::Image(alt, _url) => {
                    content_column.push(
                        container(
                            text(format!("[Image: {}]", alt))
                                .size(14)
                                .color(if self.use_dark_theme {
                                    iced::Color::from_rgb(0.6, 0.6, 0.6)
                                } else {
                                    iced::Color::from_rgb(0.4, 0.4, 0.4)
                                })
                        )
                        .padding([5, 0])
                    )
                }
                MarkdownElement::Bold(text) => {
                    content_column.push(
                        container(
                            iced::widget::text(format!("**{}**", text))
                                .size(16)
                                .font(Font::with_name("Helvetica Bold"))
                        )
                        .padding([5, 0])
                    )
                }
                MarkdownElement::Italic(text) => {
                    content_column.push(
                        container(
                            iced::widget::text(format!("_{}_", text))
                                .size(16)
                                .style(text::italic)
                        )
                        .padding([5, 0])
                    )
                }
            };
        }

        container(
            column![
                container(header)
                    .style(container::rounded_box)
                    .width(Length::Fill),
                scrollable(
                    container(content_column.spacing(5))
                        .padding(20)
                        .width(Length::Fill)
                )
                .height(Length::Fill),
            ]
            .spacing(10)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10)
        .into()
    }

    pub fn theme(&self) -> Theme {
        if self.use_dark_theme {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

fn parse_markdown(content: &str) -> Vec<MarkdownElement> {
    let parser = Parser::new(content);
    let mut elements = Vec::new();
    let mut current_text = String::new();
    let mut in_heading = None;
    let mut in_code_block = false;
    let mut code_lang = None;
    let mut list_items = Vec::new();
    let mut in_list = false;
    let mut list_ordered = false;
    
    for event in parser {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Heading { level, .. } => {
                        in_heading = Some(level);
                    }
                    Tag::CodeBlock(kind) => {
                        in_code_block = true;
                        code_lang = match kind {
                            pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                                if lang.is_empty() {
                                    None
                                } else {
                                    Some(lang.to_string())
                                }
                            }
                            _ => None,
                        };
                    }
                    Tag::List(start) => {
                        in_list = true;
                        list_ordered = start.is_some();
                        list_items.clear();
                    }
                    Tag::Emphasis => {
                        // Start italic
                    }
                    Tag::Strong => {
                        // Start bold
                    }
                    _ => {}
                }
            }
            Event::End(tag_end) => {
                match tag_end {
                    TagEnd::Heading(level) => {
                        if !current_text.is_empty() {
                            elements.push(MarkdownElement::Heading(level, current_text.clone()));
                            current_text.clear();
                        }
                        in_heading = None;
                    }
                    TagEnd::Paragraph => {
                        if !current_text.is_empty() {
                            elements.push(MarkdownElement::Paragraph(current_text.clone()));
                            current_text.clear();
                        }
                    }
                    TagEnd::CodeBlock => {
                        elements.push(MarkdownElement::CodeBlock(current_text.clone(), code_lang.take()));
                        current_text.clear();
                        in_code_block = false;
                    }
                    TagEnd::List(_) => {
                        if !list_items.is_empty() {
                            elements.push(MarkdownElement::List(list_items.clone(), list_ordered));
                            list_items.clear();
                        }
                        in_list = false;
                    }
                    TagEnd::Item => {
                        if !current_text.is_empty() {
                            list_items.push(current_text.clone());
                            current_text.clear();
                        }
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                current_text.push_str(&text);
            }
            Event::Code(code) => {
                elements.push(MarkdownElement::InlineCode(code.to_string()));
            }
            Event::Rule => {
                elements.push(MarkdownElement::HorizontalRule);
            }
            _ => {}
        }
    }
    
    // Handle any remaining text
    if !current_text.is_empty() {
        elements.push(MarkdownElement::Paragraph(current_text));
    }
    
    elements
}

pub fn run_markdown_viewer(title: String, content: String, theme: Option<String>) -> Result<(), iced::Error> {
    iced::application(
        MarkdownView::title,
        MarkdownView::update,
        MarkdownView::view
    )
    .window(iced::window::Settings {
        size: iced::Size::new(800.0, 600.0),
        position: iced::window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(MarkdownView::theme)
    .run_with(|| MarkdownView::new(title, content, theme))
}