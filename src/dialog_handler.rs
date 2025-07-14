//! Dialog handler for connecting the UI to AI providers

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, error};
use futures::StreamExt;

use crate::{
    dialog::{DialogManager, MessageRole},
    dialog_window::{DialogCommand, DialogEvent, ExportFormat},
    ai::AiManager,
};

/// Handles dialog commands from the UI and manages AI interactions
pub struct DialogHandler {
    dialog_id: String,
    dialog_manager: DialogManager,
    ai_manager: AiManager,
    event_sender: mpsc::Sender<DialogEvent>,
    current_model: String,
}

impl DialogHandler {
    pub fn new(
        dialog_id: String,
        dialog_manager: DialogManager,
        ai_manager: AiManager,
        event_sender: mpsc::Sender<DialogEvent>,
    ) -> Self {
        Self {
            dialog_id,
            dialog_manager,
            ai_manager,
            event_sender,
            current_model: "claude-3-sonnet".to_string(),
        }
    }

    /// Start handling commands from the UI
    pub async fn start(mut self, mut command_receiver: mpsc::Receiver<DialogCommand>) -> Result<()> {
        info!("Starting dialog handler for dialog {}", self.dialog_id);

        while let Some(command) = command_receiver.recv().await {
            match command {
                DialogCommand::SendMessage { content, model } => {
                    self.handle_send_message(content, model).await?;
                }
                DialogCommand::ChangeModel { model } => {
                    self.handle_change_model(model).await?;
                }
                DialogCommand::ExportDialog { format } => {
                    self.handle_export_dialog(format).await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_send_message(&mut self, content: String, model: String) -> Result<()> {
        info!("Sending message to AI: {} chars", content.len());

        // Add user message to dialog history
        self.dialog_manager.add_message(
            &self.dialog_id,
            MessageRole::User,
            content.clone(),
            None,
        ).await?;

        // Get conversation history for context
        let history = self.dialog_manager.get_messages(&self.dialog_id, 10).await?;

        // Build conversation prompt
        let mut prompt = String::new();
        for msg in history {
            let role = match msg.role {
                MessageRole::User => "User",
                MessageRole::Assistant => "Assistant",
                MessageRole::System => "System",
            };
            prompt.push_str(&format!("{}: {}\n\n", role, msg.content));
        }
        prompt.push_str(&format!("User: {}\n\nAssistant:", content));

        // Stream response from AI
        match self.ai_manager.stream_completion(&self.current_model, &prompt).await {
            Ok(mut stream) => {
                let mut full_response = String::new();
                let start_time = std::time::Instant::now();
                let mut token_count = 0;

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(response) => {
                            full_response.push_str(&response.content);
                            token_count += 1;
                            
                            // Send token to UI
                            let _ = self.event_sender.send(DialogEvent::TokenStreamed(response.content)).await;
                        }
                        Err(e) => {
                            error!("Error streaming token: {}", e);
                            let _ = self.event_sender.send(
                                DialogEvent::Error(format!("Streaming error: {}", e))
                            ).await;
                            break;
                        }
                    }
                }

                let duration = start_time.elapsed();
                info!(
                    "Response complete: {} tokens in {:.2}s ({:.1} tokens/sec)",
                    token_count,
                    duration.as_secs_f64(),
                    token_count as f64 / duration.as_secs_f64()
                );

                // Add assistant message to dialog history
                let assistant_msg = self.dialog_manager.add_message(
                    &self.dialog_id,
                    MessageRole::Assistant,
                    full_response,
                    Some(serde_json::json!({
                        "model": model,
                        "tokens": token_count,
                        "duration_ms": duration.as_millis(),
                    })),
                ).await?;

                // Send complete message to UI
                let _ = self.event_sender.send(DialogEvent::MessageAdded(assistant_msg)).await;
                let _ = self.event_sender.send(DialogEvent::ResponseComplete).await;
            }
            Err(e) => {
                error!("Failed to get AI response: {}", e);
                let _ = self.event_sender.send(
                    DialogEvent::Error(format!("Failed to get AI response: {}", e))
                ).await;
            }
        }

        Ok(())
    }

    async fn handle_change_model(&mut self, model: String) -> Result<()> {
        info!("Changing model to: {}", model);
        
        self.current_model = model;

        Ok(())
    }

    async fn handle_export_dialog(&self, format: ExportFormat) -> Result<()> {
        info!("Exporting dialog in format: {:?}", format);

        let messages = self.dialog_manager.get_all_messages(&self.dialog_id).await?;
        let dialog_info = self.dialog_manager.get_dialog(&self.dialog_id).await?;

        let export_content = match format {
            ExportFormat::Markdown => {
                let mut content = format!("# {}\n\n", dialog_info.title);
                content.push_str(&format!("**Created**: {}\n\n", dialog_info.created_at));
                content.push_str("---\n\n");

                for msg in messages {
                    let role = match msg.role {
                        MessageRole::User => "**You**",
                        MessageRole::Assistant => "**AI**",
                        MessageRole::System => "**System**",
                    };
                    content.push_str(&format!("{}: {}\n\n", role, msg.content));
                }

                content
            }
            ExportFormat::Json => {
                serde_json::to_string_pretty(&serde_json::json!({
                    "dialog": dialog_info,
                    "messages": messages,
                }))?
            }
            ExportFormat::Text => {
                let mut content = format!("{}\n", dialog_info.title);
                content.push_str(&format!("Created: {}\n\n", dialog_info.created_at));

                for msg in messages {
                    let role = match msg.role {
                        MessageRole::User => "You",
                        MessageRole::Assistant => "AI",
                        MessageRole::System => "System",
                    };
                    content.push_str(&format!("{}: {}\n\n", role, msg.content));
                }

                content
            }
        };

        // Save to file
        let filename = format!(
            "dialog_{}_{}.{}",
            self.dialog_id,
            chrono::Local::now().format("%Y%m%d_%H%M%S"),
            match format {
                ExportFormat::Markdown => "md",
                ExportFormat::Json => "json",
                ExportFormat::Text => "txt",
            }
        );

        std::fs::write(&filename, export_content)?;
        info!("Dialog exported to: {}", filename);

        Ok(())
    }
}

/// Launch a dialog window with AI backend
pub async fn launch_dialog_window_with_ai(
    dialog_id: String,
    title: String,
    dialog_manager: DialogManager,
    ai_manager: AiManager,
) -> Result<()> {
    let (cmd_tx, cmd_rx) = mpsc::channel(100);
    let (event_tx, event_rx) = mpsc::channel(100);

    // Start the handler
    let handler = DialogHandler::new(
        dialog_id.clone(),
        dialog_manager,
        ai_manager,
        event_tx,
    );

    tokio::spawn(async move {
        if let Err(e) = handler.start(cmd_rx).await {
            error!("Dialog handler error: {}", e);
        }
    });

    // Run the window
    crate::dialog_window::run_dialog_window(dialog_id, title, cmd_tx, event_rx).await
}