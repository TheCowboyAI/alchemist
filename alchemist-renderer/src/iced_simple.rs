//! Simplified Iced renderer for testing

use anyhow::Result;
use iced::Settings;
use alchemist::renderer::{RenderRequest, RenderData};

pub fn run(request: RenderRequest) -> Result<()> {
    match &request.data {
        RenderData::Dialog { dialog_id, ai_model, messages, system_prompt } => {
            println!("Dialog renderer starting...");
            println!("Dialog ID: {}", dialog_id);
            println!("AI Model: {}", ai_model);
            println!("Messages: {} messages", messages.len());
            if let Some(prompt) = system_prompt {
                println!("System prompt: {}", prompt);
            }
            
            // TODO: Implement actual Iced dialog UI
            // For now, this is a placeholder that shows the renderer can be spawned
            println!("Dialog UI window would appear here");
            Ok(())
        }
        _ => {
            println!("Renderer for this data type not yet implemented");
            Ok(())
        }
    }
}