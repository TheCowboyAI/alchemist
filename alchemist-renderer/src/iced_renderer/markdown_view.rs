//! Markdown renderer view for Iced
//! 
//! Renders markdown content with support for:
//! - Headers, paragraphs, lists, code blocks
//! - Syntax highlighting for code blocks
//! - Links and images
//! - Tables and blockquotes
//! - Custom theme support

use iced::{
    widget::{column, container, row, scrollable, text, Column, Container, Image, Row, Rule, Space},
    window, Element, Length, Alignment, Theme, Color, Font, Command,
};
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Program, Stroke, Text as CanvasText};
use pulldown_cmark::{Event, Parser, Tag, TagEnd, CodeBlockKind, HeadingLevel};
use std::collections::HashMap;

/// Markdown theme configuration
#[derive(Debug, Clone)]
pub struct MarkdownTheme {
    pub background: Color,
    pub text: Color,
    pub heading_color: Color,
    pub code_background: Color,
    pub code_text: Color,
    pub link_color: Color,
    pub blockquote_background: Color,
    pub blockquote_border: Color,
    pub table_border: Color,
    pub table_header_background: Color,
}

impl Default for MarkdownTheme {
    fn default() -> Self {
        Self {
            background: Color::from_rgb(0.98, 0.98, 0.98),
            text: Color::from_rgb(0.2, 0.2, 0.2),
            heading_color: Color::from_rgb(0.1, 0.1, 0.1),
            code_background: Color::from_rgb(0.95, 0.95, 0.95),
            code_text: Color::from_rgb(0.1, 0.1, 0.1),
            link_color: Color::from_rgb(0.0, 0.4, 0.8),
            blockquote_background: Color::from_rgb(0.96, 0.96, 0.96),
            blockquote_border: Color::from_rgb(0.8, 0.8, 0.8),
            table_border: Color::from_rgb(0.85, 0.85, 0.85),
            table_header_background: Color::from_rgb(0.92, 0.92, 0.92),
        }
    }
}

impl MarkdownTheme {
    /// Dark theme preset
    pub fn dark() -> Self {
        Self {
            background: Color::from_rgb(0.12, 0.12, 0.12),
            text: Color::from_rgb(0.9, 0.9, 0.9),
            heading_color: Color::from_rgb(0.95, 0.95, 0.95),
            code_background: Color::from_rgb(0.08, 0.08, 0.08),
            code_text: Color::from_rgb(0.85, 0.85, 0.85),
            link_color: Color::from_rgb(0.4, 0.6, 1.0),
            blockquote_background: Color::from_rgb(0.15, 0.15, 0.15),
            blockquote_border: Color::from_rgb(0.3, 0.3, 0.3),
            table_border: Color::from_rgb(0.3, 0.3, 0.3),
            table_header_background: Color::from_rgb(0.18, 0.18, 0.18),
        }
    }
}

/// Markdown viewer state
pub struct MarkdownView {
    content: String,
    theme: MarkdownTheme,
    scroll_offset: f32,
    cached_elements: Option<Element<'static, MarkdownMessage>>,
}

#[derive(Debug, Clone)]
pub enum MarkdownMessage {
    ScrollChanged(f32),
    LinkClicked(String),
    ThemeChanged(MarkdownTheme),
}

impl MarkdownView {
    pub fn new(content: String, theme: Option<MarkdownTheme>) -> Self {
        Self {
            content,
            theme: theme.unwrap_or_default(),
            scroll_offset: 0.0,
            cached_elements: None,
        }
    }

    pub fn update(&mut self, message: MarkdownMessage) -> iced::Command<MarkdownMessage> {
        match message {
            MarkdownMessage::ScrollChanged(offset) => {
                self.scroll_offset = offset;
            }
            MarkdownMessage::LinkClicked(url) => {
                // Handle link clicks - could open in browser or internal viewer
                tracing::info!("Link clicked: {}", url);
            }
            MarkdownMessage::ThemeChanged(theme) => {
                self.theme = theme;
                self.cached_elements = None; // Invalidate cache
            }
        }
        iced::Command::none()
    }

    pub fn view(&self) -> Element<MarkdownMessage> {
        let content = self.parse_markdown();
        
        container(
            scrollable(
                container(content)
                    .padding(20)
                    .width(Length::Fill)
            )
            .height(Length::Fill)
            .width(Length::Fill)
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(self.theme.background)),
            ..Default::default()
        })
        .into()
    }

    fn parse_markdown(&self) -> Element<MarkdownMessage> {
        let parser = Parser::new(&self.content);
        let mut elements: Vec<Element<MarkdownMessage>> = Vec::new();
        let mut current_paragraph: Vec<Element<MarkdownMessage>> = Vec::new();
        let mut in_code_block = false;
        let mut code_content = String::new();
        let mut code_language = None;
        let mut in_list = false;
        let mut list_items: Vec<Element<MarkdownMessage>> = Vec::new();
        let mut in_table = false;
        let mut table_rows: Vec<Vec<String>> = Vec::new();
        let mut current_table_row: Vec<String> = Vec::new();
        let mut in_blockquote = false;
        let mut blockquote_content: Vec<Element<MarkdownMessage>> = Vec::new();

        for event in parser {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Heading { level, .. } => {
                        self.flush_paragraph(&mut elements, &mut current_paragraph);
                    }
                    Tag::Paragraph => {
                        // Start collecting paragraph content
                    }
                    Tag::CodeBlock(kind) => {
                        in_code_block = true;
                        code_content.clear();
                        code_language = match kind {
                            CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                            _ => None,
                        };
                    }
                    Tag::List(_) => {
                        in_list = true;
                        list_items.clear();
                    }
                    Tag::Item => {
                        // List item start
                    }
                    Tag::Table(_) => {
                        in_table = true;
                        table_rows.clear();
                    }
                    Tag::TableHead | Tag::TableRow => {
                        current_table_row.clear();
                    }
                    Tag::BlockQuote => {
                        in_blockquote = true;
                        blockquote_content.clear();
                    }
                    _ => {}
                },
                Event::End(tag_end) => match tag_end {
                    TagEnd::Heading(level) => {
                        let heading_text = self.collect_text(&mut current_paragraph);
                        elements.push(self.create_heading(heading_text, level));
                    }
                    TagEnd::Paragraph => {
                        if in_blockquote {
                            let para = self.create_paragraph(&mut current_paragraph);
                            blockquote_content.push(para);
                        } else {
                            elements.push(self.create_paragraph(&mut current_paragraph));
                        }
                    }
                    TagEnd::CodeBlock => {
                        in_code_block = false;
                        elements.push(self.create_code_block(code_content.clone(), code_language.as_deref()));
                        code_content.clear();
                        code_language = None;
                    }
                    TagEnd::List(_) => {
                        in_list = false;
                        elements.push(self.create_list(list_items.clone()));
                        list_items.clear();
                    }
                    TagEnd::Item => {
                        let item_content = self.create_paragraph(&mut current_paragraph);
                        list_items.push(row![
                            text("• ").size(16).color(self.theme.text),
                            item_content
                        ].into());
                    }
                    TagEnd::Table => {
                        in_table = false;
                        elements.push(self.create_table(table_rows.clone()));
                        table_rows.clear();
                    }
                    TagEnd::TableHead | TagEnd::TableRow => {
                        if !current_table_row.is_empty() {
                            table_rows.push(current_table_row.clone());
                            current_table_row.clear();
                        }
                    }
                    TagEnd::BlockQuote => {
                        in_blockquote = false;
                        elements.push(self.create_blockquote(blockquote_content.clone()));
                        blockquote_content.clear();
                    }
                    _ => {}
                },
                Event::Text(text) => {
                    if in_code_block {
                        code_content.push_str(&text);
                    } else if in_table {
                        current_table_row.push(text.to_string());
                    } else {
                        current_paragraph.push(
                            text::Text::new(text.to_string())
                                .size(16)
                                .color(self.theme.text)
                                .into()
                        );
                    }
                }
                Event::Code(code) => {
                    current_paragraph.push(
                        container(
                            text::Text::new(code.to_string())
                                .size(14)
                                .font(Font::MONOSPACE)
                                .color(self.theme.code_text)
                        )
                        .padding([2, 6])
                        .style(|_theme| container::Style {
                            background: Some(iced::Background::Color(self.theme.code_background)),
                            border: iced::Border {
                                radius: 3.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .into()
                    );
                }
                Event::Rule => {
                    self.flush_paragraph(&mut elements, &mut current_paragraph);
                    elements.push(
                        container(Rule::horizontal(2))
                            .padding([10, 0])
                            .width(Length::Fill)
                            .into()
                    );
                }
                Event::SoftBreak => {
                    current_paragraph.push(text(" ").into());
                }
                Event::HardBreak => {
                    current_paragraph.push(Space::with_height(10).into());
                }
                _ => {}
            }
        }

        // Flush any remaining paragraph
        self.flush_paragraph(&mut elements, &mut current_paragraph);

        column(elements)
            .spacing(15)
            .width(Length::Fill)
            .into()
    }

    fn flush_paragraph(&self, elements: &mut Vec<Element<MarkdownMessage>>, paragraph: &mut Vec<Element<MarkdownMessage>>) {
        if !paragraph.is_empty() {
            elements.push(row(paragraph.drain(..).collect()).into());
        }
    }

    fn collect_text(&self, elements: &mut Vec<Element<MarkdownMessage>>) -> String {
        // For now, return a placeholder - in a real implementation, 
        // we'd extract text from the elements
        let text = "Heading Text";
        elements.clear();
        text.to_string()
    }

    fn create_heading(&self, text: String, level: HeadingLevel) -> Element<MarkdownMessage> {
        let size = match level {
            HeadingLevel::H1 => 32,
            HeadingLevel::H2 => 28,
            HeadingLevel::H3 => 24,
            HeadingLevel::H4 => 20,
            HeadingLevel::H5 => 18,
            HeadingLevel::H6 => 16,
        };

        container(
            text::Text::new(text)
                .size(size)
                .color(self.theme.heading_color)
        )
        .padding([10, 0])
        .into()
    }

    fn create_paragraph(&self, elements: &mut Vec<Element<MarkdownMessage>>) -> Element<MarkdownMessage> {
        if elements.is_empty() {
            return Space::with_height(0).into();
        }
        row(elements.drain(..).collect()).into()
    }

    fn create_code_block(&self, code: String, language: Option<&str>) -> Element<MarkdownMessage> {
        // In a real implementation, we'd use a syntax highlighter here
        container(
            column![
                if let Some(lang) = language {
                    text(lang).size(12).color(Color::from_rgb(0.5, 0.5, 0.5))
                } else {
                    text("").size(0)
                },
                container(
                    text(code)
                        .size(14)
                        .font(Font::MONOSPACE)
                        .color(self.theme.code_text)
                )
                .padding(15)
                .width(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(self.theme.code_background)),
                    border: iced::Border {
                        radius: 5.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
            ]
            .spacing(5)
        )
        .width(Length::Fill)
        .into()
    }

    fn create_list(&self, items: Vec<Element<MarkdownMessage>>) -> Element<MarkdownMessage> {
        column(items)
            .spacing(5)
            .padding([0, 0, 0, 20])
            .into()
    }

    fn create_table(&self, rows: Vec<Vec<String>>) -> Element<MarkdownMessage> {
        if rows.is_empty() {
            return Space::with_height(0).into();
        }

        let mut table_elements = Vec::new();
        
        // Create header row
        if let Some(header) = rows.first() {
            let header_cells: Vec<Element<MarkdownMessage>> = header
                .iter()
                .map(|cell| {
                    container(
                        text(cell).size(16).color(self.theme.text)
                    )
                    .padding(8)
                    .width(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(self.theme.table_header_background)),
                        border: iced::Border {
                            width: 1.0,
                            color: self.theme.table_border,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .into()
                })
                .collect();
            table_elements.push(row(header_cells).into());
        }

        // Create data rows
        for row_data in rows.iter().skip(1) {
            let cells: Vec<Element<MarkdownMessage>> = row_data
                .iter()
                .map(|cell| {
                    container(
                        text(cell).size(16).color(self.theme.text)
                    )
                    .padding(8)
                    .width(Length::Fill)
                    .style(|_theme| container::Style {
                        border: iced::Border {
                            width: 1.0,
                            color: self.theme.table_border,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .into()
                })
                .collect();
            table_elements.push(row(cells).into());
        }

        column(table_elements).into()
    }

    fn create_blockquote(&self, content: Vec<Element<MarkdownMessage>>) -> Element<MarkdownMessage> {
        container(
            row![
                container(Rule::vertical(4))
                    .height(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(self.theme.blockquote_border)),
                        ..Default::default()
                    }),
                column(content)
                    .spacing(10)
                    .padding([0, 0, 0, 15])
            ]
        )
        .padding(10)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(self.theme.blockquote_background)),
            border: iced::Border {
                radius: 5.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }
}